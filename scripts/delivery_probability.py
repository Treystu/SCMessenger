#!/usr/bin/env python3
"""Offline delivery-probability evaluator -- audit component #666.

The decision this answers: "what is the probability a message is delivered across
this topology?", where delivery means an end-to-end path exists whose every link
succeeded. Links succeed independently with their own probability.

Three modes, and every result says which one produced it:

* ``exact`` -- recursive edge factoring over a parallel-reduced link set, cross-
               checked against brute-force subset enumeration whenever the link
               count is small enough for that to be cheap. The tool refuses to
               report an exact number if the two disagree beyond `--tolerance`.
* ``mc``    -- plain Monte Carlo with a Wilson interval, so an all-success run
               reports the resolution of its sample count instead of a fake zero.
* ``rare``  -- importance sampling for the failure probability. At realistic
               per-link success rates the disconnected event is rare (a 1e-4
               event needs ~298k samples for 10% relative error under plain MC),
               so this mode biases link failures, reweights by the likelihood
               ratio, and reports the relative error it achieved.

Topology JSON:

    {"source": "a", "sink": "d",
     "links": [{"a": "a", "b": "b", "p": 0.97}, ...]}

Parallel links are allowed and are independent alternatives.

Usage:
    python scripts/delivery_probability.py topo.json --mode exact
    python scripts/delivery_probability.py topo.json --mode mc --samples 200000
    python scripts/delivery_probability.py topo.json --mode rare --samples 200000
"""

from __future__ import annotations

import argparse
import json
import math
import random
from pathlib import Path
from typing import Dict, FrozenSet, Iterable, List, Optional, Sequence, Tuple

Link = Tuple[str, str, float]
Result = Dict[str, object]


# --------------------------------------------------------------------------- topology


def load_topology(path: str | Path) -> Dict[str, object]:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    links: List[Link] = []
    for i, raw in enumerate(data.get("links") or []):
        a, b = str(raw["a"]), str(raw["b"])
        p = float(raw["p"])
        if not 0.0 <= p <= 1.0:
            raise ValueError(f"link {i}: p={p} is outside [0, 1]")
        if a == b:
            raise ValueError(f"link {i}: self link {a}->{b}")
        links.append((a, b, p))
    src, dst = str(data["source"]), str(data["sink"])
    nodes = sorted({a for a, _, _ in links} | {b for _, b, _ in links} | {src, dst})
    return {"source": src, "sink": dst, "links": links, "nodes": nodes}


def _connected(live: Sequence[Link], nodes: Sequence[str], src: str, dst: str) -> bool:
    parent = {n: n for n in nodes}

    def find(x: str) -> str:
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for a, b, _ in live:
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[ra] = rb
    return find(src) == find(dst)


def _fold_parallel(links: Sequence[Link]) -> List[Link]:
    """Fold parallel links into one: 1 - prod(1 - p). Keeps factoring shallow."""
    groups: Dict[Tuple[str, str], float] = {}
    for a, b, p in links:
        key = (a, b) if a <= b else (b, a)
        groups[key] = 1.0 - (1.0 - groups.get(key, 0.0)) * (1.0 - p)
    return [(a, b, p) for (a, b), p in sorted(groups.items())]


# --------------------------------------------------------------------------- exact


def exact_bruteforce(topo: Dict[str, object]) -> float:
    """Reference implementation: sum the mass of every connecting link subset.

    Exponential in the link count by design; it exists to check the decomposition
    implementation, so it shares nothing with it beyond union-find.
    """
    links: List[Link] = list(topo["links"])  # type: ignore[arg-type]
    nodes: List[str] = list(topo["nodes"])  # type: ignore[arg-type]
    src, dst = str(topo["source"]), str(topo["sink"])
    total = 0.0
    for mask in range(1 << len(links)):
        prob = 1.0
        subset: List[Link] = []
        for i, link in enumerate(links):
            if mask >> i & 1:
                prob *= link[2]
                subset.append(link)
            else:
                prob *= 1.0 - link[2]
        if prob and _connected(subset, nodes, src, dst):
            total += prob
    return total


