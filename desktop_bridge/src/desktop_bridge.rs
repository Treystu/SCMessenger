// DesktopBridge — main UniFFI object for desktop integration.
//
// This is the primary entry point for the KMP Compose Multiplatform desktop
// client. It wraps scmessenger-core::IronCore and adds desktop-specific
// capabilities: XDG paths, notifications, tray status, BLE, power management.
//
// All methods are gated with `#[cfg(target_os = "linux")]` where appropriate.
// Non-Linux targets get stub implementations that return "unsupported" errors.

use std::sync::Arc;

/// Desktop bridge integrating IronCore with Linux desktop capabilities.
///
/// Wraps an IronCore instance and provides desktop-specific FFI functions
/// for system tray, notifications, BLE, power management, and XDG paths.
#[derive(uniffi::Object)]
pub struct DesktopBridge {
    /// Core engine instance (shared with mobile bridge).
    core: Arc<std::sync::Mutex<Option<std::sync::Arc<scmessenger_core::IronCore>>>>,
    /// XDG paths (computed once at construction).
    xdg_paths: crate::XdgPaths,
    /// Current tray status.
    tray_status: std::sync::Mutex<crate::TrayStatus>,
    /// Current power state.
    power_state: std::sync::Mutex<crate::PowerState>,
    /// Desktop delegate for callbacks.
    delegate: std::sync::Mutex<Option<Box<dyn crate::DesktopDelegate>>>,
}

#[uniffi::export]
impl DesktopBridge {
    /// Create a new DesktopBridge with default XDG paths.
    #[uniffi::constructor]
    pub fn new() -> Self {
        let xdg_paths = crate::xdg_paths::resolve_xdg_paths();
        let _ = crate::xdg_paths::ensure_directories(&xdg_paths);

        Self {
            core: Arc::new(std::sync::Mutex::new(None)),
            xdg_paths,
            tray_status: std::sync::Mutex::new(crate::tray::tray_status_for_state(
                crate::TrayIconState::Disconnected,
                0,
                0,
            )),
            power_state: std::sync::Mutex::new(crate::power::detect_power_state()),
            delegate: std::sync::Mutex::new(None),
        }
    }

    /// Create a new DesktopBridge with a custom storage path.
    #[uniffi::constructor]
    pub fn with_storage_path(storage_path: String) -> Self {
        let mut bridge = Self::new();
        bridge.xdg_paths.store_path = storage_path;
        bridge
    }

    // =======================================================================
    // XDG PATHS
    // =======================================================================

    /// Get the XDG paths for this desktop session.
    pub fn get_xdg_paths(&self) -> crate::XdgPaths {
        self.xdg_paths.clone()
    }

    /// Get the default sled store path (XDG data dir).
    pub fn get_store_path(&self) -> String {
        self.xdg_paths.store_path.clone()
    }

    // =======================================================================
    // CORE LIFECYCLE (delegates to IronCore)
    // =======================================================================

    /// Start the mesh engine.
    pub fn start(&self) -> Result<(), scmessenger_core::IronCoreError> {
        let core = scmessenger_core::IronCore::with_storage(self.xdg_paths.store_path.clone());
        core.start()?;
        *self.core.lock().expect("core lock poisoned") = Some(std::sync::Arc::new(core));
        Ok(())
    }

    /// Stop the mesh engine.
    pub fn stop(&self) {
        if let Some(ref core) = *self.core.lock().expect("core lock poisoned") {
            core.stop();
        }
        *self.core.lock().expect("core lock poisoned") = None;
    }

    /// Check if the mesh engine is running.
    pub fn is_running(&self) -> bool {
        self.core.lock().expect("core lock poisoned").is_some()
    }

    /// Get identity info from the core.
    pub fn get_identity_info(&self) -> scmessenger_core::IdentityInfo {
        if let Some(ref core) = *self.core.lock().expect("core lock poisoned") {
            core.get_identity_info()
        } else {
            scmessenger_core::IdentityInfo::default()
        }
    }

    /// Initialize identity (requires consent).
    pub fn initialize_identity(&self) -> Result<(), scmessenger_core::IronCoreError> {
        if let Some(ref core) = *self.core.lock().expect("core lock poisoned") {
            core.initialize_identity()
        } else {
            Err(scmessenger_core::IronCoreError::NotInitialized)
        }
    }

    /// Grant consent for identity operations.
    pub fn grant_consent(&self) {
        if let Some(ref core) = *self.core.lock().expect("core lock poisoned") {
            core.grant_consent();
        }
    }

    // =======================================================================
    // DESKTOP NOTIFICATIONS
    // =======================================================================

    /// Send a desktop notification.
    pub fn send_notification(
        &self,
        title: String,
        body: String,
        urgency: crate::NotificationUrgency,
    ) -> crate::NotificationResult {
        crate::notification::send_notification(title, body, urgency)
    }

    // =======================================================================
    // SYSTEM TRAY
    // =======================================================================

    /// Get the current tray status.
    pub fn get_tray_status(&self) -> crate::TrayStatus {
        self.tray_status
            .lock()
            .expect("tray_status lock poisoned")
            .clone()
    }

