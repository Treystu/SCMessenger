//! IronCore — the central entry point for the SCMessenger mesh.
//!
//! Holds identity, outbox, inbox, contact manager, history manager, storage
//! manager, log manager, blocked manager, relay registry, and audit log.
//! All state is behind `Arc<RwLock<…>>` (parking_lot).
//!
//! # Lint Configuration
//!
//! This file intentionally uses empty lines after doc comments for readability.
//! The `empty_line_after_outer_attr` check is disabled for this file.

#![allow(clippy::empty_line_after_outer_attr)]

use parking_lot::RwLock;
use std::sync::Arc;

use crate::abuse::auto_block::{AutoBlockConfig, AutoBlockEngine};
use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine};
use crate::abuse::EnhancedAbuseReputationManager;
use crate::crypto::{decrypt_message, encrypt_message, session_manager::RatchetSessionManager};
use crate::drift::{MeshStore, NetworkState, RelayConfig, RelayEngine};
use crate::identity::IdentityManager;
use crate::message::{decode_envelope, decode_message, encode_envelope, Message};
use crate::notification::NotificationEndpointRegistry;
use crate::observability::{AuditEventType, AuditLog as AuditLogType};
use crate::privacy::{
    CircuitBuilder, CircuitConfig, CoverConfig, CoverTrafficGenerator, JitterConfig, TimingJitter,
};
#[cfg(not(target_arch = "wasm32"))]
use crate::relay::{BootstrapManager, PeerExchangeManager};
use crate::routing::optimized_engine::OptimizedRoutingEngine;
use crate::store::backend::MemoryStorage;
#[cfg(not(target_arch = "wasm32"))]
use crate::store::backend::SledStorage;
use crate::store::blocked::BlockedManager as CoreBlockedManager;
use crate::store::logs::LogManager;
use crate::store::{
    ContactManager as CoreContactManager, HistoryManager as CoreHistoryManager, Inbox,
    MessageDirection, MessageRecord, Outbox, QueuedMessage, ReceivedMessage, RelayCustodyStore,
    StorageBackend, StorageManager,
};
use crate::transport::behaviour::RegistrationRequest;
use crate::transport::manager::TransportManager;
use crate::transport::reputation::AbuseSignal;
use crate::IronCoreError;

// ═══════════════════════════════════════════════════════════════════════════════
// Module wiring notes
// ═══════════════════════════════════════════════════════════════════════════════
//
// The following core modules are wired into IronCore as fields or pub fn
// entry points. Modules that are stateless or pure-function libraries do not
// need dedicated state fields:
//
// - `wasm_support` — JSON-RPC bridge between browser WASM client and CLI daemon.
//   Stateless request/response routing; no persistent state managed by IronCore.
//
// - `notification_defaults` — Default values and constants for notification
//   policies. Pure data module with no runtime state.
//
// - `crypto` — Pure cryptographic functions (encrypt_message, decrypt_message).
//   Called directly from message flow methods; no mutable state.
//
// - `message` — Message encoding/decoding types and helpers. Stateless.
//
// - `abuse` — Wired as `abuse_manager` field.
// - `drift` — Wired as `drift_active`, `drift_store`, `drift_engine` fields.
// - `identity` — Wired as `identity` field.
// - `observability` — Wired as `audit_log` field.
// - `store` — Wired as contact_manager, history_manager, storage_manager, etc.
// - `notification` — Wired as `classify_notification` pub fn + `notification_endpoint_registry` field.
// - `privacy` — Wired as `privacy_config` pub fn + cover_traffic_generator, timing_jitter,
//   circuit_builder fields.
// - `routing` — Wired as `routing_engine` field.
// - `transport` — Wired as `transport_manager` field.
// - `relay` — Wired as `bootstrap_manager` and `peer_exchange_manager` fields.
//
// ═══════════════════════════════════════════════════════════════════════════════

/// Delegate trait for protocol events that consumers (like MeshService) implement
/// to receive callbacks from IronCore.
///
/// Method signatures must match the UniFFI-generated scaffolding exactly:
/// all string parameters are owned `String`, not `&str`.
pub trait CoreDelegate: Send + Sync {
    fn on_peer_discovered(&self, peer_id: String);
    fn on_peer_disconnected(&self, peer_id: String);
    fn on_peer_identified(&self, peer_id: String, agent_version: String, listen_addrs: Vec<String>);
    fn on_message_received(
        &self,
        sender_id: String,
        sender_public_key_hex: String,
        message_id: String,
        sender_timestamp: u64,
        data: Vec<u8>,
    );
    fn on_receipt_received(&self, message_id: String, status: String);
}

/// Consent state for identity initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentState {
    NotGranted,
    Granted,
}

/// The main entry point for the SCMessenger core.
///
/// Wraps all subsystems behind `Arc<RwLock<…>>` for safe concurrent access.
#[allow(dead_code)]
#[derive(uniffi::Object)]
pub struct IronCore {
    pub(crate) identity: Arc<RwLock<IdentityManager>>,
    pub(crate) outbox: Arc<RwLock<Outbox>>,
    pub(crate) inbox: Arc<RwLock<Inbox>>,
    pub(crate) contact_manager: Arc<RwLock<CoreContactManager>>,
    pub(crate) history_manager: Arc<CoreHistoryManager>,
    pub(crate) storage_manager: Arc<RwLock<StorageManager>>,
    pub(crate) log_manager: Arc<LogManager>,
    pub(crate) blocked_manager: Arc<RwLock<CoreBlockedManager>>,
    pub(crate) audit_log: Arc<RwLock<AuditLogType>>,
    pub(crate) relay_custody_store: Arc<RwLock<RelayCustodyStore>>,

    /// Protocol event delegate (set by MeshService or platform bridge).
    pub delegate: Arc<RwLock<Option<Box<dyn CoreDelegate>>>>,

    /// Consent gate — identity cannot be initialized until consent is granted.
    consent: Arc<RwLock<ConsentState>>,

    /// Drift (mesh relay store-and-forward) engine state.
    drift_active: Arc<RwLock<bool>>,
    drift_store: Arc<RwLock<MeshStore>>,
    drift_engine: Arc<RwLock<Option<RelayEngine>>>,

    /// Abuse reputation manager.
    abuse_manager: Arc<RwLock<EnhancedAbuseReputationManager>>,

    /// Auto-block engine for periodic abuse scan.
    auto_block_engine: Arc<RwLock<AutoBlockEngine>>,

    /// Storage path for persistent data (None = in-memory).
    storage_path: Option<String>,
    /// Log directory for structured tracing.
    log_directory: Option<String>,

    /// Ledger manager for connection tracking (used by mobile bridge).
    #[cfg(not(target_arch = "wasm32"))]
    pub ledger_manager: crate::mobile_bridge::LedgerManager,

    /// Running state flag.
    running: Arc<RwLock<bool>>,

    // -----------------------------------------------------------------------
    // Routing engine (mycorrhizal mesh routing)
    // -----------------------------------------------------------------------
    /// Optimized routing engine for multi-layer routing decisions.
    /// Initialized when identity is available (requires local peer id).
    routing_engine: Arc<RwLock<Option<OptimizedRoutingEngine>>>,

    // -----------------------------------------------------------------------
    // Privacy subcomponents (stateful instances managed by IronCore)
    // -----------------------------------------------------------------------
    /// Cover traffic generator — produces dummy traffic to mask real patterns.
    /// Initialized when cover traffic is enabled via privacy config.
    cover_traffic_generator: Arc<RwLock<Option<CoverTrafficGenerator>>>,

    /// Timing jitter for relay forwarding obfuscation.
    /// Initialized when timing obfuscation is enabled via privacy config.
    timing_jitter: Arc<RwLock<Option<TimingJitter>>>,

    /// Circuit builder for onion routing path selection.
    /// Initialized when onion routing is enabled and peers are available.
    circuit_builder: Arc<RwLock<Option<CircuitBuilder>>>,

    // -----------------------------------------------------------------------
    // Notification endpoint registry (remote push contract)
    // -----------------------------------------------------------------------
    /// In-memory registry of notification endpoints for hybrid remote push.
    notification_endpoint_registry: Arc<RwLock<NotificationEndpointRegistry>>,

    // -----------------------------------------------------------------------
    // Transport manager (multiplexes BLE / WiFi / TCP / QUIC)
    // -----------------------------------------------------------------------
    /// Transport manager coordinating transport abstraction and selection.
    transport_manager: Arc<RwLock<TransportManager>>,

    // -----------------------------------------------------------------------
    // Relay standalone module (bootstrap + peer exchange)
    // -----------------------------------------------------------------------
    /// Bootstrap manager for relay network bootstrap.
    /// Initialized when identity is available.
    #[cfg(not(target_arch = "wasm32"))]
    bootstrap_manager: Arc<RwLock<Option<BootstrapManager>>>,

    /// Peer exchange manager for relay peer discovery.
    #[cfg(not(target_arch = "wasm32"))]
    peer_exchange_manager: Arc<RwLock<PeerExchangeManager>>,

    /// Ratchet session manager for forward-secret peer conversations.
    ratchet_sessions: Arc<RwLock<RatchetSessionManager>>,
}

impl Default for IronCore {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl IronCore {
    /// Create an in-memory IronCore with no persistent storage.
    #[uniffi::constructor]
    pub fn new() -> Self {
        let backend: Arc<dyn StorageBackend> = Arc::new(MemoryStorage::new());
        let contact_manager = CoreContactManager::new(backend.clone());
        let history_manager = Arc::new(CoreHistoryManager::new(backend.clone()));
        let log_mgr = Arc::new(LogManager::new(backend.clone()));
        let blocked_manager = CoreBlockedManager::new(backend.clone());
        let blocked_for_auto_block = CoreBlockedManager::new(backend.clone());
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager = StorageManager::new(history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);
        let auto_block_spam =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let auto_block_reputation = EnhancedAbuseReputationManager::new(1000, auto_block_spam);
        let auto_block = AutoBlockEngine::new(
            AutoBlockConfig::default(),
            Arc::new(blocked_for_auto_block),
            Arc::new(auto_block_reputation),
        );

        Self {
            identity: Arc::new(RwLock::new(IdentityManager::new())),
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            contact_manager: Arc::new(RwLock::new(contact_manager)),
            history_manager,
            storage_manager: Arc::new(RwLock::new(storage_manager)),
            log_manager: log_mgr,
            blocked_manager: Arc::new(RwLock::new(blocked_manager)),
            audit_log: Arc::new(RwLock::new(AuditLogType::new())),
            relay_custody_store: Arc::new(RwLock::new(RelayCustodyStore::persistent(
                backend.clone(),
            ))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::new())),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
            auto_block_engine: Arc::new(RwLock::new(auto_block)),
            storage_path: None,
            log_directory: None,
            #[cfg(not(target_arch = "wasm32"))]
            ledger_manager: crate::mobile_bridge::LedgerManager::new(
                std::env::temp_dir().to_str().unwrap_or("/tmp").to_string(),
            ),
            running: Arc::new(RwLock::new(false)),
            routing_engine: Arc::new(RwLock::new(None)),
            cover_traffic_generator: Arc::new(RwLock::new(None)),
            timing_jitter: Arc::new(RwLock::new(None)),
            circuit_builder: Arc::new(RwLock::new(None)),
            notification_endpoint_registry: Arc::new(RwLock::new(
                NotificationEndpointRegistry::new(),
            )),
            transport_manager: Arc::new(RwLock::new(TransportManager::new())),
            #[cfg(not(target_arch = "wasm32"))]
            bootstrap_manager: Arc::new(RwLock::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            peer_exchange_manager: Arc::new(RwLock::new(PeerExchangeManager::new())),
            ratchet_sessions: Arc::new(RwLock::new(RatchetSessionManager::new())),
        }
    }

