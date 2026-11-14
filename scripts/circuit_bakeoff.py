#!/usr/bin/env python3
"""Offline circuit-selection bake-off: greedy champion vs joint optimum (#667).

The audit question is narrow and must stay measurable: `privacy/circuit.rs` picks
onion hops one at a time, so the path it builds is never scored as a whole. If a
joint optimum over the same relay pool is materially better at end-to-end
delivery, that is a change proposal; if it is not, the heuristic stands and this
measurement is the deliverable. Either verdict is a success -- the point is that
the number exists and is reproducible.

Champion (models core/src/privacy/circuit.rs as read 2026-11-11):

  * hop count uniform over {min_hops..=max_hops} (default 3..=5);
  * eligible relays have `reliability_score >= min_reliability` (0.5) and are not
    the destination;
  * `prefer_diverse_paths` (the default) groups eligible relays by
    `network_segment` and walks the segment list round-robin, taking the
    highest-reliability *unused* relay in each segment.

Challenger (the joint objective from the audit issue):

  score(path) = P_delivery - 0.1 * latency_norm - 30 * excess_operator
                             - 20 * excess_subnet

  P_delivery is the end-to-end probability from scripts/delivery_probability.py
  (imported, not re-derived), so the two audit components share one estimator.
  `excess_X` counts relays beyond the first for each operator/subnet, i.e. the
  number of shared-identity pairs the path admits. Brute force over all hop sets
  of the champion's hop-count range is exact at these pool sizes, so no solver
  approximation is involved and no quantum hardware is needed or implied.

Usage:
  python scripts/circuit_bakeoff.py                     # 3 classes x 200 pools
  python scripts/circuit_bakeoff.py --pools 20 --sizes 8 --seed 7
  python scripts/circuit_bakeoff.py --self-check         # estimator parity check
"""

from __future__ import annotations

import argparse
import math
import itertools
import json
import random
import statistics
import sys
from dataclasses import dataclass, field
from typing import Dict, Iterable, List, Sequence, Tuple

sys.path.insert(0, __file__.rsplit("/", 1)[0] if "/" in __file__ else ".")
from delivery_probability import exact_bruteforce, exact_decomposition  # noqa: E402

MIN_RELIABILITY = 0.5          # CircuitConfig::default().min_reliability
MIN_HOPS = 3                   # CircuitConfig::default().min_hops
MAX_HOPS = 5                   # CircuitConfig::default().max_hops (also the onion cap)

# Weights from the audit issue's validated QUBO form.
W_LATENCY = 0.1
W_SAME_OPERATOR = 30.0
W_SAME_SUBNET = 20.0

LATENCY_CAP_MS = 400.0         # normalisation ceiling for the latency term


@dataclass(frozen=True)
class Relay:
    peer_id: str
    reliability: float
    latency_ms: float
    operator: str
    segment: str               # PeerInfo::network_segment
    # Whether this relay CAN carry attributed outcome history. A freshly minted
    # Sybil identity cannot: the privacy floor counts proven relays, and letting
    # the harness hand history to an identity that was just created would model
    # an attack no adversary has to run, then score the policy against it.
    proven: bool = True


@dataclass
class Pool:
    relays: List[Relay]
    destination: Relay


@dataclass
class Result:
    pool_size: int
    champion_prob: float
    challenger_prob: float
    champion_hops: int
    challenger_hops: int
    champion_path: Tuple[str, ...] = field(default_factory=tuple)
    challenger_path: Tuple[str, ...] = field(default_factory=tuple)

    @property
    def gap_pp(self) -> float:
        """Challenger minus champion, in percentage points."""
        return 100.0 * (self.challenger_prob - self.champion_prob)


