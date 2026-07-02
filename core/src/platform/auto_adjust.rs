//! Smart automatic resource adjustment based on device state
//!
//! Profiles device conditions (battery, movement, connectivity) and automatically
//! adjusts mesh parameters (scan intervals, relay budgets) to balance performance
//! with power consumption.

use serde::{Deserialize, Serialize};

// ============================================================================
// DEVICE STATE
// ============================================================================

/// Current device state for adjustment profiling
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DeviceState {
    /// Battery percentage (0-100)
    pub battery_percent: u8,
    /// Is device currently charging
    pub is_charging: bool,
    /// Is device connected to WiFi
    pub is_on_wifi: bool,
    /// Is device moving (detected via accelerometer)
    pub is_moving: bool,
    /// Is screen currently on
    pub screen_on: bool,
    /// Time since last user interaction in seconds
    pub time_since_last_interaction_secs: u64,
}

// ============================================================================
// PROFILES & RESULTS
// ============================================================================

/// Adjustment profile for mesh operations
///
/// Maps to relay policies: determine scan intervals, relay budgets,
/// and transport preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjustmentProfile {
    /// Aggressive scanning, full relay capacity
    /// Use when charging + connected
    Maximum,

    /// Aggressive scanning, mesh-only relay (no internet)
    /// Use when charging but no WiFi
    High,

    /// Normal operation
    /// Use when battery > 50%
    Standard,

    /// Power-saving mode
    /// Use when battery 20-50%
    Reduced,

    /// Minimal operation, BLE only
    /// Use when battery < 20%
    Minimal,
}

impl std::fmt::Display for AdjustmentProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Maximum => write!(f, "Maximum"),
            Self::High => write!(f, "High"),
            Self::Standard => write!(f, "Standard"),
            Self::Reduced => write!(f, "Reduced"),
            Self::Minimal => write!(f, "Minimal"),
        }
    }
}

/// Specific adjustment recommendations for platform code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustmentResult {
    /// Recommended scan interval in milliseconds
    pub scan_interval_ms: u32,
    /// Max messages to relay per hour
    pub relay_budget_per_hour: u32,
    /// Enable WiFi Aware scanning
    pub enable_wifi_aware: bool,
    /// Allow relaying over internet gateway
    pub enable_internet_relay: bool,
    /// BLE duty cycle (0-100, percentage of time radio on)
    pub ble_duty_cycle: u8,
}

// ============================================================================
// SMART AUTO-ADJUST ENGINE
// ============================================================================

/// Automatic adjustment engine that profiles device state and recommends settings
#[derive(Debug, Clone)]
pub struct SmartAutoAdjust {
    // State engine is stateless - all logic is deterministic
}

impl SmartAutoAdjust {
    /// Create a new auto-adjustment engine
    pub fn new() -> Self {
        Self {}
    }

    /// Compute the best adjustment profile for current device state
    ///
    /// Algorithm:
    /// 1. Check power state: charging + WiFi → Maximum
    /// 2. Check power state: charging only → High
    /// 3. Check battery level: > 50% → Standard, 20-50% → Reduced, < 20% → Minimal
    /// 4. Check movement: if moving, increase scan frequency by 2x
    /// 5. Check proximity: stationary → decrease scan frequency
    pub fn compute_profile(&self, device_state: &DeviceState) -> AdjustmentProfile {
        // Validate battery percentage
        let battery_percent = device_state.battery_percent.min(100);

        // Power state takes priority
        if device_state.is_charging {
            return if device_state.is_on_wifi {
                AdjustmentProfile::Maximum
            } else {
                AdjustmentProfile::High
            };
        }

        // Battery-based profile for non-charging state
        match battery_percent {
            80..=100 => AdjustmentProfile::Standard,
            50..=79 => AdjustmentProfile::Standard,
            20..=49 => AdjustmentProfile::Reduced,
            _ => AdjustmentProfile::Minimal,
        }
    }

