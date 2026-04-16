//! Audit logging for SCMessenger core operations.
//!
//! Provides immutable, cryptographically-linked audit trails for security-critical
//! operations. Each event is chained using Blake3 hashes to detect tampering.
//!
//! P0_SECURITY_005: Audit events are persisted to the storage backend and subject
//! to time-based retention (default: 365 days). Pruning preserves chain integrity
//! by keeping the oldest remaining event's prev_hash pointing to the pruned event's
//! hash, which is recorded in the `pruned_head_hash` field.

use crate::store::backend::StorageBackend;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// Type of audit event for categorization and policy enforcement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// New identity generated via `initialize_identity()`
    IdentityCreated,
    /// Identity explicitly deleted via backup deletion or factory reset
    IdentityDeleted,
    /// Encrypted message transmitted via `send_message()`
    MessageSent,
    /// Encrypted message received and successfully decrypted
    MessageReceived,
    /// Relay registration enabled via settings toggle
    RelayEnabled,
    /// Relay registration disabled via settings toggle  
    RelayDisabled,
    /// New contact added via QR scan or import
    ContactAdded,
    /// Contact explicitly blocked via block list
    ContactBlocked,
    /// Contact removed via explicit action
    ContactRemoved,
    /// Identity keys exported to platform backup
    BackupExported,
    /// Identity keys restored from platform backup
    BackupImported,
    /// Explicit consent granted for sensitive operation
    ConsentGranted,
    /// Storage compaction executed to reclaim space
    StorageCompacted,
}

impl fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditEventType::IdentityCreated => write!(f, "IdentityCreated"),
            AuditEventType::IdentityDeleted => write!(f, "IdentityDeleted"),
            AuditEventType::MessageSent => write!(f, "MessageSent"),
            AuditEventType::MessageReceived => write!(f, "MessageReceived"),
            AuditEventType::RelayEnabled => write!(f, "RelayEnabled"),
            AuditEventType::RelayDisabled => write!(f, "RelayDisabled"),
            AuditEventType::ContactAdded => write!(f, "ContactAdded"),
            AuditEventType::ContactBlocked => write!(f, "ContactBlocked"),
            AuditEventType::ContactRemoved => write!(f, "ContactRemoved"),
            AuditEventType::BackupExported => write!(f, "BackupExported"),
            AuditEventType::BackupImported => write!(f, "BackupImported"),
            AuditEventType::ConsentGranted => write!(f, "ConsentGranted"),
            AuditEventType::StorageCompacted => write!(f, "StorageCompacted"),
        }
    }
}

/// Individual auditable event with cryptographic chaining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier (UUIDv4)
    pub event_id: String,
    /// Semantic event category
    pub event_type: AuditEventType,
    /// Unix timestamp of event creation (seconds since epoch)
    pub timestamp_unix_secs: u64,
    /// Local identity scope of event (Blake3 of public key)
    pub identity_id: Option<String>,
    /// Remote peer involved in event (libp2p PeerId or Blake3)
    pub peer_id: Option<String>,
    /// Human-readable supplementary details
    pub details: Option<String>,
    /// Hash of prior event for integrity verification
    pub prev_hash: String,
}

impl fmt::Display for AuditEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuditEvent({}, {}, {})",
            self.event_id, self.event_type, self.timestamp_unix_secs
        )
    }
}

impl AuditEvent {
    /// Create a new audit event with generated ID and timestamp
    pub fn new(
        event_type: AuditEventType,
        identity_id: Option<String>,
        peer_id: Option<String>,
        details: Option<String>,
        prev_hash: String,
    ) -> Self {
        let event_id = Uuid::new_v4().to_string();
        let timestamp_unix_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        AuditEvent {
            event_id,
            event_type,
            timestamp_unix_secs,
            identity_id,
            peer_id,
            details,
            prev_hash,
        }
    }

    /// Compute cryptographic hash of this event for chaining
    pub fn chain_hash(&self) -> String {
        let json = serde_json::to_string(self).expect("Failed to serialize AuditEvent");
        let hash = blake3::hash(json.as_bytes());
        hash.to_hex().to_string()
    }
}

