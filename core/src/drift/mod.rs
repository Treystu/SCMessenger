//! Drift Protocol — compact binary format for mesh relay efficiency
//!
//! This module provides:
//! - DriftEnvelope: Fixed-width, binary-encoded envelope format (186 bytes overhead)
//! - DriftFrame: Transport layer framing with length, type, and CRC32
//! - LZ4 compression: Optional payload compression
//! - CRDT Mesh Network: Conflict-free replicated data structures for message stores
//! - IBLT Sketch: Invertible Bloom Lookup Table for efficient set reconciliation
//! - Sync Protocol: Mesh synchronization using IBLT for optimal bandwidth usage
//!
//! Format progression:
//! 1. DriftEnvelope: raw encrypted message with metadata
//! 2. DriftFrame: transport wrapper adding length, type, and CRC32
//! 3. Optional compression: LZ4 for large payloads
//! 4. MeshStore: CRDT-based message synchronization without conflicts
//! 5. IBLT Sketch: Set reconciliation for O(d) communication complexity
//! 6. SyncSession: State machine for coordinating multi-step sync protocol

pub mod compress;
pub mod envelope;
pub mod frame;
pub mod store;
pub mod sketch;
pub mod sync;
pub mod relay;
pub mod policy;

pub use envelope::{DriftEnvelope, EnvelopeType};
pub use frame::{DriftFrame, FrameType, FRAME_READ_TIMEOUT, FRAME_MAX_PAYLOAD};
pub use store::{MeshStore, MessageId, StoredEnvelope};
pub use sketch::IBLT;
pub use sync::{SyncMessage, SyncSession, SyncState, merge_envelopes};
pub use relay::{RelayEngine, RelayConfig, NetworkState, RelayDecision, DropReason, RelayError};
pub use policy::{PolicyEngine, DeviceState, RelayProfile, PolicyError};

use thiserror::Error;

/// Drift Protocol errors
#[derive(Debug, Error, Clone)]
pub enum DriftError {
    #[error("Buffer too short: need {need} bytes, got {got}")]
    BufferTooShort { need: usize, got: usize },

    #[error("Invalid envelope type: {0}")]
    InvalidEnvelopeType(u8),

    #[error("Ciphertext too large: {0} bytes (max {MAX})", MAX = DriftEnvelope::MAX_CIPHERTEXT)]
    CiphertextTooLarge(usize),

    #[error("Invalid version: {0}")]
    InvalidVersion(u8),

    #[error("CRC32 mismatch")]
    CrcMismatch,

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Invalid frame type: {0}")]
    InvalidFrameType(u8),

    #[error("Frame read timeout — possible Slow Loris attack")]
    Timeout,

    #[error("IO error: {0}")]
    IoError(String),
}

/// Current Drift Protocol version
pub const DRIFT_VERSION: u8 = 0x01;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_version_constant() {
        assert_eq!(DRIFT_VERSION, 0x01);
    }
}
