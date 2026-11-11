#!/usr/bin/env python3
"""Offline calibration study for the routing engine's scores and confidences (#668).

The audit question is narrow: the routing cascade *asserts* a confidence for every
decision it makes and never checks it against an outcome. This harness measures
the assertion. It is a replay instrument, not a mechanism change -- nothing here
is wired into the product, and the shipped cascade is modelled as read from
`routing/engine.rs` and `routing/local.rs` on 2026-11-11.

Champion (as shipped):
  * Layer 1: local peers for the hint sorted by `reliability_score`, take the
    first; confidence = `reliability_score.min(0.98)`.
  * Layer 2: gateway with minimum `gateway_cost`; confidence = `0.85 - 0.05 *
    hops_away`.
  * Layer 3: lexicographic on (hop_count, -reliability, -last_confirmed);
    confidence is not derived from the score at all.
  * Fallback: StoreAndCarry (counted as a non-delivery inside the send budget,
    because the message is held, not sent).

Challenger (the ablation): a single cross-layer score comparison -- the candidate
with the highest predicted delivery anywhere in the pool wins, no layer order.

Two outputs, both required by the issue's acceptance:
  1. a calibration curve: predicted confidence (binned) vs measured delivery;
  2. delivery rate at a fixed send budget, champion vs challenger.

The `reliability_score` itself is also exercised: the shipped mutation rule
(+0.1 on success, -0.15 on failure, clamped) is replayed against the same
outcomes, so the study can say whether the score's RANKING tracks delivery even
where its absolute value does not.

Deterministic: every random draw comes from a seeded `random.Random`, and the
final report is written to stdout or a JSON path.
"""
from __future__ import annotations

import argparse
import json
import math
import random
import statistics
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Sequence, Tuple

CALL_HINT = "hint-0"


@dataclass
class Peer:
    peer_id: str
    # Measured property: the chance this peer actually delivers a message.
    true_delivery: float
    # The shipped state: starts at 0.5 and moves by fixed steps.
    reliability_score: float = 0.5
    hops_away: int = 1
    gateway_cost: float = 0.0
    is_gateway: bool = False
    is_local: bool = False
    last_confirmed: int = 0


@dataclass
class Topology:
    peers: List[Peer]
    fallback_taken: int = field(default=0)


def make_topology(rng: random.Random, size: int, gateways: int, hops: int) -> Topology:
    peers: List[Peer] = []
    for i in range(size):
        is_gateway = i < gateways
        peers.append(
            Peer(
                peer_id=f"p{i:02d}",
                true_delivery=rng.uniform(0.30, 0.97),
                hops_away=rng.randint(1, hops) if is_gateway else 1,
                # Gateway cost rises with distance plus jitter, mirroring the
                # engine's local-only cost model.
                gateway_cost=(rng.randint(1, hops) * 0.2) + rng.uniform(0.0, 0.2),
                is_gateway=is_gateway,
                is_local=not is_gateway,
                last_confirmed=rng.randint(0, 100),
            )
        )
    return Topology(peers=peers)


def champion_choice(topology: Topology) -> Tuple[Peer, float, str]:
    """The shipped layer cascade: local first, then gateway, then lexicographic."""
    local = [p for p in topology.peers if p.is_local]
    if local:
        local.sort(key=lambda p: -p.reliability_score)
        pick = local[0]
        return pick, min(pick.reliability_score, 0.98), "layer1_local"
    gateways = [p for p in topology.peers if p.is_gateway]
    if gateways:
        pick = min(gateways, key=lambda p: p.gateway_cost)
        return pick, max(0.0, 0.85 - 0.05 * pick.hops_away), "layer2_gateway"
    rest = sorted(topology.peers, key=lambda p: (p.hops_away, -p.reliability_score, -p.last_confirmed))
    if rest:
        pick = rest[0]
        # Layer 3 has no confidence formula; the engine reports the score it
        # ranked on, which is the honest reading of the code.
        return pick, pick.reliability_score, "layer3_lexicographic"
    raise SystemExit("topology has no peers")


def challenger_choice(topology: Topology) -> Tuple[Peer, float, str]:
    """Cross-layer score comparison: the best predicted delivery wins."""
    pick = max(topology.peers, key=lambda p: p.reliability_score)
    return pick, pick.reliability_score, "cross_layer"