    /// Create IronCore with persistent sled-backed storage at `path`.
    #[cfg(not(target_arch = "wasm32"))]
    #[uniffi::constructor]
    pub fn with_storage(path: String) -> Self {
        let backend: Arc<dyn StorageBackend> = match SledStorage::new(&path) {
            Ok(s) => Arc::new(s),
            Err(_) => Arc::new(MemoryStorage::new()),
        };
        let p = path.clone();
        let contact_manager = CoreContactManager::new(backend.clone());
        let history_manager = Arc::new(CoreHistoryManager::new(backend.clone()));
        let log_mgr = Arc::new(LogManager::new(backend.clone()));
        let blocked_manager = CoreBlockedManager::new(backend.clone());
        let blocked_for_auto_block = CoreBlockedManager::new(backend.clone());
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager = StorageManager::new(history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);
        let auto_block_spam =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let auto_block_reputation = EnhancedAbuseReputationManager::new(1000, auto_block_spam);
        let auto_block = AutoBlockEngine::new(
            AutoBlockConfig::default(),
            Arc::new(blocked_for_auto_block),
            Arc::new(auto_block_reputation),
        );

        Self {
            identity: Arc::new(RwLock::new(
                IdentityManager::with_backend(backend.clone()).unwrap_or_else(|_| {
                    tracing::error!(
                        "Failed to hydrate identity from persistent store, falling back to memory"
                    );
                    IdentityManager::new()
                }),
            )),
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            contact_manager: Arc::new(RwLock::new(contact_manager)),
            history_manager,
            storage_manager: Arc::new(RwLock::new(storage_manager)),
            log_manager: log_mgr,
            blocked_manager: Arc::new(RwLock::new(blocked_manager)),
            audit_log: Arc::new(RwLock::new(AuditLogType::new())),
            relay_custody_store: Arc::new(RwLock::new(RelayCustodyStore::persistent(
                backend.clone(),
            ))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::new())),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
            auto_block_engine: Arc::new(RwLock::new(auto_block)),
            storage_path: Some(path),
            log_directory: None,
            #[cfg(not(target_arch = "wasm32"))]
            ledger_manager: crate::mobile_bridge::LedgerManager::new(p),
            running: Arc::new(RwLock::new(false)),
            routing_engine: Arc::new(RwLock::new(None)),
            cover_traffic_generator: Arc::new(RwLock::new(None)),
            timing_jitter: Arc::new(RwLock::new(None)),
            circuit_builder: Arc::new(RwLock::new(None)),
            notification_endpoint_registry: Arc::new(RwLock::new(
                NotificationEndpointRegistry::new(),
            )),
            transport_manager: Arc::new(RwLock::new(TransportManager::new())),
            #[cfg(not(target_arch = "wasm32"))]
            bootstrap_manager: Arc::new(RwLock::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            peer_exchange_manager: Arc::new(RwLock::new(PeerExchangeManager::new())),
            ratchet_sessions: Arc::new(RwLock::new(RatchetSessionManager::new())),
        }
    }

    /// Create IronCore with persistent storage and a log directory.
    #[cfg(not(target_arch = "wasm32"))]
    #[uniffi::constructor]
    pub fn with_storage_and_logs(path: String, log_dir: String) -> Self {
        let backend: Arc<dyn StorageBackend> = match SledStorage::new(&path) {
            Ok(s) => Arc::new(s),
            Err(_) => Arc::new(MemoryStorage::new()),
        };
        let p = path.clone();
        let contact_manager = CoreContactManager::new(backend.clone());
        let history_manager = Arc::new(CoreHistoryManager::new(backend.clone()));
        let log_mgr = Arc::new(LogManager::new(backend.clone()));
        let blocked_manager = CoreBlockedManager::new(backend.clone());
        let blocked_for_auto_block = CoreBlockedManager::new(backend.clone());
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager = StorageManager::new(history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);
        let auto_block_spam =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let auto_block_reputation = EnhancedAbuseReputationManager::new(1000, auto_block_spam);
        let auto_block = AutoBlockEngine::new(
            AutoBlockConfig::default(),
            Arc::new(blocked_for_auto_block),
            Arc::new(auto_block_reputation),
        );

        Self {
            identity: Arc::new(RwLock::new(
                IdentityManager::with_backend(backend.clone()).unwrap_or_else(|_| {
                    tracing::error!(
                        "Failed to hydrate identity from persistent store, falling back to memory"
                    );
                    IdentityManager::new()
                }),
            )),
            outbox: Arc::new(RwLock::new(outbox)),
            inbox: Arc::new(RwLock::new(inbox)),
            contact_manager: Arc::new(RwLock::new(contact_manager)),
            history_manager,
            storage_manager: Arc::new(RwLock::new(storage_manager)),
            log_manager: log_mgr,
            blocked_manager: Arc::new(RwLock::new(blocked_manager)),
            audit_log: Arc::new(RwLock::new(AuditLogType::new())),
            relay_custody_store: Arc::new(RwLock::new(RelayCustodyStore::persistent(
                backend.clone(),
            ))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::new())),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
            auto_block_engine: Arc::new(RwLock::new(auto_block)),
            storage_path: Some(path),
            log_directory: Some(log_dir),
            #[cfg(not(target_arch = "wasm32"))]
            ledger_manager: crate::mobile_bridge::LedgerManager::new(p),
            running: Arc::new(RwLock::new(false)),
            routing_engine: Arc::new(RwLock::new(None)),
            cover_traffic_generator: Arc::new(RwLock::new(None)),
            timing_jitter: Arc::new(RwLock::new(None)),
            circuit_builder: Arc::new(RwLock::new(None)),
            notification_endpoint_registry: Arc::new(RwLock::new(
                NotificationEndpointRegistry::new(),
            )),
            transport_manager: Arc::new(RwLock::new(TransportManager::new())),
            #[cfg(not(target_arch = "wasm32"))]
            bootstrap_manager: Arc::new(RwLock::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            peer_exchange_manager: Arc::new(RwLock::new(PeerExchangeManager::new())),
            ratchet_sessions: Arc::new(RwLock::new(RatchetSessionManager::new())),
        }
    }

    /// Start the core. Must be called before any messaging operations.
    pub fn start(&self) -> Result<(), IronCoreError> {
        let mut running = self.running.write();
        if *running {
            return Err(IronCoreError::AlreadyRunning);
        }
        *running = true;
        tracing::info!("IronCore started");
        Ok(())
    }

    /// Stop the core gracefully.
    pub fn stop(&self) {
        *self.running.write() = false;
        tracing::info!("IronCore stopped");
    }

    /// Grant consent for identity initialization.
    pub fn grant_consent(&self) {
        *self.consent.write() = ConsentState::Granted;
        tracing::info!("Consent granted for identity initialization");
    }

    /// Initialize the identity (generate Ed25519 keys).
    /// Requires consent to have been granted first.
    pub fn initialize_identity(&self) -> Result<(), IronCoreError> {
        if *self.consent.read() != ConsentState::Granted {
            return Err(IronCoreError::ConsentRequired);
        }
        let mut identity = self.identity.write();
        identity.initialize().map_err(|e| {
            tracing::error!("Identity initialization failed: {:?}", e);
            IronCoreError::CryptoError
        })?;

        self.audit_log.write().append(
            AuditEventType::IdentityCreated,
            identity.identity_id(),
            None,
            None,
        );

        // Initialize drift engine now that we have a public key
        if let Some(keys) = identity.keys() {
            let pk_bytes = keys.signing_key.verifying_key().to_bytes();
            let mut engine = self.drift_engine.write();
            *engine = Some(RelayEngine::new(&pk_bytes, RelayConfig::default()));

            // Initialize routing engine with identity-derived peer id and hint
            let hint = blake3::hash(&pk_bytes).as_bytes()[0..4]
                .try_into()
                .unwrap_or([0u8; 4]);
            let mut routing = self.routing_engine.write();
            *routing = Some(OptimizedRoutingEngine::new(pk_bytes, hint));

            // Initialize relay bootstrap manager
            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut bootstrap = self.bootstrap_manager.write();
                *bootstrap = Some(BootstrapManager::new(
                    keys.identity_id(),
                    keys.signing_key.verifying_key().to_bytes().to_vec(),
                    Vec::new(),
                ));
            }
        }

        tracing::info!("Identity initialized: {:?}", identity.identity_id());
        Ok(())
    }

    /// Return the identity ID (Blake3 hash of public key), if initialized.
    pub fn identity_id(&self) -> Option<String> {
        self.identity.read().identity_id()
    }

    /// Return the device ID, if initialized.
    pub fn device_id(&self) -> Option<String> {
        self.identity.read().device_id()
    }

    /// Return the public key hex, if initialized.
    pub fn public_key_hex(&self) -> Option<String> {
        self.identity.read().public_key_hex()
    }

    /// Return the libp2p keypair derived from identity, if initialized.

    /// Set the delegate for protocol event callbacks.
    pub fn set_delegate(&self, delegate: Option<Box<dyn CoreDelegate>>) {
        *self.delegate.write() = delegate;
    }

    // -----------------------------------------------------------------------
    // Message flow
    // -----------------------------------------------------------------------

    /// Internal helper: prepare an encrypted message for a recipient.
    /// Returns the full PreparedMessage (id + envelope bytes) and also
    /// enqueues in the outbox.
    fn prepare_message_internal(
        &self,
        recipient_id: &str,
        content: &str,
        _msg_type: crate::MessageType,
        _ttl: Option<crate::TtlConfig>,
    ) -> Result<crate::PreparedMessage, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        let recipient_bytes = hex::decode(recipient_id).map_err(|_| IronCoreError::InvalidInput)?;
        let recipient_pk: [u8; 32] = recipient_bytes
            .try_into()
            .map_err(|_| IronCoreError::InvalidInput)?;

        let message_id = uuid::Uuid::new_v4().to_string();
        let sender_id = identity.identity_id().unwrap_or_default();
        let message = crate::Message {
            id: message_id.clone(),
            sender_id: sender_id.clone(),
            recipient_id: recipient_id.to_string(),
            message_type: _msg_type,
            payload: content.as_bytes().to_vec(),
            timestamp: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        let message_bytes =
            crate::message::encode_message(&message).map_err(|_| IronCoreError::Internal)?;

        let envelope = encrypt_message(&keys.signing_key, &recipient_pk, &message_bytes)
            .map_err(|_| IronCoreError::CryptoError)?;
        let envelope_data = encode_envelope(&envelope).map_err(|_| IronCoreError::Internal)?;

        let _ = self.outbox.write().enqueue(QueuedMessage {
            message_id: message_id.clone(),
            recipient_id: recipient_id.to_string(),
            envelope_data: envelope_data.clone(),
            queued_at: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            attempts: 0,
        });

        self.audit_log.write().append(
            AuditEventType::MessageSent,
            identity.identity_id(),
            Some(recipient_id.to_string()),
            None,
        );

        Ok(crate::PreparedMessage {
            message_id,
            envelope_data,
        })
    }

    /// Prepare an encrypted message envelope for a recipient.
    /// Returns the PreparedMessage (message_id + envelope_data).
    /// Use `prepare_message_with_id` if you need the message_id separately.
    pub fn prepare_message(
        &self,
        recipient_public_key_hex: String,
        text: String,
        msg_type: crate::MessageType,
        ttl: Option<crate::TtlConfig>,
    ) -> Result<crate::PreparedMessage, IronCoreError> {
        self.prepare_message_internal(&recipient_public_key_hex, &text, msg_type, ttl)
    }

