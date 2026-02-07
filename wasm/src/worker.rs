// WASM Service Worker Bridge — Background sync and push notifications
//
// Provides interfaces for service worker integration, including background sync
// and push notification handling. This allows the WASM client to sync messages
// and receive notifications even when the tab is not in the foreground.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Background sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSyncConfig {
    /// Tag for identifying this sync operation
    pub sync_tag: String,
    /// Minimum interval between syncs in milliseconds
    pub min_interval_ms: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
}

impl Default for BackgroundSyncConfig {
    fn default() -> Self {
        Self {
            sync_tag: "scmessenger-sync".to_string(),
            min_interval_ms: 30000, // 30 seconds
            max_retries: 3,
        }
    }
}

/// Push notification payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotificationPayload {
    /// Title of the notification
    pub title: String,
    /// Body/message of the notification
    pub body: String,
    /// Optional icon URL
    pub icon: Option<String>,
    /// Optional badge URL
    pub badge: Option<String>,
    /// Optional tag for grouping notifications
    pub tag: Option<String>,
    /// Message data (encrypted envelope, etc.)
    pub data: Option<Vec<u8>>,
}

/// Push notification handler interface
#[derive(Debug, Clone)]
pub struct PushNotificationHandler {
    config: BackgroundSyncConfig,
    notification_queue: Arc<RwLock<Vec<PushNotificationPayload>>>,
    last_sync_time: Arc<RwLock<u64>>,
}

impl PushNotificationHandler {
    /// Create a new push notification handler
    pub fn new(config: BackgroundSyncConfig) -> Self {
        Self {
            config,
            notification_queue: Arc::new(RwLock::new(Vec::new())),
            last_sync_time: Arc::new(RwLock::new(0)),
        }
    }

    /// Handle incoming push notification
    pub fn on_push_event(&self, payload: PushNotificationPayload) -> Result<(), String> {
        // Queue the notification for processing
        self.notification_queue.write().push(payload.clone());

        // In a real implementation with web-sys, this would:
        // 1. Call self.clients.matchAll() to find open clients
        // 2. Post message to clients about the notification
        // 3. Display a visible notification via self.registration.showNotification()

        Ok(())
    }

    /// Get queued notifications
    pub fn get_notifications(&self) -> Vec<PushNotificationPayload> {
        self.notification_queue.read().clone()
    }

    /// Clear notifications after processing
    pub fn clear_notifications(&self) {
        self.notification_queue.write().clear();
    }

    /// Get time of last sync
    pub fn last_sync(&self) -> u64 {
        *self.last_sync_time.read()
    }

    /// Update last sync time
    fn set_last_sync(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        *self.last_sync_time.write() = now;
    }

    /// Check if enough time has passed for another sync
    pub fn should_sync(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let last_sync = self.last_sync();
        now.saturating_sub(last_sync) >= self.config.min_interval_ms
    }
}

/// Service worker bridge for WASM integration
#[derive(Debug)]
pub struct ServiceWorkerBridge {
    config: BackgroundSyncConfig,
    registration_status: Arc<RwLock<ServiceWorkerStatus>>,
    sync_handler: Arc<RwLock<Option<Arc<dyn SyncHandler>>>>,
    notification_handler: Arc<RwLock<Option<Arc<dyn NotificationHandler>>>>,
}

/// Service worker registration status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceWorkerStatus {
    NotRegistered,
    Registering,
    Registered,
    Updating,
    Error,
}

/// Trait for handling sync events
pub trait SyncHandler: Send + Sync {
    /// Called when background sync should occur
    fn on_sync(&self) -> Result<(), String>;
}

/// Trait for handling notifications
pub trait NotificationHandler: Send + Sync {
    /// Called when a push notification is received
    fn on_notification(&self, payload: PushNotificationPayload) -> Result<(), String>;
}

impl ServiceWorkerBridge {
    /// Create a new service worker bridge
    pub fn new(config: BackgroundSyncConfig) -> Self {
        Self {
            config,
            registration_status: Arc::new(RwLock::new(ServiceWorkerStatus::NotRegistered)),
            sync_handler: Arc::new(RwLock::new(None)),
            notification_handler: Arc::new(RwLock::new(None)),
        }
    }