/// Immutable, sequentially-hashed audit log with persistence support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// Chronologically ordered events
    pub events: Vec<AuditEvent>,
    /// Hash of most recent event for fast append
    pub last_hash: String,
    /// P0_SECURITY_005: Hash of the last event before pruning, used to
    /// maintain chain integrity when old events are pruned.
    #[serde(default)]
    pub pruned_head_hash: Option<String>,
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLog {
    /// Create a new empty audit log with genesis hash
    pub fn new() -> Self {
        let genesis_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        AuditLog {
            events: Vec::new(),
            last_hash: genesis_hash,
            pruned_head_hash: None,
        }
    }

    /// P0_SECURITY_005: Persist the audit log to the storage backend.
    /// The entire log is serialized as JSON and stored under a single key.
    pub fn persist(&self, backend: &Arc<dyn StorageBackend>) -> Result<(), AuditLogError> {
        let json = serde_json::to_string(self).map_err(|_| AuditLogError::PersistenceError)?;
        backend
            .put(b"audit_log_v1", json.as_bytes())
            .map_err(|_| AuditLogError::PersistenceError)
    }

    /// P0_SECURITY_005: Load the audit log from the storage backend.
    /// Returns a new empty log if no persisted log exists.
    pub fn load(backend: &Arc<dyn StorageBackend>) -> Self {
        match backend.get(b"audit_log_v1") {
            Ok(Some(data)) => {
                let json = String::from_utf8_lossy(&data);
                match serde_json::from_str::<AuditLog>(&json) {
                    Ok(log) => log,
                    Err(e) => {
                        tracing::warn!("Failed to deserialize audit log, starting fresh: {}", e);
                        AuditLog::new()
                    }
                }
            }
            _ => AuditLog::new(),
        }
    }

    /// P0_SECURITY_005: Prune events older than `before_timestamp` (unix seconds).
    /// Preserves chain integrity by recording the hash of the last pruned event
    /// in `pruned_head_hash`, so validation can still detect tampering of remaining events.
    pub fn prune_before(&mut self, before_timestamp: u64) -> u32 {
        let split_idx = self
            .events
            .iter()
            .position(|e| e.timestamp_unix_secs >= before_timestamp);

        let prune_count = match split_idx {
            Some(0) => 0, // Nothing to prune
            Some(idx) => {
                // Record the chain hash of the last pruned event for integrity
                if idx > 0 {
                    self.pruned_head_hash = Some(self.events[idx - 1].chain_hash());
                }
                let count = idx as u32;
                self.events.drain(0..idx);
                count
            }
            None => {
                // All events are older than the cutoff
                if !self.events.is_empty() {
                    self.pruned_head_hash = Some(self.events.last().unwrap().chain_hash());
                    let count = self.events.len() as u32;
                    self.events.clear();
                    count
                } else {
                    0
                }
            }
        };

        // Update last_hash if we pruned everything
        if self.events.is_empty() {
            self.last_hash = self
                .pruned_head_hash
                .clone()
                .unwrap_or_else(|| {
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string()
                });
        }

        prune_count
    }

    /// Append a new event to the audit trail and return it
    pub fn append(
        &mut self,
        event_type: AuditEventType,
        identity_id: Option<String>,
        peer_id: Option<String>,
        details: Option<String>,
    ) -> AuditEvent {
        let event = AuditEvent::new(
            event_type,
            identity_id,
            peer_id,
            details,
            self.last_hash.clone(),
        );
        self.last_hash = event.chain_hash();
        self.events.push(event.clone());
        event
    }

    /// Verify cryptographic integrity of entire log.
    /// If events were pruned, the first remaining event's prev_hash must match
    /// the pruned_head_hash (the chain hash of the last pruned event).
    pub fn validate_chain(&self) -> Result<(), AuditLogError> {
        if self.events.is_empty() {
            return Err(AuditLogError::EmptyLog);
        }

        // The first event must link to either the genesis hash or the pruned head hash
        let expected_head = self
            .pruned_head_hash
            .as_deref()
            .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");

        if self.events[0].prev_hash != expected_head {
            return Err(AuditLogError::ChainBroken {
                event_index: 0,
                expected: expected_head.to_string(),
                found: self.events[0].prev_hash.clone(),
            });
        }

        for i in 1..self.events.len() {
            let expected_prev_hash = self.events[i - 1].chain_hash();
            if self.events[i].prev_hash != expected_prev_hash {
                return Err(AuditLogError::ChainBroken {
                    event_index: i,
                    expected: expected_prev_hash,
                    found: self.events[i].prev_hash.clone(),
                });
            }
        }

        Ok(())
    }
}

