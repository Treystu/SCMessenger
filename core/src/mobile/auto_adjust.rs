//! Automatic adjustment engine for power/network-aware parameter tuning
//!
//! Reads device state and dynamically adjusts all mesh parameters
//! to optimize for battery, network, and motion conditions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjustmentProfile {
    Maximum,
    High,
    Standard,
    Reduced,
    Minimal,
}

impl std::fmt::Display for AdjustmentProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdjustmentProfile::Maximum => write!(f, "Maximum"),
            AdjustmentProfile::High => write!(f, "High"),
            AdjustmentProfile::Standard => write!(f, "Standard"),
            AdjustmentProfile::Reduced => write!(f, "Reduced"),
            AdjustmentProfile::Minimal => write!(f, "Minimal"),
        }
    }
}

/// Device state snapshot
#[derive(Debug, Clone, Copy)]
pub struct DeviceProfile {
    pub battery_percent: u8,
    pub is_charging: bool,
    pub is_on_wifi: bool,
    pub motion_state: MotionState,
    pub screen_on: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionState {
    Still,
    Walking,
    Running,
    Automotive,
    Unknown,
}

/// BLE adjustment parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleAdjustment {
    pub scan_interval_ms: u16,
    pub scan_window_ms: u16,
    pub advertise_interval_ms: u16,
}

impl Default for BleAdjustment {
    fn default() -> Self {
        Self {
            scan_interval_ms: 1280,
            scan_window_ms: 11,
            advertise_interval_ms: 100,
        }
    }
}

/// Relay adjustment parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayAdjustment {
    pub max_relay_per_hour: u32,
    pub priority_threshold: u8,
}

impl Default for RelayAdjustment {
    fn default() -> Self {
        Self {
            max_relay_per_hour: 100,
            priority_threshold: 50,
        }
    }
}

/// Manual overrides for fine-grained control
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManualOverride {
    pub ble_scan_interval_ms: Option<u16>,
    pub ble_advertise_interval_ms: Option<u16>,
    pub relay_max_per_hour: Option<u32>,
    pub relay_priority_threshold: Option<u8>,
}

/// Automatic adjustment engine
pub struct AutoAdjustEngine {
    manual_overrides: ManualOverride,
    last_profile: AdjustmentProfile,
}

impl AutoAdjustEngine {
    /// Create new engine with optional overrides
    pub fn new(overrides: Option<ManualOverride>) -> Self {
        Self {
            manual_overrides: overrides.unwrap_or_default(),
            last_profile: AdjustmentProfile::Standard,
        }
    }

    /// Determine adjustment profile from device state
    pub fn get_adjustment_profile(&self, profile: DeviceProfile) -> AdjustmentProfile {
        // Check for explicit disabling conditions first
        if profile.battery_percent < 10 {
            return AdjustmentProfile::Minimal;
        }

        // Screen on → more aggressive
        if profile.screen_on {
            if profile.battery_percent > 80 && profile.is_on_wifi {
                return AdjustmentProfile::Maximum;
            }
            if profile.battery_percent > 60 {
                return AdjustmentProfile::High;
            }
        }

        // Charging → favorable conditions
        if profile.is_charging {
            if profile.is_on_wifi {
                return AdjustmentProfile::Maximum;
            }
            return AdjustmentProfile::High;
        }

        // Motion affects profile
        match profile.motion_state {
            MotionState::Automotive => {
                // Moving fast: reduce overhead
                if profile.battery_percent > 40 && profile.is_on_wifi {
                    AdjustmentProfile::High
                } else if profile.battery_percent > 30 {
                    AdjustmentProfile::Reduced
                } else {
                    AdjustmentProfile::Minimal
                }
            }
            MotionState::Running | MotionState::Walking => {
                // Active: moderately reduced
                if profile.battery_percent > 30 {
                    AdjustmentProfile::Standard
                } else if profile.battery_percent > 15 {
                    AdjustmentProfile::Reduced
                } else {
                    AdjustmentProfile::Minimal
                }
            }
            MotionState::Still | MotionState::Unknown => {
                // Stationary: normal optimization
                if profile.battery_percent > 40 {
                    AdjustmentProfile::Standard
                } else if profile.battery_percent > 15 {
                    AdjustmentProfile::Reduced
                } else {
                    AdjustmentProfile::Minimal
                }
            }
        }
    }

    /// Apply adjustments to BLE parameters
    pub fn apply_ble_adjustments(&self, profile: AdjustmentProfile) -> BleAdjustment {
        // Check for manual overrides first
        if let Some(scan_interval) = self.manual_overrides.ble_scan_interval_ms {
            let mut adj = BleAdjustment::default();
            adj.scan_interval_ms = scan_interval;
            return adj;
        }

        match profile {
            AdjustmentProfile::Maximum => BleAdjustment {
                scan_interval_ms: 100,
                scan_window_ms: 100,
                advertise_interval_ms: 20,
            },
            AdjustmentProfile::High => BleAdjustment {
                scan_interval_ms: 500,
                scan_window_ms: 50,
                advertise_interval_ms: 50,
            },
            AdjustmentProfile::Standard => BleAdjustment {
                scan_interval_ms: 1280,
                scan_window_ms: 11,
                advertise_interval_ms: 100,
            },
            AdjustmentProfile::Reduced => BleAdjustment {
                scan_interval_ms: 2560,
                scan_window_ms: 10,
                advertise_interval_ms: 500,
            },
            AdjustmentProfile::Minimal => BleAdjustment {
                scan_interval_ms: 5120,
                scan_window_ms: 5,
                advertise_interval_ms: 2000,
            },
        }
    }

