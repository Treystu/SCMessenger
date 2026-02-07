// Iron Core V2 — Messaging Spine
//
// "Does this help two humans exchange an encrypted message
//  without any corporation in the middle?"
//
// If the answer is no, it doesn't belong in Phase 0.

// Allow clippy lint triggered by UniFFI-generated scaffolding code
#![allow(clippy::empty_line_after_doc_comments)]

pub mod crypto;
pub mod drift;
pub mod identity;
pub mod message;
pub mod routing;
pub mod store;
pub mod transport;

use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;

pub use crypto::{decrypt_message, encrypt_message};
pub use identity::IdentityManager;
pub use message::{DeliveryStatus, Envelope, Message, MessageType, Receipt};

// UniFFI exports
uniffi::include_scaffolding!("api");

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error, Clone)]
pub enum IronCoreError {
    #[error("Not initialized")]
    NotInitialized,
    #[error("Already running")]
    AlreadyRunning,
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Cryptography error: {0}")]
    CryptoError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for IronCoreError {
    fn from(err: anyhow::Error) -> Self {
        IronCoreError::Internal(err.to_string())
    }
}

// ============================================================================
// DATA TYPES
// ============================================================================

/// Identity information for UniFFI export
#[derive(Clone)]
pub struct IdentityInfo {
    pub identity_id: Option<String>,
    pub public_key_hex: Option<String>,
    pub initialized: bool,
}

/// Signature result for UniFFI export
#[derive(Clone)]
pub struct SignatureResult {
    pub signature: Vec<u8>,
    pub public_key_hex: String,
}

/// Message info for UniFFI export (simplified view of Message)
#[derive(Clone)]
pub struct MessageInfo {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub message_type: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

impl From<Message> for MessageInfo {
    fn from(msg: Message) -> Self {
        Self {
            id: msg.id,
            sender_id: msg.sender_id,
            recipient_id: msg.recipient_id,
            message_type: format!("{:?}", msg.message_type),
            payload: msg.payload,
            timestamp: msg.timestamp,
        }
    }
}

// ============================================================================
// CORE DELEGATE TRAIT
// ============================================================================

/// Callback interface for platform events (mobile push notifications, etc.)
pub trait CoreDelegate: Send + Sync {
    /// A new peer was discovered on the network
    fn on_peer_discovered(&self, peer_id: String);
    /// A peer disconnected
    fn on_peer_disconnected(&self, peer_id: String);
    /// An encrypted message was received and decrypted
    fn on_message_received(&self, sender_id: String, message_id: String, data: Vec<u8>);
    /// A delivery receipt was received
    fn on_receipt_received(&self, message_id: String, status: String);
}

// ============================================================================
// IRON CORE IMPLEMENTATION
// ============================================================================

pub struct IronCore {
    /// Identity and key management
    identity: Arc<RwLock<identity::IdentityManager>>,
    /// Outbound message queue
    outbox: Arc<RwLock<store::Outbox>>,
    /// Inbound message deduplication
    inbox: Arc<RwLock<store::Inbox>>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Platform delegate for callbacks
    delegate: Arc<RwLock<Option<Arc<dyn CoreDelegate>>>>,
}

impl IronCore {
    /// Create a new Iron Core instance with in-memory storage
    pub fn new() -> Self {
        Self::init(None)
    }

    /// Create Iron Core with persistent storage at the given path
    pub fn with_storage(storage_path: String) -> Self {
        Self::init(Some(storage_path))
    }

