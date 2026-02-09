// Address Reflection Service - Peer-assisted NAT discovery
//
// Each node provides this service to help other nodes discover their external
// address without relying on external STUN servers. This is the in-mesh
// equivalent of STUN, maintaining the sovereign mesh architecture.
//
// Protocol:
// 1. Node sends AddressReflectionRequest to peer via libp2p
// 2. Peer responds with AddressReflectionResponse containing observed source address
// 3. Node aggregates responses from multiple peers to determine NAT type

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ============================================================================
// MESSAGE PROTOCOL
// ============================================================================

/// Request for address reflection
///
/// Sent to a mesh peer to ask "what's my external address?"
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressReflectionRequest {
    /// Unique request identifier for matching request/response
    pub request_id: [u8; 16],
    /// Protocol version (for future compatibility)
    pub version: u8,
}

impl AddressReflectionRequest {
    /// Create a new address reflection request with random ID
    pub fn new() -> Self {
        let mut request_id = [0u8; 16];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut request_id);

        Self {
            request_id,
            version: 1,
        }
    }

    /// Create with specific request ID (for testing)
    pub fn with_id(request_id: [u8; 16]) -> Self {
        Self {
            request_id,
            version: 1,
        }
    }
}

impl Default for AddressReflectionRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from address reflection
///
/// Contains the observed source address as seen by the responding peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressReflectionResponse {
    /// Echo of request ID (for matching)
    pub request_id: [u8; 16],
    /// The observed source address (IP:port) of the requester
    pub observed_address: String,
    /// Protocol version
    pub version: u8,
}

impl AddressReflectionResponse {
    /// Create a new response
    pub fn new(request_id: [u8; 16], observed_address: SocketAddr) -> Self {
        Self {
            request_id,
            observed_address: observed_address.to_string(),
            version: 1,
        }
    }

    /// Parse observed address as SocketAddr
    pub fn parse_address(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        self.observed_address.parse()
    }
}

// ============================================================================
// SERVICE IMPLEMENTATION
// ============================================================================

/// Address reflection service
///
/// Each node runs this service to help other nodes discover their external
/// address. This is the sovereign mesh equivalent of a STUN server.
///
/// Statistics tracking:
/// - Total requests served
/// - Requests per peer
/// - Service uptime
pub struct AddressReflectionService {
    /// Total number of reflection requests served
    requests_served: Arc<AtomicU64>,
    /// Service enabled/disabled
    enabled: bool,
}

impl AddressReflectionService {
    /// Create a new address reflection service
    pub fn new() -> Self {
        Self {
            requests_served: Arc::new(AtomicU64::new(0)),
            enabled: true,
        }
    }

    /// Handle an address reflection request
    ///
    /// Takes the request and the observed source address from the transport layer,
    /// returns a response containing that observed address.
    ///
    /// # Arguments
    /// * `request` - The incoming AddressReflectionRequest
    /// * `observed_addr` - The source address observed at transport layer
    ///
    /// # Returns
    /// AddressReflectionResponse with the observed address
    pub fn handle_request(
        &self,
        request: AddressReflectionRequest,
        observed_addr: SocketAddr,
    ) -> AddressReflectionResponse {
        if !self.enabled {
            tracing::warn!("Address reflection request received but service is disabled");
        }

        // Increment service counter
        self.requests_served.fetch_add(1, Ordering::Relaxed);

        tracing::debug!(
            "Serving address reflection request {:?}, observed address: {}",
            request.request_id,
            observed_addr
        );

        AddressReflectionResponse::new(request.request_id, observed_addr)
    }

    /// Get total number of requests served
    pub fn requests_served(&self) -> u64 {
        self.requests_served.load(Ordering::Relaxed)
    }

    /// Enable the service
    pub fn enable(&mut self) {
        self.enabled = true;
        tracing::info!("Address reflection service enabled");
    }

    /// Disable the service (for testing or resource conservation)
    pub fn disable(&mut self) {
        self.enabled = false;
        tracing::info!("Address reflection service disabled");
    }

    /// Check if service is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Reset statistics (for testing)
    pub fn reset_stats(&self) {
        self.requests_served.store(0, Ordering::Relaxed);
    }
}

impl Default for AddressReflectionService {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CODEC FOR LIBP2P INTEGRATION
// ============================================================================

/// Codec for address reflection protocol
///
/// Handles serialization/deserialization of request/response messages
/// for libp2p request-response protocol.
pub mod codec {
    use super::*;
    use anyhow::Result;

    /// Encode request to bytes
    pub fn encode_request(request: &AddressReflectionRequest) -> Result<Vec<u8>> {
        bincode::serialize(request).map_err(|e| anyhow::anyhow!("Failed to encode request: {}", e))
    }

