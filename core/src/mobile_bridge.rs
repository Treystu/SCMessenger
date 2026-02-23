// Mobile bridge types for UniFFI bindings
//
// This module defines all the types declared in api.udl for mobile platform integration.
// These types are exposed via UniFFI to Android/iOS native code.

use serde::{Deserialize, Serialize};

// Re-export the contacts bridge
pub use crate::contacts_bridge::{Contact, ContactManager};
use crate::transport::swarm::SwarmHandle;
use libp2p::{Multiaddr, PeerId};
use parking_lot::{Mutex, RwLock};
use std::str::FromStr;
use std::sync::Arc;

// ============================================================================
// MOBILE SERVICE
// ============================================================================

#[derive(Debug, Clone)]
pub struct MeshServiceConfig {
    pub discovery_interval_ms: u32,
    pub battery_floor_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Stopping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionState {
    Still,
    Walking,
    Running,
    Automotive,
    Unknown,
}

/// Network connectivity type reported by the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    /// No connectivity.
    None,
    /// WiFi connection present.
    Wifi,
    /// Cellular data (any generation).
    Cellular,
    /// Both WiFi and cellular available.
    WifiAndCellular,
    /// Unknown / not yet reported.
    Unknown,
}

impl Default for NetworkType {
    fn default() -> Self {
        NetworkType::Unknown
    }
}

/// Snapshot of device state as reported by the platform layer.
///
/// This is the canonical state record stored inside `MeshService`.
/// It is richer than `DeviceProfile` (which is the UniFFI-facing input type)
/// and drives the threshold-based behavior adjustments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    /// Battery level 0–100.
    pub battery_level: u8,
    /// True while the device is plugged in / wirelessly charging.
    pub is_charging: bool,
    /// Active network type.
    pub network_type: NetworkType,
    /// Motion context reported by the platform accelerometer/activity API.
    pub motion_state: MotionState,
}

impl DeviceState {
    /// Construct from the UniFFI-facing `DeviceProfile`.
    pub fn from_profile(profile: &DeviceProfile) -> Self {
        let network_type = match (profile.has_wifi, profile.is_charging) {
            (true, _) => NetworkType::Wifi,
            (false, _) => NetworkType::Cellular,
        };
        Self {
            battery_level: profile.battery_pct,
            is_charging: profile.is_charging,
            network_type,
            motion_state: profile.motion_state,
        }
    }
}

/// Recommended behavior adjustments derived from the current `DeviceState`.
///
/// Callers (swarm thread, scan schedulers, relay logic) should query
/// `MeshService::recommended_behavior()` and honour these hints.
#[derive(Debug, Clone)]
pub struct BehaviorAdjustment {
    /// Suggested BLE / WiFi-Aware scan interval in milliseconds.
    /// Higher value = less frequent scanning = less battery drain.
    pub scan_interval_ms: u32,
    /// Whether relay duty should be active at all.
    pub relay_enabled: bool,
    /// Relay message budget (messages per hour, 0 means relay disabled).
    pub relay_budget: u32,
    /// True when the device should operate in the absolute minimum mode
    /// (battery critically low and not charging).
    pub minimal_operation: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceStats {
    pub peers_discovered: u32,
    pub messages_relayed: u32,
    pub bytes_transferred: u64,
    pub uptime_secs: u64,
}

/// Mobile mesh service wrapper integrating IronCore with mobile lifecycle.
///
/// Uses `parking_lot::Mutex` throughout — unlike `std::sync::Mutex` it never
/// poisons on panic, eliminating the PoisonError cascade that previously
/// caused a fatal crash when `start_swarm` panicked while holding `core`.
pub struct MeshService {
    _config: Mutex<MeshServiceConfig>,
    state: Mutex<ServiceState>,
    stats: Mutex<ServiceStats>,
    core: std::sync::Arc<Mutex<Option<crate::IronCore>>>,
    platform_bridge: std::sync::Arc<Mutex<Option<Box<dyn PlatformBridge>>>>,
    storage_path: Option<String>,
    swarm_bridge: std::sync::Arc<SwarmBridge>,
    bootstrap_addrs: Mutex<Vec<String>>,
    nat_status: Mutex<String>,
    relay_budget: std::sync::Arc<Mutex<u32>>,
    current_device_profile: Mutex<Option<DeviceProfile>>,
    /// Current device state snapshot — drives threshold-based behavior.
    /// Stored behind a `parking_lot::RwLock` so reads (very frequent) never
    /// contend with writes (infrequent platform callbacks).
    device_state: RwLock<Option<DeviceState>>,
}

impl MeshService {
    pub fn new(config: MeshServiceConfig) -> Self {
        Self {
            _config: Mutex::new(config),
            state: Mutex::new(ServiceState::Stopped),
            stats: Mutex::new(ServiceStats::default()),
            core: std::sync::Arc::new(Mutex::new(None)),
            platform_bridge: std::sync::Arc::new(Mutex::new(None)),
            storage_path: None,
            swarm_bridge: std::sync::Arc::new(SwarmBridge::new()),
            bootstrap_addrs: Mutex::new(Vec::new()),
            nat_status: Mutex::new("unknown".to_string()),
            relay_budget: std::sync::Arc::new(Mutex::new(200)),
            current_device_profile: Mutex::new(None),
            device_state: RwLock::new(None),
        }
    }

    /// Create MeshService with persistent storage
    pub fn with_storage(config: MeshServiceConfig, storage_path: String) -> Self {
        Self {
            _config: Mutex::new(config),
            state: Mutex::new(ServiceState::Stopped),
            stats: Mutex::new(ServiceStats::default()),
            core: std::sync::Arc::new(Mutex::new(None)),
            platform_bridge: std::sync::Arc::new(Mutex::new(None)),
            storage_path: Some(storage_path),
            swarm_bridge: std::sync::Arc::new(SwarmBridge::new()),
            bootstrap_addrs: Mutex::new(Vec::new()),
            nat_status: Mutex::new("unknown".to_string()),
            relay_budget: std::sync::Arc::new(Mutex::new(200)),
            current_device_profile: Mutex::new(None),
            device_state: RwLock::new(None),
        }
    }

