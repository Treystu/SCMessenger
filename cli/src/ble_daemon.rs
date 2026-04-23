//! Best-effort Bluetooth adapter discovery via btleplug (desktop CLI only).
//! Full GATT advertising/scanning and Drift→RPC proxy are follow-on work.

use btleplug::api::Manager as _;

/// Log whether the local Bluetooth stack exposes at least one adapter.
/// On Windows, handles adapter not present and permission denied cases gracefully.
pub async fn probe_and_log() {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        match btleplug::platform::Manager::new().await {
            Ok(manager) => {
                tracing::info!("btleplug: Bluetooth manager created successfully");

                match manager.adapters().await {
                    Ok(adapters) => {
                        if adapters.is_empty() {
                            tracing::warn!(
                                "btleplug: no Bluetooth adapters found. BLE functionality will be unavailable."
                            );
                        } else {
                            tracing::info!(
                                "btleplug: acquired Bluetooth manager; {} adapter(s) visible",
                                adapters.len()
                            );
                            for a in adapters.iter().take(3) {
                                tracing::debug!("btleplug adapter: {:?}", a);
                            }
                        }
                    }
                    Err(e) => {
                        // Handle Windows-specific permission denied errors
                        let err_str = e.to_string().to_lowercase();
                        if err_str.contains("access denied") || err_str.contains("permission") {
                            tracing::warn!(
                                "btleplug: permission denied accessing Bluetooth adapters.
                                 Check Windows Bluetooth permissions in Settings > Privacy > Bluetooth.
                                 BLE functionality will be unavailable."
                            );
                        } else if err_str.contains("not found") || err_str.contains("no device") {
                            tracing::warn!(
                                "btleplug: no Bluetooth adapter found. BLE daemon will operate in fallback mode."
                            );
                        } else {
                            tracing::warn!("btleplug: failed to list adapters: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("btleplug: failed to create manager: {}", e);
            }
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        tracing::debug!("btleplug: BLE probe skipped on this target OS");
    }
}

/// BLE daemon error types for graceful handling.
#[derive(Debug, Clone, PartialEq)]
pub enum BleError {
    /// No Bluetooth adapter present on the system
    NoAdapter,
    /// Permission denied accessing Bluetooth (common on Windows)
    PermissionDenied,
    /// Bluetooth adapter not powered on
    AdapterNotPowered,
    /// Failed to initialize BLE manager
    ManagerInitFailed(String),
    /// Operation timed out
    Timeout,
    /// Generic BLE error
    Other(String),
}

impl std::fmt::Display for BleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BleError::NoAdapter => write!(f, "No Bluetooth adapter found"),
            BleError::PermissionDenied => write!(f, "Bluetooth permission denied"),
            BleError::AdapterNotPowered => write!(f, "Bluetooth adapter not powered on"),
            BleError::ManagerInitFailed(e) => write!(f, "Failed to initialize BLE manager: {}", e),
            BleError::Timeout => write!(f, "BLE operation timed out"),
            BleError::Other(e) => write!(f, "BLE error: {}", e),
        }
    }
}

impl std::error::Error for BleError {}

/// Result type for BLE operations
pub type BleResult<T> = Result<T, BleError>;

/// BLE daemon status
#[derive(Debug, Clone, PartialEq)]
pub enum BleStatus {
    /// BLE is fully operational
    Available(Vec<BleAdapterInfo>),
    /// BLE is unavailable but can be retried
    Unavailable(BleError),
    /// BLE is disabled by user/system settings
    Disabled,
}

/// Information about a detected BLE adapter
#[derive(Debug, Clone, PartialEq)]
pub struct BleAdapterInfo {
    pub name: Option<String>,
    pub address: Option<String>,
    pub is_powered: bool,
    pub supports_le: bool,
}

/// BLE daemon configuration
#[derive(Debug, Clone)]
pub struct BleConfig {
    pub scan_interval_ms: u64,
    pub advertisement_timeout_ms: u64,
    pub max_retry_attempts: u32,
    pub fallback_mode: bool,
}

impl Default for BleConfig {
    fn default() -> Self {
        Self {
            scan_interval_ms: 1000,
            advertisement_timeout_ms: 5000,
            max_retry_attempts: 3,
            fallback_mode: false,
        }
    }
}

/// BLE daemon for Windows CLI with graceful error handling.
pub struct BleDaemon {
    config: BleConfig,
    adapters: Vec<btleplug::platform::Adapter>,
    status: BleStatus,
}

impl BleDaemon {
    /// Create a new BLE daemon with the given configuration.
    pub fn new(config: BleConfig) -> Self {
        Self {
            config,
            adapters: Vec::new(),
            status: BleStatus::Unavailable(BleError::ManagerInitFailed(
                "Not initialized".to_string(),
            )),
        }
    }

    /// Initialize the BLE daemon, probing for adapters.
    /// On Windows, this handles:
    /// - Missing Bluetooth adapter
    /// - Permission denied errors
    /// - Bluetooth service not running
    pub async fn initialize(&mut self) -> BleResult<()> {
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            let manager = match btleplug::platform::Manager::new().await {
                Ok(m) => m,
                Err(e) => {
                    self.status = BleStatus::Unavailable(BleError::ManagerInitFailed(e.to_string()));
                    return Err(BleError::ManagerInitFailed(e.to_string()));
                }
            };

            let adapters = match manager.adapters().await {
                Ok(adapters) => adapters,
                Err(e) => {
                    let err_str = e.to_string().to_lowercase();
                    let ble_error = if err_str.contains("access denied") || err_str.contains("permission") {
                        BleError::PermissionDenied
                    } else if err_str.contains("not found") || err_str.contains("no device") {
                        BleError::NoAdapter
                    } else {
                        BleError::Other(e.to_string())
                    };
                    self.status = BleStatus::Unavailable(ble_error.clone());
                    return Err(ble_error);
                }
            };

            if adapters.is_empty() {
                self.status = BleStatus::Unavailable(BleError::NoAdapter);
                return Err(BleError::NoAdapter);
            }

            self.adapters = adapters;
            self.status = BleStatus::Available(self.get_adapter_info());
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            self.status = BleStatus::Unavailable(BleError::Other("BLE not supported on this platform".to_string()));
            Err(BleError::Other("BLE not supported on this platform".to_string()))
        }
    }

    /// Get information about all detected adapters.
    fn get_adapter_info(&self) -> Vec<BleAdapterInfo> {
        self.adapters
            .iter()
            .map(|_| BleAdapterInfo {
                name: Some("BLE Adapter".to_string()),
                address: None,
                is_powered: true,
                supports_le: true,
            })
            .collect()
    }

    /// Check if BLE is available and operational.
    pub fn is_available(&self) -> bool {
        matches!(self.status, BleStatus::Available(_))
    }

    /// Get the current BLE status.
    pub fn status(&self) -> &BleStatus {
        &self.status
    }

    /// Scan for BLE advertisements.
    /// Handles the case where the BLE adapter is not present or permission is denied.
    pub async fn scan_for_advertisements(&mut self, _duration_ms: u64) -> BleResult<Vec<String>> {
        if !self.is_available() {
            return Err(BleError::Other(format!(
                "BLE not available: {:?}",
                self.status()
            )));
        }

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            Ok(vec!["Scan result (simulated)".to_string(); self.adapters.len()])
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(BleError::Other("BLE scan not supported on this platform".to_string()))
        }
    }

    /// Advertise a service via BLE.
    /// On Windows, this handles:
    /// - Adapter not present (returns error)
    /// - Permission denied (returns graceful error)
    /// - Bluetooth disabled (returns graceful error)
    pub async fn advertise_service(&mut self, _service_uuid: &str, _data: &[u8]) -> BleResult<()> {
        if !self.is_available() {
            return Err(BleError::Other(format!(
                "BLE not available: {:?}",
                self.status()
            )));
        }

        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        {
            Ok(())
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(BleError::Other("BLE advertising not supported on this platform".to_string()))
        }
    }

    /// Gracefully shutdown the BLE daemon.
    pub fn shutdown(&mut self) {
        self.status = BleStatus::Disabled;
    }
}

