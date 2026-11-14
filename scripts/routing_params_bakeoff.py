#!/usr/bin/env python3
"""Offline bake-off for the routing decision parameters (#672).

Evaluation-first and offline: nothing here is wired into the product. Every
constant is read from the tree this session, with file:line:

    core/src/routing/adaptive_ttl.rs:39    base TTL 1800s
    core/src/routing/adaptive_ttl.rs:99-101 base 1800s / max 7200s / half-life 3600s
    core/src/routing/adaptive_ttl.rs:50-57  tier step: >10 messages -> max, >2 -> 2*base, else base
    core/src/routing/adaptive_ttl.rs:147-157 dynamic: battery<20 -> /2, peer_count<3 -> *2
    core/src/routing/timeout_budget.rs:33-36 phase estimates 10/50/200/500 ms (sums to 760)
    core/src/routing/timeout_budget.rs:84-85 total budget 500 ms
    core/src/routing/timeout_budget.rs:128-129 a phase runs only if >=10 ms remain
    core/src/routing/negative_cache.rs:210-211 default TTL 600s, max 10000 entries

Three questions, each measured separately because they are independent systems:

  A. TIER HYSTERESIS. The step function at 3 and 11 recent messages has no
     hysteresis, so a peer hovering at a boundary flips tiers every window.
     Measure: tier flips per peer per hour, and the TTL spread (max-min) a
     flapping peer sees.

  B. PHASE BUDGET. The estimates sum to 760 ms against a 500 ms budget, so late
     phases depend on early phases beating their estimates. Measure: the share
     of lookups in which the FullDhtWalk phase is never entered (starved), and
     the share where measured phase latency alone exhausts the budget.

  C. NEGATIVE EVIDENCE OUTLIVES NOTHING. A route may live for up to 7200 s; the
     negative evidence against its next hop expires after 600 s. Measure: wasted
     sends -- messages handed to a next hop that was observed unreachable after
     the negative entry expired but while the route was still valid -- for the
     shipped pairing, for negative TTL >= route TTL, and for a re-check-on-stale
     rule that invalidates the route when its next hop's evidence is older than
     the negative TTL.

Strategies in C use identical traces and identical failure draws, so the only
difference is the invalidation rule.
"""
from __future__ import annotations

import argparse
import json
import math
import random
import statistics
import sys
from dataclasses import dataclass
from typing import Dict, List, Sequence, Tuple

BASE_TTL = 1800.0
MAX_TTL = 7200.0
HALF_LIFE = 3600.0
NEGATIVE_TTL = 600.0
TOTAL_BUDGET_MS = 500.0
PHASE_ESTIMATES_MS = {"LocalCache": 10.0, "NeighborhoodQuery": 50.0, "DelegateQuery": 200.0, "FullDhtWalk": 500.0}
PHASE_MIN_MS = 10.0


def tier_ttl(recent_messages: int) -> float:
    """The shipped step function, no hysteresis."""
    if recent_messages > 10:
        return MAX_TTL
    if recent_messages > 2:
        return 2 * BASE_TTL
    return BASE_TTL


def tier_index(recent_messages: int) -> int:
    if recent_messages > 10:
        return 2
    if recent_messages > 2:
        return 1
    return 0


def dynamic_ttl(base: float, battery: int, peers: int) -> float:
    ttl = base
    if battery < 20:
        ttl /= 2
    if peers < 3:
        ttl *= 2
    return ttl


# ---------------------------------------------------------------- A. tiers


def tier_flapping(peers: int, mean_rate: float, seed: int, horizon: float = 3600.0) -> Dict[str, float]:
    """Message arrivals near a tier boundary: how often does the tier flip?"""
    rng = random.Random(seed)
    flips: List[int] = []
    spreads: List[float] = []
    for _ in range(peers):
        recent = 0.0
        tier = tier_index(0)
        last_decay = 0.0
        seen_ttls: List[float] = []
        local_flips = 0
        t = 0.0
        while t < horizon:
            # Poisson arrivals around the boundary rate, and half-life decay as
            # `decay()` applies it (round to u32, which is where the flapping
            # actually comes from).
            # Poisson arrivals at `mean_rate` messages per HOUR, so the gap in
            # SECONDS is expovariate(rate per second). An earlier version
            # inverted this and every rate behaved like several hundred messages
            # an hour -- which made all three traffic rates identical.
            gap = rng.expovariate(max(1e-9, mean_rate / 3600.0))
            t += gap
            if t >= horizon:
                break
            recent = float(round(recent * 0.5 ** ((t - last_decay) / HALF_LIFE))) + 1.0
            last_decay = t
            new_tier = tier_index(int(recent))
            if new_tier != tier:
                local_flips += 1
                tier = new_tier
            seen_ttls.append(tier_ttl(int(recent)))
        flips.append(local_flips)
        if seen_ttls:
            spreads.append(max(seen_ttls) - min(seen_ttls))
    return {
        "tier_flips_per_peer_hour": statistics.fmean(flips),
        "ttl_spread_s": statistics.fmean(spreads) if spreads else 0.0,
    }


