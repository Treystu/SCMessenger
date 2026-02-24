// scmessenger-wasm — WebAssembly bindings for browser environments

pub mod connection_state;
pub mod transport;

use crate::transport::WebSocketRelay;
use futures::StreamExt;
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
        let mut defaults = MeshSettings::default();
        defaults.battery_floor = 0; // Web = always plugged in
        defaults.ble_enabled = false; // No BLE in browser
        defaults.wifi_aware_enabled = false; // No WiFi Aware in browser
        defaults.wifi_direct_enabled = false; // No WiFi Direct in browser
        defaults.internet_enabled = true;
        Self {
            inner: Arc::new(RustIronCore::new()),
            rx_messages: Arc::new(Mutex::new(Vec::new())),
            settings_manager: None,
            settings: Arc::new(Mutex::new(defaults)),
        }
    }

    #[wasm_bindgen(js_name = withStorage)]
    pub fn with_storage(storage_path: String) -> Self {
        init_logging();
        let manager = MeshSettingsManager::new(storage_path.clone());
        let loaded = manager.load().unwrap_or_else(|_| {
            let mut defaults = MeshSettings::default();
            defaults.battery_floor = 0;
            defaults.ble_enabled = false;
            defaults.wifi_aware_enabled = false;
            defaults.wifi_direct_enabled = false;
            defaults.internet_enabled = true;
            defaults
        });
        Self {
            inner: Arc::new(RustIronCore::with_storage(storage_path)),
            rx_messages: Arc::new(Mutex::new(Vec::new())),
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

    /// Connect to a WebSocket relay and start decrypting incoming frames.
    ///
    /// Call `subscribe()` on the relay before `connect()` to avoid a race window
    /// (safe in single-threaded WASM, but the ordering is explicit and clear).
    /// Spawns a single-threaded async loop via `wasm_bindgen_futures::spawn_local`
    /// that feeds each incoming frame through `IronCore::receive_message`.
    /// Successfully decoded messages are pushed into an internal buffer that JS
    /// can drain at any time by calling `drainReceivedMessages()`.
    ///
    /// Returns an error string if the WebSocket connection cannot be initiated.
    #[wasm_bindgen(js_name = startReceiveLoop)]
    pub fn start_receive_loop(&self, relay_url: String) -> Result<(), JsValue> {
        let relay = WebSocketRelay::new(relay_url);

        // Subscribe before connecting — installs the ingress_tx so the onmessage
        // callback has a live sender the moment the socket opens.
        let mut rx = relay.subscribe();

        relay
            .connect()
            .map_err(|e| JsValue::from_str(&format!("WebSocket connect failed: {}", e)))?;

        // Clone the Arcs that the async task will capture. Both are cheap
        // reference-count bumps; no locks are held across the await point.
        let inner = Arc::clone(&self.inner);
        let rx_messages = Arc::clone(&self.rx_messages);
        let settings = Arc::clone(&self.settings);

        wasm_bindgen_futures::spawn_local(async move {
            // `relay` is moved here so the WebSocket handle (and therefore the
            // registered JS callbacks) stay alive for the lifetime of the loop.
            let _relay_keep_alive = relay;

            while let Some(bytes) = rx.next().await {
                // Relay enforcement: drop inbound frames when relay is OFF
                if !settings.lock().relay_enabled {
                    tracing::debug!("Dropping inbound frame: relay toggle OFF");
                    continue;
                }
                match inner.receive_message(bytes) {
                    Ok(msg) => {
                        let wasm_msg = WasmMessage {
                            id: msg.id.clone(),
                            sender_id: msg.sender_id.clone(),
                            text: msg.text_content(),
                            timestamp: msg.timestamp,
                        };
                        rx_messages.lock().push(wasm_msg);
                    }
                    Err(e) => {
                        tracing::warn!("receive_message failed (frame dropped): {:?}", e);
                    }
                }
            }

            tracing::info!("WebSocket ingress loop terminated");
        });

        Ok(())
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
        let mut defaults = MeshSettings::default();
        defaults.battery_floor = 0;
        defaults.ble_enabled = false;
        defaults.wifi_aware_enabled = false;
        defaults.wifi_direct_enabled = false;
        defaults.internet_enabled = true;
        serde_wasm_bindgen::to_value(&WasmMeshSettings::from(defaults)).unwrap()
    }
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
}