    /// Apply adjustments to relay parameters
    pub fn apply_relay_adjustments(&self, profile: AdjustmentProfile) -> RelayAdjustment {
        // Check for manual overrides first
        if let Some(max_relay) = self.manual_overrides.relay_max_per_hour {
            let mut adj = RelayAdjustment::default();
            adj.max_relay_per_hour = max_relay;
            return adj;
        }

        match profile {
            AdjustmentProfile::Maximum => RelayAdjustment {
                max_relay_per_hour: 500,
                priority_threshold: 10,
            },
            AdjustmentProfile::High => RelayAdjustment {
                max_relay_per_hour: 300,
                priority_threshold: 30,
            },
            AdjustmentProfile::Standard => RelayAdjustment {
                max_relay_per_hour: 100,
                priority_threshold: 50,
            },
            AdjustmentProfile::Reduced => RelayAdjustment {
                max_relay_per_hour: 30,
                priority_threshold: 70,
            },
            AdjustmentProfile::Minimal => RelayAdjustment {
                max_relay_per_hour: 5,
                priority_threshold: 90,
            },
        }
    }

    /// Set manual override for BLE scan interval
    pub fn override_ble_scan_interval(&mut self, interval_ms: Option<u16>) {
        self.manual_overrides.ble_scan_interval_ms = interval_ms;
    }

    /// Set manual override for BLE advertise interval
    pub fn override_ble_advertise_interval(&mut self, interval_ms: Option<u16>) {
        self.manual_overrides.ble_advertise_interval_ms = interval_ms;
    }

    /// Set manual override for relay max per hour
    pub fn override_relay_max_per_hour(&mut self, max: Option<u32>) {
        self.manual_overrides.relay_max_per_hour = max;
    }

    /// Set manual override for relay priority threshold
    pub fn override_relay_priority_threshold(&mut self, threshold: Option<u8>) {
        self.manual_overrides.relay_priority_threshold = threshold;
    }

    /// Clear all manual overrides
    pub fn clear_overrides(&mut self) {
        self.manual_overrides = ManualOverride::default();
    }

    /// Get current manual overrides
    pub fn get_overrides(&self) -> &ManualOverride {
        &self.manual_overrides
    }

    /// Get the last computed adjustment profile
    pub fn get_last_profile(&self) -> AdjustmentProfile {
        self.last_profile
    }
}

/// Comprehensive adjustment result
pub struct AdjustmentResult {
    pub profile: AdjustmentProfile,
    pub ble: BleAdjustment,
    pub relay: RelayAdjustment,
}

