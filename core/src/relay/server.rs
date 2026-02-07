//! Relay Server — accepts connections and stores messages for offline peers

use super::protocol::{RelayCapability, RelayMessage, PROTOCOL_VERSION};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Relay server configuration
#[derive(Debug, Clone)]
pub struct RelayServerConfig {
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Maximum envelopes to store per peer
    pub max_stored_per_peer: usize,
    /// TTL for stored envelopes in seconds
    pub store_ttl_secs: u64,
    /// Bandwidth limit in bytes per second (0 = unlimited)
    pub bandwidth_limit: usize,
}

impl Default for RelayServerConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            max_stored_per_peer: 10000,
            store_ttl_secs: 24 * 3600, // 24 hours
            bandwidth_limit: 0,
        }
    }
}

/// Stored envelope with metadata
#[derive(Debug, Clone)]
struct StoredEnvelope {
    /// The envelope bytes
    data: Vec<u8>,
    /// Unix timestamp when this was stored
    stored_at: u64,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ConnectionState {
    Connecting,
    Handshaking,
    Connected,
    Disconnected,
}

/// Statistics about relay server operations
#[derive(Debug, Clone)]
pub struct RelayServerStats {
    /// Number of active connections
    pub connections_active: usize,
    /// Total envelopes currently stored
    pub envelopes_stored: usize,
    /// Total envelopes delivered/pulled
    pub envelopes_delivered: u64,
    /// Total bytes relayed
    pub bytes_relayed: u64,
}

/// Relay server error types
#[derive(Debug, Error)]
pub enum RelayServerError {
    #[error("Connection limit exceeded")]
    ConnectionLimitExceeded,
    #[error("Storage limit exceeded for peer")]
    StorageLimitExceeded,
    #[error("Invalid peer state")]
    InvalidPeerState,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid handshake")]
    InvalidHandshake,
}

/// A connected relay peer session
#[derive(Debug)]
#[allow(dead_code)]
struct RelayPeerSession {
    /// Peer's ID (Blake3 hash of public key)
    peer_id: String,
    /// Peer's address
    address: SocketAddr,
    /// Current connection state
    state: ConnectionState,
    /// Peer's advertised capabilities
    capabilities: RelayCapability,
}

/// The relay server
pub struct RelayServer {
    /// Server configuration
    config: RelayServerConfig,
    /// Connected peers: peer_id -> session
    peers: Arc<RwLock<HashMap<String, RelayPeerSession>>>,
    /// Stored envelopes: target_peer_id -> Vec<StoredEnvelope>
    storage: Arc<RwLock<HashMap<String, VecDeque<StoredEnvelope>>>>,
    /// Server statistics
    stats: Arc<RwLock<RelayServerStats>>,
}

impl RelayServer {
    /// Create a new relay server with default configuration
    pub fn new() -> Self {
        Self::with_config(RelayServerConfig::default())
    }

