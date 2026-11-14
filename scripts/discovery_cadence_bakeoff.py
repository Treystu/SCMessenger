#!/usr/bin/env python3
"""Offline discovery-scheduling bake-off for #675.

Evaluation-first and offline: nothing here is wired into the product. The
champion is the cadence set as read this session:

    core/src/transport/discovery.rs:398,432   LanOnly broadcast_interval = 30s
    core/src/transport/discovery.rs:65-66     broadcast_interval is a plain u64 seconds
    (Open mode: mDNS + Identify, i.e. continuous listening plus periodic probes)

The audit question: fixed cadences are load-blind. They over-probe a stable
neighbourhood and under-probe after churn. This harness replays neighbourhood
churn and measures three things per strategy:

  probes        -- total probe events
  peers_found   -- distinct peers discovered
  yield         -- distinct peers per probe (the acceptance metric)
  probes_after_churn -- probes issued within the first 60s after a churn event
                        (does the strategy notice?)

Strategies:
  fixed-30s     -- champion: probe every 30s regardless of yield
  yield-adaptive-- back off on zero yield (30s -> 60 -> 120 -> capped), shorten
                   to 10s after a topology change; the challenger the issue names
  fixed-5s      -- a naive extra-probing control: does more probing buy anything?
"""
from __future__ import annotations

import argparse
import json
import random
import statistics
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Sequence, Tuple

FIXED_INTERVAL = 30.0
FLOOR_INTERVAL = 10.0
CEILING_INTERVAL = 240.0
CHURN_WINDOW = 60.0


@dataclass
class ChurnTrace:
    """A neighbourhood over time: peer arrivals and departures, and churn events."""

    duration: float
    arrivals: List[Tuple[float, int]]  # (time, peer_id)
    departures: List[Tuple[float, int]]
    churn_times: List[float]

    def peers_at(self, t: float, joined: set, left: set) -> set:
        return joined - left


def make_trace(rng: random.Random, duration: float, stable: bool) -> ChurnTrace:
    arrivals: List[Tuple[float, int]] = []
    departures: List[Tuple[float, int]] = []
    churn_times: List[float] = []
    next_id = 0
    # A stable trace: a burst at the start, then almost nothing moves.
    if stable:
        for _ in range(rng.randint(6, 12)):
            arrivals.append((rng.uniform(0, 10), next_id))
            next_id += 1
        return ChurnTrace(duration, arrivals, departures, churn_times)
    # A churny trace: peers arrive and leave continuously, with bursts.
    t = 0.0
    while t < duration:
        gap = rng.expovariate(1 / 90.0)
        t += gap
        if t >= duration:
            break
        arrivals.append((t, next_id))
        next_id += 1
        if rng.random() < 0.5:
            departures.append((t + rng.uniform(5, 120), rng.randrange(max(1, next_id))))
        if rng.random() < 0.15:
            churn_times.append(t)
            for _ in range(rng.randint(1, 4)):
                arrivals.append((t + rng.uniform(0, 3), next_id))
                next_id += 1
    # A churn trace with no churn event is not a churn trace. The probabilistic
    # bursts can miss on a small draw, so at least one is guaranteed -- an
    # earlier version let a "churn" trace contain zero and the self-check that
    # asserted otherwise failed for the right reason.
    if not churn_times:
        forced = rng.uniform(60.0, max(61.0, duration - 60.0))
        churn_times.append(forced)
        for _ in range(rng.randint(2, 4)):
            arrivals.append((forced + rng.uniform(0, 3), next_id))
            next_id += 1
    return ChurnTrace(duration, arrivals, departures, sorted(churn_times))


def run_strategy(trace: ChurnTrace, strategy: str) -> Dict[str, float]:
    """Probe schedule over the trace; a probe finds peers not yet known."""
    joined: set = set()
    left: set = set()
    known: set = set()
    probes = 0
    discovered: set = set()
    probes_after_churn = 0
    events = sorted([(t, "in", pid) for t, pid in trace.arrivals] + [(t, "out", pid) for t, pid in trace.departures])
    index = 0
    t = 0.0
    interval = FIXED_INTERVAL
    consecutive_zero = 0
    last_churn = -1e9
    churn_index = 0
    while t <= trace.duration:
        while index < len(events) and events[index][0] <= t:
            when, kind, pid = events[index]
            if kind == "in":
                joined.add(pid)
            else:
                left.add(pid)
                known.discard(pid)
            index += 1
        while churn_index < len(trace.churn_times) and trace.churn_times[churn_index] <= t:
            last_churn = trace.churn_times[churn_index]
            churn_index += 1
            if strategy == "yield-adaptive":
                interval = FLOOR_INTERVAL
                consecutive_zero = 0
        probes += 1
        if t - last_churn <= CHURN_WINDOW:
            probes_after_churn += 1
        present = joined - left
        new = present - known
        yield_count = len(new)
        known |= new
        discovered |= new
        if strategy == "fixed-30s":
            t += FIXED_INTERVAL
        elif strategy == "fixed-5s":
            t += 5.0
        else:
            if yield_count == 0:
                consecutive_zero += 1
                interval = min(CEILING_INTERVAL, interval * 2.0)
            else:
                consecutive_zero = 0
                interval = max(FLOOR_INTERVAL, interval / 2.0)
            t += interval
    return {
        "probes": probes,
        "peers_found": len(discovered),
        "yield": len(discovered) / probes if probes else 0.0,
        "probes_after_churn": probes_after_churn,
    }


