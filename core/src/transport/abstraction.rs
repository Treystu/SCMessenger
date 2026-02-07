//! Transport abstraction layer for SCMessenger
//!
//! Defines the core types and events for transport-agnostic messaging.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Represents different transport types available in the mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportType {
    /// Bluetooth Low Energy
    BLE,
    /// WiFi Aware (formerly WiFi Neighbor Awareness Network)
    WiFiAware,
    /// WiFi Direct (peer-to-peer)
    WiFiDirect,
    /// Internet-based transport (cellular, traditional WiFi)
    Internet,
    /// Local transport for testing
    Local,
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportType::BLE => write!(f, "BLE"),
            TransportType::WiFiAware => write!(f, "WiFiAware"),
            TransportType::WiFiDirect => write!(f, "WiFiDirect"),
            TransportType::Internet => write!(f, "Internet"),
            TransportType::Local => write!(f, "Local"),
        }
    }
}

/// Capabilities of a transport type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportCapabilities {
    /// Maximum payload size in bytes
    pub max_payload_size: usize,
    /// Whether this transport supports streaming
    pub supports_streaming: bool,
    /// Whether this transport is bidirectional
    pub is_bidirectional: bool,
    /// Estimated bandwidth in bits per second
    pub estimated_bandwidth_bps: u64,
    /// Estimated latency in milliseconds
    pub estimated_latency_ms: u32,
}

impl TransportCapabilities {
    /// Creates new transport capabilities
    pub fn new(
        max_payload_size: usize,
        supports_streaming: bool,
        is_bidirectional: bool,
        estimated_bandwidth_bps: u64,
        estimated_latency_ms: u32,
    ) -> Self {
        Self {
            max_payload_size,
            supports_streaming,
            is_bidirectional,
            estimated_bandwidth_bps,
            estimated_latency_ms,
        }
    }

    /// Get default capabilities for a transport type
    pub fn for_transport(transport: TransportType) -> Self {
        match transport {
            TransportType::BLE => Self {
                max_payload_size: 512,
                supports_streaming: false,
                is_bidirectional: true,
                estimated_bandwidth_bps: 2_000_000,     // 2 Mbps
                estimated_latency_ms: 50,
            },
            TransportType::WiFiAware => Self {
                max_payload_size: 2048,
                supports_streaming: true,
                is_bidirectional: true,
                estimated_bandwidth_bps: 80_000_000,    // 80 Mbps
                estimated_latency_ms: 10,
            },
            TransportType::WiFiDirect => Self {
                max_payload_size: 4096,
                supports_streaming: true,
                is_bidirectional: true,
                estimated_bandwidth_bps: 250_000_000,   // 250 Mbps
                estimated_latency_ms: 5,
            },
            TransportType::Internet => Self {
                max_payload_size: 8192,
                supports_streaming: true,
                is_bidirectional: true,
                estimated_bandwidth_bps: 100_000_000,   // 100 Mbps average
                estimated_latency_ms: 100,
            },
            TransportType::Local => Self {
                max_payload_size: 65536,
                supports_streaming: true,
                is_bidirectional: true,
                estimated_bandwidth_bps: 10_000_000_000, // 10 Gbps
                estimated_latency_ms: 1,
            },
        }
    }
}

/// Events from transport layer to routing engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportEvent {
    /// A peer was discovered on a transport
    PeerDiscovered {
        peer_id: [u8; 32],
        transport: TransportType,
        addr: Vec<u8>,
    },
    /// A peer disconnected from a transport
    PeerDisconnected {
        peer_id: [u8; 32],
        transport: TransportType,
    },
    /// Data received from a peer
    DataReceived {
        peer_id: [u8; 32],
        transport: TransportType,
        data: Vec<u8>,
    },
    /// Transport encountered an error
    TransportError {
        transport: TransportType,
        error: String,
    },
    /// Connection established with a peer
    ConnectionEstablished {
        peer_id: [u8; 32],
        transport: TransportType,
    },
}

impl fmt::Display for TransportEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportEvent::PeerDiscovered {
                peer_id,
                transport,
                ..
            } => write!(f, "PeerDiscovered {{ peer_id: {:x?}, transport: {} }}", &peer_id[..8], transport),
            TransportEvent::PeerDisconnected {
                peer_id,
                transport,
            } => write!(f, "PeerDisconnected {{ peer_id: {:x?}, transport: {} }}", &peer_id[..8], transport),
            TransportEvent::DataReceived {
                peer_id,
                transport,
                data,
            } => write!(f, "DataReceived {{ peer_id: {:x?}, transport: {}, data_len: {} }}", &peer_id[..8], transport, data.len()),
            TransportEvent::TransportError {
                transport,
                error,
            } => write!(f, "TransportError {{ transport: {}, error: {} }}", transport, error),
            TransportEvent::ConnectionEstablished {
                peer_id,
                transport,
            } => write!(f, "ConnectionEstablished {{ peer_id: {:x?}, transport: {} }}", &peer_id[..8], transport),
        }
    }
}

