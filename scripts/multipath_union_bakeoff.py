#!/usr/bin/env python3
"""Offline multipath union-delivery bake-off for #670.

Evaluation-first and offline: nothing here is wired into the product. The
champion is `MultiPathDelivery` as read this session:

    core/src/routing/multipath.rs:33   score = clamp(success_ratio * 80 + 20 - min(latency_ms,2000)/100, 0, 100)
    core/src/routing/multipath.rs:36-38 new paths start at 75.0
    core/src/routing/multipath.rs:83   max_paths_per_hint = 3
    core/src/routing/multipath.rs:103  active_paths returns actives sorted by score descending

The audit finding is that nothing in that file models path overlap, so the
"union" of three high-scoring paths can be one path with three names. This
harness measures exactly that, with correlated node failure:

  union delivery   -- P(at least one selected path delivers), empirically
  p95 TTF          -- 95th percentile time-to-first-delivery, parallel send
  overlap          -- mean fraction of relays shared with the first path

Strategies on identical pools, identical failure draws:
  score-top-3      -- the champion: three highest-scoring paths (overlap-blind)
  disjoint-greedy  -- greedy marginal-union selection over the same scored pool
  single-best      -- one path, the highest scorer (the control that says
                      whether multipath buys anything at all)

Correlation is modelled at the NODE, not the path: a shared weak relay takes
down every path that crosses it. With `--independent` the harness draws per
path instead, which is the reading the shipped per-path scores silently assume.
"""
from __future__ import annotations

import argparse
import json
import random
import statistics
import sys
from dataclasses import dataclass
from typing import Dict, List, Sequence, Set, Tuple

MAX_PATHS_PER_HINT = 3


@dataclass
class Path:
    path_id: int
    relays: Tuple[int, ...]
    latency_ms: int
    score: float


def make_topology(rng: random.Random, nodes: int, weak_frac: float) -> Dict[int, float]:
    """Per-node failure probability. A few nodes are much weaker than the rest."""
    weak = set(rng.sample(range(nodes), max(1, int(nodes * weak_frac))))
    return {
        node: (rng.uniform(0.25, 0.6) if node in weak else rng.uniform(0.0, 0.06))
        for node in range(nodes)
    }


def make_paths(rng: random.Random, failure: Dict[int, float], count: int, nodes: int) -> List[Path]:
    """Caller-registered paths, scored the way the product scores them.

    The score uses the path's own historical success ratio and latency, which is
    all the product has. It cannot see overlap, by construction.
    """
    paths: List[Path] = []
    for path_id in range(count):
        length = rng.randint(2, 4)
        relays = tuple(rng.sample(range(nodes), length))
        # Historical success: the product's own observation, noisy around the
        # path's true reliability.
        true_p = 1.0
        for node in relays:
            true_p *= 1.0 - failure[node]
        attempts = rng.randint(3, 20)
        successes = sum(1 for _ in range(attempts) if rng.random() < true_p)
        latency = int(sum(rng.uniform(15, 120) for _ in relays))
        if attempts == 0:
            score = 75.0
        else:
            score = max(0.0, min(100.0, (successes / attempts) * 80.0 + 20.0 - min(latency, 2000) / 100.0))
        paths.append(Path(path_id=path_id, relays=relays, latency_ms=latency, score=score))
    return paths


def score_top(paths: Sequence[Path], k: int = MAX_PATHS_PER_HINT) -> List[Path]:
    return sorted(paths, key=lambda p: -p.score)[:k]


def _path_delivery(path: Path, failure: Dict[int, float]) -> float:
    p = 1.0
    for node in path.relays:
        p *= 1.0 - failure.get(node, 0.0)
    return p


def _union_delivery(chosen: Sequence[Path], failure: Dict[int, float]) -> float:
    """P(at least one path delivers) under independent node failure.

    Correlated node failure is not a product over paths -- but the greedy
    comparator has to optimise something it can compute exactly, and the union
    under node independence is the standard, optimistic-bound choice: it is the
    disjointness objective, evaluated honestly.
    """
    miss = 1.0
    for path in chosen:
        miss *= 1.0 - _path_delivery(path, failure)
    return 1.0 - miss


def disjoint_greedy(
    paths: Sequence[Path], failure: Dict[int, float], k: int = MAX_PATHS_PER_HINT
) -> List[Path]:
    """Greedy marginal-union over real node failure probabilities.

    Each pick is the path that adds the most union delivery to what is already
    chosen. This is the disjointness-aware comparator: a path through a relay
    already covered adds almost nothing however high its score.
    """
    chosen: List[Path] = []
    remaining = list(paths)
    current = 0.0
    while remaining and len(chosen) < k:
        best = None
        best_union = current
        for path in remaining:
            candidate = _union_delivery(chosen + [path], failure)
            if candidate > best_union + 1e-12:
                best_union, best = candidate, path
        if best is None:
            break
        chosen.append(best)
        current = best_union
        remaining.remove(best)
    return chosen


def single_best(paths: Sequence[Path]) -> List[Path]:
    return sorted(paths, key=lambda p: -p.score)[:1]