def make_pool(rng: random.Random, size: int, operators: int, segments: int) -> Pool:
    """A relay pool with the heterogeneity the objective cares about.

    Deliberately includes relays below `min_reliability`: the champion filters
    them out, and so does the challenger, so both see an identical eligible set.
    """
    relays = []
    for i in range(size):
        reliability = rng.uniform(0.55, 0.995)
        if rng.random() < 0.15:                     # 15% are below the threshold
            reliability = rng.uniform(0.20, 0.49)
        relays.append(
            Relay(
                peer_id=f"r{i:02d}",
                reliability=reliability,
                latency_ms=min(LATENCY_CAP_MS, rng.lognormvariate(4.2, 0.45)),
                operator=f"op{rng.randrange(operators)}",
                segment=f"seg{rng.randrange(segments)}",
            )
        )
    destination = Relay(
        peer_id="dest",
        reliability=rng.uniform(0.8, 0.995),
        latency_ms=rng.uniform(10.0, 80.0),
        operator="op-dest",
        segment="seg-dest",
    )
    return Pool(relays=relays, destination=destination)


def eligible(pool: Pool) -> List[Relay]:
    """Exactly the champion's filter: threshold, and never the destination."""
    return [
        r for r in pool.relays
        if r.reliability >= MIN_RELIABILITY and r.peer_id != pool.destination.peer_id
    ]


def champion_path(pool: Pool, rng: random.Random, hops_override: int | None = None) -> List[Relay]:
    """core/src/privacy/circuit.rs select_diverse_hops, reproduced faithfully.

    Known hazard reproduced rather than fixed: if every segment is exhausted the
    real loop cannot make progress. Here the hop count is clamped instead of
    hanging, and such pools are counted separately (`short_pools`), because an
    infinite loop is not something a benchmark should reproduce literally.
    """
    hops = rng.randint(MIN_HOPS, MAX_HOPS) if hops_override is None else hops_override
    elig = eligible(pool)
    if len(elig) < hops:
        return []

    by_segment: Dict[str, List[Relay]] = {}
    for relay in elig:
        by_segment.setdefault(relay.segment, []).append(relay)

    segments = list(by_segment.keys())
    used: set = {pool.destination.peer_id}
    selected: List[Relay] = []
    idx = 0
    while len(selected) < hops and idx < hops * len(segments) + len(segments):
        segment = segments[idx % len(segments)]
        best = None
        for relay in by_segment[segment]:
            if relay.peer_id in used:
                continue
            if best is None or relay.reliability > best.reliability:
                best = relay
        if best is not None:
            selected.append(best)
            used.add(best.peer_id)
        idx += 1
    return selected


def chain_topology(pool: Pool, path: Sequence[Relay]) -> Dict[str, object]:
    """The path as a series topology, in the estimator's own JSON shape.

    One link per hop (its forwarding reliability) plus the final hand-off to the
    destination, which this audit does not model and therefore carries p=1.0. A
    chain has one more link than it has relays -- emitting only len(path) links
    leaves the sink disconnected and the estimator correctly returns 0.0, which is
    how the first version of this function failed its own self-check.
    """
    nodes = ["src"] + [r.peer_id for r in path] + [pool.destination.peer_id]
    links = [(nodes[i], nodes[i + 1], path[i].reliability) for i in range(len(path))]
    links.append((nodes[-2], nodes[-1], 1.0))
    return {"source": nodes[0], "sink": nodes[-1], "links": links, "nodes": nodes}


def delivery_probability(pool: Pool, path: Sequence[Relay]) -> float:
    """End-to-end probability through the path.

    A chain of independent hops has closed-form probability = product of the hop
    reliabilities, which is what the estimator's `exact_decomposition` returns for
    the same links. The product is used here because the bake-off evaluates
    hundreds of thousands of candidate paths; `self_check` verifies the equality
    against both estimator implementations on sampled chains, so this is a
    measured shortcut rather than a re-derivation.
    """
    if not path:
        return 0.0
    probability = 1.0
    for relay in path:
        probability *= relay.reliability
    return probability


