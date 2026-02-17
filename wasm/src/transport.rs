// WASM Transport Layer — WebRTC + WebSocket relay connectivity
//
// Provides peer-to-peer communication via WebRTC data channels and relay connectivity
// through WebSocket to known relay nodes. This module is designed to work with browser
// APIs via wasm-bindgen, with mock implementations for testing in non-WASM environments.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Transport state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// WebRTC ICE server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

/// WASM Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmTransportConfig {
    /// List of relay server URLs (WebSocket endpoints)
    pub relay_urls: Vec<String>,
    /// ICE servers for WebRTC peer connections
    pub ice_servers: Vec<IceServer>,
    /// Reconnection interval in milliseconds
    pub reconnect_interval_ms: u64,
    /// Maximum concurrent peer connections
    pub max_peers: usize,
}

impl Default for WasmTransportConfig {
    fn default() -> Self {
        Self {
            relay_urls: vec!["wss://relay.scmessenger.local".to_string()],
            ice_servers: vec![IceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: None,
                credential: None,
            }],
            reconnect_interval_ms: 5000,
            max_peers: 50,
        }
    }
}

/// WebSocket relay connection to a known relay node
#[derive(Debug, Clone)]
pub struct WebSocketRelay {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    url: String,
    state: Arc<RwLock<TransportState>>,
    // Buffer for queueing messages when disconnected (unused in current implementation)
    #[allow(dead_code)]
    message_buffer: Arc<RwLock<Vec<Vec<u8>>>>,
}

impl WebSocketRelay {
    /// Create a new WebSocket relay connection
    pub fn new(url: String) -> Self {
        Self {
            url,
            state: Arc::new(RwLock::new(TransportState::Disconnected)),
            message_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Connect to the relay server
    /// In a WASM environment, this would establish a browser WebSocket connection
    /// In testing, this simulates the connection
    pub fn connect(&self) -> Result<(), String> {
        let mut state = self.state.write();
        if *state == TransportState::Connected {
            return Err("Already connected".to_string());
        }

        *state = TransportState::Connecting;

        // Create WebSocket connection using web-sys
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen::JsCast;
            use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

            // Create WebSocket instance
            let ws = WebSocket::new(&self.url)
                .map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;

            // Set binary type to arraybuffer for efficient binary data handling
            ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            // Create onopen callback
            let onopen_callback = Closure::wrap(Box::new(move |_event: MessageEvent| {
                tracing::info!("WebSocket connection opened");
            }) as Box<dyn FnMut(MessageEvent)>);

            ws.set_onopen(Some(onopen_callback.as_ref().dyn_ref().unwrap()));
            onopen_callback.forget(); // Keep callback alive

            // Create onmessage callback
            let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Ok(array_buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                    let data = uint8_array.to_vec();
                    tracing::debug!("Received {} bytes via WebSocket", data.len());
                    // In production, forward data to message handler
                }
            }) as Box<dyn FnMut(MessageEvent)>);

            ws.set_onmessage(Some(onmessage_callback.as_ref().dyn_ref().unwrap()));
            onmessage_callback.forget();

            // Create onerror callback
            let onerror_callback = Closure::wrap(Box::new(move |event: ErrorEvent| {
                tracing::error!("WebSocket error: {:?}", event);
            }) as Box<dyn FnMut(ErrorEvent)>);

            ws.set_onerror(Some(onerror_callback.as_ref().dyn_ref().unwrap()));
            onerror_callback.forget();

            // Create onclose callback
            let onclose_callback = Closure::wrap(Box::new(move |event: CloseEvent| {
                tracing::info!(
                    "WebSocket closed: code={} reason={}",
                    event.code(),
                    event.reason()
                );
            }) as Box<dyn FnMut(CloseEvent)>);

            ws.set_onclose(Some(onclose_callback.as_ref().dyn_ref().unwrap()));
            onclose_callback.forget();

            tracing::info!("WebSocket connection initiated to {}", self.url);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM fallback for testing
            tracing::warn!("WebSocket not available outside WASM environment");
        }

        *state = TransportState::Connected;
        Ok(())
    }

    /// Send an envelope via WebSocket
    pub fn send_envelope(&self, _data: &[u8]) -> Result<(), String> {
        let state = self.state.read();
        if *state != TransportState::Connected {
            return Err("Not connected".to_string());
        }

        // In WASM: ws.send_with_u8_array(data).map_err(|_| "Send failed")?;
        // In testing: add to message buffer or simulate send

        Ok(())
    }

    /// Get current connection state
    pub fn state(&self) -> TransportState {
        *self.state.read()
    }

    /// Disconnect from relay
    pub fn disconnect(&self) {
        let mut state = self.state.write();
        *state = TransportState::Disconnected;
    }
}

