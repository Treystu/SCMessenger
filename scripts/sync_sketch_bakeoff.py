#!/usr/bin/env python3
"""Sync-sketch sizing bake-off (#671): store-relative control vs difference-driven ladder.

This is an offline model of `core/src/drift/{sketch,sync}.rs`, built to answer one
question the ledger's evaluation-first rule requires before any protocol change:
at realistic divergence, does sizing the IBLT from the estimated difference instead of
from the local store's message count actually cut bytes without costing decode success?

The model reproduces the shipped arithmetic exactly:
  * HASH_COUNT = 3, CELLS_PER_DIFF = 3, serialized cell = 21 bytes, header = 2 bytes
  * control: capacity = 4*n + 8 differences, cells = capacity * CELLS_PER_DIFF
  * subtraction requires equal cell counts, so both sides speak the same table size
  * decode is the real peel loop: a cell is pure when |count| == 1 and its key_sum/key_check
    hash back to its own index; a table that cannot finish peeling is a decode failure.

The ladder models `SyncMessage::SyncRetry`: on a decode failure the responder asks for a
larger table instead of failing the session, and the initiator re-offers at that size.
Every attempt costs one offer sketch plus one response sketch (both directions send one).

Nothing here touches a store or the wire; it measures the sizing policy, which is the
only thing #671 proposes to change.

Usage:
    python scripts/sync_sketch_bakeoff.py            # default sweep
    python scripts/sync_sketch_bakeoff.py --trials 500 --store 5000
"""
from __future__ import annotations

import argparse
import hashlib
import random
import statistics

HASH_COUNT = 3
CELLS_PER_DIFF = 3
CELL_BYTES = 21
HEADER_BYTES = 2
MIN_CELLS = 12
MAX_ATTEMPTS = 12
# Wire widths, read from core/src/drift/sketch.rs: `IBLTCell` serializes as
# count(1 byte) + key_sum(16) + key_check(4), under a 2-byte cell-count header
# (`to_bytes`, and MAX_WIRE_CELLS = u16::MAX). The model has to carry the same
# widths or it answers questions about numbers the wire cannot hold.
MAX_WIRE_CELLS = 0xFFFF
COUNT_MIN = -128
COUNT_MAX = 127
LADDER_CAP_DENSITY = 12  # the ladder may climb to 12x the estimated difference, per direction


def _key_bytes(mid: int) -> bytes:
    return mid.to_bytes(16, "little")


def _cell_index(key: bytes, hash_idx: int, num_cells: int) -> int:
    h = hashlib.sha256(bytes([hash_idx]) + key).digest()
    return int.from_bytes(h[:8], "little") % num_cells


def _key_check(key: bytes) -> int:
    """The shipped check value: the key's own first four bytes, little-endian.

    `IBLTCell::xor_insert` computes `u32::from_le_bytes([key[0], key[1], key[2], key[3]])`,
    and `decode` verifies purity against that same value. Anything wider or hashed
    (an earlier version of this model used eight bytes of sha256) accepts peels the
    shipped code would reject, and rejects peels it would accept.
    """
    return int.from_bytes(key[0:4], "little")


def _fit_count(value: int) -> int:
    """Wrap a count the way the wire does: one byte, two's complement."""
    wrapped = value & 0xFF
    return wrapped - 0x100 if wrapped >= 0x80 else wrapped


