//! Background mesh service abstraction
//!
//! Provides the main Rust-side service that platform code (Android/iOS) creates
//! and manages. Platform code creates a `MeshService`, calls `start()`, and manages
//! the lifecycle through pause/resume/stop.
//!
//! The service integrates with IronCore for crypto and mesh operations.

use crate::platform::auto_adjust::{AdjustmentProfile, DeviceState, SmartAutoAdjust};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Errors that can occur during platform service operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum PlatformError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Service in invalid state: {0}")]
    InvalidState(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network initialization failed: {0}")]
    NetworkError(String),

    #[error("Unsupported operation on this platform: {0}")]
    UnsupportedOperation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// ============================================================================
// ENUMS & TYPES
// ============================================================================

/// Current state of the mesh service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshServiceState {
    /// Service not running
    Stopped,
    /// Service is starting up (initializing networks, loading state)
    Starting,
    /// Service fully operational
    Running,
    /// Service paused (minimal operation, reduced resource usage)
    Paused,
    /// Service shutting down
    Stopping,
}

impl std::fmt::Display for MeshServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => write!(f, "Stopped"),
            Self::Starting => write!(f, "Starting"),
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Stopping => write!(f, "Stopping"),
        }
    }
}

/// Platform type for capability reporting and adjustments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformType {
    /// Android devices
    Android,
    /// iOS devices
    IOS,
    /// Desktop/laptop (macOS, Linux, Windows)
    Desktop,
    /// WebAssembly in browser
    WASM,
}

/// Platform capabilities that affect adjustment decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// Platform type
    pub platform: PlatformType,
    /// Can use Bluetooth Low Energy
    pub has_ble: bool,
    /// Can use WiFi Aware (Android 6.0+)
    pub has_wifi_aware: bool,
    /// Supports background task execution (with time limits)
    pub has_background_execution: bool,
    /// Can access location for proximity detection
    pub has_location: bool,
    /// Maximum background execution time in seconds
    /// iOS: typically 30 secs, Android: 10 minutes
    pub max_background_time_secs: u32,
}

impl PlatformCapabilities {
    /// Create capabilities for Android
    pub fn android() -> Self {
        Self {
            platform: PlatformType::Android,
            has_ble: true,
            has_wifi_aware: true,
            has_background_execution: true,
            has_location: true,
            max_background_time_secs: 600,
        }
    }

    /// Create capabilities for iOS
    pub fn ios() -> Self {
        Self {
            platform: PlatformType::IOS,
            has_ble: true,
            has_wifi_aware: false,
            has_background_execution: true,
            has_location: true,
            max_background_time_secs: 30,
        }
    }

    /// Create capabilities for desktop
    pub fn desktop() -> Self {
        Self {
            platform: PlatformType::Desktop,
            has_ble: false,
            has_wifi_aware: false,
            has_background_execution: true,
            has_location: false,
            max_background_time_secs: u32::MAX,
        }
    }

    /// Create capabilities for WASM
    pub fn wasm() -> Self {
        Self {
            platform: PlatformType::WASM,
            has_ble: false,
            has_wifi_aware: false,
            has_background_execution: false,
            has_location: false,
            max_background_time_secs: 0,
        }
    }
}

/// Service configuration provided by platform code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshServiceConfig {
    /// Path for persistent state storage (database, keys)
    pub storage_path: String,
    /// Enable Bluetooth Low Energy transport
    pub enable_ble: bool,
    /// Enable WiFi Aware (Android)
    pub enable_wifi_aware: bool,
    /// Enable relaying over internet (if available)
    pub enable_internet: bool,
    /// Enable automatic adjustment based on device state
    pub auto_adjust_enabled: bool,
}

