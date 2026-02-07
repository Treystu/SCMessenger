//! User-configurable mesh settings
//!
//! Settings that control mesh behavior at the application level:
//! - Relay enabled/disabled (THE critical toggle)
//! - Discovery and privacy modes
//! - Message TTL and hop limits
//! - Battery floor constraints

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Errors that can occur during settings validation
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum SettingsError {
    #[error("Invalid relay budget: cannot set relay budget to 0 while relay is enabled")]
    InvalidRelayBudget,

    #[error("Invalid battery floor: must be 0-100, got {0}")]
    InvalidBatteryFloor(u8),

    #[error("Invalid hop count: must be 1-20, got {0}")]
    InvalidHopCount(u8),

    #[error("Invalid message TTL: must be > 0, got {0}")]
    InvalidMessageTTL(u32),

    #[error("Invalid settings combination: {0}")]
    InvalidCombination(String),
}

// ============================================================================
// ENUMS
// ============================================================================

/// Privacy mode affects encryption and routing behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyMode {
    /// No onion routing or cover traffic
    /// Messages visible to 1-hop peers
    Standard,

    /// 3-hop onion routing
    /// Requires 3 intermediate nodes to decrypt
    Enhanced,

    /// 5-hop onion routing + cover traffic
    /// Maximum privacy, higher CPU cost
    Maximum,
}

impl std::fmt::Display for PrivacyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "Standard"),
            Self::Enhanced => write!(f, "Enhanced"),
            Self::Maximum => write!(f, "Maximum"),
        }
    }
}

impl Default for PrivacyMode {
    fn default() -> Self {
        Self::Standard
    }
}

/// Discovery mode controls how peers discover each other
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryMode {
    /// Full mDNS + Identify. Fast discovery, zero privacy.
    ///
    /// Use for development and trusted LANs.
    /// Broadcasts PeerId, IP, port to everyone.
    Open,

    /// Manual peer addition only. Connect to explicit addresses.
    ///
    /// Kademlia for known bootstrap nodes. Identify disabled.
    Closed,

    /// Encrypted BLE beacons or stealth mode
    ///
    /// Maximum stealth — invisible on the network.
    Stealth,
}

impl std::fmt::Display for DiscoveryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "Open"),
            Self::Closed => write!(f, "Closed"),
            Self::Stealth => write!(f, "Stealth"),
        }
    }
}

impl Default for DiscoveryMode {
    fn default() -> Self {
        Self::Open
    }
}

// ============================================================================
// MESH SETTINGS
// ============================================================================

/// User-configurable mesh settings
///
/// Critical invariant: if relay_enabled is true, relay_budget_override
/// must NOT be Some(0). Relay and messaging are coupled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSettings {
    /// Enable/disable relay functionality
    /// THE critical toggle — relay is tightly coupled to messaging
    pub relay_enabled: bool,

    /// Enable automatic resource adjustment based on device state
    pub auto_adjust_enabled: bool,

    /// Manual override for scan interval (milliseconds)
    /// If None, use auto-adjust recommendation
    pub scan_interval_override_ms: Option<u32>,

    /// Manual override for relay budget (messages per hour)
    /// Must be >= 1 if relay_enabled, or None to use auto-adjust
    pub relay_budget_override: Option<u32>,

    /// Enable Bluetooth Low Energy
    pub enable_ble: bool,

    /// Enable WiFi Aware (Android 6.0+)
    pub enable_wifi_aware: bool,

    /// Enable relaying through internet gateway
    pub enable_internet_relay: bool,

    /// Stop relaying when battery falls below this percentage
    pub battery_floor_percent: u8,

    /// Discovery mode
    pub discovery_mode: DiscoveryMode,

    /// Privacy mode affects encryption and routing
    pub privacy_mode: PrivacyMode,

    /// Maximum hop count for messages (1-20)
    pub max_hop_count: u8,

    /// Message time-to-live in hours
    pub message_ttl_hours: u32,
}

impl MeshSettings {
    /// Validate settings
    pub fn validate(&self) -> Result<(), SettingsError> {
        // Relay budget enforcement: if relay is enabled, budget must not be 0
        if self.relay_enabled {
            if let Some(budget) = self.relay_budget_override {
                if budget == 0 {
                    return Err(SettingsError::InvalidRelayBudget);
                }
            }
        }

        // Battery floor must be 0-100
        if self.battery_floor_percent > 100 {
            return Err(SettingsError::InvalidBatteryFloor(self.battery_floor_percent));
        }

        // Hop count must be 1-20
        if self.max_hop_count < 1 || self.max_hop_count > 20 {
            return Err(SettingsError::InvalidHopCount(self.max_hop_count));
        }

        // Message TTL must be > 0
        if self.message_ttl_hours == 0 {
            return Err(SettingsError::InvalidMessageTTL(self.message_ttl_hours));
        }

        Ok(())
    }

    /// Check if relay is currently active (enabled + not below battery floor)
    pub fn is_relay_active(&self, current_battery_percent: u8) -> bool {
        self.relay_enabled && current_battery_percent >= self.battery_floor_percent
    }
}

