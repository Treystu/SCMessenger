//! Peer Exchange — learn about new relay nodes from connected peers

use super::protocol::{RelayCapability, RelayPeerInfoMessage};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Peer exchange error types
#[derive(Debug, Error)]
pub enum PeerExchangeError {
    #[error("Peer list is full")]
    PeerListFull,
    #[error("Invalid peer info")]
    InvalidPeerInfo,
}

/// Information about a relay peer
#[derive(Debug, Clone)]
pub struct RelayPeerInfo {
    /// Peer's ID (Blake3 hash of public key)
    pub peer_id: String,
    /// Known addresses (TCP, etc.)
    pub addresses: Vec<String>,
    /// Last time we saw this peer online (Unix timestamp)
    pub last_seen: u64,
    /// Reliability score (0.0-1.0, where 1.0 is most reliable)
    pub reliability_score: f32,
    /// Peer's capabilities
    pub capabilities: RelayCapability,
}

impl RelayPeerInfo {
    /// Create a new peer info entry
    pub fn new(peer_id: String, addresses: Vec<String>, capabilities: RelayCapability) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            peer_id,
            addresses,
            last_seen: now,
            reliability_score: 0.5, // Start neutral
            capabilities,
        }
    }

    /// Update last seen time to now
    pub fn mark_seen(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_seen = now;
    }

    /// Increase reliability score (capped at 1.0)
    pub fn record_success(&mut self) {
        self.reliability_score = (self.reliability_score + 0.05).min(1.0);
    }

    /// Decrease reliability score (floor at 0.0)
    pub fn record_failure(&mut self) {
        self.reliability_score = (self.reliability_score - 0.1).max(0.0);
    }

    /// Convert to message format
    pub fn to_message(&self) -> RelayPeerInfoMessage {
        RelayPeerInfoMessage {
            peer_id: self.peer_id.clone(),
            addresses: self.addresses.clone(),
            last_seen: self.last_seen,
            reliability_score: self.reliability_score,
            capabilities: self.capabilities,
        }
    }

    /// Create from message format
    pub fn from_message(msg: RelayPeerInfoMessage) -> Self {
        Self {
            peer_id: msg.peer_id,
            addresses: msg.addresses,
            last_seen: msg.last_seen,
            reliability_score: msg.reliability_score,
            capabilities: msg.capabilities,
        }
    }
}

/// Manages peer exchange and discovery
pub struct PeerExchangeManager {
    /// Known relay peers: peer_id -> RelayPeerInfo
    peers: HashMap<String, RelayPeerInfo>,
    /// Maximum number of known peers (prevents unbounded memory usage)
    max_known_peers: usize,
    /// TTL for peers not seen within this duration (seconds)
    peer_ttl_secs: u64,
}

impl PeerExchangeManager {
    /// Create a new peer exchange manager
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            max_known_peers: 10000,
            peer_ttl_secs: 7 * 24 * 3600, // 7 days
        }
    }

    /// Create with custom configuration
    pub fn with_config(max_known_peers: usize, peer_ttl_secs: u64) -> Self {
        Self {
            peers: HashMap::new(),
            max_known_peers,
            peer_ttl_secs,
        }
    }

    /// Add or update a peer
    pub fn add_peer(&mut self, peer: RelayPeerInfo) -> Result<(), PeerExchangeError> {
        if self.peers.contains_key(&peer.peer_id) {
            // Update existing peer
            self.peers.insert(peer.peer_id.clone(), peer);
            Ok(())
        } else {
            // Check if we're at capacity
            if self.peers.len() >= self.max_known_peers {
                return Err(PeerExchangeError::PeerListFull);
            }

            self.peers.insert(peer.peer_id.clone(), peer);
            Ok(())
        }
    }

    /// Get a peer by ID
    pub fn get_peer(&self, peer_id: &str) -> Option<RelayPeerInfo> {
        self.peers.get(peer_id).cloned()
    }

    /// Get all known peers
    pub fn get_all_peers(&self) -> Vec<RelayPeerInfo> {
        self.peers.values().cloned().collect()
    }

    /// Get peers sorted by reliability (highest first)
    pub fn get_peers_by_reliability(&self) -> Vec<RelayPeerInfo> {
        let mut peers = self.peers.values().cloned().collect::<Vec<_>>();
        peers.sort_by(|a, b| b.reliability_score.partial_cmp(&a.reliability_score).unwrap_or(std::cmp::Ordering::Equal));
        peers
    }

    /// Merge another peer list
    pub fn merge_peer_list(&mut self, peers: Vec<RelayPeerInfo>) {
        for peer in peers {
            let _ = self.add_peer(peer);
        }
    }

    /// Remove stale peers (not seen within TTL)
    pub fn prune_stale(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.peers.retain(|_, peer| now - peer.last_seen < self.peer_ttl_secs);
    }

    /// Record a successful connection to a peer
    pub fn record_success(&mut self, peer_id: &str) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.record_success();
            peer.mark_seen();
        }
    }

    /// Record a failed connection to a peer
    pub fn record_failure(&mut self, peer_id: &str) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.record_failure();
        }
    }

    /// Get number of known peers
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Check if we know about a peer
    pub fn has_peer(&self, peer_id: &str) -> bool {
        self.peers.contains_key(peer_id)
    }

    /// Clear all known peers
    pub fn clear(&mut self) {
        self.peers.clear();
    }

    /// Exchange peers: take a list of new peers and return our best peers
    pub fn exchange_peers(&mut self, incoming_peers: Vec<RelayPeerInfo>) -> Vec<RelayPeerInfo> {
        // Merge incoming peers
        self.merge_peer_list(incoming_peers);

        // Return our best peers (sorted by reliability)
        self.get_peers_by_reliability()
            .into_iter()
            .take(100)
            .collect()
    }
}

