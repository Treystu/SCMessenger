// Iron Core V2 — Messaging Spine
#![allow(clippy::empty_line_after_doc_comments)]
//
// "Does this help two humans exchange an encrypted message
//  without any corporation in the middle?"
//
// If the answer is no, it doesn't belong in Phase 0.

pub mod crypto;
pub mod identity;
pub mod message;
pub mod privacy;
pub mod store;
pub mod transport;

// Mobile bridge modules
pub mod contacts_bridge;
pub mod mobile_bridge;

use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use zeroize::Zeroize;

pub use crypto::{decrypt_message, encrypt_message};
pub use identity::IdentityManager;
pub use message::{DeliveryStatus, Envelope, Message, MessageType, Receipt};

// Mobile bridge exports for UniFFI
pub use contacts_bridge::{Contact, ContactManager};
pub use mobile_bridge::*;

// UniFFI scaffolding - clippy warnings in generated code
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
    #[error("Storage error")]
    StorageError,
    #[error("Cryptography error")]
    CryptoError,
    #[error("Network error")]
    NetworkError,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Internal error")]
    Internal,
}

impl From<anyhow::Error> for IronCoreError {
    fn from(_err: anyhow::Error) -> Self {
        IronCoreError::Internal
    }
}

// ============================================================================
// DATA TYPES
// ============================================================================

/// Identity information for UniFFI / WASM export.
///
/// ## Canonical Identity: `public_key_hex`
///
/// `public_key_hex` is the **canonical persisted and exchanged identity**.
/// It is the hex-encoded Ed25519 public key used for:
/// - Contact exchange (QR codes, import/export payloads)
/// - Message encryption (recipient addressing)
/// - History attribution (sender identification)
/// - Cross-platform identity resolution
///
/// Other identifiers are **derived/operational metadata**:
/// - `identity_id`: Blake3 hash of the public key (legacy display ID, 64 hex chars)
/// - `libp2p_peer_id`: libp2p PeerId derived from the Ed25519 keypair (transport-layer routing)
///
/// When persisting contacts or exchanging identity information, always use
/// `public_key_hex` as the primary key. The `identity_id` and `libp2p_peer_id`
/// can be derived from it and should not be used as standalone identifiers.
#[derive(Clone)]
pub struct IdentityInfo {
    /// Blake3 hash of the public key — derived/display identifier (64 hex chars).
    /// **Not canonical.** Use `public_key_hex` for persistence and exchange.
    pub identity_id: Option<String>,
    /// Hex-encoded Ed25519 public key — **CANONICAL identity** for all platforms.
    pub public_key_hex: Option<String>,
    pub initialized: bool,
    pub nickname: Option<String>,
    /// libp2p PeerId — transport-layer routing identifier. Derived from the Ed25519 keypair.
    /// **Not canonical.** Use `public_key_hex` for persistence and exchange.
    pub libp2p_peer_id: Option<String>,
}

/// Signature result for UniFFI export
#[derive(Clone)]
pub struct SignatureResult {
    pub signature: Vec<u8>,
    pub public_key_hex: String,
}