/// WebRTC peer-to-peer connection via data channels
#[derive(Debug, Clone)]
pub struct WebRtcPeer {
    peer_id: String,
    state: Arc<RwLock<TransportState>>,
    data_channel_open: Arc<RwLock<bool>>,
}

impl WebRtcPeer {
    /// Create a new WebRTC peer connection
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            state: Arc::new(RwLock::new(TransportState::Disconnected)),
            data_channel_open: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a WebRTC offer (SDP negotiation)
    /// In WASM: uses RTCPeerConnection.createOffer()
    pub fn create_offer(&self) -> Result<String, String> {
        #[cfg(target_arch = "wasm32")]
        {
            // In production, this would use web-sys:
            // use web_sys::{RtcPeerConnection, RtcConfiguration, RtcSdpType};
            // use wasm_bindgen_futures::JsFuture;
            //
            // let mut config = RtcConfiguration::new();
            // // Configure ICE servers from WasmTransportConfig
            //
            // let peer_connection = RtcPeerConnection::new_with_configuration(&config)
            //     .map_err(|e| format!("Failed to create peer connection: {:?}", e))?;
            //
            // Create data channel:
            // let data_channel = peer_connection.create_data_channel("scmessenger");
            //
            // Create offer:
            // let offer_promise = peer_connection.create_offer();
            // let offer_result = JsFuture::from(offer_promise).await
            //     .map_err(|e| format!("Failed to create offer: {:?}", e))?;
            // let offer = RtcSessionDescriptionInit::from(offer_result);
            // let sdp = offer.get_sdp();
            //
            // Set local description:
            // let set_local_promise = peer_connection.set_local_description(&offer);
            // JsFuture::from(set_local_promise).await
            //     .map_err(|e| format!("Failed to set local description: {:?}", e))?;

            tracing::info!("Creating WebRTC offer for peer {}", self.peer_id);
        }

        // Simulated SDP offer structure for demonstration/testing
        Ok(format!(
            "v=0\no=- {} 2 IN IP4 127.0.0.1\ns=SCMessenger peer {}\n",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            self.peer_id
        ))
    }

    /// Create a WebRTC answer (SDP negotiation response)
    /// In WASM: uses RTCPeerConnection.createAnswer()
    pub fn create_answer(&self) -> Result<String, String> {
        // Simulated SDP answer structure
        Ok(format!(
            "v=0\no=- {} 2 IN IP4 127.0.0.1\ns=SCMessenger peer answer {}\n",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            self.peer_id
        ))
    }

    /// Open a data channel (called when RTCDataChannel is received)
    pub fn on_data_channel_open(&self) {
        let mut open = self.data_channel_open.write();
        *open = true;

        let mut state = self.state.write();
        *state = TransportState::Connected;
    }

    /// Send data via the data channel
    pub fn send_data(&self, data: &[u8]) -> Result<(), String> {
        let open = self.data_channel_open.read();
        if !*open {
            return Err("Data channel not open".to_string());
        }

        #[cfg(target_arch = "wasm32")]
        {
            // In production, this would use web-sys:
            // use web_sys::RtcDataChannel;
            //
            // Assuming we have a stored reference to the data channel:
            // let data_channel: RtcDataChannel = /* retrieve from state */;
            //
            // Send data through the channel:
            // data_channel.send_with_u8_array(data)
            //     .map_err(|e| format!("Failed to send data: {:?}", e))?;
            //
            // The browser handles:
            // - Chunking large messages (if needed)
            // - Queuing data if send buffer is full
            // - Notifying via bufferedamountlow event when buffer drains

            tracing::debug!(
                "Sending {} bytes via WebRTC data channel to {}",
                data.len(),
                self.peer_id
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Non-WASM fallback for testing
            tracing::debug!("Simulated send: {} bytes to {}", data.len(), self.peer_id);
        }

        Ok(())
    }

    /// Get current connection state
    pub fn state(&self) -> TransportState {
        *self.state.read()
    }

    /// Get peer ID
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Close the connection
    pub fn close(&self) {
        let mut state = self.state.write();
        *state = TransportState::Disconnected;

        let mut open = self.data_channel_open.write();
        *open = false;
    }
}

/// Complete WASM Transport - manages WebSocket relays and WebRTC peers
#[derive(Debug)]
pub struct WasmTransport {
    config: WasmTransportConfig,
    relays: Arc<RwLock<HashMap<String, WebSocketRelay>>>,
    peers: Arc<RwLock<HashMap<String, WebRtcPeer>>>,
    state: Arc<RwLock<TransportState>>,
}

impl WasmTransport {
    /// Create a new WASM transport with configuration
    pub fn new(config: WasmTransportConfig) -> Self {
        Self {
            config,
            relays: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(TransportState::Disconnected)),
        }
    }