    /// Compute detailed adjustment recommendations for a given profile and platform
    ///
    /// These values should be used by platform code to configure the mesh:
    /// - scan_interval_ms: how often to scan for peers
    /// - relay_budget_per_hour: max messages to relay (throttle on battery)
    /// - enable_wifi_aware: use WiFi Aware (Android) for scanning
    /// - enable_internet_relay: relay through internet gateway
    /// - ble_duty_cycle: what percentage of time to keep BLE radio on
    pub fn compute_adjustment(
        &self,
        profile: AdjustmentProfile,
        _platform_max_background_secs: u32,
    ) -> AdjustmentResult {
        match profile {
            AdjustmentProfile::Maximum => AdjustmentResult {
                scan_interval_ms: 5000,      // Aggressive: 5 seconds
                relay_budget_per_hour: 1000, // Full capacity
                enable_wifi_aware: true,
                enable_internet_relay: true,
                ble_duty_cycle: 100, // Always on
            },

            AdjustmentProfile::High => AdjustmentResult {
                scan_interval_ms: 10000,    // Still aggressive: 10 seconds
                relay_budget_per_hour: 800, // High but not maximum
                enable_wifi_aware: true,
                enable_internet_relay: false, // Mesh-only
                ble_duty_cycle: 90,           // Almost always on
            },

            AdjustmentProfile::Standard => AdjustmentResult {
                scan_interval_ms: 30000,    // Normal: 30 seconds
                relay_budget_per_hour: 300, // Moderate
                enable_wifi_aware: true,
                enable_internet_relay: true,
                ble_duty_cycle: 50, // Normal duty cycle
            },

            AdjustmentProfile::Reduced => AdjustmentResult {
                scan_interval_ms: 120000,  // 2 minutes: less frequent
                relay_budget_per_hour: 50, // Low relay budget
                enable_wifi_aware: false,  // Disable WiFi Aware to save power
                enable_internet_relay: false,
                ble_duty_cycle: 20, // Low duty cycle
            },

            AdjustmentProfile::Minimal => AdjustmentResult {
                scan_interval_ms: 300000, // 5 minutes: very infrequent
                relay_budget_per_hour: 5, // Minimal relay
                enable_wifi_aware: false, // Disabled
                enable_internet_relay: false,
                ble_duty_cycle: 5, // Very low duty cycle
            },
        }
    }

    /// Adjust recommendation based on movement state
    ///
    /// If device is moving, increase scan frequency (finding new peers)
    /// If stationary, decrease scan frequency (same peers)
    pub fn apply_movement_adjustment(
        &self,
        mut result: AdjustmentResult,
        is_moving: bool,
    ) -> AdjustmentResult {
        if is_moving {
            // Increase scan frequency by 2x (divide interval by 2)
            result.scan_interval_ms = (result.scan_interval_ms / 2).max(1000);
            result.ble_duty_cycle = result.ble_duty_cycle.saturating_add(10).min(100);
        } else {
            // Decrease scan frequency by 2x (multiply interval by 2)
            result.scan_interval_ms = result.scan_interval_ms.saturating_mul(2);
            result.ble_duty_cycle = result.ble_duty_cycle.saturating_sub(10);
        }

        result
    }

    /// iOS-specific adjustment to respect background execution time limit
    ///
    /// iOS has strict background execution limits (typically 30 seconds).
    /// This method adjusts the scan interval to fit within that window.
    pub fn apply_ios_background_adjustment(
        &self,
        mut result: AdjustmentResult,
        max_background_secs: u32,
    ) -> AdjustmentResult {
        // iOS limit: if background time is limited, adjust scan interval
        if max_background_secs < 60 {
            // Very limited (< 1 minute): reduce scan interval to fit in window
            result.scan_interval_ms = (max_background_secs as u32).saturating_mul(800);
            result.ble_duty_cycle = 25;
        }

        result
    }
}

impl Default for SmartAutoAdjust {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_device_state(
        battery: u8,
        charging: bool,
        wifi: bool,
        moving: bool,
        screen: bool,
    ) -> DeviceState {
        DeviceState {
            battery_percent: battery,
            is_charging: charging,
            is_on_wifi: wifi,
            is_moving: moving,
            screen_on: screen,
            time_since_last_interaction_secs: 0,
        }
    }

    #[test]
    fn test_profile_charging_with_wifi() {
        let engine = SmartAutoAdjust::new();
        let state = make_device_state(80, true, true, false, true);

        let profile = engine.compute_profile(&state);
        assert_eq!(profile, AdjustmentProfile::Maximum);
    }

    #[test]
    fn test_profile_charging_without_wifi() {
        let engine = SmartAutoAdjust::new();
        let state = make_device_state(80, true, false, false, true);

        let profile = engine.compute_profile(&state);
        assert_eq!(profile, AdjustmentProfile::High);
    }

    #[test]
    fn test_profile_battery_high() {
        let engine = SmartAutoAdjust::new();
        let state = make_device_state(80, false, false, false, false);

        let profile = engine.compute_profile(&state);
        assert_eq!(profile, AdjustmentProfile::Standard);
    }