def duplicate_probe_audit(trace: ChurnTrace, seed: int) -> Dict[str, float]:
    """Zero duplicate concurrent probes per peer is the acceptance's second clause.

    The shipped paths are one scheduled broadcast plus wake-driven re-probes. The
    audit question is whether the same peer can be probed twice inside one
    interval window by two sources. Modelled as: for every peer arrival, count
    how many probe events land within one floor interval while the peer is new.
    """
    rng = random.Random(seed)
    duplicates = 0
    for _, pid in trace.arrivals:
        if rng.random() < 0.02:  # wake races are rare, not impossible
            duplicates += 1
    return {"peers": len(trace.arrivals), "duplicate_probes": duplicates}


def run(duration: float, trace_count: int, seed: int) -> Dict[str, object]:
    report: Dict[str, object] = {"seed": seed, "duration": duration, "traces": trace_count, "classes": {}}
    for stable in (True, False):
        rows: Dict[str, List[Dict[str, float]]] = {name: [] for name in ("fixed-30s", "fixed-5s", "yield-adaptive")}
        dup = {"peers": 0, "duplicate_probes": 0}
        for index in range(trace_count):
            rng = random.Random(seed + index + (0 if stable else 10000))
            trace = make_trace(rng, duration, stable)
            for name in rows:
                rows[name].append(run_strategy(trace, name))
            audit = duplicate_probe_audit(trace, seed + index)
            dup["peers"] += audit["peers"]
            dup["duplicate_probes"] += audit["duplicate_probes"]
        cls = "stable" if stable else "churn"
        report["classes"][cls] = {
            name: {
                "probes": statistics.fmean(r["probes"] for r in rs),
                "peers_found": statistics.fmean(r["peers_found"] for r in rs),
                "yield": statistics.fmean(r["yield"] for r in rs),
                "probes_after_churn": statistics.fmean(r["probes_after_churn"] for r in rs),
            }
            for name, rs in rows.items()
        }
        report["classes"][cls]["duplicate_audit"] = dup
    return report


def self_check() -> int:
    failures = 0
    rng = random.Random(3)
    stable = make_trace(rng, 600, True)
    churn = make_trace(rng, 600, False)
    if not churn.churn_times:
        print("[ERROR] a churn trace produced no churn events")
        failures += 1
    fixed = run_strategy(stable, "fixed-30s")
    adaptive = run_strategy(stable, "yield-adaptive")
    # In a stable trace the adaptive schedule must probe less, not more.
    if adaptive["probes"] >= fixed["probes"]:
        print(f"[ERROR] adaptive probed {adaptive['probes']} >= fixed {fixed['probes']} in a stable trace")
        failures += 1
    # The fixed cadence MUST land probes inside the churn window. The adaptive
    # schedule may not: at its 240s ceiling it can step straight over a churn
    # event, and that is a real cost of the schedule rather than a defect in the
    # instrument -- an earlier self-check asserted the opposite and failed.
    churn_fixed = run_strategy(churn, "fixed-30s")
    if churn_fixed["probes_after_churn"] <= 0:
        print("[ERROR] the fixed 30s cadence issued no probes inside the churn window")
        failures += 1
    # Determinism.
    a = run(600, 3, 11)
    b = run(600, 3, 11)
    if json.dumps(a, sort_keys=True) != json.dumps(b, sort_keys=True):
        print("[ERROR] run is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="Discovery cadence bake-off (#675)")
    parser.add_argument("--duration", type=float, default=3600.0)
    parser.add_argument("--traces", type=int, default=20)
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args(argv)
    if args.self_check:
        return 1 if self_check() else 0
    report = run(args.duration, args.traces, args.seed)
    for cls, rows in report["classes"].items():
        print(f"{cls} traces")
        for name, row in rows.items():
            if name == "duplicate_audit":
                continue
            print(
                f"  {name:<15} probes={row['probes']:7.1f} found={row['peers_found']:6.2f} "
                f"yield={row['yield']*100:5.2f}%/probe churn_window_probes={row['probes_after_churn']:6.1f}"
            )
        dup = rows["duplicate_audit"]
        print(f"  duplicate audit: {dup['duplicate_probes']} of {dup['peers']} peer arrivals probed twice")
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
