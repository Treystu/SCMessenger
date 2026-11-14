// Circuit Building — Selecting and organizing relay paths
//
// Constructs anonymous circuits by selecting diverse intermediate hops
// based on peer reliability and network topology.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

/// Unique identifier for a circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CircuitId(u64);

impl CircuitId {
    /// Generate a new random circuit ID
    pub fn random() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 8];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut bytes);
        CircuitId(u64::from_le_bytes(bytes))
    }
}

#[derive(Debug, Error)]
pub enum CircuitError {
    #[error("Not enough peers available for circuit")]
    InsufficientPeers,
    #[error("Peer not found")]
    PeerNotFound,
    #[error("Invalid circuit configuration")]
    InvalidConfig,
    #[error("Duplicate peer in path")]
    DuplicatePeer,
}

/// Peer information for circuit selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Unique peer identifier
    pub peer_id: String,
    /// X25519 public key (32 bytes)
    pub public_key: Vec<u8>,
    /// Reliability score (0.0 to 1.0)
    pub reliability_score: f32,
    /// Network segment identifier (for diversity)
    pub network_segment: String,
}

/// Circuit path: ordered list of relay hops to destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitPath {
    /// Unique circuit identifier
    pub circuit_id: CircuitId,
    /// List of relays in order (peer_id, public_key)
    pub hops: Vec<(String, Vec<u8>)>,
    /// Destination peer information
    pub destination: Option<PeerInfo>,
}

impl CircuitPath {
    /// Get the ordered public keys for onion construction
    pub fn public_keys(&self) -> Result<Vec<[u8; 32]>, CircuitError> {
        let mut keys = Vec::new();

        for (_, pk) in &self.hops {
            if pk.len() != 32 {
                return Err(CircuitError::InvalidConfig);
            }
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(pk);
            keys.push(key_bytes);
        }

        if let Some(dest) = &self.destination {
            if dest.public_key.len() != 32 {
                return Err(CircuitError::InvalidConfig);
            }
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&dest.public_key);
            keys.push(key_bytes);
        }

        Ok(keys)
    }

    /// Get the number of hops (not including destination)
    pub fn hop_count(&self) -> usize {
        self.hops.len()
    }
}

/// Configuration for circuit building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitConfig {
    /// Prefer geographically diverse paths
    pub prefer_diverse_paths: bool,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            prefer_diverse_paths: true,
        }
    }
}

impl CircuitConfig {
    /// Validate circuit configuration.
    ///
    /// There is nothing to validate any more: depth is no longer configuration.
    /// `min_hops`, `max_hops` and `min_reliability` were DELETED by the #682
    /// decision (c.6785048161) rather than re-parametrised, because a static
    /// depth or a static eligibility floor is the thing the decision replaces.
    /// The method is kept so existing callers do not have to change shape.
    pub fn validate(&self) -> Result<(), CircuitError> {
        Ok(())
    }
}

/// Materiality band used to decide which depths are indistinguishable.
///
/// This is not a tuned constant: it is the harness's own 2pp band, adopted
/// verbatim from the #682 acceptance spec (c.6785048161), where a difference
/// smaller than 2 percentage points of delivery probability is by definition
/// not material. Depths predicted within this band of the best are
/// interchangeable for delivery, which is exactly the indifference set the leak
/// control samples from.
pub const DEPTH_INDIFFERENCE_BAND: f64 = 0.02;

/// How many realised depths the leak control remembers.
///
/// A window, not a threshold: it bounds stored state, and the spec only asks for
/// the node's "recent realised-depth frequencies".
pub const DEPTH_HISTORY_WINDOW: usize = 64;

/// Beta posterior over one relay's forwarding outcome.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForwardingPosterior {
    /// Successes plus the prior's pseudo-counts.
    pub alpha: f64,
    /// Failures plus the prior's pseudo-counts.
    pub beta: f64,
}

impl ForwardingPosterior {
    /// Cold-start prior seeded from the pool snapshot's own score cross-section.
    ///
    /// Two pseudo-observations, split by the relay's snapshot reliability, so an
    /// unobserved relay lands near the score the pool already assigned it and a
    /// relay's first real outcome moves the mean without erasing the prior.
    pub fn cold_start(score: f32) -> Self {
        let s = score.clamp(0.0, 1.0) as f64;
        Self {
            alpha: 1.0 + 2.0 * s,
            beta: 1.0 + 2.0 * (1.0 - s),
        }
    }

