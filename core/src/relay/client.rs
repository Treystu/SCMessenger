//! Relay Client — connects to relay peers and synchronizes messages

use super::protocol::{RelayCapability, RelayMessage, PROTOCOL_VERSION};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};

/// Relay client configuration
#[derive(Debug, Clone)]
pub struct RelayClientConfig {
    /// List of known relay addresses to try connecting to
    pub known_relays: Vec<String>,
    /// Interval for reconnection attempts (with exponential backoff)
    pub reconnect_interval: Duration,
    /// Interval for pulling stored envelopes from relays
    pub pull_interval: Duration,
}

impl Default for RelayClientConfig {
    fn default() -> Self {
        Self {
            known_relays: Vec::new(),
            reconnect_interval: Duration::from_secs(5),
            pull_interval: Duration::from_secs(30),
        }
    }
}

/// Connection state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Attempting to connect
    Connecting,
    /// Connected, performing handshake
    Handshaking,
    /// Fully connected and authenticated
    Connected,
    /// Connection was closed
    Disconnected,
}

/// A relay connection
#[derive(Debug)]
pub struct RelayConnection {
    /// The relay's address
    pub address: String,
    /// Current connection state
    pub state: ConnectionState,
    /// Relay peer ID (if known)
    pub relay_peer_id: Option<String>,
    /// Relay capabilities
    pub relay_capabilities: Option<RelayCapability>,
    /// Last time we were successfully connected
    pub last_connected_at: Option<u64>,
}

impl RelayConnection {
    /// Create a new relay connection
    pub fn new(address: String) -> Self {
        Self {
            address,
            state: ConnectionState::Disconnected,
            relay_peer_id: None,
            relay_capabilities: None,
            last_connected_at: None,
        }
    }

    /// Update connection state
    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
        if state == ConnectionState::Connected {
            self.last_connected_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }
    }

    /// Check if connection is active
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }
}

/// Relay client error types
#[derive(Debug, Error)]
pub enum RelayClientError {
    #[error("Not connected to any relay")]
    NotConnected,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("No known relays configured")]
    NoKnownRelays,
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Message error: {0}")]
    MessageError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Relay client for connecting to relay servers
pub struct RelayClient {
    /// Configuration
    config: RelayClientConfig,
    /// Our peer ID
    our_peer_id: String,
    /// Our capabilities
    our_capabilities: RelayCapability,
    /// Active connections
    connections: Arc<RwLock<Vec<RelayConnection>>>,
    /// Last pull timestamp per relay address
    last_pull: Arc<RwLock<HashMap<String, u64>>>,
    /// Active network sockets by relay address
    sockets: Arc<RwLock<HashMap<String, Arc<Mutex<TcpStream>>>>>,
}

