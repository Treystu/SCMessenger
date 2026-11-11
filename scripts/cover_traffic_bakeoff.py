#!/usr/bin/env python3
"""Offline privacy/overhead measurement for cover traffic (#673).

Evaluation-first and offline: nothing here is wired into the product. The
champion configuration is `CoverConfig::default()` and the padding/timing
primitives as read this session:

    core/src/privacy/cover.rs     rate_per_minute 10, message_size 1024, enabled true,
                                  scheduler fires on elapsed interval only (no coupling
                                  to real traffic volume)
    core/src/privacy/padding.rs   STANDARD_SIZES = [256, 512, 1024, 2048, 4096]
    core/src/privacy/timing.rs    jitter High 0-50 ms, Normal 50-500 ms, Low 100-2000 ms

The issue's complaint is that the tradeoff has never been measured, so this
harness measures three things per configuration on identical workload traces:

  1. CLASSIFIER ADVANTAGE -- a naive-Bayes adversary, given per-packet size and
     inter-arrival gap, separates cover packets from real ones. Accuracy over
     0.5 is the advantage. Perfect cover traffic makes this 0.0.
  2. BYTE OVERHEAD -- cover bytes paid per real byte carried, split into the
     idle regime and the burst regime, because the fixed cadence is worst at
     idle by construction.
  3. TIMING FINGERPRINT -- the coefficient of variation of inter-arrival gaps in
     the combined stream. A perfectly regular cadence is itself a signature;
     a stream whose gaps look like the workload's own is not.

Configurations: the shipped fixed cadence, the same rate with Poisson arrivals
(the classical alternative), a traffic-coupled rate, and each of those with the
Normal-priority jitter the product already implements.
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

RATE_PER_MINUTE = 10.0
COVER_SIZE = 1024
STANDARD_SIZES = [256, 512, 1024, 2048, 4096]
JITTER_NORMAL_MS = (50.0, 500.0)


@dataclass
class Packet:
    t: float  # seconds
    size: int
    is_cover: bool


def workload(rng: random.Random, duration: float, regime: str) -> List[Packet]:
    """Real traffic: an idle mesh, a chatty one, and one with a burst."""
    packets: List[Packet] = []
    t = 0.0
    if regime == "idle":
        rate = 0.2  # messages per minute
    elif regime == "chatty":
        rate = 12.0
    else:
        rate = 2.0
    while t < duration:
        if regime == "burst" and 0.4 * duration < t < 0.45 * duration:
            gap = rng.expovariate(60.0 / 60.0)  # 60/min inside the burst
        else:
            gap = rng.expovariate(max(1e-9, rate / 60.0))
        t += gap
        if t >= duration:
            break
        size = rng.choice(STANDARD_SIZES)
        packets.append(Packet(t, size, False))
    return packets


def pad(size: int) -> int:
    for standard in STANDARD_SIZES:
        if size <= standard:
            return standard
    return size


def cover_stream(rng: random.Random, duration: float, mode: str, real: Sequence[Packet]) -> List[Packet]:
    """Cover packets for one configuration."""
    packets: List[Packet] = []
    if mode.startswith("coupled"):
        # Rate follows real traffic: one cover packet per two real packets, and
        # nothing at all when the mesh is quiet.
        for index, packet in enumerate(real):
            if index % 2 == 0:
                packets.append(Packet(packet.t + rng.uniform(0.0, 0.2), COVER_SIZE, True))
        return packets
    interval = 60.0 / RATE_PER_MINUTE
    t = rng.uniform(0.0, interval)
    while t < duration:
        packets.append(Packet(t, COVER_SIZE, True))
        if mode.startswith("poisson"):
            t += rng.expovariate(1.0 / interval)
        else:
            t += interval
    return packets


def apply_jitter(rng: random.Random, packets: Sequence[Packet]) -> List[Packet]:
    out = []
    for packet in packets:
        lo, hi = JITTER_NORMAL_MS
        offset = rng.uniform(lo, hi) / 1000.0
        out.append(Packet(packet.t + offset, packet.size, packet.is_cover))
    return sorted(out, key=lambda p: p.t)


def features(stream: Sequence[Packet]) -> List[Tuple[float, float, bool]]:
    """Per-packet (size, log gap, is_cover) for the adversary."""
    rows: List[Tuple[float, float, bool]] = []
    previous = None
    for packet in stream:
        gap = 1.0 if previous is None else max(1e-3, packet.t - previous)
        rows.append((float(packet.size), math.log(gap), packet.is_cover))
        previous = packet.t
    return rows


def naive_bayes(rows: Sequence[Tuple[float, float, bool]], seed: int) -> float:
    """Accuracy of a two-class Gaussian adversary; 0.5 + advantage."""
    rng = random.Random(seed)
    if len(rows) < 10:
        return 0.5
    shuffled = list(rows)
    rng.shuffle(shuffled)
    split = int(0.7 * len(shuffled))
    train, test = shuffled[:split], shuffled[split:]
    stats: Dict[bool, List[Tuple[float, float]]] = {True: [], False: []}
    for size, gap, label in train:
        stats[label].append((size, gap))
    model: Dict[bool, Tuple[Tuple[float, float], Tuple[float, float], float]] = {}
    for label, values in stats.items():
        if not values:
            return 0.5
        sizes = [v[0] for v in values]
        gaps = [v[1] for v in values]
        model[label] = (
            (statistics.fmean(sizes), max(1e-6, statistics.pstdev(sizes))),
            (statistics.fmean(gaps), max(1e-6, statistics.pstdev(gaps))),
            len(values) / len(train),
        )

    def log_likelihood(label: bool, size: float, gap: float) -> float:
        (ms, ss), (mg, sg), prior = model[label]
        return (
            -0.5 * ((size - ms) / ss) ** 2
            - math.log(ss)
            - 0.5 * ((gap - mg) / sg) ** 2
            - math.log(sg)
            + math.log(max(1e-9, prior))
        )

    # BALANCED accuracy, not raw accuracy. With 600 cover packets against a
    # dozen real ones in an idle trace, a classifier that always says "cover"
    # scores ~98% raw and looks like an oracle while knowing nothing. Balanced
    # accuracy averages the per-class recalls, so 0.5 means "no better than
    # guessing within each class", which is the baseline the advantage must be
    # measured against. An earlier version of this function reported raw
    # accuracy and produced advantage figures that were pure class imbalance.
    recalls: List[float] = []
    for label in (True, False):
        members = [row for row in test if row[2] == label]
        if not members:
            continue
        hits = 0
        for size, gap, _ in members:
            guess = log_likelihood(True, size, gap) > log_likelihood(False, size, gap)
            hits += int(guess == label)
        recalls.append(hits / len(members))
    if not recalls:
        return 0.5
    return statistics.fmean(recalls)


def timing_fingerprint(stream: Sequence[Packet]) -> float:
    gaps = [b.t - a.t for a, b in zip(stream, stream[1:])]
    if len(gaps) < 2:
        return 0.0
    mean = statistics.fmean(gaps)
    return statistics.pstdev(gaps) / mean if mean else 0.0


def evaluate(mode: str, regime: str, duration: float, seed: int, jitter: bool) -> Dict[str, float]:
    rng = random.Random(f"{seed}-{mode}-{regime}".__hash__() & 0xFFFFFFFF)
    real = workload(rng, duration, regime)
    cover = cover_stream(rng, duration, mode, real)
    stream = real + cover
    if jitter:
        stream = apply_jitter(rng, stream)
    else:
        stream.sort(key=lambda p: p.t)
    rows = features(stream)
    accuracy = naive_bayes(rows, seed)
    real_bytes = sum(p.size for p in real)
    cover_bytes = sum(p.size for p in cover)
    return {
        "packets_real": float(len(real)),
        "packets_cover": float(len(cover)),
        "advantage": max(0.0, accuracy - 0.5),
        "byte_overhead": cover_bytes / real_bytes if real_bytes else float("inf"),
        "timing_cv": timing_fingerprint(stream),
    }


MODES = ("fixed", "poisson", "coupled", "fixed+jitter", "poisson+jitter", "coupled+jitter")


def run(duration: float, seed: int) -> Dict[str, object]:
    report: Dict[str, object] = {"seed": seed, "duration": duration, "regimes": {}}
    for regime in ("idle", "chatty", "burst"):
        report["regimes"][regime] = {}
        for mode in MODES:
            base = mode.replace("+jitter", "")
            report["regimes"][regime][mode] = evaluate(base, regime, duration, seed, "+jitter" in mode)
    return report


def self_check() -> int:
    failures = 0
    # The shipped cadence must be perfectly regular before jitter.
    rng = random.Random(3)
    real = workload(rng, 600, "idle")
    fixed = sorted(cover_stream(rng, 600, "fixed", real), key=lambda p: p.t)
    gaps = [b.t - a.t for a, b in zip(fixed, fixed[1:])]
    if gaps and (max(gaps) - min(gaps)) > 1e-9:
        print("[ERROR] the fixed cadence is not perfectly regular")
        failures += 1
    # Shipped cover packets are all one size: padding must be doing the work.
    if any(p.size != COVER_SIZE for p in fixed):
        print("[ERROR] cover packets are not a single size")
        failures += 1
    # Padding never shrinks and always lands on a standard size.
    for size in (1, 256, 257, 5000):
        if pad(size) < size or (pad(size) not in STANDARD_SIZES and pad(size) != size):
            print(f"[ERROR] pad({size}) = {pad(size)}")
            failures += 1
    # A classifier on a stream where cover is indistinguishable must be near 0.5.
    # (Synthetic: every packet identical in both features.)
    flat = [(512.0, math.log(1.0), index % 2 == 0) for index in range(400)]
    if abs(naive_bayes(flat, 5) - 0.5) > 0.25:
        print("[ERROR] the adversary is not calibrated on an indistinguishable stream")
        failures += 1
    # Determinism.
    if json.dumps(run(300, 11), sort_keys=True) != json.dumps(run(300, 11), sort_keys=True):
        print("[ERROR] run is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="Cover traffic privacy/overhead bake-off (#673)")
    parser.add_argument("--duration", type=float, default=3600.0)
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args(argv)
    if args.self_check:
        return 1 if self_check() else 0
    report = run(args.duration, args.seed)
    for regime, modes in report["regimes"].items():
        print(f"{regime} workload")
        for mode, row in modes.items():
            overhead = row["byte_overhead"]
            shown = "  inf" if math.isinf(overhead) else f"{overhead:5.2f}"
            print(
                f"  {mode:<15} cover={row['packets_cover']:6.0f}pk advantage={row['advantage']*100:5.1f}% "
                f"overhead={shown}x timing_cv={row['timing_cv']:.3f}"
            )
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