/// Prepared outbound message metadata for UniFFI export.
///
/// `message_id` must be persisted by mobile clients so delivery receipts can
/// be correlated exactly to the outbound history record.
#[derive(Clone)]
pub struct PreparedMessage {
    pub message_id: String,
    pub envelope_data: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct IdentityBackupV1 {
    version: u8,
    secret_key_hex: String,
    nickname: Option<String>,
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
    /// A peer's libp2p identity was confirmed
    fn on_peer_identified(&self, peer_id: String, listen_addrs: Vec<String>);
    /// An encrypted message was received and decrypted.
    /// `sender_public_key_hex` is the sender's Ed25519 public key (64 hex chars) —
    /// pass this to `prepare_receipt()` to send a delivery acknowledgement.
    // `sender_id` is the sender's Blake3 identity_id (64 hex chars) — use for display.
    fn on_message_received(
        &self,
        sender_id: String,
        sender_public_key_hex: String,
        message_id: String,
        sender_timestamp: u64,
        data: Vec<u8>,
    );
    /// A delivery receipt was received
    fn on_receipt_received(&self, message_id: String, status: String);
}

// ============================================================================
// IRON CORE IMPLEMENTATION
// ============================================================================

#[derive(Clone)]
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

const STORAGE_SCHEMA_VERSION: u32 = 2;

const LEGACY_IDENTITY_KEY: &[u8] = b"identity_keys";
const LEGACY_NICKNAME_KEY: &[u8] = b"identity_nickname";
const LEGACY_OUTBOX_PREFIX: &[u8] = b"outbox_";
const LEGACY_INBOX_SEEN_KEY: &[u8] = b"inbox_seen_ids";
const LEGACY_INBOX_MSG_PREFIX: &[u8] = b"inbox_msg_";
const LEGACY_ROOT_MIGRATION_SENTINEL: &str = "LEGACY_ROOT_SLED_MIGRATED";

fn read_schema_version(version_file: &Path) -> Result<u32, IronCoreError> {
    if !version_file.exists() {
        return Ok(0);
    }
    let current = std::fs::read_to_string(version_file).map_err(|_| IronCoreError::StorageError)?;
    current
        .trim()
        .parse::<u32>()
        .map_err(|_| IronCoreError::StorageError)
}

fn has_legacy_root_sled(base: &Path) -> bool {
    // Sled stores these files at the DB root. If present, old single-db layout
    // may still hold identity/outbox/inbox keys.
    base.join("conf").exists() || base.join("db").exists()
}

fn copy_missing_key(
    source: &sled::Db,
    destination: &sled::Db,
    key: &[u8],
) -> Result<bool, IronCoreError> {
    if destination
        .get(key)
        .map_err(|_| IronCoreError::StorageError)?
        .is_some()
    {
        return Ok(false);
    }
    if let Some(value) = source.get(key).map_err(|_| IronCoreError::StorageError)? {
        destination
            .insert(key, value)
            .map_err(|_| IronCoreError::StorageError)?;
        return Ok(true);
    }
    Ok(false)
}

fn copy_missing_prefix(
    source: &sled::Db,
    destination: &sled::Db,
    prefix: &[u8],
) -> Result<usize, IronCoreError> {
    let mut copied = 0usize;
    for item in source.scan_prefix(prefix) {
        let (key, value) = item.map_err(|_| IronCoreError::StorageError)?;
        if destination
            .get(&key)
            .map_err(|_| IronCoreError::StorageError)?
            .is_none()
        {
            destination
                .insert(key, value)
                .map_err(|_| IronCoreError::StorageError)?;
            copied += 1;
        }
    }
    Ok(copied)
}

fn migrate_legacy_root_store(base: &Path) -> Result<(), IronCoreError> {
    let sentinel = base.join(LEGACY_ROOT_MIGRATION_SENTINEL);
    if sentinel.exists() || !has_legacy_root_sled(base) {
        return Ok(());
    }

    let legacy = sled::open(base).map_err(|_| IronCoreError::StorageError)?;
    let identity_db = sled::open(base.join("identity")).map_err(|_| IronCoreError::StorageError)?;
    let outbox_db = sled::open(base.join("outbox")).map_err(|_| IronCoreError::StorageError)?;
    let inbox_db = sled::open(base.join("inbox")).map_err(|_| IronCoreError::StorageError)?;

    let mut copied_keys = 0usize;
    copied_keys += usize::from(copy_missing_key(
        &legacy,
        &identity_db,
        LEGACY_IDENTITY_KEY,
    )?);
    copied_keys += usize::from(copy_missing_key(
        &legacy,
        &identity_db,
        LEGACY_NICKNAME_KEY,
    )?);
    copied_keys += copy_missing_prefix(&legacy, &outbox_db, LEGACY_OUTBOX_PREFIX)?;
    copied_keys += usize::from(copy_missing_key(&legacy, &inbox_db, LEGACY_INBOX_SEEN_KEY)?);
    copied_keys += copy_missing_prefix(&legacy, &inbox_db, LEGACY_INBOX_MSG_PREFIX)?;

    identity_db
        .flush()
        .map_err(|_| IronCoreError::StorageError)?;
    outbox_db.flush().map_err(|_| IronCoreError::StorageError)?;
    inbox_db.flush().map_err(|_| IronCoreError::StorageError)?;

    std::fs::write(&sentinel, format!("migrated_keys={copied_keys}\n"))
        .map_err(|_| IronCoreError::StorageError)?;
    tracing::info!(
        "Legacy root sled migration completed (copied {} key(s))",
        copied_keys
    );
    Ok(())
}

fn ensure_storage_layout(storage_path: &str) -> Result<(), IronCoreError> {
    let base = Path::new(storage_path);
    std::fs::create_dir_all(base).map_err(|_| IronCoreError::StorageError)?;
    std::fs::create_dir_all(base.join("identity")).map_err(|_| IronCoreError::StorageError)?;
    std::fs::create_dir_all(base.join("outbox")).map_err(|_| IronCoreError::StorageError)?;
    std::fs::create_dir_all(base.join("inbox")).map_err(|_| IronCoreError::StorageError)?;

    let version_file = base.join("SCHEMA_VERSION");
    let current = read_schema_version(&version_file)?;
    if current > STORAGE_SCHEMA_VERSION {
        return Err(IronCoreError::StorageError);
    }

    if current < 2 {
        migrate_legacy_root_store(base)?;
    }

    if current != STORAGE_SCHEMA_VERSION {
        std::fs::write(&version_file, STORAGE_SCHEMA_VERSION.to_string())
            .map_err(|_| IronCoreError::StorageError)?;
    }
    Ok(())
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

        let storage_ready = if let Some(path) = &storage_path {
            match ensure_storage_layout(path) {
                Ok(()) => true,
                Err(e) => {
                    tracing::error!("Storage layout check failed at {}: {:?}", path, e);
                    false
                }
            }
        } else {
            false
        };

        let identity = if let Some(path) = &storage_path {
            if !storage_ready {
                Arc::new(RwLock::new(IdentityManager::new()))
            } else {
                let identity_path = Path::new(path).join("identity");
                Arc::new(RwLock::new(
                    IdentityManager::with_path(identity_path.to_string_lossy().as_ref())
                        .unwrap_or_else(|_| IdentityManager::new()),
                ))
            }
        } else {
            Arc::new(RwLock::new(IdentityManager::new()))
        };

        let outbox = if let Some(path) = &storage_path {
            if !storage_ready {
                store::Outbox::new()
            } else {
                let outbox_path = Path::new(path).join("outbox");
                store::Outbox::persistent(outbox_path.to_string_lossy().as_ref())
                    .unwrap_or_else(|_| store::Outbox::new())
            }
        } else {
            store::Outbox::new()
        };

        let inbox = if let Some(path) = &storage_path {
            if !storage_ready {
                store::Inbox::new()
            } else {
                let inbox_path = Path::new(path).join("inbox");
                store::Inbox::persistent(inbox_path.to_string_lossy().as_ref())
                    .unwrap_or_else(|_| store::Inbox::new())
            }
        } else {
            store::Inbox::new()
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

        // Identity lifecycle is explicit. Do not auto-generate keys on service start.
        // This preserves clean-wipe onboarding semantics and avoids silent identity drift.

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
            .map_err(|_| IronCoreError::CryptoError)
    }

    /// Get identity information
    pub fn get_identity_info(&self) -> IdentityInfo {
        let identity = self.identity.read();

        let libp2p_peer_id = identity.keys().and_then(|keys| {
            keys.to_libp2p_keypair()
                .ok()
                .map(|kp| kp.public().to_peer_id().to_string())
        });

        IdentityInfo {
            identity_id: identity.identity_id(),
            public_key_hex: identity.public_key_hex(),
            initialized: identity.keys().is_some(),
            nickname: identity.nickname(),
            libp2p_peer_id,
        }
    }

    /// Internal helper to get cloned identity keys
    pub fn get_identity_keys(&self) -> Option<identity::IdentityKeys> {
        self.identity.read().keys().cloned()
    }

    /// Set the user's nickname
    pub fn set_nickname(&self, nickname: String) -> Result<(), IronCoreError> {
        self.identity
            .write()
            .set_nickname(nickname)
            .map_err(|_| IronCoreError::StorageError)
    }

    /// Export identity key material for platform-secure backup.
    pub fn export_identity_backup(&self) -> Result<String, IronCoreError> {
        let identity = self.identity.read();
        let mut key_bytes = identity
            .export_key_bytes()
            .ok_or(IronCoreError::NotInitialized)?;
        let payload = IdentityBackupV1 {
            version: 1,
            secret_key_hex: hex::encode(&key_bytes),
            nickname: identity.nickname(),
        };
        key_bytes.zeroize();
        serde_json::to_string(&payload).map_err(|_| IronCoreError::Internal)
    }

    /// Import identity key material from a platform-secure backup payload.
    pub fn import_identity_backup(&self, backup: String) -> Result<(), IronCoreError> {
        let payload: IdentityBackupV1 =
            serde_json::from_str(&backup).map_err(|_| IronCoreError::InvalidInput)?;
        if payload.version != 1 {
            return Err(IronCoreError::InvalidInput);
        }
        let mut key_bytes =
            hex::decode(payload.secret_key_hex).map_err(|_| IronCoreError::InvalidInput)?;
        let mut identity = self.identity.write();
        let result = identity
            .import_key_bytes(&key_bytes)
            .map_err(|_| IronCoreError::StorageError);
        key_bytes.zeroize();
        result?;
        if let Some(nickname) = payload.nickname {
            identity
                .set_nickname(nickname)
                .map_err(|_| IronCoreError::StorageError)?;
        }
        Ok(())
    }

    /// Sign data with this node's identity key
    pub fn sign_data(&self, data: Vec<u8>) -> Result<SignatureResult, IronCoreError> {
        let identity = self.identity.read();

        let signature = identity
            .sign(&data)
            .map_err(|_| IronCoreError::CryptoError)?;

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
        let public_key = hex::decode(&public_key_hex).map_err(|_| IronCoreError::InvalidInput)?;

        if public_key.len() != 32 {
            return Err(IronCoreError::InvalidInput);
        }

        self.identity
            .read()
            .verify(&data, &signature, &public_key)
            .map_err(|_| IronCoreError::CryptoError)
    }

    /// Get libp2p keypair derived from identity keys
    ///
    /// This uses the same underlying Ed25519 keypair as the node's identity
    /// to derive the libp2p keypair (and thus its PeerId), so the cryptographic
    /// identity and the network identity are backed by the same keys. Note that
    /// the libp2p PeerId value is distinct from the node's `identity_id` (Blake3 hash).
    pub fn get_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        keys.to_libp2p_keypair()
            .map_err(|_| IronCoreError::CryptoError)
    }

