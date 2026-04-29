use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, mpsc, Mutex};

// =====================================================================

// Stub types for BLE mesh UI integration (Phase 1B wiring)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiEvent {
    MessageReceived {
        from: String,
        message_id: String,
        content: String,
        timestamp: u64,
    },
    NetworkStatus {
        status: String,
        peer_count: usize,
    },
    PeerDiscovered {
        peer_id: String,
        transport: String,
        public_key: String,
        identity: String,
    },
    IdentityInfo {
        peer_id: String,
        public_key: String,
    },
    IdentityExportData {
        identity_id: String,
        public_key: String,
        private_key: String,
        storage_path: String,
    },
    ContactList {
        contacts: Vec<serde_json::Value>,
    },
    HistoryList {
        peer_id: String,
        messages: Vec<serde_json::Value>,
    },
    MessageStatus {
        message_id: String,
        status: String,
    },
    Error {
        message: String,
    },
    ConfigValue {
        key: String,
        value: String,
    },
    ConfigData {
        config: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiOutbound {
    Legacy(UiEvent),
    JsonRpc(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiCommand {
    IdentityShow,
    IdentityExport,
    ContactList,
    HistoryList {
        peer_id: String,
        limit: Option<u32>,
    },
    Status,
    Send {
        recipient: String,
        message: String,
        id: Option<String>,
    },
    ContactAdd {
        peer_id: String,
        name: Option<String>,
        public_key: Option<String>,
    },
    ContactRemove {
        contact: String,
    },
    ConfigGet {
        key: String,
    },
    ConfigList,
    ConfigSet {
        key: String,
        value: String,
    },
    ConfigBootstrapAdd {
        multiaddr: String,
    },
    ConfigBootstrapRemove {
        multiaddr: String,
    },
    FactoryReset,
    Restart,
    DaemonRpc {
        id: String,
        intent: String,
    },
}

pub struct WebContext {
    pub node_peer_id: String,
    pub node_public_key: String,
    pub bootstrap_nodes: Vec<String>,
    pub ledger: Arc<Mutex<crate::ledger::ConnectionLedger>>,
    pub peers: Arc<Mutex<HashMap<PeerId, Option<String>>>>,
    pub start_time: Instant,
    pub transport_bridge: Arc<Mutex<crate::transport_bridge::TransportBridge>>,
    pub ui_port: u16,
}

impl Clone for WebContext {
    fn clone(&self) -> Self {
        Self {
            node_peer_id: self.node_peer_id.clone(),
            node_public_key: self.node_public_key.clone(),
            bootstrap_nodes: self.bootstrap_nodes.clone(),
            ledger: Arc::clone(&self.ledger),
            peers: Arc::clone(&self.peers),
            start_time: self.start_time,
            transport_bridge: Arc::clone(&self.transport_bridge),
            ui_port: self.ui_port,
        }
    }
}

impl std::fmt::Debug for WebContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebContext").finish_non_exhaustive()
    }
}

// JSON-RPC types for daemon communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub id: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<RpcErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcErrorDetail {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientIntent {
    GetIdentity,
    ScanPeers,
    GetTopology,
    SendMessage {
        recipient: String,
        message: String,
        id: Option<String>,
    },
}

pub fn rpc_result(id: Option<serde_json::Value>, result: serde_json::Value) -> RpcResponse {
    RpcResponse {
        id,
        result: Some(result),
        error: None,
    }
}

pub fn rpc_error(id: Option<serde_json::Value>, code: i32, msg: String) -> RpcResponse {
    RpcResponse {
        id,
        result: None,
        error: Some(RpcErrorDetail { code, message: msg }),
    }
}

pub async fn start(
    _port: u16,
    _ctx: Arc<WebContext>,
) -> anyhow::Result<(
    broadcast::Sender<UiOutbound>,
    mpsc::UnboundedReceiver<UiCommand>,
)> {
    let (ui_tx, _) = broadcast::channel::<UiOutbound>(256);
    let (_, ui_cmd_rx) = mpsc::unbounded_channel::<UiCommand>();
    Ok((ui_tx, ui_cmd_rx))
}

// =====================================================================