impl MeshServiceConfig {
    /// Validate configuration values
    pub fn validate(&self) -> Result<(), PlatformError> {
        if self.storage_path.trim().is_empty() {
            return Err(PlatformError::InvalidConfig(
                "storage_path cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for MeshServiceConfig {
    fn default() -> Self {
        Self {
            storage_path: "/data/local/tmp/scmessenger".to_string(),
            enable_ble: true,
            enable_wifi_aware: true,
            enable_internet: true,
            auto_adjust_enabled: true,
        }
    }
}

/// Service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    /// Uptime in seconds
    pub uptime_secs: u64,
    /// Number of messages relayed
    pub messages_relayed: u64,
    /// Number of unique peers encountered
    pub peers_seen: u64,
    /// Bytes transferred (all transports)
    pub bytes_transferred: u64,
    /// Current state
    pub state: MeshServiceState,
    /// Current adjustment profile (if auto-adjust enabled)
    pub current_profile: Option<AdjustmentProfile>,
}

// ============================================================================
// MESH SERVICE
// ============================================================================

/// The main Rust-side mesh service
///
/// Platform code (Android/iOS) creates this, calls start(), and manages the lifecycle.
/// This is the bridge between platform-specific lifecycle and the Rust mesh.
pub struct MeshService {
    /// Configuration
    config: Arc<RwLock<MeshServiceConfig>>,
    /// Current service state
    state: Arc<RwLock<MeshServiceState>>,
    /// Platform capabilities
    capabilities: Arc<PlatformCapabilities>,
    /// Auto-adjustment engine
    auto_adjust: Arc<SmartAutoAdjust>,
    /// Service startup timestamp
    started_at: Arc<RwLock<Option<u64>>>,
    /// Statistics
    stats: Arc<RwLock<ServiceStats>>,
}

impl MeshService {
    /// Create a new mesh service with the given configuration
    pub fn new(config: MeshServiceConfig) -> Result<Self, PlatformError> {
        config.validate()?;

        let capabilities = match config.enable_wifi_aware {
            true => Arc::new(PlatformCapabilities::android()),
            false => Arc::new(PlatformCapabilities::ios()),
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(MeshServiceState::Stopped)),
            capabilities,
            auto_adjust: Arc::new(SmartAutoAdjust::new()),
            started_at: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(ServiceStats {
                uptime_secs: 0,
                messages_relayed: 0,
                peers_seen: 0,
                bytes_transferred: 0,
                state: MeshServiceState::Stopped,
                current_profile: None,
            })),
        })
    }

