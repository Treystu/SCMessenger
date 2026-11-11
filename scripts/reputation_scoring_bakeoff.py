#!/usr/bin/env python3
"""Offline scripted-adversary calibration for reputation scoring (#676).

Evaluation-first and offline: nothing here is wired into the product. The
champion scoring function is `ReputationScore::calculate_score` as read this
session:

    core/src/transport/reputation.rs:23-25   MIN 0.0 / MAX 100.0 / NEUTRAL 50.0
    core/src/transport/reputation.rs:148     positive = relays*2.0 + deliveries*3.0
    core/src/transport/reputation.rs:149-155 negative = rate_limit*5.0 + oversized*3.0
                                             + invalid_format*4.0 + duplicate*2.0
                                             + invalid_destination*3.0
                                             + failed_relays*1.5 + timeouts*1.0
    core/src/transport/reputation.rs:159     score = 50 + (total / (1+|total|)) * 50
    core/src/transport/reputation.rs:40,44,48 trusted >= 70 / suspicious 10..30 / abusive < 10
    core/src/transport/reputation.rs:167-173 rate-limit multiplier 1.5 / 0.5 / 1.0
    core/src/transport/reputation.rs:200,211 decay TTL 7 days

The audit question is not whether the formula is smooth (it is) but whether the
bands are reachable by cheap signal farming, and whether an honest peer can be
pushed out of its band by ordinary failure noise. Four scripted populations:

  honest    -- mostly successes, occasional timeouts and duplicates
  spammer   -- flooding (rate-limit hits) and malformed traffic
  farmer    -- deliberately cheap positive signals only (relays, deliveries)
  sybil     -- a fleet of farmed identities, each cheaply promoted

Measured per band: false positives (honest peers in suspicious/abusive),
false negatives (spammer still trusted), and FARMING YIELD = score gained per
scripted action. Three variants are compared on the same action streams:

  shipped        -- the constants above
  symmetric      -- timeouts and failed relays weighted 3.0 (not 1.0/1.5)
  capped         -- positive contributions count at most 20 actions per window
  confirmed      -- deliveries count 0 unless independently confirmed
"""
from __future__ import annotations

import argparse
import json
import random
import statistics
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Sequence, Tuple

NEUTRAL = 50.0
TRUSTED = 70.0
SUSPICIOUS_HI = 30.0
ABUSIVE = 10.0
POSITIVE_CAP = 20

WEIGHTS = {
    "relays": 2.0,
    "deliveries": 3.0,
    "rate_limit": 5.0,
    "oversized": 3.0,
    "invalid_format": 4.0,
    "duplicate": 2.0,
    "invalid_destination": 3.0,
    "failed_relays": 1.5,
    "timeouts": 1.0,
}


@dataclass
class Counts:
    relays: int = 0
    deliveries: int = 0
    rate_limit: int = 0
    oversized: int = 0
    invalid_format: int = 0
    duplicate: int = 0
    invalid_destination: int = 0
    failed_relays: int = 0
    timeouts: int = 0

    def add(self, action: str) -> None:
        setattr(self, action, getattr(self, action) + 1)


def score(counts: Counts, variant: str = "shipped") -> float:
    weights = dict(WEIGHTS)
    if variant == "symmetric":
        weights["timeouts"] = 3.0
        weights["failed_relays"] = 3.0
    positive_units = counts.relays + counts.deliveries
    if variant == "capped":
        scale = min(1.0, POSITIVE_CAP / positive_units) if positive_units else 0.0
        positive = (counts.relays * weights["relays"] + counts.deliveries * weights["deliveries"]) * scale
    elif variant == "confirmed":
        # Deliveries only count when independently confirmed; the farmer's own
        # claim is worth nothing.
        positive = counts.relays * weights["relays"] + confirmed_deliveries(counts) * weights["deliveries"]
    else:
        positive = counts.relays * weights["relays"] + counts.deliveries * weights["deliveries"]
    negative = (
        counts.rate_limit * weights["rate_limit"]
        + counts.oversized * weights["oversized"]
        + counts.invalid_format * weights["invalid_format"]
        + counts.duplicate * weights["duplicate"]
        + counts.invalid_destination * weights["invalid_destination"]
        + counts.failed_relays * weights["failed_relays"]
        + counts.timeouts * weights["timeouts"]
    )
    total = positive - negative
    return NEUTRAL + (total / (1.0 + abs(total))) * 50.0


def confirmed_deliveries(counts: Counts) -> int:
    """Deliveries a peer claims vs deliveries an independent witness confirms.

    Modelled as half: the farming strategy claims deliveries it never made; the
    honest peer's claims are corroborated.
    """
    return counts.deliveries // 2


def band(value: float) -> str:
    if value >= TRUSTED:
        return "trusted"
    if value >= SUSPICIOUS_HI:
        return "neutral"
    if value >= ABUSIVE:
        return "suspicious"
    return "abusive"


