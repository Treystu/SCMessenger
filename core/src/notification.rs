use crate::MeshSettings;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationKind {
    DirectMessage,
    DirectMessageRequest,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessageContext {
    pub conversation_id: Option<String>,
    pub sender_peer_id: String,
    pub message_id: String,
    pub explicit_dm_request: Option<bool>,
    pub sender_is_known_contact: bool,
    pub has_existing_conversation: bool,
    pub is_self_originated: bool,
    pub is_duplicate: bool,
    pub already_seen: bool,
    pub is_blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationUiState {
    pub app_in_foreground: bool,
    pub active_conversation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDecision {
    pub kind: NotificationKind,
    pub conversation_id: String,
    pub sender_peer_id: String,
    pub message_id: String,
    pub should_alert: bool,
    pub suppression_reason: Option<String>,
}

impl NotificationDecision {
    fn suppressed(
        conversation_id: String,
        sender_peer_id: String,
        message_id: String,
        reason: &str,
    ) -> Self {
        Self {
            kind: NotificationKind::None,
            conversation_id,
            sender_peer_id,
            message_id,
            should_alert: false,
            suppression_reason: Some(reason.to_string()),
        }
    }

    fn allow(
        kind: NotificationKind,
        conversation_id: String,
        sender_peer_id: String,
        message_id: String,
    ) -> Self {
        Self {
            kind,
            conversation_id,
            sender_peer_id,
            message_id,
            should_alert: true,
            suppression_reason: None,
        }
    }
}

pub fn classify_notification(
    message: NotificationMessageContext,
    ui_state: NotificationUiState,
    settings: MeshSettings,
) -> NotificationDecision {
    let sender_peer_id = normalize_required(message.sender_peer_id);
    let message_id = normalize_required(message.message_id);
    let conversation_id = message
        .conversation_id
        .and_then(normalize_optional)
        .unwrap_or_else(|| sender_peer_id.clone());

    if sender_peer_id.is_empty() || message_id.is_empty() {
        return NotificationDecision::suppressed(
            conversation_id,
            sender_peer_id,
            message_id,
            "invalid_notification_metadata",
        );
    }

    if !settings.notifications_enabled {
        return NotificationDecision::suppressed(
            conversation_id,
            sender_peer_id,
            message_id,
            "notifications_disabled",
        );
    }

    if message.is_self_originated {
        return NotificationDecision::suppressed(
            conversation_id,
            sender_peer_id,
            message_id,
            "self_originated",
        );
    }

    if message.is_duplicate {
        return NotificationDecision::suppressed(
            conversation_id,
            sender_peer_id,
            message_id,
            "duplicate_message",
        );
    }

    if message.already_seen {
        return NotificationDecision::suppressed(
            conversation_id,
            sender_peer_id,
            message_id,
            "already_seen",
        );
    }

    if message.is_blocked {
        return NotificationDecision::suppressed(
            conversation_id,
            sender_peer_id,
            message_id,
            "sender_blocked",
        );
    }

    let explicit_request = message.explicit_dm_request.unwrap_or(false);
    let kind = if explicit_request {
        NotificationKind::DirectMessageRequest
    } else if message.sender_is_known_contact || message.has_existing_conversation {
        NotificationKind::DirectMessage
    } else {
        NotificationKind::DirectMessageRequest
    };

    match kind {
        NotificationKind::DirectMessage if !settings.notify_dm_enabled => {
            return NotificationDecision::suppressed(
                conversation_id,
                sender_peer_id,
                message_id,
                "direct_message_notifications_disabled",
            );
        }
        NotificationKind::DirectMessageRequest if !settings.notify_dm_request_enabled => {
            return NotificationDecision::suppressed(
                conversation_id,
                sender_peer_id,
                message_id,
                "direct_message_request_notifications_disabled",
            );
        }
        _ => {}
    }

    let active_conversation_id = ui_state.active_conversation_id.and_then(normalize_optional);
    let active_match = ui_state.app_in_foreground
        && active_conversation_id
            .as_ref()
            .map(|active| ids_match(active, &conversation_id))
            .unwrap_or(false);
    if active_match {
        let allow_foreground = match kind {
            NotificationKind::DirectMessage => settings.notify_dm_in_foreground,
            NotificationKind::DirectMessageRequest => settings.notify_dm_request_in_foreground,
            NotificationKind::None => false,
        };

        if !allow_foreground {
            return NotificationDecision::suppressed(
                conversation_id,
                sender_peer_id,
                message_id,
                "foreground_conversation_active",
            );
        }
    }

    NotificationDecision::allow(kind, conversation_id, sender_peer_id, message_id)
}

fn normalize_required(value: String) -> String {
    value.trim().to_string()
}

fn normalize_optional(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn ids_match(left: &str, right: &str) -> bool {
    left == right || left.eq_ignore_ascii_case(right)
}

// ═══════════════════════════════════════════════════════════════════════════════
// WS14.5: Hybrid Remote Push Interface (contract only, no backend dispatch)
// ═══════════════════════════════════════════════════════════════════════════════

/// Platform type for notification endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPlatform {
    Ios,
    Android,
    Web,
}

impl NotificationPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationPlatform::Ios => "ios",
            NotificationPlatform::Android => "android",
            NotificationPlatform::Web => "web",
        }
    }
}

