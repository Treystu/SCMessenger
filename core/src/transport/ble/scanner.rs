/// BLE Scanner implementation with duty cycle management
///
/// This module provides BLE scanning with adaptive duty cycles based on battery state.
/// Scanning can be aggressive when charging, standard when battery is good, and minimal
/// when battery is low to conserve power.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// BLE scanning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleScanConfig {
    /// Scan interval in milliseconds (default 200ms)
    pub scan_interval_ms: u64,
    /// Scan window in milliseconds (default 100ms)
    pub scan_window_ms: u64,
    /// Current duty cycle percentage (0-100)
    pub duty_cycle_percent: u8,
}

impl Default for BleScanConfig {
    fn default() -> Self {
        Self {
            scan_interval_ms: 200,
            scan_window_ms: 100,
            duty_cycle_percent: 50,
        }
    }
}

impl BleScanConfig {
    /// Create a new scan configuration
    pub fn new(scan_interval_ms: u64, scan_window_ms: u64) -> Result<Self, ScannerError> {
        let config = Self {
            scan_interval_ms,
            scan_window_ms,
            duty_cycle_percent: 50,
        };
        config.validate()?;
        Ok(config)
    }

    /// Set the duty cycle percentage
    pub fn with_duty_cycle(mut self, percent: u8) -> Result<Self, ScannerError> {
        if percent > 100 {
            return Err(ScannerError::InvalidDutyCycle);
        }
        self.duty_cycle_percent = percent;
        Ok(self)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ScannerError> {
        if self.scan_window_ms > self.scan_interval_ms {
            return Err(ScannerError::InvalidScanConfig(
                "Scan window must be <= scan interval".to_string(),
            ));
        }
        if self.scan_interval_ms == 0 || self.scan_window_ms == 0 {
            return Err(ScannerError::InvalidScanConfig(
                "Scan intervals must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Scanner state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScannerState {
    /// Idle, not scanning
    Idle,
    /// Actively scanning
    Scanning,
    /// Paused (duty cycle)
    Paused,
}

/// Battery state for duty cycle management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryState {
    /// Device is charging
    Charging,
    /// Battery above 50%
    Good,
    /// Battery between 20-50%
    Low,
    /// Battery below 20%
    Critical,
}

impl BatteryState {
    /// Create battery state from percentage (0-100)
    pub fn from_percentage(percent: u8) -> Self {
        match percent {
            0..=20 => BatteryState::Critical,
            21..=50 => BatteryState::Low,
            _ => BatteryState::Good,
        }
    }
}

/// Duty cycle manager for battery-aware scanning
pub struct DutyCycleManager {
    battery_state: BatteryState,
    wifi_available: bool,
}

impl DutyCycleManager {
    /// Create a new duty cycle manager
    pub fn new(battery_state: BatteryState, wifi_available: bool) -> Self {
        Self {
            battery_state,
            wifi_available,
        }
    }

    /// Update battery state
    pub fn set_battery_state(&mut self, state: BatteryState) {
        self.battery_state = state;
    }

    /// Update WiFi availability
    pub fn set_wifi_available(&mut self, available: bool) {
        self.wifi_available = available;
    }

    /// Get recommended duty cycle based on device state
    pub fn get_recommended_duty_cycle(&self) -> u8 {
        // Aggressive: 90% duty cycle (charging + WiFi available)
        if self.battery_state == BatteryState::Charging && self.wifi_available {
            return 90;
        }

        // Aggressive: 90% when charging
        if self.battery_state == BatteryState::Charging {
            return 90;
        }

        // Standard: 50% when battery is good
        if self.battery_state == BatteryState::Good {
            return 50;
        }

        // Reduced: 20% when battery is low
        if self.battery_state == BatteryState::Low {
            return 20;
        }

        // Minimal: 5% when battery is critical
        5
    }

    /// Get duty cycle description
    pub fn get_mode_description(&self) -> &'static str {
        match self.get_recommended_duty_cycle() {
            90 => "Aggressive",
            50 => "Standard",
            20 => "Reduced",
            5 => "Minimal",
            _ => "Unknown",
        }
    }
}

/// Errors for scanner operations
#[derive(Error, Debug, Clone)]
pub enum ScannerError {
    #[error("Invalid scan configuration: {0}")]
    InvalidScanConfig(String),
    #[error("Invalid duty cycle")]
    InvalidDutyCycle,
    #[error("Scanner not idle")]
    NotIdle,
    #[error("Scanner not scanning")]
    NotScanning,
    #[error("System time error: {0}")]
    SystemTimeError(String),
}

/// Scan result from a discovered peer
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Peer ID (blake3 hash of public key)
    pub peer_id: Vec<u8>,
    /// RSSI (Received Signal Strength Indicator) in dBm
    pub rssi: i8,
    /// Beacon data (encrypted)
    pub beacon_data: Vec<u8>,
    /// Transport info (custom per-implementation)
    pub transport_info: Vec<u8>,
    /// Timestamp when scan result was received
    pub timestamp: u64,
}

impl ScanResult {
    /// Create a new scan result
    pub fn new(
        peer_id: Vec<u8>,
        rssi: i8,
        beacon_data: Vec<u8>,
        transport_info: Vec<u8>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        Self {
            peer_id,
            rssi,
            beacon_data,
            transport_info,
            timestamp: now.as_secs(),
        }
    }

    /// Get approximate distance based on RSSI (simple estimate)
    pub fn estimate_distance(&self) -> f64 {
        // Simple path loss model: distance ≈ 10^((txPower - rssi) / (20))
        // Assume txPower = -5 dBm (typical BLE value)
        let tx_power = -5i16;
        let rssi_i16 = self.rssi as i16;

        if rssi_i16 == tx_power {
            return 1.0;
        }

        let distance = 10f64.powf((tx_power - rssi_i16 as i16) as f64 / 20.0);
        distance
    }
}

/// BLE Scanner with duty cycle management
pub struct BleScanner {
    state: ScannerState,
    config: BleScanConfig,
    duty_cycle_manager: DutyCycleManager,
    last_state_change: u64,
}

impl BleScanner {
    /// Create a new BLE scanner
    pub fn new(config: BleScanConfig, battery_state: BatteryState) -> Result<Self, ScannerError> {
        config.validate()?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ScannerError::SystemTimeError(e.to_string()))?;

        Ok(Self {
            state: ScannerState::Idle,
            config,
            duty_cycle_manager: DutyCycleManager::new(battery_state, false),
            last_state_change: now.as_millis() as u64,
        })
    }

    /// Get current scanner state
    pub fn state(&self) -> ScannerState {
        self.state
    }

    /// Get scanner configuration
    pub fn config(&self) -> &BleScanConfig {
        &self.config
    }

    /// Check if scanner is active (scanning or paused)
    pub fn is_active(&self) -> bool {
        self.state != ScannerState::Idle
    }

    /// Start scanning
    pub fn start_scanning(&mut self) -> Result<(), ScannerError> {
        match self.state {
            ScannerState::Idle => {
                self.state = ScannerState::Scanning;
                self.update_timestamp();
                Ok(())
            }
            ScannerState::Paused => {
                self.state = ScannerState::Scanning;
                self.update_timestamp();
                Ok(())
            }
            ScannerState::Scanning => Err(ScannerError::NotIdle),
        }
    }

    /// Pause scanning (for duty cycle)
    pub fn pause_scanning(&mut self) -> Result<(), ScannerError> {
        match self.state {
            ScannerState::Scanning => {
                self.state = ScannerState::Paused;
                self.update_timestamp();
                Ok(())
            }
            _ => Err(ScannerError::NotScanning),
        }
    }

    /// Stop scanning
    pub fn stop_scanning(&mut self) -> Result<(), ScannerError> {
        match self.state {
            ScannerState::Scanning | ScannerState::Paused => {
                self.state = ScannerState::Idle;
                self.update_timestamp();
                Ok(())
            }
            ScannerState::Idle => Ok(()), // Already idle
        }
    }

    /// Update battery state
    pub fn set_battery_state(&mut self, state: BatteryState) {
        self.duty_cycle_manager.set_battery_state(state);
    }

    /// Update WiFi availability
    pub fn set_wifi_available(&mut self, available: bool) {
        self.duty_cycle_manager.set_wifi_available(available);
    }

    /// Get current duty cycle percentage
    pub fn get_duty_cycle(&self) -> u8 {
        self.duty_cycle_manager.get_recommended_duty_cycle()
    }

    /// Get duty cycle mode description
    pub fn get_mode(&self) -> &'static str {
        self.duty_cycle_manager.get_mode_description()
    }

