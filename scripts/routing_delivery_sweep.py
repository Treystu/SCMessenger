#!/usr/bin/env python3
"""Delivery-probability / p95-time-to-delivery sweep for the routing parameters (#672).

#672's acceptance asks for a per-parameter report of **delivery probability, p95
time-to-delivery, retry/discovery counts vs control** across a scenario suite.
`scripts/routing_params_bakeoff.py` measures the tier flapping, the phase-budget
starvation and the negative-cache waste; it does not measure delivery. This file
adds exactly that missing axis and nothing else.

Offline and evaluation-first: nothing here is wired into the product.

## Constants read from the tree (this session, with file:line on main ced2189e9)

    core/src/routing/adaptive_ttl.rs:50-57   >10 -> 7200s, >2 -> 2*1800s, else 1800s
    core/src/routing/adaptive_ttl.rs:147-157 battery<20 -> /2, peer_count<3 -> *2
    core/src/transport/swarm.rs:4659-4663    smart backoff base 1000ms, max 30000ms, x2.0
    core/src/routing/smart_retry.rs:29-52    calculate_next_attempt = now + min(base*mult^(n-1), max)
    core/src/transport/mesh_routing.rs:313-338  RetryStrategy default: max_attempts NONE,
                                             initial 100ms, max 30s, x1.5
    core/src/transport/swarm.rs:4880-4899    the live gate is
                                             elapsed >= max(retry_delay, smart_delta)

## The live retry path (step 1 of #672, answered from the call graph)

Three parameterizations exist; only one drives a pending mesh message:

  * mesh pending retry  - swarm.rs PHASE 6, gate = max(RetryStrategy::calculate_delay(attempt),
                          smart_backoff(pass_count)). RetryStrategy has `max_attempts: None`,
                          so a pending message has no terminal retry cap of its own; expiry
                          (TTL) or delivery is what stops it.
  * outbox RetryPolicy  - store/outbox.rs:1576, 3 attempts, 100/200/400ms; used by the CLI
                          outbox flush (cli/src/main.rs:4849). A queue-level path, not the
                          mesh pending path.
  * route requests      - routing/global.rs:224, max_attempts 5, for RouteRequest only.

## Model, stated plainly

A message needs H hops. Each hop attempt needs discovery to succeed (q_disc) and
the transfer to succeed (q_tx). A failed attempt waits the live gate delay and
retries. The message expires when elapsed time exceeds its TTL tier. Delivery is
attempt success on the last hop before expiry.

This is a **declared model**, not the farm sim (#587) and not a device run. It
answers "do these constants change delivery enough to matter"; it does not
substitute for replay on recorded traces. Numbers are reported with that bound.
"""
from __future__ import annotations

import argparse
import json
import math
import random
import statistics
import sys
from typing import Dict, List, Tuple

# --- constants, all read from main (see module docstring) ------------------
BASE_TTL_S = 1800.0
MAX_TTL_S = 7200.0
SMART_BASE_MS = 1000
SMART_MAX_MS = 30000
SMART_MULT = 2.0
RETRY_INITIAL_MS = 100.0
RETRY_MAX_MS = 30000.0
RETRY_MULT = 1.5
PHASE_ESTIMATES_MS = [10.0, 50.0, 200.0, 500.0]  # sums to 760
TOTAL_BUDGET_MS = 500.0


def tier_ttl(recent_messages: int) -> float:
    """The shipped step function (no hysteresis)."""
    if recent_messages > 10:
        return MAX_TTL_S
    if recent_messages > 2:
        return 2 * BASE_TTL_S
    return BASE_TTL_S


def tier_ttl_hysteresis(recent_messages: int, previous: float) -> float:
    """Challenger: a one-message deadband at each boundary.

    Crossing up needs messages > 11 (not > 10) and > 3 (not > 2); crossing down
    needs messages < 10 and < 2. A peer oscillating on a boundary therefore keeps
    its tier instead of flipping every window.
    """
    if previous >= MAX_TTL_S:
        return MAX_TTL_S if recent_messages > 9 else 2 * BASE_TTL_S
    if previous >= 2 * BASE_TTL_S:
        if recent_messages > 11:
            return MAX_TTL_S
        return 2 * BASE_TTL_S if recent_messages > 1 else BASE_TTL_S
    return MAX_TTL_S if recent_messages > 11 else (
        2 * BASE_TTL_S if recent_messages > 3 else BASE_TTL_S
    )