/// Capabilities for a notification endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEndpointCapabilities {
    pub dm: bool,
    pub dm_request: bool,
}

/// A registered notification endpoint for remote push.
/// WS14: Contract only - no backend dispatch implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEndpoint {
    pub endpoint_id: String,
    pub platform: NotificationPlatform,
    pub token_or_subscription: String,
    pub capabilities: NotificationEndpointCapabilities,
    pub device_id: String,
    pub last_seen_ts: u64,
    pub registered_at_ts: u64,
}

/// Error type for endpoint operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationEndpointError {
    InvalidEndpointId,
    InvalidPlatform,
    InvalidToken,
    InvalidDeviceId,
    EndpointAlreadyExists,
    EndpointNotFound,
    StorageError(String),
}

impl std::fmt::Display for NotificationEndpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationEndpointError::InvalidEndpointId => write!(f, "invalid_endpoint_id"),
            NotificationEndpointError::InvalidPlatform => write!(f, "invalid_platform"),
            NotificationEndpointError::InvalidToken => write!(f, "invalid_token"),
            NotificationEndpointError::InvalidDeviceId => write!(f, "invalid_device_id"),
            NotificationEndpointError::EndpointAlreadyExists => {
                write!(f, "endpoint_already_exists")
            }
            NotificationEndpointError::EndpointNotFound => write!(f, "endpoint_not_found"),
            NotificationEndpointError::StorageError(msg) => write!(f, "storage_error: {}", msg),
        }
    }
}

/// In-memory notification endpoint registry.
/// WS14: Contract only - persistence would be added for production.
#[derive(Clone)]
pub struct NotificationEndpointRegistry {
    endpoints: Arc<Mutex<HashMap<String, NotificationEndpoint>>>,
}

