//! Audit logging for SCMessenger core operations.
//!
//! Provides immutable, cryptographically-linked audit trails for security-critical
//! operations. Each event is chained using Blake3 hashes to detect tampering.

use serde::{Deserialize, Serialize};
use std::fmt;
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

/// Immutable, sequentially-hashed audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// Chronologically ordered events
    pub events: Vec<AuditEvent>,
    /// Hash of most recent event for fast append
    pub last_hash: String,
}

impl AuditLog {
    /// Create a new empty audit log with genesis hash
    pub fn new() -> Self {
        let genesis_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        AuditLog {
            events: Vec::new(),
            last_hash: genesis_hash,
        }
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

    /// Verify cryptographic integrity of entire log
    pub fn validate_chain(&self) -> Result<(), AuditLogError> {
        if self.events.is_empty() {
            return Err(AuditLogError::EmptyLog);
        }

        let genesis_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        if self.events[0].prev_hash != genesis_hash {
            return Err(AuditLogError::ChainBroken {
                event_index: 0,
                expected: genesis_hash.to_string(),
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
    ChainBroken {
        /// Index of first corrupted event
        event_index: usize,
        /// Expected hash value
        expected: String,
        /// Actual hash value found
        found: String,
    },
    /// Validation performed on empty log
    EmptyLog,
}

impl fmt::Display for AuditLogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditLogError::ChainBroken {
                event_index,
                expected,
                found,
            } => write!(
                f,
                "Audit chain broken at event {}: expected hash {}, found {}",
                event_index, expected, found
            ),
            AuditLogError::EmptyLog => write!(f, "Audit log is empty"),
        }
    }
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