/// Errors that can occur during audit log validation
#[derive(Debug, Clone, Error)]
pub enum AuditLogError {
    /// Cryptographic chain integrity check failed
    #[error("Audit chain broken at event {event_index}: expected hash {expected}, found {found}")]
    ChainBroken {
        /// Index of first corrupted event
        event_index: usize,
        /// Expected hash value
        expected: String,
        /// Actual hash value found
        found: String,
    },
    /// Validation performed on empty log
    #[error("Audit log is empty")]
    EmptyLog,
    /// P0_SECURITY_005: Failed to persist or load audit log
    #[error("Audit log persistence error")]
    PersistenceError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::IdentityCreated,
            Some("test_identity".to_string()),
            Some("test_peer".to_string()),
            Some("Test details".to_string()),
            "prev_hash".to_string(),
        );

        // Verify UUID format
        assert!(uuid::Uuid::parse_str(&event.event_id).is_ok());
        assert_eq!(event.event_id.len(), 36); // Standard UUID hyphenated format

        // Verify timestamp is reasonable (within last hour)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(event.timestamp_unix_secs <= now);
        assert!(event.timestamp_unix_secs > now - 3600);

        // Verify fields match
        assert_eq!(event.event_type, AuditEventType::IdentityCreated);
        assert_eq!(event.identity_id, Some("test_identity".to_string()));
        assert_eq!(event.peer_id, Some("test_peer".to_string()));
        assert_eq!(event.details, Some("Test details".to_string()));
        assert_eq!(event.prev_hash, "prev_hash".to_string());
    }

    #[test]
    fn test_chain_hash_determinism() {
        let event1 = AuditEvent {
            event_id: "test-id".to_string(),
            event_type: AuditEventType::MessageSent,
            timestamp_unix_secs: 1000000,
            identity_id: Some("sender".to_string()),
            peer_id: Some("receiver".to_string()),
            details: None,
            prev_hash: "deadbeef".to_string(),
        };

        let event2 = AuditEvent {
            event_id: "test-id".to_string(),
            event_type: AuditEventType::MessageSent,
            timestamp_unix_secs: 1000000,
            identity_id: Some("sender".to_string()),
            peer_id: Some("receiver".to_string()),
            details: None,
            prev_hash: "deadbeef".to_string(),
        };

        assert_eq!(event1.chain_hash(), event2.chain_hash());
    }

    #[test]
    fn test_valid_chain() {
        let mut log = AuditLog::new();
        log.append(
            AuditEventType::IdentityCreated,
            Some("id1".to_string()),
            None,
            None,
        );
        log.append(
            AuditEventType::ContactAdded,
            Some("id1".to_string()),
            Some("peer1".to_string()),
            None,
        );
        log.append(
            AuditEventType::MessageSent,
            Some("id1".to_string()),
            Some("peer1".to_string()),
            Some("Hello".to_string()),
        );

        assert!(log.validate_chain().is_ok());
    }

    #[test]
    fn test_tampered_chain_detection() {
        let mut log = AuditLog::new();
        log.append(
            AuditEventType::IdentityCreated,
            Some("id1".to_string()),
            None,
            None,
        );
        log.append(
            AuditEventType::ContactAdded,
            Some("id1".to_string()),
            Some("peer1".to_string()),
            None,
        );
        // Append a third event so that event[2].prev_hash references event[1].chain_hash().
        // Tampering event[1] will then be detected when event[2]'s prev_hash is validated.
        log.append(
            AuditEventType::MessageSent,
            Some("id1".to_string()),
            Some("peer1".to_string()),
            None,
        );

        // Tamper with the second event
        log.events[1].details = Some("Tampered details".to_string());

        match log.validate_chain() {
            Err(AuditLogError::ChainBroken {
                event_index,
                expected,
                found,
            }) => {
                // The break is detected at event[2], whose prev_hash no longer matches
                // the recomputed chain_hash of the tampered event[1].
                assert_eq!(event_index, 2);
                assert_ne!(expected, found);
            }
            _ => panic!("Expected ChainBroken error"),
        }
    }

    #[test]
    fn test_empty_log_validation() {
        let log = AuditLog::new();
        match log.validate_chain() {
            Err(AuditLogError::EmptyLog) => {}
            _ => panic!("Expected EmptyLog error"),
        }
    }

    #[test]
    fn test_multi_event_chaining() {
        let mut log = AuditLog::new();
        let genesis_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        assert_eq!(log.last_hash, genesis_hash);

        let event1 = log.append(
            AuditEventType::IdentityCreated,
            Some("id1".to_string()),
            None,
            None,
        );
        assert_eq!(log.last_hash, event1.chain_hash());
        assert_eq!(log.events.len(), 1);

        let event2 = log.append(
            AuditEventType::ContactAdded,
            Some("id1".to_string()),
            Some("peer1".to_string()),
            None,
        );
        assert_eq!(log.last_hash, event2.chain_hash());
        assert_eq!(log.events.len(), 2);

        let event3 = log.append(
            AuditEventType::MessageSent,
            Some("id1".to_string()),
            Some("peer1".to_string()),
            Some("Test message".to_string()),
        );
        assert_eq!(log.last_hash, event3.chain_hash());
        assert_eq!(log.events.len(), 3);

        let event4 = log.append(
            AuditEventType::MessageReceived,
            Some("id1".to_string()),
            Some("peer2".to_string()),
            None,
        );
        assert_eq!(log.last_hash, event4.chain_hash());
        assert_eq!(log.events.len(), 4);

        let event5 = log.append(
            AuditEventType::BackupExported,
            Some("id1".to_string()),
            None,
            Some("Encrypted backup".to_string()),
        );
        assert_eq!(log.last_hash, event5.chain_hash());
        assert_eq!(log.events.len(), 5);

        assert!(log.validate_chain().is_ok());
    }
}
