// Rust type definitions for the `desktop` UniFFI namespace.
//
// These types translate every `dictionary`/`enum`/`callback interface` in
// `api.udl` into hand-written Rust, following the proc-macro convention
// already used by `core/src/mobile_bridge.rs` (no `.udl`-driven scaffolding
// codegen for this crate — see `build.rs`).
//
// Field names/types are taken verbatim from `api.udl`.

use serde::{Deserialize, Serialize};

// ============================================================================
// XDG PATH CONFIGURATION
// ============================================================================

/// XDG Base Directory specification paths for desktop Linux.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct XdgPaths {
    /// Data directory: $XDG_DATA_HOME/scmessenger (default: ~/.local/share/scmessenger)
    pub data_dir: String,
    /// Config directory: $XDG_CONFIG_HOME/scmessenger (default: ~/.config/scmessenger)
    pub config_dir: String,
    /// Cache directory: $XDG_CACHE_HOME/scmessenger (default: ~/.cache/scmessenger)
    pub cache_dir: String,
    /// Runtime directory: $XDG_RUNTIME_DIR/scmessenger (if available, else temp)
    pub runtime_dir: Option<String>,
    /// Sled database path: data_dir/store
    pub store_path: String,
}

// ============================================================================
// DESKTOP NOTIFICATIONS (D-Bus / Desktop Notifications Spec)
// ============================================================================

/// Desktop notification urgency level per Desktop Notifications spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

/// Desktop notification action triggered by the user.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct NotificationAction {
    /// Action identifier (e.g., "reply", "dismiss")
    pub action_id: String,
    /// Action label shown to the user
    pub label: String,
}

/// Result of a desktop notification request.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct NotificationResult {
    /// Notification ID assigned by the compositor (for replacing/dismissing)
    pub notification_id: u32,
    /// Whether the notification was successfully displayed
    pub shown: bool,
    /// Error message if not shown
    pub error_message: Option<String>,
}

/// Suppression policy for desktop notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum NotificationPriority {
    SuppressAll,
    SuppressWhenFocused,
    AllowAll,
}

// ============================================================================
// SYSTEM TRAY / APP INDICATOR
// ============================================================================

/// System tray icon state (for AppIndicator / StatusIcon backends).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum TrayIconState {
    /// No active mesh connections — idle/idle-unconnected
    Disconnected,
    /// Mesh active, no unread messages
    Connected,
    /// Mesh active, unread messages waiting
    UnreadMessages,
    /// Error state (NAT symmetric, relay unreachable)
    Error,
}

/// Status payload exposed to system tray AppIndicator widget.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct TrayStatus {
    pub icon_state: TrayIconState,
    pub unread_count: u32,
    pub connected_peers: u32,
    /// Localized tooltip text
    pub status_text: String,
}

// ============================================================================
// BLE ADAPTER STATUS (BlueZ D-Bus)
// ============================================================================

/// BLE adapter power and discovery state reported by BlueZ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum BleAdapterState {
    PoweredOff,
    PoweredOn,
    Scanning,
    Error,
}

/// BLE adapter information from BlueZ D-Bus.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct BleAdapterInfo {
    /// D-Bus object path (e.g., "/org/bluez/hci0")
    pub dbus_path: String,
    /// Human-readable name (e.g., "hci0")
    pub name: String,
    /// Bluetooth MAC address
    pub address: String,
    /// Whether the adapter is powered on
    pub powered: bool,
    /// Whether the adapter is actively scanning
    pub scanning: bool,
    /// Whether BLE advertising is active
    pub advertising: bool,
    /// Adapter state summary
    pub state: BleAdapterState,
}

/// BLE peer discovered via BlueZ D-Bus or internal mDNS.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct BlePeer {
    /// D-Bus object path or mDNS peer identifier
    pub peer_id: String,
    /// Display name (if advertised)
    pub display_name: Option<String>,
    /// RSSI in dBm (negative value)
    pub rssi: i16,
    /// Whether the peer is an SCMessenger node (matched via service UUID)
    pub is_scmessenger_node: bool,
    /// Last seen timestamp (unix epoch seconds)
    pub last_seen_secs: u64,
}

// ============================================================================
// POWER MANAGEMENT
// ============================================================================

/// Power management hint for the desktop session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum PowerProfile {
    /// Running on battery — reduce network activity
    Battery,
    /// Running on AC power — full operation
    AC,
    /// System is about to suspend
    SuspendImminent,
    /// System just resumed from suspend
    Resumed,
}

/// Power management state passed to the mesh engine.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct PowerState {
    pub profile: PowerProfile,
    /// Battery percentage (0-100, 255 if unknown)
    pub battery_pct: u8,
    /// Whether the system is on battery power
    pub on_battery: bool,
    /// Whether the screensaver / idle inhibitor is active
    pub idle_inhibited: bool,
}

// ============================================================================
// SOCKET ACTIVATION
// ============================================================================

/// Socket activation readiness state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, uniffi::Enum)]
pub enum SocketActivationState {
    /// No socket activation — daemon manages own lifecycle
    None,
    /// Listening on socket-activated FDs from systemd
    Listening,
    /// All handoff sockets accepted, no longer in activation mode
    HandoffComplete,
}

/// Socket activation status for systemd integration.
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct SocketActivationStatus {
    pub state: SocketActivationState,
    pub activated_socket_count: u32,
    pub listen_address: Option<String>,
}

// ============================================================================
// DESKTOP INTEGRATION CALLBACKS
// ============================================================================

/// Callback interface for desktop-specific events from the Rust core.
/// Implement this in Kotlin on the desktop client to receive:
///   - Notification requests
///   - Tray icon state changes
///   - Power management events
///   - BLE adapter state changes
#[uniffi::export(callback_interface)]
pub trait DesktopDelegate: Send + Sync {
    /// Request the desktop client to show a native notification.
    fn on_notification_requested(&self, title: String, body: String, urgency: NotificationUrgency);
    /// Update the system tray icon state.
    fn on_tray_state_changed(&self, status: TrayStatus);
    /// Power state changed (battery/AC/suspend).
    fn on_power_state_changed(&self, state: PowerState);
    /// BLE adapter state changed.
    fn on_ble_adapter_changed(&self, info: BleAdapterInfo);
    /// A new BLE peer was discovered.
    fn on_ble_peer_discovered(&self, peer: BlePeer);
    /// A BLE peer was lost (out of range).
    fn on_ble_peer_lost(&self, peer_id: String);
}