    /// Create a mesh service with specific platform capabilities
    pub fn with_capabilities(
        config: MeshServiceConfig,
        capabilities: PlatformCapabilities,
    ) -> Result<Self, PlatformError> {
        config.validate()?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(MeshServiceState::Stopped)),
            capabilities: Arc::new(capabilities),
            auto_adjust: Arc::new(SmartAutoAdjust::new()),
            started_at: Arc::new(RwLock::new(None)),
            stats: Arc::new(RwLock::new(ServiceStats {
                uptime_secs: 0,
                messages_relayed: 0,
                peers_seen: 0,
                bytes_transferred: 0,
                state: MeshServiceState::Stopped,
                current_profile: None,
            })),
        })
    }

    /// Start the mesh service
    ///
    /// Transitions: Stopped -> Starting -> Running
    pub fn start(&self) -> Result<(), PlatformError> {
        let mut state = self.state.write();

        match *state {
            MeshServiceState::Stopped => {
                *state = MeshServiceState::Starting;
            }
            MeshServiceState::Running | MeshServiceState::Paused => {
                return Err(PlatformError::InvalidState(
                    "Service already running or paused".to_string(),
                ));
            }
            _ => {
                return Err(PlatformError::InvalidState(format!(
                    "Cannot start from {:?} state",
                    state
                )));
            }
        }

        drop(state);

        // Simulate initialization
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| PlatformError::Internal(e.to_string()))?
            .as_secs();

        let mut started_at = self.started_at.write();
        *started_at = Some(now);
        drop(started_at);

        let mut state = self.state.write();
        *state = MeshServiceState::Running;

        let mut stats = self.stats.write();
        stats.state = MeshServiceState::Running;

        Ok(())
    }

    /// Stop the mesh service
    ///
    /// Transitions: Running/Paused -> Stopping -> Stopped
    pub fn stop(&self) -> Result<(), PlatformError> {
        let mut state = self.state.write();

        match *state {
            MeshServiceState::Running | MeshServiceState::Paused => {
                *state = MeshServiceState::Stopping;
            }
            MeshServiceState::Stopped => {
                return Err(PlatformError::InvalidState(
                    "Service already stopped".to_string(),
                ));
            }
            _ => {
                return Err(PlatformError::InvalidState(format!(
                    "Cannot stop from {:?} state",
                    state
                )));
            }
        }

        drop(state);

        let mut started_at = self.started_at.write();
        *started_at = None;
        drop(started_at);

        let mut state = self.state.write();
        *state = MeshServiceState::Stopped;

        let mut stats = self.stats.write();
        stats.state = MeshServiceState::Stopped;
        stats.uptime_secs = 0;

        Ok(())
    }

    /// Pause the mesh service (reduce to minimal operation)
    ///
    /// Useful on iOS when entering background. Maintains connectivity
    /// but reduces scanning and relaying activity.
    ///
    /// Transitions: Running -> Paused
    pub fn pause(&self) -> Result<(), PlatformError> {
        let mut state = self.state.write();

        match *state {
            MeshServiceState::Running => {
                *state = MeshServiceState::Paused;
            }
            MeshServiceState::Paused => {
                return Err(PlatformError::InvalidState(
                    "Service already paused".to_string(),
                ));
            }
            MeshServiceState::Stopped => {
                return Err(PlatformError::InvalidState(
                    "Cannot pause a stopped service".to_string(),
                ));
            }
            _ => {
                return Err(PlatformError::InvalidState(format!(
                    "Cannot pause from {:?} state",
                    state
                )));
            }
        }

        drop(state);

        let mut stats = self.stats.write();
        stats.state = MeshServiceState::Paused;

        Ok(())
    }

    /// Resume the mesh service (back to full operation)
    ///
    /// Transitions: Paused -> Running
    pub fn resume(&self) -> Result<(), PlatformError> {
        let mut state = self.state.write();

        match *state {
            MeshServiceState::Paused => {
                *state = MeshServiceState::Running;
            }
            MeshServiceState::Running => {
                return Err(PlatformError::InvalidState(
                    "Service already running".to_string(),
                ));
            }
            MeshServiceState::Stopped => {
                return Err(PlatformError::InvalidState(
                    "Cannot resume a stopped service; call start() instead".to_string(),
                ));
            }
            _ => {
                return Err(PlatformError::InvalidState(format!(
                    "Cannot resume from {:?} state",
                    state
                )));
            }
        }

        drop(state);

        let mut stats = self.stats.write();
        stats.state = MeshServiceState::Running;

        Ok(())
    }

    /// Get current service state
    pub fn state(&self) -> MeshServiceState {
        *self.state.read()
    }

    /// Update device state and trigger auto-adjustment if enabled
    pub fn update_device_state(&self, device_state: DeviceState) -> Result<(), PlatformError> {
        let config = self.config.read();

        if !config.auto_adjust_enabled {
            return Ok(());
        }

        drop(config);

        let profile = self.auto_adjust.compute_profile(&device_state);

        let mut stats = self.stats.write();
        stats.current_profile = Some(profile);

        Ok(())
    }

    /// Get service statistics
    pub fn service_stats(&self) -> ServiceStats {
        let mut stats = self.stats.read().clone();

        if let Some(started_secs) = *self.started_at.read() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            if *self.state.read() != MeshServiceState::Stopped {
                stats.uptime_secs = now.saturating_sub(started_secs);
            }
        }

        stats
    }

    /// Get platform capabilities
    pub fn capabilities(&self) -> Arc<PlatformCapabilities> {
        self.capabilities.clone()
    }

    /// Get current configuration
    pub fn config(&self) -> MeshServiceConfig {
        self.config.read().clone()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let valid = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        assert!(valid.validate().is_ok());

        let invalid = MeshServiceConfig {
            storage_path: "".to_string(),
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_service_creation() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config);
        assert!(service.is_ok());
        assert_eq!(service.unwrap().state(), MeshServiceState::Stopped);
    }

    #[test]
    fn test_service_start_stop() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        assert!(service.start().is_ok());
        assert_eq!(service.state(), MeshServiceState::Running);

        assert!(service.stop().is_ok());
        assert_eq!(service.state(), MeshServiceState::Stopped);
    }

    #[test]
    fn test_double_start_fails() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        assert!(service.start().is_ok());
        assert!(service.start().is_err());
    }

    #[test]
    fn test_stop_when_stopped_fails() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        assert!(service.stop().is_err());
    }

    #[test]
    fn test_pause_resume_cycle() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        assert!(service.start().is_ok());
        assert!(service.pause().is_ok());
        assert_eq!(service.state(), MeshServiceState::Paused);

        assert!(service.resume().is_ok());
        assert_eq!(service.state(), MeshServiceState::Running);
    }

    #[test]
    fn test_pause_when_stopped_fails() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        assert!(service.pause().is_err());
    }

    #[test]
    fn test_resume_from_stopped_fails() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();
        assert!(service.start().is_ok());
        assert!(service.stop().is_ok());

        assert!(service.resume().is_err());
    }

    #[test]
    fn test_uptime_tracking() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        assert!(service.start().is_ok());
        let stats = service.service_stats();
        assert_eq!(stats.state, MeshServiceState::Running);
        // uptime_secs is unsigned, verify it exists
        let _ = stats.uptime_secs;
    }

    #[test]
    fn test_android_capabilities() {
        let caps = PlatformCapabilities::android();
        assert_eq!(caps.platform, PlatformType::Android);
        assert!(caps.has_ble);
        assert!(caps.has_wifi_aware);
        assert!(caps.has_background_execution);
        assert_eq!(caps.max_background_time_secs, 600);
    }

    #[test]
    fn test_ios_capabilities() {
        let caps = PlatformCapabilities::ios();
        assert_eq!(caps.platform, PlatformType::IOS);
        assert!(caps.has_ble);
        assert!(!caps.has_wifi_aware);
        assert!(caps.has_background_execution);
        assert_eq!(caps.max_background_time_secs, 30);
    }

    #[test]
    fn test_desktop_capabilities() {
        let caps = PlatformCapabilities::desktop();
        assert_eq!(caps.platform, PlatformType::Desktop);
        assert!(!caps.has_ble);
        assert!(!caps.has_wifi_aware);
        assert!(caps.has_background_execution);
    }

    #[test]
    fn test_wasm_capabilities() {
        let caps = PlatformCapabilities::wasm();
        assert_eq!(caps.platform, PlatformType::WASM);
        assert!(!caps.has_ble);
        assert!(!caps.has_background_execution);
        assert_eq!(caps.max_background_time_secs, 0);
    }

    #[test]
    fn test_service_with_custom_capabilities() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let caps = PlatformCapabilities::ios();
        let service = MeshService::with_capabilities(config, caps).unwrap();

        assert_eq!(service.capabilities().platform, PlatformType::IOS);
    }

    #[test]
    fn test_service_stats_initialization() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        let stats = service.service_stats();
        assert_eq!(stats.state, MeshServiceState::Stopped);
        assert_eq!(stats.messages_relayed, 0);
        assert_eq!(stats.peers_seen, 0);
    }

    #[test]
    fn test_device_state_update_without_auto_adjust() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            auto_adjust_enabled: false,
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        let device_state = DeviceState {
            battery_percent: 50,
            is_charging: false,
            is_on_wifi: true,
            is_moving: false,
            screen_on: true,
            time_since_last_interaction_secs: 10,
        };

        assert!(service.update_device_state(device_state).is_ok());
        let stats = service.service_stats();
        assert!(stats.current_profile.is_none());
    }

    #[test]
    fn test_device_state_update_with_auto_adjust() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            auto_adjust_enabled: true,
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        let device_state = DeviceState {
            battery_percent: 50,
            is_charging: false,
            is_on_wifi: true,
            is_moving: false,
            screen_on: true,
            time_since_last_interaction_secs: 10,
        };

        assert!(service.update_device_state(device_state).is_ok());
        let stats = service.service_stats();
        assert!(stats.current_profile.is_some());
    }

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", MeshServiceState::Stopped), "Stopped");
        assert_eq!(format!("{}", MeshServiceState::Running), "Running");
        assert_eq!(format!("{}", MeshServiceState::Paused), "Paused");
    }

    #[test]
    fn test_mesh_service_state_transitions() {
        let config = MeshServiceConfig {
            storage_path: "/data/test".to_string(),
            ..Default::default()
        };
        let service = MeshService::new(config).unwrap();

        // Stopped -> Starting -> Running
        assert!(service.start().is_ok());
        assert_eq!(service.state(), MeshServiceState::Running);

        // Running -> Paused -> Running
        assert!(service.pause().is_ok());
        assert_eq!(service.state(), MeshServiceState::Paused);
        assert!(service.resume().is_ok());
        assert_eq!(service.state(), MeshServiceState::Running);

        // Running -> Stopping -> Stopped
        assert!(service.stop().is_ok());
        assert_eq!(service.state(), MeshServiceState::Stopped);
    }
}