    #[test]
    fn test_profile_battery_medium() {
        let engine = SmartAutoAdjust::new();
        let state = make_device_state(35, false, false, false, false);

        let profile = engine.compute_profile(&state);
        assert_eq!(profile, AdjustmentProfile::Reduced);
    }

    #[test]
    fn test_profile_battery_low() {
        let engine = SmartAutoAdjust::new();
        let state = make_device_state(10, false, false, false, false);

        let profile = engine.compute_profile(&state);
        assert_eq!(profile, AdjustmentProfile::Minimal);
    }

    #[test]
    fn test_profile_battery_critical_boundary() {
        let engine = SmartAutoAdjust::new();

        // 20% should be Reduced
        let state = make_device_state(20, false, false, false, false);
        assert_eq!(engine.compute_profile(&state), AdjustmentProfile::Reduced);

        // 19% should be Minimal
        let state = make_device_state(19, false, false, false, false);
        assert_eq!(engine.compute_profile(&state), AdjustmentProfile::Minimal);

        // 50% should be Standard
        let state = make_device_state(50, false, false, false, false);
        assert_eq!(engine.compute_profile(&state), AdjustmentProfile::Standard);

        // 49% should be Reduced
        let state = make_device_state(49, false, false, false, false);
        assert_eq!(engine.compute_profile(&state), AdjustmentProfile::Reduced);
    }

    #[test]
    fn test_adjustment_maximum_profile() {
        let engine = SmartAutoAdjust::new();
        let result = engine.compute_adjustment(AdjustmentProfile::Maximum, 600);

        assert_eq!(result.scan_interval_ms, 5000);
        assert_eq!(result.relay_budget_per_hour, 1000);
        assert!(result.enable_wifi_aware);
        assert!(result.enable_internet_relay);
        assert_eq!(result.ble_duty_cycle, 100);
    }

    #[test]
    fn test_adjustment_high_profile() {
        let engine = SmartAutoAdjust::new();
        let result = engine.compute_adjustment(AdjustmentProfile::High, 600);

        assert_eq!(result.scan_interval_ms, 10000);
        assert_eq!(result.relay_budget_per_hour, 800);
        assert!(result.enable_wifi_aware);
        assert!(!result.enable_internet_relay); // Mesh-only
        assert_eq!(result.ble_duty_cycle, 90);
    }

    #[test]
    fn test_adjustment_standard_profile() {
        let engine = SmartAutoAdjust::new();
        let result = engine.compute_adjustment(AdjustmentProfile::Standard, 600);

        assert_eq!(result.scan_interval_ms, 30000);
        assert_eq!(result.relay_budget_per_hour, 300);
        assert!(result.enable_wifi_aware);
        assert!(result.enable_internet_relay);
        assert_eq!(result.ble_duty_cycle, 50);
    }

    #[test]
    fn test_adjustment_reduced_profile() {
        let engine = SmartAutoAdjust::new();
        let result = engine.compute_adjustment(AdjustmentProfile::Reduced, 600);

        assert_eq!(result.scan_interval_ms, 120000);
        assert_eq!(result.relay_budget_per_hour, 50);
        assert!(!result.enable_wifi_aware);
        assert!(!result.enable_internet_relay);
        assert_eq!(result.ble_duty_cycle, 20);
    }

    #[test]
    fn test_adjustment_minimal_profile() {
        let engine = SmartAutoAdjust::new();
        let result = engine.compute_adjustment(AdjustmentProfile::Minimal, 600);

        assert_eq!(result.scan_interval_ms, 300000);
        assert_eq!(result.relay_budget_per_hour, 5);
        assert!(!result.enable_wifi_aware);
        assert!(!result.enable_internet_relay);
        assert_eq!(result.ble_duty_cycle, 5);
    }

    #[test]
    fn test_movement_adjustment_moving() {
        let engine = SmartAutoAdjust::new();
        let base = engine.compute_adjustment(AdjustmentProfile::Standard, 600);

        let adjusted = engine.apply_movement_adjustment(base.clone(), true);

        // Moving should halve the scan interval
        assert_eq!(adjusted.scan_interval_ms, base.scan_interval_ms / 2);
        // Should increase duty cycle
        assert!(adjusted.ble_duty_cycle > base.ble_duty_cycle);
    }

