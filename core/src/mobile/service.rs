//! Core MeshService for mobile platform background operation
//!
//! Provides lifecycle management, peer discovery, data reception,
//! and statistics tracking for mobile mesh nodes.

use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Starting,
    Running,
    Paused,
    BackgroundRestricted,
    Stopping,
    Stopped,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceState::Starting => write!(f, "Starting"),
            ServiceState::Running => write!(f, "Running"),
            ServiceState::Paused => write!(f, "Paused"),
            ServiceState::BackgroundRestricted => write!(f, "BackgroundRestricted"),
            ServiceState::Stopping => write!(f, "Stopping"),
            ServiceState::Stopped => write!(f, "Stopped"),
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum ServiceError {
    #[error("Service is not running")]
    NotRunning,
    #[error("Service is already running")]
    AlreadyRunning,
    #[error("Invalid state transition")]
    InvalidStateTransition,
    #[error("Platform bridge error: {0}")]
    PlatformBridgeError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Configuration for the mesh service
#[derive(Debug, Clone)]
pub struct MeshServiceConfig {
    pub storage_path: String,
    pub enable_ble: bool,
    pub enable_wifi_aware: bool,
    pub enable_internet: bool,
}

impl Default for MeshServiceConfig {
    fn default() -> Self {
        Self {
            storage_path: String::new(),
            enable_ble: true,
            enable_wifi_aware: true,
            enable_internet: false,
        }
    }
}

impl MeshServiceConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ServiceError> {
        if self.storage_path.is_empty() {
            return Err(ServiceError::ConfigError(
                "storage_path cannot be empty".to_string(),
            ));
        }

        if !self.enable_ble && !self.enable_wifi_aware && !self.enable_internet {
            return Err(ServiceError::ConfigError(
                "At least one transport must be enabled".to_string(),
            ));
        }

        Ok(())
    }
}

/// Callback interface for platform events
pub trait PlatformBridge: Send + Sync {
    // BLE Operations
    fn request_ble_scan(&self) -> Result<(), String>;
    fn request_ble_advertise(&self) -> Result<(), String>;

    // WiFi Aware Operations
    fn request_wifi_aware_publish(&self) -> Result<(), String>;
    fn request_wifi_aware_subscribe(&self) -> Result<(), String>;

    // Notifications
    fn show_notification(&self, title: &str, body: &str) -> Result<(), String>;
    fn update_notification(&self, id: u32, title: &str, body: &str) -> Result<(), String>;

    // Device Status Queries
    fn get_battery_level(&self) -> u8; // 0-100
    fn is_charging(&self) -> bool;
    fn is_on_wifi(&self) -> bool;
    fn get_motion_state(&self) -> MotionState;