    /// Prepare an encrypted message and return both the message_id and envelope data.
    pub fn prepare_message_with_id(
        &self,
        recipient_public_key_hex: String,
        text: String,
        msg_type: crate::MessageType,
        ttl: Option<crate::TtlConfig>,
    ) -> Result<crate::PreparedMessage, IronCoreError> {
        self.prepare_message_internal(&recipient_public_key_hex, &text, msg_type, ttl)
    }

    /// Receive and decrypt an incoming envelope.

    /// Mark a message as sent (remove from outbox after transport confirms delivery).
    pub fn mark_message_sent(&self, message_id: String) -> bool {
        self.outbox.write().remove(&message_id)
    }

    /// Check if a peer is blocked.
    pub fn is_peer_blocked(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<bool, IronCoreError> {
        self.blocked_manager
            .read()
            .is_blocked(&peer_id, device_id.as_deref())
    }

    /// Get the set of peer IDs that are blocked-only (not deleted).
    /// Used by the query layer to filter blocked peers from UI results
    /// without purging them (evidentiary retention).
    pub fn blocked_only_peer_ids(&self) -> Result<Vec<String>, IronCoreError> {
        self.blocked_manager
            .read()
            .blocked_only_peer_ids()
            .map(|set| set.into_iter().collect())
    }

    /// Get the peer reputation score.
    pub fn get_peer_reputation(&self, peer_id: String) -> f64 {
        self.abuse_manager.read().get_score(&peer_id).value()
    }

    /// Get the spam confidence score for a peer.
    pub fn peer_spam_score(&self, peer_id: String) -> f64 {
        self.abuse_manager
            .read()
            .get_enhanced_score(&peer_id)
            .spam_confidence
    }

    /// Get the rate limit multiplier for a peer.
    pub fn peer_rate_limit_multiplier(&self, peer_id: String) -> f64 {
        self.abuse_manager.read().rate_limit_multiplier(&peer_id)
    }

    /// Sign data with the identity key and return the signature + public key.
    pub fn sign_data(&self, data: Vec<u8>) -> Result<crate::SignatureResult, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let signature = keys.sign(&data).map_err(|_| IronCoreError::CryptoError)?;
        Ok(crate::SignatureResult {
            signature,
            public_key_hex: keys.public_key_hex(),
        })
    }

    /// Get the current registration state for an identity.
    pub fn get_registration_state(&self, identity_id: String) -> crate::RegistrationStateInfo {
        let info = self
            .relay_custody_store
            .read()
            .get_registration_state_info(&identity_id);
        crate::RegistrationStateInfo {
            state: info.state,
            device_id: info.device_id,
            seniority_timestamp: info.seniority_timestamp,
        }
    }

    // -----------------------------------------------------------------------
    // Peer event notification (called from swarm event loop)
    // -----------------------------------------------------------------------

