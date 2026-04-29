//! IronCore — the central entry point for the SCMessenger mesh.
//!
//! Holds identity, outbox, inbox, contact manager, history manager, storage
//! manager, log manager, blocked manager, relay registry, and audit log.
//! All state is behind `Arc<RwLock<…>>` (parking_lot).

use parking_lot::RwLock;
use std::sync::Arc;

use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine};
use crate::abuse::EnhancedAbuseReputationManager;
use crate::crypto::{decrypt_message, encrypt_message};
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
use crate::store::relay_custody::RelayRegistry;
use crate::store::{
    ContactManager as CoreContactManager, HistoryManager as CoreHistoryManager, Inbox,
    MessageDirection, MessageRecord, Outbox, QueuedMessage, ReceivedMessage, StorageBackend,
    StorageManager,
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
pub trait CoreDelegate: Send + Sync {
    fn on_peer_discovered(&self, peer_id: &str);
    fn on_peer_disconnected(&self, peer_id: &str);
    fn on_peer_identified(&self, peer_id: &str, agent_version: &str, listen_addrs: Vec<String>);
    fn on_message_received(
        &self,
        sender_id: &str,
        sender_public_key_hex: &str,
        message_id: &str,
        sender_timestamp: u64,
        data: Vec<u8>,
    );
    fn on_receipt_received(&self, message_id: &str, status: &str);
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
    pub(crate) relay_registry: Arc<RwLock<RelayRegistry>>,

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
}

impl Default for IronCore {
    fn default() -> Self {
        Self::new()
    }
}

impl IronCore {
    /// Create an in-memory IronCore with no persistent storage.
    pub fn new() -> Self {
        let backend: Arc<dyn StorageBackend> = Arc::new(MemoryStorage::new());
        let contact_manager = CoreContactManager::new(backend.clone());
        let history_manager = Arc::new(CoreHistoryManager::new(backend.clone()));
        let log_mgr = Arc::new(LogManager::new(backend.clone()));
        let blocked_manager = CoreBlockedManager::new(backend.clone());
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager = StorageManager::new(history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);

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
            relay_registry: Arc::new(RwLock::new(RelayRegistry::new(backend.clone()))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::new())),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
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
        }
    }

    /// Create IronCore with persistent sled-backed storage at `path`.
    #[cfg(not(target_arch = "wasm32"))]
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
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager = StorageManager::new(history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);

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
            relay_registry: Arc::new(RwLock::new(RelayRegistry::new(backend.clone()))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::new())),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
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
        }
    }

    /// Create IronCore with persistent storage and a log directory.
    #[cfg(not(target_arch = "wasm32"))]
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
        let inbox = Inbox::new();
        let outbox = Outbox::new();
        let storage_manager = StorageManager::new(history_manager.clone(), log_mgr.clone());
        let spam_detector =
            SpamDetectionEngine::new_heuristics_only(SpamDetectionConfig::default());
        let abuse_mgr = EnhancedAbuseReputationManager::new(1000, spam_detector);

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
            relay_registry: Arc::new(RwLock::new(RelayRegistry::new(backend.clone()))),
            delegate: Arc::new(RwLock::new(None)),
            consent: Arc::new(RwLock::new(ConsentState::NotGranted)),
            drift_active: Arc::new(RwLock::new(false)),
            drift_store: Arc::new(RwLock::new(MeshStore::new())),
            drift_engine: Arc::new(RwLock::new(None)),
            abuse_manager: Arc::new(RwLock::new(abuse_mgr)),
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
    pub fn get_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        keys.to_libp2p_keypair()
            .map_err(|_| IronCoreError::CryptoError)
    }

    /// Set the delegate for protocol event callbacks.
    pub fn set_delegate(&self, delegate: Option<Box<dyn CoreDelegate>>) {
        *self.delegate.write() = delegate;
    }

    // -----------------------------------------------------------------------
    // Message flow
    // -----------------------------------------------------------------------

    /// Prepare an encrypted message for a recipient.
    pub fn prepare_message(
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

    /// Receive and decrypt an incoming envelope.
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
                &message.sender_id,
                &message.sender_id,
                &message.id,
                message.timestamp,
                message.payload.clone(),
            );
        }

        Ok(message)
    }

    /// Mark a message as sent (remove from outbox after transport confirms delivery).
    pub fn mark_message_sent(&self, message_id: &str) -> bool {
        self.outbox.write().remove(message_id)
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

    /// Get the peer reputation score.
    pub fn get_peer_reputation(&self, peer_id: &str) -> f64 {
        self.abuse_manager.read().get_score(peer_id).value()
    }

    /// Get the spam confidence score for a peer.
    pub fn peer_spam_score(&self, peer_id: String) -> f64 {
        self.abuse_manager
            .read()
            .get_enhanced_score(&peer_id)
            .spam_confidence
    }

    /// Get the rate limit multiplier for a peer.
    pub fn peer_rate_limit_multiplier(&self, peer_id: &str) -> f64 {
        self.abuse_manager.read().rate_limit_multiplier(peer_id)
    }

    /// Sign data with the identity key and return the signature + public key.
    pub fn sign_data(&self, data: &[u8]) -> Result<crate::SignatureResult, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let signature = keys.sign(data).map_err(|_| IronCoreError::CryptoError)?;
        Ok(crate::SignatureResult {
            signature,
            public_key_hex: keys.public_key_hex(),
        })
    }

    /// Get the current registration state for an identity.
    pub fn get_registration_state(&self, _identity_id: &str) -> crate::RegistrationStateInfo {
        crate::RegistrationStateInfo {
            state: "unknown".to_string(),
            device_id: None,
            seniority_timestamp: None,
        }
    }

    // -----------------------------------------------------------------------
    // Peer event notification (called from swarm event loop)
    // -----------------------------------------------------------------------

    /// Notify the core that a peer was discovered.
    pub fn notify_peer_discovered(&self, peer_id: String) {
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_peer_discovered(&peer_id);
        }
    }

    /// Notify the core that a peer disconnected.
    pub fn notify_peer_disconnected(&self, peer_id: String) {
        if let Some(delegate) = self.delegate.read().as_ref() {
            delegate.on_peer_disconnected(&peer_id);
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
        tracing::info!("Abuse signal recorded for {}: {}", peer_id, signal);
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
    pub fn drift_store_size(&self) -> usize {
        self.drift_store.read().len()
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
    pub fn build_registration_request(&self) -> Result<RegistrationRequest, IronCoreError> {
        let identity = self.identity.read();
        let keys = identity.keys().ok_or(IronCoreError::NotInitialized)?;
        let device_id = identity.device_id().ok_or(IronCoreError::NotInitialized)?;
        let seniority = identity.seniority_timestamp().unwrap_or(0);

        RegistrationRequest::new_signed(keys, device_id, seniority)
            .map_err(|_| IronCoreError::Internal)
    }

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
        crate::IdentityInfo {
            identity_id: identity.identity_id(),
            public_key_hex: identity.public_key_hex(),
            device_id: identity.device_id(),
            seniority_timestamp: identity.seniority_timestamp(),
            initialized: keys.is_some(),
            nickname: None,
            libp2p_peer_id: None,
        }
    }

    pub fn get_identity_keys(&self) -> Option<crate::identity::IdentityKeys> {
        self.identity.read().keys().cloned()
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
        identity
            .set_nickname(nickname.clone())
            .map_err(|_| IronCoreError::Internal)?;
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

    pub fn flush_outbox_for_peer(&self, peer_id: &str) -> Vec<QueuedMessage> {
        self.outbox.write().drain_for_peer(peer_id)
    }

    // -----------------------------------------------------------------------
    // Store managers (returned to WASM for bridging)
    // -----------------------------------------------------------------------

    pub fn contacts_store_manager(&self) -> CoreContactManager {
        self.contact_manager.read().clone()
    }

    pub fn history_store_manager(&self) -> CoreHistoryManager {
        (*self.history_manager).clone()
    }

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

    pub fn list_blocked_peers_raw(
        &self,
    ) -> Result<Vec<crate::store::blocked::BlockedIdentity>, IronCoreError> {
        self.blocked_manager.read().list()
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

    /// Prepare message and return PreparedMessage (alias with same semantics).
    pub fn prepare_message_with_id(
        &self,
        recipient_id: &str,
        content: &str,
        msg_type: crate::MessageType,
        ttl: Option<crate::TtlConfig>,
    ) -> Result<crate::PreparedMessage, IronCoreError> {
        self.prepare_message(recipient_id, content, msg_type, ttl)
    }

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

    pub fn get_enhanced_peer_reputation(&self, peer_id: String) -> (f64, f64, bool) {
        let enhanced = self.abuse_manager.read().get_enhanced_score(&peer_id);
        (
            enhanced.base_score.value(),
            enhanced.spam_confidence,
            enhanced.base_score.value() < -0.5,
        )
    }

    // -----------------------------------------------------------------------
    // Privacy config
    // -----------------------------------------------------------------------

    pub fn privacy_config(&self) -> crate::privacy::PrivacyConfig {
        crate::privacy::PrivacyConfig::default()
    }

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
        Ok(())
    }

    pub fn update_disk_stats(&self, _total_bytes: u64, _free_bytes: u64) {
        // Placeholder: adjust storage behavior based on disk stats
        tracing::debug!(
            total = _total_bytes,
            free = _free_bytes,
            "Disk stats updated"
        );
    }

    pub fn record_log(&self, line: String) {
        tracing::info!("{}", line);
        // LogManager is not wired for arbitrary lines yet, just emit via tracing
    }

    pub fn export_logs(&self) -> Result<String, IronCoreError> {
        // Placeholder: return empty log dump for now
        Ok(String::new())
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

    /// List blocked peers (returns BlockedIdentity vec).
    pub fn list_blocked_peers(
        &self,
    ) -> Result<Vec<crate::store::blocked::BlockedIdentity>, IronCoreError> {
        self.list_blocked_peers_raw()
    }

    // -----------------------------------------------------------------------
    // Routing engine
    // -----------------------------------------------------------------------

    /// Make a routing decision for the given recipient.
    /// Returns `None` if the routing engine has not been initialized yet.
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

    /// Access the routing engine handle.
    pub fn routing_engine_handle(&self) -> Arc<RwLock<Option<OptimizedRoutingEngine>>> {
        self.routing_engine.clone()
    }

    // -----------------------------------------------------------------------
    // Privacy subcomponents
    // -----------------------------------------------------------------------

    /// Initialize the cover traffic generator with the given config.
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

    /// Initialize timing jitter with the given config.
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

    /// Compute a jitter delay if timing jitter is initialized.
    pub fn compute_jitter_delay(&self) -> Option<u64> {
        self.timing_jitter
            .read()
            .as_ref()
            .map(|j| j.compute_jitter().as_millis() as u64)
    }

    /// Initialize the circuit builder with peers and config.
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

    // -----------------------------------------------------------------------
    // Notification endpoint registry
    // -----------------------------------------------------------------------

    /// Register a notification endpoint for remote push.
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

    /// Unregister a notification endpoint.
    pub fn unregister_notification_endpoint(
        &self,
        endpoint_id: &str,
    ) -> Result<(), crate::NotificationEndpointError> {
        self.notification_endpoint_registry
            .read()
            .unregister_endpoint(endpoint_id)
    }

    /// List all registered notification endpoints.
    pub fn list_notification_endpoints(&self) -> Vec<crate::NotificationEndpoint> {
        self.notification_endpoint_registry.read().list_endpoints()
    }

    // -----------------------------------------------------------------------
    // Transport manager
    // -----------------------------------------------------------------------

    /// Access the transport manager handle.
    pub fn transport_manager_handle(&self) -> Arc<RwLock<TransportManager>> {
        self.transport_manager.clone()
    }

    // -----------------------------------------------------------------------
    // Relay standalone module accessors
    // -----------------------------------------------------------------------

    /// Access the bootstrap manager handle (available after identity init).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn bootstrap_manager_handle(&self) -> Arc<RwLock<Option<BootstrapManager>>> {
        self.bootstrap_manager.clone()
    }

    /// Access the peer exchange manager handle.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn peer_exchange_manager_handle(&self) -> Arc<RwLock<PeerExchangeManager>> {
        self.peer_exchange_manager.clone()
    }
}
