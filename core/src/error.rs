//! SCMessenger core error types.
//!
//! All library errors are structured and typed. Applications may use anyhow
//! for ergonomics, but scmessenger_core never erases its own errors.
//!
//! This module provides a hierarchical error system for the mesh networking,
//! transport, and serialization layers.

use thiserror::Error;

// Re-export types needed for error variants
use crate::drift::relay::NetworkState;
use crate::routing::local::PeerId;

/// Top-level error for mesh operations.
///
/// This error type covers all mesh-level failures including sync protocol
/// errors, transport failures, storage issues, and authentication problems.
#[derive(Error, Debug)]
pub enum MeshError {
    /// Sync protocol version mismatch between peers.
    ///
    /// This occurs when two nodes attempt to sync but are running incompatible
    /// protocol versions. The sync operation should be aborted.
    #[error("sync protocol version mismatch: got {received}, expected {expected}")]
    VersionMismatch {
        /// The version received from the remote peer
        received: u32,
        /// The version expected by this node
        expected: u32,
    },

    /// Transport layer failure.
    ///
    /// Wraps all network-level errors including connection failures,
    /// handshake errors, and protocol violations.
    #[error("transport layer failure: {0}")]
    Transport(#[from] TransportError),

    /// Relay request denied due to peer state.
    ///
    /// The relay operation was rejected because the peer is not in an
    /// appropriate state to relay messages.
    #[error("relay denied: peer {peer_id:?} is in state {state:?}")]
    RelayDenied {
        /// The peer that denied the relay request
        peer_id: PeerId,
        /// The current state of the peer
        state: NetworkState,
    },

    /// Storage quota exceeded.
    ///
    /// The operation would exceed the configured storage limits.
    /// Consider evicting old messages or increasing the quota.
    #[error("storage quota exceeded: {used} / {max} bytes")]
    StorageQuota {
        /// Current storage usage in bytes
        used: usize,
        /// Maximum allowed storage in bytes
        max: usize,
    },

    /// Serialization or deserialization failure.
    ///
    /// Wraps all encoding/decoding errors from bincode or other serialization
    /// libraries.
    #[error("serialization failure: {0}")]
    Serialization(#[from] SerializationError),

    /// Peer authentication failed.
    ///
    /// The peer failed to prove its identity or the cryptographic proof
    /// was invalid.
    #[error("peer authentication failed: {0}")]
    Auth(String),

    /// IBLT (Invertible Bloom Lookup Table) decode failure.
    ///
    /// The IBLT data structure could not be decoded, possibly due to
    /// corruption or incompatible parameters.
    #[error("IBLT decode failure: {reason}")]
    IbltDecode {
        /// Description of why the decode failed
        reason: String,
    },

    /// Rate limit exceeded for peer.
    ///
    /// The peer has exceeded the maximum allowed sync request frequency.
    /// This is a DoS protection mechanism.
    #[error("rate limited: peer {peer_id:?}")]
    RateLimited {
        /// The peer that exceeded the rate limit
        peer_id: PeerId,
    },

    /// Invalid state for the requested operation.
    ///
    /// The operation cannot be completed because the component is in
    /// an unexpected state (e.g., not initialized, already running).
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Resource not found.
    ///
    /// The requested resource (message, peer, key, etc.) was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Cryptographic operation failed.
    ///
    /// Encryption, decryption, signing, or verification failed.
    #[error("cryptographic error: {0}")]
    Crypto(String),

    /// Configuration error.
    ///
    /// Invalid or missing configuration prevented the operation.
    #[error("configuration error: {0}")]
    Config(String),

    /// Invalid input provided.
    ///
    /// The caller provided invalid input for the operation.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Storage operation failed.
    ///
    /// Database or file system operation failed.
    #[error("storage error: {0}")]
    Storage(String),

    /// Identity operation failed.
    ///
    /// Identity key generation, loading, or validation failed.
    #[error("identity error: {0}")]
    Identity(String),

    /// Message operation failed.
    ///
    /// Message parsing, validation, or processing failed.
    #[error("message error: {0}")]
    Message(String),
}

/// Transport layer errors.
///
/// These errors represent failures at the network transport level,
/// including connection establishment, encryption handshakes, and
/// connection lifecycle issues.
#[derive(Error, Debug)]
pub enum TransportError {
    /// Noise protocol handshake failed.
    ///
    /// The cryptographic handshake for establishing a secure channel
    /// could not be completed. This may indicate a MITM attack or
    /// incompatible protocol versions.
    #[error("noise handshake failed: {0}")]
    NoiseHandshake(String),

    /// Connection reset by peer.
    ///
    /// The remote peer closed the connection unexpectedly.
    #[error("connection reset by peer: {peer_id:?}")]
    ConnectionReset {
        /// The peer that reset the connection
        peer_id: PeerId,
    },

    /// I/O error during transport operation.
    ///
    /// Wraps standard I/O errors from the underlying transport.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Connection timeout.
    ///
    /// The connection attempt timed out before completing.
    #[error("connection timeout: {0}")]
    Timeout(String),

    /// Dial failure.
    ///
    /// Failed to establish a connection to the peer.
    #[error("dial failed: {peer_id:?}: {reason}")]
    DialFailed {
        /// The peer we tried to connect to
        peer_id: PeerId,
        /// Reason for failure
        reason: String,
    },

    /// No active connection to peer.
    ///
    /// The peer is not currently connected.
    #[error("not connected to peer: {peer_id:?}")]
    NotConnected {
        /// The peer that is not connected
        peer_id: PeerId,
    },

    /// Invalid multiaddress.
    ///
    /// The provided multiaddress could not be parsed or is invalid.
    #[error("invalid multiaddress: {0}")]
    InvalidMultiaddr(String),

    /// Transport protocol error.
    ///
    /// Protocol violation or unexpected message during transport.
    #[error("transport protocol error: {0}")]
    ProtocolError(String),
}

/// Serialization and deserialization errors.
///
/// These errors occur when encoding or decoding data structures
/// for network transmission or storage.
#[derive(Error, Debug)]
pub enum SerializationError {
    /// Bincode encoding failed.
    ///
    /// The data structure could not be serialized to binary format.
    /// This is typically a programming error (e.g., trying to serialize
    /// an unsupported type).
    #[error("bincode encode failed: {0}")]
    Encode(#[from] Box<bincode::ErrorKind>),

    /// Schema version not supported.
    ///
    /// The data was encoded with a schema version that this node
    /// does not understand. An upgrade may be required.
    #[error("schema version {version} not supported")]
    UnsupportedVersion {
        /// The unsupported schema version
        version: u16,
    },

    /// JSON serialization error.
    ///
    /// Wraps serde_json errors for JSON encoding/decoding.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid UTF-8 string.
    ///
    /// A string contained invalid UTF-8 bytes.
    #[error("invalid UTF-8: {0}")]
    InvalidUtf8(String),

    /// Unexpected data format.
    ///
    /// The data format was unexpected or malformed.
    #[error("unexpected format: expected {expected}, got {got}")]
    UnexpectedFormat {
        /// Expected format description
        expected: String,
        /// Actual format found
        got: String,
    },

    /// Data too large.
    ///
    /// The data exceeds the maximum allowed size.
    #[error("data too large: {size} bytes (max: {max})")]
    TooLarge {
        /// Actual size in bytes
        size: usize,
        /// Maximum allowed size
        max: usize,
    },

    /// Missing required field.
    ///
    /// A required field was missing during deserialization.
    #[error("missing required field: {0}")]
    MissingField(String),
}

/// Result type alias for mesh operations.
pub type MeshResult<T> = Result<T, MeshError>;

/// Result type alias for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

/// Result type alias for serialization operations.
pub type SerializationResult<T> = Result<T, SerializationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_error_display() {
        let err = MeshError::VersionMismatch {
            received: 2,
            expected: 1,
        };
        assert_eq!(
            err.to_string(),
            "sync protocol version mismatch: got 2, expected 1"
        );
    }

    #[test]
    fn test_transport_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "test");
        let transport_err: TransportError = io_err.into();
        assert!(transport_err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_serialization_error_from_json() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let ser_err: SerializationError = json_err.into();
        assert!(ser_err.to_string().contains("JSON error"));
    }

    #[test]
    fn test_mesh_error_from_transport() {
        let transport_err = TransportError::NoiseHandshake("test failure".to_string());
        let mesh_err: MeshError = transport_err.into();
        assert!(mesh_err.to_string().contains("transport layer failure"));
    }

    #[test]
    fn test_mesh_error_from_serialization() {
        let ser_err = SerializationError::UnsupportedVersion { version: 99 };
        let mesh_err: MeshError = ser_err.into();
        assert!(mesh_err.to_string().contains("serialization failure"));
    }
}