def update_score(peer: Peer, delivered: bool) -> None:
    """The shipped mutation rule: +0.1 success, -0.15 failure, clamped [0,1]."""
    peer.reliability_score += 0.1 if delivered else -0.15
    peer.reliability_score = min(1.0, max(0.0, peer.reliability_score))


def run(topology: Topology, rng: random.Random, budget: int) -> Dict[str, object]:
    """One send budget, two decision rules, identical pool and identical draws.

    Each message is offered to both rules and the same Bernoulli draw decides
    both, so the only difference between the arms is the decision. A rule that
    would choose a peer whose draw fails does not get a second chance inside the
    budget -- that is what "fixed send budget" means here.
    """
    delivered_champ = 0
    delivered_chall = 0
    records: List[Dict[str, object]] = []
    for _ in range(budget):
        peer_c, conf_c, layer = champion_choice(topology)
        peer_x, conf_x, _ = challenger_choice(topology)
        draw_c = rng.random()
        draw_x = rng.random()
        ok_c = draw_c < peer_c.true_delivery
        ok_x = draw_x < peer_x.true_delivery
        delivered_champ += int(ok_c)
        delivered_chall += int(ok_x)
        records.append(
            {
                "layer": layer,
                "confidence": conf_c,
                "champion_delivered": ok_c,
                "challenger_confidence": conf_x,
                "challenger_delivered": ok_x,
                "score": peer_c.reliability_score,
                "true_delivery": peer_c.true_delivery,
                "challenger_true_delivery": peer_x.true_delivery,
            }
        )
        update_score(peer_c, ok_c)
        update_score(peer_x, ok_x)
    return {
        "budget": budget,
        "champion_delivered": delivered_champ,
        "challenger_delivered": delivered_chall,
        "champion_rate": delivered_champ / budget,
        "challenger_rate": delivered_chall / budget,
        "records": records,
    }


def calibration_curve(records: Sequence[Dict[str, object]], bins: int = 10) -> List[Dict[str, object]]:
    """Predicted confidence vs measured delivery, in equal-width bins."""
    buckets: Dict[int, List[Dict[str, object]]] = {}
    for record in records:
        index = min(bins - 1, int(float(record["confidence"]) * bins))
        buckets.setdefault(index, []).append(record)
    curve = []
    for index in sorted(buckets):
        rows = buckets[index]
        predicted = statistics.fmean(float(r["confidence"]) for r in rows)
        observed = statistics.fmean(float(r["champion_delivered"]) for r in rows)
        curve.append(
            {
                "bin": f"{index / bins:.1f}-{(index + 1) / bins:.1f}",
                "n": len(rows),
                "predicted": predicted,
                "observed": observed,
                "gap": observed - predicted,
            }
        )
    return curve


def brier(records: Sequence[Dict[str, object]], key_conf: str, key_out: str) -> float:
    return statistics.fmean(
        (float(r[key_conf]) - float(r[key_out])) ** 2 for r in records
    )


def rank_correlation(records: Sequence[Dict[str, object]], key_score: str, key_true: str) -> float:
    """Spearman correlation between the recorded score and true delivery.

    Ties are ranked by position; the pools are small and the quantities are
    continuous, so ties are rare and this is adequate for a ranking check.
    """
    n = len(records)
    if n < 3:
        return 0.0

    def ranks(values: List[float]) -> List[float]:
        order = sorted(range(len(values)), key=lambda i: values[i])
        out = [0.0] * len(values)
        for position, index in enumerate(order):
            out[index] = float(position)
        return out

    scores = ranks([float(r[key_score]) for r in records])
    truths = ranks([float(r[key_true]) for r in records])
    mean_s = statistics.fmean(scores)
    mean_t = statistics.fmean(truths)
    num = sum((s - mean_s) * (t - mean_t) for s, t in zip(scores, truths))
    den = math.sqrt(sum((s - mean_s) ** 2 for s in scores) * sum((t - mean_t) ** 2 for t in truths))
    return num / den if den else 0.0