    /// Update the tray status and notify the delegate.
    pub fn update_tray_status(
        &self,
        state: crate::TrayIconState,
        unread_count: u32,
        connected_peers: u32,
    ) {
        let status = crate::tray::tray_status_for_state(state, unread_count, connected_peers);
        *self.tray_status.lock().expect("tray_status lock poisoned") = status.clone();

        if let Some(ref delegate) = *self.delegate.lock().expect("delegate lock poisoned") {
            delegate.on_tray_state_changed(status);
        }
    }

    // =======================================================================
    // POWER MANAGEMENT
    // =======================================================================

    /// Get the current power state.
    pub fn get_power_state(&self) -> crate::PowerState {
        self.power_state
            .lock()
            .expect("power_state lock poisoned")
            .clone()
    }

    /// Refresh power state from system.
    pub fn refresh_power_state(&self) {
        let state = crate::power::detect_power_state();
        *self.power_state.lock().expect("power_state lock poisoned") = state.clone();

        if let Some(ref delegate) = *self.delegate.lock().expect("delegate lock poisoned") {
            delegate.on_power_state_changed(state);
        }
    }

    /// Handle a power event (suspend/resume).
    pub fn handle_power_event(&self, event: crate::PowerProfile) {
        let state = crate::power::power_state_for_event(event);
        *self.power_state.lock().expect("power_state lock poisoned") = state.clone();

        if let Some(ref delegate) = *self.delegate.lock().expect("delegate lock poisoned") {
            delegate.on_power_state_changed(state);
        }
    }

    // =======================================================================
    // SOCKET ACTIVATION
    // =======================================================================

    /// Check socket activation status.
    pub fn check_socket_activation(&self) -> crate::SocketActivationStatus {
        crate::socket_activation::check_socket_activation()
    }

    // =======================================================================
    // DELEGATE
    // =======================================================================

    /// Set the desktop delegate for callbacks.
    pub fn set_delegate(&self, delegate: Option<Box<dyn crate::DesktopDelegate>>) {
        *self.delegate.lock().expect("delegate lock poisoned") = delegate;
    }
}

// =======================================================================
// BLE (Linux only)
// =======================================================================
//
// Each `#[cfg]` variant gets its own `#[uniffi::export]` impl block: `#[cfg]`
// does not reliably suppress UniFFI scaffolding generation inside a single
// `uniffi::export` block (the attribute macro runs before `cfg` stripping),
// so two cfg-gated methods with the same name in one block produce duplicate
// scaffolding symbols. See:
// https://mozilla.github.io/uniffi-rs/latest/proc_macro/index.html#conditional-compilation

#[cfg(target_os = "linux")]
#[uniffi::export]
impl DesktopBridge {
    /// Get BLE adapter info (Linux only).
    pub fn get_ble_adapter_info(&self) -> Result<crate::BleAdapterInfo, crate::DesktopBridgeError> {
        crate::ble::get_adapter_info_sync().map_err(crate::DesktopBridgeError::General)
    }

    /// List discovered BLE peers (Linux only).
    pub fn list_ble_peers(&self) -> Result<Vec<crate::BlePeer>, crate::DesktopBridgeError> {
        crate::ble::list_discovered_peers_sync().map_err(crate::DesktopBridgeError::General)
    }

    /// Start BLE scan (Linux only).
    pub fn start_ble_scan(&self) -> Result<(), crate::DesktopBridgeError> {
        crate::ble::start_scan_sync().map_err(crate::DesktopBridgeError::General)
    }

    /// Stop BLE scan (Linux only).
    pub fn stop_ble_scan(&self) -> Result<(), crate::DesktopBridgeError> {
        crate::ble::stop_scan_sync().map_err(crate::DesktopBridgeError::General)
    }
}

#[cfg(not(target_os = "linux"))]
#[uniffi::export]
impl DesktopBridge {
    /// Get BLE adapter info (non-Linux stub).
    pub fn get_ble_adapter_info(&self) -> Result<crate::BleAdapterInfo, crate::DesktopBridgeError> {
        Err(crate::DesktopBridgeError::General("BLE adapter info is only supported on Linux".to_string()))
    }

    /// List discovered BLE peers (non-Linux stub).
    pub fn list_ble_peers(&self) -> Result<Vec<crate::BlePeer>, crate::DesktopBridgeError> {
        Err(crate::DesktopBridgeError::General("BLE peer discovery is only supported on Linux".to_string()))
    }

    /// Start BLE scan (non-Linux stub).
    pub fn start_ble_scan(&self) -> Result<(), crate::DesktopBridgeError> {
        Err(crate::DesktopBridgeError::General("BLE scanning is only supported on Linux".to_string()))
    }

    /// Stop BLE scan (non-Linux stub).
    pub fn stop_ble_scan(&self) -> Result<(), crate::DesktopBridgeError> {
        Err(crate::DesktopBridgeError::General("BLE scanning is only supported on Linux".to_string()))
    }
}

impl Default for DesktopBridge {
    fn default() -> Self {
        Self::new()
    }
}
