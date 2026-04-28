//! Mock Transport components for testing
//!
//! Provides mock implementations for swarm handling and transport operations.

use libp2p::PeerId;
use scmessenger_core::transport::SwarmHandle;

/// Mock SwarmHandle that provides controlled behavior for testing
#[derive(Debug, Clone)]
pub struct MockSwarmHandle {
    pub peer_id: PeerId,
    pub peers: Vec<PeerId>,
}

impl MockSwarmHandle {
    /// Create a new mock swarm handle with the given peer ID
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            peers: Vec::new(),
        }
    }

    /// Create a mock swarm with some initial peers
    pub fn with_peers(peer_id: PeerId, peers: Vec<PeerId>) -> Self {
        Self { peer_id, peers }
    }

    /// Get the mock peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Get the list of mock peers
    pub fn get_peers(&self) -> &[PeerId] {
        &self.peers
    }

    /// Add a peer to the mock swarm
    pub fn add_peer(&mut self, peer: PeerId) {
        self.peers.push(peer);
    }

    /// Remove a peer from the mock swarm
    pub fn remove_peer(&mut self, peer: &PeerId) {
        self.peers.retain(|p| p != peer);
    }
}

/// Create a mock swarm handle with a random peer ID
pub fn create_mock_swarm() -> MockSwarmHandle {
    MockSwarmHandle::new(PeerId::random())
}

/// Create a mock swarm handle with a specific number of peers
pub fn create_mock_swarm_with_peers(num_peers: usize) -> MockSwarmHandle {
    let peer_id = PeerId::random();
    let peers = (0..num_peers).map(|_| PeerId::random()).collect();
    MockSwarmHandle::with_peers(peer_id, peers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_swarm_creation() {
        let swarm = create_mock_swarm();
        assert!(swarm.peer_id().to_bytes().len() > 0);
        assert!(swarm.get_peers().is_empty());
    }

    #[test]
    fn test_mock_swarm_with_peers() {
        let swarm = create_mock_swarm_with_peers(3);
        assert_eq!(swarm.get_peers().len(), 3);
    }

    #[test]
    fn test_mock_swarm_peer_management() {
        let mut swarm = create_mock_swarm();

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        swarm.add_peer(peer1);
        swarm.add_peer(peer2);

        assert_eq!(swarm.get_peers().len(), 2);
        assert!(swarm.get_peers().contains(&peer1));
        assert!(swarm.get_peers().contains(&peer2));

        swarm.remove_peer(&peer1);
        assert_eq!(swarm.get_peers().len(), 1);
        assert!(!swarm.get_peers().contains(&peer1));
    }
}
