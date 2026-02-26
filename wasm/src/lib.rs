// scmessenger-wasm — WebAssembly bindings for browser environments

pub mod connection_state;
pub mod transport;

use libp2p::{Multiaddr, PeerId};
use parking_lot::Mutex;
use scmessenger_core::{
    DiscoveryMode, IdentityInfo, IronCore as RustIronCore, MeshSettings, MeshSettingsManager,
    SignatureResult,
};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_logging() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}

#[wasm_bindgen]
pub struct IronCore {
    inner: Arc<RustIronCore>,
    /// Buffer of successfully decoded messages waiting to be drained by JS.
    rx_messages: Arc<Mutex<Vec<WasmMessage>>>,
    /// Active libp2p swarm handle for browser networking.
    swarm_handle: Arc<Mutex<Option<scmessenger_core::transport::SwarmHandle>>>,
    /// Settings manager for persistence (uses localStorage path or in-memory).
    settings_manager: Option<MeshSettingsManager>,
    /// Cached in-memory settings for the current session.
    settings: Arc<Mutex<MeshSettings>>,
}

#[wasm_bindgen]
impl IronCore {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        init_logging();
        // Web defaults: always plugged in, internet-only transport
        let defaults = MeshSettings {
            battery_floor: 0,           // Web = always plugged in
            ble_enabled: false,         // No BLE in browser
            wifi_aware_enabled: false,  // No WiFi Aware in browser
            wifi_direct_enabled: false, // No WiFi Direct in browser
            internet_enabled: true,
            ..MeshSettings::default()
        };
        Self {
            inner: Arc::new(RustIronCore::new()),
            rx_messages: Arc::new(Mutex::new(Vec::new())),
            swarm_handle: Arc::new(Mutex::new(None)),
            settings_manager: None,
            settings: Arc::new(Mutex::new(defaults)),
        }
    }

    #[wasm_bindgen(js_name = withStorage)]
    pub fn with_storage(storage_path: String) -> Self {
        init_logging();
        let manager = MeshSettingsManager::new(storage_path.clone());
        let loaded = manager.load().unwrap_or_else(|_| MeshSettings {
            battery_floor: 0,
            ble_enabled: false,
            wifi_aware_enabled: false,
            wifi_direct_enabled: false,
            internet_enabled: true,
            ..MeshSettings::default()
        });
        Self {
            inner: Arc::new(RustIronCore::with_storage(storage_path)),
            rx_messages: Arc::new(Mutex::new(Vec::new())),
            swarm_handle: Arc::new(Mutex::new(None)),
            settings_manager: Some(manager),
            settings: Arc::new(Mutex::new(loaded)),
        }
    }

    pub fn start(&self) -> Result<(), JsValue> {
        self.inner
            .start()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    pub fn stop(&self) {
        self.inner.stop();
    }

    #[wasm_bindgen(js_name = isRunning)]
    pub fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    #[wasm_bindgen(js_name = initializeIdentity)]
    pub fn initialize_identity(&self) -> Result<(), JsValue> {
        self.inner
            .initialize_identity()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = getIdentityInfo)]
    pub fn get_identity_info(&self) -> JsValue {
        let info = self.inner.get_identity_info();
        serde_wasm_bindgen::to_value(&WasmIdentityInfo::from(info)).unwrap()
    }

    #[wasm_bindgen(js_name = signData)]
    pub fn sign_data(&self, data: Vec<u8>) -> Result<JsValue, JsValue> {
        self.inner
            .sign_data(data)
            .map(|sig| serde_wasm_bindgen::to_value(&WasmSignatureResult::from(sig)).unwrap())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = verifySignature)]
    pub fn verify_signature(
        &self,
        data: Vec<u8>,
        signature: Vec<u8>,
        public_key_hex: String,
    ) -> Result<bool, JsValue> {
        self.inner
            .verify_signature(data, signature, public_key_hex)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = prepareMessage)]
    pub fn prepare_message(
        &self,
        recipient_public_key_hex: String,
        text: String,
    ) -> Result<Vec<u8>, JsValue> {
        // Relay enforcement: when OFF, block all outbound messaging (parity with Android/iOS)
        if !self.settings.lock().relay_enabled {
            return Err(JsValue::from_str(
                "Messaging blocked: mesh participation is disabled (relay toggle OFF)",
            ));
        }
        self.inner
            .prepare_message(recipient_public_key_hex, text)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = receiveMessage)]
    pub fn receive_message(&self, envelope_bytes: Vec<u8>) -> Result<JsValue, JsValue> {
        // Relay enforcement: when OFF, silently drop inbound messages (parity with Android/iOS)
        if !self.settings.lock().relay_enabled {
            return Err(JsValue::from_str(
                "Message dropped: mesh participation is disabled (relay toggle OFF)",
            ));
        }
        self.inner
            .receive_message(envelope_bytes)
            .map(|msg| {
                serde_wasm_bindgen::to_value(&WasmMessage {
                    id: msg.id.clone(),
                    sender_id: msg.sender_id.clone(),
                    text: msg.text_content(),
                    timestamp: msg.timestamp,
                })
                .unwrap()
            })
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = outboxCount)]
    pub fn outbox_count(&self) -> u32 {
        self.inner.outbox_count()
    }

    #[wasm_bindgen(js_name = inboxCount)]
    pub fn inbox_count(&self) -> u32 {
        self.inner.inbox_count()
    }

    /// Start libp2p swarm networking for the browser client.
    ///
    /// `bootstrapAddrs` must be a JS array of libp2p multiaddr strings.
    #[wasm_bindgen(js_name = startSwarm)]
    pub async fn start_swarm(&self, bootstrap_addrs: JsValue) -> Result<(), JsValue> {
        let bootstrap_addrs = parse_bootstrap_addrs(bootstrap_addrs)?;
        start_swarm_runtime(
            Arc::clone(&self.inner),
            Arc::clone(&self.rx_messages),
            Arc::clone(&self.settings),
            Arc::clone(&self.swarm_handle),
            bootstrap_addrs,
        )
        .await
    }

    /// Stop libp2p swarm networking for the browser client.
    #[wasm_bindgen(js_name = stopSwarm)]
    pub async fn stop_swarm(&self) -> Result<(), JsValue> {
        let maybe_handle = self.swarm_handle.lock().take();
        if let Some(handle) = maybe_handle {
            handle
                .shutdown()
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to stop swarm: {}", e)))?;
        }
        Ok(())
    }

    /// Send a prepared encrypted envelope to a connected libp2p peer.
    #[wasm_bindgen(js_name = sendPreparedEnvelope)]
    pub async fn send_prepared_envelope(
        &self,
        peer_id: String,
        envelope_bytes: Vec<u8>,
    ) -> Result<(), JsValue> {
        if !self.settings.lock().relay_enabled {
            return Err(JsValue::from_str(
                "Messaging blocked: mesh participation is disabled (relay toggle OFF)",
            ));
        }

        let peer_id: PeerId = peer_id
            .parse()
            .map_err(|e| JsValue::from_str(&format!("Invalid peer ID: {}", e)))?;

        let handle = self
            .swarm_handle
            .lock()
            .clone()
            .ok_or_else(|| JsValue::from_str("Swarm is not running"))?;

        handle
            .send_message(peer_id, envelope_bytes)
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to send envelope: {}", e)))
    }

    /// Get currently connected peer IDs.
    #[wasm_bindgen(js_name = getPeers)]
    pub async fn get_peers(&self) -> Result<JsValue, JsValue> {
        let handle = self
            .swarm_handle
            .lock()
            .clone()
            .ok_or_else(|| JsValue::from_str("Swarm is not running"))?;

        let peers = handle
            .get_peers()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to get peers: {}", e)))?;

        let peer_strings: Vec<String> = peers.into_iter().map(|p| p.to_string()).collect();
        serde_wasm_bindgen::to_value(&peer_strings)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize peers: {}", e)))
    }

    #[wasm_bindgen(js_name = getConnectionPathState)]
    pub async fn get_connection_path_state(&self) -> Result<String, JsValue> {
        let maybe_handle = self.swarm_handle.lock().clone();
        let Some(handle) = maybe_handle else {
            return Ok("Disconnected".to_string());
        };

        let peers = handle
            .get_peers()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to get peers: {}", e)))?;
        if peers.is_empty() {
            return Ok("Bootstrapping".to_string());
        }

        let listeners = handle
            .get_listeners()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to get listeners: {}", e)))?;
        if listeners.is_empty() {
            Ok("RelayFallback".to_string())
        } else {
            Ok("DirectPreferred".to_string())
        }
    }

    #[wasm_bindgen(js_name = exportDiagnostics)]
    pub async fn export_diagnostics(&self) -> Result<String, JsValue> {
        // Clone the handle out of the lock before any await so the MutexGuard
        // is not held across the suspension point.
        let handle_opt = self.swarm_handle.lock().clone();
        let peers = if let Some(handle) = handle_opt {
            handle
                .get_peers()
                .await
                .map(|p| p.into_iter().map(|id| id.to_string()).collect::<Vec<_>>())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let connection_path = self.get_connection_path_state().await?;
        let payload = serde_json::json!({
            "running": self.is_running(),
            "connection_path_state": connection_path,
            "peers": peers,
            "inbox_count": self.inbox_count(),
            "outbox_count": self.outbox_count(),
            "relay_enabled": self.settings.lock().relay_enabled,
            "timestamp_ms": js_sys::Date::now(),
        });

        Ok(payload.to_string())
    }

    /// DEPRECATED shim for pre-0.1.2 clients.
    ///
    /// This maps a relay URL to a libp2p websocket multiaddr and starts the
    /// wasm swarm path.  Unlike the original fire-and-forget API, this version
    /// is `async` and propagates start errors back to the JS caller so callers
    /// can detect and handle failures rather than silently missing them.
    ///
    /// Prefer `startSwarm(bootstrapAddrs)` for new code.
    #[wasm_bindgen(js_name = startReceiveLoop)]
    pub async fn start_receive_loop(&self, relay_url: String) -> Result<(), JsValue> {
        tracing::warn!(
            "startReceiveLoop(relayUrl) is deprecated; use startSwarm(bootstrapAddrs) instead"
        );
        let relay_multiaddr = relay_url_to_multiaddr(&relay_url)
            .map_err(|e| JsValue::from_str(&format!("Invalid relay URL: {}", e)))?;

        start_swarm_runtime(
            Arc::clone(&self.inner),
            Arc::clone(&self.rx_messages),
            Arc::clone(&self.settings),
            Arc::clone(&self.swarm_handle),
            vec![relay_multiaddr],
        )
        .await
    }

    /// Drain and return all messages that have arrived since the last call.
    ///
    /// Returns a `js_sys::Array` of plain JS objects with the same shape as
    /// the object returned by `receiveMessage`:
    /// `{ id, senderId, text, timestamp }`.
    ///
    /// The internal buffer is cleared on each call; messages are not duplicated
    /// across successive calls.
    #[wasm_bindgen(js_name = drainReceivedMessages)]
    pub fn drain_received_messages(&self) -> js_sys::Array {
        let mut buf = self.rx_messages.lock();
        let drained: Vec<WasmMessage> = buf.drain(..).collect();
        drop(buf);

        let array = js_sys::Array::new();
        for msg in drained {
            if let Ok(js_val) = serde_wasm_bindgen::to_value(&msg) {
                array.push(&js_val);
            }
        }
        array
    }

    // ── Settings Management ──────────────────────────────────────────────

    /// Return the current MeshSettings as a JS object.
    #[wasm_bindgen(js_name = getSettings)]
    pub fn get_settings(&self) -> JsValue {
        let s = self.settings.lock();
        serde_wasm_bindgen::to_value(&WasmMeshSettings::from(s.clone())).unwrap()
    }

    /// Apply a partial or full settings update from JS.
    /// Accepts a JS object matching the WasmMeshSettings shape.
    #[wasm_bindgen(js_name = updateSettings)]
    pub fn update_settings(&self, js_settings: JsValue) -> Result<(), JsValue> {
        let wasm_settings: WasmMeshSettings = serde_wasm_bindgen::from_value(js_settings)
            .map_err(|e| JsValue::from_str(&format!("Invalid settings: {}", e)))?;
        let settings: MeshSettings = wasm_settings.into();

        // Persist if we have a storage manager
        if let Some(ref mgr) = self.settings_manager {
            mgr.save(settings.clone())
                .map_err(|e| JsValue::from_str(&format!("Failed to save settings: {:?}", e)))?;
        }

        *self.settings.lock() = settings;
        Ok(())
    }

    /// Return the default settings for the Web platform.
    #[wasm_bindgen(js_name = getDefaultSettings)]
    pub fn get_default_settings(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&WasmMeshSettings::from(MeshSettings {
            battery_floor: 0,
            ble_enabled: false,
            wifi_aware_enabled: false,
            wifi_direct_enabled: false,
            internet_enabled: true,
            ..MeshSettings::default()
        }))
        .unwrap()
    }

    // ── Identity Management ──────────────────────────────────────────────

    /// Set the nickname for the local identity.
    #[wasm_bindgen(js_name = setNickname)]
    pub fn set_nickname(&self, nickname: String) -> Result<(), JsValue> {
        self.inner
            .set_nickname(nickname)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Export the local identity as a backup string (for import on another device).
    #[wasm_bindgen(js_name = exportIdentityBackup)]
    pub fn export_identity_backup(&self) -> Result<String, JsValue> {
        self.inner
            .export_identity_backup()
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Import an identity from a backup string produced by `exportIdentityBackup`.
    #[wasm_bindgen(js_name = importIdentityBackup)]
    pub fn import_identity_backup(&self, backup: String) -> Result<(), JsValue> {
        self.inner
            .import_identity_backup(backup)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Derive the Ed25519 public key hex from a libp2p PeerId string.
    #[wasm_bindgen(js_name = extractPublicKeyFromPeerId)]
    pub fn extract_public_key_from_peer_id(&self, peer_id: String) -> Result<String, JsValue> {
        self.inner
            .extract_public_key_from_peer_id(peer_id)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    // ── Messaging (extended) ─────────────────────────────────────────────

    /// Prepare an encrypted message envelope and return both the message ID
    /// and the raw envelope bytes. Use the ID to track delivery receipts.
    #[wasm_bindgen(js_name = prepareMessageWithId)]
    pub fn prepare_message_with_id(
        &self,
        recipient_public_key_hex: String,
        text: String,
    ) -> Result<JsValue, JsValue> {
        if !self.settings.lock().relay_enabled {
            return Err(JsValue::from_str(
                "Messaging blocked: mesh participation is disabled (relay toggle OFF)",
            ));
        }
        self.inner
            .prepare_message_with_id(recipient_public_key_hex, text)
            .map(|p| {
                serde_wasm_bindgen::to_value(&WasmPreparedMessage {
                    message_id: p.message_id,
                    envelope_data: p.envelope_data,
                })
                .unwrap()
            })
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Prepare a delivery receipt envelope to send back to the original sender.
    /// Call this after successfully decoding a received message.
    #[wasm_bindgen(js_name = prepareReceipt)]
    pub fn prepare_receipt(
        &self,
        recipient_public_key_hex: String,
        message_id: String,
    ) -> Result<Vec<u8>, JsValue> {
        self.inner
            .prepare_receipt(recipient_public_key_hex, message_id)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Generate a cover traffic payload — random bytes that look like an
    /// encrypted message. Broadcast via `sendPreparedEnvelope` or the
    /// swarm's send-to-all to obscure real traffic patterns.
    /// `sizeBytes` is clamped to [16, 1024].
    #[wasm_bindgen(js_name = prepareCoverTraffic)]
    pub fn prepare_cover_traffic(&self, size_bytes: u32) -> Result<Vec<u8>, JsValue> {
        self.inner
            .prepare_cover_traffic(size_bytes)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    /// Remove a message from the Rust outbox after it has been delivered.
    /// Returns `true` if the message was found and removed, `false` if not found.
    #[wasm_bindgen(js_name = markMessageSent)]
    pub fn mark_message_sent(&self, message_id: String) -> bool {
        self.inner.mark_message_sent(message_id)
    }
}

fn parse_bootstrap_addrs(value: JsValue) -> Result<Vec<String>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }

    serde_wasm_bindgen::from_value(value)
        .map_err(|e| JsValue::from_str(&format!("bootstrapAddrs must be string[]: {}", e)))
}

fn relay_url_to_multiaddr(relay_url: &str) -> Result<String, String> {
    let (is_secure, rest) = if let Some(rest) = relay_url.strip_prefix("wss://") {
        (true, rest)
    } else if let Some(rest) = relay_url.strip_prefix("ws://") {
        (false, rest)
    } else {
        return Err("URL must start with ws:// or wss://".to_string());
    };

    let authority = rest
        .split('/')
        .next()
        .ok_or_else(|| "Missing relay host".to_string())?;
    if authority.is_empty() {
        return Err("Missing relay host".to_string());
    }

    let (host, port) = if let Some((host, port_str)) = authority.rsplit_once(':') {
        if host.contains(':') && !host.starts_with('[') {
            (authority.to_string(), if is_secure { 443 } else { 80 })
        } else {
            let parsed_port = port_str
                .parse::<u16>()
                .map_err(|_| format!("Invalid relay port: {}", port_str))?;
            (
                host.trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string(),
                parsed_port,
            )
        }
    } else {
        (
            authority
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string(),
            if is_secure { 443 } else { 80 },
        )
    };

    let host_segment = if host.parse::<std::net::Ipv4Addr>().is_ok() {
        format!("/ip4/{}", host)
    } else if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("/ip6/{}", host)
    } else {
        format!("/dns4/{}", host)
    };

    let ws_segment = if is_secure { "wss" } else { "ws" };
    Ok(format!("{}/tcp/{}/{}", host_segment, port, ws_segment))
}

async fn start_swarm_runtime(
    inner: Arc<RustIronCore>,
    rx_messages: Arc<Mutex<Vec<WasmMessage>>>,
    settings: Arc<Mutex<MeshSettings>>,
    swarm_handle: Arc<Mutex<Option<scmessenger_core::transport::SwarmHandle>>>,
    bootstrap_addrs: Vec<String>,
) -> Result<(), JsValue> {
    if swarm_handle.lock().is_some() {
        return Err(JsValue::from_str("Swarm is already running"));
    }

    if !inner.is_running() {
        inner
            .start()
            .map_err(|e| JsValue::from_str(&format!("Failed to start core: {}", e)))?;
    }

    if inner.get_identity_keys().is_none() {
        inner
            .initialize_identity()
            .map_err(|e| JsValue::from_str(&format!("Failed to initialize identity: {}", e)))?;
    }

    let identity_keys = inner
        .get_identity_keys()
        .ok_or_else(|| JsValue::from_str("Identity keys unavailable after initialization"))?;

    let libp2p_keys = identity_keys
        .to_libp2p_keypair()
        .map_err(|e| JsValue::from_str(&format!("Failed to derive libp2p keypair: {}", e)))?;

    let bootstrap_multiaddrs: Vec<Multiaddr> = bootstrap_addrs
        .iter()
        .map(|raw| {
            raw.parse::<Multiaddr>().map_err(|e| {
                JsValue::from_str(&format!("Invalid bootstrap multiaddr '{}': {}", raw, e))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);
    let handle = scmessenger_core::transport::start_swarm_with_config(
        libp2p_keys,
        None,
        event_tx,
        None,
        bootstrap_multiaddrs,
        false, // WASM browser clients are full user nodes, not headless relays
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("Failed to start swarm: {}", e)))?;

    *swarm_handle.lock() = Some(handle);

    let swarm_handle_for_loop = Arc::clone(&swarm_handle);
    wasm_bindgen_futures::spawn_local(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                scmessenger_core::transport::SwarmEvent::MessageReceived {
                    peer_id,
                    envelope_data,
                } => {
                    if !settings.lock().relay_enabled {
                        tracing::debug!("Dropping swarm inbound frame: relay toggle OFF");
                        continue;
                    }

                    match inner.receive_message(envelope_data) {
                        Ok(msg) => {
                            rx_messages.lock().push(WasmMessage {
                                id: msg.id.clone(),
                                sender_id: msg.sender_id.clone(),
                                text: msg.text_content(),
                                timestamp: msg.timestamp,
                            });
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to decode swarm message from {}: {:?}",
                                peer_id,
                                e
                            );
                        }
                    }
                }
                scmessenger_core::transport::SwarmEvent::PeerDiscovered(peer_id) => {
                    inner.notify_peer_discovered(peer_id.to_string());
                }
                scmessenger_core::transport::SwarmEvent::PeerDisconnected(peer_id) => {
                    inner.notify_peer_disconnected(peer_id.to_string());
                }
                scmessenger_core::transport::SwarmEvent::PeerIdentified { peer_id, .. } => {
                    tracing::info!("Swarm identified peer {}", peer_id);
                }
                scmessenger_core::transport::SwarmEvent::AddressReflected { .. }
                | scmessenger_core::transport::SwarmEvent::ListeningOn(_)
                | scmessenger_core::transport::SwarmEvent::TopicDiscovered { .. }
                | scmessenger_core::transport::SwarmEvent::LedgerReceived { .. }
                | scmessenger_core::transport::SwarmEvent::NatStatusChanged(_) => {}
            }
        }

        *swarm_handle_for_loop.lock() = None;
        tracing::info!("WASM swarm event loop terminated");
    });

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmIdentityInfo {
    identity_id: Option<String>,
    public_key_hex: Option<String>,
    initialized: bool,
}

impl From<IdentityInfo> for WasmIdentityInfo {
    fn from(info: IdentityInfo) -> Self {
        Self {
            identity_id: info.identity_id,
            public_key_hex: info.public_key_hex,
            initialized: info.initialized,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmSignatureResult {
    signature: Vec<u8>,
    public_key_hex: String,
}

impl From<SignatureResult> for WasmSignatureResult {
    fn from(sig: SignatureResult) -> Self {
        Self {
            signature: sig.signature,
            public_key_hex: sig.public_key_hex,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmPreparedMessage {
    message_id: String,
    envelope_data: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmMessage {
    id: String,
    sender_id: String,
    text: Option<String>,
    timestamp: u64,
}

/// Web-facing MeshSettings with camelCase field names for JS interop.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WasmMeshSettings {
    relay_enabled: bool,
    max_relay_budget: u32,
    battery_floor: u8,
    ble_enabled: bool,
    wifi_aware_enabled: bool,
    wifi_direct_enabled: bool,
    internet_enabled: bool,
    discovery_mode: String,
    onion_routing: bool,
    cover_traffic_enabled: bool,
    message_padding_enabled: bool,
    timing_obfuscation_enabled: bool,
}

impl From<MeshSettings> for WasmMeshSettings {
    fn from(s: MeshSettings) -> Self {
        Self {
            relay_enabled: s.relay_enabled,
            max_relay_budget: s.max_relay_budget,
            battery_floor: s.battery_floor,
            ble_enabled: s.ble_enabled,
            wifi_aware_enabled: s.wifi_aware_enabled,
            wifi_direct_enabled: s.wifi_direct_enabled,
            internet_enabled: s.internet_enabled,
            discovery_mode: match s.discovery_mode {
                DiscoveryMode::Normal => "normal".to_string(),
                DiscoveryMode::Cautious => "cautious".to_string(),
                DiscoveryMode::Paranoid => "paranoid".to_string(),
            },
            onion_routing: s.onion_routing,
            cover_traffic_enabled: s.cover_traffic_enabled,
            message_padding_enabled: s.message_padding_enabled,
            timing_obfuscation_enabled: s.timing_obfuscation_enabled,
        }
    }
}

impl From<WasmMeshSettings> for MeshSettings {
    fn from(w: WasmMeshSettings) -> Self {
        Self {
            relay_enabled: w.relay_enabled,
            max_relay_budget: w.max_relay_budget,
            battery_floor: w.battery_floor,
            ble_enabled: w.ble_enabled,
            wifi_aware_enabled: w.wifi_aware_enabled,
            wifi_direct_enabled: w.wifi_direct_enabled,
            internet_enabled: w.internet_enabled,
            discovery_mode: match w.discovery_mode.as_str() {
                "cautious" => DiscoveryMode::Cautious,
                "paranoid" => DiscoveryMode::Paranoid,
                _ => DiscoveryMode::Normal,
            },
            onion_routing: w.onion_routing,
            cover_traffic_enabled: w.cover_traffic_enabled,
            message_padding_enabled: w.message_padding_enabled,
            timing_obfuscation_enabled: w.timing_obfuscation_enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_core_creation() {
        let core = IronCore::new();
        core.start().unwrap();
        assert!(core.is_running());
        core.stop();
        assert!(!core.is_running());
    }

    #[wasm_bindgen_test]
    fn test_wasm_identity() {
        let core = IronCore::new();
        core.initialize_identity().unwrap();
        let info = core.get_identity_info();
        assert!(!info.is_null());
    }

    #[test]
    fn test_relay_url_to_multiaddr_ws_defaults() {
        let addr = relay_url_to_multiaddr("ws://relay.example.com").unwrap();
        assert_eq!(addr, "/dns4/relay.example.com/tcp/80/ws");
    }

    #[test]
    fn test_relay_url_to_multiaddr_wss_defaults() {
        let addr = relay_url_to_multiaddr("wss://relay.example.com").unwrap();
        assert_eq!(addr, "/dns4/relay.example.com/tcp/443/wss");
    }

    #[test]
    fn test_relay_url_to_multiaddr_ipv4_port() {
        let addr = relay_url_to_multiaddr("wss://1.2.3.4:7443/mesh").unwrap();
        assert_eq!(addr, "/ip4/1.2.3.4/tcp/7443/wss");
    }

    #[test]
    fn test_relay_url_to_multiaddr_rejects_http() {
        let err = relay_url_to_multiaddr("https://relay.example.com").unwrap_err();
        assert!(err.contains("ws:// or wss://"));
    }
}