    pub fn start(&self) -> Result<(), crate::IronCoreError> {
        let mut state = self.state.lock();

        if *state == ServiceState::Running {
            return Err(crate::IronCoreError::AlreadyRunning);
        }

        *state = ServiceState::Starting;
        drop(state);

        // Initialize IronCore
        let core = if let Some(ref path) = self.storage_path {
            crate::IronCore::with_storage(path.clone())
        } else {
            crate::IronCore::new()
        };

        // Start the core
        core.start()?;

        // Store the core instance
        *self.core.lock() = Some(core);

        // Update state
        *self.state.lock() = ServiceState::Running;

        tracing::info!("MeshService started");
        Ok(())
    }

    pub fn stop(&self) {
        let mut state = self.state.lock();

        if *state == ServiceState::Stopped {
            return;
        }

        *state = ServiceState::Stopping;
        drop(state);

        // Stop the core
        if let Some(ref core) = *self.core.lock() {
            core.stop();
        }

        // Clear the core instance
        *self.core.lock() = None;

        // Update state
        *self.state.lock() = ServiceState::Stopped;

        tracing::info!("MeshService stopped");
    }

    pub fn pause(&self) {
        tracing::info!("MeshService paused (activity reduced)");
    }

    pub fn resume(&self) {
        tracing::info!("MeshService resumed (full activity)");
    }

    pub fn get_state(&self) -> ServiceState {
        *self.state.lock()
    }

    pub fn get_stats(&self) -> ServiceStats {
        let mut stats = self.stats.lock().clone();
        let peers = self.swarm_bridge.get_peers();
        stats.peers_discovered = peers.len() as u32;
        stats
    }

    pub fn reset_stats(&self) {
        *self.stats.lock() = ServiceStats::default();
        tracing::info!("MeshService stats reset");
    }

    pub fn set_platform_bridge(&self, bridge: Option<Box<dyn PlatformBridge>>) {
        *self.platform_bridge.lock() = bridge;
    }

    /// Configure bootstrap node multiaddrs for NAT traversal.
    /// Call this BEFORE start_swarm() to have bootstrap nodes dialed on startup.
    pub fn set_bootstrap_nodes(&self, addrs: Vec<String>) {
        tracing::info!("Setting {} bootstrap node(s)", addrs.len());
        for addr in &addrs {
            tracing::info!("  Bootstrap: {}", addr);
        }
        *self.bootstrap_addrs.lock() = addrs;
    }

    /// Get current NAT status string.
    pub fn get_nat_status(&self) -> String {
        self.nat_status.lock().clone()
    }