impl Default for PeerExchangeManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_peer(id: &str) -> RelayPeerInfo {
        RelayPeerInfo::new(
            id.to_string(),
            vec!["127.0.0.1:8080".to_string()],
            RelayCapability::full_relay(),
        )
    }

    #[test]
    fn test_relay_peer_info_creation() {
        let peer = test_peer("peer1");
        assert_eq!(peer.peer_id, "peer1");
        assert_eq!(peer.addresses.len(), 1);
        assert_eq!(peer.reliability_score, 0.5);
    }

    #[test]
    fn test_mark_seen() {
        let mut peer = test_peer("peer1");
        let old_last_seen = peer.last_seen;

        std::thread::sleep(std::time::Duration::from_millis(10));
        peer.mark_seen();

        assert!(peer.last_seen >= old_last_seen);
    }

    #[test]
    fn test_record_success() {
        let mut peer = test_peer("peer1");
        let initial_score = peer.reliability_score;

        peer.record_success();
        assert!(peer.reliability_score > initial_score);
        assert!(peer.reliability_score <= 1.0);
    }

    #[test]
    fn test_record_failure() {
        let mut peer = test_peer("peer1");
        let initial_score = peer.reliability_score;

        peer.record_failure();
        assert!(peer.reliability_score < initial_score);
        assert!(peer.reliability_score >= 0.0);
    }

    #[test]
    fn test_score_bounds() {
        let mut peer = test_peer("peer1");

        // Maximize score
        for _ in 0..100 {
            peer.record_success();
        }
        assert_eq!(peer.reliability_score, 1.0);

        // Minimize score
        for _ in 0..100 {
            peer.record_failure();
        }
        assert_eq!(peer.reliability_score, 0.0);
    }

    #[test]
    fn test_peer_message_conversion() {
        let peer = test_peer("peer1");
        let msg = peer.to_message();

        assert_eq!(msg.peer_id, peer.peer_id);
        assert_eq!(msg.reliability_score, peer.reliability_score);

        let peer2 = RelayPeerInfo::from_message(msg);
        assert_eq!(peer.peer_id, peer2.peer_id);
    }

    #[test]
    fn test_peer_exchange_manager_creation() {
        let manager = PeerExchangeManager::new();
        assert_eq!(manager.peer_count(), 0);
        assert_eq!(manager.max_known_peers, 10000);
    }

    #[test]
    fn test_add_peer() {
        let mut manager = PeerExchangeManager::new();
        let peer = test_peer("peer1");

        assert!(manager.add_peer(peer).is_ok());
        assert_eq!(manager.peer_count(), 1);
        assert!(manager.has_peer("peer1"));
    }

    #[test]
    fn test_add_peer_duplicate() {
        let mut manager = PeerExchangeManager::new();
        let peer1 = test_peer("peer1");
        let mut peer2 = test_peer("peer1");
        peer2.reliability_score = 0.9;

        manager.add_peer(peer1).unwrap();
        manager.add_peer(peer2).unwrap();

        assert_eq!(manager.peer_count(), 1);
        assert_eq!(manager.get_peer("peer1").unwrap().reliability_score, 0.9);
    }

    #[test]
    fn test_add_peer_capacity() {
        let mut manager = PeerExchangeManager::with_config(2, 7 * 24 * 3600);

        manager.add_peer(test_peer("peer1")).unwrap();
        manager.add_peer(test_peer("peer2")).unwrap();

        let result = manager.add_peer(test_peer("peer3"));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_peer() {
        let mut manager = PeerExchangeManager::new();
        let peer = test_peer("peer1");

        manager.add_peer(peer.clone()).ok();

        let retrieved = manager.get_peer("peer1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().peer_id, "peer1");
    }

    #[test]
    fn test_get_all_peers() {
        let mut manager = PeerExchangeManager::new();

        manager.add_peer(test_peer("peer1")).ok();
        manager.add_peer(test_peer("peer2")).ok();
        manager.add_peer(test_peer("peer3")).ok();

        let peers = manager.get_all_peers();
        assert_eq!(peers.len(), 3);
    }

    #[test]
    fn test_get_peers_by_reliability() {
        let mut manager = PeerExchangeManager::new();

        let mut peer1 = test_peer("peer1");
        peer1.reliability_score = 0.3;

        let mut peer2 = test_peer("peer2");
        peer2.reliability_score = 0.9;

        let mut peer3 = test_peer("peer3");
        peer3.reliability_score = 0.6;

        manager.add_peer(peer1).ok();
        manager.add_peer(peer2).ok();
        manager.add_peer(peer3).ok();

        let ordered = manager.get_peers_by_reliability();
        assert_eq!(ordered[0].peer_id, "peer2");
        assert_eq!(ordered[1].peer_id, "peer3");
        assert_eq!(ordered[2].peer_id, "peer1");
    }

    #[test]
    fn test_merge_peer_list() {
        let mut manager = PeerExchangeManager::new();

        manager.add_peer(test_peer("peer1")).ok();

        let new_peers = vec![test_peer("peer2"), test_peer("peer3")];
        manager.merge_peer_list(new_peers);

        assert_eq!(manager.peer_count(), 3);
    }

    #[test]
    fn test_record_success_2() {
        let mut manager = PeerExchangeManager::new();
        manager.add_peer(test_peer("peer1")).ok();

        let initial_score = manager.get_peer("peer1").unwrap().reliability_score;
        manager.record_success("peer1");
        let new_score = manager.get_peer("peer1").unwrap().reliability_score;

        assert!(new_score > initial_score);
    }

    #[test]
    fn test_record_failure_2() {
        let mut manager = PeerExchangeManager::new();
        manager.add_peer(test_peer("peer1")).ok();

        let initial_score = manager.get_peer("peer1").unwrap().reliability_score;
        manager.record_failure("peer1");
        let new_score = manager.get_peer("peer1").unwrap().reliability_score;

        assert!(new_score < initial_score);
    }

    #[test]
    fn test_prune_stale() {
        let mut manager = PeerExchangeManager::with_config(100, 0); // TTL = 0 seconds

        manager.add_peer(test_peer("peer1")).ok();
        assert_eq!(manager.peer_count(), 1);

        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.prune_stale();

        assert_eq!(manager.peer_count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut manager = PeerExchangeManager::new();

        manager.add_peer(test_peer("peer1")).ok();
        manager.add_peer(test_peer("peer2")).ok();

        assert_eq!(manager.peer_count(), 2);

        manager.clear();
        assert_eq!(manager.peer_count(), 0);
    }

    #[test]
    fn test_exchange_peers() {
        let mut manager = PeerExchangeManager::new();

        manager.add_peer(test_peer("peer1")).ok();

        let incoming = vec![test_peer("peer2"), test_peer("peer3")];
        let outgoing = manager.exchange_peers(incoming);

        assert_eq!(manager.peer_count(), 3);
        assert_eq!(outgoing.len(), 3);
    }

    #[test]
    fn test_exchange_peers_truncation() {
        let mut manager = PeerExchangeManager::new();

        // Add many peers
        for i in 0..150 {
            let peer = test_peer(&format!("peer{}", i));
            manager.add_peer(peer).ok();
        }

        let incoming = vec![];
        let outgoing = manager.exchange_peers(incoming);

        // Should return max 100
        assert!(outgoing.len() <= 100);
    }
}