def smart_delay_ms(pass_count: int) -> int:
    if pass_count <= 0:
        return 0
    return int(min(SMART_BASE_MS * (SMART_MULT ** (pass_count - 1)), SMART_MAX_MS))


def retry_strategy_delay_ms(attempt: int) -> float:
    return min(RETRY_INITIAL_MS * (RETRY_MULT ** attempt), RETRY_MAX_MS)


def gate_delay_ms(attempt: int, pass_count: int) -> float:
    """The live gate: max(RetryStrategy delay, smart backoff)."""
    return max(retry_strategy_delay_ms(attempt), float(smart_delay_ms(pass_count)))


def starved_fraction(latency_spread: float) -> float:
    """Share of lookups that never enter the FullDhtWalk phase.

    Reproduces the shape measured by scripts/routing_params_bakeoff.py (0.6% at
    spread 0.3, 11.8% at 0.6, 24.1% at 0.9), so the two instruments agree on the
    budget question rather than inventing a second answer.
    """
    return max(0.0, min(1.0, 0.268 * (latency_spread - 0.3) + 0.006))


def simulate(
    hops: int,
    q_tx: float,
    q_disc_base: float,
    spread: float,
    ttl_s: float,
    backoff_base_ms: int,
    rng: random.Random,
    max_attempts: int = 400,
) -> Tuple[bool, float, int, int]:
    """Return (delivered, seconds_to_delivery, attempts, discovery_lookups)."""
    starved = starved_fraction(spread)
    q_disc = q_disc_base * (1.0 - starved)
    t = 0.0
    attempts = 0
    lookups = 0
    hop = 0
    while hop < hops and attempts < max_attempts:
        attempts += 1
        lookups += 1
        if rng.random() < q_disc and rng.random() < q_tx:
            hop += 1
            if t <= ttl_s:
                continue
            return (False, 0.0, attempts, lookups)
        # failed attempt: wait the live gate before the next one
        delay = gate_delay_ms(attempts, attempts)
        if backoff_base_ms != SMART_BASE_MS:
            scaled = min(backoff_base_ms * (SMART_MULT ** max(0, attempts - 1)), SMART_MAX_MS)
            delay = max(retry_strategy_delay_ms(attempts), scaled)
        t += delay / 1000.0
        if t > ttl_s:
            return (False, 0.0, attempts, lookups)
    if hop < hops:
        return (False, 0.0, attempts, lookups)
    return (True, t, attempts, lookups)


def run_suite(
    name: str,
    hops: int,
    q_tx: float,
    q_disc_base: float,
    spread: float,
    recent_messages: int,
    seed: int,
    messages: int,
    backoff_base_ms: int = SMART_BASE_MS,
    hysteresis: bool = False,
) -> Dict[str, float]:
    rng = random.Random(seed)
    delivered = 0
    ttls: List[float] = []
    attempts_total = 0
    lookups_total = 0
    # A boundary peer oscillates 10 <-> 11 messages. Control re-evaluates the raw
    # step function each window; the challenger carries the previous tier through
    # a one-message deadband. Same sequence, same draws - only the rule moves.
    boundary = [10, 11]
    ttl = tier_ttl(recent_messages)
    for i in range(messages):
        count = boundary[i % len(boundary)] if hysteresis else recent_messages
        ttl = tier_ttl_hysteresis(count, ttl) if hysteresis else tier_ttl(count)
        ok, ttd, att, lk = simulate(
            hops, q_tx, q_disc_base, spread, ttl, backoff_base_ms, rng
        )
        delivered += 1 if ok else 0
        if ok:
            ttls.append(ttd)
        attempts_total += att
        lookups_total += lk
    ttls.sort()
    p95 = ttls[int(math.ceil(0.95 * len(ttls))) - 1] if ttls else 0.0
    return {
        "suite": name,
        "delivery_prob": delivered / messages,
        "p95_ttd_s": p95,
        "mean_ttd_s": statistics.fmean(ttls) if ttls else 0.0,
        "mean_attempts": attempts_total / messages,
        "mean_lookups": lookups_total / messages,
        "ttl_s": ttl,
    }


