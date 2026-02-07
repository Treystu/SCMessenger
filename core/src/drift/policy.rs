//! Smart auto-adjust system for relay aggressiveness based on device state

use super::relay::RelayConfig;
use thiserror::Error;

/// Device state information (provided by platform layer)
#[derive(Debug, Clone)]
pub struct DeviceState {
    pub battery_percent: u8,
    pub is_charging: bool,
    pub has_wifi: bool,
    pub is_moving: bool, // Based on accelerometer/GPS
    pub timestamp: u64,
}

/// Auto-computed relay aggressiveness profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayProfile {
    /// Charging + WiFi: maximum relay aggressiveness
    Maximum,
    /// Charging, no WiFi: high relay
    High,
    /// Battery > 50%: standard relay
    Standard,
    /// Battery 20-50%: reduced relay
    Reduced,
    /// Battery < 20%: minimal relay (passive only)
    Minimal,
}

/// Policy engine errors
#[derive(Debug, Error, Clone)]
pub enum PolicyError {
    #[error("Relay budget cannot be zero — violates relay=messaging coupling")]
    RelayBudgetCannotBeZero,
}

/// Policy engine computes relay parameters from device state
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    /// User overrides (None = use auto)
    scan_interval_override: Option<u64>,
    relay_budget_override: Option<u32>,
    battery_floor_override: Option<u8>,
    /// Current computed profile
    current_profile: RelayProfile,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            scan_interval_override: None,
            relay_budget_override: None,
            battery_floor_override: None,
            current_profile: RelayProfile::Standard,
        }
    }

    /// Update device state and recompute profile
    pub fn update_device_state(&mut self, state: &DeviceState) -> RelayProfile {
        self.current_profile = self.compute_profile(state);
        self.current_profile
    }

    /// Get current scan interval in milliseconds
    pub fn scan_interval_ms(&self) -> u64 {
        if let Some(override_val) = self.scan_interval_override {
            return override_val;
        }

        match self.current_profile {
            RelayProfile::Maximum => 500,   // Very frequent
            RelayProfile::High => 1000,     // Frequent
            RelayProfile::Standard => 5000, // Every 5 seconds
            RelayProfile::Reduced => 15000, // Every 15 seconds
            RelayProfile::Minimal => 60000, // Every 1 minute
        }
    }

    /// Get current max relay messages per hour
    pub fn relay_budget_per_hour(&self) -> u32 {
        if let Some(override_val) = self.relay_budget_override {
            return override_val;
        }

        match self.current_profile {
            RelayProfile::Maximum => 5000,
            RelayProfile::High => 3000,
            RelayProfile::Standard => 1000,
            RelayProfile::Reduced => 300,
            RelayProfile::Minimal => 50,
        }
    }

    /// Get the RelayConfig to apply to the RelayEngine
    pub fn to_relay_config(&self) -> RelayConfig {
        let battery_floor = self
            .battery_floor_override
            .unwrap_or_else(|| match self.current_profile {
                RelayProfile::Maximum => 10,
                RelayProfile::High => 15,
                RelayProfile::Standard => 20,
                RelayProfile::Reduced => 30,
                RelayProfile::Minimal => 50,
            });

        RelayConfig {
            max_relay_per_hour: self.relay_budget_per_hour(),
            max_hop_count: 16,
            min_relay_priority: 0,
            battery_floor_percent: battery_floor,
            relay_opaque: true,
        }
    }

    /// User override setters (respect the coupling: can't set relay to 0 while messaging)
    pub fn set_scan_interval_override(&mut self, ms: Option<u64>) {
        self.scan_interval_override = ms;
    }

    /// Set relay budget override (enforces coupling: must be > 0)
    pub fn set_relay_budget_override(&mut self, budget: Option<u32>) -> Result<(), PolicyError> {
        if let Some(0) = budget {
            return Err(PolicyError::RelayBudgetCannotBeZero);
        }
        self.relay_budget_override = budget;
        Ok(())
    }

    /// Set battery floor override
    pub fn set_battery_floor_override(&mut self, percent: Option<u8>) {
        self.battery_floor_override = percent;
    }

    /// Check if device should be in reduced mode
    pub fn should_reduce(&self) -> bool {
        matches!(
            self.current_profile,
            RelayProfile::Reduced | RelayProfile::Minimal
        )
    }

    /// Get current profile (for testing/diagnostics)
    pub fn current_profile(&self) -> RelayProfile {
        self.current_profile
    }

    /// Compute profile from device state
    fn compute_profile(&self, state: &DeviceState) -> RelayProfile {
        // Charging + WiFi = Maximum
        if state.is_charging && state.has_wifi {
            return RelayProfile::Maximum;
        }

        // Charging without WiFi = High
        if state.is_charging {
            return RelayProfile::High;
        }

        // Not charging — check battery
        match state.battery_percent {
            51..=100 => RelayProfile::Standard,
            20..=50 => RelayProfile::Reduced,
            _ => RelayProfile::Minimal,
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_device_state(
        battery: u8,
        charging: bool,
        wifi: bool,
        moving: bool,
    ) -> DeviceState {
        DeviceState {
            battery_percent: battery,
            is_charging: charging,
            has_wifi: wifi,
            is_moving: moving,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    #[test]
    fn test_profile_maximum() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::Maximum);
    }

    #[test]
    fn test_profile_high() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, false, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::High);
    }

    #[test]
    fn test_profile_standard_high_battery() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(75, false, false, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::Standard);
    }

    #[test]
    fn test_profile_standard_at_boundary() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(51, false, false, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::Standard);
    }

    #[test]
    fn test_profile_reduced_mid_battery() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(35, false, false, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::Reduced);
    }

    #[test]
    fn test_profile_reduced_at_boundary_high() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(50, false, false, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::Reduced);
    }

    #[test]
    fn test_profile_reduced_at_boundary_low() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(20, false, false, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::Reduced);
    }

    #[test]
    fn test_profile_minimal_low_battery() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(19, false, false, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::Minimal);
    }

    #[test]
    fn test_profile_minimal_critical() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(5, false, false, false);

        let profile = engine.update_device_state(&state);
        assert_eq!(profile, RelayProfile::Minimal);
    }

    #[test]
    fn test_scan_interval_maximum() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);
        engine.update_device_state(&state);

        assert_eq!(engine.scan_interval_ms(), 500);
    }

    #[test]
    fn test_scan_interval_high() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.scan_interval_ms(), 1000);
    }

    #[test]
    fn test_scan_interval_standard() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(75, false, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.scan_interval_ms(), 5000);
    }

    #[test]
    fn test_scan_interval_reduced() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(35, false, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.scan_interval_ms(), 15000);
    }

    #[test]
    fn test_scan_interval_minimal() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(10, false, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.scan_interval_ms(), 60000);
    }

    #[test]
    fn test_scan_interval_override() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);
        engine.update_device_state(&state);

        // Override should work
        engine.set_scan_interval_override(Some(2000));
        assert_eq!(engine.scan_interval_ms(), 2000);

        // Clear override
        engine.set_scan_interval_override(None);
        assert_eq!(engine.scan_interval_ms(), 500);
    }

    #[test]
    fn test_relay_budget_maximum() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);
        engine.update_device_state(&state);

        assert_eq!(engine.relay_budget_per_hour(), 5000);
    }

    #[test]
    fn test_relay_budget_high() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.relay_budget_per_hour(), 3000);
    }

    #[test]
    fn test_relay_budget_standard() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(75, false, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.relay_budget_per_hour(), 1000);
    }

    #[test]
    fn test_relay_budget_reduced() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(35, false, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.relay_budget_per_hour(), 300);
    }

    #[test]
    fn test_relay_budget_minimal() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(10, false, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.relay_budget_per_hour(), 50);
    }

    #[test]
    fn test_relay_budget_override() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(75, false, false, false);
        engine.update_device_state(&state);

        assert_eq!(engine.relay_budget_per_hour(), 1000);

        // Override should work
        let result = engine.set_relay_budget_override(Some(2000));
        assert!(result.is_ok());
        assert_eq!(engine.relay_budget_per_hour(), 2000);

        // Clear override
        let _ = engine.set_relay_budget_override(None);
        assert_eq!(engine.relay_budget_per_hour(), 1000);
    }

    #[test]
    fn test_coupling_relay_budget_cannot_be_zero() {
        let mut engine = PolicyEngine::new();

        // This should fail — enforces coupling
        let result = engine.set_relay_budget_override(Some(0));
        assert!(matches!(
            result,
            Err(PolicyError::RelayBudgetCannotBeZero)
        ));

        // Budget should not have changed
        assert_eq!(engine.relay_budget_per_hour(), 1000);
    }

    #[test]
    fn test_battery_floor_maximum() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);
        engine.update_device_state(&state);

        let config = engine.to_relay_config();
        assert_eq!(config.battery_floor_percent, 10);
    }

    #[test]
    fn test_battery_floor_minimal() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(10, false, false, false);
        engine.update_device_state(&state);

        let config = engine.to_relay_config();
        assert_eq!(config.battery_floor_percent, 50);
    }

    #[test]
    fn test_battery_floor_override() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(75, false, false, false);
        engine.update_device_state(&state);

        engine.set_battery_floor_override(Some(25));
        let config = engine.to_relay_config();
        assert_eq!(config.battery_floor_percent, 25);

        // Clear override
        engine.set_battery_floor_override(None);
        let config2 = engine.to_relay_config();
        assert_eq!(config2.battery_floor_percent, 20);
    }

    #[test]
    fn test_to_relay_config_maximum() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);
        engine.update_device_state(&state);

        let config = engine.to_relay_config();
        assert_eq!(config.max_relay_per_hour, 5000);
        assert_eq!(config.max_hop_count, 16);
        assert_eq!(config.min_relay_priority, 0);
        assert_eq!(config.battery_floor_percent, 10);
        assert!(config.relay_opaque);
    }

    #[test]
    fn test_to_relay_config_minimal() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(10, false, false, false);
        engine.update_device_state(&state);

        let config = engine.to_relay_config();
        assert_eq!(config.max_relay_per_hour, 50);
        assert_eq!(config.battery_floor_percent, 50);
    }

    #[test]
    fn test_should_reduce_false_for_standard() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(75, false, false, false);
        engine.update_device_state(&state);

        assert!(!engine.should_reduce());
    }

    #[test]
    fn test_should_reduce_false_for_maximum() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);
        engine.update_device_state(&state);

        assert!(!engine.should_reduce());
    }

    #[test]
    fn test_should_reduce_true_for_reduced() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(35, false, false, false);
        engine.update_device_state(&state);

        assert!(engine.should_reduce());
    }

    #[test]
    fn test_should_reduce_true_for_minimal() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(10, false, false, false);
        engine.update_device_state(&state);

        assert!(engine.should_reduce());
    }

    #[test]
    fn test_default_policy_engine() {
        let engine = PolicyEngine::default();
        assert_eq!(engine.current_profile(), RelayProfile::Standard);
    }

    #[test]
    fn test_current_profile_getter() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);

        engine.update_device_state(&state);
        assert_eq!(engine.current_profile(), RelayProfile::Maximum);
    }

    #[test]
    fn test_profile_transitions() {
        let mut engine = PolicyEngine::new();

        // Start at high battery
        let state1 = make_device_state(80, false, false, false);
        engine.update_device_state(&state1);
        assert_eq!(engine.current_profile(), RelayProfile::Standard);

        // Drop to mid battery
        let state2 = make_device_state(35, false, false, false);
        engine.update_device_state(&state2);
        assert_eq!(engine.current_profile(), RelayProfile::Reduced);

        // Plug in
        let state3 = make_device_state(35, true, false, false);
        engine.update_device_state(&state3);
        assert_eq!(engine.current_profile(), RelayProfile::High);

        // Connect WiFi
        let state4 = make_device_state(35, true, true, false);
        engine.update_device_state(&state4);
        assert_eq!(engine.current_profile(), RelayProfile::Maximum);

        // Unplug
        let state5 = make_device_state(35, false, true, false);
        engine.update_device_state(&state5);
        assert_eq!(engine.current_profile(), RelayProfile::Reduced);
    }

    #[test]
    fn test_relay_config_has_correct_defaults() {
        let engine = PolicyEngine::new();
        let config = engine.to_relay_config();

        assert_eq!(config.max_hop_count, 16);
        assert_eq!(config.min_relay_priority, 0);
        assert!(config.relay_opaque);
    }

    #[test]
    fn test_override_combinations() {
        let mut engine = PolicyEngine::new();
        let state = make_device_state(100, true, true, false);
        engine.update_device_state(&state);

        // Set all overrides
        engine.set_scan_interval_override(Some(1000));
        let _ = engine.set_relay_budget_override(Some(2000));
        engine.set_battery_floor_override(Some(15));

        assert_eq!(engine.scan_interval_ms(), 1000);
        assert_eq!(engine.relay_budget_per_hour(), 2000);

        let config = engine.to_relay_config();
        assert_eq!(config.battery_floor_percent, 15);
    }
}