    /// Decode request from bytes
    pub fn decode_request(bytes: &[u8]) -> Result<AddressReflectionRequest> {
        bincode::deserialize(bytes).map_err(|e| anyhow::anyhow!("Failed to decode request: {}", e))
    }

    /// Encode response to bytes
    pub fn encode_response(response: &AddressReflectionResponse) -> Result<Vec<u8>> {
        bincode::serialize(response)
            .map_err(|e| anyhow::anyhow!("Failed to encode response: {}", e))
    }

    /// Decode response from bytes
    pub fn decode_response(bytes: &[u8]) -> Result<AddressReflectionResponse> {
        bincode::deserialize(bytes)
            .map_err(|e| anyhow::anyhow!("Failed to decode response: {}", e))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let request = AddressReflectionRequest::new();
        assert_eq!(request.version, 1);
        assert_ne!(request.request_id, [0u8; 16]); // Should be random
    }

    #[test]
    fn test_request_with_id() {
        let id = [1u8; 16];
        let request = AddressReflectionRequest::with_id(id);
        assert_eq!(request.request_id, id);
        assert_eq!(request.version, 1);
    }

    #[test]
    fn test_response_creation() {
        let id = [2u8; 16];
        let addr: SocketAddr = "203.0.113.1:30000".parse().unwrap();
        let response = AddressReflectionResponse::new(id, addr);

        assert_eq!(response.request_id, id);
        assert_eq!(response.observed_address, "203.0.113.1:30000");
        assert_eq!(response.version, 1);
    }

    #[test]
    fn test_response_parse_address() {
        let id = [3u8; 16];
        let addr: SocketAddr = "192.168.1.100:8080".parse().unwrap();
        let response = AddressReflectionResponse::new(id, addr);

        let parsed = response.parse_address().unwrap();
        assert_eq!(parsed, addr);
    }

    #[test]
    fn test_service_creation() {
        let service = AddressReflectionService::new();
        assert_eq!(service.requests_served(), 0);
        assert!(service.is_enabled());
    }

    #[test]
    fn test_service_handle_request() {
        let service = AddressReflectionService::new();
        let request = AddressReflectionRequest::with_id([4u8; 16]);
        let observed_addr: SocketAddr = "10.0.0.5:9999".parse().unwrap();

        let response = service.handle_request(request.clone(), observed_addr);

        assert_eq!(response.request_id, request.request_id);
        assert_eq!(response.observed_address, "10.0.0.5:9999");
        assert_eq!(service.requests_served(), 1);
    }

    #[test]
    fn test_service_multiple_requests() {
        let service = AddressReflectionService::new();
        let addr: SocketAddr = "1.2.3.4:5000".parse().unwrap();

        for i in 0..10 {
            let request = AddressReflectionRequest::with_id([i; 16]);
            service.handle_request(request, addr);
        }

        assert_eq!(service.requests_served(), 10);
    }

    #[test]
    fn test_service_enable_disable() {
        let mut service = AddressReflectionService::new();
        assert!(service.is_enabled());

        service.disable();
        assert!(!service.is_enabled());

        service.enable();
        assert!(service.is_enabled());
    }

    #[test]
    fn test_service_reset_stats() {
        let service = AddressReflectionService::new();
        let request = AddressReflectionRequest::new();
        let addr: SocketAddr = "127.0.0.1:1234".parse().unwrap();

        service.handle_request(request, addr);
        assert_eq!(service.requests_served(), 1);

        service.reset_stats();
        assert_eq!(service.requests_served(), 0);
    }

    #[test]
    fn test_codec_request_roundtrip() {
        let request = AddressReflectionRequest::with_id([5u8; 16]);

        let encoded = codec::encode_request(&request).unwrap();
        let decoded = codec::decode_request(&encoded).unwrap();

        assert_eq!(decoded, request);
    }

    #[test]
    fn test_codec_response_roundtrip() {
        let response = AddressReflectionResponse::new(
            [6u8; 16],
            "198.51.100.42:7777".parse().unwrap(),
        );

        let encoded = codec::encode_response(&response).unwrap();
        let decoded = codec::decode_response(&encoded).unwrap();

        assert_eq!(decoded, response);
    }

    #[test]
    fn test_codec_invalid_data() {
        let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
        assert!(codec::decode_request(&invalid_bytes).is_err());
        assert!(codec::decode_response(&invalid_bytes).is_err());
    }

    #[test]
    fn test_request_default() {
        let request = AddressReflectionRequest::default();
        assert_eq!(request.version, 1);
    }

    #[test]
    fn test_service_default() {
        let service = AddressReflectionService::default();
        assert_eq!(service.requests_served(), 0);
        assert!(service.is_enabled());
    }
}
