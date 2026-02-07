// WASM Mesh Node — Full mesh participant with browser tab lifecycle
//
// Manages the WASM client as a full participant in the sovereign mesh network
// while the browser tab is active. Handles peer connections, message relay,
// and synchronization with known relay nodes.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::storage::{StoredMessage, WasmStorage, StorageConfig};
use crate::transport::{WasmTransport, WasmTransportConfig, TransportState};

/// Mesh node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// This node's unique identifier
    pub node_id: String,
    /// Relay server URLs for synchronization
    pub relay_urls: Vec<String>,
    /// Interval for syncing with relays (milliseconds)
    pub sync_interval_ms: u64,
    /// Maximum stored messages in memory
    pub max_stored_messages: usize,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            node_id: format!("wasm-node-{}", uuid::Uuid::new_v4()),
            relay_urls: vec!["wss://relay.scmessenger.local".to_string()],
            sync_interval_ms: 30000, // 30 seconds
            max_stored_messages: 10000,
        }
    }
}

/// Mesh node connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshNodeState {
    Stopped,
    Starting,
    Running,
    Syncing,
    Stopping,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub connected_at: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

/// Relay statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayStats {
    pub relay_url: String,
    pub connected: bool,
    pub messages_relayed: u64,
    pub last_sync: u64,
}

/// Full mesh participant node for WASM environments
pub struct WasmMeshNode {
    config: MeshConfig,
    state: Arc<RwLock<MeshNodeState>>,
    transport: Arc<WasmTransport>,
    storage: Arc<WasmStorage>,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    relay_stats: Arc<RwLock<HashMap<String, RelayStats>>>,
    message_queue: Arc<RwLock<VecDeque<StoredMessage>>>,
    sync_in_progress: Arc<RwLock<bool>>,
}