    /// Calculate scan duration for the current duty cycle
    pub fn calculate_scan_duration_ms(&self) -> u64 {
        let cycle = self.get_duty_cycle();
        (self.config.scan_interval_ms * cycle as u64) / 100
    }

    /// Calculate pause duration for the current duty cycle
    pub fn calculate_pause_duration_ms(&self) -> u64 {
        let cycle = self.get_duty_cycle();
        let pause_percent = 100u64.saturating_sub(cycle as u64);
        (self.config.scan_interval_ms * pause_percent) / 100
    }

    /// Update the last state change timestamp
    fn update_timestamp(&mut self) {
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            self.last_state_change = duration.as_millis() as u64;
        }
    }

    /// Get time since last state change in milliseconds
    pub fn time_since_state_change_ms(&self) -> u64 {
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let now = duration.as_millis() as u64;
            now.saturating_sub(self.last_state_change)
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_scan_config_default() {
        let config = BleScanConfig::default();
        assert_eq!(config.scan_interval_ms, 200);
        assert_eq!(config.scan_window_ms, 100);
        assert_eq!(config.duty_cycle_percent, 50);
    }

    #[test]
    fn test_ble_scan_config_validation_valid() {
        let config = BleScanConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ble_scan_config_validation_window_exceeds_interval() {
        let config = BleScanConfig {
            scan_interval_ms: 100,
            scan_window_ms: 200,
            duty_cycle_percent: 50,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ble_scan_config_validation_zero_interval() {
        let config = BleScanConfig {
            scan_interval_ms: 0,
            scan_window_ms: 50,
            duty_cycle_percent: 50,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ble_scan_config_with_duty_cycle() {
        let config = BleScanConfig::default()
            .with_duty_cycle(75)
            .expect("Valid duty cycle");
        assert_eq!(config.duty_cycle_percent, 75);
    }

    #[test]
    fn test_ble_scan_config_invalid_duty_cycle() {
        let result = BleScanConfig::default().with_duty_cycle(101);
        assert!(result.is_err());
    }

    #[test]
    fn test_battery_state_from_percentage() {
        assert_eq!(BatteryState::from_percentage(10), BatteryState::Critical);
        assert_eq!(BatteryState::from_percentage(20), BatteryState::Critical);
        assert_eq!(BatteryState::from_percentage(35), BatteryState::Low);
        assert_eq!(BatteryState::from_percentage(50), BatteryState::Low);
        assert_eq!(BatteryState::from_percentage(75), BatteryState::Good);
        assert_eq!(BatteryState::from_percentage(100), BatteryState::Good);
    }

    #[test]
    fn test_duty_cycle_manager_aggressive_when_charging() {
        let manager = DutyCycleManager::new(BatteryState::Charging, false);
        assert_eq!(manager.get_recommended_duty_cycle(), 90);
        assert_eq!(manager.get_mode_description(), "Aggressive");
    }

    #[test]
    fn test_duty_cycle_manager_aggressive_with_charging_and_wifi() {
        let manager = DutyCycleManager::new(BatteryState::Charging, true);
        assert_eq!(manager.get_recommended_duty_cycle(), 90);
        assert_eq!(manager.get_mode_description(), "Aggressive");
    }

    #[test]
    fn test_duty_cycle_manager_standard_good_battery() {
        let manager = DutyCycleManager::new(BatteryState::Good, false);
        assert_eq!(manager.get_recommended_duty_cycle(), 50);
        assert_eq!(manager.get_mode_description(), "Standard");
    }

    #[test]
    fn test_duty_cycle_manager_reduced_low_battery() {
        let manager = DutyCycleManager::new(BatteryState::Low, false);
        assert_eq!(manager.get_recommended_duty_cycle(), 20);
        assert_eq!(manager.get_mode_description(), "Reduced");
    }

    #[test]
    fn test_duty_cycle_manager_minimal_critical_battery() {
        let manager = DutyCycleManager::new(BatteryState::Critical, false);
        assert_eq!(manager.get_recommended_duty_cycle(), 5);
        assert_eq!(manager.get_mode_description(), "Minimal");
    }

    #[test]
    fn test_duty_cycle_manager_set_battery_state() {
        let mut manager = DutyCycleManager::new(BatteryState::Good, false);
        assert_eq!(manager.get_recommended_duty_cycle(), 50);

        manager.set_battery_state(BatteryState::Critical);
        assert_eq!(manager.get_recommended_duty_cycle(), 5);
    }

    #[test]
    fn test_scan_result_creation() {
        let result = ScanResult::new(
            vec![0x01, 0x02, 0x03],
            -50,
            vec![0xAA; 30],
            vec![],
        );

        assert_eq!(result.peer_id, vec![0x01, 0x02, 0x03]);
        assert_eq!(result.rssi, -50);
        assert_eq!(result.beacon_data.len(), 30);
    }

    #[test]
    fn test_scan_result_distance_estimate() {
        let result = ScanResult::new(vec![], -10, vec![], vec![]);
        let distance = result.estimate_distance();

        assert!(distance > 0.0);
        assert!(distance < 100.0); // Reasonable range
    }

    #[test]
    fn test_ble_scanner_creation() {
        let config = BleScanConfig::default();
        let scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        assert_eq!(scanner.state(), ScannerState::Idle);
        assert!(!scanner.is_active());
    }

    #[test]
    fn test_ble_scanner_start_stop() {
        let config = BleScanConfig::default();
        let mut scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        scanner.start_scanning().expect("Start scanning");
        assert_eq!(scanner.state(), ScannerState::Scanning);
        assert!(scanner.is_active());

        scanner.stop_scanning().expect("Stop scanning");
        assert_eq!(scanner.state(), ScannerState::Idle);
        assert!(!scanner.is_active());
    }

    #[test]
    fn test_ble_scanner_pause_resume() {
        let config = BleScanConfig::default();
        let mut scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        scanner.start_scanning().expect("Start");
        scanner.pause_scanning().expect("Pause");
        assert_eq!(scanner.state(), ScannerState::Paused);

        scanner.start_scanning().expect("Resume");
        assert_eq!(scanner.state(), ScannerState::Scanning);
    }

    #[test]
    fn test_ble_scanner_double_start_error() {
        let config = BleScanConfig::default();
        let mut scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        scanner.start_scanning().expect("First start");
        let result = scanner.start_scanning();
        assert!(result.is_err());
    }

    #[test]
    fn test_ble_scanner_set_battery_state() {
        let config = BleScanConfig::default();
        let mut scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        assert_eq!(scanner.get_duty_cycle(), 50);

        scanner.set_battery_state(BatteryState::Critical);
        assert_eq!(scanner.get_duty_cycle(), 5);
    }

    #[test]
    fn test_ble_scanner_set_wifi_available() {
        let config = BleScanConfig::default();
        let mut scanner =
            BleScanner::new(config, BatteryState::Charging).expect("Scanner creation");

        assert_eq!(scanner.get_duty_cycle(), 90);

        scanner.set_wifi_available(true);
        assert_eq!(scanner.get_duty_cycle(), 90); // Still aggressive
    }

    #[test]
    fn test_ble_scanner_calculate_scan_duration() {
        let config = BleScanConfig::default();
        let scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        // 50% duty cycle with 200ms interval = 100ms
        let duration = scanner.calculate_scan_duration_ms();
        assert_eq!(duration, 100);
    }

    #[test]
    fn test_ble_scanner_calculate_pause_duration() {
        let config = BleScanConfig::default();
        let scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        // 50% duty cycle with 200ms interval = 100ms pause
        let pause = scanner.calculate_pause_duration_ms();
        assert_eq!(pause, 100);
    }

    #[test]
    fn test_ble_scanner_mode_description() {
        let config = BleScanConfig::default();
        let scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        assert_eq!(scanner.get_mode(), "Standard");
    }

    #[test]
    fn test_ble_scanner_time_since_state_change() {
        let config = BleScanConfig::default();
        let scanner = BleScanner::new(config, BatteryState::Good).expect("Scanner creation");

        let time = scanner.time_since_state_change_ms();
        assert!(time < 1000); // Should be fresh
    }

    #[test]
    fn test_ble_scanner_invalid_config() {
        let config = BleScanConfig {
            scan_interval_ms: 100,
            scan_window_ms: 200,
            duty_cycle_percent: 50,
        };

        let result = BleScanner::new(config, BatteryState::Good);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_battery_states() {
        let states = vec![
            BatteryState::Charging,
            BatteryState::Good,
            BatteryState::Low,
            BatteryState::Critical,
        ];

        for state in states {
            let manager = DutyCycleManager::new(state, false);
            let cycle = manager.get_recommended_duty_cycle();
            assert!(cycle > 0 && cycle <= 100);
        }
    }
}