impl AutoAdjustEngine {
    /// Compute all adjustments from device profile
    pub fn compute_adjustments(&mut self, device: DeviceProfile) -> AdjustmentResult {
        let profile = self.get_adjustment_profile(device);
        self.last_profile = profile;

        AdjustmentResult {
            profile,
            ble: self.apply_ble_adjustments(profile),
            relay: self.apply_relay_adjustments(profile),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_battery() {
        let engine = AutoAdjustEngine::new(None);
        let device = DeviceProfile {
            battery_percent: 5,
            is_charging: false,
            is_on_wifi: true,
            motion_state: MotionState::Still,
            screen_on: true,
        };

        let profile = engine.get_adjustment_profile(device);
        assert_eq!(profile, AdjustmentProfile::Minimal);
    }

    #[test]
    fn test_screen_on_high_battery() {
        let engine = AutoAdjustEngine::new(None);
        let device = DeviceProfile {
            battery_percent: 85,
            is_charging: false,
            is_on_wifi: true,
            motion_state: MotionState::Still,
            screen_on: true,
        };

        let profile = engine.get_adjustment_profile(device);
        assert_eq!(profile, AdjustmentProfile::Maximum);
    }

    #[test]
    fn test_charging_with_wifi() {
        let engine = AutoAdjustEngine::new(None);
        let device = DeviceProfile {
            battery_percent: 40,
            is_charging: true,
            is_on_wifi: true,
            motion_state: MotionState::Still,
            screen_on: false,
        };

        let profile = engine.get_adjustment_profile(device);
        assert_eq!(profile, AdjustmentProfile::Maximum);
    }

    #[test]
    fn test_charging_without_wifi() {
        let engine = AutoAdjustEngine::new(None);
        let device = DeviceProfile {
            battery_percent: 40,
            is_charging: true,
            is_on_wifi: false,
            motion_state: MotionState::Still,
            screen_on: false,
        };

        let profile = engine.get_adjustment_profile(device);
        assert_eq!(profile, AdjustmentProfile::High);
    }

    #[test]
    fn test_automotive_mode() {
        let engine = AutoAdjustEngine::new(None);
        let device = DeviceProfile {
            battery_percent: 50,
            is_charging: false,
            is_on_wifi: true,
            motion_state: MotionState::Automotive,
            screen_on: false,
        };

        let profile = engine.get_adjustment_profile(device);
        assert_eq!(profile, AdjustmentProfile::High);
    }

    #[test]
    fn test_ble_adjustments_maximum() {
        let engine = AutoAdjustEngine::new(None);
        let adj = engine.apply_ble_adjustments(AdjustmentProfile::Maximum);

        assert_eq!(adj.scan_interval_ms, 100);
        assert_eq!(adj.advertise_interval_ms, 20);
    }

    #[test]
    fn test_ble_adjustments_minimal() {
        let engine = AutoAdjustEngine::new(None);
        let adj = engine.apply_ble_adjustments(AdjustmentProfile::Minimal);

        assert_eq!(adj.scan_interval_ms, 5120);
        assert_eq!(adj.advertise_interval_ms, 2000);
    }

    #[test]
    fn test_relay_adjustments_maximum() {
        let engine = AutoAdjustEngine::new(None);
        let adj = engine.apply_relay_adjustments(AdjustmentProfile::Maximum);

        assert_eq!(adj.max_relay_per_hour, 500);
        assert_eq!(adj.priority_threshold, 10);
    }

    #[test]
    fn test_relay_adjustments_minimal() {
        let engine = AutoAdjustEngine::new(None);
        let adj = engine.apply_relay_adjustments(AdjustmentProfile::Minimal);

        assert_eq!(adj.max_relay_per_hour, 5);
        assert_eq!(adj.priority_threshold, 90);
    }

    #[test]
    fn test_manual_override_ble_scan() {
        let mut override_cfg = ManualOverride::default();
        override_cfg.ble_scan_interval_ms = Some(999);

        let engine = AutoAdjustEngine::new(Some(override_cfg));
        let adj = engine.apply_ble_adjustments(AdjustmentProfile::Maximum);

        assert_eq!(adj.scan_interval_ms, 999);
    }

    #[test]
    fn test_manual_override_relay() {
        let mut override_cfg = ManualOverride::default();
        override_cfg.relay_max_per_hour = Some(42);

        let engine = AutoAdjustEngine::new(Some(override_cfg));
        let adj = engine.apply_relay_adjustments(AdjustmentProfile::Maximum);

        assert_eq!(adj.max_relay_per_hour, 42);
    }

    #[test]
    fn test_override_setter_methods() {
        let mut engine = AutoAdjustEngine::new(None);
        engine.override_ble_scan_interval(Some(555));
        engine.override_relay_max_per_hour(Some(77));

        let ble_adj = engine.apply_ble_adjustments(AdjustmentProfile::Standard);
        let relay_adj = engine.apply_relay_adjustments(AdjustmentProfile::Standard);

        assert_eq!(ble_adj.scan_interval_ms, 555);
        assert_eq!(relay_adj.max_relay_per_hour, 77);
    }

    #[test]
    fn test_clear_overrides() {
        let mut engine = AutoAdjustEngine::new(None);
        engine.override_ble_scan_interval(Some(555));
        engine.clear_overrides();

        let ble_adj = engine.apply_ble_adjustments(AdjustmentProfile::Standard);
        assert_eq!(ble_adj.scan_interval_ms, 1280);
    }

    #[test]
    fn test_compute_adjustments() {
        let mut engine = AutoAdjustEngine::new(None);
        let device = DeviceProfile {
            battery_percent: 85,
            is_charging: false,
            is_on_wifi: true,
            motion_state: MotionState::Still,
            screen_on: true,
        };

        let result = engine.compute_adjustments(device);
        assert_eq!(result.profile, AdjustmentProfile::Maximum);
        assert_eq!(result.ble.scan_interval_ms, 100);
        assert_eq!(result.relay.max_relay_per_hour, 500);
    }

    #[test]
    fn test_low_battery_walking() {
        let engine = AutoAdjustEngine::new(None);
        let device = DeviceProfile {
            battery_percent: 20,
            is_charging: false,
            is_on_wifi: false,
            motion_state: MotionState::Walking,
            screen_on: false,
        };

        let profile = engine.get_adjustment_profile(device);
        assert_eq!(profile, AdjustmentProfile::Reduced);
    }

    #[test]
    fn test_motion_state_still_vs_walking() {
        let engine = AutoAdjustEngine::new(None);

        let still_device = DeviceProfile {
            battery_percent: 45,
            is_charging: false,
            is_on_wifi: false,
            motion_state: MotionState::Still,
            screen_on: false,
        };

        let walking_device = DeviceProfile {
            battery_percent: 45,
            is_charging: false,
            is_on_wifi: false,
            motion_state: MotionState::Walking,
            screen_on: false,
        };

        let still_profile = engine.get_adjustment_profile(still_device);
        let walking_profile = engine.get_adjustment_profile(walking_device);

        assert_eq!(still_profile, AdjustmentProfile::Standard);
        assert_eq!(walking_profile, AdjustmentProfile::Standard);
    }
}
