//! Mobile mesh settings and configuration management
//!
//! Comprehensive configuration that can be serialized to/from JSON,
//! with validation of relay=messaging coupling invariant.

use crate::transport::discovery::DiscoveryMode;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Manual configuration overrides for fine-grained tuning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManualOverrides {
    pub ble_scan_interval_ms: Option<u16>,
    pub ble_advertise_interval_ms: Option<u16>,
    pub relay_priority_threshold: Option<u8>,
    pub relay_max_per_hour: Option<u32>,
}

impl ManualOverrides {
    pub fn is_empty(&self) -> bool {
        self.ble_scan_interval_ms.is_none()
            && self.ble_advertise_interval_ms.is_none()
            && self.relay_priority_threshold.is_none()
            && self.relay_max_per_hour.is_none()
    }
}

/// Complete mesh settings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSettings {
    /// Enable/disable relay functionality (COUPLES with messaging)
    pub relay_enabled: bool,

    /// Enable automatic adjustment based on device state
    pub auto_adjust_enabled: bool,

    /// Manual parameter overrides
    pub manual_overrides: Option<ManualOverrides>,

    /// Discovery mode (Open, Manual, DarkBLE, Silent)
    pub discovery_mode: DiscoveryMode,

    /// Maximum relay messages per hour (minimum 1 if relay_enabled)
    pub max_relay_budget: u32,

    /// Minimum battery percentage before forcing minimal profile
    pub battery_floor_percent: u8,

    /// Enable BLE transport
    pub enable_ble: bool,

    /// Enable WiFi Aware transport
    pub enable_wifi_aware: bool,

    /// Enable internet connectivity
    pub enable_internet: bool,
}

impl Default for MeshSettings {
    fn default() -> Self {
        Self {
            relay_enabled: false,
            auto_adjust_enabled: true,
            manual_overrides: None,
            discovery_mode: DiscoveryMode::Open,
            max_relay_budget: 100,
            battery_floor_percent: 10,
            enable_ble: true,
            enable_wifi_aware: true,
            enable_internet: false,
        }
    }
}

impl MeshSettings {
    /// Create new settings with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    ///
    /// Key invariant: relay=messaging coupling
    /// If relay is enabled, max_relay_budget must be >= 1
    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.relay_enabled && self.max_relay_budget == 0 {
            return Err(SettingsError::ConfigError(
                "relay_enabled requires max_relay_budget >= 1".to_string(),
            ));
        }

        if self.battery_floor_percent > 100 {
            return Err(SettingsError::ConfigError(
                "battery_floor_percent cannot exceed 100".to_string(),
            ));
        }

        if !self.enable_ble && !self.enable_wifi_aware && !self.enable_internet {
            return Err(SettingsError::ConfigError(
                "At least one transport must be enabled".to_string(),
            ));
        }