    fn init(storage_path: Option<String>) -> Self {
        // Initialize tracing (idempotent)
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init();

        let identity = if let Some(path) = storage_path.as_ref() {
            Arc::new(RwLock::new(
                IdentityManager::with_path(path).unwrap_or_else(|e| {
                    tracing::warn!(
                        "Failed to open persistent storage at '{}': {}. Falling back to in-memory storage. \
                         Identity keys will NOT persist across restarts.",
                        path, e
                    );
                    IdentityManager::new()
                }),
            ))
        } else {
            Arc::new(RwLock::new(IdentityManager::new()))
        };

        let (outbox, inbox) = if let Some(path) = storage_path {
            // Open sled database for outbox and inbox
            match sled::open(&path) {
                Ok(db) => {
                    let outbox = store::Outbox::with_storage(db.clone())
                        .unwrap_or_else(|e| {
                            tracing::warn!("Failed to open persistent outbox: {}. Using in-memory outbox.", e);
                            store::Outbox::new()
                        });

                    let inbox = store::Inbox::with_storage(db)
                        .unwrap_or_else(|e| {
                            tracing::warn!("Failed to open persistent inbox: {}. Using in-memory inbox.", e);
                            store::Inbox::new()
                        });

                    (outbox, inbox)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to open sled database at '{}': {}. Using in-memory stores.",
                        path, e
                    );
                    (store::Outbox::new(), store::Inbox::new())
                }
            }
        } else {
            (store::Outbox::new(), store::Inbox::new())
        };

        Self {
            identity,
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            running: Arc::new(RwLock::new(false)),
            delegate: Arc::new(RwLock::new(None)),
        }
    }

    // ------------------------------------------------------------------------
    // LIFECYCLE
    // ------------------------------------------------------------------------

    pub fn start(&self) -> Result<(), IronCoreError> {
        let mut running = self.running.write();
        if *running {
            return Err(IronCoreError::AlreadyRunning);
        }

        tracing::info!("Iron Core V2 starting...");

        // Initialize identity if not already done (single write lock to avoid TOCTOU race)
        {
            let mut identity = self.identity.write();
            if identity.keys().is_none() {
                identity
                    .initialize()
                    .map_err(|e| IronCoreError::StorageError(e.to_string()))?;
            }
        }

        *running = true;
        tracing::info!("Iron Core V2 started");

        Ok(())
    }

    pub fn stop(&self) {
        let mut running = self.running.write();
        if !*running {
            return;
        }

        tracing::info!("Iron Core V2 stopping...");
        *running = false;
        tracing::info!("Iron Core V2 stopped");
    }

    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    // ------------------------------------------------------------------------
    // IDENTITY & CRYPTOGRAPHY
    // ------------------------------------------------------------------------

    /// Initialize identity keys (generate new or load existing)
    pub fn initialize_identity(&self) -> Result<(), IronCoreError> {
        self.identity
            .write()
            .initialize()
            .map_err(|e| IronCoreError::CryptoError(e.to_string()))
    }

    /// Get identity information
    pub fn get_identity_info(&self) -> IdentityInfo {
        let identity = self.identity.read();

        IdentityInfo {
            identity_id: identity.identity_id(),
            public_key_hex: identity.public_key_hex(),
            initialized: identity.keys().is_some(),
        }
    }

    /// Sign data with this node's identity key
    pub fn sign_data(&self, data: Vec<u8>) -> Result<SignatureResult, IronCoreError> {
        let identity = self.identity.read();

        let signature = identity
            .sign(&data)
            .map_err(|e| IronCoreError::CryptoError(e.to_string()))?;

        let public_key_hex = identity
            .public_key_hex()
            .ok_or(IronCoreError::NotInitialized)?;

        Ok(SignatureResult {
            signature,
            public_key_hex,
        })
    }

    /// Verify a signature against a public key
    pub fn verify_signature(
        &self,
        data: Vec<u8>,
        signature: Vec<u8>,
        public_key_hex: String,
    ) -> Result<bool, IronCoreError> {
        let public_key =
            hex::decode(&public_key_hex).map_err(|e| IronCoreError::InvalidInput(e.to_string()))?;

        if public_key.len() != 32 {
            return Err(IronCoreError::InvalidInput(
                "Public key must be 32 bytes".to_string(),
            ));
        }

        self.identity
            .read()
            .verify(&data, &signature, &public_key)
            .map_err(|e| IronCoreError::CryptoError(e.to_string()))
    }