    /// Extract Ed25519 public key hex from a libp2p PeerID string
    pub fn extract_public_key_from_peer_id(
        &self,
        peer_id: String,
    ) -> Result<String, IronCoreError> {
        let bytes = bs58::decode(&peer_id)
            .into_vec()
            .map_err(|_| IronCoreError::InvalidInput)?;
        // libp2p Ed25519 PeerId: 0x00 0x24 0x08 0x01 0x12 0x20 <32 bytes>
        // Total = 38 bytes. Verify the protobuf prefix.
        if bytes.len() == 38
            && bytes[0] == 0x00  // identity multihash
            && bytes[1] == 0x24  // length 36
            && bytes[2] == 0x08  // protobuf field 1 (key type)
            && bytes[3] == 0x01  // Ed25519
            && bytes[4] == 0x12  // protobuf field 2 (key data)
            && bytes[5] == 0x20
        // 32 bytes
        {
            Ok(hex::encode(&bytes[6..38]))
        } else if bytes.len() >= 32 {
            // Fallback for non-standard PeerIds: take last 32 bytes
            Ok(hex::encode(&bytes[bytes.len() - 32..]))
        } else {
            Err(IronCoreError::InvalidInput)
        }
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
        self.prepare_message_with_id(recipient_public_key_hex, text)
            .map(|prepared| prepared.envelope_data)
    }