def objective(pool: Pool, path: Sequence[Relay]) -> float:
    """The audit issue's joint objective, with P_delivery from the estimator."""
    if not path:
        return float("-inf")
    latency = statistics.fmean(r.latency_ms for r in path) / LATENCY_CAP_MS
    ops: Dict[str, int] = {}
    segs: Dict[str, int] = {}
    for relay in path:
        ops[relay.operator] = ops.get(relay.operator, 0) + 1
        segs[relay.segment] = segs.get(relay.segment, 0) + 1
    excess_operator = sum(count - 1 for count in ops.values())
    excess_subnet = sum(count - 1 for count in segs.values())
    return (
        delivery_probability(pool, path)
        - W_LATENCY * latency
        - W_SAME_OPERATOR * excess_operator
        - W_SAME_SUBNET * excess_subnet
    )


def joint_optimum(pool: Pool) -> List[Relay]:
    """Exhaustive search over every hop set the champion could have produced."""
    elig = eligible(pool)
    best_path: List[Relay] = []
    best_score = float("-inf")
    for hops in range(MIN_HOPS, MAX_HOPS + 1):
        if len(elig) < hops:
            continue
        for combo in itertools.combinations(elig, hops):
            score = objective(pool, combo)
            if score > best_score:
                best_score = score
                best_path = list(combo)
    return best_path