    pub fn start_swarm(&self, listen_addr: String) -> Result<(), crate::IronCoreError> {
        // Extract keys while holding the lock, then DROP the lock before any
        // runtime/thread work.  This is critical: if anything below panics
        // while the lock is held, parking_lot will NOT poison it (unlike
        // std::sync::Mutex), but releasing early is still the safest pattern.
        let libp2p_keys = {
            let core_guard = self.core.lock();
            let core = core_guard
                .as_ref()
                .ok_or(crate::IronCoreError::NotInitialized)?;
            let identity_keys = core
                .get_identity_keys()
                .ok_or(crate::IronCoreError::NotInitialized)?;
            identity_keys
                .to_libp2p_keypair()
                .map_err(|_| crate::IronCoreError::CryptoError)?
        }; // ← core lock released here, before any runtime code

        tracing::info!(
            "Starting Swarm with PeerID: {}",
            libp2p_keys.public().to_peer_id()
        );

        let listen_multiaddr: Option<libp2p::Multiaddr> = if listen_addr.is_empty() {
            None
        } else {
            Some(
                listen_addr
                    .parse()
                    .map_err(|_| crate::IronCoreError::InvalidInput)?,
            )
        };

        let swarm_bridge = self.swarm_bridge.clone();
        let core = self.core.clone();
        let relay_budget_init = self.relay_budget.clone();

        // Spawn a dedicated OS thread that owns its own Tokio runtime.
        // This is the safest approach for mobile: we cannot rely on being
        // called from a Tokio context, and we must not hold any Mutex across
        // the thread boundary.
        std::thread::Builder::new()
            .name("scm-swarm".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .thread_name("scm-swarm-worker")
                    .build();

                match rt {
                    Ok(rt) => {
                        rt.block_on(async move {
                            let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);

                            match crate::transport::start_swarm(
                                libp2p_keys,
                                listen_multiaddr,
                                event_tx,
                            )
                            .await
                            {
                                Ok(handle) => {
                                    tracing::info!("Swarm started, wiring bridge");
                                    swarm_bridge.set_handle(handle.clone());
                                    // Apply stored relay budget
                                    let budget = *relay_budget_init.lock();
                                    if let Err(e) = handle.set_relay_budget(budget).await {
                                        tracing::warn!(
                                            "Failed to set initial relay budget: {:?}",
                                            e
                                        );
                                    }
                                    while let Some(event) = event_rx.recv().await {
                                        match event {
                                            crate::transport::SwarmEvent::MessageReceived {
                                                peer_id,
                                                envelope_data,
                                            } => {
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    match core_ref.receive_message(envelope_data) {
                                                        Ok(msg) => tracing::info!(
                                                            "Received message {} from {}",
                                                            msg.id,
                                                            peer_id
                                                        ),
                                                        Err(e) => tracing::warn!(
                                                            "receive_message error from {}: {:?}",
                                                            peer_id,
                                                            e
                                                        ),
                                                    }
                                                }
                                            }
                                            crate::transport::SwarmEvent::PeerDiscovered(
                                                peer_id,
                                            ) => {
                                                tracing::info!(
                                                    "Peer discovered via Swarm: {}",
                                                    peer_id
                                                );
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    core_ref.notify_peer_discovered(
                                                        peer_id.to_string(),
                                                    );
                                                }
                                            }
                                            crate::transport::SwarmEvent::PeerDisconnected(
                                                peer_id,
                                            ) => {
                                                tracing::info!(
                                                    "Peer disconnected via Swarm: {}",
                                                    peer_id
                                                );
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    core_ref.notify_peer_disconnected(
                                                        peer_id.to_string(),
                                                    );
                                                }
                                            }
                                            crate::transport::SwarmEvent::PeerIdentified {
                                                peer_id,
                                                listen_addrs,
                                                ..
                                            } => {
                                                tracing::info!(
                                                    "Peer identified via Swarm: {}",
                                                    peer_id
                                                );
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    if let Some(delegate) =
                                                        core_ref.delegate.read().as_ref()
                                                    {
                                                        let addrs_str: Vec<String> = listen_addrs
                                                            .iter()
                                                            .map(|a| a.to_string())
                                                            .collect();
                                                        delegate.on_peer_identified(
                                                            peer_id.to_string(),
                                                            addrs_str,
                                                        );
                                                    }
                                                }
                                            }
                                            other => {
                                                tracing::debug!("Swarm event: {:?}", other);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to start swarm: {:?}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to create swarm Tokio runtime: {}", e);
                    }
                }
            })
            .map_err(|_| crate::IronCoreError::Internal)?;

        Ok(())
    }

    pub fn get_swarm_bridge(&self) -> std::sync::Arc<SwarmBridge> {
        self.swarm_bridge.clone()
    }

    pub fn update_device_state(&self, profile: DeviceProfile) {
        let new_state = DeviceState::from_profile(&profile);

        // Read old state for transition logging (cheap read-lock).
        let old_state = self.device_state.read().clone();

        // Log any meaningful transitions before storing the new state.
        if let Some(ref old) = old_state {
            if old.battery_level != new_state.battery_level {
                tracing::debug!(
                    "Battery level changed: {}% → {}%",
                    old.battery_level,
                    new_state.battery_level
                );
            }
            if old.is_charging != new_state.is_charging {
                tracing::info!(
                    "Charging state changed: {} → {}",
                    old.is_charging,
                    new_state.is_charging
                );
            }
            if old.network_type != new_state.network_type {
                tracing::info!(
                    "Network type changed: {:?} → {:?}",
                    old.network_type,
                    new_state.network_type
                );
            }
            if old.motion_state != new_state.motion_state {
                tracing::info!(
                    "Motion state changed: {:?} → {:?}",
                    old.motion_state,
                    new_state.motion_state
                );
            }

            // Threshold-crossing events deserve explicit log entries.
            let was_critical = old.battery_level <= 10 && !old.is_charging;
            let is_critical = new_state.battery_level <= 10 && !new_state.is_charging;
            let was_low = old.battery_level <= 20 && !old.is_charging;
            let is_low = new_state.battery_level <= 20 && !new_state.is_charging;

            if !was_critical && is_critical {
                tracing::warn!(
                    "Battery CRITICAL ({}%, not charging) — entering minimal operation",
                    new_state.battery_level
                );
            } else if was_critical && !is_critical {
                tracing::info!(
                    "Battery recovered from critical ({}%{})",
                    new_state.battery_level,
                    if new_state.is_charging { ", charging" } else { "" }
                );
            } else if !was_low && is_low {
                tracing::warn!(
                    "Battery LOW ({}%, not charging) — reducing scan and relay activity",
                    new_state.battery_level
                );
            } else if was_low && !is_low {
                tracing::info!(
                    "Battery recovered from low ({}%{})",
                    new_state.battery_level,
                    if new_state.is_charging { ", charging" } else { "" }
                );
            }
        } else {
            // First report — just log the initial state.
            tracing::info!(
                "Device state initialised: battery={}% charging={} network={:?} motion={:?}",
                new_state.battery_level,
                new_state.is_charging,
                new_state.network_type,
                new_state.motion_state
            );
        }

        // Persist the new DeviceState.
        *self.device_state.write() = Some(new_state.clone());

        // Also keep the legacy DeviceProfile for callers that still use it.
        *self.current_device_profile.lock() = Some(profile);

        // Derive and apply behavior adjustments.
        let adj = Self::compute_behavior(&new_state);

        if adj.minimal_operation {
            tracing::warn!(
                "Applying MINIMAL operation mode (battery={}%)",
                new_state.battery_level
            );
        }

        self.set_relay_budget(adj.relay_budget);
    }

    /// Compute recommended behavior from a device state snapshot.
    ///
    /// This is a pure function — no side-effects — so callers can call it at
    /// any time without acquiring locks.
    pub fn compute_behavior(state: &DeviceState) -> BehaviorAdjustment {
        let battery = state.battery_level;
        let charging = state.is_charging;

        // Minimal mode: critical battery and not charging.
        if battery <= 10 && !charging {
            return BehaviorAdjustment {
                scan_interval_ms: 30_000, // 30 s — barely alive
                relay_enabled: false,
                relay_budget: 0,
                minimal_operation: true,
            };
        }

        // Low battery: reduce everything but keep messaging alive.
        if battery <= 20 && !charging {
            return BehaviorAdjustment {
                scan_interval_ms: 10_000, // 10 s
                relay_enabled: false,     // no relay duty when low
                relay_budget: 0,
                minimal_operation: false,
            };
        }

        // Stationary with good battery or charging: maximise relay duty.
        let stationary = matches!(state.motion_state, MotionState::Still);
        if charging || (battery >= 50 && stationary) {
            return BehaviorAdjustment {
                scan_interval_ms: 500, // very frequent
                relay_enabled: true,
                relay_budget: 200,
                minimal_operation: false,
            };
        }

        // Normal operation (battery 21–49, not charging, possibly moving).
        BehaviorAdjustment {
            scan_interval_ms: 2_000, // 2 s
            relay_enabled: true,
            relay_budget: 100,
            minimal_operation: false,
        }
    }

    /// Return the recommended behavior adjustments for the *current* device state.
    ///
    /// Returns `None` if no device state has been reported yet.
    pub fn recommended_behavior(&self) -> Option<BehaviorAdjustment> {
        self.device_state
            .read()
            .as_ref()
            .map(Self::compute_behavior)
    }

    /// Return a clone of the most recently stored `DeviceState`, if any.
    pub fn get_device_state(&self) -> Option<DeviceState> {
        self.device_state.read().clone()
    }

    pub fn set_relay_budget(&self, messages_per_hour: u32) {
        tracing::info!("Relay budget set: {} msgs/hour", messages_per_hour);
        *self.relay_budget.lock() = messages_per_hour;
        // If swarm is already running, forward the budget update immediately
        let handle_guard = self.swarm_bridge.handle.lock();
        if let Some(ref handle) = *handle_guard {
            let rt = self.swarm_bridge.get_runtime_handle();
            rt.block_on(handle.set_relay_budget(messages_per_hour)).ok();
        }
    }

    pub fn on_peer_discovered(&self, peer_id: String) {
        let mut stats = self.stats.lock();
        stats.peers_discovered += 1;
        tracing::info!("Peer discovered: {}", peer_id);
    }

    pub fn on_peer_disconnected(&self, peer_id: String) {
        tracing::info!("Peer disconnected: {}", peer_id);
    }

    pub fn on_data_received(&self, peer_id: String, data: Vec<u8>) {
        let mut stats = self.stats.lock();
        stats.bytes_transferred += data.len() as u64;
        drop(stats);

        if let Some(core) = self.get_core() {
            match core.receive_message(data) {
                Ok(msg) => {
                    tracing::info!("Message received from {}: {:?}", peer_id, msg.id);
                    let mut stats = self.stats.lock();
                    stats.messages_relayed += 1;
                }
                Err(e) => {
                    tracing::error!("Failed to process received message: {:?}", e);
                }
            }
        }
    }

    /// Helper to get the core instance exposed to UniFFI
    pub fn get_core(&self) -> Option<std::sync::Arc<crate::IronCore>> {
        self.core.lock().clone().map(std::sync::Arc::new)
    }

    /// Check if service is running
    pub fn is_running(&self) -> bool {
        *self.state.lock() == ServiceState::Running
    }
}

// PlatformBridge callback trait (implemented by mobile platforms)
pub trait PlatformBridge: Send + Sync {
    fn on_battery_changed(&self, battery_pct: u8, is_charging: bool);
    fn on_network_changed(&self, has_wifi: bool, has_cellular: bool);
    fn on_motion_changed(&self, motion: MotionState);
    fn on_ble_data_received(&self, peer_id: String, data: Vec<u8>);
    fn on_entering_background(&self);
    fn on_entering_foreground(&self);
    fn send_ble_packet(&self, peer_id: String, data: Vec<u8>);
}

// ============================================================================
// AUTO-ADJUST ENGINE
// ============================================================================

#[derive(Debug, Clone)]
pub struct DeviceProfile {
    pub battery_pct: u8,
    pub is_charging: bool,
    pub has_wifi: bool,
    pub motion_state: MotionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdjustmentProfile {
    Maximum,
    High,
    Standard,
    Reduced,
    Minimal,
}

#[derive(Debug, Clone)]
pub struct BleAdjustment {
    pub scan_interval_ms: u32,
    pub advertise_interval_ms: u32,
    pub tx_power_dbm: i8,
}

#[derive(Debug, Clone)]
pub struct RelayAdjustment {
    pub max_per_hour: u32,
    pub priority_threshold: u8,
    pub max_payload_bytes: u32,
}

pub struct AutoAdjustEngine {
    ble_scan_override: std::sync::Mutex<Option<u32>>,
    relay_max_override: std::sync::Mutex<Option<u32>>,
}

impl Default for AutoAdjustEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoAdjustEngine {
    pub fn new() -> Self {
        Self {
            ble_scan_override: std::sync::Mutex::new(None),
            relay_max_override: std::sync::Mutex::new(None),
        }
    }