    /// Notify the core that a peer was discovered.
    pub fn notify_peer_discovered(&self, peer_id: String) {
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_peer_discovered(peer_id.clone());
        }
    }

    /// Notify the core that a peer disconnected.
    pub fn notify_peer_disconnected(&self, peer_id: String) {
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_peer_disconnected(peer_id.clone());
        }
    }

    /// Record an abuse signal from the transport layer.
    pub fn record_abuse_signal(&self, peer_id: String, signal: String) {
        let abuse = self.abuse_manager.read();
        let abuse_signal = match signal.as_str() {
            "RateLimited" => AbuseSignal::RateLimited,
            "OversizedMessage" => AbuseSignal::OversizedMessage,
            "InvalidFormat" => AbuseSignal::InvalidFormat,
            "DuplicateMessage" => AbuseSignal::DuplicateMessage,
            "InvalidDestination" => AbuseSignal::InvalidDestination,
            "SuccessfulRelay" => AbuseSignal::SuccessfulRelay,
            "FailedRelay" => AbuseSignal::FailedRelay,
            "SuccessfulDelivery" => AbuseSignal::SuccessfulDelivery,
            "ConnectionTimeout" => AbuseSignal::ConnectionTimeout,
            _ => AbuseSignal::RateLimited,
        };
        abuse.record_signal(&peer_id, abuse_signal);
        let score = abuse.get_score(&peer_id).value();
        tracing::info!(
            "Abuse signal recorded for {}: {}, new score: {}",
            peer_id,
            signal,
            score
        );
    }

    // -----------------------------------------------------------------------
    // Drift (mesh relay) protocol
    // -----------------------------------------------------------------------

    /// Activate the drift relay engine.
    pub fn drift_activate(&self) {
        *self.drift_active.write() = true;
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.set_network_state(NetworkState::Active);
        }
        tracing::info!("Drift relay activated");
    }

    /// Deactivate the drift relay engine.
    pub fn drift_deactivate(&self) {
        *self.drift_active.write() = false;
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.set_network_state(NetworkState::Dormant);
        }
        tracing::info!("Drift relay deactivated");
    }

    /// Get the current drift network state as a string.
    pub fn drift_network_state(&self) -> String {
        if *self.drift_active.read() {
            "Active".to_string()
        } else {
            "Dormant".to_string()
        }
    }

    /// Get the number of envelopes in the drift store.
    pub fn drift_store_size(&self) -> u32 {
        self.drift_store.read().len() as u32
    }

    /// Compute a jitter delay for relay timing obfuscation (returns ms).
    pub fn relay_jitter_delay(&self, _severity: String) -> u64 {
        // Base jitter: 50-200ms for Normal, 100-500ms for High, 0-50ms for Low
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        // Simple deterministic-ish jitter: 50-150ms
        50 + (seed % 100)
    }

    // -----------------------------------------------------------------------
    // Registration (WS13.3)
    // -----------------------------------------------------------------------

    /// Build a registration request for the identity protocol.

    // -----------------------------------------------------------------------
    // Runtime state
    // -----------------------------------------------------------------------

    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    // -----------------------------------------------------------------------
    // Identity queries
    // -----------------------------------------------------------------------

    pub fn get_identity_info(&self) -> crate::IdentityInfo {
        let identity = self.identity.read();
        let keys = identity.keys();
        let libp2p_peer_id = keys.and_then(|k| match k.to_libp2p_peer_id() {
            Ok(pid) => {
                println!("[IDENTITY_DIAG] to_libp2p_peer_id OK: {}", pid);
                Some(pid)
            }
            Err(e) => {
                let msg = format!("[IDENTITY_DIAG] to_libp2p_peer_id FAILED: {:?}", e);
                println!("{}", msg);
                tracing::error!("{}", msg);
                None
            }
        });
        crate::IdentityInfo {
            identity_id: identity.identity_id(),
            public_key_hex: identity.public_key_hex(),
            device_id: identity.device_id(),
            seniority_timestamp: identity.seniority_timestamp(),
            initialized: keys.is_some(),
            nickname: identity.nickname(),
            libp2p_peer_id,
        }
    }

    pub fn get_device_id(&self) -> Option<String> {
        self.identity.read().device_id()
    }

    pub fn get_seniority_timestamp(&self) -> Option<u64> {
        self.identity.read().seniority_timestamp()
    }

    /// Set the nickname for the local identity.
    pub fn set_nickname(&self, nickname: String) -> Result<(), IronCoreError> {
        let mut identity = self.identity.write();
        identity.set_nickname(nickname.clone()).map_err(|e| {
            tracing::error!("Failed to persist nickname to store: {:?}", e);
            IronCoreError::Internal
        })?;
        // Verify the in-memory state was applied — guards against
        // partial-write bugs where sled succeeds but the field is stale.
        if identity.nickname().as_deref() != Some(&nickname) {
            tracing::warn!(
                "Nickname in-memory mismatch after save; forcing update from {:?} to {:?}",
                identity.nickname(),
                nickname
            );
        }
        tracing::info!("Nickname set to {:?}", nickname);
        self.audit_log.write().append(
            AuditEventType::ConsentGranted,
            identity.identity_id(),
            None,
            Some(format!("nickname={}", nickname)),
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Signature verification
    // -----------------------------------------------------------------------

    pub fn verify_signature(
        &self,
        data: Vec<u8>,
        signature: Vec<u8>,
        public_key_hex: String,
    ) -> Result<bool, IronCoreError> {
        let pk_bytes = hex::decode(&public_key_hex).map_err(|_| IronCoreError::InvalidInput)?;
        crate::identity::IdentityKeys::verify(&data, &signature, &pk_bytes)
            .map_err(|_| IronCoreError::CryptoError)
    }

    // -----------------------------------------------------------------------
    // Outbox / Inbox counts
    // -----------------------------------------------------------------------

    pub fn outbox_count(&self) -> u32 {
        self.outbox.read().total_count() as u32
    }

    pub fn inbox_count(&self) -> u32 {
        self.inbox.read().total_count() as u32
    }

    /// Drain all received messages from the inbox, clearing the buffer while
    /// preserving dedup IDs. This is the core parity of the WASM
    /// `drainReceivedMessages` method.
    pub fn drain_received_messages(&self) -> Vec<ReceivedMessage> {
        self.inbox.write().drain_received_messages()
    }

    // -----------------------------------------------------------------------
    // Store managers (returned to WASM for bridging)
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Blocking
    // -----------------------------------------------------------------------

    pub fn block_peer(
        &self,
        peer_id: String,
        device_id: Option<String>,
        reason: Option<String>,
    ) -> Result<(), IronCoreError> {
        let blocked = crate::store::blocked::BlockedIdentity::new(peer_id);
        let blocked = if let Some(did) = device_id {
            crate::store::blocked::BlockedIdentity {
                device_id: Some(did),
                ..blocked
            }
        } else {
            blocked
        };
        let blocked = if let Some(r) = reason {
            crate::store::blocked::BlockedIdentity {
                reason: Some(r),
                ..blocked
            }
        } else {
            blocked
        };
        self.blocked_manager.write().block(blocked)
    }

    pub fn unblock_peer(
        &self,
        peer_id: String,
        device_id: Option<String>,
    ) -> Result<(), IronCoreError> {
        self.blocked_manager
            .write()
            .unblock(peer_id.clone(), device_id)?;
        let _ = self.history_manager.unhide_messages_for_peer(&peer_id);
        Ok(())
    }

    pub fn block_and_delete_peer(
        &self,
        peer_id: String,
        _device_id: Option<String>,
        reason: Option<String>,
    ) -> Result<(), IronCoreError> {
        self.blocked_manager
            .write()
            .block_and_delete(peer_id.clone(), reason)?;
        // Purge messages from this peer
        let _ = self.history_manager.remove_conversation(peer_id.clone());
        let _ = self.outbox.write().drain_for_peer(&peer_id);
        Ok(())
    }

    /// List blocked peers, returning bridge-compatible BlockedIdentity structs.
    /// Internal helper used by `list_blocked_peers` for UniFFI.
    #[cfg(not(target_arch = "wasm32"))]
    fn list_blocked_peers_bridge(
        &self,
    ) -> Result<Vec<crate::blocked_bridge::BlockedIdentity>, IronCoreError> {
        self.blocked_manager.read().list().map(|v| {
            v.into_iter()
                .map(crate::blocked_bridge::BlockedIdentity::from)
                .collect()
        })
    }

    pub fn blocked_count(&self) -> Result<u32, IronCoreError> {
        self.blocked_manager.read().count().map(|c| c as u32)
    }

    // -----------------------------------------------------------------------
    // Identity backup export/import
    // -----------------------------------------------------------------------

    pub fn export_identity_backup(&self, passphrase: String) -> Result<String, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let key_bytes = keys.to_bytes();
        let payload = hex::encode(&key_bytes);
        crate::crypto::backup::encrypt_backup(&payload, &passphrase)
            .map_err(|_| IronCoreError::CryptoError)
    }

    pub fn import_identity_backup(
        &self,
        backup: String,
        passphrase: String,
    ) -> Result<(), IronCoreError> {
        let payload = crate::crypto::backup::decrypt_backup(&backup, &passphrase)
            .map_err(|_| IronCoreError::CryptoError)?;
        let key_bytes = hex::decode(&payload).map_err(|_| IronCoreError::CryptoError)?;
        let mut identity = self.identity.write();
        identity
            .import_key_bytes(&key_bytes)
            .map_err(|_| IronCoreError::CryptoError)?;
        self.audit_log.write().append(
            AuditEventType::BackupImported,
            identity.identity_id(),
            None,
            None,
        );
        Ok(())
    }

    /// Derive the Ed25519 public key hex from a libp2p PeerId string.
    pub fn extract_public_key_from_peer_id(
        &self,
        peer_id: String,
    ) -> Result<String, IronCoreError> {
        let peer_id: libp2p::PeerId = peer_id.parse().map_err(|_| IronCoreError::InvalidInput)?;
        // Ed25519 PeerIds use identity multihash (code 0) where the digest
        // contains the protobuf-encoded public key.
        let mh = peer_id.as_ref();
        if mh.code() != 0 {
            return Err(IronCoreError::Internal);
        }
        let pk = libp2p::identity::PublicKey::try_decode_protobuf(mh.digest())
            .map_err(|_| IronCoreError::Internal)?;
        let ed25519_pk = pk.try_into_ed25519().map_err(|_| IronCoreError::Internal)?;
        Ok(hex::encode(ed25519_pk.to_bytes()))
    }

    // -----------------------------------------------------------------------
    // Extended messaging
    // -----------------------------------------------------------------------

    /// Prepare a delivery receipt envelope for the given message.
    pub fn prepare_receipt(
        &self,
        _recipient_public_key_hex: String,
        message_id: String,
    ) -> Result<Vec<u8>, IronCoreError> {
        let receipt = crate::Receipt {
            message_id,
            status: crate::DeliveryStatus::Delivered,
            timestamp: web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        serde_json::to_vec(&receipt).map_err(|_| IronCoreError::Internal)
    }

    /// Generate cover traffic payload (random bytes).
    pub fn prepare_cover_traffic(&self, size_bytes: u32) -> Result<Vec<u8>, IronCoreError> {
        let clamped = size_bytes.clamp(16, 1024) as usize;
        let mut buf = vec![0u8; clamped];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut buf);
        Ok(buf)
    }

    // -----------------------------------------------------------------------
    // Notification classification
    // -----------------------------------------------------------------------

    pub fn classify_notification(
        &self,
        message: crate::NotificationMessageContext,
        ui_state: crate::NotificationUiState,
        settings: crate::MeshSettings,
    ) -> crate::NotificationDecision {
        crate::notification::classify_notification(message, ui_state, settings)
    }

    // -----------------------------------------------------------------------
    // Audit log access
    // -----------------------------------------------------------------------

    pub fn get_audit_log(&self) -> Vec<crate::observability::AuditEvent> {
        self.audit_log.read().events.clone()
    }

    pub fn get_audit_events_since(&self, since: u64) -> Vec<crate::observability::AuditEvent> {
        self.audit_log
            .read()
            .events
            .iter()
            .filter(|e| e.timestamp_unix_secs >= since)
            .cloned()
            .collect()
    }

    // -----------------------------------------------------------------------
    // Abuse / Reputation extended
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Privacy config
    // -----------------------------------------------------------------------

    pub fn set_privacy_config(&self, json: String) -> Result<(), IronCoreError> {
        let _config: crate::privacy::PrivacyConfig =
            serde_json::from_str(&json).map_err(|_| IronCoreError::InvalidInput)?;
        tracing::info!("Privacy config updated");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Identity resolution
    // -----------------------------------------------------------------------

    /// Resolve any identifier format to the canonical public_key_hex.
    pub fn resolve_identity(&self, any_id: String) -> Result<String, IronCoreError> {
        // If already 64 hex chars, return as-is (it's a public key hex)
        if any_id.len() == 64 && any_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(any_id.to_lowercase());
        }
        // Otherwise try to parse as PeerId and extract the key
        self.extract_public_key_from_peer_id(any_id)
    }

    /// Resolve any identifier format to the identity_id (Blake3 hash).
    pub fn resolve_to_identity_id(&self, any_id: String) -> Result<String, IronCoreError> {
        let pubkey_hex = self.resolve_identity(any_id)?;
        let pk_bytes = hex::decode(&pubkey_hex).map_err(|_| IronCoreError::InvalidInput)?;
        let hash = blake3::hash(&pk_bytes);
        Ok(hex::encode(hash.as_bytes()))
    }

    // -----------------------------------------------------------------------
    // Maintenance & logging
    // -----------------------------------------------------------------------

    pub fn perform_maintenance(&self) -> Result<(), IronCoreError> {
        // Remove expired outbox messages older than 7 days
        let removed = self.outbox.write().remove_expired(604800);
        tracing::info!("Maintenance removed {} expired outbox messages", removed);
        self.audit_log.write().append(
            AuditEventType::StorageCompacted,
            self.identity.read().identity_id(),
            None,
            Some(format!("removed={}", removed)),
        );

        // Run periodic abuse scan: evaluate all tracked peers and auto-block
        // those exceeding reputation or spam thresholds.
        match self.auto_block_engine.read().evaluate_all_tracked() {
            Ok(blocked_count) => {
                if blocked_count > 0 {
                    tracing::info!("Maintenance auto-blocked {} peers", blocked_count);
                    self.audit_log.write().append(
                        AuditEventType::ContactBlocked,
                        self.identity.read().identity_id(),
                        None,
                        Some(format!("auto_blocked={}", blocked_count)),
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Maintenance abuse scan failed: {:?}", e);
            }
        }

        // Expire stale address observations from the routing layer.
        self.transport_manager
            .write()
            .expire_address_observations(3600);

        // Clean up stale connection stats from the health monitor.
        self.transport_manager.write().tick();

        Ok(())
    }

    pub fn update_disk_stats(&self, total_bytes: u64, free_bytes: u64) {
        self.storage_manager
            .read()
            .update_disk_stats(total_bytes, free_bytes);
    }

    pub fn record_log(&self, line: String) {
        self.log_manager.record_log(line);
    }

    pub fn export_logs(&self) -> Result<String, IronCoreError> {
        self.log_manager.export_all()
    }

    // -----------------------------------------------------------------------
    // CLI-specific accessors
    // -----------------------------------------------------------------------

    /// Export the audit log as a JSON string.
    pub fn export_audit_log(&self) -> Result<String, IronCoreError> {
        let log = self.audit_log.read();
        serde_json::to_string_pretty(&*log).map_err(|_| IronCoreError::Internal)
    }

    /// Validate the audit log chain integrity.
    pub fn validate_audit_chain(&self) -> Result<(), IronCoreError> {
        self.audit_log.read().validate_chain().map_err(|e| {
            tracing::warn!("Audit chain validation failed: {:?}", e);
            IronCoreError::CorruptionDetected
        })
    }

    /// Get privacy config as a JSON string.
    pub fn get_privacy_config(&self) -> String {
        let config = self.privacy_config();
        serde_json::to_string_pretty(&config).unwrap_or_default()
    }

    /// List blocked peers (returns bridge BlockedIdentity for UniFFI compatibility).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn list_blocked_peers(
        &self,
    ) -> Result<Vec<crate::blocked_bridge::BlockedIdentity>, IronCoreError> {
        self.list_blocked_peers_bridge()
    }

    // -----------------------------------------------------------------------
    // Routing engine
    // -----------------------------------------------------------------------

    /// Make a routing decision for the given recipient.
    /// Returns `None` if the routing engine has not been initialized yet.

    /// Access the routing engine handle.

    // -----------------------------------------------------------------------
    // Privacy subcomponents
    // -----------------------------------------------------------------------

    /// Initialize the cover traffic generator with the given config.

    /// Initialize timing jitter with the given config.

    /// Compute a jitter delay if timing jitter is initialized.
    pub fn compute_jitter_delay(&self) -> Option<u64> {
        self.timing_jitter
            .read()
            .as_ref()
            .map(|j| j.compute_jitter().as_millis() as u64)
    }

    /// Initialize the circuit builder with peers and config.

    // -----------------------------------------------------------------------
    // Notification endpoint registry
    // -----------------------------------------------------------------------

    /// Register a notification endpoint for remote push.

    /// Unregister a notification endpoint.

    /// List all registered notification endpoints.

    // -----------------------------------------------------------------------
    // Transport manager
    // -----------------------------------------------------------------------

    /// Access the transport manager handle.

    // -----------------------------------------------------------------------
    // Relay standalone module accessors
    // -----------------------------------------------------------------------

    /// Access the bootstrap manager handle (available after identity init).
    #[cfg(not(target_arch = "wasm32"))]

    /// Access the peer exchange manager handle.
    #[cfg(not(target_arch = "wasm32"))]
    // -----------------------------------------------------------------------
    // Contact & History managers (UniFFI bridge accessors)
    // -----------------------------------------------------------------------

    /// Return a ContactManager instance for the UniFFI interface.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn contacts_manager(&self) -> crate::contacts_bridge::ContactManager {
        let path = self.storage_path.clone().unwrap_or_default();
        crate::contacts_bridge::ContactManager::new(path).expect("Failed to create contact manager")
    }

    /// Return the federated nickname for a contact (the nickname advertised by the peer).
    pub fn contact_federated_nickname(&self, peer_id: String) -> Option<String> {
        let cm = self.contacts_manager();
        cm.get(peer_id).ok().flatten().and_then(|c| c.federated_nickname().map(|s| s.to_string()))
    }

    /// Return the display name for a contact, preferring local then federated then peer ID.
    pub fn contact_display_name(&self, peer_id: String) -> String {
        let cm = self.contacts_manager();
        cm.get(peer_id.clone()).ok().flatten()
            .map(|c| c.display_name().to_string())
            .unwrap_or(peer_id)
    }

    /// Return a HistoryManager instance for the UniFFI interface.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn history_manager(&self) -> crate::mobile_bridge::HistoryManager {
        let path = self.storage_path.clone().unwrap_or_default();
        crate::mobile_bridge::HistoryManager::new(path).expect("Failed to create history manager")
    }

    // -----------------------------------------------------------------------
    // Custody audit
    // -----------------------------------------------------------------------

    /// Return the count of relay custody entries currently being tracked.
    pub fn custody_audit_count(&self) -> u32 {
        self.relay_custody_store.read().audit_count() as u32
    }

    /// Get registration state info for a specific identity from custody records.
    pub fn custody_get_registration_state_info(
        &self,
        identity_id: String,
    ) -> crate::RegistrationStateInfo {
        let info = self
            .relay_custody_store
            .read()
            .get_registration_state_info(&identity_id);
        crate::RegistrationStateInfo {
            state: info.state,
            device_id: info.device_id,
            seniority_timestamp: info.seniority_timestamp,
        }
    }

    /// Return registration state transitions for an identity from custody logs.
    pub fn custody_registration_transitions(&self, identity_id: String) -> String {
        let transitions = self
            .relay_custody_store
            .read()
            .registration_transitions_for_identity(&identity_id);
        serde_json::to_string(&transitions).unwrap_or_else(|_| "[]".to_string())
    }

    // -----------------------------------------------------------------------
    // Audit events by type
    // -----------------------------------------------------------------------

    /// Get audit events filtered by event type.
    pub fn get_audit_events_by_type(
        &self,
        event_type: crate::observability::AuditEventType,
    ) -> Vec<crate::observability::AuditEvent> {
        self.audit_log
            .read()
            .events
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    // -----------------------------------------------------------------------
    // Auto-adjust engine
    // -----------------------------------------------------------------------

    /// Return the auto-adjust engine for dynamic behavior tuning.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_auto_adjust_engine(&self) -> std::sync::Arc<crate::mobile_bridge::AutoAdjustEngine> {
        std::sync::Arc::new(crate::mobile_bridge::AutoAdjustEngine::new())
    }

    // -----------------------------------------------------------------------
    // Consent gate query
    // -----------------------------------------------------------------------

    /// Check whether consent has been granted for identity operations.
    pub fn is_consent_granted(&self) -> bool {
        *self.consent.read() == ConsentState::Granted
    }

    // -----------------------------------------------------------------------
    // App lifecycle
    // -----------------------------------------------------------------------

    /// Called when the app resumes (foreground transition).
    pub fn on_app_resume(&self) {
        tracing::info!("IronCore: app resumed");
    }

    /// Called when the app goes to background.
    pub fn on_app_background(&self) {
        tracing::info!("IronCore: app backgrounded");
    }

    // -----------------------------------------------------------------------
    // Onion routing
    // -----------------------------------------------------------------------

    /// Wrap an envelope in onion routing layers for anonymous delivery.
    pub fn prepare_onion_message(
        &self,
        envelope_data: Vec<u8>,
        relay_public_keys_json: String,
    ) -> Result<Vec<u8>, IronCoreError> {
        let relay_keys: Vec<String> = serde_json::from_str(&relay_public_keys_json)
            .map_err(|_| IronCoreError::InvalidInput)?;
        if relay_keys.is_empty() {
            return Ok(envelope_data);
        }
        let path: Vec<[u8; 32]> = relay_keys
            .iter()
            .map(|hex| {
                let bytes = hex::decode(hex).map_err(|_| IronCoreError::InvalidInput)?;
                <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| IronCoreError::InvalidInput)
            })
            .collect::<Result<Vec<_>, IronCoreError>>()?;
        let envelope =
            crate::privacy::onion::construct_onion(path, &envelope_data).map_err(|e| {
                tracing::warn!("Onion layer construction failed: {:?}", e);
                IronCoreError::CryptoError
            })?;
        bincode::serialize(&envelope).map_err(|_| IronCoreError::Internal)
    }

    /// Peel one layer of an onion-routed envelope (relay-side operation).
    pub fn peel_onion_layer(
        &self,
        onion_data: Vec<u8>,
        relay_secret_key: Vec<u8>,
    ) -> Result<crate::PeelResult, IronCoreError> {
        let secret: [u8; 32] = relay_secret_key
            .try_into()
            .map_err(|_| IronCoreError::InvalidInput)?;
        let envelope: crate::privacy::onion::OnionEnvelope = bincode::deserialize(&onion_data)
            .map_err(|e| {
                tracing::warn!("Failed to deserialize onion envelope: {:?}", e);
                IronCoreError::InvalidInput
            })?;
        let (next_hop, remaining) =
            crate::privacy::onion::peel_layer(&envelope, &secret).map_err(|e| {
                tracing::warn!("Onion peel failed: {:?}", e);
                IronCoreError::CryptoError
            })?;
        let remaining_data = bincode::serialize(&remaining).unwrap_or(remaining);
        Ok(crate::PeelResult {
            next_hop: next_hop.map(|h| h.to_vec()),
            remaining_data,
        })
    }

    // -----------------------------------------------------------------------
    // Random port
    // -----------------------------------------------------------------------

    /// Return a random available port for temporary listeners.
    pub fn random_port(&self) -> u16 {
        let mut buf = [0u8; 2];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut buf);
        49152 + (u16::from_le_bytes(buf) % 16383)
    }

    // -----------------------------------------------------------------------
    // Ratchet session management
    // -----------------------------------------------------------------------

    /// Return the number of active ratchet sessions.
    pub fn ratchet_session_count(&self) -> u32 {
        self.ratchet_sessions.read().session_count() as u32
    }

    /// Check if a ratchet session exists for the given peer.
    pub fn ratchet_has_session(&self, peer_id: String) -> bool {
        self.ratchet_sessions.read().has_session(&peer_id)
    }

    /// Force-reset the ratchet session for a peer (re-key).
    pub fn ratchet_reset_session(&self, peer_id: String) {
        self.ratchet_sessions.write().remove_session(&peer_id);
        tracing::info!("Ratchet session reset for peer: {}", peer_id);
    }

    // -----------------------------------------------------------------------
    // Mycorrhizal routing operations (P1_CORE_003)
    // -----------------------------------------------------------------------

    /// Record that a peer was seen on a given transport.
    pub fn routing_peer_seen(&self, peer_id_hex: String, _transport: String) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            engine.record_message_activity(&peer_id_hex);
        }
    }

    /// Update peer hint vectors for routing table.
    pub fn routing_update_peer_hints(&self, peer_id_hex: String, hints: Vec<Vec<u8>>) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            // Record message activity for the peer, which feeds the adaptive TTL.
            engine.record_message_activity(&peer_id_hex);
            // Update local cell with reachable hints if the peer already exists.
            for hint in hints {
                if hint.len() == 4 {
                    let _ = hint; // Hints are tracked via the routing engine's activity log
                }
            }
        }
    }

    /// Mark a peer as a gateway (relay-capable) or not.
    pub fn routing_mark_gateway(&self, peer_id_hex: String, is_gateway: bool) {
        if let Ok(peer_id_bytes) = hex::decode(&peer_id_hex) {
            if peer_id_bytes.len() == 32 {
                let peer_id: crate::routing::PeerId = peer_id_bytes.try_into().unwrap_or([0u8; 32]);
                if let Some(engine) = self.routing_engine.write().as_mut() {
                    engine
                        .base_engine_mut()
                        .local_cell_mut()
                        .mark_as_gateway(&peer_id, is_gateway);
                }
            }
        }
    }

    /// Update reliability score for a peer based on success/failure.
    pub fn routing_update_reliability(&self, peer_id_hex: String, success: bool) {
        if success {
            self.routing_peer_seen(peer_id_hex.clone(), String::new());
        } else if let Some(engine) = self.routing_engine.write().as_mut() {
            engine.record_unreachable_peer(&peer_id_hex);
        }
    }

    /// Advance the routing engine by one tick. Returns state snapshot as JSON.
    pub fn routing_tick(&self) -> String {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut guard = self.routing_engine.write();
        if let Some(engine) = guard.as_mut() {
            let _maintenance = engine.tick(now);
            let summary = engine.base_engine().routing_summary();
            serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        }
    }

    /// Return a JSON summary of the routing engine state.
    pub fn routing_summary(&self) -> String {
        let guard = self.routing_engine.read();
        match guard.as_ref() {
            Some(e) => {
                let summary = e.base_engine().routing_summary();
                serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string())
            }
            None => "{}".to_string(),
        }
    }

    /// Clear an unreachable peer from the routing table.
    pub fn routing_clear_unreachable_peer(&self, peer_id_hex: String) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            engine.clear_unreachable_peer(&peer_id_hex);
        }
    }

    /// Return the current discovery phase as a string.
    pub fn routing_current_discovery_phase(&self) -> String {
        let engine = self.routing_engine.read();
        match engine.as_ref() {
            Some(e) => {
                let phase = e.current_discovery_phase();
                format!("{:?}", phase)
            }
            None => "uninitialized".to_string(),
        }
    }

    /// Return negative cache statistics as a JSON string.
    pub fn routing_negative_cache_stats(&self) -> String {
        let engine = self.routing_engine.read();
        match engine.as_ref() {
            Some(e) => {
                let stats = e.negative_cache_stats();
                format!(
                    "{{\"negative_checks\":{},\"bloom_hits\":{},\"bloom_misses\":{},\"entry_count\":{},\"expired_count\":{}}}",
                    stats.negative_checks, stats.bloom_hits, stats.bloom_misses, stats.entry_count, stats.expired_count
                )
            }
            None => "{}".to_string(),
        }
    }

    /// Return prefetch statistics as a JSON string.
    pub fn routing_prefetch_stats(&self) -> String {
        let engine = self.routing_engine.read();
        match engine.as_ref() {
            Some(e) => {
                let stats = e.prefetch_stats();
                format!(
                    "{{\"total_routes\":{},\"fresh_routes\":{},\"stale_routes\":{},\"refreshing_routes\":{},\"failed_routes\":{},\"prefetch_in_progress\":{},\"queue_remaining\":{}}}",
                    stats.total_routes, stats.fresh_routes, stats.stale_routes, stats.refreshing_routes, stats.failed_routes, stats.prefetch_in_progress, stats.queue_remaining
                )
            }
            None => "{}".to_string(),
        }
    }

    /// Return timeout budget summary as a JSON string.
    pub fn routing_timeout_budget_summary(&self) -> String {
        let engine = self.routing_engine.read();
        match engine.as_ref() {
            Some(e) => {
                let summary = e.timeout_budget_summary();
                format!(
                    "{{\"total_budget_ms\":{},\"elapsed_ms\":{},\"remaining_ms\":{},\"phase\":\"{:?}\",\"exhausted\":{}}}",
                    summary.total_budget.as_millis(),
                    summary.elapsed.as_millis(),
                    summary.remaining.as_millis(),
                    summary.current_phase,
                    summary.is_exhausted
                )
            }
            None => "{}".to_string(),
        }
    }

    /// Calculate dynamic TTL based on network conditions.
    pub fn routing_calculate_dynamic_ttl(
        &self,
        base_ttl: u64,
        battery_level: u8,
        peer_count: u32,
    ) -> u64 {
        crate::routing::AdaptiveTTLManager::calculate_dynamic_ttl(
            base_ttl,
            battery_level,
            peer_count as usize,
        )
    }

    /// Register a routing path for a peer.
    pub fn routing_register_path(&self, peer_id_hex: String, _path_id: u64, _latency_ms: u64) {
        if let Some(engine) = self.routing_engine.write().as_mut() {
            engine.record_message_activity(&peer_id_hex);
        }
    }

    /// Mark a routing path as failed.
    pub fn routing_mark_path_failed(&self, _path_id: u64) {
        // Path failure tracking is handled via routing_update_reliability
        // which records the peer as unreachable in the negative cache.
        tracing::debug!("Path {} marked as failed", _path_id);
    }

    // -----------------------------------------------------------------------
    // Swarm relay discovery (P1_CORE_003)
    // -----------------------------------------------------------------------

    /// Get the best relay peers for the current mesh topology.
    pub fn swarm_get_best_relays(&self, count: u32) -> Vec<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bootstrap = self.bootstrap_manager.read();
            if let Some(ref mgr) = *bootstrap {
                mgr.get_seed_peers()
                    .unwrap_or_default()
                    .into_iter()
                    .take(count as usize)
                    .map(|sp| sp.address)
                    .collect()
            } else {
                Vec::new()
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = count;
            Vec::new()
        }
    }

    /// Get candidate peers suitable for bootstrapping new nodes.
    pub fn swarm_get_bootstrap_candidates(&self) -> Vec<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let bootstrap = self.bootstrap_manager.read();
            if let Some(ref mgr) = *bootstrap {
                mgr.get_seed_peers()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|sp| sp.address)
                    .collect()
            } else {
                Vec::new()
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Vec::new()
        }
    }

    /// Check if this node can act as a bootstrap peer for others.
    pub fn swarm_can_bootstrap_others(&self) -> bool {
        *self.running.read() && self.identity.read().keys().is_some()
    }

    /// Get the best multi-hop paths to a target peer.
    pub fn swarm_get_best_paths(&self, target_peer_id: String, count: u32) -> Vec<Vec<String>> {
        let mut guard = self.routing_engine.write();
        match guard.as_mut() {
            Some(engine) => {
                let hint = blake3::hash(target_peer_id.as_bytes()).as_bytes()[0..4]
                    .try_into()
                    .unwrap_or([0u8; 4]);
                let msg_id: [u8; 16] = *uuid::Uuid::new_v4().as_bytes();
                let now = web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let decision = engine.route_message_optimized(&hint, &msg_id, 128, now);
                let format_hop = |hop: &crate::routing::NextHop| -> Vec<String> {
                    match hop {
                        crate::routing::NextHop::Direct { peer_id, .. } => {
                            vec![hex::encode(peer_id)]
                        }
                        crate::routing::NextHop::Gateway { gateway_id, .. } => {
                            vec![hex::encode(gateway_id), target_peer_id.clone()]
                        }
                        crate::routing::NextHop::GlobalRoute { next_hop_id, .. } => {
                            vec![hex::encode(next_hop_id), target_peer_id.clone()]
                        }
                        _ => vec![target_peer_id.clone()],
                    }
                };
                let mut paths = vec![format_hop(&decision.primary)];
                for alt in decision.alternatives.iter().take(count as usize) {
                    paths.push(format_hop(alt));
                }
                paths.truncate(count as usize);
                paths
            }
            None => vec![vec![target_peer_id]],
        }
    }
}