/// Commands from routing engine to transport layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportCommand {
    /// Send data to a peer
    SendData {
        peer_id: [u8; 32],
        data: Vec<u8>,
        priority: u8,
    },
    /// Connect to a peer at a specific address
    Connect { peer_id: [u8; 32], addr: Vec<u8> },
    /// Disconnect from a peer
    Disconnect { peer_id: [u8; 32] },
    /// Start peer discovery
    StartDiscovery,
    /// Stop peer discovery
    StopDiscovery,
}

impl fmt::Display for TransportCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportCommand::SendData {
                peer_id,
                data,
                priority,
            } => write!(f, "SendData {{ peer_id: {:x?}, data_len: {}, priority: {} }}", &peer_id[..8], data.len(), priority),
            TransportCommand::Connect { peer_id, .. } => {
                write!(f, "Connect {{ peer_id: {:x?} }}", &peer_id[..8])
            }
            TransportCommand::Disconnect { peer_id } => {
                write!(f, "Disconnect {{ peer_id: {:x?} }}", &peer_id[..8])
            }
            TransportCommand::StartDiscovery => write!(f, "StartDiscovery"),
            TransportCommand::StopDiscovery => write!(f, "StopDiscovery"),
        }
    }
}

/// Errors that can occur in the transport layer
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum TransportError {
    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Transport not available: {0}")]
    TransportNotAvailable(String),

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid payload: {0}")]
    InvalidPayload(String),

    #[error("Transport error: {0}")]
    TransportIoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_type_display() {
        assert_eq!(TransportType::BLE.to_string(), "BLE");
        assert_eq!(TransportType::WiFiAware.to_string(), "WiFiAware");
        assert_eq!(TransportType::WiFiDirect.to_string(), "WiFiDirect");
        assert_eq!(TransportType::Internet.to_string(), "Internet");
        assert_eq!(TransportType::Local.to_string(), "Local");
    }

    #[test]
    fn test_transport_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TransportType::BLE);
        set.insert(TransportType::WiFiAware);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&TransportType::BLE));
    }

    #[test]
    fn test_transport_capabilities_creation() {
        let caps = TransportCapabilities::new(512, false, true, 2_000_000, 50);
        assert_eq!(caps.max_payload_size, 512);
        assert!(!caps.supports_streaming);
        assert!(caps.is_bidirectional);
        assert_eq!(caps.estimated_bandwidth_bps, 2_000_000);
        assert_eq!(caps.estimated_latency_ms, 50);
    }

    #[test]
    fn test_transport_capabilities_for_ble() {
        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        assert_eq!(caps.max_payload_size, 512);
        assert!(!caps.supports_streaming);
        assert!(caps.is_bidirectional);
        assert_eq!(caps.estimated_bandwidth_bps, 2_000_000);
        assert_eq!(caps.estimated_latency_ms, 50);
    }

    #[test]
    fn test_transport_capabilities_for_wifi_aware() {
        let caps = TransportCapabilities::for_transport(TransportType::WiFiAware);
        assert_eq!(caps.max_payload_size, 2048);
        assert!(caps.supports_streaming);
        assert!(caps.is_bidirectional);
    }

    #[test]
    fn test_transport_capabilities_for_wifi_direct() {
        let caps = TransportCapabilities::for_transport(TransportType::WiFiDirect);
        assert_eq!(caps.max_payload_size, 4096);
        assert!(caps.supports_streaming);
        assert_eq!(caps.estimated_bandwidth_bps, 250_000_000);
    }

    #[test]
    fn test_transport_capabilities_for_internet() {
        let caps = TransportCapabilities::for_transport(TransportType::Internet);
        assert_eq!(caps.estimated_latency_ms, 100);
        assert_eq!(caps.max_payload_size, 8192);
    }

    #[test]
    fn test_transport_capabilities_for_local() {
        let caps = TransportCapabilities::for_transport(TransportType::Local);
        assert_eq!(caps.estimated_latency_ms, 1);
        assert_eq!(caps.max_payload_size, 65536);
        assert_eq!(caps.estimated_bandwidth_bps, 10_000_000_000);
    }

    #[test]
    fn test_transport_event_display() {
        let peer_id = [0u8; 32];
        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        let display = format!("{}", event);
        assert!(display.contains("PeerDiscovered"));
        assert!(display.contains("BLE"));
    }

    #[test]
    fn test_transport_command_display() {
        let peer_id = [0u8; 32];
        let cmd = TransportCommand::SendData {
            peer_id,
            data: vec![1, 2, 3],
            priority: 5,
        };
        let display = format!("{}", cmd);
        assert!(display.contains("SendData"));
        assert!(display.contains("priority: 5"));
    }

    #[test]
    fn test_transport_error_display() {
        let error = TransportError::PeerNotFound("test_peer".to_string());
        assert!(error.to_string().contains("Peer not found"));
    }

    #[test]
    fn test_transport_error_types() {
        let _e1 = TransportError::TransportNotAvailable("BLE".to_string());
        let _e2 = TransportError::SendFailed("connection lost".to_string());
        let _e3 = TransportError::ConnectionFailed("timeout".to_string());
        let _e4 = TransportError::InvalidPayload("too large".to_string());
        let _e5 = TransportError::TransportIoError("io error".to_string());
        let _e6 = TransportError::SerializationError("serde error".to_string());
        let _e7 = TransportError::Timeout("5s".to_string());
        let _e8 = TransportError::Internal("panic".to_string());
    }

    #[test]
    fn test_transport_event_peer_discovered() {
        let peer_id = [1u8; 32];
        let addr = vec![192, 168, 1, 1];
        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::WiFiDirect,
            addr,
        };
        match event {
            TransportEvent::PeerDiscovered {
                peer_id: pid,
                transport: t,
                addr: a,
            } => {
                assert_eq!(pid, [1u8; 32]);
                assert_eq!(t, TransportType::WiFiDirect);
                assert_eq!(a, vec![192, 168, 1, 1]);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_transport_event_data_received() {
        let peer_id = [2u8; 32];
        let data = vec![10, 20, 30];
        let event = TransportEvent::DataReceived {
            peer_id,
            transport: TransportType::Internet,
            data: data.clone(),
        };
        match event {
            TransportEvent::DataReceived {
                peer_id: pid,
                transport: t,
                data: d,
            } => {
                assert_eq!(pid, [2u8; 32]);
                assert_eq!(t, TransportType::Internet);
                assert_eq!(d, data);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_transport_command_send_data() {
        let peer_id = [3u8; 32];
        let data = vec![40, 50, 60];
        let cmd = TransportCommand::SendData {
            peer_id,
            data: data.clone(),
            priority: 10,
        };
        match cmd {
            TransportCommand::SendData {
                peer_id: pid,
                data: d,
                priority: p,
            } => {
                assert_eq!(pid, [3u8; 32]);
                assert_eq!(d, data);
                assert_eq!(p, 10);
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_transport_command_connect() {
        let peer_id = [4u8; 32];
        let addr = vec![1, 2, 3, 4, 5];
        let cmd = TransportCommand::Connect {
            peer_id,
            addr: addr.clone(),
        };
        match cmd {
            TransportCommand::Connect {
                peer_id: pid,
                addr: a,
            } => {
                assert_eq!(pid, [4u8; 32]);
                assert_eq!(a, addr);
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_transport_command_disconnect() {
        let peer_id = [5u8; 32];
        let cmd = TransportCommand::Disconnect { peer_id };
        match cmd {
            TransportCommand::Disconnect { peer_id: pid } => {
                assert_eq!(pid, [5u8; 32]);
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_serialization_transport_event() {
        let peer_id = [6u8; 32];
        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        let serialized = bincode::serialize(&event).expect("serialization failed");
        let deserialized: TransportEvent =
            bincode::deserialize(&serialized).expect("deserialization failed");
        match deserialized {
            TransportEvent::PeerDiscovered {
                peer_id: pid,
                transport: t,
                ..
            } => {
                assert_eq!(pid, peer_id);
                assert_eq!(t, TransportType::BLE);
            }
            _ => panic!("Wrong event type after deserialization"),
        }
    }

    #[test]
    fn test_all_transport_types_distinct() {
        let types = vec![
            TransportType::BLE,
            TransportType::WiFiAware,
            TransportType::WiFiDirect,
            TransportType::Internet,
            TransportType::Local,
        ];
        for i in 0..types.len() {
            for j in (i + 1)..types.len() {
                assert_ne!(types[i], types[j]);
            }
        }
    }

    #[test]
    fn test_transport_capabilities_clone() {
        let caps1 = TransportCapabilities::for_transport(TransportType::BLE);
        let caps2 = caps1.clone();
        assert_eq!(caps1.max_payload_size, caps2.max_payload_size);
        assert_eq!(caps1.estimated_bandwidth_bps, caps2.estimated_bandwidth_bps);
    }

    #[test]
    fn test_transport_error_clone() {
        let err1 = TransportError::PeerNotFound("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1.to_string(), err2.to_string());
    }
}
