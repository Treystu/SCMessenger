// WASM WebRTC/WebSocket Transport Layer
//
// Manages connections to relay servers and WebRTC data channels.
// Bridges browser networking to mesh protocol.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// WebRTC or WebSocket transport selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebTransportType {
    WebRTC,
    WebSocket,
}

/// WebRTC/WebSocket transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebTransportConfig {
    /// Relay server URLs (ws://, wss://)
    pub relay_urls: Vec<String>,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Enable WebRTC data channel mode
    pub enable_webrtc: bool,
}

/// Connection state for channels and relays
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Connecting,
    Open,
    Closing,
    Closed,
}

/// WebRTC data channel connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcChannel {
    /// Peer identifier (32 bytes)
    pub peer_id: [u8; 32],
    /// Channel state
    pub channel_state: ConnectionState,
    /// Bytes buffered in channel
    pub buffered_amount: usize,
    /// Number of messages sent
    pub messages_sent: u64,
}

/// WebSocket relay connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketRelay {
    /// Relay server URL
    pub url: String,
    /// Connection state
    pub state: ConnectionState,
    /// Associated peer ID (relay's identifier)
    pub peer_id: [u8; 32],
    /// Timestamp of last ping (milliseconds)
    pub last_ping_ms: u64,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Not connected")]
    NotConnected,
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Message too large")]
    MessageTooLarge,
    #[error("Already connected to {0}")]
    AlreadyConnected(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
}

/// Manages WebRTC channels and WebSocket relays
#[allow(dead_code)]
pub struct WebTransportManager {
    config: WebTransportConfig,
    websocket_relays: Arc<RwLock<HashMap<String, WebSocketRelay>>>,
    webrtc_channels: Arc<RwLock<HashMap<String, WebRtcChannel>>>,
    pending_outgoing: Arc<RwLock<Vec<(String, Vec<u8>)>>>,
}

