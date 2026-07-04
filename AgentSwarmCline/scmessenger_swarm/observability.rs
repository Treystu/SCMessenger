//! SCMessenger v0.2.1 Observability Component
//! 
//! This module implements the structured tracing payload for mandatory relay protocol events.
//! It provides the `RelayTracePayload` struct that captures essential telemetry data about
//! message processing through relay nodes.

use std::fmt;

/// Represents a structured tracing payload for mandatory relay protocol events.
/// 
/// This struct captures key telemetry information about how messages traverse the SCMessenger
/// relay network, including identification, node information, and performance metrics.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RelayTracePayload {
    /// The unique identifier of the SCMessenger payload traversing the relay.
    /// Must not be empty.
    pub message_id: String,
    
    /// The cryptographic hash identifying the specific relay node processing the message.
    /// Must be a 64-character SHA-256 hex string.
    pub relay_node_hash: String,
    
    /// The processing latency introduced by the relay node, measured in milliseconds.
    /// `0` is permissible for local loopback scenarios.
    pub latency_ms: u64,
}

impl fmt::Display for RelayTracePayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "[RelayTrace] msg_id={}, node={}, latency={}ms", 
            self.message_id, 
            self.relay_node_hash, 
            self.latency_ms
        )
    }
}

impl RelayTracePayload {
    /// Validates the integrity and correctness of the relay trace payload.
    /// 
    /// # Returns
    /// * `Ok(())` if all validation checks pass
    /// * `Err(RelayTraceError)` if any validation constraint is violated
    /// 
    /// # Validation Rules
    /// 1. `message_id` must not be empty
    /// 2. `relay_node_hash` must be exactly 64 hexadecimal characters
    pub fn validate(&self) -> Result<(), RelayTraceError> {
        // Validate message_id is not empty
        if self.message_id.is_empty() {
            return Err(RelayTraceError::EmptyMessageId);
        }
        
        // Validate relay_node_hash
        if self.relay_node_hash.is_empty() || self.relay_node_hash.len() != 64 || 
           !self.relay_node_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(RelayTraceError::InvalidNodeHash);
        }
        
        Ok(())
    }
}

/// Error types that can occur during validation of relay trace payloads.
#[derive(Debug)]
pub enum RelayTraceError {
    /// The message ID field was empty.
    EmptyMessageId,
    
    /// The relay node hash was invalid (empty, wrong length, or non-hex characters).
    InvalidNodeHash,
}

impl fmt::Display for RelayTraceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelayTraceError::EmptyMessageId => write!(f, "Message ID cannot be empty"),
            RelayTraceError::InvalidNodeHash => write!(f, "Relay node hash must be a 64-character hexadecimal string"),
        }
    }
}

impl std::error::Error for RelayTraceError {}