    // ------------------------------------------------------------------------
    // MESSAGING
    // ------------------------------------------------------------------------

    /// Encrypt and prepare a text message for a recipient.
    ///
    /// Returns the serialized envelope bytes ready for transmission.
    pub fn prepare_message(
        &self,
        recipient_public_key_hex: String,
        text: String,
    ) -> Result<Vec<u8>, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        let sender_id = identity
            .identity_id()
            .ok_or(IronCoreError::NotInitialized)?;

        // Decode recipient public key
        let recipient_public_key = hex::decode(&recipient_public_key_hex)
            .map_err(|e| IronCoreError::InvalidInput(e.to_string()))?;
        if recipient_public_key.len() != 32 {
            return Err(IronCoreError::InvalidInput(
                "Recipient public key must be 32 bytes".to_string(),
            ));
        }
        let mut recipient_bytes = [0u8; 32];
        recipient_bytes.copy_from_slice(&recipient_public_key);

        // Create plaintext message
        let msg = Message::text(sender_id, recipient_public_key_hex.clone(), &text);

        // Serialize the message
        let plaintext =
            message::encode_message(&msg).map_err(|e| IronCoreError::Internal(e.to_string()))?;

        // Encrypt
        let envelope = crypto::encrypt_message(&keys.signing_key, &recipient_bytes, &plaintext)
            .map_err(|e| IronCoreError::CryptoError(e.to_string()))?;

        // Serialize envelope for wire
        let envelope_bytes = message::encode_envelope(&envelope)
            .map_err(|e| IronCoreError::Internal(e.to_string()))?;

        Ok(envelope_bytes)
    }

    /// Decrypt a received envelope and return the plaintext message.
    pub fn receive_message(&self, envelope_bytes: Vec<u8>) -> Result<MessageInfo, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        // Deserialize envelope
        let envelope = message::decode_envelope(&envelope_bytes)
            .map_err(|e| IronCoreError::Internal(e.to_string()))?;

        // Decrypt
        let plaintext = crypto::decrypt_message(&keys.signing_key, &envelope)
            .map_err(|e| IronCoreError::CryptoError(e.to_string()))?;

        // Deserialize message
        let msg = message::decode_message(&plaintext)
            .map_err(|e| IronCoreError::Internal(e.to_string()))?;

        // Reject messages with stale or future timestamps (5-minute window)
        const MESSAGE_TIMESTAMP_WINDOW_SECS: u64 = 300;
        if !msg.is_recent(MESSAGE_TIMESTAMP_WINDOW_SECS) {
            return Err(IronCoreError::InvalidInput(
                "Message timestamp outside acceptable window (±5 minutes)".to_string(),
            ));
        }

        // Dedup check
        let mut inbox = self.inbox.write();
        let is_new = inbox.receive(store::ReceivedMessage {
            message_id: msg.id.clone(),
            sender_id: msg.sender_id.clone(),
            payload: msg.payload.clone(),
            received_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        if !is_new {
            return Err(IronCoreError::InvalidInput("Duplicate message".to_string()));
        }

        Ok(msg.into())
    }

    /// Get the number of queued outbound messages
    pub fn outbox_count(&self) -> u32 {
        self.outbox.read().total_count() as u32
    }

    /// Get the number of received messages
    pub fn inbox_count(&self) -> u32 {
        self.inbox.read().total_count() as u32
    }

    // ------------------------------------------------------------------------
    // DELEGATE
    // ------------------------------------------------------------------------

    pub fn set_delegate(&self, delegate: Option<Box<dyn CoreDelegate>>) {
        *self.delegate.write() = delegate.map(|d| Arc::from(d) as Arc<dyn CoreDelegate>);
    }
}

impl Default for IronCore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iron_core_creation() {
        let core = IronCore::new();
        assert!(!core.is_running());
    }

