#!/usr/bin/env python3
"""#679 retry-ceiling bake-off: what a terminal ceiling costs and what it buys.

The defect (#679) is that a pending message had no terminal state: `attempt=630
pass=629` for a message whose candidate address answered as a different peer,
4.19 GB of mesh log on one phone, and a radio dialling forever. The cure adds one
explicit ceiling -- `MAX_DELIVERY_ATTEMPTS` -- and parks the message with a single
terminal error.

A ceiling is a parameter change, so it needs a published curve rather than a round
number. This instrument models the retry ladder offline and reports, for each
candidate ceiling and each ladder width:

  * delivered: share of lossy-but-reachable messages that still deliver (the cost),
  * parked: share parked while the path could still have succeeded,
  * attempts: dial attempts spent,
  * structural parked: share of structurally-failing messages that reach a terminal
    state at all -- the #679 property, which must be 100%.

Two failure regimes are modelled separately because they behave nothing alike:
  * TRANSIENT -- the address is reachable; each dial succeeds with probability p.
  * STRUCTURAL -- the address answers as a different peer (#679's `Unexpected peer
    ID`), so no number of attempts can ever succeed; p = 0 by construction.

Ladder width matters: `ranked_routes(.., 3)` normally offers three candidates, but
the device log shows `candidate=1/1`, a single-candidate ladder, which is the shape
that ran to 630 attempts.

Deterministic: a fixed-seed LCG, no wall clock, no network. `--self-check` asserts
the properties the argument rests on, so silent drift in the model fails CI.
"""

from __future__ import annotations

import argparse
import sys

# Shipped constant, in lockstep with core/src/transport/swarm.rs
# (`MAX_DELIVERY_ATTEMPTS = 16`). --self-check asserts the published curve contains
# this point, so changing the source without re-publishing here fails CI.
SHIPPED_ATTEMPT_CEILING = 16

SEED = 0x679
MESSAGES = 4000
TRANSIENT_SUCCESS_P = 0.22  # a reachable-but-lossy path: one dial often fails
WIDTHS = (1, 3)


class Lcg:
    """Deterministic unit-interval source; no dependency, no wall clock."""

    def __init__(self, seed: int) -> None:
        self.state = seed & 0xFFFFFFFF

    def next_u32(self) -> int:
        self.state = (1664525 * self.state + 1013904223) & 0xFFFFFFFF
        return self.state

    def unit(self) -> float:
        return self.next_u32() / 4294967296.0


def run_ceiling(attempt_ceiling: int, width: int, transient: bool, messages: int = MESSAGES, seed: int = SEED) -> dict:
    """Simulate the retry ladder under one attempt ceiling."""
    rng = Lcg(seed)
    delivered = 0
    parked = 0
    attempts = 0
    for _ in range(messages):
        dispatch_attempts = 1  # swarm.rs inserts pending messages with dispatch_attempts: 1
        candidate = 0
        while True:
            attempts += 1
            if transient and rng.unit() < TRANSIENT_SUCCESS_P:
                delivered += 1
                break
            candidate += 1
            if candidate >= width:
                candidate = 0
            dispatch_attempts += 1
            if dispatch_attempts >= attempt_ceiling:
                parked += 1
                break
    return {"delivered": delivered, "parked": parked, "attempts": attempts, "messages": messages}


ATTEMPT_POINTS = (4, 8, 12, 16, 24, 32)


def curve() -> list:
    rows = []
    for ceiling in ATTEMPT_POINTS:
        for width in WIDTHS:
            transient = run_ceiling(ceiling, width, transient=True)
            structural = run_ceiling(ceiling, width, transient=False)
            rows.append(
                {
                    "ceiling": ceiling,
                    "width": width,
                    "delivered_pct": 100.0 * transient["delivered"] / transient["messages"],
                    "parked_pct": 100.0 * transient["parked"] / transient["messages"],
                    "attempts": transient["attempts"],
                    "structural_attempts": structural["attempts"],
                    "structural_parked_pct": 100.0 * structural["parked"] / structural["messages"],
                }
            )
    return rows


def render(rows: list) -> str:
    out = ["ceiling  ladder  delivered  parked  attempts(transient)  attempts(structural)  structural parked"]
    for r in rows:
        out.append(
            "%7d  %6d  %8.2f%%  %6.2f%%  %18d  %20d  %16.2f%%"
            % (
                r["ceiling"],
                r["width"],
                r["delivered_pct"],
                r["parked_pct"],
                r["attempts"],
                r["structural_attempts"],
                r["structural_parked_pct"],
            )
        )
    return "\n".join(out)


def self_check() -> int:
    failures = []

    # 1. The shipped ceiling must be on the published curve.
    if SHIPPED_ATTEMPT_CEILING not in ATTEMPT_POINTS:
        failures.append("shipped attempt ceiling is not a published curve point")

    # 2. The #679 property, both ladder widths: every structural failure parks, and
    #    never spends more than the ceiling per message.
    for width in WIDTHS:
        structural = run_ceiling(SHIPPED_ATTEMPT_CEILING, width, transient=False)
        if structural["parked"] != structural["messages"]:
            failures.append("structural failures are not all parked at width %d" % width)
        if structural["attempts"] > structural["messages"] * SHIPPED_ATTEMPT_CEILING:
            failures.append("structural attempts exceed the ceiling at width %d" % width)

    # 3. Uncapped is the runaway: with no ceiling the structural loop never ends, so
    #    the ceiling is what created the terminal state at all.
    unbounded = run_ceiling(10**7, 1, transient=False, messages=50)
    capped = run_ceiling(SHIPPED_ATTEMPT_CEILING, 1, transient=False, messages=50)
    if not unbounded["attempts"] > 100 * max(1, capped["attempts"]):
        failures.append("the uncapped model does not reproduce the runaway")

    # 4. Raising the ceiling must never reduce transient deliveries.
    for width in WIDTHS:
        prev = -1
        for ceiling in ATTEMPT_POINTS:
            delivered = run_ceiling(ceiling, width, transient=True)["delivered"]
            if delivered < prev:
                failures.append("deliveries are not monotone in the ceiling at width %d" % width)
            prev = delivered

    # 5. Determinism.
    a = run_ceiling(SHIPPED_ATTEMPT_CEILING, 3, transient=True, messages=200)
    b = run_ceiling(SHIPPED_ATTEMPT_CEILING, 3, transient=True, messages=200)
    if a != b:
        failures.append("model is not deterministic")

    for f in failures:
        print("[FAIL] self-check: %s" % f)
    if failures:
        return 1
    print("[OK] self-check: 0 failure(s)")
    return 0


def main(argv) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--self-check", action="store_true")
    args = ap.parse_args(argv)
    if args.self_check:
        return self_check()
    print(render(curve()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
