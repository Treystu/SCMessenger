#!/usr/bin/env python3
"""Dial-policy replay bake-off for issue #674.

Question (#674): is the fixed ranked-ladder plus exponential-backoff dial policy the
champion, or does a bandit-ranked challenger establish connections with fewer probes
at no loss of reliability?

Scope, stated so nobody mistakes this for a field measurement: this is an offline
replay of the *policy arithmetic* over a synthetic but adversarial peer population.
It does not read device traces. It exists because the instrument #674 asked for did
not exist on any branch, so the issue could only be answered by opinion.

Models
  champion     static ladder ordered by a prior reliability score, exponential
               backoff between attempts, capped at MAX_ATTEMPTS
  bandit       UCB1 over peers, updated from observed dial outcomes, same backoff
               and same attempt cap
  oracle       upper bound used only for regret accounting (dial only good peers)

Pre-registered acceptance (fixed before the first run, never adjusted after):
  a challenger ships only if
    success_rate >= champion.success_rate - 0.01   (reliability cannot regress)
    and attempts_per_connection <= champion.attempts_per_connection * 0.95
No other outcome changes a dial policy.

Exit status: 0 when every model runs and the report is produced; 1 when the
instrument's own self-check fails or the table is empty.
"""

from __future__ import annotations

import argparse
import math
import random
import statistics
import sys

MAX_ATTEMPTS = 3
BACKOFF_BASE_MS = 250
BASE_LATENCY_MS = 40.0
RELIABILITY_FLOOR = 0.01
ATTEMPT_TARGET = 0.95


class Peer:
    __slots__ = ("pid", "p_success", "latency_ms", "prior_score", "up", "pulls", "wins", "lat_sum")

    def __init__(self, pid: int, p_success: float, latency_ms: float, prior_score: float):
        self.pid = pid
        self.p_success = p_success
        self.latency_ms = latency_ms
        self.prior_score = prior_score
        self.up = True
        self.pulls = 0
        self.wins = 0
        self.lat_sum = 0.0

    def observed_mean(self) -> float:
        return (self.wins + 1.0) / (self.pulls + 2.0)

    def observed_latency(self) -> float:
        return self.lat_sum / self.pulls if self.pulls else 0.0


def build_population(rng: random.Random, n: int) -> list[Peer]:
    """Bimodal population: most peers unreliable, a minority solid. Prior scores are
    deliberately mis-ranked relative to ground truth so a static ladder has something
    to be wrong about."""
    peers = []
    for pid in range(n):
        good = rng.random() < 0.25
        p = rng.uniform(0.70, 0.95) if good else rng.uniform(0.05, 0.40)
        lat = rng.gauss(80.0 if good else 160.0, 25.0)
        prior = rng.uniform(0.0, 1.0)  # uninformative prior: the ladder starts blind
        peers.append(Peer(pid, p, max(5.0, lat), prior))
    return peers


def build_churn_trace(rng: random.Random, n_peers: int, epochs: int) -> list[list[bool]]:
    """One availability snapshot per epoch, computed once and replayed by every policy.

    #711: churn used to draw from the same `random.Random` a policy dials with. A
    policy that dials more often therefore consumes more of that stream, so at one
    seed each policy saw a DIFFERENT availability history - the comparison measured
    the RNG's response to the policy, not the policy. The trace is built from its own
    seeded generator and never sees dial traffic, so every model replays identical
    availability and differs only in the order it dials.
    """
    trace: list[list[bool]] = []
    for epoch in range(epochs):
        if epoch % 5 == 0:
            trace.append([rng.random() > 0.25 for _ in range(n_peers)])
        else:
            trace.append(list(trace[-1]) if trace else [True] * n_peers)
    return trace


def apply_churn(peers: list[Peer], snapshot: list[bool]) -> None:
    """Set each peer's availability from the precomputed snapshot for this epoch."""
    for peer, up in zip(peers, snapshot):
        peer.up = up


def dial_once(rng: random.Random, peer: Peer) -> tuple[bool, float]:
    if not peer.up:
        return False, 0.0
    ok = rng.random() < peer.p_success
    return ok, peer.latency_ms if ok else 0.0


def ucb_explore(total_pulls: int, pulls: int) -> float:
    """UCB1's exploration term: sqrt(2 * ln(total) / pulls).

    Logarithmic in the total number of pulls and inverse-square-root in this
    peer's own, which is the only form that decays: as a peer is measured, its
    bonus falls away and the policy settles on what it has learned. The latency
    arm used 2 * sqrt(total) / sqrt(pulls) instead - sqrt of the total, not its
    log - so its exploration kept GROWING with every dial anywhere in the pool
    (#746), and the arm spent the budget re-probing peers it had already measured
    while the plain arm settled. A local build is never used for this: the
    pinned values in self_check are the evidence.
    """
    if pulls == 0:
        return float("inf")  # explore unseen peers first
    return math.sqrt(2.0 * math.log(max(2, total_pulls)) / pulls)