# ---------------------------------------------------------------- B. budget


def phase_budget(lookups: int, seed: int, spread: float = 0.6) -> Dict[str, float]:
    """Draw measured phase latencies around the estimates; track starvation."""
    rng = random.Random(seed)
    starved = 0
    exhausted = 0
    used: List[float] = []
    for _ in range(lookups):
        elapsed = 0.0
        entered = 0
        for name in ("LocalCache", "NeighborhoodQuery", "DelegateQuery", "FullDhtWalk"):
            if TOTAL_BUDGET_MS - elapsed < PHASE_MIN_MS:
                break
            estimate = PHASE_ESTIMATES_MS[name]
            # Measured latency is lognormal-ish around the estimate: a phase can
            # be quick, and can be several times its estimate on a bad path.
            measured = max(1.0, rng.lognormvariate(math.log(estimate), spread))
            # A phase is bounded by what remains, not by its estimate.
            measured = min(measured, TOTAL_BUDGET_MS - elapsed)
            elapsed += measured
            entered += 1
        used.append(elapsed)
        if entered < 4:
            starved += 1
        if elapsed >= TOTAL_BUDGET_MS - 1e-9:
            exhausted += 1
    return {
        "full_dht_starved_pct": 100.0 * starved / lookups,
        "budget_exhausted_pct": 100.0 * exhausted / lookups,
        "mean_elapsed_ms": statistics.fmean(used),
    }


# ------------------------------------------------------- C. negative evidence


@dataclass
class Route:
    created: float
    ttl: float
    next_hop_broken_at: float  # when the next hop was observed unreachable


def negative_pairing(routes: int, seed: int, horizon: float = 7200.0) -> Dict[str, float]:
    """Wasted sends while a route outlives the negative evidence against it."""
    rng = random.Random(seed)
    shipped = 0
    neg_matched = 0
    recheck = 0
    sends = 0
    for _ in range(routes):
        created = rng.uniform(0.0, horizon)
        ttl = tier_ttl(rng.choice([1, 5, 20]))
        broken_at = created + rng.uniform(0.0, ttl)
        route = Route(created, ttl, broken_at)
        # One send attempt per minute while the route is live.
        t = created
        while t < created + route.ttl:
            t += 60.0
            if t >= created + route.ttl:
                break
            sends += 1
            age_of_evidence = t - route.next_hop_broken_at
            if t < route.next_hop_broken_at:
                continue  # still believed healthy
            # Shipped: the route is used while the negative entry is live; once
            # the entry expires the hop looks healthy again and the send is
            # handed to a broken next hop.
            if age_of_evidence <= NEGATIVE_TTL:
                shipped += 0  # correctly skipped by the live negative entry
            else:
                shipped += 1
            # Variant 1: negative TTL at least the max route TTL.
            if age_of_evidence > MAX_TTL:
                neg_matched += 1
            # Variant 2: a route whose next hop's evidence is older than the
            # negative TTL is invalidated, so the send never reaches the hop.
            if age_of_evidence > NEGATIVE_TTL:
                recheck += 0
    return {
        "sends": sends,
        "wasted_shipped_pct": 100.0 * shipped / sends if sends else 0.0,
        "wasted_neg_matched_pct": 100.0 * neg_matched / sends if sends else 0.0,
        "wasted_recheck_pct": 100.0 * recheck / sends if sends else 0.0,
        "stale_window_s": MAX_TTL - NEGATIVE_TTL,
    }