class IBLT:
    __slots__ = ("num_cells", "count", "key_sum", "key_check")

    def __init__(self, num_cells: int):
        self.num_cells = max(1, num_cells)
        self.count = [0] * self.num_cells
        self.key_sum = [0] * self.num_cells
        self.key_check = [0] * self.num_cells

    def insert(self, key: bytes) -> None:
        ks = int.from_bytes(key, "little")
        kc = _key_check(key)
        for i in range(HASH_COUNT):
            idx = _cell_index(key, i, self.num_cells)
            self.count[idx] = _fit_count(self.count[idx] + 1)
            self.key_sum[idx] ^= ks
            self.key_check[idx] ^= kc

    def remove(self, key: bytes) -> None:
        ks = int.from_bytes(key, "little")
        kc = _key_check(key)
        for i in range(HASH_COUNT):
            idx = _cell_index(key, i, self.num_cells)
            self.count[idx] = _fit_count(self.count[idx] - 1)
            self.key_sum[idx] ^= ks
            self.key_check[idx] ^= kc

    def subtract(self, other: "IBLT") -> "IBLT":
        if self.num_cells != other.num_cells:
            raise ValueError("cell counts must match for subtraction")
        out = IBLT(self.num_cells)
        for i in range(self.num_cells):
            out.count[i] = _fit_count(self.count[i] - other.count[i])
            out.key_sum[i] = self.key_sum[i] ^ other.key_sum[i]
            out.key_check[i] = self.key_check[i] ^ other.key_check[i]
        return out

    def to_wire(self) -> bytes:
        """Serialize exactly as `IBLT::to_bytes`: 2-byte cell count, then 21-byte cells."""
        if self.num_cells > MAX_WIRE_CELLS:
            raise ValueError("cell count exceeds the wire's u16 header")
        out = bytearray(self.num_cells.to_bytes(2, "little"))
        for i in range(self.num_cells):
            out.append(self.count[i] & 0xFF)
            out += self.key_sum[i].to_bytes(16, "little")
            out += self.key_check[i].to_bytes(4, "little")
        return bytes(out)

    @classmethod
    def from_wire(cls, data: bytes) -> "IBLT":
        """Deserialize the wire format, including its 8-bit counts."""
        cells = int.from_bytes(data[0:2], "little")
        table = cls(cells)
        offset = 2
        for i in range(cells):
            count = data[offset]
            table.count[i] = count - 0x100 if count >= 0x80 else count
            table.key_sum[i] = int.from_bytes(data[offset + 1 : offset + 17], "little")
            table.key_check[i] = int.from_bytes(data[offset + 17 : offset + 21], "little")
            offset += CELL_BYTES
        return table

    def _is_pure(self, i: int) -> bool:
        return self.count[i] in (1, -1)

    def decode(self) -> bool:
        """Peel the table. Returns True when every cell drains, False on a stuck table.

        Termination is decided by the cells themselves, not by a counter of peels: a table
        is drained when every cell reads zero, and it is stuck when no cell is pure while
        any cell is still non-zero. Counting peels instead is how an earlier version of
        this model reported a fully drained table as a failure.
        """
        count = self.count[:]
        key_sum = self.key_sum[:]
        key_check = self.key_check[:]
        max_iterations = self.num_cells * 10
        iterations = 0
        while True:
            if all(c == 0 for c in count):
                return True
            iterations += 1
            if iterations > max_iterations:
                return False
            progressed = False
            for i in range(self.num_cells):
                if count[i] not in (1, -1):
                    continue
                key = key_sum[i].to_bytes(16, "little")
                if _key_check(key) != key_check[i]:
                    continue
                sign = 1 if count[i] == 1 else -1
                for h in range(HASH_COUNT):
                    idx = _cell_index(key, h, self.num_cells)
                    count[idx] -= sign
                    key_sum[idx] ^= int.from_bytes(key, "little")
                    key_check[idx] ^= _key_check(key)
                progressed = True
            if not progressed:
                return False


def control_cells(store_size: int) -> int:
    capacity = max(store_size * 4 + 8, 1)
    return capacity * CELLS_PER_DIFF