    /// Create a new relay server with custom configuration
    pub fn with_config(config: RelayServerConfig) -> Self {
        Self {
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RelayServerStats {
                connections_active: 0,
                envelopes_stored: 0,
                envelopes_delivered: 0,
                bytes_relayed: 0,
            })),
        }
    }

    /// Register a new peer connection
    pub fn register_peer(
        &self,
        peer_id: String,
        address: SocketAddr,
        capabilities: RelayCapability,
    ) -> Result<(), RelayServerError> {
        let mut peers = self.peers.write();

        // Check connection limit
        if peers.len() >= self.config.max_connections {
            return Err(RelayServerError::ConnectionLimitExceeded);
        }

        peers.insert(
            peer_id.clone(),
            RelayPeerSession {
                peer_id,
                address,
                state: ConnectionState::Connecting,
                capabilities,
            },
        );

        // Update stats
        let mut stats = self.stats.write();
        stats.connections_active = peers.len();

        Ok(())
    }

    /// Mark a peer as handshake completed
    pub fn complete_handshake(&self, peer_id: &str) -> Result<(), RelayServerError> {
        let mut peers = self.peers.write();

        if let Some(session) = peers.get_mut(peer_id) {
            session.state = ConnectionState::Connected;
            Ok(())
        } else {
            Err(RelayServerError::InvalidPeerState)
        }
    }

    /// Store envelopes for a peer who is currently offline
    pub fn store_for_peer(
        &self,
        target_peer_id: &str,
        envelopes: Vec<Vec<u8>>,
    ) -> Result<(u32, u32), RelayServerError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut storage = self.storage.write();
        let peer_queue = storage
            .entry(target_peer_id.to_string())
            .or_insert_with(VecDeque::new);

        let mut accepted = 0u32;
        let mut rejected = 0u32;

        for envelope in envelopes {
            if peer_queue.len() >= self.config.max_stored_per_peer {
                rejected += 1;
                continue;
            }

            peer_queue.push_back(StoredEnvelope {
                data: envelope,
                stored_at: now,
            });
            accepted += 1;
        }

        // Update stats
        let mut stats = self.stats.write();
        stats.envelopes_stored = storage
            .values()
            .map(|q| q.len())
            .sum();

        Ok((accepted, rejected))
    }

    /// Retrieve stored envelopes for a peer
    pub fn get_stored_for(
        &self,
        peer_id: &str,
        since_timestamp: u64,
    ) -> Result<Vec<Vec<u8>>, RelayServerError> {
        let mut storage = self.storage.write();

        if let Some(queue) = storage.get_mut(peer_id) {
            let envelopes: Vec<Vec<u8>> = queue
                .iter()
                .filter(|env| env.stored_at >= since_timestamp)
                .map(|env| env.data.clone())
                .collect();

            // Mark as delivered (remove them)
            queue.clear();

            // Update stats
            let mut stats = self.stats.write();
            stats.envelopes_delivered += envelopes.len() as u64;
            stats.envelopes_stored = storage
                .values()
                .map(|q| q.len())
                .sum();

            Ok(envelopes)
        } else {
            Ok(Vec::new())
        }
    }

    /// Remove a peer from the relay
    pub fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.peers.write();
        peers.remove(peer_id);

        let mut stats = self.stats.write();
        stats.connections_active = peers.len();
    }

    /// Clean up expired stored envelopes
    pub fn cleanup_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let ttl = self.config.store_ttl_secs;
        let mut storage = self.storage.write();

        for queue in storage.values_mut() {
            queue.retain(|env| now - env.stored_at < ttl);
        }

        // Clean up empty queues
        storage.retain(|_, queue| !queue.is_empty());

        // Update stats
        let mut stats = self.stats.write();
        stats.envelopes_stored = storage
            .values()
            .map(|q| q.len())
            .sum();
    }

    /// Get current server statistics
    pub fn get_stats(&self) -> RelayServerStats {
        self.stats.read().clone()
    }

    /// Check if a peer is connected
    pub fn is_peer_connected(&self, peer_id: &str) -> bool {
        let peers = self.peers.read();
        peers.get(peer_id).map_or(false, |s| s.state == ConnectionState::Connected)
    }

    /// Get the number of stored envelopes for a peer
    pub fn stored_count_for_peer(&self, peer_id: &str) -> usize {
        let storage = self.storage.read();
        storage.get(peer_id).map_or(0, |q| q.len())
    }

    /// Generate a handshake acknowledgment message
    pub fn create_handshake_ack(&self, peer_id: String) -> RelayMessage {
        RelayMessage::HandshakeAck {
            version: PROTOCOL_VERSION,
            peer_id,
            capabilities: RelayCapability::full_relay(),
        }
    }

    /// Update bytes relayed statistics
    pub fn add_bytes_relayed(&self, bytes: u64) {
        let mut stats = self.stats.write();
        stats.bytes_relayed += bytes;
    }
}

impl Default for RelayServer {
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

    fn test_server() -> RelayServer {
        RelayServer::new()
    }

    #[test]
    fn test_server_creation() {
        let server = test_server();
        let stats = server.get_stats();
        assert_eq!(stats.connections_active, 0);
        assert_eq!(stats.envelopes_stored, 0);
    }

    #[test]
    fn test_register_peer() {
        let server = test_server();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        let result = server.register_peer(
            "peer1".to_string(),
            addr,
            RelayCapability::full_relay(),
        );

        assert!(result.is_ok());
        let stats = server.get_stats();
        assert_eq!(stats.connections_active, 1);
    }

    #[test]
    fn test_connection_limit() {
        let config = RelayServerConfig {
            max_connections: 2,
            ..Default::default()
        };
        let server = RelayServer::with_config(config);
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        // Register first peer
        assert!(server
            .register_peer("peer1".to_string(), addr, RelayCapability::full_relay())
            .is_ok());

        // Register second peer
        assert!(server
            .register_peer("peer2".to_string(), addr, RelayCapability::full_relay())
            .is_ok());

        // Third should fail
        let result = server.register_peer("peer3".to_string(), addr, RelayCapability::full_relay());
        assert!(result.is_err());
    }