    /// Register the service worker
    pub fn register(&self, script_url: &str) -> Result<(), String> {
        let mut status = self.registration_status.write();
        if *status != ServiceWorkerStatus::NotRegistered {
            return Err(format!("Cannot register from status {:?}", status));
        }

        *status = ServiceWorkerStatus::Registering;
        drop(status);

        // In a real WASM environment with web-sys, this would:
        // navigator.serviceWorker.register(script_url).then(registration => {
        //     // Handle registration
        // })

        let mut status = self.registration_status.write();
        *status = ServiceWorkerStatus::Registered;
        Ok(())
    }

    /// Unregister the service worker
    pub fn unregister(&self) -> Result<(), String> {
        let mut status = self.registration_status.write();
        *status = ServiceWorkerStatus::NotRegistered;
        Ok(())
    }

    /// Register for background sync
    pub fn register_sync(&self) -> Result<(), String> {
        let status = self.registration_status.read();
        if *status != ServiceWorkerStatus::Registered {
            return Err(format!(
                "Service worker not registered: {:?}",
                status
            ));
        }
        drop(status);

        // In a real WASM environment with web-sys, this would:
        // registration.sync.register(config.sync_tag).then(...).catch(...)

        Ok(())
    }

    /// Set sync event handler
    pub fn set_sync_handler(&self, handler: Arc<dyn SyncHandler>) {
        *self.sync_handler.write() = Some(handler);
    }

    /// Set notification event handler
    pub fn set_notification_handler(&self, handler: Arc<dyn NotificationHandler>) {
        *self.notification_handler.write() = Some(handler);
    }

    /// Handle sync event (called by service worker)
    pub fn on_sync_event(&self) -> Result<(), String> {
        let handler = self.sync_handler.read();
        match handler.as_ref() {
            Some(h) => h.on_sync(),
            None => Err("No sync handler registered".to_string()),
        }
    }

    /// Handle notification event (called by service worker)
    pub fn on_notification_event(&self, payload: PushNotificationPayload) -> Result<(), String> {
        let handler = self.notification_handler.read();
        match handler.as_ref() {
            Some(h) => h.on_notification(payload),
            None => Err("No notification handler registered".to_string()),
        }
    }

    /// Get current registration status
    pub fn status(&self) -> ServiceWorkerStatus {
        *self.registration_status.read()
    }

    /// Get configuration
    pub fn config(&self) -> &BackgroundSyncConfig {
        &self.config
    }
}

/// Default sync handler implementation
#[derive(Debug)]
pub struct DefaultSyncHandler;

impl SyncHandler for DefaultSyncHandler {
    fn on_sync(&self) -> Result<(), String> {
        // Default implementation: just log that sync occurred
        // Real implementation would sync with relays
        Ok(())
    }
}

/// Default notification handler implementation
#[derive(Debug)]
pub struct DefaultNotificationHandler;