        Ok(())
    }

    /// Set relay enabled, enforcing coupling with messaging
    pub fn set_relay_enabled(&mut self, enabled: bool) {
        self.relay_enabled = enabled;
        // Enforce minimum budget when relay is enabled
        if enabled && self.max_relay_budget == 0 {
            self.max_relay_budget = 1;
        }
    }

    /// Set max relay budget, validating coupling
    pub fn set_max_relay_budget(&mut self, budget: u32) -> Result<(), SettingsError> {
        if self.relay_enabled && budget == 0 {
            return Err(SettingsError::ConfigError(
                "Cannot set max_relay_budget to 0 while relay_enabled=true".to_string(),
            ));
        }
        self.max_relay_budget = budget;
        Ok(())
    }

    /// Load settings from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, SettingsError> {
        let content = std::fs::read_to_string(path)?;
        let settings: MeshSettings = serde_json::from_str(&content)?;
        settings.validate()?;
        Ok(settings)
    }

    /// Save settings to JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), SettingsError> {
        self.validate()?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Convert to pretty JSON string
    pub fn to_json_string(&self) -> Result<String, SettingsError> {
        self.validate()?;
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Create from JSON string
    pub fn from_json_string(json: &str) -> Result<Self, SettingsError> {
        let settings: MeshSettings = serde_json::from_str(json)?;
        settings.validate()?;
        Ok(settings)
    }

    /// Enable all transports
    pub fn enable_all_transports(&mut self) {
        self.enable_ble = true;
        self.enable_wifi_aware = true;
        self.enable_internet = true;
    }

    /// Disable all non-essential transports (keep BLE)
    pub fn disable_non_essential_transports(&mut self) {
        self.enable_wifi_aware = false;
        self.enable_internet = false;
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "MeshSettings {{\n  relay_enabled: {},\n  auto_adjust_enabled: {},\n  discovery_mode: {:?},\n  max_relay_budget: {},\n  battery_floor: {}%,\n  transports: BLE={}, WiFiAware={}, Internet={}\n}}",
            self.relay_enabled,
            self.auto_adjust_enabled,
            self.discovery_mode,
            self.max_relay_budget,
            self.battery_floor_percent,
            self.enable_ble,
            self.enable_wifi_aware,
            self.enable_internet,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = MeshSettings::default();
        assert!(!settings.relay_enabled);
        assert!(settings.auto_adjust_enabled);
        assert_eq!(settings.battery_floor_percent, 10);
        assert!(settings.enable_ble);
    }

    #[test]
    fn test_validate_default_settings() {
        let settings = MeshSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_relay_coupling_validation() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = true;
        settings.max_relay_budget = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_relay_enabled_with_budget() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = true;
        settings.max_relay_budget = 50;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_battery_floor_validation() {
        let mut settings = MeshSettings::default();
        settings.battery_floor_percent = 101;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_all_transports_disabled_fails() {
        let mut settings = MeshSettings::default();
        settings.enable_ble = false;
        settings.enable_wifi_aware = false;
        settings.enable_internet = false;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_set_relay_enabled_enforces_budget() {
        let mut settings = MeshSettings::default();
        settings.max_relay_budget = 0;
        settings.set_relay_enabled(true);
        assert_eq!(settings.max_relay_budget, 1);
    }

    #[test]
    fn test_set_max_relay_budget_fails_when_enabled_zero() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = true;
        let result = settings.set_max_relay_budget(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_max_relay_budget_succeeds() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = true;
        let result = settings.set_max_relay_budget(50);
        assert!(result.is_ok());
        assert_eq!(settings.max_relay_budget, 50);
    }

    #[test]
    fn test_json_roundtrip() {
        let original = MeshSettings {
            relay_enabled: true,
            auto_adjust_enabled: false,
            manual_overrides: None,
            discovery_mode: DiscoveryMode::Manual,
            max_relay_budget: 100,
            battery_floor_percent: 15,
            enable_ble: true,
            enable_wifi_aware: false,
            enable_internet: false,
        };

        let json = original.to_json_string().unwrap();
        let recovered = MeshSettings::from_json_string(&json).unwrap();

        assert_eq!(recovered.relay_enabled, original.relay_enabled);
        assert_eq!(recovered.auto_adjust_enabled, original.auto_adjust_enabled);
        assert_eq!(recovered.max_relay_budget, original.max_relay_budget);
        assert_eq!(recovered.battery_floor_percent, original.battery_floor_percent);
    }

    #[test]
    fn test_json_with_manual_overrides() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = true;
        settings.manual_overrides = Some(ManualOverrides {
            ble_scan_interval_ms: Some(500),
            ble_advertise_interval_ms: Some(100),
            relay_priority_threshold: Some(75),
            relay_max_per_hour: Some(50),
        });

        let json = settings.to_json_string().unwrap();
        let recovered = MeshSettings::from_json_string(&json).unwrap();

        let overrides = recovered.manual_overrides.unwrap();
        assert_eq!(overrides.ble_scan_interval_ms, Some(500));
        assert_eq!(overrides.relay_max_per_hour, Some(50));
    }

    #[test]
    fn test_json_string_contains_expected_fields() {
        let settings = MeshSettings::default();
        let json = settings.to_json_string().unwrap();
        assert!(json.contains("relay_enabled"));
        assert!(json.contains("auto_adjust_enabled"));
        assert!(json.contains("discovery_mode"));
    }

    #[test]
    fn test_invalid_json_fails() {
        let result = MeshSettings::from_json_string("{ invalid json }");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_validation_enforced() {
        let json = r#"{"relay_enabled": true, "max_relay_budget": 0, "auto_adjust_enabled": true, "discovery_mode": "Open", "battery_floor_percent": 10, "enable_ble": true, "enable_wifi_aware": false, "enable_internet": false}"#;
        let result = MeshSettings::from_json_string(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_all_transports() {
        let mut settings = MeshSettings::default();
        settings.enable_internet = false;
        settings.enable_all_transports();
        assert!(settings.enable_ble);
        assert!(settings.enable_wifi_aware);
        assert!(settings.enable_internet);
    }

    #[test]
    fn test_disable_non_essential_transports() {
        let mut settings = MeshSettings::default();
        settings.enable_internet = true;
        settings.disable_non_essential_transports();
        assert!(settings.enable_ble);
        assert!(!settings.enable_wifi_aware);
        assert!(!settings.enable_internet);
    }

    #[test]
    fn test_manual_overrides_is_empty() {
        let overrides = ManualOverrides::default();
        assert!(overrides.is_empty());

        let mut overrides = ManualOverrides::default();
        overrides.ble_scan_interval_ms = Some(100);
        assert!(!overrides.is_empty());
    }

    #[test]
    fn test_settings_summary() {
        let settings = MeshSettings::default();
        let summary = settings.summary();
        assert!(summary.contains("relay_enabled"));
        assert!(summary.contains("battery_floor"));
    }

    #[test]
    fn test_file_save_and_load() {
        use std::fs;

        let temp_dir = std::env::temp_dir().join(format!("scm_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("settings.json");

        let original = MeshSettings {
            relay_enabled: true,
            auto_adjust_enabled: false,
            manual_overrides: None,
            discovery_mode: DiscoveryMode::Manual,
            max_relay_budget: 200,
            battery_floor_percent: 20,
            enable_ble: true,
            enable_wifi_aware: true,
            enable_internet: false,
        };

        original.save(&file_path).unwrap();
        let loaded = MeshSettings::load(&file_path).unwrap();

        assert_eq!(loaded.relay_enabled, original.relay_enabled);
        assert_eq!(loaded.max_relay_budget, original.max_relay_budget);
        assert_eq!(loaded.battery_floor_percent, original.battery_floor_percent);

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_file_load_nonexistent_fails() {
        let result = MeshSettings::load("/nonexistent/path/settings.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_save_with_invalid_settings_fails() {
        use std::fs;

        let temp_dir = std::env::temp_dir().join(format!("scm_test_inv_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("settings.json");

        let mut settings = MeshSettings::default();
        settings.battery_floor_percent = 101;

        let result = settings.save(&file_path);
        assert!(result.is_err());

        let _ = fs::remove_file(&file_path);
        let _ = fs::remove_dir(&temp_dir);
    }
}