    pub fn compute_profile(&self, device: DeviceProfile) -> AdjustmentProfile {
        // Logic from core/src/mobile/auto_adjust.rs
        if device.is_charging && device.has_wifi {
            AdjustmentProfile::Maximum
        } else if device.battery_pct > 50 {
            AdjustmentProfile::High
        } else if device.battery_pct > 30 {
            AdjustmentProfile::Standard
        } else if device.battery_pct > 15 {
            AdjustmentProfile::Reduced
        } else {
            AdjustmentProfile::Minimal
        }
    }

    pub fn compute_ble_adjustment(&self, profile: AdjustmentProfile) -> BleAdjustment {
        let (scan_interval, advertise_interval, tx_power) = match profile {
            AdjustmentProfile::Maximum => (500, 100, 4),
            AdjustmentProfile::High => (1000, 200, 0),
            AdjustmentProfile::Standard => (2000, 500, -4),
            AdjustmentProfile::Reduced => (5000, 1000, -8),
            AdjustmentProfile::Minimal => (10000, 2000, -12),
        };

        BleAdjustment {
            scan_interval_ms: self
                .ble_scan_override
                .lock()
                .unwrap()
                .unwrap_or(scan_interval),
            advertise_interval_ms: advertise_interval,
            tx_power_dbm: tx_power,
        }
    }

    pub fn compute_relay_adjustment(&self, profile: AdjustmentProfile) -> RelayAdjustment {
        let (max_per_hour, priority_threshold, max_payload) = match profile {
            AdjustmentProfile::Maximum => (1000, 0, 65536),
            AdjustmentProfile::High => (500, 50, 32768),
            AdjustmentProfile::Standard => (200, 100, 16384),
            AdjustmentProfile::Reduced => (100, 150, 8192),
            AdjustmentProfile::Minimal => (50, 200, 4096),
        };

        RelayAdjustment {
            max_per_hour: self
                .relay_max_override
                .lock()
                .unwrap()
                .unwrap_or(max_per_hour),
            priority_threshold,
            max_payload_bytes: max_payload,
        }
    }