impl NotificationEndpointRegistry {
    pub fn new() -> Self {
        Self {
            endpoints: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new notification endpoint.
    /// WS14: Validates inputs but does NOT dispatch to any backend.
    pub fn register_endpoint(
        &self,
        platform: NotificationPlatform,
        token_or_subscription: String,
        capabilities: NotificationEndpointCapabilities,
        device_id: String,
    ) -> Result<NotificationEndpoint, NotificationEndpointError> {
        if token_or_subscription.is_empty() {
            return Err(NotificationEndpointError::InvalidToken);
        }
        if device_id.is_empty() {
            return Err(NotificationEndpointError::InvalidDeviceId);
        }

        let endpoint_id = format!(
            "{}-{}",
            platform.as_str(),
            &device_id[..8.min(device_id.len())]
        );
        let now = now_ms();

        let endpoint = NotificationEndpoint {
            endpoint_id: endpoint_id.clone(),
            platform,
            token_or_subscription,
            capabilities,
            device_id,
            last_seen_ts: now,
            registered_at_ts: now,
        };

        let mut endpoints = self.endpoints.lock();
        endpoints.insert(endpoint_id.clone(), endpoint.clone());

        Ok(endpoint)
    }

    /// Unregister a notification endpoint.
    pub fn unregister_endpoint(&self, endpoint_id: &str) -> Result<(), NotificationEndpointError> {
        let mut endpoints = self.endpoints.lock();
        endpoints
            .remove(endpoint_id)
            .ok_or(NotificationEndpointError::EndpointNotFound)?;
        Ok(())
    }

    /// List all registered endpoints.
    pub fn list_endpoints(&self) -> Vec<NotificationEndpoint> {
        let endpoints = self.endpoints.lock();
        endpoints.values().cloned().collect()
    }

    /// Update last seen timestamp for an endpoint.
    pub fn touch_endpoint(&self, endpoint_id: &str) -> Result<(), NotificationEndpointError> {
        let mut endpoints = self.endpoints.lock();
        let endpoint = endpoints
            .get_mut(endpoint_id)
            .ok_or(NotificationEndpointError::EndpointNotFound)?;
        endpoint.last_seen_ts = now_ms();
        Ok(())
    }
}

impl Default for NotificationEndpointRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn now_ms() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_message() -> NotificationMessageContext {
        NotificationMessageContext {
            conversation_id: None,
            sender_peer_id: "sender-1".to_string(),
            message_id: "msg-1".to_string(),
            explicit_dm_request: None,
            sender_is_known_contact: false,
            has_existing_conversation: false,
            is_self_originated: false,
            is_duplicate: false,
            already_seen: false,
            is_blocked: false,
        }
    }

    fn base_ui_state() -> NotificationUiState {
        NotificationUiState {
            app_in_foreground: false,
            active_conversation_id: None,
        }
    }

    #[test]
    fn unknown_sender_defaults_to_direct_message_request() {
        let decision =
            classify_notification(base_message(), base_ui_state(), MeshSettings::default());

        assert_eq!(decision.kind, NotificationKind::DirectMessageRequest);
        assert!(decision.should_alert);
        assert_eq!(decision.conversation_id, "sender-1");
    }

    #[test]
    fn known_contact_defaults_to_direct_message() {
        let mut message = base_message();
        message.sender_is_known_contact = true;

        let decision = classify_notification(message, base_ui_state(), MeshSettings::default());
        assert_eq!(decision.kind, NotificationKind::DirectMessage);
    }

    #[test]
    fn explicit_request_overrides_known_contact_inference() {
        let mut message = base_message();
        message.sender_is_known_contact = true;
        message.explicit_dm_request = Some(true);

        let decision = classify_notification(message, base_ui_state(), MeshSettings::default());
        assert_eq!(decision.kind, NotificationKind::DirectMessageRequest);
    }

    #[test]
    fn disabled_notifications_suppress_delivery() {
        let settings = MeshSettings {
            notifications_enabled: false,
            ..MeshSettings::default()
        };

        let decision = classify_notification(base_message(), base_ui_state(), settings);
        assert_eq!(decision.kind, NotificationKind::None);
        assert_eq!(
            decision.suppression_reason.as_deref(),
            Some("notifications_disabled")
        );
    }

    #[test]
    fn duplicates_are_suppressed() {
        let mut message = base_message();
        message.is_duplicate = true;

        let decision = classify_notification(message, base_ui_state(), MeshSettings::default());
        assert_eq!(decision.kind, NotificationKind::None);
        assert_eq!(
            decision.suppression_reason.as_deref(),
            Some("duplicate_message")
        );
    }

    #[test]
    fn foreground_direct_messages_follow_foreground_toggle() {
        let mut message = base_message();
        message.sender_is_known_contact = true;
        message.conversation_id = Some("thread-1".to_string());
        let ui_state = NotificationUiState {
            app_in_foreground: true,
            active_conversation_id: Some("thread-1".to_string()),
        };

        let suppressed =
            classify_notification(message.clone(), ui_state.clone(), MeshSettings::default());
        assert_eq!(suppressed.kind, NotificationKind::None);
        assert_eq!(
            suppressed.suppression_reason.as_deref(),
            Some("foreground_conversation_active")
        );

        let allowed_settings = MeshSettings {
            notify_dm_in_foreground: true,
            ..MeshSettings::default()
        };
        let allowed = classify_notification(message, ui_state, allowed_settings);
        assert_eq!(allowed.kind, NotificationKind::DirectMessage);
        assert!(allowed.should_alert);
    }
}