    /// Encrypt and prepare a text message, returning both message ID and envelope.
    ///
    /// Mobile clients should use this API for robust receipt correlation.
    pub fn prepare_message_with_id(
        &self,
        recipient_public_key_hex: String,
        text: String,
    ) -> Result<PreparedMessage, IronCoreError> {
        // Trim whitespace from the key (defensive, mobile apps may include it)
        let recipient_key_trimmed = recipient_public_key_hex.trim().to_string();

        // Validate the recipient public key at the core boundary
        if let Err(e) = crate::crypto::validate_ed25519_public_key(&recipient_key_trimmed) {
            eprintln!(
                "[IronCore] prepare_message: public key validation failed — key_len={} hex_chars={} error={}",
                recipient_key_trimmed.len(),
                recipient_key_trimmed.chars().take(16).collect::<String>(),
                e
            );
            return Err(IronCoreError::InvalidInput);
        }

        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        let sender_id = identity
            .identity_id()
            .ok_or(IronCoreError::NotInitialized)?;

        // Decode recipient public key
        let recipient_public_key = hex::decode(&recipient_key_trimmed).map_err(|e| {
            eprintln!("[IronCore] prepare_message: hex decode failed — {}", e);
            IronCoreError::InvalidInput
        })?;
        if recipient_public_key.len() != 32 {
            eprintln!(
                "[IronCore] prepare_message: decoded key wrong length — got {} bytes, expected 32",
                recipient_public_key.len()
            );
            return Err(IronCoreError::InvalidInput);
        }
        let mut recipient_bytes = [0u8; 32];
        recipient_bytes.copy_from_slice(&recipient_public_key);

        // Create plaintext message
        let msg = Message::text(sender_id, recipient_key_trimmed.clone(), &text);
        let message_id = msg.id.clone();

        // Serialize the message
        let plaintext = message::encode_message(&msg).map_err(|_| IronCoreError::Internal)?;

        // Encrypt
        let envelope = crypto::encrypt_message(&keys.signing_key, &recipient_bytes, &plaintext)
            .map_err(|_| IronCoreError::CryptoError)?;

        // Serialize envelope for wire
        let envelope_bytes =
            message::encode_envelope(&envelope).map_err(|_| IronCoreError::Internal)?;

        // Persist outbound envelope for retry/reconciliation.
        self.outbox
            .write()
            .enqueue(store::QueuedMessage {
                message_id: message_id.clone(),
                recipient_id: recipient_key_trimmed.clone(),
                envelope_data: envelope_bytes.clone(),
                queued_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                attempts: 0,
            })
            .map_err(|_| IronCoreError::StorageError)?;

        Ok(PreparedMessage {
            message_id,
            envelope_data: envelope_bytes,
        })
    }