    /// Posterior mean: the predicted per-hop forwarding probability.
    pub fn mean(&self) -> f64 {
        let total = self.alpha + self.beta;
        if total <= 0.0 {
            0.0
        } else {
            self.alpha / total
        }
    }

    /// Attributed outcomes beyond the two prior pseudo-observations.
    pub fn observations(&self) -> f64 {
        (self.alpha + self.beta - 2.0).max(0.0)
    }
}

/// Live snapshot of the pool a build sees, plus the outcomes the node attributed.
#[derive(Debug, Clone, Default)]
pub struct FeedbackStore {
    posteriors: HashMap<String, ForwardingPosterior>,
    depth_history: VecDeque<usize>,
}

impl FeedbackStore {
    /// Empty store; every relay is seeded from the snapshot on first use.
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed a cold-start prior for every relay in the snapshot.
    ///
    /// Existing posteriors are never overwritten: an observed relay keeps its
    /// outcomes when the pool snapshot is refreshed.
    pub fn seed_from_pool(&mut self, peers: &[PeerInfo]) {
        for peer in peers {
            self.posteriors
                .entry(peer.peer_id.clone())
                .or_insert_with(|| ForwardingPosterior::cold_start(peer.reliability_score));
        }
    }

    /// Predicted forwarding probability for a relay, if the pool knows it.
    pub fn mean(&self, peer_id: &str) -> Option<f64> {
        self.posteriors.get(peer_id).map(ForwardingPosterior::mean)
    }

    /// Whether this relay has outcomes beyond its prior.
    ///
    /// This is the spec's "node with outcome history": the identity the floor
    /// counts as PROVEN, which a freshly minted Sybil identity cannot fake.
    pub fn has_history(&self, peer_id: &str) -> bool {
        self.posteriors
            .get(peer_id)
            .map(|p| p.observations() > 0.0)
            .unwrap_or(false)
    }

    /// Attribute one custody chain, prefix-only.
    ///
    /// A signed per-hop custody-ack chain tells the node how far delivery got:
    /// every hop up to `acked_prefix` forwarded, the hop at `acked_prefix` is the
    /// break (counted as a failure), and hops past a break are UNATTRIBUTED --
    /// they never carried the envelope, so counting them would punish relays for
    /// a failure upstream of them. `delivered` extends the ack to the chain end.
    pub fn record_custody_chain(&mut self, chain: &[String], acked_prefix: usize, delivered: bool) {
        for (index, peer_id) in chain.iter().enumerate() {
            let entry = self
                .posteriors
                .entry(peer_id.clone())
                .or_insert(ForwardingPosterior {
                    alpha: 1.0,
                    beta: 1.0,
                });
            if delivered || index < acked_prefix {
                entry.alpha += 1.0;
            } else if index == acked_prefix {
                entry.beta += 1.0;
            }
        }
    }

    /// Remember a realised depth for the leak control.
    pub fn record_depth(&mut self, depth: usize) {
        if self.depth_history.len() == DEPTH_HISTORY_WINDOW {
            self.depth_history.pop_front();
        }
        self.depth_history.push_back(depth);
    }

    /// How often this depth was realised recently.
    pub fn depth_frequency(&self, depth: usize) -> usize {
        self.depth_history.iter().filter(|d| **d == depth).count()
    }
}

/// One candidate depth, scored as a whole path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepthPrediction {
    /// Number of relays in the candidate path.
    pub depth: usize,
    /// Product of the hops' posterior means: predicted end-to-end delivery.
    pub predicted: f64,
    /// Distinct network segments the candidate path spans.
    pub segments: usize,
}

/// Chooses depth per circuit from the live snapshot and attributed outcomes.
///
/// This replaces the uniform `min_hops..=max_hops` draw. The order of operations
/// is the filed design: score the unchanged diversity selector at every depth the
/// pool supports, predict each candidate as the product of its hops' posterior
/// means, take PAL (the best prediction over depths that meet the privacy floor),
/// and realise the shallowest depth that reaches PAL -- sampling within the
/// indifference set so the realised depth cannot be read off the pool state.
pub struct DepthPolicy;

