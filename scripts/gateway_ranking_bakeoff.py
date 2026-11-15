#!/usr/bin/env python3
"""Offline gateway-ranking bake-off for #669.

Evaluation-first and offline: nothing here is wired into the product. The
champion is `NeighborhoodTable::gateway_cost` as read this session:

    core/src/routing/neighborhood.rs:265-274
        cost = hops_away * energy_multiplier / max(avg_reliability, 0.01)
    core/src/routing/neighborhood.rs:35-38
        Charging 0.5 / High 1.0 / Low 2.0 / Critical 8.0
    core/src/routing/neighborhood.rs:166-168
        max_staleness 3600, max_gateways 100, max_hops 4

The oracle is the gateway's true end-to-end delivery probability, which the
estimator supplies in this simulation as a noisy function of hops and the
gateway's own reliability. The audit measures what the ranking costs:

  regret        -- true delivery of the chosen gateway minus the best available
  inversion     -- chosen gateway was not the oracle's first choice
  spearman      -- rank correlation of cost order vs oracle order

Scenario classes are separated because the hypothesis is specific: the cost is
separably first-hop shaped, so inversions should concentrate where energy
classes differ across candidates while hops and reliability are comparable.

Also swept: the energy multipliers (including a flat no-energy variant) and the
0.01 reliability floor, to see whether the losses are the energy term's or the
floor's -- and whether any variant reduces regret without regressing a class.
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

SHIPPED_MULTIPLIERS = {"Charging": 0.5, "High": 1.0, "Low": 2.0, "Critical": 8.0}
ENERGY_CLASSES = list(SHIPPED_MULTIPLIERS)

VARIANTS: Dict[str, Dict[str, float]] = {
    "shipped (0.5/1/2/8)": dict(SHIPPED_MULTIPLIERS),
    "softened (0.75/1/1.5/3)": {"Charging": 0.75, "High": 1.0, "Low": 1.5, "Critical": 3.0},
    "energy-blind (1/1/1/1)": {cls: 1.0 for cls in ENERGY_CLASSES},
    "energy-logarithmic (0.5/1/2/4)": {"Charging": 0.5, "High": 1.0, "Low": 2.0, "Critical": 4.0},
}


@dataclass
class Gateway:
    hops: int
    energy: str
    avg_reliability: float
    true_delivery: float


def make_gateway(rng: random.Random, class_name: str) -> Gateway:
    """A candidate with a true delivery probability the cost function cannot see."""
    hops = rng.randint(1, 4)
    if class_name == "uniform-energy":
        energy = "High"
    elif class_name == "mixed-energy":
        energy = rng.choice(ENERGY_CLASSES)
    elif class_name == "critical-heavy":
        energy = rng.choice(["Critical", "Critical", "Low", "High"])
    else:
        raise SystemExit(f"unknown scenario class {class_name}")
    # The advertised cell average, and the path's true delivery. The difference
    # between them is the whole audit: avg_reliability is a CELL average, so a
    # strong cell with a weak first hop can advertise high while delivering low.
    avg_reliability = rng.uniform(0.35, 0.98)
    hop_penalty = 0.94 ** (hops - 1)
    true_delivery = min(
        0.99,
        max(
            0.05,
            avg_reliability * hop_penalty * rng.uniform(0.75, 1.15)
            + (0.05 if energy == "Charging" else 0.0),
        ),
    )
    return Gateway(hops=hops, energy=energy, avg_reliability=avg_reliability, true_delivery=true_delivery)


def cost(gateway: Gateway, multipliers: Dict[str, float], floor: float) -> float:
    return gateway.hops * multipliers[gateway.energy] / max(gateway.avg_reliability, floor)


def ranks(values: Sequence[float]) -> List[float]:
    order = sorted(range(len(values)), key=lambda i: values[i])
    out = [0.0] * len(values)
    for position, index in enumerate(order):
        out[index] = float(position)
    return out


def spearman(a: Sequence[float], b: Sequence[float]) -> float:
    if len(a) < 3:
        return 0.0
    ra, rb = ranks(a), ranks(b)
    ma, mb = statistics.fmean(ra), statistics.fmean(rb)
    num = sum((x - ma) * (y - mb) for x, y in zip(ra, rb))
    den = math.sqrt(sum((x - ma) ** 2 for x in ra) * sum((y - mb) ** 2 for y in rb))
    return num / den if den else 0.0


def evaluate_class(
    class_name: str,
    topologies: int,
    pool_min: int,
    pool_max: int,
    seed: int,
    multipliers: Dict[str, float],
    floor: float,
) -> Dict[str, float]:
    rng = random.Random(f"{seed}-{class_name}".__hash__() & 0xFFFFFFFF)
    regrets: List[float] = []
    inversions = 0
    total = 0
    correlations: List[float] = []
    for _ in range(topologies):
        size = rng.randint(pool_min, pool_max)
        pool = [make_gateway(rng, class_name) for _ in range(size)]
        costs = [cost(g, multipliers, floor) for g in pool]
        chosen = min(range(size), key=lambda i: costs[i])
        best = max(range(size), key=lambda i: pool[i].true_delivery)
        regrets.append(pool[best].true_delivery - pool[chosen].true_delivery)
        inversions += int(chosen != best)
        total += 1
        correlations.append(spearman(costs, [-g.true_delivery for g in pool]))
    return {
        "mean_regret": statistics.fmean(regrets),
        "p90_regret": sorted(regrets)[int(0.9 * (len(regrets) - 1))],
        "inversion_rate": inversions / total,
        "spearman_cost_vs_delivery": statistics.fmean(correlations),
    }


def run(seed: int, topologies: int, pool_min: int, pool_max: int, floor: float) -> Dict[str, object]:
    classes = ["uniform-energy", "mixed-energy", "critical-heavy"]
    report: Dict[str, object] = {"seed": seed, "topologies_per_class": topologies, "floor": floor, "variants": []}
    for name, multipliers in VARIANTS.items():
        rows = {
            cls: evaluate_class(cls, topologies, pool_min, pool_max, seed, multipliers, floor)
            for cls in classes
        }
        report["variants"].append({"variant": name, "multipliers": multipliers, "classes": rows})
    return report


def self_check() -> int:
    failures = 0
    # The shipped cost must match the documented formula exactly.
    gw = Gateway(hops=3, energy="Low", avg_reliability=0.5, true_delivery=0.5)
    expected = 3 * 2.0 / 0.5
    if abs(cost(gw, SHIPPED_MULTIPLIERS, 0.01) - expected) > 1e-9:
        print(f"[ERROR] shipped cost formula mismatch: {cost(gw, SHIPPED_MULTIPLIERS, 0.01)} != {expected}")
        failures += 1
    # The floor must bound an absurdly low advertised reliability.
    tiny = Gateway(hops=1, energy="High", avg_reliability=0.0, true_delivery=0.5)
    if cost(tiny, SHIPPED_MULTIPLIERS, 0.01) != 1 / 0.01:
        print("[ERROR] reliability floor is not applied as max(rel, floor)")
        failures += 1
    # Empty-pool guard: a class must produce a finite mean regret.
    row = evaluate_class("mixed-energy", 5, 2, 4, 3, SHIPPED_MULTIPLIERS, 0.01)
    if not math.isfinite(row["mean_regret"]):
        print("[ERROR] mean regret is not finite")
        failures += 1
    # Determinism.
    a = run(11, 3, 2, 4, 0.01)
    b = run(11, 3, 2, 4, 0.01)
    if json.dumps(a, sort_keys=True) != json.dumps(b, sort_keys=True):
        print("[ERROR] run is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="Gateway cost regret bake-off (#669)")
    parser.add_argument("--topologies", type=int, default=300)
    parser.add_argument("--pool-min", type=int, default=3)
    parser.add_argument("--pool-max", type=int, default=10)
    parser.add_argument("--floor", type=float, default=0.01)
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args(argv)

    if args.self_check:
        return 1 if self_check() else 0

    report = run(args.seed, args.topologies, args.pool_min, args.pool_max, args.floor)
    for variant in report["variants"]:
        print(f"{variant['variant']}")
        for cls, row in variant["classes"].items():
            print(
                f"   {cls:<15} mean_regret={row['mean_regret']*100:5.2f}pp "
                f"p90={row['p90_regret']*100:5.2f}pp "
                f"inversions={row['inversion_rate']*100:5.1f}% "
                f"spearman={row['spearman_cost_vs_delivery']:+.3f}"
            )
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