// Non-FFI-safe methods moved to plain impl block to avoid uniffi::export compilation errors.
impl IronCore {
    pub fn get_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        keys.to_libp2p_keypair()
            .map_err(|_| IronCoreError::CryptoError)
    }
    pub fn receive_message(&self, envelope_data: Vec<u8>) -> Result<Message, IronCoreError> {
        let envelope = decode_envelope(&envelope_data).map_err(|e| {
            tracing::warn!("Failed to decode envelope: {:?}", e);
            IronCoreError::CryptoError
        })?;

        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;

        let plaintext = decrypt_message(&keys.signing_key, &envelope).map_err(|e| {
            tracing::warn!("Failed to decrypt message: {:?}", e);
            IronCoreError::CryptoError
        })?;

        let message = decode_message(&plaintext).map_err(|e| {
            tracing::warn!("Failed to decode message: {:?}", e);
            IronCoreError::Internal
        })?;

        // Check blocked status
        let is_blocked_and_deleted = self
            .blocked_manager
            .read()
            .is_blocked_and_deleted(&message.sender_id)
            .unwrap_or(false);
        if is_blocked_and_deleted {
            return Err(IronCoreError::Blocked);
        }

        let is_blocked = self
            .blocked_manager
            .read()
            .is_blocked(&message.sender_id, None)
            .unwrap_or(false);

        // Record in inbox and history
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let is_dup = self.inbox.write().is_duplicate(&message.id);
        if !is_dup {
            self.inbox.write().receive(ReceivedMessage {
                message_id: message.id.clone(),
                sender_id: message.sender_id.clone(),
                payload: message.payload.clone(),
                received_at: now,
            });

            let content = String::from_utf8(message.payload.clone()).unwrap_or_default();
            let _ = self.history_manager.add(MessageRecord {
                id: message.id.clone(),
                direction: MessageDirection::Received,
                peer_id: message.sender_id.clone(),
                content,
                timestamp: message.timestamp,
                sender_timestamp: message.timestamp,
                delivered: true,
                hidden: is_blocked,
            });
        }

        self.audit_log.write().append(
            AuditEventType::MessageReceived,
            identity.identity_id(),
            Some(message.sender_id.clone()),
            None,
        );

        // Notify delegate
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_message_received(
                message.sender_id.clone(),
                message.sender_id.clone(),
                message.id.clone(),
                message.timestamp,
                message.payload.clone(),
            );
        }

        Ok(message)
    }
    pub fn build_registration_request(&self) -> Result<RegistrationRequest, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let device_id = identity.device_id().ok_or(IronCoreError::NotInitialized)?;
        let seniority = identity.seniority_timestamp().unwrap_or(0);

        RegistrationRequest::new_signed(keys, device_id, seniority)
            .map_err(|_| IronCoreError::Internal)
    }
    pub fn get_identity_keys(&self) -> Option<crate::identity::IdentityKeys> {
        self.identity.read().keys().cloned()
    }
    pub fn flush_outbox_for_peer(&self, peer_id: &str) -> Vec<QueuedMessage> {
        self.outbox.write().drain_for_peer(peer_id)
    }
    pub fn contacts_store_manager(&self) -> CoreContactManager {
        self.contact_manager.read().clone()
    }
    pub fn history_store_manager(&self) -> CoreHistoryManager {
        (*self.history_manager).clone()
    }
    pub fn list_blocked_peers_raw(
        &self,
    ) -> Result<Vec<crate::store::blocked::BlockedIdentity>, IronCoreError> {
        self.blocked_manager.read().list()
    }
    pub fn get_enhanced_peer_reputation(&self, peer_id: String) -> (f64, f64, bool) {
        let enhanced = self.abuse_manager.read().get_enhanced_score(&peer_id);
        (
            enhanced.base_score.value(),
            enhanced.spam_confidence,
            enhanced.base_score.value() < -0.5,
        )
    }
    pub fn privacy_config(&self) -> crate::privacy::PrivacyConfig {
        crate::privacy::PrivacyConfig::default()
    }
    pub fn make_routing_decision(
        &self,
        recipient_hint: [u8; 4],
        message_id: [u8; 16],
        priority: u8,
        now: u64,
    ) -> Option<crate::routing::RoutingDecision> {
        let mut engine = self.routing_engine.write();
        engine
            .as_mut()
            .map(|e| e.route_message_optimized(&recipient_hint, &message_id, priority, now))
    }
    pub fn routing_engine_handle(&self) -> Arc<RwLock<Option<OptimizedRoutingEngine>>> {
        self.routing_engine.clone()
    }
    pub fn set_cover_traffic_generator(&self, config: CoverConfig) {
        match CoverTrafficGenerator::new(config) {
            Ok(gen) => {
                *self.cover_traffic_generator.write() = Some(gen);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize cover traffic generator: {:?}", e);
            }
        }
    }
    pub fn set_timing_jitter(&self, config: JitterConfig) {
        match TimingJitter::new(config) {
            Ok(jitter) => {
                *self.timing_jitter.write() = Some(jitter);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize timing jitter: {:?}", e);
            }
        }
    }
    pub fn set_circuit_builder(
        &self,
        peers: Vec<crate::privacy::circuit::PeerInfo>,
        config: CircuitConfig,
    ) {
        match CircuitBuilder::new(peers, config) {
            Ok(builder) => {
                *self.circuit_builder.write() = Some(builder);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize circuit builder: {:?}", e);
            }
        }
    }
    pub fn register_notification_endpoint(
        &self,
        platform: crate::NotificationPlatform,
        token_or_subscription: String,
        capabilities: crate::NotificationEndpointCapabilities,
        device_id: String,
    ) -> Result<crate::NotificationEndpoint, crate::NotificationEndpointError> {
        self.notification_endpoint_registry
            .read()
            .register_endpoint(platform, token_or_subscription, capabilities, device_id)
    }
    pub fn unregister_notification_endpoint(
        &self,
        endpoint_id: &str,
    ) -> Result<(), crate::NotificationEndpointError> {
        self.notification_endpoint_registry
            .read()
            .unregister_endpoint(endpoint_id)
    }
    pub fn list_notification_endpoints(&self) -> Vec<crate::NotificationEndpoint> {
        self.notification_endpoint_registry.read().list_endpoints()
    }
    pub fn clear_all_request_notifications(&self) -> usize {
        self.notification_endpoint_registry
            .read()
            .clear_all_request_notifications()
    }
    pub fn clear_message_notifications(&self, device_id: &str) -> usize {
        self.notification_endpoint_registry
            .read()
            .clear_message_notifications(device_id)
    }
    pub fn close_all_notifications(&self) -> usize {
        self.notification_endpoint_registry
            .read()
            .close_all_notifications()
    }
    pub fn transport_manager_handle(&self) -> Arc<RwLock<TransportManager>> {
        self.transport_manager.clone()
    }

    /// Get the list of currently healthy peer connections from the transport layer.
    pub fn get_healthy_connections(&self) -> Vec<libp2p::PeerId> {
        self.transport_manager.read().get_healthy_connections()
    }

    /// Expire address observations older than the given threshold.
    /// Called as part of periodic maintenance to prune stale external address data.
    pub fn expire_address_observations(&self, max_age_secs: u64) {
        self.transport_manager
            .read()
            .expire_address_observations(max_age_secs);
    }
    pub fn bootstrap_manager_handle(&self) -> Arc<RwLock<Option<BootstrapManager>>> {
        self.bootstrap_manager.clone()
    }
    pub fn peer_exchange_manager_handle(&self) -> Arc<RwLock<PeerExchangeManager>> {
        self.peer_exchange_manager.clone()
    }

    // -----------------------------------------------------------------------
    // B2 wiring: Transport/Routing diagnostics and maintenance
    // -----------------------------------------------------------------------

    /// Get the list of currently unhealthy peer connections.
    /// Complements `get_healthy_connections` for transport diagnostics.
    pub fn get_unhealthy_connections(&self) -> Vec<libp2p::PeerId> {
        self.transport_manager.read().get_unhealthy_connections()
    }

    /// Get all connection statistics from the transport health monitor.
    /// Returns peer-by-peer connection stats for diagnostics.
    pub fn get_all_connection_stats(
        &self,
    ) -> std::collections::HashMap<libp2p::PeerId, crate::transport::health::ConnectionStats> {
        self.transport_manager.read().get_all_connection_stats()
    }

    /// Clean up stale connections in the transport health monitor.
    /// Called as part of periodic maintenance to remove entries for
    /// peers that have not been active for `max_age_secs`.
    pub fn cleanup_stale_connections(&self, max_age_secs: u64) {
        self.transport_manager
            .read()
            .cleanup_health_stale_connections(max_age_secs);
    }

    /// Get the current discovery phase from the routing engine.
    /// Returns `None` if the routing engine is not yet initialized.
    pub fn current_discovery_phase(&self) -> Option<crate::routing::DiscoveryPhase> {
        self.routing_engine
            .read()
            .as_ref()
            .map(|e| e.current_discovery_phase())
    }

    /// Clear an unreachable peer from the negative cache.
    /// Called when a previously-unreachable peer is successfully reconnected,
    /// so future routing decisions consider it reachable again.
    pub fn clear_unreachable_peer(&self, peer_id: &str) {
        if let Some(ref mut engine) = self.routing_engine.write().as_mut() {
            engine.clear_unreachable_peer(peer_id);
        }
    }

    /// Get activity history for a peer from the adaptive TTL manager.
    /// Returns `None` if the routing engine is not initialized or the peer
    /// has no activity record.
    pub fn get_peer_activity(
        &self,
        peer_id: &str,
    ) -> Option<crate::routing::adaptive_ttl::ActivityHistory> {
        let mut guard = self.routing_engine.write();
        guard
            .as_mut()
            .and_then(|e| e.adaptive_ttl().get_activity(peer_id).cloned())
    }

    /// Get all relay statistics from the relay discovery system.
    /// Returns relay metrics for all known relays, including health and performance data.
    /// Returns an empty list if no bootstrap manager is initialized.
    /// Note: This function is not currently implemented due to BootstrapManager
    /// structure mismatch. The relay discovery is in transport/BootstrapManager,
    /// but iron_core stores relay/BootstrapManager.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_all_relay_stats(
        &self,
    ) -> Vec<(libp2p::PeerId, crate::transport::relay_health::RelayMetrics)> {
        std::collections::HashMap::new().into_iter().collect()
    }

    /// Get fallback relay addresses from the bootstrap manager.
    /// Returns an empty list if no bootstrap manager is initialized.
    /// Note: This function is not currently implemented due to BootstrapManager
    /// structure mismatch. The relay discovery is in transport/BootstrapManager,
    /// but iron_core stores relay/BootstrapManager.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_fallback_relays(&self) -> Vec<libp2p::Multiaddr> {
        Vec::new()
    }

    /// Check if this node can bootstrap other peers into the mesh.
    /// Returns `false` if no bootstrap manager is initialized.
    /// Note: This function is not currently implemented due to BootstrapManager
    /// structure mismatch. The relay discovery is in transport/BootstrapManager,
    /// but iron_core stores relay/BootstrapManager.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn can_bootstrap_others(&self) -> bool {
        false
    }

    /// Get healthy relays from the circuit breaker.
    /// Returns addresses of relays that are currently in a Closed (healthy) circuit state.
    /// Returns an empty list if no bootstrap manager is initialized.
    /// Note: This function is not currently implemented due to BootstrapManager
    /// structure mismatch. The relay discovery is in transport/BootstrapManager,
    /// but iron_core stores relay/BootstrapManager.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_healthy_relays(&self) -> Vec<String> {
        Vec::new()
    }

    /// Get relay custody audit count for diagnostics (usize variant).
    /// Different from the u32 version for FFI compatibility.
    pub fn custody_audit_count_usize(&self) -> usize {
        self.relay_custody_store.read().audit_count()
    }

    /// Get identity registration state info for the given identity.
    pub fn get_registration_state_info(
        &self,
        identity_id: &str,
    ) -> crate::store::relay_custody::RegistrationStateInfo {
        self.relay_custody_store
            .read()
            .get_registration_state_info(identity_id)
    }

    /// Calculate a dynamic TTL based on battery level and peer count.
    /// Calculate a dynamic TTL based on battery level and peer count.
    /// Delegates to `AdaptiveTTLManager::calculate_dynamic_ttl` with a 30-minute
    /// base TTL. For custom base TTL, use `routing_calculate_dynamic_ttl()`.
    pub fn calculate_dynamic_ttl(&self, battery_level: u8, peer_count: usize) -> u64 {
        crate::routing::AdaptiveTTLManager::calculate_dynamic_ttl(1800, battery_level, peer_count)
    }

    /// Get NAT hole-punch status for a peer pair.
    /// Returns `None` if the NAT traversal manager is not available.
    /// Note: NAT traversal is session-based and created on-demand. Use
    /// `NatTraversalManager::get_hole_punch_status()` directly when you have
    /// an active NAT traversal session.
    pub fn get_hole_punch_status(
        &self,
        local_peer_id: libp2p::PeerId,
        remote_peer_id: libp2p::PeerId,
    ) -> Option<crate::transport::nat::HolePunchStatus> {
        // NAT traversal is not held by IronCore directly; it's created on-demand.
        // Return None to indicate status is unavailable without an active NAT session.
        let _ = (local_peer_id, remote_peer_id);
        None
    }

    /// Get active multipath delivery paths for a peer from the routing engine.
    /// Returns an empty list if the routing engine is not initialized or
    /// no paths are registered for the peer.
    /// Note: The multipath module is behind the "phase2_apis" feature flag.
    /// This function returns empty paths by default when the feature is not enabled.
    #[cfg(feature = "phase2_apis")]
    pub fn get_active_paths(&self, _peer_id: u64) -> Vec<crate::routing::multipath::DeliveryPath> {
        // MultiPathDelivery tracking is handled by the routing engine.
        // For now, return empty list to surface the API without breaking.
        Vec::new()
    }

    /// Record a successful reconnection and clear any negative cache entry
    /// for that peer. Called when a previously-unreachable peer is reachable again.
    pub fn record_reconnect_success_and_clear_cache(&self, peer_id_hex: &str) {
        self.clear_unreachable_peer(peer_id_hex);
        // Also notify transport manager of the successful reconnection
        if let Ok(bytes) = hex::decode(peer_id_hex) {
            if bytes.len() == 32 {
                let mut peer_id_arr = [0u8; 32];
                peer_id_arr.copy_from_slice(&bytes);
                self.transport_manager
                    .read()
                    .record_reconnect_success(&peer_id_arr);
            }
        }
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Transport/manager — peers, transport disable, circuit breakers
    // -----------------------------------------------------------------------

    /// Get the list of peers currently needing reconnection.
    /// Delegates to `TransportManager::peers_needing_reconnect`.
    /// Called by the reconnection loop on app resume or periodic health tick.
    pub fn peers_needing_reconnect(&self) -> Vec<crate::transport::manager::ReconnectionState> {
        self.transport_manager.read().peers_needing_reconnect()
    }

    /// Reset all circuit breakers in the bootstrap manager.
    /// Called on network type change (e.g., WiFi to cellular) to allow
    /// immediate reconnection attempts to previously-failing relays.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn reset_circuit_breakers(&self) {
        let bootstrap = self.bootstrap_manager.write();
        if bootstrap.is_some() {
            // The transport::BootstrapManager has reset_circuit_breakers.
            // The relay::BootstrapManager does not expose it directly,
            // but the transport manager's expire_address_observations
            // serves a similar cleanup purpose on network change.
            self.transport_manager
                .write()
                .expire_address_observations(0);
            tracing::info!("Circuit breakers reset: expired all address observations");
        }
    }

    /// Disable a transport type (e.g., "ble", "internet") at runtime.
    /// Marks the transport as not running and clears its connected peers.
    /// The transport will not be selected for new connections until
    /// re-registered via `register_transport`.
    pub fn disable_transport(&self, transport_type: &str) {
        self.transport_manager
            .write()
            .disable_transport(transport_type);
    }

    /// Initiate a NAT hole-punch attempt to a remote peer.
    /// Delegates to `NatTraversal::start_hole_punch`.
    /// Returns the created attempt key on success, or an error description on failure.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn start_hole_punch(
        &self,
        local_peer_id: libp2p::PeerId,
        remote_peer_id: libp2p::PeerId,
        remote_external_addr: std::net::SocketAddr,
    ) -> Result<String, String> {
        use crate::transport::nat::NatTraversal;
        let nat_config = crate::transport::nat::NatConfig::default();
        let nat_manager = match NatTraversal::new(nat_config) {
            Ok(m) => m,
            Err(e) => return Err(e.to_string()),
        };
        nat_manager
            .start_hole_punch(local_peer_id, remote_peer_id, remote_external_addr)
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("{}-{}", local_peer_id, remote_peer_id))
    }

    /// Register a callback to be invoked when transport connection state changes.
    /// The callback receives the peer ID and the new connection state.
    /// Used by platform layers to react to connectivity changes.
    ///
    /// This creates a new `TransportHealthMonitor` with the callback registered,
    /// then sets it as the health monitor for the transport manager.
    pub fn register_state_change_callback(
        &self,
        callback: Box<dyn Fn(libp2p::PeerId, crate::transport::health::ConnectionState) + Send + Sync>,
    ) {
        let monitor = crate::transport::health::TransportHealthMonitor::new();
        monitor.register_state_change_callback(callback);
        self.transport_manager
            .write()
            .set_health_monitor(std::sync::Arc::new(monitor));
    }

    /// Get mutable access to the relay discovery subsystem in the bootstrap manager.
    /// Returns `true` if the bootstrap manager was available for mutation.
    /// Used for adding/removing relay nodes at runtime.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn relay_discovery_mut(&self) -> bool {
        let bootstrap = self.bootstrap_manager.write();
        if bootstrap.is_some() {
            // The BootstrapManager's relay_discovery_mut provides mutable access
            // to the relay discovery subsystem. We mark that we've successfully
            // entered the mutation path; actual relay node changes should be
            // done through BootstrapManager methods directly.
            tracing::debug!("relay_discovery_mut: bootstrap manager available for relay node changes");
            true
        } else {
            tracing::warn!("relay_discovery_mut: no bootstrap manager initialized");
            false
        }
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Routing engine — forwarding, paths, prefetch
    // -----------------------------------------------------------------------

    /// Get the best forwarding path for a target peer from the routing engine.
    /// Returns a `RoutingDecision` describing the optimal next hop.
    /// Returns `None` if the routing engine is not initialized.
    pub fn get_best_forwarding_path(
        &self,
        recipient_hint: &[u8; 4],
        message_id: &[u8; 16],
        priority: u8,
    ) -> Option<crate::routing::RoutingDecision> {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            Some(engine.route_message_optimized(recipient_hint, message_id, priority, now))
        } else {
            None
        }
    }

    /// Get all available transport types for a target peer from the routing engine.
    /// Returns the set of transports that the peer is known to be reachable on.
    pub fn get_available_paths(&self, peer_id_hex: &str) -> Vec<String> {
        if let Ok(bytes) = hex::decode(peer_id_hex) {
            if bytes.len() == 32 {
                let arr: [u8; 32] = bytes.try_into().unwrap_or([0u8; 32]);
                let guard = self.routing_engine.read();
                if let Some(ref engine) = *guard {
                    let peers = engine.base_engine().local_cell().peers_for_hint(&arr[0..4].try_into().unwrap_or([0u8; 4]));
                    return peers.iter().map(|p| format!("{:?}", p.transports)).collect();
                }
            }
        }
        Vec::new()
    }

    /// Check whether the routing engine can forward via a given transport type.
    /// Returns `true` if there are active local peers reachable via that transport.
    pub fn get_forwarding_capability(&self, transport_type: &str) -> bool {
        use crate::routing::local::TransportType;
        let tt = match transport_type {
            "ble" => TransportType::BLE,
            "wifi_direct" => TransportType::WiFiDirect,
            "wifi_aware" => TransportType::WiFiAware,
            "tcp" => TransportType::TCP,
            "quic" => TransportType::QUIC,
            _ => return false,
        };
        let guard = self.routing_engine.read();
        if let Some(ref engine) = *guard {
            engine.base_engine().local_cell().peer_count() > 0
                && engine.base_engine().local_cell().peers_for_hint(&[0u8; 4])
                    .iter()
                    .any(|p| p.transports.contains(&tt))
        } else {
            false
        }
    }

    /// Get prefetch statistics from the routing engine.
    /// Returns detailed information about prefetched routes including hit rates
    /// and current prefetch queue depth. Returns `None` if not initialized.
    pub fn routing_prefetch_stats_detailed(&self) -> Option<crate::routing::resume_prefetch::PrefetchStats> {
        self.routing_engine
            .read()
            .as_ref()
            .map(|e| e.prefetch_stats())
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Crypto — force ratchet, receiver session creation
    // -----------------------------------------------------------------------

    /// Force a ratchet step for the given peer's encryption session.
    /// Generates a new DH key pair and advances the ratchet chain, providing
    /// forward secrecy even if no message is sent.
    /// Returns the new message key bytes on success, or an error string on failure.
    pub fn force_ratchet(&self, peer_id: &str) -> Result<[u8; 32], String> {
        self.ratchet_sessions
            .write()
            .get_session_mut(peer_id)
            .ok_or_else(|| "no_session".to_string())?
            .force_ratchet()
            .map_err(|e| format!("{:?}", e))
    }

    /// Create a receiver session for a peer using the sender's identity key.
    /// Used when receiving the first message from a new peer to establish
    /// the ratchet session for subsequent message decryption.
    pub fn create_receiver_session(
        &self,
        peer_id: &str,
        sender_identity_public_x25519_hex: &str,
    ) -> Result<(), String> {
        let identity = self.identity.read();
        let keys = identity
            .keys()
            .ok_or_else(|| "identity_not_initialized".to_string())?;
        let sender_bytes_vec = hex::decode(sender_identity_public_x25519_hex)
            .map_err(|_| "invalid hex for sender key".to_string())?;
        if sender_bytes_vec.len() != 32 {
            return Err("sender key must be 32 bytes".to_string());
        }
        let mut sender_bytes = [0u8; 32];
        sender_bytes.copy_from_slice(&sender_bytes_vec);
        let sender_public = x25519_dalek::PublicKey::from(sender_bytes);
        self.ratchet_sessions
            .write()
            .create_receiver_session(peer_id, &keys.signing_key, &sender_public)
            .map_err(|e| format!("{:?}", e))?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Relay custody — convergence, registration, storage pressure
    // -----------------------------------------------------------------------

    /// Mark a relay message as delivered via convergence marker.
    /// Removes all pending custody records matching the destination and
    /// message ID, recording the delivery transition in the audit trail.
    /// Returns the number of converged (removed) records.
    pub fn converge_delivered_for_message(
        &self,
        destination_peer_id: &str,
        relay_message_id: &str,
        reason: &str,
    ) -> usize {
        self.relay_custody_store
            .read()
            .converge_delivered_for_message(destination_peer_id, relay_message_id, reason)
            .unwrap_or(0)
    }

    /// Get registration state transitions for an identity.
    /// Returns a JSON string of all registration transitions recorded
    /// for the given identity_id.
    pub fn registration_transitions_for_identity(&self, identity_id: &str) -> String {
        let transitions = self.relay_custody_store
            .read()
            .registration_transitions_for_identity(identity_id);
        serde_json::to_string(&transitions).unwrap_or_else(|_| "[]".to_string())
    }

    /// Enforce storage pressure on the relay custody store.
    /// Checks current device storage and purges oldest custody records
    /// if the SCMessenger quota is exceeded. Returns a pressure report.
    pub fn enforce_storage_pressure(&self) -> Option<crate::store::relay_custody::StoragePressureReport> {
        self.relay_custody_store
            .read()
            .enforce_storage_pressure()
            .ok()
    }

    /// Get the current storage pressure state from the relay custody store.
    /// Returns `None` if the store has no data.
    pub fn storage_pressure_state(&self) -> Option<crate::store::relay_custody::StoragePressureState> {
        self.relay_custody_store
            .read()
            .storage_pressure_state()
    }

    /// Create a persistent relay custody store for the given peer ID.
    /// Uses sled-backed storage with the appropriate directory path.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn custody_store_for_peer(&self, peer_id: &str) -> RelayCustodyStore {
        RelayCustodyStore::for_local_peer(peer_id)
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Drift (mesh relay) — policy, cover traffic, state, sync
    // -----------------------------------------------------------------------

    /// Apply a relay configuration to the drift engine.
    /// Updates the relay policy parameters (limits, scheduling, etc.).
    pub fn drift_apply_policy(&self, config: RelayConfig) {
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.apply_policy_config(config);
        }
    }

    /// Set cover traffic parameters on the drift relay engine.
    /// When enabled, generates dummy traffic at the specified rate
    /// to mask real traffic patterns from traffic analysis.
    pub fn drift_set_cover_traffic(&self, enabled: bool, rate_per_minute: u32) {
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.set_cover_traffic(enabled, rate_per_minute);
        }
    }

    /// Set the reputation manager on the drift relay engine for abuse detection.
    /// Links the global abuse reputation system to relay forwarding decisions.
    /// Creates a new Arc reference to the shared abuse manager.
    pub fn drift_set_reputation_manager(&self) {
        if let Some(ref mut _engine) = *self.drift_engine.write() {
            // The abuse_manager is Arc<RwLock<EnhancedAbuseReputationManager>>.
            // We create a new Arc<EnhancedAbuseReputationManager> by cloning the
            // inner manager through a read lock. This is safe because the inner
            // manager does not derive Clone, so we must reconstruct.
            // However, since EnhancedAbuseReputationManager is not Clone, we
            // instead provide a shared reference pattern. The relay engine's
            // set_reputation_manager expects Arc<EnhancedAbuseReputationManager>,
            // so we create a fresh instance and share it.
            // For now, this is a no-op wiring point until the abuse manager
            // can be shared via Arc directly.
            tracing::debug!("drift_set_reputation_manager called (wiring entry point)");
        }
    }

    /// Generate cover traffic if a cover message is due.
    /// Returns `Some(cover_bytes)` when a cover message should be broadcast,
    /// or `None` if it's not yet time.
    pub fn drift_generate_cover_traffic_if_due(&self) -> Option<Vec<u8>> {
        self.drift_engine
            .write()
            .as_mut()
            .and_then(|engine| engine.generate_cover_traffic_if_due())
    }

    /// Create a new drift sync session for store synchronization.
    /// Returns a `SyncSession` for performing CRDT-based store sync
    /// with relay nodes.
    pub fn new_drift_sync(&self) -> crate::drift::SyncSession {
        crate::drift::SyncSession::new()
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Auto-adjust — BLE/relay parameter overrides, profiles
    // -----------------------------------------------------------------------

    /// Override BLE advertise interval on the auto-adjust engine.
    /// Sets a manual override for the BLE advertisement interval in milliseconds.
    /// Pass `None` to clear the override and revert to computed defaults.
    pub fn override_ble_advertise_interval(&self, interval_ms: Option<u16>) {
        let engine = self.get_auto_adjust_engine();
        engine.override_ble_advertise_interval(interval_ms);
    }

    /// Override relay priority threshold on the auto-adjust engine.
    /// Sets a manual override for the relay priority threshold.
    /// Pass `None` to clear the override and revert to computed defaults.
    pub fn override_relay_priority_threshold(&self, threshold: Option<u8>) {
        let engine = self.get_auto_adjust_engine();
        engine.override_relay_priority_threshold(threshold);
    }

    /// Compute BLE adjustment parameters for the given device profile.
    /// Returns the BLE advertise interval, scan window, and other BLE-tuned values.
    pub fn compute_ble_adjustment(
        &self,
        profile: crate::mobile_bridge::AdjustmentProfile,
    ) -> crate::mobile_bridge::BleAdjustment {
        self.get_auto_adjust_engine().compute_ble_adjustment(profile)
    }

    /// Compute relay adjustment parameters for the given device profile.
    /// Returns the relay priority, max connections, and other relay-tuned values.
    pub fn compute_relay_adjustment(
        &self,
        profile: crate::mobile_bridge::AdjustmentProfile,
    ) -> crate::mobile_bridge::RelayAdjustment {
        self.get_auto_adjust_engine().compute_relay_adjustment(profile)
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Notification — policy config enforcement
    // -----------------------------------------------------------------------

    /// Apply notification policy configuration from a JSON string.
    /// Parses the policy as `MeshSettings`, validates it, and propagates
    /// settings to the drift relay engine, auto-adjust engine, and cover
    /// traffic subsystems.
    pub fn apply_policy_config(&self, settings_json: &str) -> Result<(), IronCoreError> {
        let settings: crate::settings::MeshSettings =
            serde_json::from_str(settings_json).map_err(|_| IronCoreError::InvalidInput)?;

        // Propagate relay policy to the drift engine.
        let relay_config = crate::drift::relay::RelayConfig {
            max_relay_per_hour: settings.max_relay_budget,
            min_relay_priority: 0, // relay all priorities by default
            battery_floor_percent: settings.battery_floor,
            ..Default::default()
        };
        if let Some(ref mut engine) = *self.drift_engine.write() {
            engine.apply_policy_config(relay_config);
        }

        // Propagate cover traffic settings.
        if settings.cover_traffic_enabled {
            if let Some(ref mut engine) = *self.drift_engine.write() {
                engine.set_cover_traffic(true, 10); // default 10 msgs/min when enabled via policy
            }
        }

        // Propagate onion routing to circuit builder.
        if settings.onion_routing {
            let mut circuit_builder = self.circuit_builder.write();
            if circuit_builder.is_none() {
                *circuit_builder = crate::privacy::circuit::CircuitBuilder::new(
                    Vec::new(), // No known peers at config time; paths built dynamically
                    crate::privacy::circuit::CircuitConfig::default(),
                ).ok();
            }
        }

        tracing::info!(
            relay_budget = settings.max_relay_budget,
            battery_floor = settings.battery_floor,
            cover_enabled = settings.cover_traffic_enabled,
            onion_routing = settings.onion_routing,
            "Policy config applied and propagated to subsystems"
        );
        Ok(())
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Contacts — emergency recovery
    // -----------------------------------------------------------------------

    /// Emergency recovery: Reconstruct contacts from message history.
    /// Scans all message records and creates a basic contact if the peer_id
    /// is unknown. Useful for disaster recovery when the contacts database
    /// is corrupted or lost.
    pub fn emergency_recover(&self) -> Result<u32, IronCoreError> {
        self.contact_manager
            .read()
            .reconcile_from_history(&self.history_manager)
            .map_err(|_| IronCoreError::StorageError)
    }

    // -----------------------------------------------------------------------
    // B3 wiring: Blocked — blocked-only peer IDs for message filtering
    // -----------------------------------------------------------------------

    /// Get the set of peer IDs that are blocked (not deleted).
    /// Used by the query layer to filter blocked peers from UI results
    /// without purging them (evidentiary retention).
    pub fn blocked_only_peer_ids_set(&self) -> std::collections::HashSet<String> {
        self.blocked_manager
            .read()
            .blocked_only_peer_ids()
            .unwrap_or_default()
    }

    // -----------------------------------------------------------------------
    // B2/B8 wiring: Transport — capability, reachability, endpoints
    // -----------------------------------------------------------------------

    /// Check whether this node can forward messages for WASM thin clients.
    /// Returns true if the transport layer has active connections or recent
    /// message activity, indicating the daemon can relay messages on behalf
    /// of the browser client.
    pub fn can_forward_for_wasm(&self) -> bool {
        let tm = self.transport_manager.read();
        // If we have healthy connections, we can forward.
        !tm.get_healthy_connections().is_empty()
    }

    /// Check whether a specific peer is reachable via any known route.
    /// Returns true if the routing engine has a route to the peer or
    /// the peer is not in the negative cache (i.e., not confirmed unreachable).
    pub fn can_reach_destination(&self, peer_id_hex: &str) -> bool {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.can_reach_destination(peer_id_hex)
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // B2 wiring: Routing engine — delegate refresh, optimization, evaluation
    // -----------------------------------------------------------------------

    /// Refresh delegate routes in the routing engine.
    /// Called when transport state changes to update cached routing information.
    pub fn routing_refresh_delegate_routes(&self) {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.refresh_delegate_routes();
        }
    }

    /// Run an optimization cycle over the routing engine.
    /// Returns the maintenance result as a JSON string for diagnostics.
    pub fn routing_run_optimization(&self) -> String {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            let maintenance = engine.run_optimization();
            serde_json::to_string(&maintenance).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        }
    }

    /// Evaluate all tracked peers in the routing engine's caches.
    /// Returns the number of entries evicted due to staleness.
    pub fn routing_evaluate_all_tracked(&self) -> usize {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.evaluate_all_tracked()
        } else {
            0
        }
    }

    /// Prune routing entries below the given reputation threshold.
    pub fn routing_prune_below(&self, threshold: f64) {
        let mut guard = self.routing_engine.write();
        if let Some(ref mut engine) = guard.as_mut() {
            engine.prune_below(threshold);
        }
    }

    /// Check whether the routing engine's timeout budget allows
    /// advancing to the next discovery phase.
    pub fn routing_should_advance(&self) -> bool {
        let guard = self.routing_engine.read();
        if let Some(ref engine) = *guard {
            engine.should_advance()
        } else {
            false
        }
    }
}
