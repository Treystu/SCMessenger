// Iron Core V2 — Messaging Spine
#![allow(clippy::empty_line_after_doc_comments)]
//
// "Does this help two humans exchange an encrypted message
//  without any corporation in the middle?"
//
// If the answer is no, it doesn't belong in Phase 0.

pub mod crypto;
pub mod drift;
pub mod identity;
pub mod message;
pub mod notification;
pub mod observability;
pub mod notification_defaults;
pub mod privacy;
pub mod routing;
pub mod store;
pub mod transport;
pub mod wasm_support;

// Relay module requires quinn which is not available on WASM
#[cfg(not(target_arch = "wasm32"))]
pub mod relay;

// Mobile bridge modules
#[cfg(not(target_arch = "wasm32"))]
pub mod blocked_bridge;
#[cfg(not(target_arch = "wasm32"))]
pub mod contacts_bridge;
#[cfg(not(target_arch = "wasm32"))]
pub mod mobile_bridge;

use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use zeroize::Zeroize;

use observability::{AuditEventType, AuditLog as AuditLogType};

pub use crypto::{decrypt_message, encrypt_message};
pub use identity::IdentityManager;
pub use message::{DeliveryStatus, Envelope, Message, MessageType, Receipt};
pub use notification::{
    classify_notification as classify_notification_policy, NotificationDecision,
    NotificationEndpoint, NotificationEndpointCapabilities, NotificationEndpointError,
    NotificationEndpointRegistry, NotificationKind, NotificationMessageContext,
    NotificationPlatform, NotificationUiState,
};

// Mobile bridge exports for UniFFI
#[cfg(not(target_arch = "wasm32"))]
pub use blocked_bridge::{
    blocked_identity_new, blocked_identity_with_device_id, blocked_identity_with_notes,
    blocked_identity_with_reason, BlockedIdentity, BlockedManager,
};
#[cfg(not(target_arch = "wasm32"))]
pub use contacts_bridge::{Contact, ContactManager};
#[cfg(not(target_arch = "wasm32"))]
pub use mobile_bridge::*;

// UniFFI scaffolding - clippy warnings in generated code
#[cfg(not(target_arch = "wasm32"))]
uniffi::include_scaffolding!("api");

// ============================================================================
// WASM-SPECIFIC TYPES (not available on mobile/native)
// ============================================================================