    /// Encrypt and prepare a delivery receipt for the original sender.
    ///
    /// `recipient_public_key_hex` — the sender's Ed25519 public key (hex, 64 chars).
    /// `message_id` — the ID of the message being acknowledged.
    ///
    /// Returns encrypted envelope bytes ready for transmission, identical wire
    /// format to `prepare_message`. The recipient can distinguish receipts from
    /// text messages via `Message::message_type`.
    pub fn prepare_receipt(
        &self,
        recipient_public_key_hex: String,
        message_id: String,
    ) -> Result<Vec<u8>, IronCoreError> {
        // Trim whitespace (defensive, matches prepare_message)
        let recipient_key_trimmed = recipient_public_key_hex.trim().to_string();

        if let Err(e) = crate::crypto::validate_ed25519_public_key(&recipient_key_trimmed) {
            eprintln!(
                "[IronCore] prepare_receipt: public key validation failed — key_len={} error={}",
                recipient_key_trimmed.len(),
                e
            );
            return Err(IronCoreError::InvalidInput);
        }

        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        let sender_id = identity
            .identity_id()
            .ok_or(IronCoreError::NotInitialized)?;

        let recipient_public_key =
            hex::decode(&recipient_key_trimmed).map_err(|_| IronCoreError::InvalidInput)?;
        if recipient_public_key.len() != 32 {
            return Err(IronCoreError::InvalidInput);
        }
        let mut recipient_bytes = [0u8; 32];
        recipient_bytes.copy_from_slice(&recipient_public_key);

        let receipt = Receipt::delivered(message_id);
        let msg = Message::receipt(sender_id, recipient_key_trimmed, &receipt);

        let plaintext = message::encode_message(&msg).map_err(|_| IronCoreError::Internal)?;

        let envelope = crypto::encrypt_message(&keys.signing_key, &recipient_bytes, &plaintext)
            .map_err(|_| IronCoreError::CryptoError)?;

        let envelope_bytes =
            message::encode_envelope(&envelope).map_err(|_| IronCoreError::Internal)?;

        Ok(envelope_bytes)
    }

