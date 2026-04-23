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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionPathState {
    Disconnected,
    Bootstrapping,
    DirectPreferred,
    RelayFallback,
    RelayOnly,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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
    #[default]
    Unknown,
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
    log_directory: Option<String>,
    swarm_bridge: std::sync::Arc<SwarmBridge>,
    bootstrap_addrs: Mutex<Vec<String>>,
    nat_status: std::sync::Arc<Mutex<String>>,
    relay_budget: std::sync::Arc<Mutex<u32>>,
    swarm_headless_mode: std::sync::Arc<Mutex<Option<bool>>>,
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
            log_directory: None,
            swarm_bridge: std::sync::Arc::new(SwarmBridge::new()),
            bootstrap_addrs: Mutex::new(Vec::new()),
            nat_status: std::sync::Arc::new(Mutex::new("unknown".to_string())),
            relay_budget: std::sync::Arc::new(Mutex::new(200)),
            swarm_headless_mode: std::sync::Arc::new(Mutex::new(None)),
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
            log_directory: None,
            swarm_bridge: std::sync::Arc::new(SwarmBridge::new()),
            bootstrap_addrs: Mutex::new(Vec::new()),
            nat_status: std::sync::Arc::new(Mutex::new("unknown".to_string())),
            relay_budget: std::sync::Arc::new(Mutex::new(200)),
            swarm_headless_mode: std::sync::Arc::new(Mutex::new(None)),
            current_device_profile: Mutex::new(None),
            device_state: RwLock::new(None),
        }
    }

    /// Create MeshService with persistent storage and structured tracing
    pub fn with_storage_and_logs(
        config: MeshServiceConfig,
        storage_path: String,
        log_directory: String,
    ) -> Self {
        Self {
            _config: Mutex::new(config),
            state: Mutex::new(ServiceState::Stopped),
            stats: Mutex::new(ServiceStats::default()),
            core: std::sync::Arc::new(Mutex::new(None)),
            platform_bridge: std::sync::Arc::new(Mutex::new(None)),
            storage_path: Some(storage_path),
            log_directory: Some(log_directory),
            swarm_bridge: std::sync::Arc::new(SwarmBridge::new()),
            bootstrap_addrs: Mutex::new(Vec::new()),
            nat_status: std::sync::Arc::new(Mutex::new("unknown".to_string())),
            relay_budget: std::sync::Arc::new(Mutex::new(200)),
            swarm_headless_mode: std::sync::Arc::new(Mutex::new(None)),
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

        tracing::info!(
            "MeshService::start: storage_path={:?}, log_directory={:?}",
            self.storage_path,
            self.log_directory
        );

        // Initialize IronCore
        let core = if let Some(ref log_dir) = self.log_directory {
            if let Some(ref path) = self.storage_path {
                tracing::info!("MeshService::start: Creating IronCore::with_storage_and_logs");
                let core = crate::IronCore::with_storage_and_logs(path.clone(), log_dir.clone());
                tracing::info!("MeshService::start: IronCore::with_storage_and_logs completed");
                core
            } else {
                tracing::info!("MeshService::start: Creating IronCore::new (no storage path)");
                crate::IronCore::new()
            }
        } else if let Some(ref path) = self.storage_path {
            tracing::info!("MeshService::start: Creating IronCore::with_storage at {:?}", path);
            let core = crate::IronCore::with_storage(path.clone());
            tracing::info!("MeshService::start: IronCore::with_storage completed");
            core
        } else {
            tracing::info!("MeshService::start: Creating IronCore::new (no storage)");
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
        *self.swarm_headless_mode.lock() = None;

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

    pub fn get_connection_path_state(&self) -> ConnectionPathState {
        let peers = self.swarm_bridge.get_peers();
        let listeners = self.swarm_bridge.get_listeners();
        let bootstrap = self.bootstrap_addrs.lock().clone();
        let nat = self.nat_status.lock().clone();

        if peers.is_empty() {
            return if bootstrap.is_empty() {
                ConnectionPathState::Disconnected
            } else {
                ConnectionPathState::Bootstrapping
            };
        }

        if !listeners.is_empty() && nat != "symmetric" {
            return ConnectionPathState::DirectPreferred;
        }

        if !bootstrap.is_empty() {
            return ConnectionPathState::RelayFallback;
        }

        ConnectionPathState::RelayOnly
    }

    pub fn export_diagnostics(&self) -> String {
        let stats = self.get_stats();
        let payload = serde_json::json!({
            "service_state": format!("{:?}", self.get_state()),
            "connection_path_state": format!("{:?}", self.get_connection_path_state()),
            "nat_status": self.get_nat_status(),
            "bootstrap_addrs": self.bootstrap_addrs.lock().clone(),
            "peers": self.swarm_bridge.get_peers(),
            "listeners": self.swarm_bridge.get_listeners(),
            "external_addrs": self.swarm_bridge.get_external_addresses(),
            "relay_budget": *self.relay_budget.lock(),
            "stats": {
                "peers_discovered": stats.peers_discovered,
                "messages_relayed": stats.messages_relayed,
                "bytes_transferred": stats.bytes_transferred,
                "uptime_secs": stats.uptime_secs
            },
            "timestamp_ms": current_timestamp(),
        });

        payload.to_string()
    }

    fn resolve_swarm_keypair_and_mode(
        &self,
    ) -> Result<(libp2p::identity::Keypair, bool), crate::IronCoreError> {
        let identity_keypair = {
            let core_guard = self.core.lock();
            let core = core_guard
                .as_ref()
                .ok_or(crate::IronCoreError::NotInitialized)?;
            core.get_libp2p_keypair().ok()
        };

        if let Some(keypair) = identity_keypair {
            return Ok((keypair, false));
        }

        tracing::info!("No identity keypair available; using persisted headless network key");
        let keypair = self.load_or_create_headless_network_keypair()?;
        Ok((keypair, true))
    }

    fn load_or_create_headless_network_keypair(
        &self,
    ) -> Result<libp2p::identity::Keypair, crate::IronCoreError> {
        const HEADLESS_KEY_FILE: &str = "relay_network_key.pb";

        let Some(storage_path) = self.storage_path.as_ref() else {
            tracing::warn!("MeshService has no storage path; using ephemeral headless keypair");
            return Ok(libp2p::identity::Keypair::generate_ed25519());
        };

        let storage_dir = std::path::PathBuf::from(storage_path);
        std::fs::create_dir_all(&storage_dir).map_err(|_| crate::IronCoreError::StorageError)?;
        let key_path = storage_dir.join(HEADLESS_KEY_FILE);

        if key_path.exists() {
            let bytes = std::fs::read(&key_path).map_err(|_| crate::IronCoreError::StorageError)?;
            match libp2p::identity::Keypair::from_protobuf_encoding(&bytes) {
                Ok(keypair) => return Ok(keypair),
                Err(err) => {
                    tracing::warn!(
                        "Failed to decode headless network key at {} ({}); rotating key",
                        key_path.display(),
                        err
                    );
                }
            }
        }

        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let encoded = keypair
            .to_protobuf_encoding()
            .map_err(|_| crate::IronCoreError::Internal)?;
        std::fs::write(&key_path, encoded).map_err(|_| crate::IronCoreError::StorageError)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600));
        }

        Ok(keypair)
    }

    pub fn start_swarm(&self, listen_addr: String) -> Result<(), crate::IronCoreError> {
        // Extract keys while holding the lock, then DROP the lock before any
        // runtime/thread work.  This is critical: if anything below panics
        // while the lock is held, parking_lot will NOT poison it (unlike
        // std::sync::Mutex), but releasing early is still the safest pattern.
        let (libp2p_keys, headless_mode) = self.resolve_swarm_keypair_and_mode()?;

        let has_existing_handle = self.swarm_bridge.handle.lock().is_some();
        let existing_mode = *self.swarm_headless_mode.lock();
        if has_existing_handle {
            if existing_mode == Some(headless_mode) {
                tracing::info!(
                    "Swarm already running in {} mode; skipping restart",
                    if headless_mode { "headless" } else { "full" }
                );
                return Ok(());
            }

            tracing::info!(
                "Swarm mode change requested ({} -> {}); restarting swarm",
                if existing_mode == Some(true) {
                    "headless"
                } else {
                    "full"
                },
                if headless_mode { "headless" } else { "full" }
            );
            self.swarm_bridge.shutdown();
            *self.swarm_bridge.handle.lock() = None;
            *self.swarm_headless_mode.lock() = None;
        }

        tracing::info!(
            "Starting Swarm with PeerID: {}",
            libp2p_keys.public().to_peer_id()
        );
        eprintln!(
            "=== OWN_IDENTITY: {} ===",
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
        let bootstrap_addrs = self.bootstrap_addrs.lock().clone();
        let nat_status = self.nat_status.clone();
        let swarm_mode_state = self.swarm_headless_mode.clone();
        let service_storage_path = self.storage_path.clone();

        // Spawn a dedicated OS thread that owns its own Tokio runtime.
        // This is the safest approach for mobile: we cannot rely on being
        // called from a Tokio context, and we must not hold any Mutex across
        // the thread boundary.
        std::thread::Builder::new()
            .name("scm-swarm".to_string())
            .spawn(move || {
                #[cfg(not(target_arch = "wasm32"))]
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .thread_name("scm-swarm-worker")
                    .build();

                #[cfg(target_arch = "wasm32")]
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();

                match rt {
                    Ok(rt) => {
                        rt.block_on(async move {
                            let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);
                            let bootstrap_multiaddrs: Vec<libp2p::Multiaddr> = bootstrap_addrs
                                .iter()
                                .filter_map(|raw| match raw.parse::<libp2p::Multiaddr>() {
                                    Ok(addr) => Some(addr),
                                    Err(e) => {
                                        tracing::warn!(
                                            "Invalid bootstrap multiaddr '{}': {}",
                                            raw,
                                            e
                                        );
                                        None
                                    }
                                })
                                .collect();

                            tracing::info!(
                                "Starting swarm with {} bootstrap addr(s)",
                                bootstrap_multiaddrs.len()
                            );

                            let iron_core_handle = {
                                let core_guard = core.lock();
                                core_guard.clone()
                            };

                            match crate::transport::start_swarm_with_config(
                                libp2p_keys,
                                listen_multiaddr,
                                event_tx,
                                None,
                                bootstrap_multiaddrs,
                                service_storage_path,
                                iron_core_handle.map(|c| {
                                    // CRITICAL: We need a Weak<IronCore> that points to a live Arc.
                                    // Since IronCore itself is a collection of Arcs, we need to wrap
                                    // the IronCore struct in an Arc to downgrade it correctly.
                                    Arc::downgrade(&Arc::new(c))
                                }),
                                headless_mode,
                            )
                            .await
                            {
                                Ok(handle) => {
                                    tracing::info!("Swarm started, wiring bridge");
                                    swarm_bridge.set_handle(handle.clone());
                                    *swarm_mode_state.lock() = Some(headless_mode);
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
                                                    match core_ref.receive_message(envelope_data.clone()) {
                                                        Ok(msg) => {
                                                            tracing::info!(
                                                                "Received message {} from {}",
                                                                msg.id,
                                                                peer_id
                                                            );
                                                            eprintln!(
                                                                "[IronCore] ✓ Received message {} from {} (type={:?})",
                                                                msg.id,
                                                                peer_id,
                                                                msg.message_type
                                                            );
                                                        }
                                                        Err(e) => {
                                                            let err_detail = format!("{:?}", e);
                                                            tracing::warn!(
                                                                "receive_message error from {}: {}",
                                                                peer_id,
                                                                err_detail
                                                            );
                                                            // CRITICAL: eprintln! is the ONLY way to surface
                                                            // errors on mobile — tracing goes to /dev/null.
                                                            eprintln!(
                                                                "[IronCore] ✗ receive_message FAILED from {}: {} (envelope_len={})",
                                                                peer_id,
                                                                err_detail,
                                                                envelope_data.len()
                                                            );
                                                        }
                                                    }
                                                } else {
                                                    eprintln!(
                                                        "[IronCore] ✗ receive_message SKIPPED from {}: core not initialized",
                                                        peer_id
                                                    );
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
                                                agent_version,
                                                listen_addrs,
                                                ..
                                            } => {
                                                let registration_request = if headless_mode {
                                                    None
                                                } else {
                                                    let core_guard = core.lock();
                                                    core_guard
                                                        .as_ref()
                                                        .and_then(|core_ref| {
                                                            core_ref.build_registration_request().ok()
                                                        })
                                                };
                                                if let Some(request) = registration_request {
                                                    if let Err(err) =
                                                        handle.register_identity(peer_id, request).await
                                                    {
                                                        tracing::warn!(
                                                            "Failed to register local identity with {}: {:?}",
                                                            peer_id,
                                                            err
                                                        );
                                                    }
                                                }
                                                tracing::info!(
                                                    "Peer identified via Swarm: {} (agent: {})",
                                                    peer_id,
                                                    agent_version
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
                                                            agent_version.clone(),
                                                            addrs_str,
                                                        );
                                                    }
                                                }
                                            }
                                            crate::transport::SwarmEvent::NatStatusChanged(
                                                status,
                                            ) => {
                                                tracing::info!("🔭 NAT status updated: {}", status);
                                                *nat_status.lock() = status;
                                            }
                                            crate::transport::SwarmEvent::PortMapping(status) => {
                                                tracing::info!("🌐 Port mapping updated: {}", status);
                                            }
                                            crate::transport::SwarmEvent::AbuseSignalDetected {
                                                peer_id,
                                                signal,
                                            } => {
                                                tracing::info!(
                                                    "Abuse signal detected from {}: {}",
                                                    peer_id,
                                                    signal
                                                );
                                                let core_guard = core.lock();
                                                if let Some(core_ref) = core_guard.as_ref() {
                                                    core_ref.record_abuse_signal(
                                                        peer_id.to_string(),
                                                        signal,
                                                    );
                                                }
                                            }
                                            other => {
                                                tracing::debug!("Swarm event: {:?}", other);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    *swarm_mode_state.lock() = None;
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
                    if new_state.is_charging {
                        ", charging"
                    } else {
                        ""
                    }
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
                    if new_state.is_charging {
                        ", charging"
                    } else {
                        ""
                    }
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

        eprintln!(
            "[IronCore] on_data_received from {} ({} bytes)",
            peer_id,
            data.len()
        );
        if let Some(core) = self.get_core() {
            match core.receive_message(data) {
                Ok(msg) => {
                    tracing::info!("Message received from {}: {:?}", peer_id, msg.id);
                    eprintln!(
                        "[IronCore] ✓ BLE message received from {}: {}",
                        peer_id, msg.id
                    );
                    let mut stats = self.stats.lock();
                    stats.messages_relayed += 1;
                }
                Err(e) => {
                    tracing::error!("Failed to process received message: {:?}", e);
                    eprintln!(
                        "[IronCore] ✗ BLE receive_message FAILED from {}: {:?}",
                        peer_id, e
                    );
                }
            }
        } else {
            eprintln!(
                "[IronCore] ✗ on_data_received SKIPPED from {}: core not initialized",
                peer_id
            );
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
#[serde(default)]
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
    pub cover_traffic_enabled: bool,
    pub message_padding_enabled: bool,
    pub timing_obfuscation_enabled: bool,
    pub notifications_enabled: bool,
    pub notify_dm_enabled: bool,
    pub notify_dm_request_enabled: bool,
    pub notify_dm_in_foreground: bool,
    pub notify_dm_request_in_foreground: bool,
    pub sound_enabled: bool,
    pub badge_enabled: bool,
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
            cover_traffic_enabled: false,
            message_padding_enabled: false,
            timing_obfuscation_enabled: false,
            notifications_enabled: crate::notification_defaults::notifications_enabled(),
            notify_dm_enabled: crate::notification_defaults::notify_dm_enabled(),
            notify_dm_request_enabled: crate::notification_defaults::notify_dm_request_enabled(),
            notify_dm_in_foreground: crate::notification_defaults::notify_dm_in_foreground(),
            notify_dm_request_in_foreground:
                crate::notification_defaults::notify_dm_request_in_foreground(),
            sound_enabled: crate::notification_defaults::sound_enabled(),
            badge_enabled: crate::notification_defaults::badge_enabled(),
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
// BOOTSTRAP CONFIGURATION
// ============================================================================

/// Configuration for bootstrap node resolution.
///
/// Resolution order: `env_override_key` → `remote_url` → `static_nodes`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    /// Static fallback multiaddr strings.
    pub static_nodes: Vec<String>,
    /// Optional remote URL to fetch a JSON array of multiaddr strings.
    pub remote_url: Option<String>,
    /// Timeout for the remote fetch, in seconds.
    pub fetch_timeout_secs: u32,
    /// Optional environment variable name for a comma-separated override list.
    pub env_override_key: Option<String>,
}

/// Resolves bootstrap node addresses using a deterministic priority chain.
pub struct BootstrapResolver {
    config: BootstrapConfig,
}

impl BootstrapResolver {
    pub fn new(config: BootstrapConfig) -> Self {
        Self { config }
    }

    /// Resolve bootstrap nodes: env → remote → static.
    pub fn resolve(&self) -> Vec<String> {
        // 1. Environment override (highest priority)
        if let Some(ref env_key) = self.config.env_override_key {
            if let Ok(val) = std::env::var(env_key) {
                let addrs: Vec<String> = val
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !addrs.is_empty() {
                    tracing::info!(
                        "Bootstrap resolved via env var '{}': {} addr(s)",
                        env_key,
                        addrs.len()
                    );
                    return addrs;
                }
            }
        }

        // 2. Remote URL fetch (medium priority) — non-WASM only
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref url) = self.config.remote_url {
            match self.fetch_remote(url) {
                Ok(addrs) if !addrs.is_empty() => {
                    tracing::info!("Bootstrap resolved via remote URL: {} addr(s)", addrs.len());
                    return addrs;
                }
                Ok(_) => {
                    tracing::warn!(
                        "Remote bootstrap URL returned empty list; falling back to static"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Remote bootstrap fetch failed: {}; falling back to static",
                        e
                    );
                }
            }
        }

        // 3. Static fallback (lowest priority)
        tracing::info!(
            "Bootstrap resolved via static fallback: {} addr(s)",
            self.config.static_nodes.len()
        );
        self.config.static_nodes.clone()
    }

    /// Return the raw static fallback list without env/remote resolution.
    pub fn static_fallback(&self) -> Vec<String> {
        self.config.static_nodes.clone()
    }

    /// Attempt to fetch bootstrap nodes from a remote URL (non-WASM only).
    /// Expects a JSON array of strings: `["/ip4/1.2.3.4/tcp/9001/p2p/..."]`
    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_remote(&self, url: &str) -> Result<Vec<String>, String> {
        let timeout = web_time::Duration::from_secs(self.config.fetch_timeout_secs as u64);
        let resp = ureq::AgentBuilder::new()
            .timeout(timeout)
            .build()
            .get(url)
            .call()
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        let body = resp
            .into_string()
            .map_err(|e| format!("Failed to read response body: {}", e))?;
        let addrs: Vec<String> =
            serde_json::from_str(&body).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        Ok(addrs)
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
    #[serde(default)]
    pub sender_timestamp: u64,
    pub delivered: bool,
    /// When `true` the message is from a blocked-only peer and is retained for
    /// evidentiary purposes but must be filtered out of all UI-facing queries.
    /// The flag is cleared when the peer is unblocked.
    #[serde(default)]
    pub hidden: bool,
}

impl MessageRecord {
    fn adjust_legacy_timestamps(mut self) -> Self {
        if self.sender_timestamp == 0 {
            self.sender_timestamp = self.timestamp;
        }
        self
    }
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
        let db = sled::Config::default()
            .path(path)
            .mode(sled::Mode::LowSpace)
            .use_compression(false)
            .open()
            .map_err(|_| crate::IronCoreError::StorageError)?;

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
            Ok(Some(record.adjust_legacy_timestamps()))
        } else {
            Ok(None)
        }
    }

    pub fn recent(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        self.recent_internal(peer_filter, limit, false)
    }

    /// Like `recent()` but also returns messages that are hidden due to the
    /// sender being blocked.  Used by administrative / evidentiary access paths.
    pub fn recent_including_hidden(
        &self,
        peer_filter: Option<String>,
        limit: u32,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        self.recent_internal(peer_filter, limit, true)
    }

    fn recent_internal(
        &self,
        peer_filter: Option<String>,
        limit: u32,
        include_hidden: bool,
    ) -> Result<Vec<MessageRecord>, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let mut records = Vec::new();

        for item in db.iter() {
            let (_, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            let record = record.adjust_legacy_timestamps();

            // Evidentiary retention: skip hidden messages in normal queries.
            if record.hidden && !include_hidden {
                continue;
            }

            if let Some(ref peer) = peer_filter {
                if &record.peer_id == peer {
                    records.push(record);
                }
            } else {
                records.push(record);
            }
        }

        // Do not rely on sled key order (message IDs are not time-ordered).
        // Sort explicitly so callers receive newest records first.
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp).then_with(|| b.id.cmp(&a.id)));
        if records.len() > limit as usize {
            records.truncate(limit as usize);
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
            let record = record.adjust_legacy_timestamps();

            if record.peer_id.eq_ignore_ascii_case(&peer_id) {
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
            let record = record.adjust_legacy_timestamps();

            // Evidentiary retention: skip hidden messages in search results.
            if record.hidden {
                continue;
            }

            if record.content.to_lowercase().contains(&query_lower) {
                results.push(record);
            }
        }

        Ok(results)
    }

    /// Unhide all stored messages for a given peer (called on unblock).
    pub fn unhide_messages_for_peer(&self, peer_id: &str) -> Result<u32, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let mut to_update: Vec<(Vec<u8>, MessageRecord)> = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            if record.hidden && record.peer_id.eq_ignore_ascii_case(peer_id) {
                to_update.push((key.to_vec(), record));
            }
        }

        let count = to_update.len() as u32;
        for (key, mut record) in to_update {
            record.hidden = false;
            let updated =
                serde_json::to_vec(&record).map_err(|_| crate::IronCoreError::Internal)?;
            db.insert(key, updated)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }
        Ok(count)
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
            let record = record.adjust_legacy_timestamps();
            // P0_SECURITY_001: Case-insensitive peer ID matching to match generic HistoryManager behavior
            if record.peer_id.eq_ignore_ascii_case(&peer_id) {
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
            let record = record.adjust_legacy_timestamps();

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

    pub fn flush(&self) {
        if let Ok(db) = self.db.lock() {
            let _ = db.flush();
        }
    }

    /// Enforce a maximum message retention cap.
    ///
    /// Keeps the `max_messages` most recent messages (by timestamp) and
    /// removes the rest.  Returns the number of pruned records.
    pub fn enforce_retention(&self, max_messages: u32) -> Result<u32, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let total = db.len();
        if total <= max_messages as usize {
            return Ok(0);
        }

        // Collect all (key, timestamp) pairs
        let mut entries: Vec<(Vec<u8>, u64)> = Vec::with_capacity(total);
        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            entries.push((key.to_vec(), record.timestamp));
        }

        // Sort by timestamp descending (newest first)
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove everything after max_messages
        let mut pruned: u32 = 0;
        for (key, _) in entries.into_iter().skip(max_messages as usize) {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
            pruned += 1;
        }

        Ok(pruned)
    }

    /// Remove all messages with timestamp before the given Unix epoch seconds.
    ///
    /// Returns the number of pruned records.
    pub fn prune_before(&self, before_timestamp: u64) -> Result<u32, crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        let mut keys_to_remove = Vec::new();

        for item in db.iter() {
            let (key, value) = item.map_err(|_| crate::IronCoreError::StorageError)?;
            let record: MessageRecord =
                serde_json::from_slice(&value).map_err(|_| crate::IronCoreError::Internal)?;
            if record.timestamp < before_timestamp {
                keys_to_remove.push(key.to_vec());
            }
        }

        let pruned = keys_to_remove.len() as u32;
        for key in keys_to_remove {
            db.remove(key)
                .map_err(|_| crate::IronCoreError::StorageError)?;
        }

        Ok(pruned)
    }

    pub fn delete(&self, id: String) -> Result<(), crate::IronCoreError> {
        let db = self.db.lock().unwrap();
        db.remove(id.as_bytes())
            .map_err(|_| crate::IronCoreError::StorageError)?;
        Ok(())
    }
}

// ============================================================================
// CONNECTION LEDGER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub multiaddr: String,
    pub peer_id: Option<String>,
    pub public_key: Option<String>,
    pub nickname: Option<String>,
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
                public_key: None,
                nickname: None,
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

    pub fn annotate_identity(
        &self,
        multiaddr: String,
        peer_id: String,
        public_key: Option<String>,
        nickname: Option<String>,
    ) {
        let normalized_public_key = public_key.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        let normalized_nickname = nickname.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        let mut entries = self.entries.lock().unwrap();
        if let Some(entry) = entries.iter_mut().find(|e| e.multiaddr == multiaddr) {
            entry.peer_id = Some(peer_id);
            if normalized_public_key.is_some() {
                entry.public_key = normalized_public_key;
            }
            if normalized_nickname.is_some() {
                entry.nickname = normalized_nickname;
            }
            entry.last_seen = Some(current_timestamp());
            return;
        }

        entries.push(LedgerEntry {
            multiaddr,
            peer_id: Some(peer_id),
            public_key: normalized_public_key,
            nickname: normalized_nickname,
            success_count: 0,
            failure_count: 0,
            last_seen: Some(current_timestamp()),
            topics: Vec::new(),
        });
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
    #[cfg(not(target_arch = "wasm32"))]
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create global Tokio runtime");

    #[cfg(target_arch = "wasm32")]
    let rt = tokio::runtime::Builder::new_current_thread()
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
    ///
    /// `recipient_identity_id` and `intended_device_id` are WS13 tight-pair metadata.
    /// Pass `None` for both if the caller has no device record for the recipient.
    pub fn send_message(
        &self,
        peer_id: String,
        data: Vec<u8>,
        recipient_identity_id: Option<String>,
        intended_device_id: Option<String>,
    ) -> Result<(), crate::IronCoreError> {
        let handle_guard = self.handle.lock();
        let handle = handle_guard
            .as_ref()
            .ok_or(crate::IronCoreError::NetworkError)?;

        // Parse peer ID
        let peer_id = PeerId::from_str(&peer_id).map_err(|_| crate::IronCoreError::InvalidInput)?;

        // Block on async operation
        let rt = self.get_runtime_handle();
        rt.block_on(handle.send_message(peer_id, data, recipient_identity_id, intended_device_id))
            .map_err(|_| crate::IronCoreError::NetworkError)
    }

    /// Send an encrypted message envelope and return the raw swarm error string
    /// on failure so adapters can classify retryable vs terminal rejection.
    pub fn send_message_status(
        &self,
        peer_id: String,
        data: Vec<u8>,
        recipient_identity_id: Option<String>,
        intended_device_id: Option<String>,
    ) -> Option<String> {
        let handle_guard = self.handle.lock();
        let handle = match handle_guard.as_ref() {
            Some(handle) => handle,
            None => return Some("swarm_bridge_unavailable".to_string()),
        };

        let peer_id = match PeerId::from_str(&peer_id) {
            Ok(peer_id) => peer_id,
            Err(_) => return Some("invalid_peer_id".to_string()),
        };

        let rt = self.get_runtime_handle();
        rt.block_on(handle.send_message(peer_id, data, recipient_identity_id, intended_device_id))
            .err()
            .map(|err| err.to_string())
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
            tracing::warn!("send_to_all_peers: no connected peers");
            return Err(crate::IronCoreError::NetworkError);
        }

        let mut sent = 0usize;
        for peer_id in peers {
            match rt.block_on(handle.send_message(peer_id, data.clone(), None, None)) {
                Ok(()) => sent += 1,
                Err(e) => {
                    tracing::warn!("send_to_all_peers: failed to send to {}: {:?}", peer_id, e)
                }
            }
        }

        if sent == 0 {
            tracing::warn!("send_to_all_peers: failed to deliver to every connected peer");
            return Err(crate::IronCoreError::NetworkError);
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
    pub fn publish_topic(&self, topic: String, data: Vec<u8>) -> Result<(), crate::IronCoreError> {
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
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
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
    fn test_fresh_install_without_identity_resolves_headless_mode_with_persisted_key() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        let service = MeshService::with_storage(
            MeshServiceConfig {
                discovery_interval_ms: 5_000,
                battery_floor_pct: 20,
            },
            path.clone(),
        );
        service.start().unwrap();

        let (first_keypair, first_headless) = service.resolve_swarm_keypair_and_mode().unwrap();
        assert!(
            first_headless,
            "fresh install should default to headless mode"
        );

        let key_path = std::path::Path::new(&path).join("relay_network_key.pb");
        assert!(
            key_path.exists(),
            "headless key should persist on first resolve"
        );
        service.stop();

        let reloaded = MeshService::with_storage(
            MeshServiceConfig {
                discovery_interval_ms: 5_000,
                battery_floor_pct: 20,
            },
            path,
        );
        reloaded.start().unwrap();
        let (second_keypair, second_headless) = reloaded.resolve_swarm_keypair_and_mode().unwrap();
        assert!(second_headless);
        assert_eq!(
            first_keypair.public().to_peer_id(),
            second_keypair.public().to_peer_id(),
            "headless key should be stable across restarts"
        );
    }

    #[test]
    fn test_identity_creation_upgrades_resolved_mode_from_headless_to_full() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        let service = MeshService::with_storage(
            MeshServiceConfig {
                discovery_interval_ms: 5_000,
                battery_floor_pct: 20,
            },
            path,
        );
        service.start().unwrap();

        let (_, headless_before) = service.resolve_swarm_keypair_and_mode().unwrap();
        assert!(headless_before);

        let core = service
            .get_core()
            .expect("core should be available after start");
        core.grant_consent();
        core.initialize_identity().unwrap();

        let (full_keypair, headless_after) = service.resolve_swarm_keypair_and_mode().unwrap();
        assert!(
            !headless_after,
            "identity initialization should upgrade to full mode"
        );

        let identity_keypair = core.get_libp2p_keypair().unwrap();
        assert_eq!(
            full_keypair.public().to_peer_id(),
            identity_keypair.public().to_peer_id(),
            "full mode should use identity-derived libp2p keypair"
        );
    }

    #[test]
    fn test_connection_path_state_parity_independent_of_role_mode() {
        let service = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 5_000,
            battery_floor_pct: 20,
        });
        service.set_bootstrap_nodes(vec!["/dns4/bootstrap.example/tcp/443/wss".to_string()]);

        *service.swarm_headless_mode.lock() = Some(true);
        let headless_state = service.get_connection_path_state();

        *service.swarm_headless_mode.lock() = Some(false);
        let full_state = service.get_connection_path_state();

        assert_eq!(
            headless_state, full_state,
            "connection-path semantics should not differ by role mode"
        );
        assert_eq!(headless_state, ConnectionPathState::Bootstrapping);
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

    #[test]
    fn test_connection_path_state_bootstrapping_without_peers() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 5_000,
            battery_floor_pct: 20,
        });
        svc.set_bootstrap_nodes(vec!["/dns4/bootstrap.example/tcp/443/wss".to_string()]);
        assert_eq!(
            svc.get_connection_path_state(),
            ConnectionPathState::Bootstrapping
        );
    }

    #[test]
    fn test_export_diagnostics_contains_state_fields() {
        let svc = MeshService::new(MeshServiceConfig {
            discovery_interval_ms: 5_000,
            battery_floor_pct: 20,
        });
        let json = svc.export_diagnostics();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(v.get("service_state").is_some());
        assert!(v.get("connection_path_state").is_some());
        assert!(v.get("nat_status").is_some());
        assert!(v.get("timestamp_ms").is_some());
    }

    #[test]
    fn test_history_manager_persists_across_restart() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();

        {
            let history = HistoryManager::new(path.clone()).unwrap();
            history
                .add(MessageRecord {
                    id: "msg-persist-1".to_string(),
                    direction: MessageDirection::Sent,
                    peer_id: "peer-one".to_string(),
                    content: "hello".to_string(),
                    timestamp: 1_777_000_000,
                    sender_timestamp: 1_777_000_000,
                    delivered: false,
                    hidden: false,
                })
                .unwrap();
            history.mark_delivered("msg-persist-1".to_string()).unwrap();
            assert_eq!(history.count(), 1);
        }

        let reloaded = HistoryManager::new(path).unwrap();
        let record = reloaded
            .get("msg-persist-1".to_string())
            .unwrap()
            .expect("message record should persist");
        assert_eq!(record.peer_id, "peer-one");
        assert!(record.delivered);
    }

    #[test]
    fn test_history_manager_recent_sorts_by_timestamp_not_key_order() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let history = HistoryManager::new(path).unwrap();

        history
            .add(MessageRecord {
                id: "z_old".to_string(),
                direction: MessageDirection::Sent,
                peer_id: "peer-a".to_string(),
                content: "old".to_string(),
                timestamp: 100,
                sender_timestamp: 100,
                delivered: false,
                hidden: false,
            })
            .unwrap();
        history
            .add(MessageRecord {
                id: "a_new".to_string(),
                direction: MessageDirection::Sent,
                peer_id: "peer-a".to_string(),
                content: "new".to_string(),
                timestamp: 200,
                sender_timestamp: 200,
                delivered: false,
                hidden: false,
            })
            .unwrap();
        history
            .add(MessageRecord {
                id: "m_other".to_string(),
                direction: MessageDirection::Received,
                peer_id: "peer-b".to_string(),
                content: "other".to_string(),
                timestamp: 300,
                sender_timestamp: 300,
                delivered: true,
                hidden: false,
            })
            .unwrap();

        let latest_any = history.recent(None, 1).unwrap();
        assert_eq!(latest_any.len(), 1);
        assert_eq!(latest_any[0].id, "m_other");

        let peer_a = history.recent(Some("peer-a".to_string()), 2).unwrap();
        assert_eq!(peer_a.len(), 2);
        assert_eq!(peer_a[0].id, "a_new");
        assert_eq!(peer_a[1].id, "z_old");
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
        std::thread::sleep(web_time::Duration::from_millis(10));
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