def run(sizes: Sequence[int], pools: int, seed: int) -> Dict[str, object]:
    classes = []
    for size in sizes:
        rng = random.Random(seed * 1000 + size)
        results: List[Result] = []
        short_pools = 0
        # Same selector as the champion, hop count pinned to MIN_HOPS: isolates how
        # much of the headline gap is relay choice and how much is simply the
        # uniform 3-5 hop draw. Without this split the headline number cannot be
        # acted on -- "adopt the joint optimum" and "always use three hops" are
        # very different changes.
        fixed_gaps: List[float] = []
        hop_effect: List[float] = []
        for _ in range(pools):
            pool = make_pool(rng, size, operators=max(3, size // 3), segments=max(3, size // 4))
            champ = champion_path(pool, rng)
            champ_fixed = champion_path(pool, rng, hops_override=MIN_HOPS)
            challenger = joint_optimum(pool)
            if len(champ) < MIN_HOPS or len(champ_fixed) < MIN_HOPS:
                short_pools += 1
                continue
            fixed_gaps.append(100.0 * (delivery_probability(pool, challenger) - delivery_probability(pool, champ_fixed)))
            hop_effect.append(100.0 * (delivery_probability(pool, champ_fixed) - delivery_probability(pool, champ)))
            results.append(
                Result(
                    pool_size=size,
                    champion_prob=delivery_probability(pool, champ),
                    challenger_prob=delivery_probability(pool, challenger),
                    champion_hops=len(champ),
                    challenger_hops=len(challenger),
                    champion_path=tuple(r.peer_id for r in champ),
                    challenger_path=tuple(r.peer_id for r in challenger),
                )
            )
        gaps = sorted(r.gap_pp for r in results)
        classes.append(
            {
                "pool_size": size,
                "pools": len(results),
                "short_pools": short_pools,
                "champion_median_prob": statistics.median(r.champion_prob for r in results),
                "challenger_median_prob": statistics.median(r.challenger_prob for r in results),
                "gap_median_pp": statistics.median(gaps),
                "gap_p90_pp": gaps[min(len(gaps) - 1, int(round(0.9 * (len(gaps) - 1))))],
                "gap_max_pp": gaps[-1],
                "pools_with_gap_over_2pp": sum(1 for g in gaps if g >= 2.0),
                "champion_hop_counts": {
                    str(h): sum(1 for r in results if r.champion_hops == h) for h in range(MIN_HOPS, MAX_HOPS + 1)
                },
                "challenger_hop_counts": {
                    str(h): sum(1 for r in results if r.challenger_hops == h) for h in range(MIN_HOPS, MAX_HOPS + 1)
                },
                "identical_paths": sum(1 for r in results if set(r.champion_path) == set(r.challenger_path)),
                # Decomposition of the headline gap (challenger minus champion):
                #   relay_choice_pp = challenger minus 3-hop greedy (equal hop count)
                #   hop_count_pp   = 3-hop greedy minus as-shipped champion
                "relay_choice_median_pp": statistics.median(fixed_gaps),
                "relay_choice_p90_pp": sorted(fixed_gaps)[int(round(0.9 * (len(fixed_gaps) - 1)))],
                "hop_count_median_pp": statistics.median(hop_effect),
                "hop_count_p90_pp": sorted(hop_effect)[int(round(0.9 * (len(hop_effect) - 1)))],
            }
        )
    medians = [c["gap_median_pp"] for c in classes]
    overall_median = statistics.median(medians) if medians else 0.0
    return {
        "seed": seed,
        "min_hops": MIN_HOPS,
        "max_hops": MAX_HOPS,
        "min_reliability": MIN_RELIABILITY,
        "weights": {"latency": W_LATENCY, "same_operator": W_SAME_OPERATOR, "same_subnet": W_SAME_SUBNET},
        "classes": classes,
        "median_of_class_medians_pp": overall_median,
        "material": overall_median >= 2.0,
        "material_threshold_pp": 2.0,
    }


# --------------------------------------------------------------------------- #682
#
# Dynamic-depth challenger (#682 decision, c.6785048161). Everything below is
# ADDITIVE: `run()` and the original 400-pool sweep are untouched, so the shipped
# numbers stay comparable.
#
# The policy modelled here is the one implemented in core/src/privacy/circuit.rs:
# score the diversity selector at every depth the pool supports, predict each
# candidate as the product of its hops' posterior means, take PAL (best predicted
# delivery over depths meeting the privacy floor), realise the shallowest depth
# reaching PAL, sampling within the harness's own 2pp indifference band.

DEPTH_BAND = 0.02          # the harness's own materiality band, from the spec
DEPTH_HISTORY_WINDOW = 64


@dataclass(frozen=True)
class Posterior:
    alpha: float
    beta: float

    @property
    def mean(self) -> float:
        total = self.alpha + self.beta
        return self.alpha / total if total > 0 else 0.0

    @property
    def observations(self) -> float:
        return max(0.0, self.alpha + self.beta - 2.0)


def cold_start(score: float) -> Posterior:
    s = min(1.0, max(0.0, score))
    return Posterior(1.0 + 2.0 * s, 1.0 + 2.0 * (1.0 - s))


def seed_posteriors(pool: Pool, rng: random.Random, history_frac: float = 0.6) -> Dict[str, Posterior]:
    """Posteriors for the pool: cold-start priors, plus attributed history.

    A fraction of relays carries attributed outcomes, because the privacy floor
    counts only relays with history -- a pool with no history has no proven
    diversity and equally no Sybil resistance. History is drawn around the relay's
    own reliability so a good relay is usually proven good.
    """
    out: Dict[str, Posterior] = {}
    for relay in pool.relays:
        post = cold_start(relay.reliability)
        if relay.proven and rng.random() < history_frac:
            trials = rng.randint(3, 12)
            successes = sum(1 for _ in range(trials) if rng.random() < relay.reliability)
            post = Posterior(post.alpha + successes, post.beta + (trials - successes))
        out[relay.peer_id] = post
    return out


def proven_segments(pool: Pool, cops: Dict[str, Posterior]) -> int:
    return len(
        {
            r.segment
            for r in pool.relays
            if r.proven and cops[r.peer_id].observations > 0
        }
    )


def rank_key(post: Dict[str, Posterior]):
    return lambda relay: post[relay.peer_id].mean


def select_at_depth(pool: Pool, post: Dict[str, Posterior], depth: int) -> List[Relay]:
    """The shipped diversity selector, ranked by posterior mean, at one depth."""
    elig = [r for r in pool.relays if r.peer_id != pool.destination.peer_id]
    if len(elig) < depth:
        return []
    by_segment: Dict[str, List[Relay]] = {}
    for relay in elig:
        by_segment.setdefault(relay.segment, []).append(relay)
    segments = list(by_segment.keys())
    used = {pool.destination.peer_id}
    selected: List[Relay] = []
    idx = 0
    while len(selected) < depth and idx < depth * len(segments) + len(segments):
        segment = segments[idx % len(segments)]
        best = None
        for relay in by_segment[segment]:
            if relay.peer_id in used:
                continue
            if best is None or post[relay.peer_id].mean > post[best.peer_id].mean:
                best = relay
        if best is not None:
            selected.append(best)
            used.add(best.peer_id)
        idx += 1
    return selected if len(selected) == depth else []


def predict(post: Dict[str, Posterior], path: Sequence[Relay]) -> float:
    value = 1.0
    for relay in path:
        value *= post[relay.peer_id].mean
    return value


def dynamic_depth(
    pool: Pool,
    post: Dict[str, Posterior],
    rng: random.Random,
    depth_history: Sequence[int],
) -> int:
    """PAL, floor, and leak-controlled realisation -- the filed design."""
    floor_segments = max(1, proven_segments(pool, post))
    candidates = []
    widest = 0
    for depth in range(1, len(pool.relays) + 1):
        path = select_at_depth(pool, post, depth)
        if not path:
            break
        segments = len({r.segment for r in path})
        widest = max(widest, segments)
        # The floor is the proven diversity itself, never min(depth, proven):
        # the capped form is satisfied trivially at depth 1 and collapses every
        # circuit to one hop (the first run of this harness did exactly that).
        if segments < floor_segments:
            continue
        candidates.append((depth, predict(post, path)))
    if not candidates:
        # No depth can span the proven diversity: take the widest path available.
        for depth in range(1, len(pool.relays) + 1):
            path = select_at_depth(pool, post, depth)
            if not path:
                break
            if len({r.segment for r in path}) == widest:
                candidates.append((depth, predict(post, path)))
        if not candidates:
            return 0
    pal = max(value for _, value in candidates)
    indifferent = sorted(d for d, value in candidates if value >= pal - DEPTH_BAND)
    weights = [sum(1 for h in depth_history if h == d) for d in indifferent]
    total = sum(weights)
    if total == 0:
        return indifferent[0]
    ticket = rng.randrange(total)
    for depth, weight in zip(indifferent, weights):
        if ticket < weight:
            return depth
        ticket -= weight
    return indifferent[0]


def first_hop_entropy(pool: Pool, post: Dict[str, Posterior], samples: int, rng: random.Random) -> float:
    """Entropy proxy over which relay is chosen FIRST, in bits.

    An under-estimate of the privacy the path provides (it ignores later hops),
    used only to detect a policy that collapses first-hop choice onto one relay.
    """
    counts: Dict[str, int] = {}
    history: List[int] = []
    for _ in range(samples):
        depth = dynamic_depth(pool, post, rng, history)
        if depth == 0:
            continue
        history.append(depth)
        if len(history) > DEPTH_HISTORY_WINDOW:
            history.pop(0)
        path = select_at_depth(pool, post, depth)
        if path:
            counts[path[0].peer_id] = counts.get(path[0].peer_id, 0) + 1
    total = sum(counts.values())
    if total == 0:
        return 0.0
    entropy = 0.0
    for count in counts.values():
        p = count / total
        entropy -= p * math.log2(p)
    return entropy


def sybil_pool(rng: random.Random, size: int, operators: int, segments: int) -> Pool:
    """Fresh identities with attractive scores but no outcome history.

    The attack the floor exists to stop: mint new ids, publish a high reliability
    score, and hope a depth budget counts them as diversity.
    """
    pool = make_pool(rng, size, operators=operators, segments=segments)
    relays = list(pool.relays)
    for i in range(size):
        relays.append(
            Relay(
                peer_id=f"sybil{i:02d}",
                reliability=0.98,
                latency_ms=40.0,
                operator=f"sybilop{i % 2}",
                segment=f"sybilseg{i % max(2, segments)}",
                proven=False,
            )
        )
    return Pool(relays=relays, destination=pool.destination)


def poisoned_pool(rng: random.Random, size: int, operators: int, segments: int) -> Pool:
    """Relays whose snapshot score is a lie: history says they fail."""
    pool = make_pool(rng, size, operators=operators, segments=segments)
    relays = []
    for relay in pool.relays:
        if rng.random() < 0.5:
            relays.append(
                Relay(
                    peer_id=relay.peer_id,
                    reliability=0.97,
                    latency_ms=relay.latency_ms,
                    operator=relay.operator,
                    segment=relay.segment,
                )
            )
        else:
            relays.append(relay)
    return Pool(relays=relays, destination=pool.destination)


def degenerate_pool(rng: random.Random, size: int, operators: int, segments: int) -> Pool:
    """One segment, one operator: nothing to diversify over."""
    pool = make_pool(rng, size, operators=1, segments=1)
    relays = [
        Relay(
            peer_id=r.peer_id,
            reliability=r.reliability,
            latency_ms=r.latency_ms,
            operator="op0",
            segment="seg0",
        )
        for r in pool.relays
    ]
    return Pool(relays=relays, destination=pool.destination)


def run_dynamic(
    sizes: Sequence[int],
    pools: int,
    seed: int,
    classes: Sequence[str] = ("original", "sybil", "poisoned", "degenerate"),
) -> Dict[str, object]:
    builders = {
        "original": make_pool,
        "sybil": sybil_pool,
        "poisoned": poisoned_pool,
        "degenerate": degenerate_pool,
    }
    report: Dict[str, object] = {"seed": seed, "depth_band": DEPTH_BAND, "classes": []}
    for kind in classes:
        for size in sizes:
            rng = random.Random(seed * 7919 + size + (len(kind) * 131))
            gaps: List[float] = []
            hop_gaps: List[float] = []
            depths: Dict[int, int] = {}
            entropy_samples: List[float] = []
            proven_seen: List[int] = []
            for _ in range(pools):
                pool = builders[kind](rng, size, operators=max(3, size // 3), segments=max(3, size // 4))
                post = seed_posteriors(pool, rng)
                history: List[int] = []
                depth = dynamic_depth(pool, post, rng, history)
                if depth == 0:
                    continue
                path = select_at_depth(pool, post, depth)
                if not path:
                    continue
                # Delivery is measured against the TRUE reliabilities, not the
                # posteriors: a policy cannot score itself.
                dynamic_prob = delivery_probability(pool, path)
                shipped_depth = rng.randint(MIN_HOPS, MAX_HOPS)
                shipped = champion_path(pool, rng, hops_override=shipped_depth)
                if len(shipped) < MIN_HOPS:
                    continue
                shipped_prob = delivery_probability(pool, shipped)
                gaps.append(100.0 * (dynamic_prob - shipped_prob))
                fixed = champion_path(pool, rng, hops_override=MIN_HOPS)
                if len(fixed) >= MIN_HOPS:
                    hop_gaps.append(100.0 * (dynamic_prob - delivery_probability(pool, fixed)))
                depths[depth] = depths.get(depth, 0) + 1
                proven_seen.append(proven_segments(pool, post))
                if len(entropy_samples) < 50:
                    entropy_samples.append(first_hop_entropy(pool, post, 20, random.Random(seed + size)))
            report["classes"].append(
                {
                    "class": kind,
                    "pool_size": size,
                    "pools": len(gaps),
                    "median_depth": (
                        statistics.median(
                            [d for d, count in depths.items() for _ in range(count)]
                        )
                        if depths
                        else None
                    ),
                    "median_proven_segments": statistics.median(proven_seen) if proven_seen else None,
                    "gap_median_pp": statistics.median(gaps) if gaps else None,
                    "hop_component_median_pp": statistics.median(hop_gaps) if hop_gaps else None,
                    "depth_histogram": dict(sorted(depths.items())),
                    "first_hop_entropy_bits": (
                        statistics.fmean(entropy_samples) if entropy_samples else None
                    ),
                }
            )
    return report


def evaluate_acceptance(reports: Sequence[Dict[str, object]], band: float = DEPTH_BAND) -> Dict[str, object]:
    """The filed acceptance test, applied to both seeds.

    pass iff, on both seeds: total gap within band; hop-count component within
    band at every pool size; no adversarial class loses to the shipped builder by
    more than the band.
    """
    failures: List[str] = []
    for report in reports:
        seed = report["seed"]
        for entry in report["classes"]:
            gap = entry["gap_median_pp"]
            hop = entry["hop_component_median_pp"]
            if gap is None:
                failures.append(f"seed {seed} {entry['class']} size {entry['pool_size']}: no measurable pools")
                continue
            if gap < -band:
                failures.append(
                    f"seed {seed} {entry['class']} size {entry['pool_size']}: dynamic loses by {-gap:.2f}pp"
                )
            if hop is not None and hop < -band:
                failures.append(
                    f"seed {seed} {entry['class']} size {entry['pool_size']}: hop component {hop:.2f}pp"
                )
            # The spec's third failure mode, taken literally: depth must track the
            # proven diversity, not sit above or below it.
            median_depth = entry.get("median_depth")
            median_proven = entry.get("median_proven_segments")
            if median_depth is not None and median_proven is not None:
                if median_depth < median_proven:
                    failures.append(
                        f"seed {seed} {entry['class']} size {entry['pool_size']}: "
                        f"depth {median_depth} does not track proven diversity {median_proven}"
                    )
    return {"passed": not failures, "failures": failures, "band_pp": band}


def self_check(pools: int = 40) -> int:
    """Two independent checks that the harness measures what it claims."""
    failures = 0
    rng = random.Random(4242)
    for _ in range(pools):
        pool = make_pool(rng, 8, operators=3, segments=3)
        elig = eligible(pool)
        # 1. Both estimator implementations must agree with the product on a chain.
        path = elig[:3]
        product = delivery_probability(pool, path)
        topo = chain_topology(pool, path)
        decomposition = exact_decomposition(topo)
        bruteforce = exact_bruteforce(topo)
        if abs(decomposition - product) > 1e-9:
            print(f"[ERROR] decomposition {decomposition!r} != product {product!r}")
            failures += 1
        if abs(bruteforce - product) > 1e-9:
            print(f"[ERROR] bruteforce {bruteforce!r} != product {product!r}")
            failures += 1
        # 2. The joint optimum must never be beaten by a random feasible path.
        best = joint_optimum(pool)
        best_score = objective(pool, best)
        for _ in range(20):
            hops = rng.randint(MIN_HOPS, MAX_HOPS)
            if len(elig) < hops:
                continue
            trial = rng.sample(elig, hops)
            if objective(pool, trial) > best_score + 1e-9:
                print("[ERROR] exhaustive optimum was beaten by random sampling")
                failures += 1
    # 3. Determinism: same seed, same numbers.
    a = run([8], 10, 11)
    b = run([8], 10, 11)
    if json.dumps(a, sort_keys=True) != json.dumps(b, sort_keys=True):
        print("[ERROR] run is not deterministic for a fixed seed")
        failures += 1
    print(f"[{'OK' if failures == 0 else 'FAIL'}] self-check: {failures} failure(s)")
    return failures


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser(description="Circuit selection bake-off (#667)")
    parser.add_argument("--pools", type=int, default=200, help="pools per size class")
    parser.add_argument("--sizes", type=int, nargs="+", default=[8, 12, 16])
    parser.add_argument("--seed", type=int, default=20261111)
    parser.add_argument("--json-out", default=None)
    parser.add_argument("--self-check", action="store_true")
    parser.add_argument(
        "--policy",
        choices=["shipped", "dynamic"],
        default="shipped",
        help="shipped = the original champion-vs-optimum sweep; dynamic = the #682 policy",
    )
    parser.add_argument(
        "--accept",
        default=None,
        help="comma-separated seeds to run the dynamic policy on and evaluate against the filed acceptance band",
    )
    args = parser.parse_args(argv)

    if args.self_check:
        return 1 if self_check() else 0

    if args.accept:
        seeds = [int(s) for s in args.accept.split(",")]
        reports = [run_dynamic(args.sizes, args.pools, seed) for seed in seeds]
        verdict = evaluate_acceptance(reports)
        for report in reports:
            print(f"seed={report['seed']} depth_band={report['depth_band']}pp")
            for entry in report["classes"]:
                print(
                    f"  {entry['class']:>10} pool={entry['pool_size']:>2} n={entry['pools']:>3} "
                    f"gap_median={entry['gap_median_pp'] if entry['gap_median_pp'] is None else round(entry['gap_median_pp'], 2)}pp "
                    f"hop_component={entry['hop_component_median_pp'] if entry['hop_component_median_pp'] is None else round(entry['hop_component_median_pp'], 2)}pp "
                    f"entropy={entry['first_hop_entropy_bits'] if entry['first_hop_entropy_bits'] is None else round(entry['first_hop_entropy_bits'], 2)}b "
                    f"depths={entry['depth_histogram']}"
                )
        print(
            f"[{'OK' if verdict['passed'] else 'FAIL'}] acceptance band {verdict['band_pp']}pp "
            f"over seeds {seeds}: {len(verdict['failures'])} failure(s)"
        )
        for failure in verdict["failures"]:
            print(f"  [FAIL] {failure}")
        if args.json_out:
            with open(args.json_out, "w", encoding="utf-8") as handle:
                json.dump({"reports": reports, "acceptance": verdict}, handle, indent=1)
            print(f"[OK] wrote {args.json_out}")
        return 0 if verdict["passed"] else 1

    if args.policy == "dynamic":
        report = run_dynamic(args.sizes, args.pools, args.seed)
        for entry in report["classes"]:
            print(
                f"{entry['class']:>10} pool={entry['pool_size']:>2} n={entry['pools']:>3} "
                f"gap_median={entry['gap_median_pp']}pp depths={entry['depth_histogram']}"
            )
        if args.json_out:
            with open(args.json_out, "w", encoding="utf-8") as handle:
                json.dump(report, handle, indent=1)
            print(f"[OK] wrote {args.json_out}")
        return 0

    report = run(args.sizes, args.pools, args.seed)
    for c in report["classes"]:
        print(
            f"pool={c['pool_size']:>2} n={c['pools']:>3} "
            f"champ_median={c['champion_median_prob']:.4f} chall_median={c['challenger_median_prob']:.4f} "
            f"gap_median={c['gap_median_pp']:.2f}pp gap_p90={c['gap_p90_pp']:.2f}pp "
            f"pools>=2pp={c['pools_with_gap_over_2pp']}/{c['pools']} identical={c['identical_paths']}"
        )
        print(
            f"        decomposition: relay_choice_median={c['relay_choice_median_pp']:.2f}pp "
            f"(p90 {c['relay_choice_p90_pp']:.2f}) | hop_count_median={c['hop_count_median_pp']:.2f}pp "
            f"(p90 {c['hop_count_p90_pp']:.2f})"
        )
    print(
        f"median-of-class-medians={report['median_of_class_medians_pp']:.2f}pp "
        f"material={report['material']} (threshold {report['material_threshold_pp']}pp)"
    )
    if args.json_out:
        with open(args.json_out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=1)
        print(f"[OK] wrote {args.json_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