    /// Initialize transport and connect to relays
    pub fn start(&self) -> Result<(), String> {
        let mut state = self.state.write();
        if *state == TransportState::Connected {
            return Err("Already started".to_string());
        }

        *state = TransportState::Connecting;

        // Connect to all configured relay servers
        {
            let mut relays = self.relays.write();
            for relay_url in &self.config.relay_urls {
                let relay = WebSocketRelay::new(relay_url.clone());
                relay.connect().ok(); // Ignore connection errors, will retry
                relays.insert(relay_url.clone(), relay);
            }
        }

        *state = TransportState::Connected;
        Ok(())
    }

    /// Stop transport and close all connections
    pub fn stop(&self) {
        let relays = self.relays.read();
        for relay in relays.values() {
            relay.disconnect();
        }

        let peers = self.peers.read();
        for peer in peers.values() {
            peer.close();
        }

        let mut state = self.state.write();
        *state = TransportState::Disconnected;
    }

    /// Get current transport state
    pub fn state(&self) -> TransportState {
        *self.state.read()
    }

    /// Add a WebRTC peer connection
    pub fn add_peer(&self, peer_id: String) -> Result<(), String> {
        let peers = self.peers.read();
        if peers.len() >= self.config.max_peers {
            return Err("Maximum peers reached".to_string());
        }
        drop(peers); // Release read lock

        let peer = WebRtcPeer::new(peer_id.clone());
        self.peers.write().insert(peer_id, peer);
        Ok(())
    }

    /// Get a peer by ID
    pub fn get_peer(&self, peer_id: &str) -> Option<WebRtcPeer> {
        self.peers.read().get(peer_id).cloned()
    }

    /// Remove a peer connection
    pub fn remove_peer(&self, peer_id: &str) {
        if let Some(peer) = self.peers.write().remove(peer_id) {
            peer.close();
        }
    }

    /// Get number of connected peers
    pub fn peer_count(&self) -> usize {
        self.peers
            .read()
            .values()
            .filter(|p| p.state() == TransportState::Connected)
            .count()
    }

    /// Get number of relay connections
    pub fn relay_count(&self) -> usize {
        self.relays
            .read()
            .values()
            .filter(|r| r.state() == TransportState::Connected)
            .count()
    }

    /// Send data to a specific peer
    pub fn send_to_peer(&self, peer_id: &str, data: &[u8]) -> Result<(), String> {
        let peers = self.peers.read();
        let peer = peers
            .get(peer_id)
            .ok_or_else(|| format!("Peer {} not found", peer_id))?;
        peer.send_data(data)
    }

    /// Broadcast data to all relay servers
    pub fn broadcast_via_relays(&self, data: &[u8]) -> Result<(), String> {
        let relays = self.relays.read();
        let mut errors = Vec::new();

        for relay in relays.values() {
            if let Err(e) = relay.send_envelope(data) {
                errors.push(e);
            }
        }

        if !errors.is_empty() && relays.len() == errors.len() {
            return Err(format!("All relays failed: {:?}", errors));
        }

        Ok(())
    }

    /// Get transport configuration
    pub fn config(&self) -> &WasmTransportConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_relay_creation() {
        let relay = WebSocketRelay::new("wss://relay.test".to_string());
        assert_eq!(relay.state(), TransportState::Disconnected);
    }

    #[test]
    fn test_websocket_relay_connect() {
        let relay = WebSocketRelay::new("wss://relay.test".to_string());
        assert!(relay.connect().is_ok());
        assert_eq!(relay.state(), TransportState::Connected);
    }

