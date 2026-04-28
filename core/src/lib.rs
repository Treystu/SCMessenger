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
pub mod message;
pub mod notification;
pub mod notification_defaults;
pub mod observability;
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

use observability::{AuditEvent, AuditEventType, AuditLog as AuditLogType};

pub use crypto::{decrypt_message, encrypt_message};
pub use identity::IdentityManager;
pub use message::{DeliveryStatus, Envelope, Message, MessageType, Receipt, TtlConfig};
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

// =====================================================================