def run_static(
    peers: list[Peer],
    trace: list[list[bool]],
    rng: random.Random,
    epochs: int,
    seen: list[int] | None = None,
) -> dict:
    """Champion: ladder frozen in prior-score order, exponential backoff, cap."""
    ladder = sorted(peers, key=lambda p: -p.prior_score)
    attempts = successes = wasted = 0
    latencies = []
    for _ in range(epochs):
        apply_churn(peers, trace[_])
        if seen is not None:
            seen.append(sum(1 for p in peers if p.up))
        for attempt in range(MAX_ATTEMPTS):
            peer = ladder[attempt % len(ladder)]
            ok, lat = dial_once(rng, peer)
            attempts += 1
            if ok:
                successes += 1
                latencies.append(lat + BACKOFF_BASE_MS * (2**attempt - 1))
                break
            wasted += 1
    return summarize(successes, attempts, wasted, latencies, epochs)


def run_bandit(
    peers: list[Peer],
    trace: list[list[bool]],
    rng: random.Random,
    epochs: int,
    seen: list[int] | None = None,
) -> dict:
    """Challenger: UCB1 ordering refreshed after every dial outcome."""
    attempts = successes = wasted = 0
    latencies = []
    for _ in range(epochs):
        apply_churn(peers, trace[_])
        if seen is not None:
            seen.append(sum(1 for p in peers if p.up))
        for attempt in range(MAX_ATTEMPTS):
            total = sum(p.pulls for p in peers) + 1
            def ucb(p: Peer) -> float:
                return p.observed_mean() + ucb_explore(total, p.pulls)
            peer = max(peers, key=ucb)
            ok, lat = dial_once(rng, peer)
            peer.pulls += 1
            attempts += 1
            if ok:
                peer.wins += 1
                successes += 1
                latencies.append(lat + BACKOFF_BASE_MS * (2**attempt - 1))
                break
            wasted += 1
    return summarize(successes, attempts, wasted, latencies, epochs)


def run_bandit_latency(
    peers: list[Peer],
    trace: list[list[bool]],
    rng: random.Random,
    epochs: int,
    seen: list[int] | None = None,
) -> dict:
    """Challenger 2: UCB1 with an explicit latency penalty, so exploration does not
    buy wins with slow peers. Without this row the bake-off would recommend a policy
    that shortens the ladder while raising tail latency - a trade it never measured."""
    attempts = successes = wasted = 0
    latencies = []
    for _ in range(epochs):
        apply_churn(peers, trace[_])
        if seen is not None:
            seen.append(sum(1 for p in peers if p.up))
        for attempt in range(MAX_ATTEMPTS):
            total = sum(p.pulls for p in peers) + 1
            def ucb(p: Peer) -> float:
                # Same logarithmic exploration as the plain arm; only the latency
                # penalty distinguishes the two challengers.
                explore = ucb_explore(total, p.pulls)
                lat_penalty = 0.25 * (p.observed_latency() / BASE_LATENCY_MS)
                return p.observed_mean() + explore - lat_penalty
            peer = max(peers, key=ucb)
            ok, lat = dial_once(rng, peer)
            peer.pulls += 1
            peer.lat_sum += lat
            attempts += 1
            if ok:
                peer.wins += 1
                successes += 1
                latencies.append(lat + BACKOFF_BASE_MS * (2**attempt - 1))
                break
            wasted += 1
    return summarize(successes, attempts, wasted, latencies, epochs)


def summarize(successes: int, attempts: int, wasted: int, latencies: list[float], epochs: int) -> dict:
    return {
        "success_rate": successes / epochs if epochs else 0.0,
        "attempts": attempts,
        "attempts_per_connection": attempts / successes if successes else float("inf"),
        "wasted": wasted,
        "p95_ms": statistics.quantiles(latencies, n=20)[-1] if len(latencies) >= 20 else (max(latencies) if latencies else 0.0),
    }