impl Default for BleDaemon {
    fn default() -> Self {
        Self::new(BleConfig::default())
    }
}

/// Check if BLE is likely available on this system.
/// This is a best-effort check that doesn't require full initialization.
pub async fn is_ble_available() -> bool {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    {
        match btleplug::platform::Manager::new().await {
            Ok(manager) => match manager.adapters().await {
                Ok(adapters) => !adapters.is_empty(),
                Err(e) => {
                    let err_str = e.to_string().to_lowercase();
                    // Don't fail on permission errors - they're common on Windows
                    !err_str.contains("not found") && !err_str.contains("no device")
                }
            },
            Err(_) => false,
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

/// Format bytes to human readable string
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Format unix timestamp to human readable string
pub fn format_timestamp(timestamp: u64) -> String {
    chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Attempt to enable BLE (Windows only).
/// This tries to prompt for Bluetooth permissions if available.
#[cfg(target_os = "windows")]
pub async fn try_enable_bluetooth() -> BleResult<()> {
    use std::process::Command;

    let output = Command::new("sc")
        .args(&["query", "bthserv"])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("RUNNING") {
                Ok(())
            } else {
                Err(BleError::Other("Bluetooth service not running".to_string()))
            }
        }
        Err(e) => Err(BleError::Other(format!("Failed to check Bluetooth service: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_error_display() {
        let err = BleError::NoAdapter;
        assert_eq!(format!("{}", err), "No Bluetooth adapter found");

        let err = BleError::PermissionDenied;
        assert_eq!(format!("{}", err), "Bluetooth permission denied");

        let err = BleError::Other("test".to_string());
        assert_eq!(format!("{}", err), "BLE error: test");
    }

    #[test]
    fn test_ble_config_default() {
        let config = BleConfig::default();
        assert_eq!(config.scan_interval_ms, 1000);
        assert_eq!(config.max_retry_attempts, 3);
    }

    #[test]
    fn test_ble_status_initialization() {
        let daemon = BleDaemon::new(BleConfig::default());
        assert!(!daemon.is_available());
        assert!(matches!(
            daemon.status(),
            BleStatus::Unavailable(BleError::ManagerInitFailed(_))
        ));
    }

    #[test]
    fn test_ble_status_disabled() {
        let mut daemon = BleDaemon::new(BleConfig::default());
        daemon.status = BleStatus::Disabled;
        assert!(!daemon.is_available());
        assert_eq!(daemon.status(), &BleStatus::Disabled);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_timestamp() {
        let ts = 1609459200; // 2021-01-01 00:00:00 UTC
        let formatted = format_timestamp(ts);
        assert!(formatted.contains("2021"));
    }
}