    /// Generate a cover traffic payload — random bytes that look like an encrypted
    /// message. Broadcast via send_to_all_peers() to obscure real traffic patterns.
    /// `size_bytes` controls payload size (16–1024); values outside range are clamped.
    pub fn prepare_cover_traffic(&self, size_bytes: u32) -> Result<Vec<u8>, IronCoreError> {
        use crate::privacy::cover::{CoverConfig, CoverTrafficGenerator};
        let size = (size_bytes as usize).clamp(16, 1024);
        let config = CoverConfig {
            enabled: true,
            message_size: size,
            rate_per_minute: 1,
        };
        let gen = CoverTrafficGenerator::new(config).map_err(|_| IronCoreError::Internal)?;
        let cover = gen
            .generate_cover_message()
            .map_err(|_| IronCoreError::Internal)?;
        // Encode as flat Vec<u8>: ephemeral_key (32) | recipient_hint (32) | encrypted_payload
        let mut out = Vec::with_capacity(32 + 32 + cover.encrypted_payload.len());
        out.extend_from_slice(&cover.ephemeral_key);
        out.extend_from_slice(&cover.recipient_hint);
        out.extend_from_slice(&cover.encrypted_payload);
        Ok(out)
    }

    /// Decrypt a received envelope and return the plaintext message.
    pub fn receive_message(&self, envelope_bytes: Vec<u8>) -> Result<Message, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        // Deserialize envelope
        let envelope =
            message::decode_envelope(&envelope_bytes).map_err(|_| IronCoreError::Internal)?;

        // Decrypt
        let plaintext = crypto::decrypt_message(&keys.signing_key, &envelope)
            .map_err(|_| IronCoreError::CryptoError)?;