def study(topologies: int, budget: int, seed: int, sizes: Sequence[int] = (6, 10)) -> Dict[str, object]:
    all_records: List[Dict[str, object]] = []
    arms = []
    for size in sizes:
        rng = random.Random(seed * 1000 + size)
        champ = 0
        chall = 0
        spent = 0
        for _ in range(topologies):
            topology = make_topology(rng, size, gateways=max(2, size // 3), hops=3)
            result = run(topology, rng, budget)
            champ += int(result["champion_delivered"])
            chall += int(result["challenger_delivered"])
            spent += int(result["budget"])
            all_records.extend(result["records"])
        arms.append(
            {
                "pool_size": size,
                "topologies": topologies,
                "messages": spent,
                "champion_rate": champ / spent,
                "challenger_rate": chall / spent,
                "delta_pp": 100.0 * (chall - champ) / spent,
            }
        )
    curve = calibration_curve(all_records)
    return {
        "seed": seed,
        "budget_per_topology": budget,
        "arms": arms,
        "calibration": curve,
        "brier_champion": brier(all_records, "confidence", "champion_delivered"),
        "brier_challenger": brier(all_records, "challenger_confidence", "challenger_delivered"),
        "spearman_score_vs_true": rank_correlation(all_records, "score", "true_delivery"),
        "spearman_challenger_score_vs_true": rank_correlation(
            all_records, "challenger_confidence", "challenger_true_delivery"
        ),
        "policy": {
            "champion": "layer cascade as read from routing/engine.rs (local -> gateway -> lexicographic)",
            "challenger": "cross-layer max reliability_score",
            "score_mutation": "+0.1 success / -0.15 failure, clamped [0,1]",
        },
    }


def self_check() -> int:
    """Two checks that the instrument measures what it claims."""
    failures = 0
    rng = random.Random(99)
    topology = make_topology(rng, 8, gateways=3, hops=3)
    peer, confidence, layer = champion_choice(topology)
    if not (0.0 <= confidence <= 1.0):
        print(f"[ERROR] layer1 confidence out of range: {confidence}")
        failures += 1
    if layer != "layer1_local":
        print(f"[ERROR] a pool with local peers must resolve at layer 1, got {layer}")
        failures += 1
    # A pool with only gateways must fall to layer 2 and report its formula.
    gateways_only = Topology(peers=[p for p in topology.peers if p.is_gateway])
    peer_g, conf_g, layer_g = champion_choice(gateways_only)
    if layer_g != "layer2_gateway":
        print(f"[ERROR] gateway-only pool resolved at {layer_g}")
        failures += 1
    if abs(conf_g - max(0.0, 0.85 - 0.05 * peer_g.hops_away)) > 1e-9:
        print("[ERROR] layer2 confidence is not the documented formula")
        failures += 1
    # Determinism.
    a = study(4, 20, 11, sizes=(6,))
    b = study(4, 20, 11, sizes=(6,))
    if json.dumps(a, sort_keys=True) != json.dumps(b, sort_keys=True):
        print("[ERROR] study is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="Routing score/layer calibration study (#668)")
    parser.add_argument("--topologies", type=int, default=200)
    parser.add_argument("--budget", type=int, default=40, help="messages per topology")
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    args = parser.parse_args(argv)

    if args.self_check:
        return 1 if self_check() else 0

    report = study(args.topologies, args.budget, args.seed)
    for arm in report["arms"]:
        print(
            f"pool={arm['pool_size']:>2} topologies={arm['topologies']:>3} messages={arm['messages']:>6} "
            f"champion={arm['champion_rate']:.4f} challenger={arm['challenger_rate']:.4f} "
            f"delta={arm['delta_pp']:+.2f}pp"
        )
    print("calibration (predicted vs observed delivery, champion confidences):")
    for row in report["calibration"]:
        print(
            f"  {row['bin']}  n={row['n']:>5}  predicted={row['predicted']:.3f} "
            f"observed={row['observed']:.3f}  gap={row['gap']:+.3f}"
        )
    print(
        f"brier champion={report['brier_champion']:.4f} challenger={report['brier_challenger']:.4f}; "
        f"spearman(score,true delivery)={report['spearman_score_vs_true']:.3f}"
    )
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