    pub fn override_ble_scan_interval(&self, interval_ms: u32) {
        *self.ble_scan_override.lock().unwrap() = Some(interval_ms);
    }

    pub fn override_relay_max_per_hour(&self, max: u32) {
        *self.relay_max_override.lock().unwrap() = Some(max);
    }

    pub fn clear_overrides(&self) {
        *self.ble_scan_override.lock().unwrap() = None;
        *self.relay_max_override.lock().unwrap() = None;
    }
}

// ============================================================================
// MESH SETTINGS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryMode {
    Normal,
    Cautious,
    Paranoid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSettings {
    pub relay_enabled: bool,
    pub max_relay_budget: u32,
    pub battery_floor: u8,
    pub ble_enabled: bool,
    pub wifi_aware_enabled: bool,
    pub wifi_direct_enabled: bool,
    pub internet_enabled: bool,
    pub discovery_mode: DiscoveryMode,
    pub onion_routing: bool,
}

impl Default for MeshSettings {
    fn default() -> Self {
        Self {
            relay_enabled: true,
            max_relay_budget: 200,
            battery_floor: 20,
            ble_enabled: true,
            wifi_aware_enabled: true,
            wifi_direct_enabled: true,
            internet_enabled: true,
            discovery_mode: DiscoveryMode::Normal,
            onion_routing: false,
        }
    }
}

pub struct MeshSettingsManager {
    storage_path: std::path::PathBuf,
}

impl MeshSettingsManager {
    pub fn new(storage_path: String) -> Self {
        Self {
            storage_path: std::path::PathBuf::from(storage_path),
        }
    }

    pub fn load(&self) -> Result<MeshSettings, crate::IronCoreError> {
        let settings_file = self.storage_path.join("mesh_settings.json");
        if settings_file.exists() {
            let data = std::fs::read_to_string(&settings_file)
                .map_err(|_| crate::IronCoreError::StorageError)?;
            let settings: MeshSettings =
                serde_json::from_str(&data).map_err(|_| crate::IronCoreError::Internal)?;
            Ok(settings)
        } else {
            Ok(MeshSettings::default())
        }
    }

    pub fn save(&self, settings: MeshSettings) -> Result<(), crate::IronCoreError> {
        self.validate(settings.clone())?;

        std::fs::create_dir_all(&self.storage_path)
            .map_err(|_| crate::IronCoreError::StorageError)?;

        let settings_file = self.storage_path.join("mesh_settings.json");
        let data =
            serde_json::to_string_pretty(&settings).map_err(|_| crate::IronCoreError::Internal)?;
        std::fs::write(&settings_file, data).map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(())
    }

    pub fn validate(&self, settings: MeshSettings) -> Result<(), crate::IronCoreError> {
        // NOTE: relay_enabled controls BOTH sending and receiving
        // When false, ALL communication stops (bidirectional shutdown)
        // This enforces the relay=messaging principle in practice

        // If relay is enabled, max_relay_budget must be > 0
        if settings.relay_enabled && settings.max_relay_budget == 0 {
            return Err(crate::IronCoreError::InvalidInput);
        }

        // At least one transport must be enabled
        if !settings.ble_enabled
            && !settings.wifi_aware_enabled
            && !settings.wifi_direct_enabled
            && !settings.internet_enabled
        {
            return Err(crate::IronCoreError::InvalidInput);
        }

        // Battery floor must be reasonable
        if settings.battery_floor > 50 {
            return Err(crate::IronCoreError::InvalidInput);
        }

        Ok(())
    }

    pub fn default_settings(&self) -> MeshSettings {
        MeshSettings::default()
    }
}

// ============================================================================
// MESSAGE HISTORY
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    Sent,
    Received,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: String,
    pub direction: MessageDirection,
    pub peer_id: String,
    pub content: String,
    pub timestamp: u64,
    pub delivered: bool,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryStats {
    pub total_messages: u32,
    pub sent_count: u32,
    pub received_count: u32,
    pub undelivered_count: u32,
}

pub struct HistoryManager {
    db: std::sync::Arc<std::sync::Mutex<sled::Db>>,
}

impl HistoryManager {
    pub fn new(storage_path: String) -> Result<Self, crate::IronCoreError> {
        let path = std::path::PathBuf::from(storage_path).join("history.db");
        let db = sled::open(path).map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(Self {
            db: std::sync::Arc::new(std::sync::Mutex::new(db)),
        })
    }

