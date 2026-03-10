//! Peer Discovery Broadcasting
//!
//! This module implements active relay functionality where relay nodes
//! broadcast peer join/leave events to all connected clients.

use libp2p::PeerId;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Import from local relay module
use crate::relay::protocol::{RelayCapability, RelayMessage, RelayPeerInfoMessage};

/// Tracks connected peers and handles broadcasting
pub struct PeerBroadcaster {
    /// Currently connected peers and their info
    connected_peers: HashMap<PeerId, PeerInfo>,
}

/// Information about a connected peer
struct PeerInfo {
    /// Peer's addresses
    addresses: Vec<String>,
    /// When we first saw this peer
    _connected_at: u64,
    /// Peer's capabilities (if known)
    capabilities: Option<RelayCapability>,
}

impl Default for PeerBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl PeerBroadcaster {
    /// Create a new peer broadcaster
    pub fn new() -> Self {
        Self {
            connected_peers: HashMap::new(),
        }
    }

    /// Record a new peer connection
    pub fn peer_connected(&mut self, peer_id: PeerId, addresses: Vec<String>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.connected_peers.insert(
            peer_id,
            PeerInfo {
                addresses,
                _connected_at: now,
                capabilities: Some(RelayCapability::full_relay()),
            },
        );
    }

    /// Record a peer disconnection
    pub fn peer_disconnected(&mut self, peer_id: &PeerId) {
        self.connected_peers.remove(peer_id);
    }

    /// Create a PeerJoined message for a newly connected peer
    pub fn create_peer_joined_message(&self, peer_id: &PeerId) -> Option<RelayMessage> {
        let info = self.connected_peers.get(peer_id)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Some(RelayMessage::PeerJoined {
            peer_info: RelayPeerInfoMessage {
                peer_id: peer_id.to_string(),
                addresses: info.addresses.clone(),
                last_seen: now,
                reliability_score: 1.0,
                capabilities: info.capabilities.unwrap_or_default(),
            },
        })
    }

    /// Create a PeerLeft message for a disconnected peer
    pub fn create_peer_left_message(peer_id: &PeerId) -> RelayMessage {
        RelayMessage::PeerLeft {
            peer_id: peer_id.to_string(),
        }
    }

    /// Create a PeerListResponse with all currently connected peers
    pub fn create_peer_list_response(&self) -> RelayMessage {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let peers: Vec<RelayPeerInfoMessage> = self
            .connected_peers
            .iter()
            .map(|(peer_id, info)| RelayPeerInfoMessage {
                peer_id: peer_id.to_string(),
                addresses: info.addresses.clone(),
                last_seen: now,
                reliability_score: 1.0,
                capabilities: info.capabilities.unwrap_or_default(),
            })
            .collect();

        RelayMessage::PeerListResponse { peers }
    }

    /// Get list of all connected peers (excluding the specified peer)
    pub fn get_peers_except(&self, exclude: &PeerId) -> Vec<PeerId> {
        self.connected_peers
            .keys()
            .filter(|&peer_id| peer_id != exclude)
            .cloned()
            .collect()
    }

    /// Get count of connected peers
    pub fn peer_count(&self) -> usize {
        self.connected_peers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_broadcaster() {
        let mut broadcaster = PeerBroadcaster::new();
        let peer1 = PeerId::random();
        let peer2 = PeerId::random();

        // Add peers
        broadcaster.peer_connected(peer1, vec!["/ip4/1.2.3.4/tcp/1234".to_string()]);
        broadcaster.peer_connected(peer2, vec!["/ip4/5.6.7.8/tcp/5678".to_string()]);

        assert_eq!(broadcaster.peer_count(), 2);

        // Test peer list
        let peers = broadcaster.get_peers_except(&peer1);
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0], peer2);

        // Remove peer
        broadcaster.peer_disconnected(&peer1);
        assert_eq!(broadcaster.peer_count(), 1);
    }
}