/// Mesh settings for WASM targets
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct MeshSettings {
    pub relay_enabled: bool,
    pub max_relay_budget: u32,
    pub battery_floor: u8,
    pub ble_enabled: bool,
    pub wifi_aware_enabled: bool,
    pub wifi_direct_enabled: bool,
    pub internet_enabled: bool,
    pub discovery_mode: DiscoveryMode,
    pub onion_routing: bool,
    pub cover_traffic_enabled: bool,
    pub message_padding_enabled: bool,
    pub timing_obfuscation_enabled: bool,
    pub notifications_enabled: bool,
    pub notify_dm_enabled: bool,
    pub notify_dm_request_enabled: bool,
    pub notify_dm_in_foreground: bool,
    pub notify_dm_request_in_foreground: bool,
    pub sound_enabled: bool,
    pub badge_enabled: bool,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiscoveryMode {
    Normal,
    Cautious,
    Paranoid,
}

#[cfg(target_arch = "wasm32")]
impl Default for DiscoveryMode {
    fn default() -> Self {
        DiscoveryMode::Normal
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for MeshSettings {
    fn default() -> Self {
        Self {
            relay_enabled: true,
            max_relay_budget: 200,
            battery_floor: 20,
            ble_enabled: true,
            wifi_aware_enabled: true,
            wifi_direct_enabled: true,
            internet_enabled: true,
            discovery_mode: DiscoveryMode::Normal,
            onion_routing: false,
            cover_traffic_enabled: false,
            message_padding_enabled: false,
            timing_obfuscation_enabled: false,
            notifications_enabled: true,
            notify_dm_enabled: true,
            notify_dm_request_enabled: true,
            notify_dm_in_foreground: false,
            notify_dm_request_in_foreground: true,
            sound_enabled: true,
            badge_enabled: true,
        }
    }
}

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
    #[error("Peer is blocked")]
    Blocked,
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
    /// Installation-local UUIDv4 used by WS13 tight-pair routing.
    pub device_id: Option<String>,
    /// Activation timestamp (unix seconds) for this installation instance.
    pub seniority_timestamp: Option<u64>,
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

#[derive(Clone)]
pub struct RegistrationStateInfo {
    pub state: String,
    pub device_id: Option<String>,
    pub seniority_timestamp: Option<u64>,
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
    fn on_peer_identified(&self, peer_id: String, agent_version: String, listen_addrs: Vec<String>);
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
    /// Global contact manager
    contacts: Arc<RwLock<store::ContactManager>>,
    /// Unified message history
    history: Arc<RwLock<store::HistoryManager>>,
    /// Storage management (retention/pruning)
    storage_manager: Arc<store::storage::StorageManager>,
    /// Log summarization/management
    log_manager: Arc<store::logs::LogManager>,
    /// Persistent blocked identity manager
    blocked_manager: Arc<store::blocked::BlockedManager>,
    /// Relay registration registry backed by the canonical root store
    relay_registry: Arc<store::RelayRegistry>,
    /// Tamper-evident audit log for security-critical operations
    audit_log: Arc<RwLock<AuditLogType>>,
    /// UniFFI-facing contacts manager (non-wasm builds only)
    #[cfg(not(target_arch = "wasm32"))]
    contacts_bridge_manager: Arc<crate::contacts_bridge::ContactManager>,
    /// UniFFI-facing history manager (non-wasm builds only)
    #[cfg(not(target_arch = "wasm32"))]
    history_bridge_manager: Arc<crate::mobile_bridge::HistoryManager>,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Platform delegate for callbacks
    delegate: Arc<RwLock<Option<Arc<dyn CoreDelegate>>>>,
}

const STORAGE_SCHEMA_VERSION: u32 = 3;

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

#[cfg(not(target_arch = "wasm32"))]
fn has_legacy_root_sled(base: &Path) -> bool {
    // Sled stores these files at the DB root. If present, old single-db layout
    // may still hold identity/outbox/inbox keys.
    base.join("conf").exists() || base.join("db").exists()
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
fn migrate_legacy_root_store(base: &Path) -> Result<(), IronCoreError> {
    let sentinel = base.join(LEGACY_ROOT_MIGRATION_SENTINEL);
    if sentinel.exists() || !has_legacy_root_sled(base) {
        return Ok(());
    }

    let open_db = |p: &Path| -> Result<sled::Db, IronCoreError> {
        sled::Config::default()
            .path(p)
            .mode(sled::Mode::LowSpace)
            .use_compression(false)
            .open()
            .map_err(|_| IronCoreError::StorageError)
    };

    let legacy = open_db(base)?;
    let identity_db = open_db(&base.join("identity"))?;
    let outbox_db = open_db(&base.join("outbox"))?;
    let inbox_db = open_db(&base.join("inbox"))?;

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
    println!(
        "Legacy root sled migration completed (copied {} key(s))",
        copied_keys
    );
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn migrate_legacy_cli_storage(base: &Path) -> Result<(), IronCoreError> {
    if let Some(parent) = base.parent() {
        // Move contacts folder from parent to base if it exists
        let old_contacts = parent.join("contacts");
        let new_contacts = base.join("contacts");
        if old_contacts.exists() && !new_contacts.exists() {
            tracing::info!(
                "Migrating legacy CLI contacts from {} to {}",
                old_contacts.display(),
                new_contacts.display()
            );
            let _ = std::fs::rename(old_contacts, new_contacts);
        }

        // Move history folder from parent to base if it exists
        let old_history = parent.join("history");
        let new_history = base.join("history");
        if old_history.exists() && !new_history.exists() {
            tracing::info!(
                "Migrating legacy CLI history from {} to {}",
                old_history.display(),
                new_history.display()
            );
            let _ = std::fs::rename(old_history, new_history);
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
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

    if current < 3 {
        migrate_legacy_cli_storage(base)?;
    }

    if current != STORAGE_SCHEMA_VERSION {
        std::fs::write(&version_file, STORAGE_SCHEMA_VERSION.to_string())
            .map_err(|_| IronCoreError::StorageError)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn fresh_uniffi_storage_root() -> String {
    let root = std::env::temp_dir().join(format!("scmessenger-uniffi-{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&root);
    root.to_string_lossy().to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn init_uniffi_contacts_manager(
    preferred_root: Option<&str>,
) -> Arc<crate::contacts_bridge::ContactManager> {
    let primary_root = preferred_root
        .map(|p| p.to_string())
        .unwrap_or_else(fresh_uniffi_storage_root);
    let manager = crate::contacts_bridge::ContactManager::new(primary_root).unwrap_or_else(|_| {
        crate::contacts_bridge::ContactManager::new(fresh_uniffi_storage_root())
            .expect("failed to initialize UniFFI ContactManager")
    });
    Arc::new(manager)
}

#[cfg(not(target_arch = "wasm32"))]
fn init_uniffi_history_manager(
    preferred_root: Option<&str>,
) -> Arc<crate::mobile_bridge::HistoryManager> {
    let primary_root = preferred_root
        .map(|p| p.to_string())
        .unwrap_or_else(fresh_uniffi_storage_root);
    let manager = crate::mobile_bridge::HistoryManager::new(primary_root).unwrap_or_else(|_| {
        crate::mobile_bridge::HistoryManager::new(fresh_uniffi_storage_root())
            .expect("failed to initialize UniFFI HistoryManager")
    });
    Arc::new(manager)
}

impl IronCore {
    /// Create a new Iron Core instance with in-memory storage
    pub fn new() -> Self {
        Self::init(None, None)
    }

    /// Create Iron Core with persistent storage at the given path
    pub fn with_storage(storage_path: String) -> Self {
        Self::init(Some(storage_path), None)
    }

    /// Create Iron Core with persistent storage and structured tracing
    pub fn with_storage_and_logs(storage_path: String, log_directory: String) -> Self {
        Self::init(Some(storage_path), Some(log_directory))
    }

    fn init(storage_path: Option<String>, log_directory: Option<String>) -> Self {
        // Initialize tracing: file-based if log_directory provided, stdout otherwise
        if let Some(log_dir) = log_directory {
            if let Err(e) = store::tracing_init::init_file_tracing(&log_dir) {
                eprintln!("Failed to initialize file tracing: {}", e);
                // Fallback to stdout tracing
                let _ = tracing_subscriber::fmt()
                    .with_env_filter(
                        tracing_subscriber::EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                    )
                    .try_init();
            }
        } else {
            // Initialize tracing (idempotent, mobile-safe with try_init)
            let _ = tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                )
                .try_init();
        }

        #[allow(unused_variables)]
        let storage_ready = if let Some(path) = &storage_path {
            #[cfg(not(target_arch = "wasm32"))]
            match ensure_storage_layout(path) {
                Ok(()) => true,
                Err(e) => {
                    tracing::error!("Storage layout check failed at {}: {:?}", path, e);
                    false
                }
            }
            #[cfg(target_arch = "wasm32")]
            true
        } else {
            false
        };

        #[allow(unused_variables)]
        let identity = if let Some(path) = &storage_path {
            if !storage_ready {
                Arc::new(RwLock::new(IdentityManager::new()))
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                let backend = Arc::new(
                    crate::store::backend::SledStorage::new(
                        Path::new(path).join("identity").to_string_lossy().as_ref(),
                    )
                    .unwrap(),
                );
                #[cfg(target_arch = "wasm32")]
                let backend = Arc::new(crate::store::backend::MemoryStorage::new());

                Arc::new(RwLock::new(
                    IdentityManager::with_backend(backend.clone())
                        .unwrap_or_else(|_| IdentityManager::new()),
                ))
            }
        } else {
            Arc::new(RwLock::new(IdentityManager::new()))
        };

        #[allow(unused_variables)]
        let outbox = if let Some(path) = &storage_path {
            if !storage_ready {
                store::Outbox::new()
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                let backend = Arc::new(
                    crate::store::backend::SledStorage::new(
                        Path::new(path).join("outbox").to_string_lossy().as_ref(),
                    )
                    .unwrap(),
                );
                #[cfg(target_arch = "wasm32")]
                let backend = Arc::new(crate::store::backend::MemoryStorage::new());

                store::Outbox::persistent(backend)
            }
        } else {
            store::Outbox::new()
        };

        #[allow(unused_variables)]
        let inbox = if let Some(path) = &storage_path {
            if !storage_ready {
                store::Inbox::new()
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                let backend = Arc::new(
                    crate::store::backend::SledStorage::new(
                        Path::new(path).join("inbox").to_string_lossy().as_ref(),
                    )
                    .unwrap(),
                );
                #[cfg(target_arch = "wasm32")]
                let backend = Arc::new(crate::store::backend::MemoryStorage::new());

                store::Inbox::persistent(backend)
            }
        } else {
            store::Inbox::new()
        };

        let contacts = if let Some(path) = &storage_path {
            if !storage_ready {
                store::ContactManager::new(Arc::new(store::backend::MemoryStorage::new()))
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                let backend = Arc::new(
                    crate::store::backend::SledStorage::new(
                        Path::new(path).join("contacts").to_string_lossy().as_ref(),
                    )
                    .unwrap(),
                );
                #[cfg(target_arch = "wasm32")]
                let backend = Arc::new(crate::store::backend::MemoryStorage::new());

                store::ContactManager::new(backend)
            }
        } else {
            store::ContactManager::new(Arc::new(store::backend::MemoryStorage::new()))
        };

        let history = if let Some(path) = &storage_path {
            if !storage_ready {
                store::HistoryManager::new(Arc::new(store::backend::MemoryStorage::new()))
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                let backend = Arc::new(
                    crate::store::backend::SledStorage::new(
                        Path::new(path).join("history").to_string_lossy().as_ref(),
                    )
                    .unwrap(),
                );
                #[cfg(target_arch = "wasm32")]
                let backend = Arc::new(crate::store::backend::MemoryStorage::new());

                store::HistoryManager::new(backend)
            }
        } else {
            store::HistoryManager::new(Arc::new(store::backend::MemoryStorage::new()))
        };

        let history_arc = Arc::new(RwLock::new(history));

        // Root backend for logs and storage metadata
        let root_backend: Arc<dyn store::backend::StorageBackend> =
            if let Some(path) = &storage_path {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    match crate::store::backend::SledStorage::new(
                        Path::new(path).join("root").to_string_lossy().as_ref(),
                    ) {
                        Ok(sled) => Arc::new(sled),
                        Err(e) => {
                            tracing::warn!(
                                "Failed to open root sled backend, falling back to memory: {}",
                                e
                            );
                            Arc::new(store::backend::MemoryStorage::new())
                        }
                    }
                }
                #[cfg(target_arch = "wasm32")]
                Arc::new(store::backend::MemoryStorage::new())
            } else {
                Arc::new(store::backend::MemoryStorage::new())
            };

        let log_manager = Arc::new(store::logs::LogManager::new(root_backend.clone()));
        let storage_manager = Arc::new(store::storage::StorageManager::new(
            history_arc.read().clone().into(),
            log_manager.clone(),
        ));
        let blocked_manager = Arc::new(store::blocked::BlockedManager::new(root_backend.clone()));
        let relay_registry = Arc::new(store::RelayRegistry::new(root_backend.clone()));

        #[cfg(not(target_arch = "wasm32"))]
        let contacts_bridge_manager = init_uniffi_contacts_manager(storage_path.as_deref());
        #[cfg(not(target_arch = "wasm32"))]
        let history_bridge_manager = init_uniffi_history_manager(storage_path.as_deref());

        Self {
            identity,
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            contacts: Arc::new(RwLock::new(contacts)),
            history: history_arc,
            storage_manager,
            log_manager,
            blocked_manager,
            relay_registry,
            audit_log: Arc::new(RwLock::new(AuditLogType::new())),
            #[cfg(not(target_arch = "wasm32"))]
            contacts_bridge_manager,
            #[cfg(not(target_arch = "wasm32"))]
            history_bridge_manager,
            running: Arc::new(RwLock::new(false)),
            delegate: Arc::new(RwLock::new(None)),
        }
    }

    // ------------------------------------------------------------------------
    // ASYNC INIT (WASM ONLY)
    // ------------------------------------------------------------------------

    #[cfg(target_arch = "wasm32")]
    pub async fn with_storage_async(storage_path: String) -> Self {
        Self::init_async(Some(storage_path)).await
    }

    #[cfg(target_arch = "wasm32")]
    async fn init_async(storage_path: Option<String>) -> Self {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init();

        let identity = if let Some(path) = &storage_path {
            let identity_path = Path::new(path).join("identity");
            let backend = Arc::new(
                crate::store::backend::IndexedDbStorage::new(
                    identity_path.to_string_lossy().as_ref(),
                )
                .await
                .expect("Failed to open IndexedDB for identity"),
            );

            Arc::new(RwLock::new(
                IdentityManager::with_backend(backend.clone())
                    .unwrap_or_else(|_| IdentityManager::new()),
            ))
        } else {
            Arc::new(RwLock::new(IdentityManager::new()))
        };

        let outbox = if let Some(path) = &storage_path {
            let outbox_path = Path::new(path).join("outbox");
            let backend = Arc::new(
                crate::store::backend::IndexedDbStorage::new(
                    outbox_path.to_string_lossy().as_ref(),
                )
                .await
                .expect("Failed to open IndexedDB for outbox"),
            );
            store::Outbox::persistent(backend)
        } else {
            store::Outbox::new()
        };

        let inbox = if let Some(path) = &storage_path {
            let inbox_path = Path::new(path).join("inbox");
            let backend = Arc::new(
                crate::store::backend::IndexedDbStorage::new(inbox_path.to_string_lossy().as_ref())
                    .await
                    .expect("Failed to open IndexedDB for inbox"),
            );
            store::Inbox::persistent(backend)
        } else {
            store::Inbox::new()
        };

        let contacts = if let Some(path) = &storage_path {
            let contacts_path = Path::new(path).join("contacts");
            let backend = Arc::new(
                crate::store::backend::IndexedDbStorage::new(
                    contacts_path.to_string_lossy().as_ref(),
                )
                .await
                .expect("Failed to open IndexedDB for contacts"),
            );
            store::ContactManager::new(backend)
        } else {
            store::ContactManager::new(Arc::new(store::backend::MemoryStorage::new()))
        };

        let history = if let Some(path) = &storage_path {
            let history_path = Path::new(path).join("history");
            let backend = Arc::new(
                crate::store::backend::IndexedDbStorage::new(
                    history_path.to_string_lossy().as_ref(),
                )
                .await
                .expect("Failed to open IndexedDB for history"),
            );
            store::HistoryManager::new(backend)
        } else {
            store::HistoryManager::new(Arc::new(store::backend::MemoryStorage::new()))
        };

        let history_arc = Arc::new(RwLock::new(history));
        let root_backend: Arc<dyn store::backend::StorageBackend> =
            Arc::new(store::backend::MemoryStorage::new());
        let log_manager = Arc::new(store::logs::LogManager::new(root_backend.clone()));
        let storage_manager = Arc::new(store::storage::StorageManager::new(
            history_arc.read().clone().into(),
            log_manager.clone(),
        ));
        let blocked_manager = Arc::new(store::blocked::BlockedManager::new(root_backend.clone()));
        let relay_registry = Arc::new(store::RelayRegistry::new(root_backend));

        Self {
            identity,
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            contacts: Arc::new(RwLock::new(contacts)),
            history: history_arc,
            storage_manager,
            log_manager,
            blocked_manager,
            relay_registry,
            audit_log: Arc::new(RwLock::new(AuditLogType::new())),
            running: Arc::new(RwLock::new(false)),
            delegate: Arc::new(RwLock::new(None)),
        }
    }

    /// Internal helper: emit an audit event to the tamper-evident log.
    /// Fire-and-forget — never propagates errors to callers.
    fn emit_audit(
        &self,
        event_type: AuditEventType,
        peer_id: Option<String>,
        details: Option<String>,
    ) {
        let identity_id = self
            .identity
            .read()
            .identity_id()
            .unwrap_or_else(|| "unknown".to_string());
        let mut log = self.audit_log.write();
        let _ = log.append(event_type, Some(identity_id), peer_id, details);
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
            .map_err(|_| IronCoreError::CryptoError)?;
        self.emit_audit(AuditEventType::IdentityCreated, None, None);
        Ok(())
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
            device_id: identity.device_id(),
            seniority_timestamp: identity.seniority_timestamp(),
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

    /// Get device ID for this installation (WS13.1)
    pub fn get_device_id(&self) -> Option<String> {
        self.identity.read().device_id()
    }

    /// Get seniority timestamp for this installation (WS13.1)
    pub fn get_seniority_timestamp(&self) -> Option<u64> {
        self.identity.read().seniority_timestamp()
    }

    /// Get the relay registration state for an identity/public-key lookup.
    pub fn get_registration_state(&self, identity_id: String) -> RegistrationStateInfo {
        let info = self.relay_registry.get_state_info(&identity_id);
        RegistrationStateInfo {
            state: info.state,
            device_id: info.device_id,
            seniority_timestamp: info.seniority_timestamp,
        }
    }

    pub(crate) fn build_registration_request(
        &self,
    ) -> Result<transport::RegistrationRequest, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let device_id = identity.device_id().ok_or(IronCoreError::NotInitialized)?;
        let seniority_ts = identity
            .seniority_timestamp()
            .ok_or(IronCoreError::NotInitialized)?;
        transport::RegistrationRequest::new_signed(keys, device_id, seniority_ts)
            .map_err(|_| IronCoreError::InvalidInput)
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
        self.emit_audit(AuditEventType::BackupExported, None, None);
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
        drop(identity);
        self.emit_audit(AuditEventType::BackupImported, None, None);
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

    /// Resolve any ID format to canonical public_key_hex (64 hex chars).
    ///
    /// Accepts:
    /// - `public_key_hex` (64 hex chars) - returns as-is
    /// - `identity_id` (64 hex chars, Blake3 hash) - looks up in contacts
    /// - `libp2p_peer_id` (base58, starts with "12D3Koo") - extracts public key
    ///
    /// This provides a single resolution point for ID unification across platforms.
    pub fn resolve_identity(&self, any_id: String) -> Result<String, IronCoreError> {
        let trimmed = any_id.trim();

        // Check if it's already a valid 64-char hex public key
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            // Could be public_key_hex OR identity_id (both 64 hex chars)
            // Check contacts for identity_id match FIRST, before key-shape test,
            // because some identity hashes are valid Ed25519 points and would be
            // misclassified as public keys.
            let contacts = self.contacts.read();
            if let Ok(all_contacts) = contacts.list() {
                for contact in &all_contacts {
                    // Check if this is an identity_id (Blake3 hash of public key)
                    if let Ok(pub_bytes) = hex::decode(&contact.public_key) {
                        let hash = blake3::hash(&pub_bytes);
                        let computed_identity_id = hex::encode(hash.as_bytes());
                        if computed_identity_id.eq_ignore_ascii_case(trimmed) {
                            return Ok(contact.public_key.to_lowercase());
                        }
                    }
                    // Also check if the input matches the contact's public key directly
                    if contact.public_key.eq_ignore_ascii_case(trimmed) {
                        return Ok(contact.public_key.to_lowercase());
                    }
                }
            }
            drop(contacts);

            // No contact match — verify it's a valid Ed25519 public key
            if let Ok(bytes) = hex::decode(trimmed) {
                if bytes.len() == 32
                    && ed25519_dalek::VerifyingKey::from_bytes(bytes.as_slice().try_into().unwrap())
                        .is_ok()
                {
                    return Ok(trimmed.to_lowercase());
                }
            }

            // Not a valid public key and not a known identity_id — assume public key format
            return Ok(trimmed.to_lowercase());
        }

        // Check if it's a libp2p peer ID (base58, typically starts with "12D3Koo")
        if trimmed.starts_with("1") && trimmed.len() > 40 {
            return self
                .extract_public_key_from_peer_id(trimmed.to_string())
                .map(|pk| pk.to_lowercase());
        }

        Err(IronCoreError::InvalidInput)
    }

    /// Resolve any ID format to canonical identity_id (Blake3 hash of public key).
    ///
    /// This provides a single resolution point for user identification across platforms.
    pub fn resolve_to_identity_id(&self, any_id: String) -> Result<String, IronCoreError> {
        let pk_hex = self.resolve_identity(any_id)?;
        let pk_bytes = hex::decode(&pk_hex).map_err(|_| IronCoreError::InvalidInput)?;
        let hash = blake3::hash(&pk_bytes);
        Ok(hex::encode(hash.as_bytes()))
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

        // Enforce strict payload cap at the core boundary so callers get
        // a stable InvalidInput error for oversize messages.
        message::codec::validate_payload_size(text.as_bytes())
            .map_err(|_| IronCoreError::InvalidInput)?;

        // Create plaintext message
        let msg = Message::text(sender_id, recipient_key_trimmed.clone(), &text);
        let message_id = msg.id.clone();

        // Structured tracing: Create span for packet lifecycle
        let span = tracing::info_span!(
            "packet_lifecycle",
            message_id = %message_id,
            recipient = %recipient_key_trimmed
        );
        let _guard = span.enter();

        tracing::info!(
            event = "message_created",
            payload_size = text.len(),
            timestamp = msg.timestamp
        );

        // Auto-save to history (Outgoing)
        let history = self.history.write();
        let local_ts = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = history.add(store::MessageRecord {
            id: message_id.clone(),
            direction: store::MessageDirection::Sent,
            peer_id: recipient_key_trimmed.clone(),
            content: text.clone(),
            timestamp: local_ts,
            sender_timestamp: msg.timestamp,
            delivered: false,
            hidden: false,
        });

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
                queued_at: web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                attempts: 0,
            })
            .map_err(|_| IronCoreError::StorageError)?;

        // Audit: message prepared for sending
        self.emit_audit(
            AuditEventType::MessageSent,
            Some(recipient_key_trimmed.clone()),
            None,
        );

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

    pub fn classify_notification(
        &self,
        message: NotificationMessageContext,
        ui_state: NotificationUiState,
        settings: MeshSettings,
    ) -> NotificationDecision {
        notification::classify_notification(message, ui_state, settings)
    }

    /// Decrypt a received envelope and return the plaintext message.
    pub fn receive_message(&self, envelope_bytes: Vec<u8>) -> Result<Message, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or_else(|| {
            eprintln!("[IronCore] receive_message FAILED: identity keys not initialized");
            IronCoreError::NotInitialized
        })?;

        // Deserialize envelope
        let envelope = message::decode_envelope(&envelope_bytes).map_err(|e| {
            let err_msg = format!(
                "[IronCore] receive_message FAILED: envelope decode error (len={}, err={:?})\n",
                envelope_bytes.len(),
                e
            );
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/scm_debug.log")
                .map(|mut f| {
                    use std::io::Write;
                    f.write_all(err_msg.as_bytes())
                });
            eprintln!("{}", err_msg);
            IronCoreError::Internal
        })?;

        // Decrypt
        let plaintext = crypto::decrypt_message(&keys.signing_key, &envelope)
            .map_err(|e| {
                let err_msg = format!("[IronCore] receive_message FAILED: decrypt error — sender_key={}, local_key={}, err={:?}\n", hex::encode(&envelope.sender_public_key), hex::encode(keys.signing_key.verifying_key().to_bytes()), e);
                let _ = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/scm_debug.log").map(|mut f| { use std::io::Write; f.write_all(err_msg.as_bytes()) });
                eprintln!("{}", err_msg);
                IronCoreError::CryptoError
            })?;

        // Deserialize message
        let msg = message::decode_message(&plaintext).map_err(|e| {
            let err_msg = format!("[IronCore] receive_message FAILED: message decode error (plaintext_len={}, err={:?})\n", plaintext.len(), e);
            let _ = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/scm_debug.log").map(|mut f| { use std::io::Write; f.write_all(err_msg.as_bytes()) });
            eprintln!("{}", err_msg);
            IronCoreError::Internal
        })?;

        // Ingress filtering: check blocked state for this sender BEFORE dedup or
        // history persistence.  We derive the sender identity id the same way the
        // history record does (Blake3 hash of the envelope sender public key).
        let sender_identity_id = hex::encode(blake3::hash(&envelope.sender_public_key).as_bytes());

        // Helper: check blocked state against both the identity id (Blake3 hash)
        // and the raw sender_id embedded in the message, since the two can differ
        // for legacy or cross-version peers.
        let sender_is_blocked_deleted = self
            .blocked_manager
            .is_blocked_and_deleted(&sender_identity_id)
            .unwrap_or(false)
            || self
                .blocked_manager
                .is_blocked_and_deleted(&msg.sender_id)
                .unwrap_or(false);

        let sender_is_blocked = self
            .blocked_manager
            .is_blocked(&sender_identity_id, None)
            .unwrap_or(false)
            || self
                .blocked_manager
                .is_blocked(&msg.sender_id, None)
                .unwrap_or(false);

        // Blocked + Deleted: reject the payload entirely — do not store, do not
        // invoke the delegate, do not dedup-register.  Returning an error
        // prevents callers (CLI, WASM, mobile) from surfacing the decrypted
        // content, which is the correct ingress-drop semantic.
        if sender_is_blocked_deleted {
            return Err(IronCoreError::Blocked);
        }

        // Blocked-only: tag the history record as hidden for evidentiary retention.
        let is_blocked_only = sender_is_blocked;

        // Dedup check
        let mut inbox = self.inbox.write();
        let is_new = inbox.receive(store::ReceivedMessage {
            message_id: msg.id.clone(),
            sender_id: msg.sender_id.clone(),
            payload: msg.payload.clone(),
            received_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        let sender_pub_key_hex = hex::encode(&envelope.sender_public_key);

        // Auto-save to history (Incoming)
        if is_new && msg.message_type == message::MessageType::Text {
            if let Some(text) = msg.text_content() {
                let local_ts = web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                // For blocked-only (hidden) messages, use the derived sender_identity_id
                // as the peer_id so that unhide_messages_for_peer() — which is keyed by
                // the same identifier used in block()/unblock() — can reliably match.
                // For non-blocked messages, continue using msg.sender_id which is
                // typically identical (both derive from Blake3 of the sender public key).
                let record_peer_id = if is_blocked_only {
                    sender_identity_id.clone()
                } else {
                    msg.sender_id.clone()
                };
                let _ = self.history.read().add(store::MessageRecord {
                    id: msg.id.clone(),
                    direction: store::MessageDirection::Received,
                    peer_id: record_peer_id.clone(),
                    content: text.clone(),
                    timestamp: local_ts,
                    sender_timestamp: msg.timestamp,
                    delivered: true,
                    // Evidentiary retention: mark as hidden if sender is blocked.
                    // The record is persisted but filtered from UI queries.
                    hidden: is_blocked_only,
                });
                // Also write to the mobile bridge history so that mobile apps
                // (which query the sled-based HistoryManager) can restore hidden
                // messages on unblock.  Without this, suppressing the delegate
                // callback for blocked-only peers would leave the mobile bridge
                // store empty, breaking the unblock-restore flow.
                #[cfg(not(target_arch = "wasm32"))]
                if is_blocked_only {
                    let mobile_record = crate::mobile_bridge::MessageRecord {
                        id: msg.id.clone(),
                        direction: crate::mobile_bridge::MessageDirection::Received,
                        peer_id: record_peer_id,
                        content: text,
                        timestamp: local_ts,
                        sender_timestamp: msg.timestamp,
                        delivered: true,
                        hidden: true,
                    };
                    let _ = self.history_bridge_manager.add(mobile_record);
                }
            }
        }

        // Final receipt transitions delivery state in-core so all platform
        // adapters observe coherent outbox/history state.
        // Zero-Status Architecture: Receipt processing is internal only — the Core
        // never emits delivery status events across the FFI boundary.
        let mut is_receipt_message = false;
        if msg.message_type == message::MessageType::Receipt {
            is_receipt_message = true;
            if let Ok(receipt) = bincode::deserialize::<message::Receipt>(&msg.payload) {
                let log_receipt_ignore = |message_id: &str, reason: &str| {
                    let err_msg = format!(
                        "[IronCore] ignoring receipt for message {}: {}\n",
                        message_id, reason
                    );
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/tmp/scm_debug.log")
                        .map(|mut f| {
                            use std::io::Write;
                            f.write_all(err_msg.as_bytes())
                        });
                    eprintln!("{}", err_msg);
                };
                let local_public_key_hex = hex::encode(keys.signing_key.verifying_key().to_bytes());
                let expected_sender_public_key_hex = hex::encode(&envelope.sender_public_key);
                let expected_sender_identity =
                    hex::encode(blake3::hash(&envelope.sender_public_key).as_bytes());
                let outbound_record = self
                    .history
                    .read()
                    .get(receipt.message_id.clone())
                    .ok()
                    .flatten()
                    .filter(|record| record.direction == store::MessageDirection::Sent);
                if outbound_record.is_none() {
                    log_receipt_ignore(&receipt.message_id, "message not found or is not outbound");
                } else if !msg.recipient_id.eq_ignore_ascii_case(&local_public_key_hex) {
                    log_receipt_ignore(
                        &receipt.message_id,
                        "recipient mismatch (msg recipient != local key)",
                    );
                } else if !msg
                    .sender_id
                    .eq_ignore_ascii_case(&expected_sender_identity)
                {
                    log_receipt_ignore(
                        &receipt.message_id,
                        "sender identity does not match envelope sender key",
                    );
                } else if outbound_record.as_ref().is_some_and(|record| {
                    let matches_expected_sender = record
                        .peer_id
                        .eq_ignore_ascii_case(&expected_sender_identity)
                        || record
                            .peer_id
                            .eq_ignore_ascii_case(&expected_sender_public_key_hex)
                        || record.peer_id.eq_ignore_ascii_case(&msg.sender_id);
                    !matches_expected_sender
                }) {
                    log_receipt_ignore(
                        &receipt.message_id,
                        "sender identity does not match outbound recipient",
                    );
                } else {
                    // Backward compat: legacy Read receipts from older peers are
                    // treated as Delivered so they still clear the outbox/history.
                    #[allow(deprecated)]
                    let is_delivered_or_read = matches!(
                        receipt.status,
                        message::DeliveryStatus::Delivered | message::DeliveryStatus::Read
                    );
                    if is_delivered_or_read {
                        tracing::info!(
                            event = "receipt_verified",
                            message_id = %receipt.message_id,
                            sender_identity = %expected_sender_identity,
                            status = "delivered"
                        );
                        let _ = self.mark_message_sent(receipt.message_id.clone());
                        let _ = self
                            .history
                            .read()
                            .mark_delivered(receipt.message_id.clone());
                    }
                }
            }
        }

        // Notify delegate
        // Zero-Status Architecture: Receipt processing is internal only.
        // The Core never emits delivery status events across the FFI boundary.
        // on_receipt_received is intentionally suppressed to decouple the UI.
        //
        // Evidentiary retention: blocked-only peer messages are stored hidden.
        // The delegate is NOT invoked so the UI never surfaces a notification
        // or tries to insert the message into the mobile-bridge history store.
        if let Some(delegate) = self.delegate.read().as_ref() {
            if !is_receipt_message && !is_blocked_only {
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

    #[cfg(not(target_arch = "wasm32"))]
    pub fn contacts_manager(&self) -> Arc<crate::contacts_bridge::ContactManager> {
        self.contacts_bridge_manager.clone()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn contacts_manager(&self) -> store::ContactManager {
        self.contacts.read().clone()
    }

    pub fn contacts_store_manager(&self) -> store::ContactManager {
        self.contacts.read().clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn history_manager(&self) -> Arc<crate::mobile_bridge::HistoryManager> {
        self.history_bridge_manager.clone()
    }

    #[cfg(target_arch = "wasm32")]
    pub fn history_manager(&self) -> store::HistoryManager {
        self.history.read().clone()
    }

    pub fn history_store_manager(&self) -> store::HistoryManager {
        self.history.read().clone()
    }

    pub fn update_disk_stats(&self, total_bytes: u64, free_bytes: u64) {
        self.storage_manager
            .update_disk_stats(total_bytes, free_bytes);
    }

    pub fn perform_maintenance(&self) -> Result<(), IronCoreError> {
        self.storage_manager.perform_maintenance()
    }

    pub fn record_log(&self, line: String) {
        self.log_manager.record_log(line);
    }

    pub fn export_logs(&self) -> Result<String, IronCoreError> {
        self.log_manager.export_all()
    }

    // ========================================================================
    // BLOCKING
    // ========================================================================

    /// Block a peer by ID
    pub fn block_peer(&self, peer_id: String, reason: Option<String>) -> Result<(), IronCoreError> {
        use crate::store::blocked::BlockedIdentity;
        let mut blocked = BlockedIdentity::new(peer_id.clone());
        if let Some(r) = reason {
            blocked.reason = Some(r);
        }
        self.blocked_manager.block(blocked)?;
        self.emit_audit(AuditEventType::ContactBlocked, Some(peer_id), None);
        Ok(())
    }

    /// Unblock a peer and restore any evidentiary-retained messages to visible.
    pub fn unblock_peer(&self, peer_id: String) -> Result<(), IronCoreError> {
        self.blocked_manager.unblock(peer_id.clone(), None)?;
        self.emit_audit(AuditEventType::ContactRemoved, Some(peer_id.clone()), None);
        // Restore visibility of messages that were hidden during the block period.
        // Log but do not fail the unblock operation if restoration encounters a
        // storage error — the important state transition (unblock) has already
        // succeeded and the user is no longer blocked.
        match self.history.read().unhide_messages_for_peer(&peer_id) {
            Ok(count) => {
                tracing::info!(
                    event = "unblock_messages_restored",
                    peer_id = %peer_id,
                    count = count,
                );
            }
            Err(e) => {
                tracing::warn!(
                    event = "unblock_messages_restore_failed",
                    peer_id = %peer_id,
                    error = ?e,
                    "failed to restore hidden messages after unblock; messages may remain hidden"
                );
            }
        }
        // Also unhide in the mobile bridge history (sled-based) so that mobile
        // apps see the restored messages immediately.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self
                .history_bridge_manager
                .unhide_messages_for_peer(&peer_id);
        }
        Ok(())
    }

    /// Block a peer AND delete them (cascade purge).
    ///
    /// This performs three actions atomically from the caller's perspective:
    /// 1. Marks the peer as `blocked + deleted` in the block store, causing the
    ///    ingress layer to **reject** all future payloads without persisting them.
    /// 2. **Purges** all existing stored messages for this peer from all history
    ///    stores (core and mobile bridge).
    /// 3. **Removes** the contact record from both core and mobile bridge contact
    ///    stores.
    ///
    /// This is irreversible — purged messages cannot be recovered.
    pub fn block_and_delete_peer(
        &self,
        peer_id: String,
        reason: Option<String>,
    ) -> Result<(), IronCoreError> {
        // 1. Set blocked+deleted state so future ingress rejects payloads.
        self.blocked_manager
            .block_and_delete(peer_id.clone(), reason)?;
        self.emit_audit(AuditEventType::ContactBlocked, Some(peer_id.clone()), Some("block_and_delete".to_string()));
        // 2. Purge all existing stored messages for this peer from core history.
        //    Propagate storage errors so callers know when the purge is incomplete.
        self.history.read().remove_conversation(peer_id.clone())?;
        // Also purge from the mobile bridge history (sled-based).
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self
                .history_bridge_manager
                .remove_conversation(peer_id.clone());
        }
        // 3. Remove the contact record.
        let _ = self.contacts.read().remove(peer_id.clone());
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self.contacts_bridge_manager.remove(peer_id.clone());
        }
        Ok(())
    }

    /// Check if a peer is blocked
    pub fn is_peer_blocked(&self, peer_id: String) -> Result<bool, IronCoreError> {
        self.blocked_manager.is_blocked(&peer_id, None)
    }

    /// List all blocked peers (mobile bridge only - not available on WASM)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn list_blocked_peers(
        &self,
    ) -> Result<Vec<crate::blocked_bridge::BlockedIdentity>, IronCoreError> {
        let core_list = self.blocked_manager.list()?;
        Ok(core_list
            .into_iter()
            .map(crate::blocked_bridge::BlockedIdentity::from)
            .collect())
    }

    /// List all blocked peers — returns the core store type.
    /// Available on all platforms including WASM (unlike `list_blocked_peers` which
    /// returns the UniFFI bridge type and is gated to non-wasm32 targets).
    pub fn list_blocked_peers_raw(
        &self,
    ) -> Result<Vec<store::blocked::BlockedIdentity>, IronCoreError> {
        self.blocked_manager.list()
    }

    /// Get count of blocked peers
    pub fn blocked_count(&self) -> Result<u32, IronCoreError> {
        self.blocked_manager.count().map(|c| c as u32)
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
        assert!(info_after.device_id.is_some());
        assert!(info_after.seniority_timestamp.is_some());

        // Public key should be 64 hex chars (32 bytes)
        assert_eq!(info_after.public_key_hex.unwrap().len(), 64);
        let parsed_uuid = uuid::Uuid::parse_str(info_after.device_id.as_deref().unwrap()).unwrap();
        assert_eq!(parsed_uuid.get_version_num(), 4);
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let seniority = info_after.seniority_timestamp.unwrap();
        assert!(seniority > 0);
        assert!(seniority <= now);
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
        let original_info = first.get_identity_info();
        drop(first);

        let second = IronCore::with_storage(path);
        let reloaded = second.get_identity_info();
        assert!(reloaded.initialized);
        assert_eq!(reloaded.nickname.as_deref(), Some("persisted-hydrate"));
        assert_eq!(reloaded.identity_id, original_info.identity_id);
        assert_eq!(reloaded.device_id, original_info.device_id);
        assert_eq!(
            reloaded.seniority_timestamp,
            original_info.seniority_timestamp
        );
    }

    #[test]
    fn test_with_storage_migrates_legacy_root_identity_layout() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();

        // Simulate pre-schema-split storage where identity keys lived in root sled.
        let legacy_store = sled::open(&path).unwrap();
        let legacy_keys = identity::IdentityKeys::generate();
        legacy_store
            .insert(LEGACY_IDENTITY_KEY, legacy_keys.to_bytes())
            .unwrap();
        legacy_store
            .insert(LEGACY_NICKNAME_KEY, b"legacy-nick")
            .unwrap();
        legacy_store.flush().unwrap();
        drop(legacy_store);

        let core = IronCore::with_storage(path.clone());
        let info = core.get_identity_info();
        assert!(info.initialized);
        assert_eq!(
            info.public_key_hex.as_deref(),
            Some(legacy_keys.public_key_hex().as_str())
        );
        assert_eq!(info.nickname.as_deref(), Some("legacy-nick"));
        assert!(info.device_id.is_some());
        assert!(info.seniority_timestamp.is_some());

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

    #[test]
    fn test_identity_backup_roundtrip() {
        let core = IronCore::new();
        core.initialize_identity().unwrap();

        let backup = core.export_identity_backup().unwrap();
        assert!(!backup.is_empty());

        // Backup payload is valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&backup).unwrap();
        assert_eq!(parsed["version"], 1);
        assert!(parsed["secret_key_hex"].is_string());
        assert!(parsed.get("device_id").is_none());
        assert!(parsed.get("seniority_timestamp").is_none());

        // Import into a fresh core and verify identity is restored
        let core2 = IronCore::new();
        core2.import_identity_backup(backup).unwrap();

        let orig = core.get_identity_info();
        let restored = core2.get_identity_info();
        assert_eq!(
            orig.public_key_hex, restored.public_key_hex,
            "public key must be identical after import"
        );
        assert_ne!(
            orig.device_id, restored.device_id,
            "device metadata should remain installation-local across restore"
        );
        assert!(restored.seniority_timestamp.is_some());
    }

    #[test]
    fn test_import_identity_backup_invalid_version() {
        let core = IronCore::new();
        let bad = r#"{"version":99,"secret_key_hex":"aabb","nickname":null}"#.to_string();
        assert!(core.import_identity_backup(bad).is_err());
    }

    #[test]
    fn test_import_identity_backup_invalid_hex() {
        let core = IronCore::new();
        let bad = r#"{"version":1,"secret_key_hex":"not-hex!!","nickname":null}"#.to_string();
        assert!(core.import_identity_backup(bad).is_err());
    }

    #[test]
    fn test_mark_message_sent_removes_from_outbox() {
        let core = IronCore::new();
        let recipient = IronCore::new();
        core.initialize_identity().unwrap();
        recipient.initialize_identity().unwrap();

        let recipient_pk = recipient.get_identity_info().public_key_hex.unwrap();
        let prepared = core
            .prepare_message_with_id(recipient_pk, "hello".to_string())
            .unwrap();
        assert_eq!(core.outbox_count(), 1);

        // Mark the message as sent — it should be removed from the outbox.
        let removed = core.mark_message_sent(prepared.message_id);
        assert!(removed);
        assert_eq!(core.outbox_count(), 0);
    }

    #[test]
    fn test_mark_message_sent_unknown_id_returns_false() {
        let core = IronCore::new();
        core.initialize_identity().unwrap();
        let removed = core.mark_message_sent("nonexistent-id".to_string());
        assert!(!removed);
    }

    #[test]
    fn test_prepare_message_payload_boundaries() {
        let sender = IronCore::new();
        let recipient = IronCore::new();
        sender.initialize_identity().unwrap();
        recipient.initialize_identity().unwrap();
        let recipient_pk = recipient.get_identity_info().public_key_hex.unwrap();

        let within_8191 = "a".repeat(8191);
        let at_8192 = "a".repeat(8192);
        let over_8193 = "a".repeat(8193);

        assert!(sender
            .prepare_message(recipient_pk.clone(), within_8191)
            .is_ok());
        assert!(sender
            .prepare_message(recipient_pk.clone(), at_8192)
            .is_ok());
        assert!(matches!(
            sender.prepare_message(recipient_pk, over_8193),
            Err(IronCoreError::InvalidInput)
        ));
    }

    #[test]
    fn test_delivery_receipt_marks_history_and_outbox_delivered() {
        let sender = IronCore::new();
        let recipient = IronCore::new();
        sender.initialize_identity().unwrap();
        recipient.initialize_identity().unwrap();

        let recipient_pk = recipient.get_identity_info().public_key_hex.unwrap();
        let sender_pk = sender.get_identity_info().public_key_hex.unwrap();

        let prepared = sender
            .prepare_message_with_id(recipient_pk, "ws4 receipt convergence".to_string())
            .unwrap();
        assert_eq!(sender.outbox_count(), 1);

        recipient
            .receive_message(prepared.envelope_data.clone())
            .expect("recipient should decrypt original message");

        let receipt_envelope = recipient
            .prepare_receipt(sender_pk, prepared.message_id.clone())
            .expect("recipient should prepare delivery receipt");

        sender
            .receive_message(receipt_envelope)
            .expect("sender should decrypt receipt");

        assert_eq!(sender.outbox_count(), 0);
        let history = sender.history_store_manager();
        let record = history
            .get(prepared.message_id)
            .expect("history lookup should succeed")
            .expect("history record should exist");
        assert!(record.delivered);
    }

    #[test]
    fn test_mismatched_sender_receipt_is_ignored() {
        let sender = IronCore::new();
        let recipient = IronCore::new();
        let attacker = IronCore::new();
        sender.initialize_identity().unwrap();
        recipient.initialize_identity().unwrap();
        attacker.initialize_identity().unwrap();

        let recipient_pk = recipient.get_identity_info().public_key_hex.unwrap();
        let sender_pk = sender.get_identity_info().public_key_hex.unwrap();

        let prepared = sender
            .prepare_message_with_id(recipient_pk, "forged receipt should be ignored".to_string())
            .unwrap();
        assert_eq!(sender.outbox_count(), 1);

        let forged_receipt_envelope = attacker
            .prepare_receipt(sender_pk, prepared.message_id.clone())
            .expect("attacker can craft syntactically valid receipt envelope");

        sender
            .receive_message(forged_receipt_envelope)
            .expect("sender should still decrypt forged receipt envelope");

        assert_eq!(sender.outbox_count(), 1);
        let history = sender.history_store_manager();
        let record = history
            .get(prepared.message_id)
            .expect("history lookup should succeed")
            .expect("history record should exist");
        assert!(!record.delivered);
    }

    #[test]
    fn test_blocklist_persistence_across_calls() {
        let core = IronCore::new();

        // Initially no blocked peers
        assert_eq!(core.blocked_count().unwrap(), 0);
        assert!(core.list_blocked_peers().unwrap().is_empty());

        // Block a peer
        core.block_peer("peer123".into(), Some("test reason".into()))
            .unwrap();

        // Verify persistence across separate calls
        assert_eq!(core.blocked_count().unwrap(), 1);
        assert!(core.is_peer_blocked("peer123".into()).unwrap());
        assert!(!core.is_peer_blocked("peer456".into()).unwrap());

        let list = core.list_blocked_peers().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].peer_id, "peer123");

        // Unblock
        core.unblock_peer("peer123".into()).unwrap();
        assert_eq!(core.blocked_count().unwrap(), 0);
        assert!(!core.is_peer_blocked("peer123".into()).unwrap());
    }

    #[test]
    fn test_resolve_identity_checks_contacts_before_key_shape() {
        let core = IronCore::new();
        core.initialize_identity().unwrap();

        // Add a contact with a known public key
        let contacts = core.contacts.read();
        let fake_pk = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
        let contact = crate::store::Contact::new(fake_pk.to_string(), fake_pk.to_string());
        let _ = contacts.add(contact);
        drop(contacts);

        // Compute the identity_id (Blake3 hash of public key bytes)
        let pub_bytes = hex::decode(fake_pk).unwrap();
        let hash = blake3::hash(&pub_bytes);
        let identity_id = hex::encode(hash.as_bytes());

        // Resolving the identity_id should return the contact's public key, not the identity_id itself
        let resolved = core.resolve_identity(identity_id.clone()).unwrap();
        assert_eq!(resolved, fake_pk.to_lowercase());
        // The resolved value should differ from the identity_id input
        assert_ne!(resolved, identity_id.to_lowercase());
    }
}