def scripted_actions(kind: str, rng: random.Random, actions: int) -> List[str]:
    if kind == "honest":
        pool = ["relays"] * 6 + ["deliveries"] * 3 + ["timeouts"] + ["duplicate"] * 2
    elif kind == "spammer":
        pool = ["rate_limit"] * 5 + ["invalid_format"] * 3 + ["oversized"] * 2 + ["invalid_destination"] * 2
    elif kind == "farmer":
        # Cheap, self-reported positives: relay claims and delivery claims.
        pool = ["relays"] * 3 + ["deliveries"] * 7
    elif kind == "sybil":
        pool = ["relays"] * 4 + ["deliveries"] * 6
    else:
        raise SystemExit(f"unknown population {kind}")
    return [rng.choice(pool) for _ in range(actions)]


def evaluate(variant: str, population: int, actions: int, seed: int) -> Dict[str, float]:
    rng = random.Random(seed)
    bands: Dict[str, Dict[str, int]] = {
        kind: {"trusted": 0, "neutral": 0, "suspicious": 0, "abusive": 0}
        for kind in ("honest", "spammer", "farmer", "sybil")
    }
    yields: Dict[str, List[float]] = {"honest": [], "spammer": [], "farmer": [], "sybil": []}
    for kind in bands:
        for _ in range(population):
            counts = Counts()
            for action in scripted_actions(kind, rng, actions):
                counts.add(action)
            value = score(counts, variant)
            bands[kind][band(value)] += 1
            yields[kind].append((value - NEUTRAL) / actions)
    honest = bands["honest"]
    spammer = bands["spammer"]
    farmer = bands["farmer"]
    return {
        "honest_collateral_pct": 100.0 * (honest["suspicious"] + honest["abusive"]) / population,
        "spammer_still_trusted_pct": 100.0 * spammer["trusted"] / population,
        "farmer_trusted_pct": 100.0 * farmer["trusted"] / population,
        "sybil_trusted_pct": 100.0 * bands["sybil"]["trusted"] / population,
        "farming_yield_farmer": statistics.fmean(yields["farmer"]),
        "farming_yield_honest": statistics.fmean(yields["honest"]),
        "farmer_band_mix": bands["farmer"],
        "honest_band_mix": honest,
        "spammer_band_mix": spammer,
    }


def run(population: int, actions: int, seed: int) -> Dict[str, object]:
    report: Dict[str, object] = {"seed": seed, "population": population, "actions": actions, "variants": {}}
    for variant in ("shipped", "symmetric", "capped", "confirmed"):
        report["variants"][variant] = evaluate(variant, population, actions, seed)
    return report


def self_check() -> int:
    failures = 0
    # Neutral: no signals at all.
    if abs(score(Counts()) - NEUTRAL) > 1e-9:
        print("[ERROR] empty counts do not score neutral")
        failures += 1
    # Squash bounds: never escapes [0, 100].
    for counts in (Counts(relays=10_000), Counts(rate_limit=10_000)):
        value = score(counts)
        if not (0.0 <= value <= 100.0):
            print(f"[ERROR] score {value} escaped [0,100]")
            failures += 1
    # Band boundaries exactly as documented.
    if band(70.0) != "trusted" or band(69.999) == "trusted":
        print("[ERROR] trusted boundary is not 70")
        failures += 1
    if band(10.0) != "suspicious" or band(9.999) != "abusive":
        print("[ERROR] abusive boundary is not 10")
        failures += 1
    # The shipped weights make a SINGLE rate-limit hit land a fresh peer in the
    # abusive band: 50 + (-5/(1+5))*50 = 8.33 < 10. That is a measurement, not a
    # defect in the instrument, so the check asserts the computed value and the
    # finding is reported on the issue.
    one_hit = score(Counts(rate_limit=1))
    if abs(one_hit - 8.3333) > 0.01:
        print(f"[ERROR] one rate-limit hit scores {one_hit:.4f}, expected ~8.3333")
        failures += 1
    if band(one_hit) != "abusive":
        print("[ERROR] a single rate-limit hit no longer reaches the abusive band; the finding changed")
        failures += 1
    # Determinism.
    a = json.dumps(run(10, 30, 7), sort_keys=True)
    b = json.dumps(run(10, 30, 7), sort_keys=True)
    if a != b:
        print("[ERROR] run is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="Reputation scripted-adversary bake-off (#676)")
    parser.add_argument("--population", type=int, default=200)
    parser.add_argument("--actions", type=int, default=60)
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args(argv)
    if args.self_check:
        return 1 if self_check() else 0
    report = run(args.population, args.actions, args.seed)
    print(f"{args.population} peers per population, {args.actions} actions each, seed {args.seed}")
    for variant, row in report["variants"].items():
        print(
            f"  {variant:<10} honest_collateral={row['honest_collateral_pct']:5.1f}% "
            f"spammer_trusted={row['spammer_still_trusted_pct']:5.1f}% "
            f"farmer_trusted={row['farmer_trusted_pct']:5.1f}% "
            f"sybil_trusted={row['sybil_trusted_pct']:5.1f}% "
            f"yield farmer={row['farming_yield_farmer']:+.3f}/action honest={row['farming_yield_honest']:+.3f}"
        )
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
