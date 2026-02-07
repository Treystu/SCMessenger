//! iOS background mode orchestration
//!
//! Manages iOS-specific background capabilities including BLE modes,
//! location services, background fetch, and processing.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackgroundMode {
    BluetoothCentral,
    BluetoothPeripheral,
    Location,
    BackgroundFetch,
    BackgroundProcessing,
}

impl std::fmt::Display for BackgroundMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackgroundMode::BluetoothCentral => write!(f, "BluetoothCentral"),
            BackgroundMode::BluetoothPeripheral => write!(f, "BluetoothPeripheral"),
            BackgroundMode::Location => write!(f, "Location"),
            BackgroundMode::BackgroundFetch => write!(f, "BackgroundFetch"),
            BackgroundMode::BackgroundProcessing => write!(f, "BackgroundProcessing"),
        }
    }
}

/// Core Bluetooth state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothState {
    Inactive,
    Active,
    Restricted,
    Unknown,
}

/// CoreBluetooth activity state
#[derive(Debug, Clone, Copy)]
pub struct CoreBluetoothState {
    pub central_active: bool,
    pub peripheral_active: bool,
    pub state: BluetoothState,
}

impl CoreBluetoothState {
    pub fn is_any_active(&self) -> bool {
        self.central_active || self.peripheral_active
    }

    pub fn new() -> Self {
        Self {
            central_active: false,
            peripheral_active: false,
            state: BluetoothState::Inactive,
        }
    }

    pub fn set_central_active(&mut self, active: bool) {
        self.central_active = active;
        if active {
            self.state = BluetoothState::Active;
        }
    }

    pub fn set_peripheral_active(&mut self, active: bool) {
        self.peripheral_active = active;
        if active {
            self.state = BluetoothState::Active;
        }
    }

    pub fn set_restricted(&mut self, restricted: bool) {
        if restricted {
            self.state = BluetoothState::Restricted;
        } else if self.is_any_active() {
            self.state = BluetoothState::Active;
        } else {
            self.state = BluetoothState::Inactive;
        }
    }
}

impl Default for CoreBluetoothState {
    fn default() -> Self {
        Self::new()
    }
}

/// iOS background configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosBackgroundConfig {
    pub enabled_modes: HashSet<BackgroundMode>,
    pub fetch_interval_secs: u32,
    pub location_accuracy: LocationAccuracy,
    pub allow_always_location: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationAccuracy {
    FullAccuracy,
    ReducedAccuracy,
    Disabled,
}

impl Default for IosBackgroundConfig {
    fn default() -> Self {
        let mut enabled_modes = HashSet::new();
        enabled_modes.insert(BackgroundMode::BluetoothCentral);
        enabled_modes.insert(BackgroundMode::BluetoothPeripheral);
        enabled_modes.insert(BackgroundMode::BackgroundFetch);

        Self {
            enabled_modes,
            fetch_interval_secs: 900, // 15 minutes
            location_accuracy: LocationAccuracy::ReducedAccuracy,
            allow_always_location: false,
        }
    }
}

impl IosBackgroundConfig {
    /// Check if a mode is enabled
    pub fn is_mode_enabled(&self, mode: BackgroundMode) -> bool {
        self.enabled_modes.contains(&mode)
    }

    /// Enable a background mode
    pub fn enable_mode(&mut self, mode: BackgroundMode) {
        self.enabled_modes.insert(mode);
    }

    /// Disable a background mode
    pub fn disable_mode(&mut self, mode: BackgroundMode) {
        self.enabled_modes.remove(&mode);
    }