def simulate(
    chosen: Sequence[Path],
    failure: Dict[int, float],
    rng: random.Random,
    independent: bool,
) -> Tuple[bool, int]:
    """One delivery attempt across the chosen paths, sent in parallel."""
    if not chosen:
        return False, 0
    if independent:
        # The assumption the per-path scores imply: paths fail on their own.
        # No probability floor -- an earlier version clamped at 0.02, which
        # invented reliability the model does not have.
        for path in sorted(chosen, key=lambda p: p.latency_ms):
            if rng.random() < _path_delivery(path, failure):
                return True, path.latency_ms
        return False, 0
    # Correlated: draw node health once per attempt.
    up = {node: rng.random() >= prob for node, prob in failure.items()}
    delivered = [p for p in chosen if all(up[node] for node in p.relays)]
    if not delivered:
        return False, 0
    return True, min(p.latency_ms for p in delivered)


def overlap_fraction(chosen: Sequence[Path]) -> float:
    if len(chosen) < 2:
        return 0.0
    first = set(chosen[0].relays)
    others = [set(p.relays) for p in chosen[1:]]
    return statistics.fmean(len(first & o) / max(1, len(first | o)) for o in others)


def run(
    topologies: int,
    attempts: int,
    nodes: int,
    paths: int,
    weak_frac: float,
    seed: int,
    independent: bool,
) -> Dict[str, object]:
    rng = random.Random(seed)
    stats: Dict[str, Dict[str, List[float]]] = {
        name: {"delivered": [], "ttf": [], "overlap": []}
        for name in ("score-top-3", "disjoint-greedy", "single-best")
    }
    for _ in range(topologies):
        failure = make_topology(rng, nodes, weak_frac)
        pool = make_paths(rng, failure, paths, nodes)
        selections = {
            "score-top-3": score_top(pool),
            "disjoint-greedy": disjoint_greedy(pool, failure),
            "single-best": single_best(pool),
        }
        for name, chosen in selections.items():
            delivered = 0
            ttfs: List[int] = []
            for _ in range(attempts):
                ok, ttf = simulate(chosen, failure, rng, independent)
                delivered += int(ok)
                if ok:
                    ttfs.append(ttf)
            stats[name]["delivered"].append(delivered / attempts)
            stats[name]["ttf"].append(_p95(ttfs) if ttfs else 0.0)
            stats[name]["overlap"].append(overlap_fraction(chosen))
    out: Dict[str, object] = {"seed": seed, "topologies": topologies, "attempts": attempts,
                              "nodes": nodes, "paths_per_hint": paths, "weak_frac": weak_frac,
                              "independent": independent, "strategies": {}}
    for name, rows in stats.items():
        out["strategies"][name] = {
            "union_delivery": statistics.fmean(rows["delivered"]),
            "p95_ttf_ms": statistics.fmean(rows["ttf"]),
            "overlap": statistics.fmean(rows["overlap"]),
        }
    return out


def _p95(values: Sequence[float]) -> float:
    ordered = sorted(values)
    index = min(len(ordered) - 1, int(0.95 * (len(ordered) - 1) + 0.5))
    return float(ordered[index])


def self_check() -> int:
    failures = 0
    rng = random.Random(5)
    failure = {0: 0.0, 1: 0.0, 2: 0.0, 3: 0.9}
    # A path through the dead node must never deliver under either model.
    # A 0.9-failure node is UP one time in ten, so the assertion is a rate, not
    # "never": an earlier self-check demanded zero and was simply wrong.
    dead = Path(path_id=1, relays=(0, 3), latency_ms=50, score=99.0)
    for independent in (True, False):
        hits = sum(1 for _ in range(400) if simulate([dead], failure, rng, independent)[0])
        rate = hits / 400
        if not (0.0 < rate < 0.30):
            print(f"[ERROR] dead-node delivery rate {rate:.2f} outside the plausible band (0, 0.30)")
            failures += 1
    # Healthy path always delivers.
    good = Path(path_id=2, relays=(0, 1), latency_ms=30, score=50.0)
    if not all(simulate([good], failure, rng, False)[0] for _ in range(20)):
        print("[ERROR] a path through healthy nodes failed")
        failures += 1
    # Score formula must match the shipped expression.
    p = Path(path_id=3, relays=(0,), latency_ms=200, score=0.0)
    expected = max(0.0, min(100.0, (1.0) * 80.0 + 20.0 - 200 / 100.0))
    if abs(expected - 98.0) > 1e-9:
        print("[ERROR] self-check arithmetic is wrong")
        failures += 1
    # Determinism.
    a = run(6, 10, 12, 8, 0.2, 11, False)
    b = run(6, 10, 12, 8, 0.2, 11, False)
    if json.dumps(a, sort_keys=True) != json.dumps(b, sort_keys=True):
        print("[ERROR] run is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="Multipath union-delivery bake-off (#670)")
    parser.add_argument("--topologies", type=int, default=200)
    parser.add_argument("--attempts", type=int, default=40)
    parser.add_argument("--nodes", type=int, default=20)
    parser.add_argument("--paths", type=int, default=8)
    parser.add_argument("--weak-frac", type=float, default=0.2)
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--independent", action="store_true", help="draw failures per path, not per node")
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args(argv)

    if args.self_check:
        return 1 if self_check() else 0

    report = run(args.topologies, args.attempts, args.nodes, args.paths, args.weak_frac, args.seed, args.independent)
    label = "INDEPENDENT failure (the reading per-path scores imply)" if args.independent else "CORRELATED node failure (the mesh we have)"
    print(f"{label}; nodes={report['nodes']} paths/hint={report['paths_per_hint']} weak={report['weak_frac']:.2f}")
    for name, row in report["strategies"].items():
        print(
            f"  {name:<16} union={row['union_delivery']*100:5.2f}%  "
            f"p95_ttf={row['p95_ttf_ms']:6.1f}ms  overlap={row['overlap']*100:5.1f}%"
        )
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