def exact_decomposition(topo: Dict[str, object]) -> float:
    """Exact delivery probability by recursive edge factoring.

    Condition on one undecided edge at a time: present contributes p times the
    sub-problem, absent contributes 1-p. States that are already connected
    return 1 immediately, which prunes the tree far below 2^n on real topologies.
    """
    nodes: List[str] = list(topo["nodes"])  # type: ignore[arg-type]
    src, dst = str(topo["source"]), str(topo["sink"])
    links = _fold_parallel(list(topo["links"]))  # type: ignore[arg-type]
    memo: Dict[Tuple[FrozenSet[int], FrozenSet[Tuple[str, str]]], float] = {}

    def solve(remaining: FrozenSet[int], present: FrozenSet[Tuple[str, str]]) -> float:
        live = [(a, b, 1.0) for (a, b) in present]
        if _connected(live, nodes, src, dst):
            return 1.0
        if not remaining:
            return 0.0
        key = (remaining, present)
        cached = memo.get(key)
        if cached is not None:
            return cached
        i = min(remaining)
        a, b, p = links[i]
        rest = remaining - {i}
        value = p * solve(rest, present | {(a, b)}) + (1.0 - p) * solve(rest, present)
        memo[key] = value
        return value

    return solve(frozenset(range(len(links))), frozenset())


def exact_reliability(topo: Dict[str, object], *, tolerance: float = 1e-9) -> Result:
    """Exact delivery probability, with both implementations cross-checked."""
    n = len(topo["links"])  # type: ignore[arg-type]
    brute: Optional[float] = exact_bruteforce(topo) if n <= 20 else None
    exact = exact_decomposition(topo)
    delta: Optional[float] = None if brute is None else abs(brute - exact)
    if delta is not None and delta > tolerance:
        raise AssertionError(
            f"exact implementations disagree: bruteforce={brute!r} decomposition={exact!r} "
            f"delta={delta:.3e} > tolerance={tolerance:.1e}"
        )
    return {
        "mode": "exact",
        "delivery_probability": exact,
        "failure_probability": 1.0 - exact,
        "link_count": n,
        "reduced_link_count": len(_fold_parallel(list(topo["links"]))),  # type: ignore[arg-type]
        "bruteforce_checked": brute is not None,
        "bruteforce_delta": delta,
        "tolerance": tolerance,
    }


# --------------------------------------------------------------------------- monte carlo


def monte_carlo(topo: Dict[str, object], samples: int = 200_000, seed: int = 0) -> Result:
    if samples <= 0:
        raise ValueError("samples must be positive")
    links: List[Link] = list(topo["links"])  # type: ignore[arg-type]
    nodes: List[str] = list(topo["nodes"])  # type: ignore[arg-type]
    src, dst = str(topo["source"]), str(topo["sink"])
    rng = random.Random(seed)
    hits = 0
    for _ in range(samples):
        live = [(a, b, p) for a, b, p in links if rng.random() < p]
        if _connected(live, nodes, src, dst):
            hits += 1
    p_hat = hits / samples
    z = 1.959963984540054
    denom = 1.0 + z * z / samples
    centre = (p_hat + z * z / (2 * samples)) / denom
    half = z * math.sqrt(p_hat * (1 - p_hat) / samples + z * z / (4 * samples * samples)) / denom
    return {
        "mode": "mc",
        "delivery_probability": p_hat,
        "failure_probability": 1.0 - p_hat,
        "samples": samples,
        "failures_observed": samples - hits,
        "standard_error": math.sqrt(max(p_hat * (1 - p_hat), 0.0) / samples),
        "wilson_95": [max(0.0, centre - half), min(1.0, centre + half)],
        "notes": (
            f"no failure observed; the failure estimate is the 1/{samples} resolution of this sample "
            "count, not a measurement -- use --mode rare for the failure probability"
            if hits == samples
            else ""
        ),
    }


# --------------------------------------------------------------------------- rare event