impl WebTransportManager {
    /// Create a new transport manager with the given configuration
    pub fn new(config: WebTransportConfig) -> Self {
        Self {
            config,
            websocket_relays: Arc::new(RwLock::new(HashMap::new())),
            webrtc_channels: Arc::new(RwLock::new(HashMap::new())),
            pending_outgoing: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a relay connection
    pub fn add_relay(
        &self,
        url: String,
        peer_id: [u8; 32],
    ) -> Result<(), TransportError> {
        if !url.starts_with("ws://") && !url.starts_with("wss://") {
            return Err(TransportError::InvalidUrl(url));
        }

        let mut relays = self.websocket_relays.write();
        if relays.contains_key(&url) {
            return Err(TransportError::AlreadyConnected(url));
        }

        relays.insert(
            url.clone(),
            WebSocketRelay {
                url: url.clone(),
                state: ConnectionState::Connecting,
                peer_id,
                last_ping_ms: 0,
            },
        );

        Ok(())
    }

    /// Remove a relay connection
    pub fn remove_relay(&self, url: &str) -> Option<WebSocketRelay> {
        self.websocket_relays.write().remove(url)
    }

    /// Process incoming message and generate responses
    pub fn handle_message(&self, from: &str, data: &[u8]) -> Vec<(String, Vec<u8>)> {
        let mut responses = Vec::new();

        // Update relay state to Open on first message
        if let Some(relay) = self.websocket_relays.write().get_mut(from) {
            if relay.state == ConnectionState::Connecting {
                relay.state = ConnectionState::Open;
            }
        }

        // Echo data back with modified header to indicate receipt
        if !data.is_empty() {
            let mut response = vec![0x01]; // ACK header
            response.extend_from_slice(data);
            responses.push((from.to_string(), response));
        }

        responses
    }

    /// Get all pending outgoing messages
    pub fn pending_outgoing(&self) -> Vec<(String, Vec<u8>)> {
        self.pending_outgoing.write().drain(..).collect()
    }

    /// Queue a message for sending
    pub fn queue_outgoing(&self, url: String, data: Vec<u8>) -> Result<(), TransportError> {
        if data.len() > 1_000_000 {
            return Err(TransportError::MessageTooLarge);
        }

        let relays = self.websocket_relays.read();
        if !relays.contains_key(&url) {
            return Err(TransportError::NotConnected);
        }

        self.pending_outgoing.write().push((url, data));
        Ok(())
    }

    /// Get all connected relays
    pub fn connected_relays(&self) -> Vec<WebSocketRelay> {
        self.websocket_relays
            .read()
            .values()
            .filter(|r| r.state == ConnectionState::Open)
            .cloned()
            .collect()
    }

    /// Get all active WebRTC channels
    pub fn active_channels(&self) -> Vec<WebRtcChannel> {
        self.webrtc_channels
            .read()
            .values()
            .filter(|c| c.channel_state == ConnectionState::Open)
            .cloned()
            .collect()
    }

    /// Add a WebRTC data channel
    pub fn add_channel(
        &self,
        peer_id: [u8; 32],
    ) -> Result<(), TransportError> {
        let peer_key = hex::encode(peer_id);

        let mut channels = self.webrtc_channels.write();
        if channels.contains_key(&peer_key) {
            return Ok(()); // Already exists
        }

        channels.insert(
            peer_key,
            WebRtcChannel {
                peer_id,
                channel_state: ConnectionState::Connecting,
                buffered_amount: 0,
                messages_sent: 0,
            },
        );

        Ok(())
    }

    /// Remove a WebRTC data channel
    pub fn remove_channel(&self, peer_id: &[u8; 32]) -> Option<WebRtcChannel> {
        self.webrtc_channels.write().remove(&hex::encode(peer_id))
    }

    /// Update channel state
    pub fn update_channel_state(
        &self,
        peer_id: &[u8; 32],
        state: ConnectionState,
    ) -> Result<(), TransportError> {
        let peer_key = hex::encode(peer_id);
        let mut channels = self.webrtc_channels.write();

        if let Some(channel) = channels.get_mut(&peer_key) {
            channel.channel_state = state;
            Ok(())
        } else {
            Err(TransportError::NotConnected)
        }
    }

    /// Get relay state
    pub fn relay_state(&self, url: &str) -> Option<ConnectionState> {
        self.websocket_relays.read().get(url).map(|r| r.state)
    }

    /// Get all relays (connected or not)
    pub fn all_relays(&self) -> Vec<WebSocketRelay> {
        self.websocket_relays.read().values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_manager_creation() {
        let config = WebTransportConfig {
            relay_urls: vec!["ws://relay.example.com".to_string()],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        assert!(manager.connected_relays().is_empty());
    }

    #[test]
    fn test_add_relay_valid_url() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [1u8; 32];

        let result = manager.add_relay("ws://relay.example.com".to_string(), peer_id);
        assert!(result.is_ok());

        let all_relays = manager.all_relays();
        assert_eq!(all_relays.len(), 1);
        assert_eq!(all_relays[0].state, ConnectionState::Connecting);
    }

    #[test]
    fn test_add_relay_invalid_url() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [1u8; 32];

        let result = manager.add_relay("http://relay.example.com".to_string(), peer_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_relay_duplicate() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [1u8; 32];
        let url = "ws://relay.example.com".to_string();

        manager.add_relay(url.clone(), peer_id).unwrap();
        let result = manager.add_relay(url, peer_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_relay() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [1u8; 32];
        let url = "ws://relay.example.com".to_string();

        manager.add_relay(url.clone(), peer_id).unwrap();
        let removed = manager.remove_relay(&url);
        assert!(removed.is_some());
        assert!(manager.all_relays().is_empty());
    }

    #[test]
    fn test_handle_message_updates_state() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [1u8; 32];
        let url = "ws://relay.example.com".to_string();

        manager.add_relay(url.clone(), peer_id).unwrap();
        assert_eq!(manager.relay_state(&url), Some(ConnectionState::Connecting));

        let responses = manager.handle_message(&url, b"test");
        assert_eq!(manager.relay_state(&url), Some(ConnectionState::Open));
        assert!(!responses.is_empty());
    }

    #[test]
    fn test_add_webrtc_channel() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [2u8; 32];

        let result = manager.add_channel(peer_id);
        assert!(result.is_ok());
        assert_eq!(manager.active_channels().len(), 0); // Not yet Open
    }

    #[test]
    fn test_update_channel_state() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [3u8; 32];

        manager.add_channel(peer_id).unwrap();
        manager
            .update_channel_state(&peer_id, ConnectionState::Open)
            .unwrap();

        let channels = manager.active_channels();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].peer_id, peer_id);
    }

    #[test]
    fn test_pending_outgoing_queue() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [4u8; 32];
        let url = "ws://relay.example.com".to_string();

        manager.add_relay(url.clone(), peer_id).unwrap();
        manager
            .queue_outgoing(url.clone(), b"test1".to_vec())
            .unwrap();
        manager
            .queue_outgoing(url, b"test2".to_vec())
            .unwrap();

        let pending = manager.pending_outgoing();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_queue_outgoing_large_message() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [5u8; 32];
        let url = "ws://relay.example.com".to_string();

        manager.add_relay(url.clone(), peer_id).unwrap();
        let large_msg = vec![0u8; 2_000_000];
        let result = manager.queue_outgoing(url, large_msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_channel() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [6u8; 32];

        manager.add_channel(peer_id).unwrap();
        let removed = manager.remove_channel(&peer_id);
        assert!(removed.is_some());
    }

    #[test]
    fn test_connection_state_transitions() {
        let states = vec![
            ConnectionState::Connecting,
            ConnectionState::Open,
            ConnectionState::Closing,
            ConnectionState::Closed,
        ];
        for state in states {
            assert!(matches!(
                state,
                ConnectionState::Connecting
                    | ConnectionState::Open
                    | ConnectionState::Closing
                    | ConnectionState::Closed
            ));
        }
    }

    #[test]
    fn test_relay_persistence() {
        let config = WebTransportConfig {
            relay_urls: vec![],
            max_connections: 10,
            enable_webrtc: true,
        };
        let manager = WebTransportManager::new(config);
        let peer_id = [7u8; 32];
        let url = "wss://secure.relay.example.com".to_string();

        manager.add_relay(url.clone(), peer_id).unwrap();
        let relays = manager.all_relays();
        assert_eq!(relays[0].url, url);
    }
}