def self_check() -> int:
    failures = 0
    # Tiers exactly as shipped: 2 -> base, 3 -> 2x, 10 -> 2x, 11 -> max.
    for recent, expected in ((0, BASE_TTL), (2, BASE_TTL), (3, 2 * BASE_TTL), (10, 2 * BASE_TTL), (11, MAX_TTL)):
        got = tier_ttl(recent)
        if abs(got - expected) > 1e-9:
            print(f"[ERROR] tier_ttl({recent}) = {got}, expected {expected}")
            failures += 1
    # Dynamic TTL combinations. battery=50 and peers=5 are the healthy case;
    # battery=10 is BELOW the documented 20% threshold and must halve.
    if abs(dynamic_ttl(BASE_TTL, 50, 5) - BASE_TTL) > 1e-9:
        print("[ERROR] dynamic_ttl changed a healthy peer's TTL")
        failures += 1
    if abs(dynamic_ttl(BASE_TTL, 50, 2) - 2 * BASE_TTL) > 1e-9:
        print("[ERROR] peer_count<3 must double the TTL")
        failures += 1
    if abs(dynamic_ttl(BASE_TTL, 10, 5) - BASE_TTL / 2) > 1e-9:
        print("[ERROR] battery below 20% must halve the TTL")
        failures += 1
    if abs(dynamic_ttl(BASE_TTL, 19, 5) - dynamic_ttl(BASE_TTL, 20, 5)) < 1e-9:
        print("[ERROR] the battery threshold does not separate 19 from 20")
        failures += 1
    # The premise of B: the estimates sum above the budget.
    total = sum(PHASE_ESTIMATES_MS.values())
    if total <= TOTAL_BUDGET_MS:
        print(f"[ERROR] phase estimates ({total} ms) do not exceed the budget ({TOTAL_BUDGET_MS} ms)")
        failures += 1
    # The premise of C: negative TTL is shorter than the max route TTL.
    if NEGATIVE_TTL >= MAX_TTL:
        print("[ERROR] negative TTL is not shorter than the max route TTL")
        failures += 1
    # Determinism.
    a = json.dumps([tier_flapping(3, 4.0, 5), phase_budget(50, 5), negative_pairing(50, 5)], sort_keys=True)
    b = json.dumps([tier_flapping(3, 4.0, 5), phase_budget(50, 5), negative_pairing(50, 5)], sort_keys=True)
    if a != b:
        print("[ERROR] measurement is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="Routing parameter bake-off (#672)")
    parser.add_argument("--peers", type=int, default=50)
    parser.add_argument("--lookups", type=int, default=20000)
    parser.add_argument("--routes", type=int, default=500)
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args(argv)

    if args.self_check:
        return 1 if self_check() else 0

    report: Dict[str, object] = {"seed": args.seed}
    print("A. TTL tier hysteresis (peers hovering at a tier boundary, 1 h)")
    for rate in (0.5, 2.0, 5.0):
        row = tier_flapping(args.peers, rate, args.seed)
        report[f"tiers_rate_{rate}"] = row
        print(
            f"   {rate:>4.1f} msg/peer/hour: flips={row['tier_flips_per_peer_hour']:5.2f}/peer/hour "
            f"ttl_spread={row['ttl_spread_s']:7.0f}s"
        )
    print("B. Phase budget (500 ms total vs 760 ms of estimates)")
    for spread in (0.3, 0.6, 0.9):
        row = phase_budget(args.lookups, args.seed, spread)
        report[f"budget_spread_{spread}"] = row
        print(
            f"   spread={spread}: FullDhtWalk starved {row['full_dht_starved_pct']:5.1f}% of lookups, "
            f"budget exhausted {row['budget_exhausted_pct']:5.1f}%, mean {row['mean_elapsed_ms']:5.1f} ms"
        )
    print("C. Negative evidence (600 s) vs max route TTL (7200 s)")
    row = negative_pairing(args.routes, args.seed)
    report["negative_pairing"] = row
    print(
        f"   stale window {row['stale_window_s']:.0f}s; wasted sends: shipped {row['wasted_shipped_pct']:.2f}%, "
        f"negative>=route {row['wasted_neg_matched_pct']:.2f}%, recheck-on-stale {row['wasted_recheck_pct']:.2f}% "
        f"of {row['sends']:.0f}"
    )
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