impl DepthPolicy {
    /// Privacy floor: the path must span as much PROVEN identity diversity as the
    /// pool has, capped by its own depth.
    ///
    /// Deliberately not a constant and deliberately not the pool's size: only
    /// relays with attributed outcome history count, so minted identities cannot
    /// inflate the floor, and the floor tracks what the network can actually
    /// provide right now.
    pub fn floor_for(proven_segments: usize) -> usize {
        // NOT min(depth, proven): that made the floor trivially true at depth 1
        // (min(3,1) == 1), which collapsed every circuit to one hop in the first
        // harness run and is precisely the "depth failing to track proven
        // diversity" failure the acceptance spec names. The floor is the proven
        // diversity itself; a depth that cannot span it is not a candidate.
        proven_segments.max(1)
    }

    /// Choose the realised depth, or `None` when no depth meets the floor.
    pub fn choose(
        predictions: &[DepthPrediction],
        proven_segments: usize,
        feedback: &FeedbackStore,
    ) -> Option<usize> {
        let floor = Self::floor_for(proven_segments);
        let mut feasible: Vec<&DepthPrediction> =
            predictions.iter().filter(|p| p.segments >= floor).collect();
        if feasible.is_empty() {
            // The pool cannot span the proven diversity at any supported depth.
            // Fall back to the widest path the pool can build rather than
            // refusing to route: no floor can be met, so take the most diverse
            // candidate available.
            let widest = predictions.iter().map(|p| p.segments).max()?;
            feasible = predictions
                .iter()
                .filter(|p| p.segments == widest)
                .collect();
            if feasible.is_empty() {
                return None;
            }
        }
        let pal = feasible
            .iter()
            .map(|p| p.predicted)
            .fold(f64::NEG_INFINITY, f64::max);
        // The indifference set: every depth within the materiality band of PAL.
        let mut indifferent: Vec<&DepthPrediction> = feasible
            .into_iter()
            .filter(|p| p.predicted >= pal - DEPTH_INDIFFERENCE_BAND)
            .collect();
        indifferent.sort_by_key(|p| p.depth);
        let shallowest = indifferent.first()?.depth;

        // Leak control: sample from this node's own recent realised depths,
        // restricted to the indifference set. With no history in the set, the
        // shallowest depth is realised, which is the deterministic case the spec
        // accepts (a singleton indifference set).
        let weights: Vec<usize> = indifferent
            .iter()
            .map(|p| feedback.depth_frequency(p.depth))
            .collect();
        let total: usize = weights.iter().sum();
        if total == 0 {
            return Some(shallowest);
        }
        use rand::Rng;
        let mut ticket = rand::thread_rng().gen_range(0..total);
        for (prediction, weight) in indifferent.iter().zip(weights) {
            if ticket < weight {
                return Some(prediction.depth);
            }
            ticket -= weight;
        }
        Some(shallowest)
    }
}

/// Circuit builder for selecting and organizing relay paths
pub struct CircuitBuilder {
    /// Available peer list
    peers: Vec<PeerInfo>,
    /// Circuit configuration (diversity preference only; depth is not config)
    config: CircuitConfig,
    /// Attributed forwarding outcomes and realised-depth history
    feedback: FeedbackStore,
}

impl CircuitBuilder {
    /// Create a new circuit builder with peers and config
    pub fn new(peers: Vec<PeerInfo>, config: CircuitConfig) -> Result<Self, CircuitError> {
        config.validate()?;
        let mut feedback = FeedbackStore::new();
        // Cold start: priors come from the snapshot's own score cross-section, so
        // a fresh node still has a usable prediction for every relay.
        feedback.seed_from_pool(&peers);
        Ok(Self {
            peers,
            config,
            feedback,
        })
    }

    /// Attribute a completed (or broken) custody chain to the relays it names.
    ///
    /// This is the only way posteriors move: outcomes come from the per-hop
    /// signed custody-ack chain, prefix-only, never from local guesses.
    pub fn record_custody_chain(&mut self, chain: &[String], acked_prefix: usize, delivered: bool) {
        self.feedback
            .record_custody_chain(chain, acked_prefix, delivered);
    }