impl RelayClient {
    /// Create a new relay client
    pub fn new(peer_id: String, config: RelayClientConfig) -> Self {
        Self {
            config,
            our_peer_id: peer_id,
            our_capabilities: RelayCapability::full_relay(),
            connections: Arc::new(RwLock::new(Vec::new())),
            last_pull: Arc::new(RwLock::new(HashMap::new())),
            sockets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set our capabilities
    pub fn set_capabilities(&mut self, capabilities: RelayCapability) {
        self.our_capabilities = capabilities;
    }

    /// Generate a handshake message
    pub fn create_handshake(&self) -> RelayMessage {
        RelayMessage::Handshake {
            version: PROTOCOL_VERSION,
            peer_id: self.our_peer_id.clone(),
            capabilities: self.our_capabilities,
        }
    }

    /// Connect to a specific relay
    pub async fn connect(&self, relay_address: String) -> Result<RelayConnection, RelayClientError> {
        let mut connection = RelayConnection::new(relay_address.clone());
        connection.set_state(ConnectionState::Connecting);
        let dial_addr = relay_address
            .strip_prefix("tcp://")
            .unwrap_or(&relay_address)
            .to_string();

        let stream = TcpStream::connect(&dial_addr)
            .await
            .map_err(|e| RelayClientError::ConnectionFailed(e.to_string()))?;
        let stream = Arc::new(Mutex::new(stream));
        connection.set_state(ConnectionState::Handshaking);

        let handshake = self.create_handshake();
        let response = self.send_and_receive_raw(&stream, handshake).await?;
        self.complete_handshake(&mut connection, response)?;

        self.sockets
            .write()
            .await
            .insert(relay_address, Arc::clone(&stream));
        Ok(connection)
    }

    /// Complete handshake with a relay
    pub fn complete_handshake(
        &self,
        connection: &mut RelayConnection,
        response: RelayMessage,
    ) -> Result<(), RelayClientError> {
        match response {
            RelayMessage::HandshakeAck {
                version,
                peer_id,
                capabilities,
            } => {
                if version != PROTOCOL_VERSION {
                    return Err(RelayClientError::HandshakeFailed(
                        "Version mismatch".to_string(),
                    ));
                }

                connection.relay_peer_id = Some(peer_id);
                connection.relay_capabilities = Some(capabilities);
                connection.set_state(ConnectionState::Connected);

                Ok(())
            }
            _ => Err(RelayClientError::HandshakeFailed(
                "Invalid response type".to_string(),
            )),
        }
    }

    /// Push envelopes to a relay for store-and-forward
    pub async fn push_envelopes(
        &self,
        envelopes: Vec<Vec<u8>>,
    ) -> Result<(u32, u32), RelayClientError> {
        let connections = self.connections.read().await;

        // Find a connected relay
        let connection = connections
            .iter()
            .find(|c| c.is_connected())
            .ok_or(RelayClientError::NotConnected)?;

        // Check if relay supports storage
        let capabilities = connection
            .relay_capabilities
            .as_ref()
            .ok_or(RelayClientError::NotConnected)?;

        if !capabilities.is_store() {
            return Err(RelayClientError::MessageError(
                "Relay does not support store-and-forward".to_string(),
            ));
        }

        let _message = RelayMessage::StoreRequest { envelopes };
        let socket = self
            .sockets
            .read()
            .await
            .get(&connection.address)
            .cloned()
            .ok_or_else(|| RelayClientError::ConnectionFailed("No active socket".to_string()))?;

        match self.send_and_receive_raw(&socket, _message).await? {
            RelayMessage::StoreAck { accepted, rejected } => Ok((accepted, rejected)),
            other => Err(RelayClientError::MessageError(format!(
                "Unexpected response to StoreRequest: {}",
                other.message_type()
            ))),
        }
    }

    /// Pull stored envelopes from relays
    pub async fn pull_envelopes(&self, since_timestamp: u64) -> Result<Vec<Vec<u8>>, RelayClientError> {
        let connections = self.connections.read().await;

        // Find a connected relay
        let connection = connections
            .iter()
            .find(|c| c.is_connected())
            .ok_or(RelayClientError::NotConnected)?;

        // Check if relay supports storage
        let capabilities = connection
            .relay_capabilities
            .as_ref()
            .ok_or(RelayClientError::NotConnected)?;

        if !capabilities.is_store() {
            return Err(RelayClientError::MessageError(
                "Relay does not support store-and-forward".to_string(),
            ));
        }

        let message = RelayMessage::PullRequest {
            since_timestamp,
            hints: Vec::new(),
        };

        // Update last pull timestamp
        let mut last_pull = self.last_pull.write().await;
        last_pull.insert(connection.address.clone(), since_timestamp);

        let socket = self
            .sockets
            .read()
            .await
            .get(&connection.address)
            .cloned()
            .ok_or_else(|| RelayClientError::ConnectionFailed("No active socket".to_string()))?;

        match self.send_and_receive_raw(&socket, message).await? {
            RelayMessage::PullResponse { envelopes } => Ok(envelopes),
            other => Err(RelayClientError::MessageError(format!(
                "Unexpected response to PullRequest: {}",
                other.message_type()
            ))),
        }
    }

    /// Get current number of active connections
    pub async fn active_connections(&self) -> usize {
        self.connections
            .read()
            .await
            .iter()
            .filter(|c| c.is_connected())
            .count()
    }

    /// Register a successful connection
    pub async fn add_connection(&self, connection: RelayConnection) {
        self.connections.write().await.push(connection);
    }

    /// Remove a connection
    pub async fn remove_connection(&self, relay_address: &str) {
        let mut connections = self.connections.write().await;
        connections.retain(|c| c.address != relay_address);
        self.sockets.write().await.remove(relay_address);
    }

    /// Send a ping to all relays
    pub async fn send_ping(&self) -> Result<(), RelayClientError> {
        let connections = self.connections.read().await;

        if connections.is_empty() {
            return Err(RelayClientError::NotConnected);
        }

        for connection in connections.iter().filter(|c| c.is_connected()) {
            if let Some(socket) = self.sockets.read().await.get(&connection.address).cloned() {
                match self.send_and_receive_raw(&socket, RelayMessage::Ping).await? {
                    RelayMessage::Pong => {}
                    other => {
                        return Err(RelayClientError::MessageError(format!(
                            "Unexpected ping response: {}",
                            other.message_type()
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// Get relay addresses that should be used for pulling
    pub fn get_relay_addresses(&self) -> Vec<String> {
        self.config.known_relays.clone()
    }

    /// Calculate exponential backoff for reconnection
    pub fn backoff_duration(&self, attempt: u32) -> Duration {
        let base_ms = self.config.reconnect_interval.as_millis() as u64;
        let backoff_ms = base_ms * (2u64.pow(std::cmp::min(attempt, 5)));
        Duration::from_millis(std::cmp::min(backoff_ms, 60000)) // Cap at 60 seconds
    }

    async fn send_and_receive_raw(
        &self,
        socket: &Arc<Mutex<TcpStream>>,
        message: RelayMessage,
    ) -> Result<RelayMessage, RelayClientError> {
        let payload = message
            .to_bytes()
            .map_err(|e| RelayClientError::SerializationError(e.to_string()))?;

        let mut stream = socket.lock().await;
        let len = payload.len() as u32;
        stream
            .write_u32(len)
            .await
            .map_err(|e| RelayClientError::ConnectionFailed(e.to_string()))?;
        stream
            .write_all(&payload)
            .await
            .map_err(|e| RelayClientError::ConnectionFailed(e.to_string()))?;
        stream
            .flush()
            .await
            .map_err(|e| RelayClientError::ConnectionFailed(e.to_string()))?;

        let response_len = stream
            .read_u32()
            .await
            .map_err(|e| RelayClientError::ConnectionFailed(e.to_string()))? as usize;
        if response_len == 0 || response_len > (16 * 1024 * 1024) {
            return Err(RelayClientError::MessageError(
                "Invalid response frame length".to_string(),
            ));
        }
        let mut response_buf = vec![0u8; response_len];
        stream
            .read_exact(&mut response_buf)
            .await
            .map_err(|e| RelayClientError::ConnectionFailed(e.to_string()))?;
        RelayMessage::from_bytes(&response_buf)
            .map_err(|e| RelayClientError::SerializationError(e.to_string()))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> RelayClient {
        RelayClient::new(
            "test_peer_id".to_string(),
            RelayClientConfig::default(),
        )
    }

    #[test]
    fn test_relay_client_creation() {
        let client = test_client();
        assert_eq!(client.our_peer_id, "test_peer_id");
        assert!(client.our_capabilities.is_relay());
    }

    #[test]
    fn test_create_handshake() {
        let client = test_client();
        let msg = client.create_handshake();

        match msg {
            RelayMessage::Handshake {
                version,
                peer_id,
                capabilities,
            } => {
                assert_eq!(version, PROTOCOL_VERSION);
                assert_eq!(peer_id, "test_peer_id");
                assert!(capabilities.is_relay());
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_set_capabilities() {
        let mut client = test_client();
        let mobile_cap = RelayCapability::mobile();

        client.set_capabilities(mobile_cap);
        assert_eq!(client.our_capabilities, mobile_cap);
    }

    #[tokio::test]
    async fn test_connect_to_relay() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let msg_len = stream.read_u32().await.unwrap() as usize;
            let mut buf = vec![0u8; msg_len];
            stream.read_exact(&mut buf).await.unwrap();
            let msg = RelayMessage::from_bytes(&buf).unwrap();
            assert!(matches!(msg, RelayMessage::Handshake { .. }));
            let ack = RelayMessage::HandshakeAck {
                version: PROTOCOL_VERSION,
                peer_id: "relay-peer".to_string(),
                capabilities: RelayCapability::full_relay(),
            };
            let ack_bytes = ack.to_bytes().unwrap();
            stream.write_u32(ack_bytes.len() as u32).await.unwrap();
            stream.write_all(&ack_bytes).await.unwrap();
            stream.flush().await.unwrap();
        });

        let client = test_client();
        let result = client.connect(addr.to_string()).await;

        assert!(result.is_ok());
        let connection = result.unwrap();
        assert_eq!(connection.address, addr.to_string());
        assert_eq!(connection.state, ConnectionState::Connected);
    }

    #[test]
    fn test_relay_connection_creation() {
        let conn = RelayConnection::new("127.0.0.1:8080".to_string());
        assert_eq!(conn.state, ConnectionState::Disconnected);
        assert!(!conn.is_connected());
        assert!(conn.relay_peer_id.is_none());
    }

    #[test]
    fn test_relay_connection_state_transitions() {
        let mut conn = RelayConnection::new("127.0.0.1:8080".to_string());

        conn.set_state(ConnectionState::Connecting);
        assert_eq!(conn.state, ConnectionState::Connecting);
        assert!(!conn.is_connected());

        conn.set_state(ConnectionState::Handshaking);
        assert_eq!(conn.state, ConnectionState::Handshaking);
        assert!(!conn.is_connected());

        conn.set_state(ConnectionState::Connected);
        assert_eq!(conn.state, ConnectionState::Connected);
        assert!(conn.is_connected());
        assert!(conn.last_connected_at.is_some());
    }

    #[test]
    fn test_complete_handshake_success() {
        let client = test_client();
        let mut connection = RelayConnection::new("127.0.0.1:8080".to_string());

        let response = RelayMessage::HandshakeAck {
            version: PROTOCOL_VERSION,
            peer_id: "relay1".to_string(),
            capabilities: RelayCapability::full_relay(),
        };

        let result = client.complete_handshake(&mut connection, response);
        assert!(result.is_ok());
        assert!(connection.is_connected());
        assert_eq!(connection.relay_peer_id, Some("relay1".to_string()));
    }

    #[test]
    fn test_complete_handshake_version_mismatch() {
        let client = test_client();
        let mut connection = RelayConnection::new("127.0.0.1:8080".to_string());

        let response = RelayMessage::HandshakeAck {
            version: 999,
            peer_id: "relay1".to_string(),
            capabilities: RelayCapability::full_relay(),
        };

        let result = client.complete_handshake(&mut connection, response);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_push_envelopes_not_connected() {
        let client = test_client();
        let result = client.push_envelopes(vec![vec![1, 2, 3]]).await;

        assert!(result.is_err());
        match result {
            Err(RelayClientError::NotConnected) => (),
            _ => panic!("Wrong error type"),
        }
    }

    #[tokio::test]
    async fn test_pull_envelopes_not_connected() {
        let client = test_client();
        let result = client.pull_envelopes(0).await;

        assert!(result.is_err());
        match result {
            Err(RelayClientError::NotConnected) => (),
            _ => panic!("Wrong error type"),
        }
    }

    #[tokio::test]
    async fn test_active_connections() {
        let client = test_client();
        assert_eq!(client.active_connections().await, 0);

        let connection = RelayConnection::new("127.0.0.1:8080".to_string());
        client.add_connection(connection).await;

        assert_eq!(client.active_connections().await, 0); // Still disconnected

        let mut connection = RelayConnection::new("127.0.0.1:8081".to_string());
        connection.set_state(ConnectionState::Connected);
        client.add_connection(connection).await;

        assert_eq!(client.active_connections().await, 1);
    }

    #[tokio::test]
    async fn test_remove_connection() {
        let client = test_client();
        let connection = RelayConnection::new("127.0.0.1:8080".to_string());
        client.add_connection(connection).await;

        assert_eq!(client.active_connections().await, 0);

        client.remove_connection("127.0.0.1:8080").await;

        // Verify it's gone by checking we can't use it
        // (would fail anyway since it's not connected)
    }

    #[tokio::test]
    async fn test_push_pull_and_ping_over_network() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            loop {
                let len = match stream.read_u32().await {
                    Ok(v) => v as usize,
                    Err(_) => break,
                };
                let mut buf = vec![0u8; len];
                stream.read_exact(&mut buf).await.unwrap();
                let request = RelayMessage::from_bytes(&buf).unwrap();
                let response = match request {
                    RelayMessage::Handshake { .. } => RelayMessage::HandshakeAck {
                        version: PROTOCOL_VERSION,
                        peer_id: "relay-peer".to_string(),
                        capabilities: RelayCapability::full_relay(),
                    },
                    RelayMessage::StoreRequest { envelopes } => RelayMessage::StoreAck {
                        accepted: envelopes.len() as u32,
                        rejected: 0,
                    },
                    RelayMessage::PullRequest { .. } => RelayMessage::PullResponse {
                        envelopes: vec![vec![9, 8, 7]],
                    },
                    RelayMessage::Ping => RelayMessage::Pong,
                    _ => RelayMessage::Disconnect {
                        reason: "unsupported".to_string(),
                    },
                };
                let bytes = response.to_bytes().unwrap();
                stream.write_u32(bytes.len() as u32).await.unwrap();
                stream.write_all(&bytes).await.unwrap();
                stream.flush().await.unwrap();
            }
        });

        let client = test_client();
        let connection = client.connect(addr.to_string()).await.unwrap();
        client.add_connection(connection).await;

        let (accepted, rejected) = client.push_envelopes(vec![vec![1], vec![2]]).await.unwrap();
        assert_eq!(accepted, 2);
        assert_eq!(rejected, 0);

        let envelopes = client.pull_envelopes(0).await.unwrap();
        assert_eq!(envelopes, vec![vec![9, 8, 7]]);

        client.send_ping().await.unwrap();
    }

    #[tokio::test]
    async fn test_send_ping_not_connected() {
        let client = test_client();
        let result = client.send_ping().await;

        assert!(result.is_err());
        match result {
            Err(RelayClientError::NotConnected) => (),
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_backoff_duration() {
        let client = test_client();

        let duration0 = client.backoff_duration(0);
        let duration1 = client.backoff_duration(1);
        let duration5 = client.backoff_duration(5);

        // Each attempt should roughly double (within rounding)
        assert!(duration1.as_millis() >= duration0.as_millis());
        assert!(duration5.as_millis() >= duration1.as_millis());

        // Should cap at 60 seconds
        let duration_large = client.backoff_duration(10);
        assert!(duration_large.as_secs() <= 60);
    }

    #[test]
    fn test_get_relay_addresses() {
        let config = RelayClientConfig {
            known_relays: vec!["relay1.example.com".to_string(), "relay2.example.com".to_string()],
            ..Default::default()
        };
        let client = RelayClient::new("peer1".to_string(), config);

        let addresses = client.get_relay_addresses();
        assert_eq!(addresses.len(), 2);
        assert!(addresses.contains(&"relay1.example.com".to_string()));
    }
}
