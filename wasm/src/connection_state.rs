// WASM Connection State Management
//
// Proper lifecycle management for WebSocket and WebRTC connections in WASM.
// Solves the memory leak problem caused by .forget() on closures by storing
// callbacks and cleaning them up on disconnect.

use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{WebSocket, RtcPeerConnection, MessageEvent, ErrorEvent, CloseEvent};

/// Managed WebSocket connection with proper callback lifecycle
#[cfg(target_arch = "wasm32")]
pub struct ManagedWebSocket {
    /// The actual WebSocket instance
    websocket: WebSocket,
    /// Stored callbacks (must be kept alive)
    onopen: Option<Closure<dyn FnMut(MessageEvent)>>,
    onmessage: Option<Closure<dyn FnMut(MessageEvent)>>,
    onerror: Option<Closure<dyn FnMut(ErrorEvent)>>,
    onclose: Option<Closure<dyn FnMut(CloseEvent)>>,
}

#[cfg(target_arch = "wasm32")]
impl ManagedWebSocket {
    /// Create a new managed WebSocket connection
    pub fn new(url: &str) -> Result<Self, String> {
        let websocket = WebSocket::new(url)
            .map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;

        websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);

        Ok(Self {
            websocket,
            onopen: None,
            onmessage: None,
            onerror: None,
            onclose: None,
        })
    }

    /// Set the onopen callback
    pub fn set_onopen<F>(&mut self, callback: F)
    where
        F: FnMut(MessageEvent) + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(MessageEvent)>);
        self.websocket
            .set_onopen(Some(closure.as_ref().unchecked_ref()));
        self.onopen = Some(closure);
    }

    /// Set the onmessage callback
    pub fn set_onmessage<F>(&mut self, callback: F)
    where
        F: FnMut(MessageEvent) + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(MessageEvent)>);
        self.websocket
            .set_onmessage(Some(closure.as_ref().unchecked_ref()));
        self.onmessage = Some(closure);
    }

    /// Set the onerror callback
    pub fn set_onerror<F>(&mut self, callback: F)
    where
        F: FnMut(ErrorEvent) + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(ErrorEvent)>);
        self.websocket
            .set_onerror(Some(closure.as_ref().unchecked_ref()));
        self.onerror = Some(closure);
    }

    /// Set the onclose callback
    pub fn set_onclose<F>(&mut self, callback: F)
    where
        F: FnMut(CloseEvent) + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(CloseEvent)>);
        self.websocket
            .set_onclose(Some(closure.as_ref().unchecked_ref()));
        self.onclose = Some(closure);
    }

    /// Send binary data
    pub fn send(&self, data: &[u8]) -> Result<(), String> {
        self.websocket
            .send_with_u8_array(data)
            .map_err(|e| format!("Failed to send data: {:?}", e))
    }

    /// Close the connection
    pub fn close(&mut self) {
        let _ = self.websocket.close();
        self.cleanup_callbacks();
    }

    /// Clean up all callbacks (called on drop or close)
    fn cleanup_callbacks(&mut self) {
        self.websocket.set_onopen(None);
        self.websocket.set_onmessage(None);
        self.websocket.set_onerror(None);
        self.websocket.set_onclose(None);

        // Drop the closures, cleaning up memory
        self.onopen = None;
        self.onmessage = None;
        self.onerror = None;
        self.onclose = None;
    }

    /// Get the ready state
    pub fn ready_state(&self) -> u16 {
        self.websocket.ready_state()
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for ManagedWebSocket {
    fn drop(&mut self) {
        self.cleanup_callbacks();
    }
}

/// Managed WebRTC peer connection with proper callback lifecycle
#[cfg(target_arch = "wasm32")]
pub struct ManagedRtcConnection {
    /// The actual RTCPeerConnection instance
    peer_connection: RtcPeerConnection,
    /// Data channel callbacks
    ondatachannel: Option<Closure<dyn FnMut(web_sys::RtcDataChannelEvent)>>,
    /// ICE candidate callbacks
    onicecandidate: Option<Closure<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>>,
    /// Connection state change callbacks
    onconnectionstatechange: Option<Closure<dyn FnMut(web_sys::Event)>>,
}

#[cfg(target_arch = "wasm32")]
impl ManagedRtcConnection {
    /// Create a new managed RTC peer connection
    pub fn new(config: &web_sys::RtcConfiguration) -> Result<Self, String> {
        let peer_connection = RtcPeerConnection::new_with_configuration(config)
            .map_err(|e| format!("Failed to create RTC peer connection: {:?}", e))?;

        Ok(Self {
            peer_connection,
            ondatachannel: None,
            onicecandidate: None,
            onconnectionstatechange: None,
        })
    }

    /// Set the ondatachannel callback
    pub fn set_ondatachannel<F>(&mut self, callback: F)
    where
        F: FnMut(web_sys::RtcDataChannelEvent) + 'static,
    {
        let closure = Closure::wrap(
            Box::new(callback) as Box<dyn FnMut(web_sys::RtcDataChannelEvent)>
        );
        self.peer_connection
            .set_ondatachannel(Some(closure.as_ref().unchecked_ref()));
        self.ondatachannel = Some(closure);
    }

    /// Set the onicecandidate callback
    pub fn set_onicecandidate<F>(&mut self, callback: F)
    where
        F: FnMut(web_sys::RtcPeerConnectionIceEvent) + 'static,
    {
        let closure = Closure::wrap(
            Box::new(callback) as Box<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>
        );
        self.peer_connection
            .set_onicecandidate(Some(closure.as_ref().unchecked_ref()));
        self.onicecandidate = Some(closure);
    }

    /// Set the onconnectionstatechange callback
    pub fn set_onconnectionstatechange<F>(&mut self, callback: F)
    where
        F: FnMut(web_sys::Event) + 'static,
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut(web_sys::Event)>);
        self.peer_connection
            .set_onconnectionstatechange(Some(closure.as_ref().unchecked_ref()));
        self.onconnectionstatechange = Some(closure);
    }

    /// Create a data channel
    pub fn create_data_channel(&self, label: &str) -> Result<web_sys::RtcDataChannel, String> {
        self.peer_connection
            .create_data_channel(label)
            .ok_or_else(|| "Failed to create data channel".to_string())
    }

    /// Get connection state
    pub fn connection_state(&self) -> web_sys::RtcPeerConnectionState {
        self.peer_connection.connection_state()
    }

    /// Close the connection
    pub fn close(&mut self) {
        self.peer_connection.close();
        self.cleanup_callbacks();
    }

    /// Clean up all callbacks
    fn cleanup_callbacks(&mut self) {
        self.peer_connection.set_ondatachannel(None);
        self.peer_connection.set_onicecandidate(None);
        self.peer_connection.set_onconnectionstatechange(None);

        // Drop the closures
        self.ondatachannel = None;
        self.onicecandidate = None;
        self.onconnectionstatechange = None;
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for ManagedRtcConnection {
    fn drop(&mut self) {
        self.cleanup_callbacks();
    }
}

/// Connection manager for all active connections
pub struct ConnectionManager {
    /// Active WebSocket connections by URL
    #[cfg(target_arch = "wasm32")]
    websockets: Arc<RwLock<std::collections::HashMap<String, Arc<RwLock<ManagedWebSocket>>>>>,
    /// Active WebRTC peer connections by peer ID
    #[cfg(target_arch = "wasm32")]
    rtc_connections: Arc<RwLock<std::collections::HashMap<String, Arc<RwLock<ManagedRtcConnection>>>>>,
    /// Connection count for statistics
    connection_count: Arc<RwLock<u64>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            websockets: Arc::new(RwLock::new(std::collections::HashMap::new())),
            #[cfg(target_arch = "wasm32")]
            rtc_connections: Arc::new(RwLock::new(std::collections::HashMap::new())),
            connection_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Add a managed WebSocket connection
    #[cfg(target_arch = "wasm32")]
    pub fn add_websocket(&self, url: String, ws: ManagedWebSocket) {
        let mut connections = self.websockets.write();
        connections.insert(url, Arc::new(RwLock::new(ws)));
        *self.connection_count.write() += 1;
    }

    /// Remove and close a WebSocket connection
    #[cfg(target_arch = "wasm32")]
    pub fn remove_websocket(&self, url: &str) {
        let mut connections = self.websockets.write();
        if let Some(ws_arc) = connections.remove(url) {
            let mut ws = ws_arc.write();
            ws.close();
            if *self.connection_count.read() > 0 {
                *self.connection_count.write() -= 1;
            }
        }
    }

    /// Add a managed WebRTC peer connection
    #[cfg(target_arch = "wasm32")]
    pub fn add_rtc_connection(&self, peer_id: String, rtc: ManagedRtcConnection) {
        let mut connections = self.rtc_connections.write();
        connections.insert(peer_id, Arc::new(RwLock::new(rtc)));
        *self.connection_count.write() += 1;
    }

    /// Remove and close an RTC peer connection
    #[cfg(target_arch = "wasm32")]
    pub fn remove_rtc_connection(&self, peer_id: &str) {
        let mut connections = self.rtc_connections.write();
        if let Some(rtc_arc) = connections.remove(peer_id) {
            let mut rtc = rtc_arc.write();
            rtc.close();
            if *self.connection_count.read() > 0 {
                *self.connection_count.write() -= 1;
            }
        }
    }

    /// Close all connections
    pub fn close_all(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            // Close all WebSocket connections
            let mut websockets = self.websockets.write();
            for (_, ws_arc) in websockets.drain() {
                let mut ws = ws_arc.write();
                ws.close();
            }

            // Close all RTC connections
            let mut rtc_connections = self.rtc_connections.write();
            for (_, rtc_arc) in rtc_connections.drain() {
                let mut rtc = rtc_arc.write();
                rtc.close();
            }
        }

        *self.connection_count.write() = 0;
    }

    /// Get total number of active connections
    pub fn connection_count(&self) -> u64 {
        *self.connection_count.read()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        self.close_all();
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_manager_creation() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.connection_count(), 0);
    }

    #[test]
    fn test_connection_manager_close_all() {
        let manager = ConnectionManager::new();
        manager.close_all();
        assert_eq!(manager.connection_count(), 0);
    }

    #[test]
    fn test_connection_manager_default() {
        let manager = ConnectionManager::default();
        assert_eq!(manager.connection_count(), 0);
    }

    // Note: WebSocket and RTC connection tests require WASM target
    // and are tested in browser environment
}