    #[test]
    fn test_lifecycle() {
        let core = IronCore::new();

        assert!(!core.is_running());

        core.start().unwrap();
        assert!(core.is_running());

        // Double-start should fail
        assert!(core.start().is_err());

        core.stop();
        assert!(!core.is_running());
    }

    #[test]
    fn test_identity_initialization() {
        let core = IronCore::new();

        let info_before = core.get_identity_info();
        assert!(!info_before.initialized);

        core.initialize_identity().unwrap();

        let info_after = core.get_identity_info();
        assert!(info_after.initialized);
        assert!(info_after.identity_id.is_some());
        assert!(info_after.public_key_hex.is_some());

        // Public key should be 64 hex chars (32 bytes)
        assert_eq!(info_after.public_key_hex.unwrap().len(), 64);
    }

    #[test]
    fn test_signing_and_verification() {
        let core = IronCore::new();
        core.initialize_identity().unwrap();

        let data = b"test message".to_vec();
        let sig_result = core.sign_data(data.clone()).unwrap();

        assert!(!sig_result.signature.is_empty());
        assert_eq!(sig_result.signature.len(), 64); // Ed25519

        let valid = core
            .verify_signature(
                data.clone(),
                sig_result.signature.clone(),
                sig_result.public_key_hex.clone(),
            )
            .unwrap();
        assert!(valid);

        // Wrong data should fail verification
        let invalid = core
            .verify_signature(
                b"wrong data".to_vec(),
                sig_result.signature,
                sig_result.public_key_hex,
            )
            .unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_invalid_public_key_length() {
        let core = IronCore::new();
        core.initialize_identity().unwrap();

        let result =
            core.verify_signature(b"data".to_vec(), vec![0u8; 64], hex::encode(vec![0u8; 16]));
        assert!(result.is_err());
    }

    #[test]
    fn test_end_to_end_messaging() {
        let alice = IronCore::new();
        let bob = IronCore::new();

        alice.initialize_identity().unwrap();
        bob.initialize_identity().unwrap();

        let bob_info = bob.get_identity_info();
        let bob_public_key = bob_info.public_key_hex.unwrap();

        let envelope_bytes = alice
            .prepare_message(bob_public_key, "Hello Bob!".to_string())
            .unwrap();

        let msg_info = bob.receive_message(envelope_bytes).unwrap();

        assert_eq!(String::from_utf8(msg_info.payload).unwrap(), "Hello Bob!");
        assert_eq!(
            msg_info.sender_id,
            alice.get_identity_info().identity_id.unwrap()
        );
    }

    #[test]
    fn test_wrong_recipient_cannot_decrypt() {
        let alice = IronCore::new();
        let bob = IronCore::new();
        let eve = IronCore::new();

        alice.initialize_identity().unwrap();
        bob.initialize_identity().unwrap();
        eve.initialize_identity().unwrap();

        let bob_public_key = bob.get_identity_info().public_key_hex.unwrap();

        let envelope_bytes = alice
            .prepare_message(bob_public_key, "Secret message".to_string())
            .unwrap();

        let result = eve.receive_message(envelope_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_message_deduplication() {
        let alice = IronCore::new();
        let bob = IronCore::new();

        alice.initialize_identity().unwrap();
        bob.initialize_identity().unwrap();

        let bob_public_key = bob.get_identity_info().public_key_hex.unwrap();

        let envelope_bytes = alice
            .prepare_message(bob_public_key, "test".to_string())
            .unwrap();

        bob.receive_message(envelope_bytes.clone()).unwrap();

        let result = bob.receive_message(envelope_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_inbox_outbox_counts() {
        let core = IronCore::new();
        assert_eq!(core.outbox_count(), 0);
        assert_eq!(core.inbox_count(), 0);
    }

    #[test]
    fn test_auto_initialize_on_start() {
        let core = IronCore::new();
        core.start().unwrap();

        let info = core.get_identity_info();
        assert!(info.initialized);

        core.stop();
    }
}