    pub fn add(&self, record: MessageRecord) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let key = record.id.as_bytes();
        let value = serde_json::to_vec(&record).map_err(|_| crate::IronCoreError::Internal)?;
        db.insert(key, value)
            .map_err(|_| crate::IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn get(&self, id: String) -> Result<Option<MessageRecord>, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        if let Some(data) = db
            .get(id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?
        {
            let record: MessageRecord =
                serde_json::from_slice(&data).map_err(|_| crate::IronCoreError::Internal)?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub fn recent(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let mut records = Vec::new();

        for item in db.iter().rev() {
            if records.len() >= limit as usize {
                break;
            }

            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;

            if let Some(ref peer) = peer_filter {
                if &record.peer_id == peer {
                    records.push(record);
                }
            } else {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn conversation(
        &self,
        peer_id: String,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        self.recent(Some(peer_id), limit)
    }

    pub fn remove_conversation(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let mut keys_to_remove = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;

            if record.peer_id == peer_id {
                keys_to_remove.push(key);
            }
        }

        for key in keys_to_remove {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }

        Ok(())
    }

    pub fn search(
        &self,
        query: String,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for item in db.iter() {
            if results.len() >= limit as usize {
                break;
            }

            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;

            if record.content.to_lowercase().contains(&query_lower) {
                results.push(record);
            }
        }

        Ok(results)
    }

    pub fn mark_delivered(&self, id: String) -> Result<(), crate::IronCoreError> {
        if let Some(mut record) = self.get(id.clone())? {
            record.delivered = true;
            self.add(record)?;
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        db.clear().map_err(|_| crate::IronCoreError::StorageError)?;
        Ok(())
    }

    pub fn clear_conversation(&self, peer_id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let mut to_delete = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            if record.peer_id == peer_id {
                to_delete.push(key.to_vec());
            }
        }

        for key in to_delete {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }

        Ok(())
    }

    pub fn stats(&self) -> Result<HistoryStats, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let mut stats = HistoryStats::default();

        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;

            stats.total_messages += 1;
            match record.direction {
                MessageDirection::Sent => stats.sent_count += 1,
                MessageDirection::Received => stats.received_count += 1,
            }
            if !record.delivered {
                stats.undelivered_count += 1;
            }
        }

        Ok(stats)
    }

    pub fn count(&self) -> u32 {
        let db = self.db.lock().unwrap();
        db.len() as u32
    }
}

// ============================================================================
// CONNECTION LEDGER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub multiaddr: String,
    pub peer_id: Option<String>,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_seen: Option<u64>,
    pub topics: Vec<String>,
}

pub struct LedgerManager {
    storage_path: std::path::PathBuf,
    entries: std::sync::Arc<std::sync::Mutex<Vec<LedgerEntry>>>,
}

impl LedgerManager {
    pub fn new(storage_path: String) -> Self {
        Self {
            storage_path: std::path::PathBuf::from(storage_path),
            entries: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn load(&self) -> Result<(), crate::IronCoreError> {
        let ledger_file = self.storage_path.join("ledger.json");
        if ledger_file.exists() {
            let data = std::fs::read_to_string(&ledger_file)
                .map_err(|_| crate::IronCoreError::StorageError)?;
            let entries: Vec<LedgerEntry> =
                serde_json::from_str(&data).map_err(|_| crate::IronCoreError::Internal)?;
            *self.entries.lock().unwrap() = entries;
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), crate::IronCoreError> {
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|_| crate::IronCoreError::StorageError)?;

        let ledger_file = self.storage_path.join("ledger.json");
        let entries = self.entries.lock().unwrap();
        let data =
            serde_json::to_string_pretty(&*entries).map_err(|_| crate::IronCoreError::Internal)?;
        std::fs::write(&ledger_file, data).map_err(|_| crate::IronCoreError::StorageError)?;

        Ok(())
    }

    pub fn record_connection(&self, multiaddr: String, peer_id: String) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
            entry.success_count += 1;
            entry.peer_id = Some(peer_id);
            entry.last_seen = Some(current_timestamp());
        } else {
            entries.push(LedgerEntry {
                multiaddr,
                peer_id: Some(peer_id),
                success_count: 1,
                failure_count: 0,
                last_seen: Some(current_timestamp()),
                topics: Vec::new(),
            });
        }
    }

    pub fn record_failure(&self, multiaddr: String) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
            entry.failure_count += 1;
        }
    }

    pub fn dialable_addresses(&self) -> Vec<LedgerEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .filter(|e| e.success_count > 0 && e.failure_count < 5)
            .cloned()
            .collect()
    }

    pub fn get_preferred_relays(&self, limit: u32) -> Vec<LedgerEntry> {
        let entries = self.entries.lock().unwrap();
        let mut preferred: Vec<LedgerEntry> = entries
            .iter()
            .filter(|e| e.success_count > 0)
            .cloned() // Clone now so we can sort
            .collect();
        // Sort by last_seen descending
        preferred.sort_by(|a, b| b.last_seen.unwrap_or(0).cmp(&a.last_seen.unwrap_or(0)));
        preferred.truncate(limit as usize);
        preferred
    }

    pub fn all_known_topics(&self) -> Vec<String> {
        let entries = self.entries.lock().unwrap();
        let mut topics: Vec<String> = entries.iter().flat_map(|e| e.topics.clone()).collect();
        topics.sort();
        topics.dedup();
        topics
    }

    pub fn summary(&self) -> String {
        let entries = self.entries.lock().unwrap();
        format!(
            "Ledger: {} entries, {} dialable",
            entries.len(),
            entries.iter().filter(|e| e.success_count > 0).count()
        )
    }
}

// ============================================================================
// SWARM BRIDGE
// ============================================================================

/// Bridge between UniFFI (synchronous) and SwarmHandle (async).
///
/// This bridge provides synchronous wrappers around async SwarmHandle operations
/// using tokio::runtime::Handle to block on futures when necessary.
pub struct SwarmBridge {
    handle: Arc<Mutex<Option<SwarmHandle>>>,
    captured_handle: Option<tokio::runtime::Handle>,
}

impl Default for SwarmBridge {
    fn default() -> Self {
        Self::new()
    }
}
// 🚨 CRITICAL: Global runtime for network operations on mobile.
// We need this because many mobile callback threads aren't in a tokio context.
static GLOBAL_RT: parking_lot::RwLock<Option<tokio::runtime::Runtime>> =
    parking_lot::RwLock::new(None);

fn get_global_runtime() -> tokio::runtime::Handle {
    let rt_read = GLOBAL_RT.read();
    if let Some(rt) = &*rt_read {
        return rt.handle().clone();
    }
    drop(rt_read);

    let mut rt_write = GLOBAL_RT.write();
    if let Some(rt) = &*rt_write {
        return rt.handle().clone();
    }

    tracing::info!("Initializing global Tokio runtime for mobile mesh...");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create global Tokio runtime");
    let handle = rt.handle().clone();
    *rt_write = Some(rt);
    handle
}

impl SwarmBridge {
    pub fn new() -> Self {
        Self {
            handle: Arc::new(Mutex::new(None)),
            captured_handle: Some(get_global_runtime()),
        }
    }

    /// Set the SwarmHandle for this bridge.
    /// This must be called after starting the swarm to wire up network operations.
    pub fn set_handle(&self, handle: SwarmHandle) {
        *self.handle.lock() = Some(handle);
    }