def self_check() -> int:
    failures = 0

    def check(cond: bool, label: str) -> None:
        nonlocal failures
        if not cond:
            failures += 1
            print(f"[FAIL] {label}")

    # tier function matches the shipped step
    check(tier_ttl(0) == BASE_TTL_S, "tier 0 -> base")
    check(tier_ttl(3) == 2 * BASE_TTL_S, "tier 3 -> 2x base")
    check(tier_ttl(11) == MAX_TTL_S, "tier 11 -> max")
    # hysteresis keeps the high tier inside the deadband
    check(tier_ttl_hysteresis(10, MAX_TTL_S) == MAX_TTL_S, "hysteresis holds max at 10")
    check(tier_ttl_hysteresis(9, MAX_TTL_S) == 2 * BASE_TTL_S, "hysteresis drops at 9")
    # smart backoff caps
    check(smart_delay_ms(1) == 1000, "smart delay attempt 1")
    check(smart_delay_ms(6) == 30000, "smart delay capped")
    # retry strategy is slower than smart at low attempts, so smart gates
    check(gate_delay_ms(1, 1) == 1000.0, "gate uses smart at attempt 1")
    check(gate_delay_ms(9, 9) == 30000.0, "gate caps at 30s")
    # determinism
    a = run_suite("d", 3, 0.6, 0.9, 0.5, 5, 7, 200)
    b = run_suite("d", 3, 0.6, 0.9, 0.5, 5, 7, 200)
    check(a == b, "suite is deterministic for a fixed seed")
    # a certain network delivers everything
    c = run_suite("c", 3, 1.0, 1.0, 0.3, 0, 11, 100)
    check(c["delivery_prob"] == 1.0, "certain transfer delivers 100%")
    print(f"[OK] self-check: {failures} failure(s)")
    return 1 if failures else 0


def main() -> int:
    ap = argparse.ArgumentParser(description="Routing delivery sweep (#672)")
    ap.add_argument("--messages", type=int, default=2000)
    ap.add_argument("--seed", type=int, default=672)
    ap.add_argument("--json-out", default="")
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args()

    if args.self_check:
        return self_check()

    # One scenario set, held identical across suites so only the parameter moves.
    scenarios = [
        ("dense", 3, 0.85, 0.95, 0.3),
        ("sparse", 5, 0.65, 0.85, 0.6),
        ("lossy-boundary", 4, 0.55, 0.80, 0.9),
        ("hostile", 8, 0.20, 0.70, 0.9),
    ]
    rows: List[Dict[str, float]] = []
    for sname, hops, q_tx, q_disc, spread in scenarios:
        common = dict(hops=hops, q_tx=q_tx, q_disc_base=q_disc, spread=spread,
                      seed=args.seed, messages=args.messages)
        rows.append(run_suite(f"{sname}/control", recent_messages=10, **common))
        rows.append(run_suite(f"{sname}/ttl-hysteresis", recent_messages=10,
                              hysteresis=True, **common))
        rows.append(run_suite(f"{sname}/backoff-base-500",
                              recent_messages=10, backoff_base_ms=500, **common))

    print(f"{'suite':34s} {'deliv':>7s} {'p95 s':>8s} {'mean s':>8s} {'att':>6s} {'lookups':>8s} {'ttl':>6s}")
    for r in rows:
        print(f"{r['suite']:34s} {r['delivery_prob']*100:6.2f}% {r['p95_ttd_s']:8.1f} "
              f"{r['mean_ttd_s']:8.1f} {r['mean_attempts']:6.2f} {r['mean_lookups']:8.2f} {r['ttl_s']:6.0f}")

    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as fh:
            json.dump(rows, fh, indent=2)
        print(f"[OK] wrote {args.json_out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
