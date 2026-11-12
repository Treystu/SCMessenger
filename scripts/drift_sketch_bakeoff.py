#!/usr/bin/env python3
"""Offline IBLT sizing bake-off for drift sync (#671).

Evaluation-first and offline: nothing here is wired into the product. The
shipped adversary is modelled exactly as read from the tree on 2026-11-11:

  core/src/drift/sketch.rs:21  HASH_COUNT = 3
  core/src/drift/sketch.rs:22  CELLS_PER_DIFF = 3
  core/src/drift/sketch.rs:79  num_cells = expected_diffs.max(1) * CELLS_PER_DIFF
  core/src/drift/sketch.rs:231 serialized = 2 + num_cells * 21
  core/src/drift/sync.rs:185   iblt_capacity = (message_count * 4 + 8).max(1)

The consequence the issue predicts: an offer costs ~252 bytes per *stored*
message whatever the true divergence, so two large stores with a small
difference pay the most for the least. This harness measures bytes per
successful sync and decode success for:

  champion   -- shipped sizing (store-sized offer), no retry ladder.
  challenger -- difference-sized offer (3 * d_hat cells plus headroom, where
                d_hat is a sampled estimate), doubling the offer on decode
                failure up to a bounded ladder instead of failing the session.

The estimator is deliberately cheap: sample a fraction of the store, count
what the peer does not have, scale up. That is what a real count-exchange
would buy without shipping both stores.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import random
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Sequence, Tuple

HASH_COUNT = 3
CELLS_PER_DIFF = 3
CELL_BYTES = 21
HEADER_BYTES = 2
KEY_BYTES = 8  # MessageId stands in as 8 bytes; cell accounting matches the real format


def _hashes(key: bytes, cells: int) -> List[int]:
    out = []
    for index in range(HASH_COUNT):
        digest = hashlib.blake2b(key, digest_size=8, salt=bytes([index, 0, 0, 0])).digest()
        out.append(int.from_bytes(digest, "big") % cells)
    return out


@dataclass
class IBLT:
    cells: int
    count: List[int] = field(default_factory=list)
    keyxor: List[Optional[int]] = field(default_factory=list)
    keyxor2: List[Optional[int]] = field(default_factory=list)

    def __post_init__(self) -> None:
        self.cells = max(1, self.cells)
        self.count = [0] * self.cells
        self.keyxor = [0] * self.cells
        self.keyxor2 = [0] * self.cells

    def insert(self, key: int) -> None:
        """XOR the raw key so peeling recovers a key, not a hash of one.

        `keyxor2` carries an independent checksum of the same key, which decoding
        uses to reject a bad peel rather than trust it.
        """
        raw = key.to_bytes(KEY_BYTES, "big")
        for index in _hashes(raw, self.cells):
            self.count[index] += 1
            self.keyxor[index] ^= key
            self.keyxor2[index] ^= _second(key)

    def remove(self, key: int) -> None:
        self.insert(key)  # XOR and addition are self-inverse in this representation

    def subtract(self, other: "IBLT") -> "IBLT":
        result = IBLT(self.cells)
        for index in range(self.cells):
            result.count[index] = self.count[index] - other.count[index]
            result.keyxor[index] = self.keyxor[index] ^ other.keyxor[index]
            result.keyxor2[index] = self.keyxor2[index] ^ other.keyxor2[index]
        return result

    def decode(self, limit: int = 100_000) -> Tuple[Optional[List[int]], Optional[List[int]]]:
        """Peel cells of count +-1. Returns (only_left, only_right) or None on failure."""
        count = list(self.count)
        xor1 = list(self.keyxor)
        xor2 = list(self.keyxor2)
        left: List[int] = []
        right: List[int] = []
        iterations = 0
        while iterations < limit:
            iterations += 1
            empty = 0
            for index in range(self.cells):
                if count[index] == 0:
                    empty += 1
                    continue
                if count[index] == 1:
                    key = xor1[index]
                    if xor2[index] != _second(key):
                        return None, None
                    left.append(key)
                    for target in _hashes(key.to_bytes(KEY_BYTES, "big"), self.cells):
                        count[target] -= 1
                        xor1[target] ^= key
                        xor2[target] ^= _second(key)
                    break
                if count[index] == -1:
                    key = xor1[index]
                    if xor2[index] != _second(key):
                        return None, None
                    right.append(key)
                    for target in _hashes(key.to_bytes(KEY_BYTES, "big"), self.cells):
                        count[target] += 1
                        xor1[target] ^= key
                        xor2[target] ^= _second(key)
                    break
            else:
                break
            if empty == self.cells:
                break
        if any(c != 0 for c in count):
            return None, None
        return sorted(left), sorted(right)

    def byte_size(self) -> int:
        return HEADER_BYTES + self.cells * CELL_BYTES


def _second(key: int) -> int:
    raw = key.to_bytes(KEY_BYTES, "big")
    return int.from_bytes(hashlib.blake2b(raw, digest_size=8, salt=b"\x01" * 8).digest(), "big")


def build_store(rng: random.Random, size: int, low: int = 1) -> List[int]:
    return [low + i for i in range(size)] if size >= 0 else []


def scenario(rng: random.Random, store_n: int, diff_d: int) -> Tuple[List[int], List[int]]:
    """Two stores of n messages sharing n-d, with d differing keys on each side."""
    base = [rng.getrandbits(40) for _ in range(store_n)]
    mine = list(base)
    theirs = list(base)
    for _ in range(diff_d):
        mine.append(rng.getrandbits(40))
        theirs.append(rng.getrandbits(40))
    return mine, theirs


def estimate_difference(rng: random.Random, mine: Sequence[int], theirs_set: set, sample: int) -> int:
    """Sampled estimate of |mine \\ theirs|, scaled to the store size.

    Sampling is WITHOUT replacement -- an earlier version drew with replacement
    and under-counted so badly that the challenger failed to decode, which is a
    defect of the instrument rather than a property of the sizing idea. The
    estimate is scaled by the store-to-sample ratio and floored at 2, because an
    exact-zero estimate would size the sketch below the header.
    """
    if not mine:
        return 0
    take = min(len(mine), max(1, sample))
    draws = rng.sample(list(mine), take)
    hits = sum(1 for key in draws if key not in theirs_set)
    if take == len(mine):
        return max(1, hits)
    # An UPPER bound, not a point estimate. A sampled count that returns few hits
    # is consistent with a much larger true difference, and sizing on the point
    # estimate produces sketches that fail to decode -- which the acceptance
    # forbids ("decode success >= control"). The bound is the usual normal
    # approximation for a proportion with a floor for the zero-hit case.
    proportion = (hits + 2.0 * math.sqrt(hits + 1.0) + 1.0) / take
    return max(2, math.ceil(min(1.0, proportion) * len(mine)))


def champion_round(mine: Sequence[int], theirs: Sequence[int]) -> Dict[str, object]:
    capacity = max(1, len(mine) * 4 + 8)
    mine_table = IBLT(capacity * CELLS_PER_DIFF)
    theirs_table = IBLT(capacity * CELLS_PER_DIFF)
    for key in mine:
        mine_table.insert(key)
    for key in theirs:
        theirs_table.insert(key)
    delta = mine_table.subtract(theirs_table)
    left, right = delta.decode()
    ok = left is not None
    # Both sides exchange an offer of this size, so the session cost is 2x.
    return {"ok": ok, "bytes": 2 * mine_table.byte_size(), "offer_cells": mine_table.cells}


def challenger_round(
    rng: random.Random, mine: Sequence[int], theirs: Sequence[int], ladder: int, sample: int
) -> Dict[str, object]:
    theirs_set = set(theirs)
    estimate = estimate_difference(rng, mine, theirs_set, sample)
    total_bytes = 0
    offer_cells = 0
    for attempt in range(ladder):
        # Headroom 1.5 on top of the shipped 3 cells per difference: an estimate
        # that is exactly right still decodes poorly at alpha=3, so the offer is
        # padded once and the ladder doubles from there.
        cells = max(1, math.ceil(estimate * 1.5) * CELLS_PER_DIFF * (2 ** attempt))
        mine_table = IBLT(cells)
        theirs_table = IBLT(cells)
        for key in mine:
            mine_table.insert(key)
        for key in theirs:
            theirs_table.insert(key)
        delta = mine_table.subtract(theirs_table)
        left, right = delta.decode()
        total_bytes += 2 * mine_table.byte_size()
        offer_cells = cells
        if left is not None:
            return {"ok": True, "bytes": total_bytes, "offer_cells": offer_cells, "attempts": attempt + 1}
    return {"ok": False, "bytes": total_bytes, "offer_cells": offer_cells, "attempts": ladder}


def run(
    store_size: int,
    ratio: float,
    trials: int,
    seed: int,
    ladder: int = 4,
    sample: int = 64,
) -> Dict[str, object]:
    rng = random.Random(seed)
    diff = max(1, int(round(store_size * ratio)))
    champ_ok = 0
    champ_bytes = 0
    chall_ok = 0
    chall_bytes = 0
    chall_bytes_when_ok = 0
    for _ in range(trials):
        mine, theirs = scenario(rng, store_size, diff)
        c = champion_round(mine, theirs)
        x = challenger_round(rng, mine, theirs, ladder, sample)
        champ_ok += int(bool(c["ok"]))
        champ_bytes += int(c["bytes"])
        chall_ok += int(bool(x["ok"]))
        chall_bytes += int(x["bytes"])
        if x["ok"]:
            chall_bytes_when_ok += int(x["bytes"])
    return {
        "store_n": store_size,
        "ratio": ratio,
        "diff_d": diff,
        "trials": trials,
        "champion_decode_rate": champ_ok / trials,
        "challenger_decode_rate": chall_ok / trials,
        "champion_bytes_per_trial": champ_bytes / trials,
        "challenger_bytes_per_trial": chall_bytes / trials,
        "challenger_bytes_per_success": (chall_bytes_when_ok / chall_ok) if chall_ok else None,
        "byte_reduction_pct": (
            100.0 * (1 - (chall_bytes / champ_bytes)) if champ_bytes else 0.0
        ),
    }


def self_check() -> int:
    failures = 0
    # Both sides must recover exactly the differing keys.
    rng = random.Random(7)
    mine, theirs = scenario(rng, 40, 3)
    capacity = 4 * 40 + 8
    a = IBLT(capacity * CELLS_PER_DIFF)
    b = IBLT(capacity * CELLS_PER_DIFF)
    for key in mine:
        a.insert(key)
    for key in theirs:
        b.insert(key)
    left, right = a.subtract(b).decode()
    if left is None or right is None:
        print("[ERROR] a generously sized table failed to decode")
        failures += 1
    else:
        if sorted(set(mine) - set(theirs)) != sorted(left):
            print("[ERROR] recovered left set is not the true difference")
            failures += 1
        if sorted(set(theirs) - set(mine)) != sorted(right):
            print("[ERROR] recovered right set is not the true difference")
            failures += 1
    # The shipped sizing must be store-proportional (the premise of the issue).
    small = champion_round(mine, theirs)
    big_mine, big_theirs = scenario(rng, 400, 3)
    big = champion_round(big_mine, big_theirs)
    if big["bytes"] <= small["bytes"] * 5:
        print(f"[ERROR] shipped offer is not store-proportional: {small['bytes']} -> {big['bytes']}")
        failures += 1
    # Determinism.
    first = run(200, 0.01, 5, seed=11)
    second = run(200, 0.01, 5, seed=11)
    if json.dumps(first, sort_keys=True) != json.dumps(second, sort_keys=True):
        print("[ERROR] run is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="IBLT sizing bake-off for drift sync (#671)")
    parser.add_argument("--store-sizes", type=int, nargs="+", default=[1000, 10000])
    parser.add_argument("--ratios", type=float, nargs="+", default=[0.001, 0.01, 0.1])
    parser.add_argument("--trials", type=int, default=20)
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--ladder", type=int, default=4)
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args(argv)

    if args.self_check:
        return 1 if self_check() else 0

    report = {"seed": args.seed, "trials": args.trials, "ladder": args.ladder, "scenarios": []}
    for size in args.store_sizes:
        for ratio in args.ratios:
            row = run(size, ratio, args.trials, args.seed, ladder=args.ladder)
            report["scenarios"].append(row)
            per_success = row["challenger_bytes_per_success"]
            print(
                f"n={row['store_n']:>6} d/n={row['ratio']:<6} d={row['diff_d']:>5}  "
                f"champ {row['champion_bytes_per_trial']:>10.0f} B ({row['champion_decode_rate']*100:.0f}% ok)  "
                f"chall {row['challenger_bytes_per_trial']:>9.0f} B ({row['challenger_decode_rate']*100:.0f}% ok"
                + (f", {per_success:.0f} B/success" if per_success else "")
                + f")  reduction {row['byte_reduction_pct']:+.1f}%"
            )
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
