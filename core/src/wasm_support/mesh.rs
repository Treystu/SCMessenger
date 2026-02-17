// WASM Mesh Node Implementation
//
// Full mesh participant while browser tab is active. Stores messages
// locally and relays to other peers with periodic sync.

use super::storage::{EvictionStrategy, WasmStore, WasmStoreConfig};
use libp2p::identity::Keypair;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// Mesh node state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmMeshState {
    Initializing,
    Active,
    Syncing,
    Paused,
    Shutdown,
}

/// Configuration for WASM mesh node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmMeshConfig {
    /// Local storage quota in bytes (default 50MB)
    pub storage_quota_bytes: usize,
    /// Relay messages while node is active
    pub relay_while_active: bool,
    /// Sync interval in milliseconds (default 30s)
    pub sync_interval_ms: u64,
}

impl Default for WasmMeshConfig {
    fn default() -> Self {
        Self {
            storage_quota_bytes: 50_000_000,
            relay_while_active: true,
            sync_interval_ms: 30_000,
        }
    }
}

#[derive(Debug, Error)]
pub enum MeshError {
    #[error("Store error: {0}")]
    StoreError(String),
    #[error("Invalid envelope")]
    InvalidEnvelope,
    #[error("Node not active")]
    NotActive,
    #[error("Relay disabled")]
    RelayDisabled,
}

/// WASM mesh node implementation
pub struct WasmMeshNode {
    config: WasmMeshConfig,
    state: Arc<RwLock<WasmMeshState>>,
    store: Arc<WasmStore>,
    last_sync_ms: Arc<RwLock<u64>>,
    message_count: Arc<RwLock<u64>>,
    identity_keys: Keypair,
    nickname: Arc<RwLock<Option<String>>>,
}