impl Default for MeshSettings {
    fn default() -> Self {
        Self {
            relay_enabled: true,
            auto_adjust_enabled: true,
            scan_interval_override_ms: None,
            relay_budget_override: None,
            enable_ble: true,
            enable_wifi_aware: true,
            enable_internet_relay: true,
            battery_floor_percent: 10,
            discovery_mode: DiscoveryMode::Open,
            privacy_mode: PrivacyMode::Standard,
            max_hop_count: 10,
            message_ttl_hours: 72,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = MeshSettings::default();
        assert!(settings.relay_enabled);
        assert!(settings.auto_adjust_enabled);
        assert!(settings.enable_ble);
        assert_eq!(settings.battery_floor_percent, 10);
        assert_eq!(settings.max_hop_count, 10);
        assert_eq!(settings.message_ttl_hours, 72);
    }

    #[test]
    fn test_default_settings_valid() {
        let settings = MeshSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_relay_budget_zero_with_relay_enabled() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = true;
        settings.relay_budget_override = Some(0);

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_relay_budget_zero_with_relay_disabled() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = false;
        settings.relay_budget_override = Some(0);

        // Should be valid — relay is disabled, so budget doesn't matter
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_relay_budget_valid_when_enabled() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = true;
        settings.relay_budget_override = Some(100);

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_battery_floor_invalid_high() {
        let mut settings = MeshSettings::default();
        settings.battery_floor_percent = 101;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_battery_floor_valid_boundary() {
        let mut settings = MeshSettings::default();

        settings.battery_floor_percent = 0;
        assert!(settings.validate().is_ok());

        settings.battery_floor_percent = 100;
        assert!(settings.validate().is_ok());

        settings.battery_floor_percent = 50;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_hop_count_invalid_zero() {
        let mut settings = MeshSettings::default();
        settings.max_hop_count = 0;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_hop_count_invalid_too_high() {
        let mut settings = MeshSettings::default();
        settings.max_hop_count = 21;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_hop_count_valid_boundary() {
        let mut settings = MeshSettings::default();

        settings.max_hop_count = 1;
        assert!(settings.validate().is_ok());

        settings.max_hop_count = 20;
        assert!(settings.validate().is_ok());

        settings.max_hop_count = 10;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_message_ttl_invalid_zero() {
        let mut settings = MeshSettings::default();
        settings.message_ttl_hours = 0;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_message_ttl_valid() {
        let mut settings = MeshSettings::default();

        settings.message_ttl_hours = 1;
        assert!(settings.validate().is_ok());

        settings.message_ttl_hours = 72;
        assert!(settings.validate().is_ok());

        settings.message_ttl_hours = 1000;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_is_relay_active_enabled_above_floor() {
        let settings = MeshSettings {
            relay_enabled: true,
            battery_floor_percent: 10,
            ..Default::default()
        };

        assert!(settings.is_relay_active(50));
        assert!(settings.is_relay_active(100));
        assert!(settings.is_relay_active(10)); // At floor is active
    }

    #[test]
    fn test_is_relay_active_enabled_below_floor() {
        let settings = MeshSettings {
            relay_enabled: true,
            battery_floor_percent: 10,
            ..Default::default()
        };

        assert!(!settings.is_relay_active(9));
        assert!(!settings.is_relay_active(0));
    }

    #[test]
    fn test_is_relay_active_disabled() {
        let settings = MeshSettings {
            relay_enabled: false,
            battery_floor_percent: 10,
            ..Default::default()
        };

        assert!(!settings.is_relay_active(50));
        assert!(!settings.is_relay_active(100));
    }

    #[test]
    fn test_privacy_mode_display() {
        assert_eq!(format!("{}", PrivacyMode::Standard), "Standard");
        assert_eq!(format!("{}", PrivacyMode::Enhanced), "Enhanced");
        assert_eq!(format!("{}", PrivacyMode::Maximum), "Maximum");
    }

    #[test]
    fn test_privacy_mode_default() {
        assert_eq!(PrivacyMode::default(), PrivacyMode::Standard);
    }

    #[test]
    fn test_discovery_mode_display() {
        assert_eq!(format!("{}", DiscoveryMode::Open), "Open");
        assert_eq!(format!("{}", DiscoveryMode::Closed), "Closed");
        assert_eq!(format!("{}", DiscoveryMode::Stealth), "Stealth");
    }

    #[test]
    fn test_discovery_mode_default() {
        assert_eq!(DiscoveryMode::default(), DiscoveryMode::Open);
    }

    #[test]
    fn test_discovery_mode_equality() {
        let mode1 = DiscoveryMode::Open;
        let mode2 = DiscoveryMode::Open;
        assert_eq!(mode1, mode2);

        let mode3 = DiscoveryMode::Closed;
        assert_ne!(mode1, mode3);
    }

    #[test]
    fn test_all_valid_settings_combinations() {
        for relay in &[true, false] {
            for privacy in &[PrivacyMode::Standard, PrivacyMode::Enhanced, PrivacyMode::Maximum] {
                for discovery in &[DiscoveryMode::Open, DiscoveryMode::Closed, DiscoveryMode::Stealth] {
                    let settings = MeshSettings {
                        relay_enabled: *relay,
                        privacy_mode: *privacy,
                        discovery_mode: discovery.clone(),
                        ..Default::default()
                    };

                    assert!(settings.validate().is_ok(),
                        "Settings should be valid: relay={}, privacy={}, discovery={}",
                        relay, privacy, discovery);
                }
            }
        }
    }

    #[test]
    fn test_settings_serialization() {
        let settings = MeshSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: MeshSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.relay_enabled, deserialized.relay_enabled);
        assert_eq!(settings.auto_adjust_enabled, deserialized.auto_adjust_enabled);
        assert_eq!(settings.battery_floor_percent, deserialized.battery_floor_percent);
    }

    #[test]
    fn test_multiple_validation_errors_first_caught() {
        let mut settings = MeshSettings::default();
        settings.relay_enabled = true;
        settings.relay_budget_override = Some(0);
        settings.battery_floor_percent = 101;

        // First error should be about relay budget
        let result = settings.validate();
        assert!(result.is_err());
        assert!(matches!(result, Err(SettingsError::InvalidRelayBudget)));
    }
}
