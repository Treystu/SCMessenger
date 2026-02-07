// sc-wasm — WebAssembly bindings for browser environments
//
// Phase 8: WASM Client Upgrade with full mesh participation,
// transport management, persistent storage, and background sync.

pub mod transport;
pub mod mesh;
pub mod storage;
pub mod worker;

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

// ============================================================================
// PHASE 8: WASM MESH NODE BINDINGS
// ============================================================================

/// WASM-bound Mesh Node for sovereign mesh participation
#[wasm_bindgen]
pub struct MeshNode {
    inner: Arc<mesh::WasmMeshNode>,
}

#[wasm_bindgen]
impl MeshNode {
    #[wasm_bindgen(constructor)]
    pub fn new(node_id: String) -> Self {
        init_logging();
        let config = mesh::MeshConfig {
            node_id,
            ..Default::default()
        };
        Self {
            inner: Arc::new(mesh::WasmMeshNode::new(config)),
        }
    }

    /// Start the mesh node
    pub fn start(&self) -> Result<(), JsValue> {
        self.inner
            .start()
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Stop the mesh node gracefully
    pub fn stop(&self) {
        self.inner.stop();
    }

    /// Get current node state as string
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&self) -> String {
        format!("{:?}", self.inner.state())
    }

    /// Send an encrypted message
    #[wasm_bindgen(js_name = sendMessage)]
    pub fn send_message(&self, recipient_hint: Option<String>, payload: Vec<u8>) -> Result<String, JsValue> {
        self.inner
            .send_message(recipient_hint, payload)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get number of connected peers
    #[wasm_bindgen(js_name = getPeerCount)]
    pub fn get_peer_count(&self) -> usize {
        self.inner.get_peer_count()
    }

    /// Synchronize with relay servers
    #[wasm_bindgen(js_name = syncWithRelay)]
    pub fn sync_with_relay(&self) -> Result<(), JsValue> {
        self.inner
            .sync_with_relay()
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get number of stored messages
    #[wasm_bindgen(js_name = getStoredMessageCount)]
    pub fn get_stored_message_count(&self) -> usize {
        self.inner.stored_message_count()
    }

    /// Export full node state (for debugging/persistence)
    #[wasm_bindgen(js_name = exportState)]
    pub fn export_state(&self) -> Result<String, JsValue> {
        self.inner
            .export_state()
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get node ID
    #[wasm_bindgen(js_name = getNodeId)]
    pub fn get_node_id(&self) -> String {
        self.inner.node_id().to_string()
    }
}

// ============================================================================
// PHASE 8: WASM SERVICE WORKER BRIDGE
// ============================================================================

/// Service worker registration and sync management
#[wasm_bindgen]
pub struct WorkerBridge {
    inner: Arc<worker::ServiceWorkerBridge>,
}

#[wasm_bindgen]
impl WorkerBridge {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        init_logging();
        Self {
            inner: Arc::new(worker::ServiceWorkerBridge::new(
                worker::BackgroundSyncConfig::default(),
            )),
        }
    }

    /// Register the service worker
    pub fn register(&self, script_url: String) -> Result<(), JsValue> {
        self.inner
            .register(&script_url)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Unregister the service worker
    pub fn unregister(&self) -> Result<(), JsValue> {
        self.inner
            .unregister()
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Register for background sync
    #[wasm_bindgen(js_name = registerSync)]
    pub fn register_sync(&self) -> Result<(), JsValue> {
        self.inner
            .register_sync()
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get current service worker status
    pub fn status(&self) -> String {
        format!("{:?}", self.inner.status())
    }
}

// ============================================================================
// PHASE 8: WASM STORAGE ACCESS
// ============================================================================

/// Storage operations for messages
#[wasm_bindgen]
pub struct Storage {
    inner: Arc<storage::WasmStorage>,
}

#[wasm_bindgen]
impl Storage {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        init_logging();
        Self {
            inner: Arc::new(storage::WasmStorage::new(storage::StorageConfig::default())),
        }
    }

    /// Export all stored messages as JSON
    #[wasm_bindgen(js_name = exportState)]
    pub fn export_state(&self) -> Result<String, JsValue> {
        self.inner
            .export_state()
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Import messages from JSON
    #[wasm_bindgen(js_name = importState)]
    pub fn import_state(&self, json: String) -> Result<(), JsValue> {
        self.inner
            .import_state(&json)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get number of stored messages
    #[wasm_bindgen(js_name = getMessageCount)]
    pub fn get_message_count(&self) -> usize {
        self.inner.message_count()
    }

    /// Get all unread messages count
    #[wasm_bindgen(js_name = getUnreadCount)]
    pub fn get_unread_count(&self) -> usize {
        self.inner.get_unread_messages().len()
    }

    /// Clear all stored messages
    pub fn clear(&self) {
        self.inner.clear();
    }
}

// ============================================================================
// TESTS
// ============================================================================

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
    fn test_mesh_node_creation() {
        let node = MeshNode::new("test-node".to_string());
        assert_eq!(node.get_node_id(), "test-node");
    }

    #[test]
    fn test_worker_bridge_creation() {
        let bridge = WorkerBridge::new();
        assert!(bridge.status().contains("NotRegistered"));
    }

    #[test]
    fn test_storage_creation() {
        let storage = Storage::new();
        assert_eq!(storage.get_message_count(), 0);
    }

    // Additional tests for Phase 8 modules are in their respective modules
}