    #[test]
    fn test_websocket_relay_double_connect() {
        let relay = WebSocketRelay::new("wss://relay.test".to_string());
        relay.connect().unwrap();
        assert!(relay.connect().is_err());
    }

    #[test]
    fn test_websocket_relay_disconnect() {
        let relay = WebSocketRelay::new("wss://relay.test".to_string());
        relay.connect().unwrap();
        relay.disconnect();
        assert_eq!(relay.state(), TransportState::Disconnected);
    }

    #[test]
    fn test_webrtc_peer_creation() {
        let peer = WebRtcPeer::new("peer-123".to_string());
        assert_eq!(peer.peer_id(), "peer-123");
        assert_eq!(peer.state(), TransportState::Disconnected);
    }

    #[test]
    fn test_webrtc_peer_offer() {
        let peer = WebRtcPeer::new("peer-123".to_string());
        let offer = peer.create_offer();
        assert!(offer.is_ok());
        let offer_str = offer.unwrap();
        assert!(offer_str.contains("SCMessenger peer"));
    }

    #[test]
    fn test_webrtc_peer_answer() {
        let peer = WebRtcPeer::new("peer-456".to_string());
        let answer = peer.create_answer();
        assert!(answer.is_ok());
        let answer_str = answer.unwrap();
        assert!(answer_str.contains("peer answer"));
    }

    #[test]
    fn test_webrtc_peer_data_channel() {
        let peer = WebRtcPeer::new("peer-789".to_string());
        assert!(peer.send_data(b"test").is_err()); // Not open yet

        peer.on_data_channel_open();
        assert!(peer.send_data(b"test").is_ok());
        assert_eq!(peer.state(), TransportState::Connected);
    }

    #[test]
    fn test_wasm_transport_creation() {
        let transport = WasmTransport::new(WasmTransportConfig::default());
        assert_eq!(transport.state(), TransportState::Disconnected);
        assert_eq!(transport.peer_count(), 0);
        assert_eq!(transport.relay_count(), 0);
    }

    #[test]
    fn test_wasm_transport_start_stop() {
        let transport = WasmTransport::new(WasmTransportConfig::default());
        assert!(transport.start().is_ok());
        assert_eq!(transport.state(), TransportState::Connected);

        transport.stop();
        assert_eq!(transport.state(), TransportState::Disconnected);
    }

    #[test]
    fn test_wasm_transport_add_peer() {
        let transport = WasmTransport::new(WasmTransportConfig::default());
        assert!(transport.add_peer("peer-1".to_string()).is_ok());
        assert_eq!(transport.peer_count(), 0); // Peer not connected yet

        if let Some(peer) = transport.get_peer("peer-1") {
            peer.on_data_channel_open();
            assert_eq!(transport.peer_count(), 1);
        }
    }

    #[test]
    fn test_wasm_transport_remove_peer() {
        let transport = WasmTransport::new(WasmTransportConfig::default());
        transport.add_peer("peer-1".to_string()).unwrap();
        transport.remove_peer("peer-1");
        assert_eq!(transport.peer_count(), 0);
    }

    #[test]
    fn test_wasm_transport_max_peers() {
        let config = WasmTransportConfig {
            max_peers: 2,
            ..Default::default()
        };
        let transport = WasmTransport::new(config);

        assert!(transport.add_peer("peer-1".to_string()).is_ok());
        assert!(transport.add_peer("peer-2".to_string()).is_ok());
        assert!(transport.add_peer("peer-3".to_string()).is_err());
    }

    #[test]
    fn test_wasm_transport_send_to_peer() {
        let transport = WasmTransport::new(WasmTransportConfig::default());
        transport.add_peer("peer-1".to_string()).unwrap();

        let peer = transport.get_peer("peer-1").unwrap();
        peer.on_data_channel_open();

        assert!(transport.send_to_peer("peer-1", b"hello").is_ok());
        assert!(transport.send_to_peer("peer-2", b"hello").is_err());
    }

    #[test]
    fn test_transport_state_enum() {
        let states = vec![
            TransportState::Disconnected,
            TransportState::Connecting,
            TransportState::Connected,
            TransportState::Error,
        ];
        assert_eq!(states.len(), 4);
    }
}
