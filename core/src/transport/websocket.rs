// WebSocket Transport Implementation
//
// Provides WebSocket fallback transport for relay connectivity when
// UDP/TCP connections are blocked by carrier-level port filtering.
// Uses tokio-tungstenite for WebSocket client implementation.

use crate::transport::internet::InternetTransportError;
use futures::{SinkExt, StreamExt};
use libp2p::Multiaddr;
use std::time::Duration;
use tokio_tungstenite::{
    connect_async,
    tungstenite::client::IntoClientRequest,
};
use tracing::{debug, info, warn};

/// WebSocket transport error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum WebSocketTransportError {
    #[error("Invalid WebSocket URL: {0}")]
    InvalidUrl(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("WebSocket handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// WebSocket transport for relay connectivity
pub struct WebSocketTransport {
    /// WebSocket stream for communication
    inner: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    /// WebSocket URL
    url: String,
    /// Connection timeout
    connect_timeout: Duration,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport
    pub fn new(url: String, connect_timeout: Duration) -> Self {
        Self {
            inner: None,
            url,
            connect_timeout,
        }
    }

    /// Create WebSocket transport from Multiaddr
    pub fn from_multiaddr(addr: &Multiaddr) -> Result<Self, WebSocketTransportError> {
        // Convert Multiaddr to WebSocket URL
        let url = multiaddr_to_websocket_url(addr)?;
        Ok(Self::new(url, Duration::from_secs(10)))
    }

    /// Connect to WebSocket endpoint
    pub async fn connect(&mut self) -> Result<(), WebSocketTransportError> {
        info!("Connecting to WebSocket relay: {}", self.url);

        // Create client request
        let request = self.url.clone().into_client_request()
            .map_err(|e| WebSocketTransportError::InvalidUrl(e.to_string()))?;

        // Apply connection timeout
        let (ws_stream, response) = tokio::time::timeout(
            self.connect_timeout,
            connect_async(request)
        ).await
        .map_err(|_| WebSocketTransportError::Timeout("WebSocket connection timeout".to_string()))?
        .map_err(|e| WebSocketTransportError::ConnectionFailed(e.to_string()))?;

        debug!("WebSocket connection established: {:?}", response);

        self.inner = Some(ws_stream);
        info!("WebSocket transport connected to {}", self.url);

        Ok(())
    }

    /// Send data over WebSocket
    pub async fn send(&mut self, data: Vec<u8>) -> Result<(), WebSocketTransportError> {
        if let Some(ref mut ws_stream) = self.inner {
            let msg = tokio_tungstenite::tungstenite::Message::Binary(data);

            tokio::time::timeout(
                Duration::from_secs(5),
                ws_stream.send(msg)
            ).await
            .map_err(|_| WebSocketTransportError::Timeout("WebSocket send timeout".to_string()))?
            .map_err(|e| WebSocketTransportError::SendFailed(e.to_string()))
        } else {
            Err(WebSocketTransportError::ConnectionFailed("Not connected".to_string()))
        }
    }

    /// Receive data from WebSocket
    pub async fn recv(&mut self) -> Result<Vec<u8>, WebSocketTransportError> {
        if let Some(ref mut ws_stream) = self.inner {
            tokio::time::timeout(
                Duration::from_secs(5),
                ws_stream.next()
            ).await
            .map_err(|_| WebSocketTransportError::Timeout("WebSocket receive timeout".to_string()))?
            .ok_or_else(|| WebSocketTransportError::ReceiveFailed("Connection closed".to_string()))?
            .map_err(|e| WebSocketTransportError::ReceiveFailed(e.to_string()))
            .and_then(|msg| {
                match msg {
                    tokio_tungstenite::tungstenite::Message::Binary(data) => Ok(data),
                    tokio_tungstenite::tungstenite::Message::Text(text) => Ok(text.into_bytes()),
                    _ => Err(WebSocketTransportError::ReceiveFailed("Unsupported message type".to_string())),
                }
            })
        } else {
            Err(WebSocketTransportError::ConnectionFailed("Not connected".to_string()))
        }
    }

    /// Close WebSocket connection
    pub async fn close(&mut self) -> Result<(), WebSocketTransportError> {
        if let Some(ref mut ws_stream) = self.inner {
            ws_stream.close(None).await
                .map_err(|e| WebSocketTransportError::ConnectionFailed(e.to_string()))?;
            self.inner = None;
            info!("WebSocket transport closed");
        }
        Ok(())
    }

    /// Check if transport is connected
    pub fn is_connected(&self) -> bool {
        self.inner.is_some()
    }
}

/// Convert Multiaddr to WebSocket URL
fn multiaddr_to_websocket_url(addr: &Multiaddr) -> Result<String, WebSocketTransportError> {
    let mut host = None;
    let mut port = None;
    let mut is_websocket = false;

    for protocol in addr.iter() {
        match protocol {
            libp2p::multiaddr::Protocol::Ip4(ip) => {
                host = Some(ip.to_string());
            }
            libp2p::multiaddr::Protocol::Ip6(ip) => {
                host = Some(format!("[{}]", ip));
            }
            libp2p::multiaddr::Protocol::Dns(domain) | libp2p::multiaddr::Protocol::Dns4(domain) | libp2p::multiaddr::Protocol::Dns6(domain) => {
                host = Some(domain.to_string());
            }
            libp2p::multiaddr::Protocol::Tcp(p) => {
                port = Some(p);
            }
            libp2p::multiaddr::Protocol::Ws(_) => {
                is_websocket = true;
            }
            libp2p::multiaddr::Protocol::Wss(_) => {
                is_websocket = true;
            }
            _ => {
                // Ignore other protocols
            }
        }
    }

    if !is_websocket {
        return Err(WebSocketTransportError::InvalidUrl("Not a WebSocket address".to_string()));
    }

    let host = host.ok_or_else(|| WebSocketTransportError::InvalidUrl("No host in address".to_string()))?;
    let port = port.unwrap_or(80);

    // Determine scheme based on port or presence of Wss protocol
    let scheme = if addr.iter().any(|p| matches!(p, libp2p::multiaddr::Protocol::Wss(_))) || port == 443 {
        "wss"
    } else {
        "ws"
    };

    Ok(format!("{}://{}:{}", scheme, host, port))
}

/// Enhanced error diagnostics for WebSocket connection failures
pub fn diagnose_websocket_error(error: WebSocketTransportError, relay_addr: &Multiaddr) -> InternetTransportError {
    match error {
        WebSocketTransportError::InvalidUrl(msg) => {
            warn!("Invalid WebSocket URL for relay {}: {}", relay_addr, msg);
            InternetTransportError::ConfigError(format!("Invalid WebSocket URL: {}", msg))
        }
        WebSocketTransportError::ConnectionFailed(msg) => {
            warn!("WebSocket connection failed for relay {}: {}", relay_addr, msg);
            InternetTransportError::ConnectionFailed(format!("WebSocket connection failed: {}", msg))
        }
        WebSocketTransportError::HandshakeFailed(msg) => {
            warn!("WebSocket handshake failed for relay {}: {}", relay_addr, msg);
            InternetTransportError::ConnectionFailed(format!("WebSocket handshake failed: {}", msg))
        }
        WebSocketTransportError::SendFailed(msg) => {
            warn!("WebSocket send failed for relay {}: {}", relay_addr, msg);
            InternetTransportError::ConnectionFailed(format!("WebSocket send failed: {}", msg))
        }
        WebSocketTransportError::ReceiveFailed(msg) => {
            warn!("WebSocket receive failed for relay {}: {}", relay_addr, msg);
            InternetTransportError::ConnectionFailed(format!("WebSocket receive failed: {}", msg))
        }
        WebSocketTransportError::Timeout(msg) => {
            warn!("WebSocket timeout for relay {}: {}", relay_addr, msg);
            InternetTransportError::ConnectionFailed(format!("WebSocket timeout: {}", msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::multiaddr::Protocol;

    #[test]
    fn test_multiaddr_to_websocket_url_ws() {
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/80/ws".parse().unwrap();
        let url = multiaddr_to_websocket_url(&addr).unwrap();
        assert_eq!(url, "ws://127.0.0.1:80");
    }

    #[test]
    fn test_multiaddr_to_websocket_url_wss() {
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/443/ws".parse().unwrap();
        let url = multiaddr_to_websocket_url(&addr).unwrap();
        assert_eq!(url, "wss://127.0.0.1:443");
    }

    #[test]
    fn test_multiaddr_to_websocket_url_wss_protocol() {
        let addr: Multiaddr = "/dns4/example.com/tcp/9001/wss".parse().unwrap();
        let url = multiaddr_to_websocket_url(&addr).unwrap();
        assert_eq!(url, "wss://example.com:9001");
    }

    #[test]
    fn test_multiaddr_to_websocket_url_invalid() {
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/80".parse().unwrap();
        let result = multiaddr_to_websocket_url(&addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_websocket_transport_creation() {
        let transport = WebSocketTransport::new("ws://localhost:8080".to_string(), Duration::from_secs(10));
        assert!(!transport.is_connected());
        assert_eq!(transport.url, "ws://localhost:8080");
    }

    #[tokio::test]
    async fn test_websocket_transport_diagnostics() {
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/80/ws".parse().unwrap();
        let error = WebSocketTransportError::ConnectionFailed("test error".to_string());
        let diagnosed = diagnose_websocket_error(error, &addr);

        match diagnosed {
            InternetTransportError::ConnectionFailed(msg) => {
                assert!(msg.contains("WebSocket connection failed"));
            }
            _ => panic!("Wrong error type"),
        }
    }
}