    /// Get all enabled modes as vector
    pub fn get_enabled_modes(&self) -> Vec<BackgroundMode> {
        let mut modes: Vec<_> = self.enabled_modes.iter().copied().collect();
        modes.sort_by_key(|m| format!("{:?}", m));
        modes
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.fetch_interval_secs < 60 {
            return Err("fetch_interval must be at least 60 seconds".to_string());
        }

        if self.fetch_interval_secs > 86400 {
            return Err("fetch_interval cannot exceed 24 hours".to_string());
        }

        if self.enabled_modes.is_empty() {
            return Err("At least one background mode must be enabled".to_string());
        }

        Ok(())
    }
}

/// iOS background strategy orchestrator
pub struct IosBackgroundStrategy {
    config: IosBackgroundConfig,
    bluetooth_state: CoreBluetoothState,
}

impl IosBackgroundStrategy {
    /// Create a new iOS background strategy
    pub fn new(config: IosBackgroundConfig) -> Result<Self, String> {
        config.validate()?;

        Ok(Self {
            config,
            bluetooth_state: CoreBluetoothState::default(),
        })
    }

    /// Get the configuration
    pub fn get_config(&self) -> &IosBackgroundConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: IosBackgroundConfig) -> Result<(), String> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// Get CoreBluetooth state
    pub fn get_bluetooth_state(&self) -> CoreBluetoothState {
        self.bluetooth_state
    }

    /// Update CoreBluetooth state
    pub fn set_bluetooth_state(&mut self, state: CoreBluetoothState) {
        self.bluetooth_state = state;
    }

    /// Schedule background fetch
    pub fn schedule_background_fetch(&self) -> Result<(), String> {
        if !self.config.is_mode_enabled(BackgroundMode::BackgroundFetch) {
            return Err("BackgroundFetch mode not enabled".to_string());
        }

        tracing::info!(
            "Scheduling background fetch every {} seconds",
            self.config.fetch_interval_secs
        );
        Ok(())
    }

    /// Handle background fetch wakeup
    pub fn on_background_fetch(&mut self) -> Result<(), String> {
        if !self.config.is_mode_enabled(BackgroundMode::BackgroundFetch) {
            return Err("BackgroundFetch not enabled".to_string());
        }

        tracing::info!("Background fetch triggered");

        // Request Bluetooth operations if enabled
        if self.config.is_mode_enabled(BackgroundMode::BluetoothCentral) {
            self.bluetooth_state.set_central_active(true);
        }

        if self.config.is_mode_enabled(BackgroundMode::BluetoothPeripheral) {
            self.bluetooth_state.set_peripheral_active(true);
        }

        Ok(())
    }

    /// Enable location-based background operation
    pub fn enable_location_background(&mut self) -> Result<(), String> {
        if !self.config.is_mode_enabled(BackgroundMode::Location) {
            return Err("Location mode not enabled".to_string());
        }

        if self.config.location_accuracy == LocationAccuracy::Disabled {
            return Err("Location accuracy is disabled".to_string());
        }

        tracing::info!(
            "Location background enabled with {:?}",
            self.config.location_accuracy
        );
        Ok(())
    }

    /// Disable location-based background operation
    pub fn disable_location_background(&mut self) -> Result<(), String> {
        tracing::info!("Location background disabled");
        Ok(())
    }

    /// Check if BLE central can operate in background
    pub fn can_run_ble_central_background(&self) -> bool {
        self.config.is_mode_enabled(BackgroundMode::BluetoothCentral)
            && self.bluetooth_state.state != BluetoothState::Restricted
    }

    /// Check if BLE peripheral can operate in background
    pub fn can_run_ble_peripheral_background(&self) -> bool {
        self.config.is_mode_enabled(BackgroundMode::BluetoothPeripheral)
            && self.bluetooth_state.state != BluetoothState::Restricted
    }

    /// Get recommended background operation profile
    pub fn get_recommended_profile(&self) -> String {
        let mut active = Vec::new();

        if self.can_run_ble_central_background() {
            active.push("BLE-Central");
        }
        if self.can_run_ble_peripheral_background() {
            active.push("BLE-Peripheral");
        }
        if self.config.is_mode_enabled(BackgroundMode::Location) {
            active.push("Location");
        }
        if self.config.is_mode_enabled(BackgroundMode::BackgroundFetch) {
            active.push("Fetch");
        }
        if self.config.is_mode_enabled(BackgroundMode::BackgroundProcessing) {
            active.push("Processing");
        }

        if active.is_empty() {
            "None".to_string()
        } else {
            active.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_bluetooth_state_creation() {
        let state = CoreBluetoothState::new();
        assert!(!state.central_active);
        assert!(!state.peripheral_active);
        assert_eq!(state.state, BluetoothState::Inactive);
    }

    #[test]
    fn test_core_bluetooth_central_activation() {
        let mut state = CoreBluetoothState::new();
        state.set_central_active(true);
        assert!(state.central_active);
        assert!(state.is_any_active());
        assert_eq!(state.state, BluetoothState::Active);
    }

    #[test]
    fn test_core_bluetooth_both_active() {
        let mut state = CoreBluetoothState::new();
        state.set_central_active(true);
        state.set_peripheral_active(true);
        assert!(state.is_any_active());
        assert_eq!(state.state, BluetoothState::Active);
    }

    #[test]
    fn test_core_bluetooth_restriction() {
        let mut state = CoreBluetoothState::new();
        state.set_central_active(true);
        state.set_restricted(true);
        assert_eq!(state.state, BluetoothState::Restricted);
    }

    #[test]
    fn test_ios_background_config_default() {
        let config = IosBackgroundConfig::default();
        assert!(config.is_mode_enabled(BackgroundMode::BluetoothCentral));
        assert!(config.is_mode_enabled(BackgroundMode::BluetoothPeripheral));
        assert!(config.is_mode_enabled(BackgroundMode::BackgroundFetch));
        assert!(!config.is_mode_enabled(BackgroundMode::Location));
    }

    #[test]
    fn test_ios_background_config_validation() {
        let mut config = IosBackgroundConfig::default();
        config.fetch_interval_secs = 30;
        assert!(config.validate().is_err());

        config.fetch_interval_secs = 100000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ios_background_config_empty_modes() {
        let config = IosBackgroundConfig {
            enabled_modes: HashSet::new(),
            fetch_interval_secs: 900,
            location_accuracy: LocationAccuracy::Disabled,
            allow_always_location: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ios_background_config_enable_disable_modes() {
        let mut config = IosBackgroundConfig::default();
        config.disable_mode(BackgroundMode::Location);
        assert!(!config.is_mode_enabled(BackgroundMode::Location));

        config.enable_mode(BackgroundMode::Location);
        assert!(config.is_mode_enabled(BackgroundMode::Location));
    }

    #[test]
    fn test_ios_background_strategy_creation() {
        let config = IosBackgroundConfig::default();
        let strategy = IosBackgroundStrategy::new(config);
        assert!(strategy.is_ok());
    }

    #[test]
    fn test_ios_background_strategy_invalid_config() {
        let mut config = IosBackgroundConfig::default();
        config.enabled_modes.clear();
        let strategy = IosBackgroundStrategy::new(config);
        assert!(strategy.is_err());
    }

    #[test]
    fn test_schedule_background_fetch() {
        let config = IosBackgroundConfig::default();
        let strategy = IosBackgroundStrategy::new(config).unwrap();
        assert!(strategy.schedule_background_fetch().is_ok());
    }

    #[test]
    fn test_schedule_background_fetch_disabled() {
        let mut config = IosBackgroundConfig::default();
        config.disable_mode(BackgroundMode::BackgroundFetch);
        let strategy = IosBackgroundStrategy::new(config).unwrap();
        assert!(strategy.schedule_background_fetch().is_err());
    }

    #[test]
    fn test_on_background_fetch() {
        let config = IosBackgroundConfig::default();
        let mut strategy = IosBackgroundStrategy::new(config).unwrap();
        assert!(strategy.on_background_fetch().is_ok());
        assert!(strategy.get_bluetooth_state().central_active);
    }

    #[test]
    fn test_enable_location_background() {
        let mut config = IosBackgroundConfig::default();
        config.enable_mode(BackgroundMode::Location);
        let mut strategy = IosBackgroundStrategy::new(config).unwrap();
        assert!(strategy.enable_location_background().is_ok());
    }

    #[test]
    fn test_enable_location_background_disabled_mode() {
        let config = IosBackgroundConfig::default();
        let mut strategy = IosBackgroundStrategy::new(config).unwrap();
        assert!(strategy.enable_location_background().is_err());
    }

    #[test]
    fn test_enable_location_background_disabled_accuracy() {
        let mut config = IosBackgroundConfig::default();
        config.enable_mode(BackgroundMode::Location);
        config.location_accuracy = LocationAccuracy::Disabled;
        let mut strategy = IosBackgroundStrategy::new(config).unwrap();
        assert!(strategy.enable_location_background().is_err());
    }

    #[test]
    fn test_can_run_ble_central() {
        let config = IosBackgroundConfig::default();
        let strategy = IosBackgroundStrategy::new(config).unwrap();
        assert!(strategy.can_run_ble_central_background());
    }

    #[test]
    fn test_cannot_run_ble_central_when_restricted() {
        let config = IosBackgroundConfig::default();
        let mut strategy = IosBackgroundStrategy::new(config).unwrap();
        let mut bt_state = strategy.get_bluetooth_state();
        bt_state.set_restricted(true);
        strategy.set_bluetooth_state(bt_state);
        assert!(!strategy.can_run_ble_central_background());
    }

    #[test]
    fn test_get_recommended_profile() {
        let config = IosBackgroundConfig::default();
        let strategy = IosBackgroundStrategy::new(config).unwrap();
        let profile = strategy.get_recommended_profile();
        assert!(profile.contains("BLE-Central"));
        assert!(profile.contains("BLE-Peripheral"));
    }

    #[test]
    fn test_get_enabled_modes() {
        let config = IosBackgroundConfig::default();
        let strategy = IosBackgroundStrategy::new(config).unwrap();
        let modes = strategy.get_config().get_enabled_modes();
        assert_eq!(modes.len(), 3);
    }

    #[test]
    fn test_set_config_valid() {
        let config1 = IosBackgroundConfig::default();
        let mut strategy = IosBackgroundStrategy::new(config1).unwrap();

        let mut config2 = IosBackgroundConfig::default();
        config2.fetch_interval_secs = 1800;
        assert!(strategy.set_config(config2).is_ok());
        assert_eq!(strategy.get_config().fetch_interval_secs, 1800);
    }

    #[test]
    fn test_set_config_invalid() {
        let config1 = IosBackgroundConfig::default();
        let mut strategy = IosBackgroundStrategy::new(config1).unwrap();

        let mut config2 = IosBackgroundConfig::default();
        config2.fetch_interval_secs = 30;
        assert!(strategy.set_config(config2).is_err());
    }

    #[test]
    fn test_bluetooth_state_deactivation() {
        let mut state = CoreBluetoothState::new();
        state.set_central_active(true);
        state.set_peripheral_active(true);
        assert!(state.is_any_active());

        state.set_central_active(false);
        state.set_peripheral_active(false);
        assert!(!state.is_any_active());
    }

    #[test]
    fn test_location_accuracy_levels() {
        let mut config = IosBackgroundConfig::default();
        config.enable_mode(BackgroundMode::Location);

        config.location_accuracy = LocationAccuracy::FullAccuracy;
        assert!(config.validate().is_ok());

        config.location_accuracy = LocationAccuracy::ReducedAccuracy;
        assert!(config.validate().is_ok());

        config.location_accuracy = LocationAccuracy::Disabled;
        assert!(config.validate().is_ok());
    }
}