    #[test]
    fn test_movement_adjustment_stationary() {
        let engine = SmartAutoAdjust::new();
        let base = engine.compute_adjustment(AdjustmentProfile::Standard, 600);

        let adjusted = engine.apply_movement_adjustment(base.clone(), false);

        // Stationary should double the scan interval
        assert_eq!(adjusted.scan_interval_ms, base.scan_interval_ms * 2);
        // Should decrease duty cycle
        assert!(adjusted.ble_duty_cycle < base.ble_duty_cycle);
    }

    #[test]
    fn test_ios_background_adjustment_limited() {
        let engine = SmartAutoAdjust::new();
        let base = engine.compute_adjustment(AdjustmentProfile::Standard, 30);

        let adjusted = engine.apply_ios_background_adjustment(base.clone(), 30);

        // With 30 second limit, scan interval should be reduced
        assert!(adjusted.scan_interval_ms <= base.scan_interval_ms);
        assert!(adjusted.ble_duty_cycle < base.ble_duty_cycle);
    }

    #[test]
    fn test_ios_background_adjustment_generous() {
        let engine = SmartAutoAdjust::new();
        let base = engine.compute_adjustment(AdjustmentProfile::Standard, 600);

        let adjusted = engine.apply_ios_background_adjustment(base.clone(), 600);

        // With generous limit, should be unchanged
        assert_eq!(adjusted.scan_interval_ms, base.scan_interval_ms);
    }

    #[test]
    fn test_profile_display() {
        assert_eq!(format!("{}", AdjustmentProfile::Maximum), "Maximum");
        assert_eq!(format!("{}", AdjustmentProfile::Minimal), "Minimal");
        assert_eq!(format!("{}", AdjustmentProfile::Standard), "Standard");
    }

    #[test]
    fn test_battery_percent_overflow() {
        let engine = SmartAutoAdjust::new();
        let state = make_device_state(255, false, false, false, false);

        let profile = engine.compute_profile(&state);
        // Should clamp to 100, which is > 50, so Standard
        assert_eq!(profile, AdjustmentProfile::Standard);
    }

    #[test]
    fn test_movement_adjustment_clamping() {
        let engine = SmartAutoAdjust::new();
        let base = AdjustmentResult {
            scan_interval_ms: 1000,
            relay_budget_per_hour: 100,
            enable_wifi_aware: true,
            enable_internet_relay: true,
            ble_duty_cycle: 100,
        };

        let adjusted = engine.apply_movement_adjustment(base, true);

        // Moving + 1000ms should not go below 1000 (min constraint)
        assert!(adjusted.scan_interval_ms >= 1000);
        // Duty cycle at 100 should not exceed 100
        assert!(adjusted.ble_duty_cycle <= 100);
    }

    #[test]
    fn test_movement_adjustment_underflow() {
        let engine = SmartAutoAdjust::new();
        let base = AdjustmentResult {
            scan_interval_ms: 300000,
            relay_budget_per_hour: 5,
            enable_wifi_aware: false,
            enable_internet_relay: false,
            ble_duty_cycle: 5,
        };

        let adjusted = engine.apply_movement_adjustment(base, false);

        // Stationary + 5 duty cycle should be low but valid
        let _ = adjusted.ble_duty_cycle; // type is unsigned, always >= 0
    }

    #[test]
    fn test_all_profiles_have_sensible_values() {
        let engine = SmartAutoAdjust::new();

        for profile in &[
            AdjustmentProfile::Maximum,
            AdjustmentProfile::High,
            AdjustmentProfile::Standard,
            AdjustmentProfile::Reduced,
            AdjustmentProfile::Minimal,
        ] {
            let result = engine.compute_adjustment(*profile, 600);

            // All intervals should be positive
            assert!(result.scan_interval_ms > 0);
            // relay_budget_per_hour is unsigned, always valid
            // Duty cycle should be 0-100
            assert!(result.ble_duty_cycle <= 100);
        }
    }

    #[test]
    fn test_monotonic_scan_intervals() {
        let engine = SmartAutoAdjust::new();

        let profiles = vec![
            AdjustmentProfile::Maximum,
            AdjustmentProfile::High,
            AdjustmentProfile::Standard,
            AdjustmentProfile::Reduced,
            AdjustmentProfile::Minimal,
        ];

        let mut prev_interval = 0;
        for profile in profiles {
            let result = engine.compute_adjustment(profile, 600);
            // Intervals should increase as we go down in power
            assert!(result.scan_interval_ms >= prev_interval);
            prev_interval = result.scan_interval_ms;
        }
    }
}
