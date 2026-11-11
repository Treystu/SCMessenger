// Circuit Building — Selecting and organizing relay paths
//
// Constructs anonymous circuits by selecting diverse intermediate hops
// based on peer reliability and network topology.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    /// Minimum number of hops (not including destination)
    pub min_hops: usize,
    /// Maximum number of hops (not including destination)
    pub max_hops: usize,
    /// Prefer geographically diverse paths
    pub prefer_diverse_paths: bool,
    /// Minimum reliability score for relay selection (0.0 to 1.0)
    pub min_reliability: f32,
}

impl Default for CircuitConfig {
    fn default() -> Self {
        Self {
            min_hops: 3,
            max_hops: 5,
            prefer_diverse_paths: true,
            min_reliability: 0.5,
        }
    }
}

impl CircuitConfig {
    /// Validate circuit configuration
    pub fn validate(&self) -> Result<(), CircuitError> {
        if self.min_hops == 0 || self.max_hops == 0 {
            return Err(CircuitError::InvalidConfig);
        }
        if self.min_hops > self.max_hops {
            return Err(CircuitError::InvalidConfig);
        }
        if self.min_reliability < 0.0 || self.min_reliability > 1.0 {
            return Err(CircuitError::InvalidConfig);
        }
        Ok(())
    }
}

/// Circuit builder for selecting and organizing relay paths
pub struct CircuitBuilder {
    /// Available peer list
    peers: Vec<PeerInfo>,
    /// Circuit configuration
    config: CircuitConfig,
}

impl CircuitBuilder {
    /// Create a new circuit builder with peers and config
    pub fn new(peers: Vec<PeerInfo>, config: CircuitConfig) -> Result<Self, CircuitError> {
        config.validate()?;
        Ok(Self { peers, config })
    }

    /// Build a circuit to a destination using weighted random selection
    pub fn build_circuit(&self, destination: PeerInfo) -> Result<CircuitPath, CircuitError> {
        // Determine number of hops
        let num_hops = self.select_hop_count();

        if num_hops > self.peers.len() {
            return Err(CircuitError::InsufficientPeers);
        }

        // Select diverse hops
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

    /// Select number of hops randomly between min and max
    fn select_hop_count(&self) -> usize {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen_range(self.config.min_hops..=self.config.max_hops)
    }

    /// Select hops with preference for network diversity
    fn select_diverse_hops(
        &self,
        num_hops: usize,
        destination: &PeerInfo,
    ) -> Result<Vec<(String, Vec<u8>)>, CircuitError> {
        let eligible: Vec<_> = self
            .peers
            .iter()
            .filter(|p| {
                p.reliability_score >= self.config.min_reliability
                    && p.peer_id != destination.peer_id
            })
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
                    a.reliability_score
                        .partial_cmp(&b.reliability_score)
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
            .filter(|p| {
                p.reliability_score >= self.config.min_reliability
                    && p.peer_id != destination.peer_id
            })
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

            let total: f64 = pool
                .iter()
                .map(|p| p.reliability_score.max(0.0) as f64)
                .sum();

            let picked = if total > 0.0 {
                let mut ticket = rng.gen::<f64>() * total;
                let mut choice = pool.len() - 1;
                for (index, peer) in pool.iter().enumerate() {
                    ticket -= peer.reliability_score.max(0.0) as f64;
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
    fn test_circuit_config_default() {
        let config = CircuitConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.min_hops, 3);
        assert_eq!(config.max_hops, 5);
    }

    #[test]
    fn test_circuit_config_invalid_min_hops() {
        let config = CircuitConfig {
            min_hops: 0,
            max_hops: 5,
            prefer_diverse_paths: true,
            min_reliability: 0.5,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_circuit_config_invalid_order() {
        let config = CircuitConfig {
            min_hops: 5,
            max_hops: 3,
            prefer_diverse_paths: true,
            min_reliability: 0.5,
        };
        assert!(config.validate().is_err());
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
    fn test_circuit_builder_invalid_config() {
        let peers = create_test_peers(5);
        let config = CircuitConfig {
            min_hops: 0,
            max_hops: 5,
            prefer_diverse_paths: true,
            min_reliability: 0.5,
        };
        let builder = CircuitBuilder::new(peers, config);
        assert!(builder.is_err());
    }

    #[test]
    fn test_circuit_builder_insufficient_peers() {
        let peers = create_test_peers(2);
        let config = CircuitConfig {
            min_hops: 3,
            max_hops: 5,
            prefer_diverse_paths: true,
            min_reliability: 0.5,
        };
        let builder = CircuitBuilder::new(peers, config).unwrap();

        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };

        let result = builder.build_circuit(destination);
        assert!(result.is_err());
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
        assert!(circuit.hop_count() >= 3);
        assert!(circuit.hop_count() <= 5);
        assert!(circuit.destination.is_some());
    }

    #[test]
    fn test_circuit_builder_diverse_paths() {
        let peers = create_test_peers(15);
        let config = CircuitConfig {
            min_hops: 5,
            max_hops: 5,
            prefer_diverse_paths: true,
            min_reliability: 0.5,
        };
        let builder = CircuitBuilder::new(peers, config).unwrap();

        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };

        let circuit = builder.build_circuit(destination).unwrap();
        assert_eq!(circuit.hop_count(), 5);

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

        let config = CircuitConfig {
            min_hops: 5,
            max_hops: 5,
            prefer_diverse_paths: true,
            min_reliability: 0.5,
        };
        let builder = CircuitBuilder::new(peers, config).unwrap();

        let destination = PeerInfo {
            peer_id: "dest".to_string(),
            public_key: vec![99; 32],
            reliability_score: 0.9,
            network_segment: "seg".to_string(),
        };

        let result = builder.build_circuit(destination);
        assert!(matches!(result, Err(CircuitError::InsufficientPeers)));
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
            min_hops: 1,
            max_hops: 1,
            prefer_diverse_paths: false,
            min_reliability: 0.5,
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
            let circuit = builder.build_circuit(destination).unwrap();
            assert_eq!(circuit.hop_count(), 1);
            if circuit.hops[0].0 == "strong" {
                strong += 1;
            }
        }

        // Expected share 0.95 / 1.46 = 0.65. Uniform sampling gave ~0.50, which
        // this bound excludes with a wide margin at n=2000.
        let share = strong as f64 / trials as f64;
        assert!(
            share > 0.60,
            "strong relay share {share:.3} is too low for a reliability-weighted draw"
        );
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
