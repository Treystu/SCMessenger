// scmessenger-wasm — WebAssembly bindings for browser environments

pub mod connection_state;
pub mod transport;

use crate::transport::WebSocketRelay;
use futures::StreamExt;
use parking_lot::Mutex;
use scmessenger_core::{IdentityInfo, IronCore as RustIronCore, SignatureResult};
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
}

#[wasm_bindgen]
impl IronCore {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        init_logging();
        Self {
            inner: Arc::new(RustIronCore::new()),
            rx_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[wasm_bindgen(js_name = withStorage)]
    pub fn with_storage(storage_path: String) -> Self {
        init_logging();
        Self {
            inner: Arc::new(RustIronCore::with_storage(storage_path)),
            rx_messages: Arc::new(Mutex::new(Vec::new())),
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
        self.inner
            .prepare_message(recipient_public_key_hex, text)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = receiveMessage)]
    pub fn receive_message(&self, envelope_bytes: Vec<u8>) -> Result<JsValue, JsValue> {
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

        wasm_bindgen_futures::spawn_local(async move {
            // `relay` is moved here so the WebSocket handle (and therefore the
            // registered JS callbacks) stay alive for the lifetime of the loop.
            let _relay_keep_alive = relay;

            while let Some(bytes) = rx.next().await {
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