    /// Internal helper to get the runtime handle for spawning
    pub fn get_runtime_handle(&self) -> tokio::runtime::Handle {
        self.captured_handle
            .clone()
            .unwrap_or_else(get_global_runtime)
    }

    /// Send an encrypted message envelope to a peer.
    pub fn send_message(&self, peer_id: String, data: Vec<u8>) -> Result<(), crate::IronCoreError> {
        let handle_guard = self.handle.lock();
        let handle = handle_guard
            .as_ref()
            .ok_or(crate::IronCoreError::NetworkError)?;

        // Parse peer ID
        let peer_id = PeerId::from_str(&peer_id).map_err(|_| crate::IronCoreError::InvalidInput)?;

        // Block on async operation
        let rt = self.get_runtime_handle();
        rt.block_on(handle.send_message(peer_id, data))
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    /// Send an encrypted message envelope to ALL connected peers.
    /// Since messages are encrypted for a specific recipient, broadcasting to all peers is safe.
    /// Only the intended recipient can decrypt the payload.
    pub fn send_to_all_peers(&self, data: Vec<u8>) -> Result<(), crate::IronCoreError> {
        let handle_guard = self.handle.lock();
        let handle = handle_guard
            .as_ref()
            .ok_or(crate::IronCoreError::NetworkError)?;

        let rt = self.get_runtime_handle();
        let peers = rt.block_on(handle.get_peers()).unwrap_or_default();

        if peers.is_empty() {
            tracing::warn!("send_to_all_peers: no connected peers, message queued locally");
            return Ok(());
        }

        let mut sent = 0usize;
        for peer_id in peers {
            match rt.block_on(handle.send_message(peer_id, data.clone())) {
                Ok(()) => sent += 1,
                Err(e) => {
                    tracing::warn!("send_to_all_peers: failed to send to {}: {:?}", peer_id, e)
                }
            }
        }

        tracing::info!("send_to_all_peers: sent to {} peers", sent);
        Ok(())
    }

    /// Dial a peer at a multiaddress.
    pub fn dial(&self, multiaddr: String) -> Result<(), crate::IronCoreError> {
        let handle_guard = self.handle.lock();
        let handle = handle_guard
            .as_ref()
            .ok_or(crate::IronCoreError::NetworkError)?;

        // Parse multiaddress
        let addr =
            Multiaddr::from_str(&multiaddr).map_err(|_| crate::IronCoreError::InvalidInput)?;

        // Block on async operation
        let rt = self.get_runtime_handle();
        rt.block_on(handle.dial(addr))
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    /// Get list of connected peer IDs.
    pub fn get_peers(&self) -> Vec<String> {
        let handle_guard = self.handle.lock();
        let handle = match handle_guard.as_ref() {
            Some(h) => h,
            None => return Vec::new(),
        };

        // Block on async operation
        let rt = self.get_runtime_handle();
        rt.block_on(handle.get_peers())
            .unwrap_or_default()
            .iter()
            .map(|peer_id| peer_id.to_string())
            .collect()
    }

    /// Get list of listening addresses.
    pub fn get_listeners(&self) -> Vec<String> {
        let handle_guard = self.handle.lock();
        let handle = match handle_guard.as_ref() {
            Some(h) => h,
            None => return Vec::new(),
        };

        // Block on async operation
        let rt = self.get_runtime_handle();
        rt.block_on(handle.get_listeners())
            .unwrap_or_default()
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }

    /// Get external addresses observed by peer nodes on the mesh.
    ///
    /// Uses the libp2p `identify` protocol: when any connected peer observes
    /// the address from which we connected them, they report it back. These
    /// addresses are NAT-mapped and confirmed by actual mesh peers — no
    /// outside infrastructure required.
    ///
    /// Use for display/diagnostics only. Do NOT include in BLE beacons
    /// (they are observed outbound NAT ports, not stable inbound addresses).
    pub fn get_external_addresses(&self) -> Vec<String> {
        let handle_guard = self.handle.lock();
        let handle = match handle_guard.as_ref() {
            Some(h) => h,
            None => return Vec::new(),
        };

        let rt = self.get_runtime_handle();
        rt.block_on(handle.get_external_addresses())
            .unwrap_or_default()
            .iter()
            .map(|addr| addr.to_string())
            .collect()
    }

    /// Get list of subscribed Gossipsub topics.
    pub fn get_topics(&self) -> Vec<String> {
        let handle_guard = self.handle.lock();
        let handle = match handle_guard.as_ref() {
            Some(h) => h,
            None => return Vec::new(),
        };

        // Block on async operation
        let rt = self.get_runtime_handle();
        rt.block_on(handle.get_topics()).unwrap_or_default()
    }

    /// Subscribe to a Gossipsub topic.
    pub fn subscribe_topic(&self, topic: String) -> Result<(), crate::IronCoreError> {
        let handle_guard = self.handle.lock();
        let handle = handle_guard
            .as_ref()
            .ok_or(crate::IronCoreError::NetworkError)?;

        // Block on async operation
        let rt = self.get_runtime_handle();
        rt.block_on(handle.subscribe_topic(topic))
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    /// Unsubscribe from a Gossipsub topic.
    pub fn unsubscribe_topic(&self, topic: String) -> Result<(), crate::IronCoreError> {
        let handle_guard = self.handle.lock();
        let handle = handle_guard
            .as_ref()
            .ok_or(crate::IronCoreError::NetworkError)?;

        let rt = self.get_runtime_handle();
        rt.block_on(handle.unsubscribe_topic(topic))
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    /// Publish data to a Gossipsub topic.
    pub fn publish_topic(
        &self,
        topic: String,
        data: Vec<u8>,
    ) -> Result<(), crate::IronCoreError> {
        let handle_guard = self.handle.lock();
        let handle = handle_guard
            .as_ref()
            .ok_or(crate::IronCoreError::NetworkError)?;

        let rt = self.get_runtime_handle();
        rt.block_on(handle.publish_topic(topic, data))
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    /// Shutdown the swarm gracefully.
    pub fn shutdown(&self) {
        let handle_guard = self.handle.lock();
        if let Some(handle) = handle_guard.as_ref() {
            let rt = self.get_runtime_handle();
            let _ = rt.block_on(handle.shutdown());
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // -----------------------------------------------------------------------
    // DeviceState / BehaviorAdjustment tests
    // -----------------------------------------------------------------------

    fn make_state(battery: u8, charging: bool, motion: MotionState) -> DeviceState {
        DeviceState {
            battery_level: battery,
            is_charging: charging,
            network_type: NetworkType::Wifi,
            motion_state: motion,
        }
    }

    #[test]
    fn test_compute_behavior_minimal_mode() {
        // <= 10% and not charging → minimal operation
        let adj = MeshService::compute_behavior(&make_state(10, false, MotionState::Still));
        assert!(adj.minimal_operation);
        assert!(!adj.relay_enabled);
        assert_eq!(adj.relay_budget, 0);
        assert!(adj.scan_interval_ms >= 10_000);

        // Charging saves it even at 5%
        let adj_charging = MeshService::compute_behavior(&make_state(5, true, MotionState::Still));
        assert!(!adj_charging.minimal_operation);
    }

    #[test]
    fn test_compute_behavior_low_battery() {
        // 20% not charging → no relay, not minimal
        let adj = MeshService::compute_behavior(&make_state(20, false, MotionState::Walking));
        assert!(!adj.minimal_operation);
        assert!(!adj.relay_enabled);
        assert_eq!(adj.relay_budget, 0);
        assert!(adj.scan_interval_ms > 2_000);

        // 21% not charging → normal
        let adj21 = MeshService::compute_behavior(&make_state(21, false, MotionState::Walking));
        assert!(adj21.relay_enabled);
    }

    #[test]
    fn test_compute_behavior_stationary_good_battery() {
        // Stationary + battery >= 50 → maximum relay
        let adj = MeshService::compute_behavior(&make_state(60, false, MotionState::Still));
        assert!(adj.relay_enabled);
        assert_eq!(adj.relay_budget, 200);
        assert!(adj.scan_interval_ms <= 500);
    }

    #[test]
    fn test_compute_behavior_charging_always_full() {
        // Charging at any battery level → full relay
        let adj = MeshService::compute_behavior(&make_state(15, true, MotionState::Automotive));
        assert!(adj.relay_enabled);
        assert_eq!(adj.relay_budget, 200);
    }

    #[test]
    fn test_compute_behavior_normal_operation() {
        // 30% not charging, moving → normal
        let adj = MeshService::compute_behavior(&make_state(30, false, MotionState::Walking));
        assert!(adj.relay_enabled);
        assert_eq!(adj.relay_budget, 100);
        assert_eq!(adj.scan_interval_ms, 2_000);
    }

    #[test]
    fn test_device_state_from_profile() {
        let profile = DeviceProfile {
            battery_pct: 55,
            is_charging: false,
            has_wifi: true,
            motion_state: MotionState::Still,
        };
        let state = DeviceState::from_profile(&profile);
        assert_eq!(state.battery_level, 55);
        assert!(!state.is_charging);
        assert_eq!(state.network_type, NetworkType::Wifi);
        assert_eq!(state.motion_state, MotionState::Still);
    }

    #[test]
    fn test_update_device_state_stores_state() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 1000,
            battery_floor_pct: 20,
        });

        assert!(svc.get_device_state().is_none());
        assert!(svc.recommended_behavior().is_none());

        let profile = DeviceProfile {
            battery_pct: 80,
            is_charging: false,
            has_wifi: true,
            motion_state: MotionState::Still,
        };
        svc.update_device_state(profile);

        let state = svc.get_device_state().unwrap();
        assert_eq!(state.battery_level, 80);

        let adj = svc.recommended_behavior().unwrap();
        assert!(adj.relay_enabled);
        assert_eq!(adj.relay_budget, 200); // stationary + good battery
    }

    #[test]
    fn test_update_device_state_transitions() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 1000,
            battery_floor_pct: 20,
        });

        // First update
        svc.update_device_state(DeviceProfile {
            battery_pct: 50,
            is_charging: false,
            has_wifi: true,
            motion_state: MotionState::Walking,
        });

        // Transition to low battery
        svc.update_device_state(DeviceProfile {
            battery_pct: 15,
            is_charging: false,
            has_wifi: false,
            motion_state: MotionState::Walking,
        });

        let adj = svc.recommended_behavior().unwrap();
        assert!(!adj.relay_enabled);
        assert_eq!(adj.relay_budget, 0);
        assert!(!adj.minimal_operation);

        // Transition to critical battery
        svc.update_device_state(DeviceProfile {
            battery_pct: 8,
            is_charging: false,
            has_wifi: false,
            motion_state: MotionState::Still,
        });

        let adj = svc.recommended_behavior().unwrap();
        assert!(adj.minimal_operation);
    }

    // -----------------------------------------------------------------------
    // Existing tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_ledger_preferred_relays() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let ledger = LedgerManager::new(path);

        // Add some entries
        ledger.record_connection("/ip4/1.2.3.4/tcp/1000".to_string(), "peer1".to_string());
        ledger.record_connection("/ip4/1.2.3.4/tcp/1000".to_string(), "peer1".to_string()); // Make it successful

        // Simulate time passing and another peer
        std::thread::sleep(std::time::Duration::from_millis(10));
        ledger.record_connection("/ip4/5.6.7.8/tcp/2000".to_string(), "peer2".to_string());
        ledger.record_connection("/ip4/5.6.7.8/tcp/2000".to_string(), "peer2".to_string());

        let preferred = ledger.get_preferred_relays(10);
        assert_eq!(preferred.len(), 2);

        // Peer 2 should be first because it was seen last
        assert_eq!(preferred[0].peer_id, Some("peer2".to_string()));
        assert_eq!(preferred[1].peer_id, Some("peer1".to_string()));

        // Test limit
        let limited = ledger.get_preferred_relays(1);
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].peer_id, Some("peer2".to_string()));
    }
}