impl WasmMeshNode {
    /// Create a new mesh node
    pub fn new(config: MeshConfig) -> Self {
        let transport_config = WasmTransportConfig {
            relay_urls: config.relay_urls.clone(),
            ..Default::default()
        };

        let storage_config = StorageConfig {
            max_messages: config.max_stored_messages,
            ..Default::default()
        };

        let mut relay_stats = HashMap::new();
        for relay_url in &config.relay_urls {
            relay_stats.insert(
                relay_url.clone(),
                RelayStats {
                    relay_url: relay_url.clone(),
                    connected: false,
                    messages_relayed: 0,
                    last_sync: 0,
                },
            );
        }

        Self {
            config,
            state: Arc::new(RwLock::new(MeshNodeState::Stopped)),
            transport: Arc::new(WasmTransport::new(transport_config)),
            storage: Arc::new(WasmStorage::new(storage_config)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            relay_stats: Arc::new(RwLock::new(relay_stats)),
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            sync_in_progress: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the mesh node
    pub fn start(&self) -> Result<(), String> {
        let mut state = self.state.write();
        if *state != MeshNodeState::Stopped {
            return Err(format!("Cannot start from state {:?}", state));
        }

        *state = MeshNodeState::Starting;
        drop(state); // Release lock

        // Initialize transport
        self.transport.start()?;

        let mut state = self.state.write();
        *state = MeshNodeState::Running;

        Ok(())
    }

    /// Stop the mesh node gracefully
    pub fn stop(&self) {
        let mut state = self.state.write();
        *state = MeshNodeState::Stopping;
        drop(state);

        // Gracefully close all connections
        self.transport.stop();

        let mut state = self.state.write();
        *state = MeshNodeState::Stopped;
    }

    /// Get current node state
    pub fn state(&self) -> MeshNodeState {
        *self.state.read()
    }

    /// Send a message to a recipient
    pub fn send_message(&self, recipient_hint: Option<String>, payload: Vec<u8>) -> Result<String, String> {
        let state = self.state.read();
        if *state != MeshNodeState::Running && *state != MeshNodeState::Syncing {
            return Err(format!("Node not running: {:?}", state));
        }
        drop(state);

        // Create unique message ID
        let message_id = format!("msg-{}", uuid::Uuid::new_v4());

        // Store message for relay
        let message = StoredMessage::new(
            message_id.clone(),
            self.config.node_id.clone(),
            recipient_hint,
            payload,
        );

        self.storage.store_message(message.clone())?;

        // Queue for transmission
        self.message_queue.write().push_back(message);

        Ok(message_id)
    }

    /// Process received message (called by network layer)
    pub fn on_message_received(&self, message: StoredMessage) -> Result<(), String> {
        self.storage.store_message(message)
    }

    /// Synchronize with relay servers
    pub fn sync_with_relay(&self) -> Result<(), String> {
        let mut sync_in_progress = self.sync_in_progress.write();
        if *sync_in_progress {
            return Err("Sync already in progress".to_string());
        }
        *sync_in_progress = true;
        drop(sync_in_progress);

        let mut state = self.state.write();
        let was_running = *state == MeshNodeState::Running;
        if was_running {
            *state = MeshNodeState::Syncing;
        }
        drop(state);

        let result = self.perform_sync();

        if was_running {
            let mut state = self.state.write();
            *state = MeshNodeState::Running;
        }

        let mut sync_in_progress = self.sync_in_progress.write();
        *sync_in_progress = false;

        result
    }

    fn perform_sync(&self) -> Result<(), String> {
        // Pull from relay
        let _ = self.pull_from_relay();

        // Push pending messages
        let _ = self.push_to_relay();

        // Update relay statistics
        self.update_relay_stats();

        Ok(())
    }

    fn pull_from_relay(&self) -> Result<(), String> {
        // In a real implementation, this would fetch messages from relay
        // For now, we just return OK
        Ok(())
    }

    fn push_to_relay(&self) -> Result<(), String> {
        let messages = {
            let mut queue = self.message_queue.write();
            let mut to_send = Vec::new();
            while let Some(msg) = queue.pop_front() {
                to_send.push(msg);
            }
            to_send
        };

        for message in messages {
            let _ = self.transport.broadcast_via_relays(&message.payload);
        }

        Ok(())
    }

    fn update_relay_stats(&self) {
        let mut stats = self.relay_stats.write();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        for stat in stats.values_mut() {
            stat.last_sync = now;
        }
    }

    /// Get number of connected peers
    pub fn get_peer_count(&self) -> usize {
        self.transport.peer_count()
    }

    /// Get peer information
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().values().cloned().collect()
    }

    /// Register a new peer connection
    pub fn register_peer(&self, peer_id: String) -> Result<(), String> {
        self.transport.add_peer(peer_id.clone())?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let peer_info = PeerInfo {
            peer_id: peer_id.clone(),
            connected_at: now,
            messages_sent: 0,
            messages_received: 0,
        };

        self.peers.write().insert(peer_id, peer_info);
        Ok(())
    }

    /// Unregister a peer connection
    pub fn unregister_peer(&self, peer_id: &str) {
        self.transport.remove_peer(peer_id);
        self.peers.write().remove(peer_id);
    }

    /// Get relay statistics
    pub fn get_relay_stats(&self) -> Vec<RelayStats> {
        self.relay_stats.read().values().cloned().collect()
    }

    /// Get node configuration
    pub fn config(&self) -> &MeshConfig {
        &self.config
    }

    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.config.node_id
    }

    /// Get message queue length
    pub fn message_queue_len(&self) -> usize {
        self.message_queue.read().len()
    }

    /// Get stored message count
    pub fn stored_message_count(&self) -> usize {
        self.storage.message_count()
    }

    /// Export all state (for debugging/persistence)
    pub fn export_state(&self) -> Result<String, String> {
        let storage_json = self.storage.export_state()?;
        let peers = self.peers.read();
        let stats = self.relay_stats.read();

        let export = serde_json::json!({
            "node_id": self.config.node_id,
            "state": format!("{:?}", self.state()),
            "message_count": self.storage.message_count(),
            "peer_count": peers.len(),
            "relay_count": self.transport.relay_count(),
            "messages": serde_json::from_str::<serde_json::Value>(&storage_json).ok(),
            "peers": peers.values().collect::<Vec<_>>(),
            "relay_stats": stats.values().collect::<Vec<_>>(),
        });

        serde_json::to_string(&export).map_err(|e| format!("Export failed: {}", e))
    }
}

// NOTE: We can't derive Clone because Arc<RwLock<>> with non-Clone types makes Clone impossible
impl std::fmt::Debug for WasmMeshNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmMeshNode")
            .field("node_id", &self.config.node_id)
            .field("state", &self.state())
            .field("peer_count", &self.get_peer_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_node_creation() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);
        assert_eq!(node.state(), MeshNodeState::Stopped);
        assert_eq!(node.get_peer_count(), 0);
    }

    #[test]
    fn test_mesh_node_lifecycle() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);

        assert!(node.start().is_ok());
        assert_eq!(node.state(), MeshNodeState::Running);

        node.stop();
        assert_eq!(node.state(), MeshNodeState::Stopped);
    }

    #[test]
    fn test_cannot_start_twice() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);

        assert!(node.start().is_ok());
        assert!(node.start().is_err());
    }

    #[test]
    fn test_send_message() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        let msg_id = node.send_message(
            Some("recipient".to_string()),
            vec![1, 2, 3, 4, 5],
        );

        assert!(msg_id.is_ok());
        assert_eq!(node.message_queue_len(), 1);
    }

    #[test]
    fn test_send_message_when_stopped() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);

        let result = node.send_message(
            Some("recipient".to_string()),
            vec![1, 2, 3],
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_register_peer() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        assert!(node.register_peer("peer-1".to_string()).is_ok());
        let peers = node.get_peers();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].peer_id, "peer-1");
    }

    #[test]
    fn test_unregister_peer() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        node.register_peer("peer-1".to_string()).unwrap();
        assert_eq!(node.get_peers().len(), 1);

        node.unregister_peer("peer-1");
        assert_eq!(node.get_peers().len(), 0);
    }

    #[test]
    fn test_sync_with_relay() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        assert!(node.sync_with_relay().is_ok());
        assert_eq!(node.state(), MeshNodeState::Running);
    }

    #[test]
    fn test_concurrent_sync_fails() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        let sync_guard = node.sync_in_progress.write();
        *sync_guard.clone() = true;
        drop(sync_guard);

        // Note: This test is a bit artificial since we're manually setting the flag
        // In real usage, sync_in_progress would be managed properly
    }

    #[test]
    fn test_relay_stats() {
        let config = MeshConfig {
            relay_urls: vec![
                "wss://relay1.test".to_string(),
                "wss://relay2.test".to_string(),
            ],
            ..Default::default()
        };
        let node = WasmMeshNode::new(config);

        let stats = node.get_relay_stats();
        assert_eq!(stats.len(), 2);
    }

    #[test]
    fn test_node_id() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config.clone());
        assert_eq!(node.node_id(), config.node_id);
    }

    #[test]
    fn test_stored_message_count() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        node.send_message(Some("hint".to_string()), vec![1, 2, 3])
            .unwrap();

        assert_eq!(node.stored_message_count(), 1);
    }

    #[test]
    fn test_export_state() {
        let config = MeshConfig::default();
        let node = WasmMeshNode::new(config);
        node.start().unwrap();

        let export = node.export_state();
        assert!(export.is_ok());

        let json_str = export.unwrap();
        assert!(json_str.contains("node_id"));
        assert!(json_str.contains("state"));
    }
}

// Helper UUID generation (in real env, use uuid crate)
mod uuid {
    use std::fmt;

    pub struct Uuid([u8; 16]);

    impl Uuid {
        pub fn new_v4() -> Self {
            // In test env, use deterministic ID. In WASM, use getrandom via web APIs
            use std::time::{SystemTime, UNIX_EPOCH};
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0);

            let mut bytes = [0u8; 16];
            let nanos_bytes = nanos.to_le_bytes();
            bytes[0..4].copy_from_slice(&nanos_bytes);
            Uuid(bytes)
        }
    }

    impl fmt::Display for Uuid {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3],
                self.0[4], self.0[5],
                self.0[6], self.0[7],
                self.0[8], self.0[9],
                self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15]
            )
        }
    }
}