def rare_event_failure_probability(
    topo: Dict[str, object],
    samples: int = 200_000,
    seed: int = 0,
    failure_inflation: Optional[float] = None,
) -> Result:
    """Failure probability by importance sampling over biased link failures.

    For each link with success p, the biased success rate is p' = 1 - min(1, (1-p)*k),
    i.e. failures are made k times more likely (capped). A draw is reweighted by
    the likelihood ratio of the outcome it actually produced, so

        E_biased[w * 1(disconnected)] = P(disconnected)

    exactly, and the standard error of that mean gives the relative error.
    """
    if samples <= 0:
        raise ValueError("samples must be positive")
    links: List[Link] = list(topo["links"])  # type: ignore[arg-type]
    nodes: List[str] = list(topo["nodes"])  # type: ignore[arg-type]
    src, dst = str(topo["source"]), str(topo["sink"])

    def biased_params(k: float) -> List[Tuple[float, float, float, float]]:
        out = []
        for _, _, p in links:
            q = min(1.0, (1.0 - p) * k)
            pb = 1.0 - q
            out.append((p, 1.0 - p, pb, q))
        return out

    def estimate(k: float, rng: random.Random, n: int) -> Tuple[float, float, int]:
        params = biased_params(k)
        total = 0.0
        total_sq = 0.0
        events = 0
        for _ in range(n):
            live: List[Link] = []
            w = 1.0
            for (a, b, _), (p, q, pb, qb) in zip(links, params):
                if rng.random() < pb:
                    live.append((a, b, p))
                    if pb > 0:
                        w *= p / pb
                else:
                    if qb > 0:
                        w *= q / qb
            if not _connected(live, nodes, src, dst):
                events += 1
                total += w
                total_sq += w * w
        mean = total / n
        var = max(total_sq / n - mean * mean, 0.0)
        return mean, math.sqrt(var / n), events

    def event_rate(k: float, n: int) -> float:
        rng = random.Random(seed + 7919)
        params = biased_params(k)
        hits = 0
        for _ in range(n):
            live = [(a, b, pi) for (a, b, pi), (_, _, pb, _) in zip(links, params) if rng.random() < pb]
            if not _connected(live, nodes, src, dst):
                hits += 1
        return hits / n

    k = failure_inflation
    if k is None:
        k = 1.0
        pilot_n = max(samples // 20, 500)
        for _ in range(10):
            if event_rate(k, pilot_n) >= 0.01:
                break
            k *= 4.0

    q, se, events = estimate(k, random.Random(seed), samples)
    rel = (se / q) if q > 0 else float("inf")
    return {
        "mode": "rare",
        "failure_probability": q,
        "delivery_probability": 1.0 - q,
        "samples": samples,
        "failure_events_sampled": events,
        "failure_inflation": k,
        "standard_error": se,
        "relative_error": rel,
        "relative_error_within_10pct": bool(rel <= 0.10),
        "notes": (
            ""
            if events
            else "no disconnected draw even after biasing; the failure probability is below the "
            "resolution of this run"
        ),
    }


# --------------------------------------------------------------------------- CLI


def main(argv: Optional[Iterable[str]] = None) -> int:
    ap = argparse.ArgumentParser(description="Offline delivery-probability evaluator (#666)")
    ap.add_argument("topology", help="topology JSON file")
    ap.add_argument("--mode", choices=("exact", "mc", "rare"), default="exact")
    ap.add_argument("--samples", type=int, default=200_000)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--failure-inflation", type=float, default=None)
    ap.add_argument("--tolerance", type=float, default=1e-9)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(list(argv) if argv is not None else None)

    topo = load_topology(args.topology)
    if args.mode == "exact":
        result = exact_reliability(topo, tolerance=args.tolerance)
    elif args.mode == "mc":
        result = monte_carlo(topo, samples=args.samples, seed=args.seed)
    else:
        result = rare_event_failure_probability(
            topo, samples=args.samples, seed=args.seed, failure_inflation=args.failure_inflation
        )
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0

    print(f"mode={result['mode']} links={result.get('link_count', len(topo['links']))}")  # type: ignore[arg-type]
    print(f"delivery_probability={float(result['delivery_probability']):.6f}")  # type: ignore[arg-type]
    if result["mode"] == "exact":
        print(
            f"bruteforce_checked={result['bruteforce_checked']} "
            f"delta={result['bruteforce_delta']}"
        )
    elif result["mode"] == "mc":
        print(f"samples={result['samples']} failures_observed={result['failures_observed']}")
        lo, hi = result["wilson_95"]  # type: ignore[misc]
        print(f"standard_error={float(result['standard_error']):.3e} wilson_95=[{lo:.6f}, {hi:.6f}]")
    else:
        print(
            f"samples={result['samples']} events={result['failure_events_sampled']} "
            f"inflation={float(result['failure_inflation']):.3g}"
        )
        print(
            f"failure_probability={float(result['failure_probability']):.3e} "
            f"relative_error={float(result['relative_error']):.3f} "
            f"within_10pct={result['relative_error_within_10pct']}"
        )
    if result.get("notes"):
        print(f"[WARNING] {result['notes']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