    // Background Time Management
    fn request_background_time(&self, duration_secs: u32) -> Result<(), String>;
    fn schedule_background_task(&self, delay_secs: u32) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionState {
    Still,
    Walking,
    Running,
    Automotive,
    Unknown,
}

/// Service statistics
#[derive(Debug, Clone, Default)]
pub struct ServiceStats {
    pub peers_connected: u32,
    pub messages_relayed: u64,
    pub uptime_secs: u64,
}

/// Core background service for mesh operation
pub struct MeshService {
    config: MeshServiceConfig,
    state: Arc<RwLock<ServiceState>>,
    platform: Arc<RwLock<Option<Arc<dyn PlatformBridge>>>>,
    stats: Arc<RwLock<ServiceStats>>,
}

impl MeshService {
    /// Create a new mesh service
    pub fn new(config: MeshServiceConfig) -> Result<Self, ServiceError> {
        config.validate()?;

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(ServiceState::Stopped)),
            platform: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(ServiceStats::default())),
        })
    }

    /// Set the platform bridge
    pub fn set_platform_bridge(&self, bridge: Option<Arc<dyn PlatformBridge>>) {
        *self.platform.write() = bridge;
    }

    /// Start the service
    pub fn start(&self) -> Result<(), ServiceError> {
        let mut state = self.state.write();

        if matches!(*state, ServiceState::Running | ServiceState::Paused) {
            return Err(ServiceError::AlreadyRunning);
        }

        *state = ServiceState::Starting;
        drop(state);

        // Initialize platform operations
        if let Some(platform) = self.platform.read().as_ref() {
            if self.config.enable_ble {
                platform
                    .request_ble_scan()
                    .map_err(|e| ServiceError::PlatformBridgeError(e))?;
                platform
                    .request_ble_advertise()
                    .map_err(|e| ServiceError::PlatformBridgeError(e))?;
            }

            if self.config.enable_wifi_aware {
                platform
                    .request_wifi_aware_publish()
                    .map_err(|e| ServiceError::PlatformBridgeError(e))?;
                platform
                    .request_wifi_aware_subscribe()
                    .map_err(|e| ServiceError::PlatformBridgeError(e))?;
            }
        }

        let mut state = self.state.write();
        *state = ServiceState::Running;

        tracing::info!("Mesh service started");
        Ok(())
    }

    /// Stop the service
    pub fn stop(&self) -> Result<(), ServiceError> {
        let mut state = self.state.write();

        if matches!(*state, ServiceState::Stopped | ServiceState::Stopping) {
            return Ok(());
        }

        *state = ServiceState::Stopping;
        drop(state);

        // Cleanup
        let mut stats = self.stats.write();
        stats.peers_connected = 0;

        let mut state = self.state.write();
        *state = ServiceState::Stopped;

        tracing::info!("Mesh service stopped");
        Ok(())
    }

    /// Pause the service (background restriction)
    pub fn pause(&self) -> Result<(), ServiceError> {
        let mut state = self.state.write();

        match *state {
            ServiceState::Running => {
                *state = ServiceState::Paused;
                Ok(())
            }
            ServiceState::Paused => Ok(()),
            _ => Err(ServiceError::InvalidStateTransition),
        }
    }

    /// Resume the service
    pub fn resume(&self) -> Result<(), ServiceError> {
        let mut state = self.state.write();

        match *state {
            ServiceState::Paused | ServiceState::BackgroundRestricted => {
                *state = ServiceState::Running;
                Ok(())
            }
            ServiceState::Running => Ok(()),
            _ => Err(ServiceError::InvalidStateTransition),
        }
    }

    /// Get current service state
    pub fn get_state(&self) -> ServiceState {
        *self.state.read()
    }

    /// Mark service as background-restricted
    pub fn set_background_restricted(&self, restricted: bool) {
        let mut state = self.state.write();

        if restricted {
            if matches!(*state, ServiceState::Running | ServiceState::Paused) {
                *state = ServiceState::BackgroundRestricted;
            }
        } else {
            if matches!(*state, ServiceState::BackgroundRestricted) {
                *state = ServiceState::Running;
            }
        }
    }

    /// Handle peer discovery event
    pub fn on_peer_discovered(&self, peer_id: String) -> Result<(), ServiceError> {
        if !matches!(self.get_state(), ServiceState::Running) {
            return Err(ServiceError::NotRunning);
        }

        let mut stats = self.stats.write();
        stats.peers_connected = stats.peers_connected.saturating_add(1);

        tracing::debug!("Peer discovered: {}", peer_id);
        Ok(())
    }

    /// Handle peer disconnection
    pub fn on_peer_disconnected(&self, peer_id: String) -> Result<(), ServiceError> {
        let mut stats = self.stats.write();
        stats.peers_connected = stats.peers_connected.saturating_sub(1);

        tracing::debug!("Peer disconnected: {}", peer_id);
        Ok(())
    }

    /// Handle received data
    pub fn on_data_received(&self, _peer_id: String, size_bytes: u32) -> Result<(), ServiceError> {
        if !matches!(self.get_state(), ServiceState::Running) {
            return Err(ServiceError::NotRunning);
        }

        let mut stats = self.stats.write();
        stats.messages_relayed = stats.messages_relayed.saturating_add(size_bytes as u64);

        tracing::debug!("Data received: {} bytes", size_bytes);
        Ok(())
    }

    /// Get service statistics
    pub fn get_service_stats(&self) -> ServiceStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write();
        *stats = ServiceStats::default();
    }

    /// Get configuration
    pub fn get_config(&self) -> MeshServiceConfig {
        self.config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPlatformBridge;

    impl PlatformBridge for MockPlatformBridge {
        fn request_ble_scan(&self) -> Result<(), String> {
            Ok(())
        }

        fn request_ble_advertise(&self) -> Result<(), String> {
            Ok(())
        }

        fn request_wifi_aware_publish(&self) -> Result<(), String> {
            Ok(())
        }

        fn request_wifi_aware_subscribe(&self) -> Result<(), String> {
            Ok(())
        }

        fn show_notification(&self, _title: &str, _body: &str) -> Result<(), String> {
            Ok(())
        }

        fn update_notification(&self, _id: u32, _title: &str, _body: &str) -> Result<(), String> {
            Ok(())
        }

        fn get_battery_level(&self) -> u8 {
            50
        }

        fn is_charging(&self) -> bool {
            false
        }

        fn is_on_wifi(&self) -> bool {
            true
        }

        fn get_motion_state(&self) -> MotionState {
            MotionState::Still
        }

        fn request_background_time(&self, _duration_secs: u32) -> Result<(), String> {
            Ok(())
        }

        fn schedule_background_task(&self, _delay_secs: u32) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = MeshServiceConfig {
            storage_path: String::new(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_all_disabled() {
        let invalid_config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: false,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_valid_config() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_service_creation() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_service_lifecycle() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();
        assert_eq!(service.get_state(), ServiceState::Stopped);

        let bridge = Arc::new(MockPlatformBridge);
        service.set_platform_bridge(Some(bridge));

        assert!(service.start().is_ok());
        assert_eq!(service.get_state(), ServiceState::Running);

        assert!(service.stop().is_ok());
        assert_eq!(service.get_state(), ServiceState::Stopped);
    }

    #[test]
    fn test_double_start_fails() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();
        let bridge = Arc::new(MockPlatformBridge);
        service.set_platform_bridge(Some(bridge));

        assert!(service.start().is_ok());
        assert!(service.start().is_err());
    }

    #[test]
    fn test_pause_resume() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();
        let bridge = Arc::new(MockPlatformBridge);
        service.set_platform_bridge(Some(bridge));

        service.start().unwrap();
        assert!(service.pause().is_ok());
        assert_eq!(service.get_state(), ServiceState::Paused);

        assert!(service.resume().is_ok());
        assert_eq!(service.get_state(), ServiceState::Running);
    }

    #[test]
    fn test_background_restricted() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();
        let bridge = Arc::new(MockPlatformBridge);
        service.set_platform_bridge(Some(bridge));

        service.start().unwrap();
        service.set_background_restricted(true);
        assert_eq!(service.get_state(), ServiceState::BackgroundRestricted);

        service.set_background_restricted(false);
        assert_eq!(service.get_state(), ServiceState::Running);
    }

    #[test]
    fn test_peer_discovery() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();
        let bridge = Arc::new(MockPlatformBridge);
        service.set_platform_bridge(Some(bridge));

        service.start().unwrap();
        service.on_peer_discovered("peer1".to_string()).unwrap();

        let stats = service.get_service_stats();
        assert_eq!(stats.peers_connected, 1);
    }

    #[test]
    fn test_peer_disconnection() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();
        let bridge = Arc::new(MockPlatformBridge);
        service.set_platform_bridge(Some(bridge));

        service.start().unwrap();
        service.on_peer_discovered("peer1".to_string()).unwrap();
        service.on_peer_disconnected("peer1".to_string()).unwrap();

        let stats = service.get_service_stats();
        assert_eq!(stats.peers_connected, 0);
    }

    #[test]
    fn test_data_received_updates_stats() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();
        let bridge = Arc::new(MockPlatformBridge);
        service.set_platform_bridge(Some(bridge));

        service.start().unwrap();
        service.on_data_received("peer1".to_string(), 1024).unwrap();
        service.on_data_received("peer2".to_string(), 512).unwrap();

        let stats = service.get_service_stats();
        assert_eq!(stats.messages_relayed, 1536);
    }

    #[test]
    fn test_reset_stats() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();
        let bridge = Arc::new(MockPlatformBridge);
        service.set_platform_bridge(Some(bridge));

        service.start().unwrap();
        service.on_peer_discovered("peer1".to_string()).unwrap();
        service.reset_stats();

        let stats = service.get_service_stats();
        assert_eq!(stats.peers_connected, 0);
    }

    #[test]
    fn test_operations_when_not_running() {
        let config = MeshServiceConfig {
            storage_path: "/tmp".to_string(),
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let service = MeshService::new(config).unwrap();

        assert!(service.on_peer_discovered("peer1".to_string()).is_err());
        assert!(service.on_data_received("peer1".to_string(), 1024).is_err());
    }
}