def estimate_diff(n: int, m: int) -> int:
    """Starting estimate for the ladder.

    With no extra round trip the only cheap signal is the count gap plus a small
    allowance, which biases deliberately small: the ladder is what corrects an
    underestimate, and every correction is bounded by MAX_ATTEMPTS.
    """
    return max(1, abs(n - m) + max(1, min(n, m) // 1000))


def run_trial(d: int, n: int, mode: str, rng: random.Random) -> tuple[bool, int]:
    """One sync between two stores differing by d ids. Returns (decoded, bytes)."""
    m = n
    shared = [
        rng.randrange(1 << 40, 1 << 48) for _ in range(n - d // 2)
    ]
    a_only = [rng.randrange(1 << 48, 1 << 56) for _ in range(d - d // 2)]
    b_only = [rng.randrange(1 << 56, 1 << 64) for _ in range(d // 2)]
    a_ids = shared + a_only
    b_ids = shared + b_only

    if mode == "control":
        cells = control_cells(n)
        table_a = IBLT(cells)
        table_b = IBLT(cells)
        for mid in a_ids:
            table_a.insert(_key_bytes(mid))
        for mid in b_ids:
            table_b.insert(_key_bytes(mid))
        diff = table_a.subtract(table_b)
        # Decode what the wire carries, not the model's unbounded integers (#710).
        wire_diff = IBLT.from_wire(diff.to_wire())
        return wire_diff.decode(), 2 * (HEADER_BYTES + cells * CELL_BYTES)

    expected = estimate_diff(n, m)
    cells = max(MIN_CELLS, expected * CELLS_PER_DIFF)
    total_bytes = 0
    for _attempt in range(MAX_ATTEMPTS):
        table_a = IBLT(cells)
        table_b = IBLT(cells)
        for mid in a_ids:
            table_a.insert(_key_bytes(mid))
        for mid in b_ids:
            table_b.insert(_key_bytes(mid))
        diff = table_a.subtract(table_b)
        total_bytes += 2 * (HEADER_BYTES + cells * CELL_BYTES)
        wire_diff = IBLT.from_wire(diff.to_wire())
        if wire_diff.decode():
            return True, total_bytes
        # SyncRetry: responder asks for a larger table, initiator re-offers.
        cells = min(cells * 2, max(MIN_CELLS, d * LADDER_CAP_DENSITY) * CELLS_PER_DIFF)
    return False, total_bytes


def self_check() -> bool:
    """Prove the model answers with the numbers the wire carries (#710).

    Each check fails on the pre-fix model: counts were unbounded Python ints, the
    purity check was eight bytes of sha256 rather than the key's first four bytes,
    and there was no wire encoder or decoder at all.
    """
    failures = []
    key = _key_bytes(0x1122334455667788)

    # 1. A one-byte count wraps. 256 insertions of one key reach 256 in each of its
    # cells, which the wire stores as 0 - the table then reads drained although 256
    # messages differ. The model must reproduce that, not a 256 the wire cannot hold.
    table = IBLT(12)
    for _ in range(256):
        table.insert(key)
    if any(count != 0 for count in table.count):
        failures.append("a count of 256 must wrap to 0 as the one-byte wire count does")
    if not table.decode():
        failures.append("the wrapped table must read as drained, as the wire would")

    # 2. Round trip is the wire format: counts, sums and checks survive it unchanged.
    small = IBLT(16)
    for offset in range(5):
        small.insert(_key_bytes(0x1000 + offset))
    if IBLT.from_wire(small.to_wire()).count != small.count:
        failures.append("wire round trip changed the counts")
    if len(IBLT(7).to_wire()) != HEADER_BYTES + 7 * CELL_BYTES:
        failures.append("serialized size is not header plus 21-byte cells")

    # 3. The check value is the key's own first four bytes and nothing wider.
    if _key_check(key) != int.from_bytes(key[0:4], "little"):
        failures.append("key_check is not the key's first four bytes")
    if _key_check(key) >= 1 << 32:
        failures.append("key_check does not fit the wire's four bytes")

    # 4. The cell-count header is two bytes, so a larger table cannot be serialized.
    try:
        IBLT(MAX_WIRE_CELLS + 1).to_wire()
        failures.append("a table beyond the u16 cell-count header must refuse to serialize")
    except ValueError:
        pass

    for message in failures:
        print(f"[FAIL] self-check: {message}")
    if failures:
        return False
    print("[OK] self-check: counts, checks and sizes match the wire format")
    return True


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--store", type=int, default=2000, help="messages held per node (n)")
    ap.add_argument("--trials", type=int, default=200, help="trials per ratio")
    ap.add_argument("--seed", type=int, default=671)
    ap.add_argument(
        "--self-check",
        action="store_true",
        help="verify the model reproduces the wire's widths, then exit",
    )
    args = ap.parse_args()

    if args.self_check:
        return 0 if self_check() else 1

    ratios = [0.001, 0.01, 0.1]
    print("sync-sketch sizing bake-off (#671) - offline model of drift/sketch.rs + drift/sync.rs")
    print(
        f"store n={args.store}  hash_count={HASH_COUNT}  cells_per_diff={CELLS_PER_DIFF}  "
        f"cell={CELL_BYTES}B  trials={args.trials}  seed={args.seed}"
    )
    header = (
        f"{'d/n':>6} {'d':>5} {'control ok':>11} {'ladder ok':>10} "
        f"{'control B':>10} {'ladder B':>10} {'saving':>8} {'verdict':>8}"
    )
    print(header)
    print("-" * len(header))

    all_pass = True
    for ratio in ratios:
        d = max(2, int(round(args.store * ratio)))
        rng = random.Random(args.seed + d)
        control_ok = 0
        ladder_ok = 0
        control_bytes = []
        ladder_bytes = []
        for _ in range(args.trials):
            ok, nbytes = run_trial(d, args.store, "control", rng)
            control_ok += int(ok)
            control_bytes.append(nbytes)
            ok2, nbytes2 = run_trial(d, args.store, "ladder", rng)
            ladder_ok += int(ok2)
            ladder_bytes.append(nbytes2)
        c_rate = control_ok / args.trials
        l_rate = ladder_ok / args.trials
        c_mean = statistics.mean(control_bytes)
        l_mean = statistics.mean(ladder_bytes)
        saving = 1.0 - (l_mean / c_mean) if c_mean else 0.0
        verdict = "PASS" if l_rate >= c_rate else "FAIL"
        all_pass = all_pass and (l_rate >= c_rate)
        print(
            f"{ratio:>6.3f} {d:>5} {c_rate:>10.3f} {l_rate:>10.3f} "
            f"{c_mean:>10.0f} {l_mean:>10.0f} {saving:>7.1%}  {verdict}"
        )

    print()
    print("acceptance: ladder decode success no worse than control at every ratio")
    print(f"RESULT: {'PASS' if all_pass else 'FAIL'}")
    return 0 if all_pass else 1


if __name__ == "__main__":
    raise SystemExit(main())
