// scmessenger-wasm — WebAssembly bindings for browser environments

pub mod transport;
pub mod connection_state;

use wasm_bindgen::prelude::*;
use scmessenger_core::{IronCore as RustIronCore, IdentityInfo, SignatureResult};
use std::sync::Arc;

#[wasm_bindgen]
pub fn init_logging() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}

#[wasm_bindgen]
pub struct IronCore {
    inner: Arc<RustIronCore>,
}

#[wasm_bindgen]
impl IronCore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        init_logging();
        Self { inner: Arc::new(RustIronCore::new()) }
    }

    #[wasm_bindgen(js_name = withStorage)]
    pub fn with_storage(storage_path: String) -> Self {
        init_logging();
        Self { inner: Arc::new(RustIronCore::with_storage(storage_path)) }
    }

    pub fn start(&self) -> Result<(), JsValue> {
        self.inner.start().map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    pub fn stop(&self) { self.inner.stop(); }

    #[wasm_bindgen(js_name = isRunning)]
    pub fn is_running(&self) -> bool { self.inner.is_running() }

    #[wasm_bindgen(js_name = initializeIdentity)]
    pub fn initialize_identity(&self) -> Result<(), JsValue> {
        self.inner.initialize_identity().map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = getIdentityInfo)]
    pub fn get_identity_info(&self) -> JsValue {
        let info = self.inner.get_identity_info();
        serde_wasm_bindgen::to_value(&WasmIdentityInfo::from(info)).unwrap()
    }

    #[wasm_bindgen(js_name = signData)]
    pub fn sign_data(&self, data: Vec<u8>) -> Result<JsValue, JsValue> {
        self.inner.sign_data(data)
            .map(|sig| serde_wasm_bindgen::to_value(&WasmSignatureResult::from(sig)).unwrap())
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = verifySignature)]
    pub fn verify_signature(&self, data: Vec<u8>, signature: Vec<u8>, public_key_hex: String) -> Result<bool, JsValue> {
        self.inner.verify_signature(data, signature, public_key_hex)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = prepareMessage)]
    pub fn prepare_message(&self, recipient_public_key_hex: String, text: String) -> Result<Vec<u8>, JsValue> {
        self.inner.prepare_message(recipient_public_key_hex, text)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = receiveMessage)]
    pub fn receive_message(&self, envelope_bytes: Vec<u8>) -> Result<JsValue, JsValue> {
        self.inner.receive_message(envelope_bytes)
            .map(|msg| {
                serde_wasm_bindgen::to_value(&WasmMessage {
                    id: msg.id.clone(),
                    sender_id: msg.sender_id.clone(),
                    text: msg.text_content(),
                    timestamp: msg.timestamp,
                }).unwrap()
            })
            .map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    #[wasm_bindgen(js_name = outboxCount)]
    pub fn outbox_count(&self) -> u32 { self.inner.outbox_count() }

    #[wasm_bindgen(js_name = inboxCount)]
    pub fn inbox_count(&self) -> u32 { self.inner.inbox_count() }
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
        Self { identity_id: info.identity_id, public_key_hex: info.public_key_hex, initialized: info.initialized }
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
        Self { signature: sig.signature, public_key_hex: sig.public_key_hex }
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