    #[test]
    fn test_store_and_retrieve() {
        let server = test_server();

        let envelopes = vec![vec![1, 2, 3], vec![4, 5, 6]];

        let (accepted, rejected) = server
            .store_for_peer("peer1", envelopes)
            .expect("Failed to store");

        assert_eq!(accepted, 2);
        assert_eq!(rejected, 0);

        let stats = server.get_stats();
        assert_eq!(stats.envelopes_stored, 2);

        let retrieved = server
            .get_stored_for("peer1", 0)
            .expect("Failed to retrieve");

        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0], vec![1, 2, 3]);
        assert_eq!(retrieved[1], vec![4, 5, 6]);
    }

    #[test]
    fn test_storage_limit_per_peer() {
        let config = RelayServerConfig {
            max_stored_per_peer: 2,
            ..Default::default()
        };
        let server = RelayServer::with_config(config);

        let envelopes = vec![vec![1], vec![2], vec![3], vec![4]];

        let (accepted, rejected) = server
            .store_for_peer("peer1", envelopes)
            .expect("Failed to store");

        assert_eq!(accepted, 2);
        assert_eq!(rejected, 2);
    }

    #[test]
    fn test_cleanup_expired() {
        let config = RelayServerConfig {
            store_ttl_secs: 0, // Immediately expire
            ..Default::default()
        };
        let server = RelayServer::with_config(config);

        server
            .store_for_peer("peer1", vec![vec![1, 2, 3]])
            .expect("Failed to store");

        let mut stats = server.get_stats();
        assert_eq!(stats.envelopes_stored, 1);

        // Sleep to ensure TTL passes
        std::thread::sleep(std::time::Duration::from_millis(10));

        server.cleanup_expired();

        stats = server.get_stats();
        assert_eq!(stats.envelopes_stored, 0);
    }

    #[test]
    fn test_remove_peer() {
        let server = test_server();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        server
            .register_peer("peer1".to_string(), addr, RelayCapability::full_relay())
            .expect("Failed to register");

        let stats = server.get_stats();
        assert_eq!(stats.connections_active, 1);

        server.remove_peer("peer1");

        let stats = server.get_stats();
        assert_eq!(stats.connections_active, 0);
    }

    #[test]
    fn test_complete_handshake() {
        let server = test_server();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        server
            .register_peer("peer1".to_string(), addr, RelayCapability::full_relay())
            .expect("Failed to register");

        assert!(!server.is_peer_connected("peer1"));

        server
            .complete_handshake("peer1")
            .expect("Failed to complete handshake");

        assert!(server.is_peer_connected("peer1"));
    }

    #[test]
    fn test_is_peer_connected() {
        let server = test_server();
        assert!(!server.is_peer_connected("peer1"));

        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        server
            .register_peer("peer1".to_string(), addr, RelayCapability::full_relay())
            .expect("Failed to register");

        assert!(!server.is_peer_connected("peer1")); // Not connected yet
        server.complete_handshake("peer1").ok();
        assert!(server.is_peer_connected("peer1"));
    }

    #[test]
    fn test_stored_count_for_peer() {
        let server = test_server();

        assert_eq!(server.stored_count_for_peer("peer1"), 0);

        server
            .store_for_peer("peer1", vec![vec![1], vec![2], vec![3]])
            .expect("Failed to store");

        assert_eq!(server.stored_count_for_peer("peer1"), 3);
    }

    #[test]
    fn test_create_handshake_ack() {
        let server = test_server();
        let msg = server.create_handshake_ack("peer1".to_string());

        match msg {
            RelayMessage::HandshakeAck {
                version,
                peer_id,
                capabilities,
            } => {
                assert_eq!(version, PROTOCOL_VERSION);
                assert_eq!(peer_id, "peer1");
                assert!(capabilities.is_relay());
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_stats_tracking() {
        let server = test_server();

        server.add_bytes_relayed(1000);
        let stats = server.get_stats();
        assert_eq!(stats.bytes_relayed, 1000);

        server.add_bytes_relayed(500);
        let stats = server.get_stats();
        assert_eq!(stats.bytes_relayed, 1500);
    }

    #[test]
    fn test_retrieve_with_timestamp_filter() {
        let server = test_server();

        // Store envelope now
        server
            .store_for_peer("peer1", vec![vec![1, 2, 3]])
            .expect("Failed to store");

        // Request with recent timestamp (should get it)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let retrieved = server
            .get_stored_for("peer1", now - 10)
            .expect("Failed to retrieve");
        assert_eq!(retrieved.len(), 1);
    }
}