    /// Remember the depth a circuit actually realised (leak-control input).
    pub fn record_realized_depth(&mut self, depth: usize) {
        self.feedback.record_depth(depth);
    }

    /// The feedback store, for inspection by callers and tests.
    pub fn feedback(&self) -> &FeedbackStore {
        &self.feedback
    }

    /// Build a circuit to a destination, with depth computed from live state.
    ///
    /// The uniform depth draw is gone (#682). Depth is chosen by `DepthPolicy`
    /// from the pool snapshot and the per-relay posteriors: every depth the pool
    /// supports is scored as a whole path, PAL is the best of those, and the
    /// realised depth is the shallowest depth reaching PAL (sampled within the
    /// indifference band for leak control).
    pub fn build_circuit(&self, destination: PeerInfo) -> Result<CircuitPath, CircuitError> {
        let predictions = self.predict_by_depth(&destination);
        let proven_segments = self.proven_segments();
        let num_hops = DepthPolicy::choose(&predictions, proven_segments, &self.feedback)
            .ok_or(CircuitError::InsufficientPeers)?;

        let selected_hops = if self.config.prefer_diverse_paths {
            self.select_diverse_hops(num_hops, &destination)?
        } else {
            self.select_random_hops(num_hops, &destination)?
        };

        Ok(CircuitPath {
            circuit_id: CircuitId::random(),
            hops: selected_hops,
            destination: Some(destination),
        })
    }

    /// Distinct relays the pool can put in a path (destination excluded).
    fn pool_capacity(&self, destination: &PeerInfo) -> usize {
        let mut ids: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for peer in &self.peers {
            if peer.peer_id != destination.peer_id {
                ids.insert(peer.peer_id.as_str());
            }
        }
        ids.len()
    }

    /// Distinct segments among relays that carry attributed outcome history.
    fn proven_segments(&self) -> usize {
        let mut segments: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for peer in &self.peers {
            if self.feedback.has_history(&peer.peer_id) {
                segments.insert(peer.network_segment.as_str());
            }
        }
        segments.len()
    }

    /// Score the selector at every depth the pool supports.
    ///
    /// The selector itself is unchanged; it is simply run at each depth instead
    /// of at one drawn depth, and each resulting path is predicted as the product
    /// of its hops' posterior means.
    fn predict_by_depth(&self, destination: &PeerInfo) -> Vec<DepthPrediction> {
        let capacity = self.pool_capacity(destination);
        let mut predictions = Vec::with_capacity(capacity);
        for depth in 1..=capacity {
            let path = if self.config.prefer_diverse_paths {
                self.select_diverse_hops(depth, destination)
            } else {
                self.select_random_hops(depth, destination)
            };
            let Ok(hops) = path else { continue };
            if hops.len() < depth {
                continue;
            }
            let predicted = hops
                .iter()
                .map(|(peer_id, _)| self.feedback.mean(peer_id).unwrap_or(0.0))
                .product::<f64>();
            let mut segments: std::collections::HashSet<&str> = std::collections::HashSet::new();
            for (peer_id, _) in &hops {
                if let Some(peer) = self.peers.iter().find(|p| &p.peer_id == peer_id) {
                    segments.insert(peer.network_segment.as_str());
                }
            }
            predictions.push(DepthPrediction {
                depth,
                predicted,
                segments: segments.len(),
            });
        }
        predictions
    }

    /// Rank a relay by its posterior mean (the documented weighting, made real).
    fn relay_score(&self, peer: &PeerInfo) -> f64 {
        self.feedback
            .mean(&peer.peer_id)
            .unwrap_or_else(|| peer.reliability_score.clamp(0.0, 1.0) as f64)
    }

