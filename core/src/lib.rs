// Iron Core V2 — Messaging Spine
#![allow(clippy::empty_line_after_doc_comments)]
//
// "Does this help two humans exchange an encrypted message
//  without any corporation in the middle?"
//
// If the answer is no, it doesn't belong in Phase 0.

pub mod abuse;
pub mod crypto;
pub mod drift;
pub mod identity;
pub mod iron_core;
pub mod message;
pub mod notification;
pub mod notification_defaults;
pub mod observability;
pub mod privacy;
pub mod routing;
pub mod settings;
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

use observability::{AuditEvent, AuditEventType};

// IronCoreError — defined in Rust rather than generated from UDL
// because the UDL-based scaffolding requires interface types to have
// matching Rust implementations (which are wired in Phase 1B).
#[derive(Debug, thiserror::Error)]
pub enum IronCoreError {
    #[error("Identity not initialized")]
    NotInitialized,
    #[error("Service already running")]
    AlreadyRunning,
    #[error("Storage error")]
    StorageError,
    #[error("Cryptographic error")]
    CryptoError,
    #[error("Network error")]
    NetworkError,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Peer is blocked")]
    Blocked,
    #[error("Consent required")]
    ConsentRequired,
    #[error("Internal error")]
    Internal,
    #[error("Data corruption detected")]
    CorruptionDetected,
}

pub use crypto::{decrypt_message, encrypt_message};
pub use identity::IdentityManager;
pub use message::{DeliveryStatus, Envelope, Message, MessageType, Receipt, TtlConfig};
pub use notification::{
    classify_notification as classify_notification_policy, NotificationDecision,
    NotificationEndpoint, NotificationEndpointCapabilities, NotificationEndpointError,
    NotificationEndpointRegistry, NotificationKind, NotificationMessageContext,
    NotificationPlatform, NotificationUiState,
};
pub use settings::{DiscoveryMode, MeshSettings};

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

// =====================================================================

pub use iron_core::{CoreDelegate, IronCore};

#[derive(Default)]
pub struct IdentityInfo {
    pub identity_id: Option<String>,
    pub public_key_hex: Option<String>,
    pub device_id: Option<String>,
    pub seniority_timestamp: Option<u64>,
    pub initialized: bool,
    pub nickname: Option<String>,
    pub libp2p_peer_id: Option<String>,
}

pub struct SignatureResult {
    pub signature: Vec<u8>,
    pub public_key_hex: String,
}

pub struct PreparedMessage {
    pub message_id: String,
    pub envelope_data: Vec<u8>,
}

pub struct PeelResult {
    pub next_hop: Option<Vec<u8>>,
    pub remaining_data: Vec<u8>,
}

pub struct RegistrationStateInfo {
    pub state: String,
    pub device_id: Option<String>,
    pub seniority_timestamp: Option<u64>,
}