def self_check() -> bool:
    """Known-answer check: a perfect ordering must waste fewer probes than a reversed
    one on a fixed seed. If this fails the model is not measuring ordering at all."""
    rng = random.Random(6741)
    peers = build_population(rng, 32)
    good = sorted(peers, key=lambda p: -p.p_success)
    bad = list(reversed(good))

    def probes(order: list[Peer]) -> int:
        r = random.Random(6742)
        w = 0
        for _ in range(120):
            for _attempt in range(MAX_ATTEMPTS):
                # Ranked traversal, not a random index: a uniformly random pick
                # is blind to ordering, so the self-check could not tell a ranked
                # policy from an unranked one.
                p = order[min(_attempt, len(order) - 1)]
                ok, _lat = dial_once(r, p)
                if ok:
                    break
                w += 1
        return w

    if not probes(good) < probes(bad):
        return False

    # #711: two policies at one seed must replay the SAME availability history. Before
    # the trace existed both policies drew churn from the RNG they dial with, so the
    # busier policy consumed more of it and the histories diverged.
    n_peers, epochs = 24, 60
    trace = build_churn_trace(random.Random(6743), n_peers, epochs)
    static_seen: list[int] = []
    bandit_seen: list[int] = []
    run_static(build_population(random.Random(6744), n_peers), trace, random.Random(6745), epochs, static_seen)
    run_bandit(build_population(random.Random(6744), n_peers), trace, random.Random(6745), epochs, bandit_seen)
    if static_seen != bandit_seen:
        print("[FAIL] policies replayed different availability histories at one seed")
        return False

    # #746: both UCB arms must use the LOGARITHMIC exploration term. Pinned at a
    # state the two forms separate by a factor of 17: at total=1000, pulls=10 the
    # log form gives 1.1754 and the sqrt(total) form the latency arm carried gives
    # 20.0, so this check fails on the pre-fix control flow and keeps the arms from
    # drifting apart again.
    checks = (((1000, 10), 1.1754), ((1000, 100), 0.3717))
    for (total, pulls), expected in checks:
        got = ucb_explore(total, pulls)
        if abs(got - expected) > 1e-3:
            print(f"[FAIL] ucb_explore({total}, {pulls}) = {got:.4f}, expected {expected} (logarithmic)")
            return False
    if ucb_explore(10, 0) != float("inf"):
        print("[FAIL] an unpulled peer must be explored first")
        return False
    # Exploration must fall as a peer is measured; sqrt(total) never does.
    if not ucb_explore(1000, 100) < ucb_explore(1000, 10):
        print("[FAIL] exploration does not decay with a peer's own pulls")
        return False
    return True


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--trials", type=int, default=30)
    ap.add_argument("--epochs", type=int, default=200)
    ap.add_argument("--peers", type=int, default=64)
    ap.add_argument("--seed", type=int, default=674)
    args = ap.parse_args()

    if not self_check():
        print("[FAIL] self-check: ordering had no effect on probes; model is invalid")
        return 1

    rows = []
    for t in range(args.trials):
        seed = args.seed + t
        trace = build_churn_trace(random.Random(seed), args.peers, args.epochs)
        static = run_static(
            build_population(random.Random(seed), args.peers), trace, random.Random(seed), args.epochs
        )
        bandit = run_bandit(
            build_population(random.Random(seed), args.peers), trace, random.Random(seed), args.epochs
        )
        bandit_lat = run_bandit_latency(
            build_population(random.Random(seed), args.peers), trace, random.Random(seed), args.epochs
        )
        rows.append((static, bandit, bandit_lat))

    def mean(rows, model, key):
        vals = [r[model][key] for r in rows]
        return sum(vals) / len(vals) if key != "p95_ms" else statistics.median(vals)

    print(f"dial-policy bake-off: peers={args.peers} epochs={args.epochs} trials={args.trials} seed={args.seed}")
    print(f"{'model':<10}{'success':>10}{'attempts/conn':>16}{'wasted':>10}{'p95_ms':>10}")
    for idx, name in ((0, "static"), (1, "bandit"), (2, "bandit-lat")):
        print(
            f"{name:<10}{mean(rows, idx, 'success_rate'):>10.3f}"
            f"{mean(rows, idx, 'attempts_per_connection'):>16.3f}"
            f"{mean(rows, idx, 'wasted'):>10.1f}"
            f"{mean(rows, idx, 'p95_ms'):>10.1f}"
        )

    s = mean(rows, 0, "success_rate")
    sa = mean(rows, 0, "attempts_per_connection")
    verdicts = []
    for idx, name in ((1, "bandit"), (2, "bandit-lat")):
        c = mean(rows, idx, "success_rate")
        ca = mean(rows, idx, "attempts_per_connection")
        reliable = c >= s - RELIABILITY_FLOOR
        cheaper = ca <= sa * ATTEMPT_TARGET
        verdicts.append((name, c - s, ca - sa, reliable and cheaper))
        print()
        print(f"[INFO] {name} vs static: success delta {c - s:+.3f}, attempts/conn delta {ca - sa:+.3f}")
    winners = [name for name, _ds, _da, ok in verdicts if ok]
    if winners:
        print(f"[OK] RESULT: {', '.join(winners)} PASS(ES) the pre-registered acceptance")
        return 0
    print("[FAIL] RESULT: no challenger passes - champion stands")
    return 1


if __name__ == "__main__":
    sys.exit(main())