impl WasmMeshNode {
    /// Create a new mesh node with the given configuration
    pub fn new(config: WasmMeshConfig) -> Self {
        let store_config = WasmStoreConfig {
            max_messages: 1000,
            max_total_bytes: config.storage_quota_bytes,
            eviction_strategy: EvictionStrategy::Priority,
        };

        Self {
            config,
            state: Arc::new(RwLock::new(WasmMeshState::Initializing)),
            store: Arc::new(WasmStore::new(store_config)),
            last_sync_ms: Arc::new(RwLock::new(0)),
            message_count: Arc::new(RwLock::new(0)),
            identity_keys: Keypair::generate_ed25519(),
            nickname: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a mesh node with default configuration
    pub fn default() -> Self {
        Self::new(WasmMeshConfig::default())
    }

    /// Start the mesh node
    pub fn start(&self) -> Result<(), MeshError> {
        let mut state = self.state.write();
        *state = WasmMeshState::Active;
        Ok(())
    }

    /// Stop the mesh node
    pub fn stop(&self) {
        let mut state = self.state.write();
        *state = WasmMeshState::Shutdown;
    }

    /// Get current state
    pub fn state(&self) -> WasmMeshState {
        *self.state.read()
    }

    /// Get identity info (matching Core's IdentityInfo structure conceptually)
    pub fn get_identity_info(&self) -> (String, String, Option<String>) {
        let peer_id = self.identity_keys.public().to_peer_id().to_string();
        let pub_key = hex::encode(self.identity_keys.public().to_protobuf_encoding());
        let nickname = self.nickname.read().clone();
        (peer_id, pub_key, nickname)
    }

    /// Set nickname
    pub fn set_nickname(&self, name: String) {
        *self.nickname.write() = Some(name);
    }

    /// Store a message envelope locally
    pub fn store_message(&self, envelope: Vec<u8>) -> Result<bool, MeshError> {
        let state = self.state.read();
        if *state != WasmMeshState::Active && *state != WasmMeshState::Syncing {
            return Err(MeshError::NotActive);
        }
        drop(state);

        if envelope.len() < 20 {
            return Err(MeshError::InvalidEnvelope);
        }

        // Extract message ID (first 16 bytes of envelope)
        let message_id: [u8; 16] = envelope[..16]
            .try_into()
            .map_err(|_| MeshError::InvalidEnvelope)?;

        // Priority is byte 16 (0-255), use it for eviction priority
        let priority = if envelope.len() > 16 {
            envelope[16]
        } else {
            100
        };

        match self.store.insert(message_id, envelope, priority) {
            Ok(is_new) => {
                if is_new {
                    *self.message_count.write() += 1;
                }
                Ok(is_new)
            }
            Err(_) => Err(MeshError::StoreError("Failed to insert".to_string())),
        }
    }

    /// Get messages matching a recipient hint
    pub fn get_messages_for_hint(&self, hint: [u8; 4]) -> Vec<Vec<u8>> {
        self.store.messages_for_hint(&hint)
    }

    /// Get total message count
    pub fn message_count(&self) -> u64 {
        *self.message_count.read()
    }

    /// Process incoming message and generate outgoing responses
    pub fn relay_incoming(&self, data: &[u8]) -> Vec<Vec<u8>> {
        let state = self.state.read();
        if !self.config.relay_while_active || *state != WasmMeshState::Active {
            return Vec::new();
        }
        drop(state);

        let mut responses = Vec::new();

        // Attempt to store the message
        if let Ok(is_new) = self.store_message(data.to_vec()) {
            if is_new {
                // Create acknowledgment: [ACK][message_id] for first 16 bytes
                let mut ack = vec![0x01]; // ACK type
                if data.len() >= 16 {
                    ack.extend_from_slice(&data[..16]);
                    responses.push(ack);
                }
            }
        }

        responses
    }

    /// Perform maintenance tick
    pub fn tick(&self, now_ms: u64) -> bool {
        let state = self.state.read();
        if *state != WasmMeshState::Active {
            return false;
        }
        drop(state);

        let last_sync = *self.last_sync_ms.read();
        if now_ms - last_sync >= self.config.sync_interval_ms {
            // Trigger sync
            let mut state = self.state.write();
            *state = WasmMeshState::Syncing;

            let mut sync_time = self.last_sync_ms.write();
            *sync_time = now_ms;

            // Return to Active after sync
            *state = WasmMeshState::Active;

            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_node_creation() {
        let node = WasmMeshNode::default();
        assert_eq!(node.state(), WasmMeshState::Initializing);
    }

    #[test]
    fn test_mesh_start_stop() {
        let node = WasmMeshNode::default();
        node.start().unwrap();
        assert_eq!(node.state(), WasmMeshState::Active);

        node.stop();
        assert_eq!(node.state(), WasmMeshState::Shutdown);
    }

    #[test]
    fn test_cannot_store_when_inactive() {
        let node = WasmMeshNode::default();
        let envelope = vec![0u8; 32];

        let result = node.store_message(envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_store_message_when_active() {
        let node = WasmMeshNode::default();
        node.start().unwrap();

        let mut envelope = vec![0u8; 32];
        envelope[0] = 1; // Non-zero message ID
        envelope[16] = 100; // Priority

        let result = node.store_message(envelope);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_envelope_too_small() {
        let node = WasmMeshNode::default();
        node.start().unwrap();

        let envelope = vec![0u8; 10]; // Too small
        let result = node.store_message(envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_count_increases() {
        let node = WasmMeshNode::default();
        node.start().unwrap();

        assert_eq!(node.message_count(), 0);

        let mut envelope1 = vec![0u8; 32];
        envelope1[0] = 1;
        node.store_message(envelope1).unwrap();
        assert_eq!(node.message_count(), 1);

        let mut envelope2 = vec![0u8; 32];
        envelope2[0] = 2;
        node.store_message(envelope2).unwrap();
        assert_eq!(node.message_count(), 2);
    }

    #[test]
    fn test_relay_incoming_stores_message() {
        let node = WasmMeshNode::default();
        node.start().unwrap();

        let mut envelope = vec![0u8; 32];
        envelope[0] = 3;
        envelope[16] = 150; // Priority

        let responses = node.relay_incoming(&envelope);
        assert!(!responses.is_empty()); // Should generate ACK
        assert_eq!(node.message_count(), 1);
    }

    #[test]
    fn test_relay_inactive_returns_empty() {
        let node = WasmMeshNode::default();
        // Don't start node

        let envelope = vec![0u8; 32];
        let responses = node.relay_incoming(&envelope);
        assert!(responses.is_empty());
    }

    #[test]
    fn test_get_messages_for_hint() {
        let node = WasmMeshNode::default();
        node.start().unwrap();

        let mut env1 = vec![0u8; 32];
        env1[0] = 1;
        env1[4] = 0x12; // Part of hint after msg_id + priority
        env1[5] = 0x34;
        env1[6] = 0x56;
        env1[7] = 0x78;

        let mut env2 = vec![0u8; 32];
        env2[0] = 2;
        env2[4] = 0x12;
        env2[5] = 0x34;
        env2[6] = 0x56;
        env2[7] = 0x78;

        node.store_message(env1).unwrap();
        node.store_message(env2).unwrap();

        let hint = [0x12u8, 0x34, 0x56, 0x78];
        let matches = node.get_messages_for_hint(hint);
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_tick_triggers_sync() {
        let config = WasmMeshConfig {
            storage_quota_bytes: 50_000_000,
            relay_while_active: true,
            sync_interval_ms: 1000,
        };
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        assert!(!node.tick(500)); // Not yet time
        assert_eq!(node.state(), WasmMeshState::Active);

        assert!(node.tick(1500)); // Time to sync
        assert_eq!(node.state(), WasmMeshState::Active); // Returned to active
    }

    #[test]
    fn test_state_transitions() {
        let node = WasmMeshNode::default();

        assert_eq!(node.state(), WasmMeshState::Initializing);

        node.start().unwrap();
        assert_eq!(node.state(), WasmMeshState::Active);

        // Simulate sync
        let config = WasmMeshConfig {
            storage_quota_bytes: 50_000_000,
            relay_while_active: true,
            sync_interval_ms: 100,
        };
        let node2 = WasmMeshNode::new(config);
        node2.start().unwrap();
        node2.tick(150);
        assert_eq!(node2.state(), WasmMeshState::Active);
    }

    #[test]
    fn test_relay_relay_disabled() {
        let config = WasmMeshConfig {
            storage_quota_bytes: 50_000_000,
            relay_while_active: false,
            sync_interval_ms: 30_000,
        };
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        let envelope = vec![0u8; 32];
        let responses = node.relay_incoming(&envelope);
        assert!(responses.is_empty());
    }

    #[test]
    fn test_duplicate_message_not_counted() {
        let node = WasmMeshNode::default();
        node.start().unwrap();

        let mut envelope = vec![0u8; 32];
        envelope[0] = 4;

        node.store_message(envelope.clone()).unwrap();
        assert_eq!(node.message_count(), 1);

        // Same envelope again
        let result = node.store_message(envelope).unwrap();
        assert!(!result); // Not new
        assert_eq!(node.message_count(), 1); // Count unchanged
    }

    #[test]
    fn test_relay_configuration() {
        let config = WasmMeshConfig {
            storage_quota_bytes: 25_000_000,
            relay_while_active: true,
            sync_interval_ms: 60_000,
        };
        let node = WasmMeshNode::new(config);

        assert_eq!(node.config.storage_quota_bytes, 25_000_000);
        assert_eq!(node.config.sync_interval_ms, 60_000);
        assert!(node.config.relay_while_active);
    }

    #[test]
    fn test_identity_management() {
        let node = WasmMeshNode::default();
        let (peer_id, _, nick) = node.get_identity_info();

        assert!(!peer_id.is_empty());
        assert!(nick.is_none());

        node.set_nickname("WasmUser".to_string());
        let (_, _, nick) = node.get_identity_info();
        assert_eq!(nick, Some("WasmUser".to_string()));
    }
}