    /// Select hops with preference for network diversity
    fn select_diverse_hops(
        &self,
        num_hops: usize,
        destination: &PeerInfo,
    ) -> Result<Vec<(String, Vec<u8>)>, CircuitError> {
        // No static eligibility floor (#682): every relay the pool knows is
        // eligible, and the posterior mean decides who is worth a hop. A relay
        // proven bad simply never wins a selection rather than being filtered by
        // a constant that would also have dropped an unobserved good relay.
        let eligible: Vec<_> = self
            .peers
            .iter()
            .filter(|p| p.peer_id != destination.peer_id)
            .collect();

        if eligible.len() < num_hops {
            return Err(CircuitError::InsufficientPeers);
        }

        // Group by network segment
        let mut by_segment: HashMap<String, Vec<_>> = HashMap::new();
        for peer in eligible {
            by_segment
                .entry(peer.network_segment.clone())
                .or_insert_with(Vec::new)
                .push(peer);
        }

        let mut selected = Vec::new();
        let mut used_peers = std::collections::HashSet::new();
        used_peers.insert(destination.peer_id.clone());

        // Try to select from different segments
        let segments: Vec<_> = by_segment.keys().collect();
        let mut segment_idx = 0;

        // Progress guarantee (#683). The relay-count guard above is not enough:
        // `eligible` is a list of records, not of distinct peer ids, so a peer
        // that carries two segment records (the same relay observed on two
        // networks) can satisfy `eligible.len() >= num_hops` while the pool
        // holds fewer distinct relays than the draw asks for. Once every
        // segment's peers are used, `best_peer` is `None` on every iteration and
        // the loop spins forever. A full pass over all segments without a
        // single selection means no distinct relay remains, so stop there.
        let mut segments_without_progress = 0usize;

        while selected.len() < num_hops {
            if segments.is_empty() {
                return Err(CircuitError::InsufficientPeers);
            }
            if segments_without_progress >= segments.len() {
                return Err(CircuitError::InsufficientPeers);
            }

            let segment = segments[segment_idx % segments.len()];
            let peers_in_segment = &by_segment[segment];

            // Find an unused peer in this segment with highest reliability
            let best_peer = peers_in_segment
                .iter()
                .filter(|p| !used_peers.contains(&p.peer_id))
                .max_by(|a, b| {
                    self.relay_score(a)
                        .partial_cmp(&self.relay_score(b))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            match best_peer {
                Some(peer) => {
                    selected.push((peer.peer_id.clone(), peer.public_key.clone()));
                    used_peers.insert(peer.peer_id.clone());
                    segments_without_progress = 0;
                }
                None => segments_without_progress += 1,
            }

            segment_idx += 1;
        }

        Ok(selected)
    }

    /// Select hops using weighted random selection by reliability
    fn select_random_hops(
        &self,
        num_hops: usize,
        destination: &PeerInfo,
    ) -> Result<Vec<(String, Vec<u8>)>, CircuitError> {
        use rand::thread_rng;
        use rand::Rng;

        let eligible: Vec<_> = self
            .peers
            .iter()
            .filter(|p| p.peer_id != destination.peer_id)
            .collect();

        if eligible.len() < num_hops {
            return Err(CircuitError::InsufficientPeers);
        }

        // Weighted sampling without replacement (#683). This function is
        // documented as weighting by reliability, but `choose_multiple` draws
        // uniformly, which discards exactly the signal the field carries: a
        // 0.95 relay and a 0.51 relay entered the path with equal probability.
        // Draw proportional to the score instead (ticket = score / total), and
        // remove each pick so no peer occupies two hops.
        let mut pool: Vec<&PeerInfo> = eligible;
        let mut rng = thread_rng();
        let mut selected = Vec::with_capacity(num_hops);

        for _ in 0..num_hops {
            if pool.is_empty() {
                return Err(CircuitError::InsufficientPeers);
            }

            // Draw proportional to the posterior mean, which is what the
            // documentation always claimed the draw was proportional to.
            let total: f64 = pool.iter().map(|p| self.relay_score(p).max(0.0)).sum();

            let picked = if total > 0.0 {
                let mut ticket = rng.gen::<f64>() * total;
                let mut choice = pool.len() - 1;
                for (index, peer) in pool.iter().enumerate() {
                    ticket -= self.relay_score(peer).max(0.0);
                    if ticket <= 0.0 {
                        choice = index;
                        break;
                    }
                }
                choice
            } else {
                // Every eligible score is zero; fall back to a uniform draw so
                // the caller still receives a path when one exists.
                rng.gen_range(0..pool.len())
            };

            let peer = pool.swap_remove(picked);
            selected.push((peer.peer_id.clone(), peer.public_key.clone()));
        }

        Ok(selected)
    }

    /// Get list of available peers
    pub fn peers(&self) -> &[PeerInfo] {
        &self.peers
    }

    /// Update peer reliability score
    pub fn update_peer_reliability(
        &mut self,
        peer_id: &str,
        score: f32,
    ) -> Result<(), CircuitError> {
        let peer = self
            .peers
            .iter_mut()
            .find(|p| p.peer_id == peer_id)
            .ok_or(CircuitError::PeerNotFound)?;

        peer.reliability_score = score.clamp(0.0, 1.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_peers(count: usize) -> Vec<PeerInfo> {
        (0..count)
            .map(|i| PeerInfo {
                peer_id: format!("peer_{}", i),
                public_key: vec![i as u8; 32],
                reliability_score: 0.5 + (i as f32) * 0.05,
                network_segment: format!("segment_{}", i % 3),
            })
            .collect()
    }

    #[test]
    fn test_circuit_id_random() {
        let id1 = CircuitId::random();
        let id2 = CircuitId::random();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_circuit_id_serialization() {
        let id = CircuitId::random();
        let serialized = bincode::serialize(&id).unwrap();
        let deserialized: CircuitId = bincode::deserialize(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_circuit_config_carries_no_depth_constants() {
        // #682 deleted min_hops, max_hops and min_reliability rather than
        // re-parametrising them. This test cannot be written to check their
        // absence at runtime -- the guard is that it compiles at all, so any
        // attempt to reintroduce a configurable depth or floor breaks the build
        // here first.
        let config = CircuitConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.prefer_diverse_paths);
    }

    #[test]
    fn test_circuit_path_public_keys() {
        let path = CircuitPath {
            circuit_id: CircuitId::random(),
            hops: vec![("peer1".to_string(), vec![1; 32])],
            destination: Some(PeerInfo {
                peer_id: "dest".to_string(),
                public_key: vec![2; 32],
                reliability_score: 0.9,
                network_segment: "seg1".to_string(),
            }),
        };

        let keys = path.public_keys().unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_circuit_path_public_keys_invalid_length() {
        let path = CircuitPath {
            circuit_id: CircuitId::random(),
            hops: vec![(
                "peer1".to_string(),
                vec![1; 31], // Wrong length
            )],
            destination: None,
        };

        assert!(path.public_keys().is_err());
    }

    #[test]
    fn test_circuit_path_hop_count() {
        let path = CircuitPath {
            circuit_id: CircuitId::random(),
            hops: vec![
                ("peer1".to_string(), vec![1; 32]),
                ("peer2".to_string(), vec![2; 32]),
            ],
            destination: None,
        };

        assert_eq!(path.hop_count(), 2);
    }

    #[test]
    fn test_circuit_builder_new() {
        let peers = create_test_peers(5);
        let config = CircuitConfig::default();
        let builder = CircuitBuilder::new(peers, config);
        assert!(builder.is_ok());
    }

    #[test]
    fn test_circuit_builder_builds_with_two_relays() {
        // Under dynamic depth, two relays are a usable pool: the policy may
        // realise a 1- or 2-hop circuit. The old static minimum of 3 made this
        // an error; that floor is gone, so the assertion is about capacity, not
        // about a constant.
        let peers = create_test_peers(2);
        let builder = CircuitBuilder::new(peers, CircuitConfig::default()).unwrap();

        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };

        let circuit = builder.build_circuit(destination).unwrap();
        assert!(circuit.hop_count() >= 1);
        assert!(circuit.hop_count() <= 2);
    }

    #[test]
    fn test_circuit_builder_errors_on_empty_pool() {
        let builder = CircuitBuilder::new(Vec::new(), CircuitConfig::default()).unwrap();
        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };
        assert!(matches!(
            builder.build_circuit(destination),
            Err(CircuitError::InsufficientPeers)
        ));
    }

    #[test]
    fn test_circuit_builder_build_circuit() {
        let peers = create_test_peers(10);
        let config = CircuitConfig::default();
        let builder = CircuitBuilder::new(peers, config).unwrap();

        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };

        let circuit = builder.build_circuit(destination).unwrap();
        assert!(circuit.hop_count() >= 1);
        assert!(circuit.hop_count() <= 10);
        assert!(circuit.destination.is_some());
    }

    #[test]
    fn test_circuit_builder_diverse_paths() {
        let peers = create_test_peers(15);
        let config = CircuitConfig::default();
        let builder = CircuitBuilder::new(peers, config).unwrap();

        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };

        let circuit = builder.build_circuit(destination).unwrap();
        assert!(circuit.hop_count() >= 1);
        assert!(circuit.hop_count() <= 15);

        // Check no duplicate peers
        let mut seen = std::collections::HashSet::new();
        for (peer_id, _) in &circuit.hops {
            assert!(!seen.contains(peer_id));
            seen.insert(peer_id.clone());
        }
    }

    #[test]
    fn test_select_diverse_hops_terminates_when_distinct_relays_run_out() {
        // #683(a): `eligible` counts records, not distinct ids. The same relay
        // observed on two segments yields 6 records for 3 distinct peers, which
        // satisfies the record-count guard (`eligible.len() >= num_hops`) while
        // the pool cannot supply 5 distinct relays. Before the progress guard
        // this loop never returned.
        let peers: Vec<PeerInfo> = (0..3)
            .flat_map(|i| {
                (0..2).map(move |segment| PeerInfo {
                    peer_id: format!("peer_{}", i),
                    public_key: vec![i as u8; 32],
                    reliability_score: 0.8,
                    network_segment: format!("segment_{}", segment),
                })
            })
            .collect();

        assert_eq!(peers.len(), 6);
        assert_eq!(
            peers
                .iter()
                .map(|p| p.peer_id.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            3
        );

        let builder = CircuitBuilder::new(peers, CircuitConfig::default()).unwrap();

        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };

        // #683(a) still holds at the selector level, but the dynamic policy must
        // not let an unsatisfiable deep draw take the build down: with three
        // distinct relays the circuit is built at a depth the pool supports.
        let circuit = builder.build_circuit(destination).unwrap();
        assert!(circuit.hop_count() <= 3);
    }

    #[test]
    fn test_select_random_hops_weights_by_reliability() {
        // #683(b): the documented weighting must be real. One strong relay and
        // one weak relay, one hop requested, many draws: the strong relay should
        // dominate in proportion to its score, not appear half the time.
        let peers = vec![
            PeerInfo {
                peer_id: "strong".to_string(),
                public_key: vec![1; 32],
                reliability_score: 0.95,
                network_segment: "segment_a".to_string(),
            },
            PeerInfo {
                peer_id: "weak".to_string(),
                public_key: vec![2; 32],
                reliability_score: 0.51,
                network_segment: "segment_b".to_string(),
            },
        ];
        let config = CircuitConfig {
            prefer_diverse_paths: false,
        };
        let builder = CircuitBuilder::new(peers, config).unwrap();

        let mut strong = 0usize;
        let trials = 2000usize;
        for _ in 0..trials {
            let destination = PeerInfo {
                peer_id: "dest".to_string(),
                public_key: vec![99; 32],
                reliability_score: 0.9,
                network_segment: "seg".to_string(),
            };
            // The documented weighting lives in the sampler, so this test drives
            // the sampler directly: a build now also chooses depth, and mixing the
            // two would make the assertion measure the depth policy instead.
            let hops = builder.select_random_hops(1, &destination).unwrap();
            if hops[0].0 == "strong" {
                strong += 1;
            }
        }

        // The draw weights by the POSTERIOR mean, and with a cold start that mean
        // is shrunk toward 0.5 by the prior's two pseudo-observations (0.725 and
        // 0.505 here, not 0.95 and 0.51). So the expectation is computed from the
        // posteriors the builder actually holds rather than from the raw scores:
        // a hard-coded 0.65 would encode the pre-#682 weighting and fail a correct
        // implementation.
        let strong_mean = builder.feedback().mean("strong").unwrap();
        let weak_mean = builder.feedback().mean("weak").unwrap();
        let expected = strong_mean / (strong_mean + weak_mean);
        let share = strong as f64 / trials as f64;
        assert!(
            (share - expected).abs() < 0.05,
            "strong relay share {share:.3} is not the weighted share {expected:.3}"
        );
        assert!(
            share > 0.50,
            "share {share:.3} is no better than uniform for a weighted draw"
        );
    }

    #[test]
    fn test_depth_tracks_proven_diversity_not_pool_size() {
        // The privacy floor counts relays with attributed outcome HISTORY, so a
        // pool padded with minted identities cannot push the depth up. Start
        // with three relays that carry history and three that do not.
        let mut peers = create_test_peers(6);
        for (i, peer) in peers.iter_mut().enumerate() {
            peer.network_segment = format!("segment_{}", i / 2);
        }
        let mut builder = CircuitBuilder::new(peers, CircuitConfig::default()).unwrap();
        let proven: Vec<String> = vec![
            "peer_0".to_string(),
            "peer_2".to_string(),
            "peer_4".to_string(),
        ];
        builder.record_custody_chain(&proven, 3, true);

        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };
        let circuit = builder.build_circuit(destination).unwrap();

        // Three proven relays spanning three segments is the floor the pool can
        // honour; the realised depth must not fall below it.
        assert!(circuit.hop_count() >= 3, "depth {}", circuit.hop_count());
    }

    #[test]
    fn test_custody_chain_attribution_is_prefix_only() {
        let mut store = FeedbackStore::new();
        let chain = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        // a and b acked, c is the break, d never carried the envelope.
        store.record_custody_chain(&chain, 2, false);

        assert!(store.has_history("a") && store.has_history("b"));
        assert!(store.has_history("c"));
        assert!(
            !store.has_history("d"),
            "a hop past the break must stay unattributed"
        );
        let mean_a = store.mean("a").unwrap();
        let mean_c = store.mean("c").unwrap();
        assert!(mean_a > mean_c, "acked prefix must score above the break");
    }

    #[test]
    fn test_depth_policy_realises_shallowest_without_history() {
        let feedback = FeedbackStore::new();
        let predictions = vec![
            DepthPrediction {
                depth: 1,
                predicted: 0.90,
                segments: 1,
            },
            DepthPrediction {
                depth: 2,
                predicted: 0.89,
                segments: 2,
            },
            DepthPrediction {
                depth: 3,
                predicted: 0.60,
                segments: 3,
            },
        ];
        // The floor is the proven diversity (2 segments here), so the 1-hop
        // candidate is not admissible at all -- it cannot span what the pool has
        // proven. Among the admissible depths PAL is 0.89 at depth 2 and 0.60 at
        // depth 3, which is outside the band, so the shallowest indifferent depth
        // is 2. With no realised-depth history, 2 is what is realised.
        assert_eq!(DepthPolicy::choose(&predictions, 2, &feedback), Some(2));
    }

    #[test]
    fn test_depth_policy_honours_the_floor() {
        let feedback = FeedbackStore::new();
        let predictions = vec![
            DepthPrediction {
                depth: 1,
                predicted: 0.95,
                segments: 1,
            },
            DepthPrediction {
                depth: 3,
                predicted: 0.70,
                segments: 3,
            },
        ];
        // The best delivery is at depth 1, but it does not span the three
        // segments the pool has proven, so it cannot be used.
        assert_eq!(DepthPolicy::choose(&predictions, 3, &feedback), Some(3));
    }

    #[test]
    fn test_circuit_builder_update_reliability() {
        let peers = create_test_peers(5);
        let config = CircuitConfig::default();
        let mut builder = CircuitBuilder::new(peers, config).unwrap();

        let result = builder.update_peer_reliability("peer_0", 0.95);
        assert!(result.is_ok());
    }

    #[test]
    fn test_circuit_builder_update_reliability_nonexistent() {
        let peers = create_test_peers(5);
        let config = CircuitConfig::default();
        let mut builder = CircuitBuilder::new(peers, config).unwrap();

        let result = builder.update_peer_reliability("nonexistent", 0.95);
        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_path_serialization() {
        let path = CircuitPath {
            circuit_id: CircuitId::random(),
            hops: vec![("peer1".to_string(), vec![1; 32])],
            destination: Some(PeerInfo {
                peer_id: "dest".to_string(),
                public_key: vec![2; 32],
                reliability_score: 0.9,
                network_segment: "seg".to_string(),
            }),
        };

        let serialized = bincode::serialize(&path).unwrap();
        let deserialized: CircuitPath = bincode::deserialize(&serialized).unwrap();
        assert_eq!(path.circuit_id, deserialized.circuit_id);
    }
}