        // Deserialize message
        let msg = message::decode_message(&plaintext).map_err(|_| IronCoreError::Internal)?;

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
            // Duplicate message IDs are expected under at-least-once delivery.
            // Re-dispatch callbacks so receivers can re-send receipts if needed.
            if let Some(delegate) = self.delegate.read().as_ref() {
                if msg.message_type == message::MessageType::Receipt {
                    if let Ok(receipt) = bincode::deserialize::<message::Receipt>(&msg.payload) {
                        let status_str = match receipt.status {
                            message::DeliveryStatus::Sent => "sent",
                            message::DeliveryStatus::Delivered => "delivered",
                            message::DeliveryStatus::Read => "read",
                            message::DeliveryStatus::Failed(_) => "failed",
                        };
                        delegate.on_receipt_received(receipt.message_id, status_str.to_string());
                    }
                } else {
                    let sender_pub_key_hex = hex::encode(&envelope.sender_public_key);
                    delegate.on_message_received(
                        msg.sender_id.clone(),
                        sender_pub_key_hex,
                        msg.id.clone(),
                        msg.timestamp,
                        msg.payload.clone(),
                    );
                }
            }
            return Ok(msg);
        }

        // Notify delegate — include sender's Ed25519 public key hex so mobile
        // platforms can send a receipt ACK without a contact-DB lookup.
        if let Some(delegate) = self.delegate.read().as_ref() {
            if msg.message_type == message::MessageType::Receipt {
                // If it's a receipt, deserialize the payload to get the true message ID it acknowledges
                if let Ok(receipt) = bincode::deserialize::<message::Receipt>(&msg.payload) {
                    let status_str = match receipt.status {
                        message::DeliveryStatus::Sent => "sent",
                        message::DeliveryStatus::Delivered => "delivered",
                        message::DeliveryStatus::Read => "read",
                        message::DeliveryStatus::Failed(_) => "failed",
                    };
                    delegate.on_receipt_received(receipt.message_id, status_str.to_string());
                } else {
                    tracing::warn!("Failed to deserialize receipt payload");
                }
            } else {
                let sender_pub_key_hex = hex::encode(&envelope.sender_public_key);
                delegate.on_message_received(
                    msg.sender_id.clone(),
                    sender_pub_key_hex,
                    msg.id.clone(),
                    msg.timestamp,
                    msg.payload.clone(),
                );
            }
        }

        Ok(msg)
    }

    /// Remove a message from the outbox after confirmed delivery.
    ///
    /// Returns `true` if the message was found and removed, `false` if it was
    /// not in the outbox (already removed or never queued).
    pub fn mark_message_sent(&self, message_id: String) -> bool {
        self.outbox.write().remove(&message_id)
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

    /// Notify the delegate that a peer was discovered on the network.
    pub fn notify_peer_discovered(&self, peer_id: String) {
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_peer_discovered(peer_id);
        }
    }

    /// Notify the delegate that a peer disconnected from the network.
    pub fn notify_peer_disconnected(&self, peer_id: String) {
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_peer_disconnected(peer_id);
        }
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
    use tempfile::tempdir;

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

        let msg = bob.receive_message(envelope_bytes).unwrap();

        assert_eq!(msg.text_content().unwrap(), "Hello Bob!");
        assert_eq!(
            msg.sender_id,
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
        assert!(result.is_ok());
    }

    #[test]
    fn test_inbox_outbox_counts() {
        let core = IronCore::new();
        assert_eq!(core.outbox_count(), 0);
        assert_eq!(core.inbox_count(), 0);
    }

    #[test]
    fn test_start_does_not_auto_initialize_identity() {
        let core = IronCore::new();
        core.start().unwrap();

        let info = core.get_identity_info();
        assert!(!info.initialized);

        core.stop();
    }

    #[test]
    fn test_with_storage_hydrates_existing_identity_without_initialize_call() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        let first = IronCore::with_storage(path.clone());
        first.initialize_identity().unwrap();
        first.set_nickname("persisted-hydrate".to_string()).unwrap();
        let original_identity = first.get_identity_info().identity_id;
        drop(first);

        let second = IronCore::with_storage(path);
        let reloaded = second.get_identity_info();
        assert!(reloaded.initialized);
        assert_eq!(reloaded.nickname.as_deref(), Some("persisted-hydrate"));
        assert_eq!(reloaded.identity_id, original_identity);
    }

    #[test]
    fn test_with_storage_migrates_legacy_root_identity_layout() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        // Simulate pre-schema-split storage where identity keys lived in root sled.
        let legacy_store = identity::IdentityStore::persistent(&path).unwrap();
        let legacy_keys = identity::IdentityKeys::generate();
        legacy_store.save_keys(&legacy_keys).unwrap();
        legacy_store.save_nickname("legacy-nick").unwrap();
        drop(legacy_store);

        let core = IronCore::with_storage(path.clone());
        let info = core.get_identity_info();
        assert!(info.initialized);
        assert_eq!(
            info.public_key_hex.as_deref(),
            Some(legacy_keys.public_key_hex().as_str())
        );
        assert_eq!(info.nickname.as_deref(), Some("legacy-nick"));

        let schema =
            std::fs::read_to_string(std::path::Path::new(&path).join("SCHEMA_VERSION")).unwrap();
        assert_eq!(schema.trim(), STORAGE_SCHEMA_VERSION.to_string());
        assert!(std::path::Path::new(&path)
            .join(LEGACY_ROOT_MIGRATION_SENTINEL)
            .exists());
    }

    #[test]
    fn test_extract_public_key_from_peer_id() {
        let core = IronCore::new();
        core.initialize_identity().unwrap();
        let info = core.get_identity_info();
        let libp2p_peer_id = info.libp2p_peer_id.unwrap();
        let extracted_hex = core
            .extract_public_key_from_peer_id(libp2p_peer_id)
            .unwrap();
        let original_hex = info.public_key_hex.unwrap();
        assert_eq!(
            extracted_hex, original_hex,
            "Extracted public key must match the identity's own public key"
        );
    }

    #[test]
    fn test_outbox_persists_across_restart_with_storage() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        let alice = IronCore::with_storage(path.clone());
        let bob = IronCore::new();
        alice.initialize_identity().unwrap();
        bob.initialize_identity().unwrap();

        let bob_pk = bob.get_identity_info().public_key_hex.unwrap();
        let _ = alice
            .prepare_message(bob_pk, "persist me".to_string())
            .unwrap();
        assert_eq!(alice.outbox_count(), 1);
        drop(alice);

        let reloaded = IronCore::with_storage(path);
        assert_eq!(reloaded.outbox_count(), 1);
    }

    #[test]
    fn test_inbox_persists_across_restart_with_storage() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        let alice = IronCore::with_storage(path.clone());
        let bob = IronCore::new();
        alice.initialize_identity().unwrap();
        bob.initialize_identity().unwrap();

        let alice_pk = alice.get_identity_info().public_key_hex.unwrap();
        let envelope = bob
            .prepare_message(alice_pk, "hi from bob".to_string())
            .unwrap();
        alice.receive_message(envelope).unwrap();
        assert_eq!(alice.inbox_count(), 1);
        drop(alice);

        let reloaded = IronCore::with_storage(path);
        assert_eq!(reloaded.inbox_count(), 1);
    }
}