impl NotificationHandler for DefaultNotificationHandler {
    fn on_notification(&self, payload: PushNotificationPayload) -> Result<(), String> {
        // Default implementation: just accept the notification
        // Real implementation would process encrypted messages
        let _ = payload;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_sync_config_defaults() {
        let config = BackgroundSyncConfig::default();
        assert_eq!(config.sync_tag, "scmessenger-sync");
        assert_eq!(config.min_interval_ms, 30000);
    }

    #[test]
    fn test_push_notification_payload() {
        let payload = PushNotificationPayload {
            title: "New Message".to_string(),
            body: "You have a new encrypted message".to_string(),
            icon: Some("icon.png".to_string()),
            badge: Some("badge.png".to_string()),
            tag: Some("message".to_string()),
            data: Some(vec![1, 2, 3, 4, 5]),
        };

        assert_eq!(payload.title, "New Message");
        assert!(payload.data.is_some());
    }

    #[test]
    fn test_push_notification_handler_creation() {
        let handler = PushNotificationHandler::new(BackgroundSyncConfig::default());
        assert_eq!(handler.get_notifications().len(), 0);
    }

    #[test]
    fn test_push_notification_handler_queue() {
        let handler = PushNotificationHandler::new(BackgroundSyncConfig::default());

        let payload = PushNotificationPayload {
            title: "Test".to_string(),
            body: "Test notification".to_string(),
            icon: None,
            badge: None,
            tag: None,
            data: None,
        };

        assert!(handler.on_push_event(payload).is_ok());
        assert_eq!(handler.get_notifications().len(), 1);

        handler.clear_notifications();
        assert_eq!(handler.get_notifications().len(), 0);
    }

    #[test]
    fn test_push_notification_handler_should_sync() {
        let handler = PushNotificationHandler::new(BackgroundSyncConfig {
            min_interval_ms: 100, // 100ms for testing
            ..Default::default()
        });

        // Should sync initially
        assert!(handler.should_sync());

        handler.set_last_sync();

        // Shouldn't sync immediately after
        assert!(!handler.should_sync());
    }

    #[test]
    fn test_service_worker_bridge_creation() {
        let bridge = ServiceWorkerBridge::new(BackgroundSyncConfig::default());
        assert_eq!(bridge.status(), ServiceWorkerStatus::NotRegistered);
    }

    #[test]
    fn test_service_worker_register() {
        let bridge = ServiceWorkerBridge::new(BackgroundSyncConfig::default());
        assert!(bridge.register("worker.js").is_ok());
        assert_eq!(bridge.status(), ServiceWorkerStatus::Registered);
    }

    #[test]
    fn test_service_worker_cannot_register_twice() {
        let bridge = ServiceWorkerBridge::new(BackgroundSyncConfig::default());
        bridge.register("worker.js").unwrap();
        assert!(bridge.register("worker.js").is_err());
    }

    #[test]
    fn test_service_worker_unregister() {
        let bridge = ServiceWorkerBridge::new(BackgroundSyncConfig::default());
        bridge.register("worker.js").unwrap();
        assert!(bridge.unregister().is_ok());
        assert_eq!(bridge.status(), ServiceWorkerStatus::NotRegistered);
    }

    #[test]
    fn test_service_worker_sync_requires_registration() {
        let bridge = ServiceWorkerBridge::new(BackgroundSyncConfig::default());
        assert!(bridge.register_sync().is_err());

        bridge.register("worker.js").unwrap();
        assert!(bridge.register_sync().is_ok());
    }

    #[test]
    fn test_service_worker_sync_handler() {
        let bridge = ServiceWorkerBridge::new(BackgroundSyncConfig::default());
        let handler = Arc::new(DefaultSyncHandler);
        bridge.set_sync_handler(handler);

        assert!(bridge.on_sync_event().is_ok());
    }

    #[test]
    fn test_service_worker_notification_handler() {
        let bridge = ServiceWorkerBridge::new(BackgroundSyncConfig::default());
        let handler = Arc::new(DefaultNotificationHandler);
        bridge.set_notification_handler(handler);

        let payload = PushNotificationPayload {
            title: "Test".to_string(),
            body: "Test".to_string(),
            icon: None,
            badge: None,
            tag: None,
            data: None,
        };

        assert!(bridge.on_notification_event(payload).is_ok());
    }

    #[test]
    fn test_service_worker_handler_not_registered() {
        let bridge = ServiceWorkerBridge::new(BackgroundSyncConfig::default());
        assert!(bridge.on_sync_event().is_err());

        let payload = PushNotificationPayload {
            title: "Test".to_string(),
            body: "Test".to_string(),
            icon: None,
            badge: None,
            tag: None,
            data: None,
        };

        assert!(bridge.on_notification_event(payload).is_err());
    }

    #[test]
    fn test_default_sync_handler() {
        let handler = DefaultSyncHandler;
        assert!(handler.on_sync().is_ok());
    }

    #[test]
    fn test_default_notification_handler() {
        let handler = DefaultNotificationHandler;
        let payload = PushNotificationPayload {
            title: "Test".to_string(),
            body: "Test".to_string(),
            icon: None,
            badge: None,
            tag: None,
            data: None,
        };
        assert!(handler.on_notification(payload).is_ok());
    }

    #[test]
    fn test_service_worker_status_enum() {
        let statuses = vec![
            ServiceWorkerStatus::NotRegistered,
            ServiceWorkerStatus::Registering,
            ServiceWorkerStatus::Registered,
            ServiceWorkerStatus::Updating,
            ServiceWorkerStatus::Error,
        ];
        assert_eq!(statuses.len(), 5);
    }
}
