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
        nickname: Option<String>,
        libp2p_peer_id: Option<String>,
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
    /// Optional IronCore reference for WASM JSON-RPC bridge handlers
    /// (contacts, settings, history, blocking). None when core is not
    /// available (e.g. bootstrap-only CLI modes).
    pub core: Option<Arc<scmessenger_core::IronCore>>,
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
            core: self.core.clone(),
        }
    }
}

impl std::fmt::Debug for WebContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebContext").finish_non_exhaustive()
    }
}

// JSON-RPC types imported from core (shared wire contract with WASM client).
use scmessenger_core::wasm_support::rpc::{
    parse_intent, rpc_error, rpc_result, ClientIntent, JsonRpcErrorBody, JsonRpcNotification,
    JsonRpcRequest, JsonRpcResponse, ERR_PARAMS,
};

/// WebSocket message sender handle connected clients.
type WsSender = futures::channel::mpsc::UnboundedSender<warp::ws::Message>;
type WsSenderList = Arc<Mutex<Vec<WsSender>>>;

use warp::Filter;

/// Start the warp HTTP + WebSocket server on `127.0.0.1:<port>`.
///
/// Returns a broadcast sender for pushing server events to connected clients
/// and a command receiver for inbound UI commands from the WebSocket bridge.
pub async fn start(
    port: u16,
    ctx: Arc<WebContext>,
) -> anyhow::Result<(
    broadcast::Sender<UiOutbound>,
    mpsc::UnboundedReceiver<UiCommand>,
)> {
    let (ui_tx, _) = broadcast::channel::<UiOutbound>(256);
    let (ui_cmd_tx, ui_cmd_rx) = mpsc::unbounded_channel::<UiCommand>();

    // Shared list of WebSocket senders for broadcast push notifications.
    let ws_senders: WsSenderList = Arc::new(Mutex::new(Vec::new()));

    // Serve a minimal landing page at GET /
    let ctx_landing = ctx.clone();
    let landing = warp::path::end().map(move || {
        let body = format!(
            "<!DOCTYPE html><html><head><title>SCMessenger</title></head>\
             <body><h1>SCMessenger</h1><p>Node: {}</p><p>Public Key: {}</p>\
             <p>Uptime: {:?}</p></body></html>",
            ctx_landing.node_peer_id,
            ctx_landing.node_public_key,
            ctx_landing.start_time.elapsed(),
        );
        warp::reply::html(body)
    });

    // WebSocket route at GET /ws — upgrades to JSON-RPC bridge
    let ws_senders_filter = ws_senders.clone();
    let ctx_ws = ctx.clone();
    let ui_tx_ws = ui_tx.clone();
    let ui_cmd_tx_ws = ui_cmd_tx.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let ctx = ctx_ws.clone();
            let ui_tx = ui_tx_ws.clone();
            let ui_cmd_tx = ui_cmd_tx_ws.clone();
            let senders = ws_senders_filter.clone();
            ws.on_upgrade(move |websocket| {
                handle_ws_connection(websocket, ctx, ui_tx, ui_cmd_tx, senders)
            })
        });

    let routes = landing.or(ws_route);

    // Bind and serve on 127.0.0.1 only (local bridge, never exposed to network).
    let bound_addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();

    // Spawn the warp server (bind panics on port conflict, but cmd_start
    // already validates availability).
    tokio::spawn(async move { warp::serve(routes).bind(bound_addr).await });

    tracing::info!("Warp HTTP+WS server listening on ws://{}", bound_addr);

    Ok((ui_tx, ui_cmd_rx))
}

/// Handle a single upgraded WebSocket connection.
///
/// Reads JSON-RPC text frames, dispatches them to `handle_jsonrpc_request`,
/// and sends the serialised response back to the client. Also subscribes to
/// the UI broadcast and forwards server-push notifications.
async fn handle_ws_connection(
    websocket: warp::ws::WebSocket,
    ctx: Arc<WebContext>,
    ui_tx: broadcast::Sender<UiOutbound>,
    ui_cmd_tx: mpsc::UnboundedSender<UiCommand>,
    senders: WsSenderList,
) {
    use futures::StreamExt;
    use futures_util::SinkExt;

    let (mut ws_tx, mut ws_rx) = websocket.split();

    // Channel to forward broadcast notifications to this client.
    let (client_tx, mut client_rx) = futures::channel::mpsc::unbounded::<warp::ws::Message>();

    // Register this client's sender for broadcast push.
    senders.lock().await.push(client_tx.clone());

    // Subscribe to UI broadcast for push notifications.
    let mut ui_rx = ui_tx.subscribe();

    // Spawn task that reads from the broadcast and forwards to this client.
    let senders_task = senders.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                res = ui_rx.recv() => {
                    match res {
                        Ok(outbound) => {
                            let text = match &outbound {
                                UiOutbound::JsonRpc(val) => serde_json::to_string(val).unwrap_or_default(),
                                UiOutbound::Legacy(evt) => serde_json::to_string(evt).unwrap_or_default(),
                            };
                            if client_tx.unbounded_send(warp::ws::Message::text(text)).is_err() {
                                break; // client disconnected
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("WS client lagged {} broadcast messages", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                _ = client_rx.next() => {
                    // Placeholder for client->broadcast relay if needed.
                }
            }
        }
        // Unregister on disconnect.
        senders_task.lock().await.retain(|s| !s.is_closed());
    });

    // Read loop: process incoming JSON-RPC text frames from the client.
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("WebSocket receive error: {}", e);
                break;
            }
        };

        let text = if msg.is_text() {
            msg.to_str().unwrap_or("").to_string()
        } else if msg.is_close() {
            break;
        } else {
            continue;
        };

        let response = handle_jsonrpc_request(&text, &ctx, &ui_cmd_tx).await;
        if let Err(e) = ws_tx
            .send(warp::ws::Message::text(
                serde_json::to_string(&response).unwrap_or_default(),
            ))
            .await
        {
            tracing::warn!("WebSocket send error: {}", e);
            break;
        }
    }

    // Cleanup: remove this client's sender.
    senders.lock().await.retain(|s| !s.is_closed());
    tracing::info!("WebSocket client disconnected");
}

/// Dispatch a single JSON-RPC request through the core intent parser and
/// return a JSON-RPC response.
async fn handle_jsonrpc_request(
    raw: &str,
    ctx: &WebContext,
    _ui_cmd_tx: &mpsc::UnboundedSender<UiCommand>,
) -> JsonRpcResponse {
    let req: JsonRpcRequest = match serde_json::from_str(raw) {
        Ok(r) => r,
        Err(e) => {
            return rpc_error(
                None,
                JsonRpcErrorBody {
                    code: -32700,
                    message: format!("Parse error: {}", e),
                    data: None,
                },
            )
        }
    };

    let id = req.id.clone();
    let intent = match parse_intent(&req) {
        Ok(i) => i,
        Err(e) => return rpc_error(id, e),
    };

    match intent {
        ClientIntent::GetIdentity {} => {
            let peer_count = ctx.peers.lock().await.len();
            let mut payload = serde_json::json!({
                "identityId": ctx.node_peer_id,
                "publicKeyHex": ctx.node_public_key,
                "peerCount": peer_count,
                "bootstrapNodes": ctx.bootstrap_nodes,
                "uptimeSecs": ctx.start_time.elapsed().as_secs(),
            });
            // Enrich with core identity fields when available
            if let Some(ref core) = ctx.core {
                let info = core.get_identity_info();
                if let Some(obj) = payload.as_object_mut() {
                    obj.insert("nickname".to_string(), serde_json::json!(info.nickname));
                    obj.insert(
                        "libp2pPeerId".to_string(),
                        serde_json::json!(info.libp2p_peer_id),
                    );
                    obj.insert(
                        "initialized".to_string(),
                        serde_json::json!(info.initialized),
                    );
                    if let Some(local_peer_id) = info.libp2p_peer_id {
                        obj.insert("localPeerId".to_string(), serde_json::json!(local_peer_id));
                    }
                }
            }
            rpc_result(id, payload)
        }
        ClientIntent::SendMessage {
            recipient,
            message,
            id: msg_id,
        } => {
            // Forward as UiCommand so the main loop picks it up.
            let cmd = UiCommand::Send {
                recipient,
                message,
                id: msg_id,
            };
            if let Err(e) = _ui_cmd_tx.send(cmd) {
                return rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: format!("Command channel closed: {}", e),
                        data: None,
                    },
                );
            }
            rpc_result(id, serde_json::json!({"status": "queued"}))
        }
        ClientIntent::ScanPeers {} => {
            let peers_lock = ctx.peers.lock().await;
            let peer_list: Vec<serde_json::Value> = peers_lock
                .iter()
                .map(|(pid, key)| {
                    serde_json::json!({
                        "peerId": pid.to_string(),
                        "publicKey": key,
                    })
                })
                .collect();
            rpc_result(id, serde_json::json!({"peers": peer_list}))
        }
        ClientIntent::GetTopology {} => {
            let peers_lock = ctx.peers.lock().await;
            let peer_count = peers_lock.len();
            let ledger = ctx.ledger.lock().await;
            let known_peers = ledger.all_known_topics();
            rpc_result(
                id,
                serde_json::json!({
                    "peerCount": peer_count,
                    "knownPeers": known_peers.len(),
                    "bootstrapNodes": ctx.bootstrap_nodes,
                }),
            )
        }
        // ── Contacts ──
        ClientIntent::GetContacts {} => {
            if let Some(ref core) = ctx.core {
                let mgr = core.contacts_manager();
                match mgr.list() {
                    Ok(contacts) => {
                        let list: Vec<serde_json::Value> = contacts
                            .into_iter()
                            .map(|c| {
                                serde_json::json!({
                                    "peerId": c.peer_id,
                                    "nickname": c.nickname,
                                    "localNickname": c.local_nickname,
                                })
                            })
                            .collect();
                        rpc_result(id, serde_json::json!({"contacts": list}))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to fetch contacts: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::AddContact { peer_id, nickname } => {
            if let Some(ref core) = ctx.core {
                let mgr = core.contacts_manager();
                let mut contact =
                    scmessenger_core::contacts_bridge::Contact::new(peer_id.clone(), String::new());
                if let Some(n) = nickname {
                    contact = contact.with_nickname(n);
                }
                match mgr.add(contact) {
                    Ok(()) => rpc_result(id, serde_json::json!({"added": true})),
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to add contact: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::RemoveContact { peer_id } => {
            if let Some(ref core) = ctx.core {
                let mgr = core.contacts_manager();
                match mgr.remove(peer_id) {
                    Ok(()) => rpc_result(id, serde_json::json!({"removed": true})),
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to remove contact: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Settings ──
        ClientIntent::GetSettings {} => {
            if let Some(ref core) = ctx.core {
                let info = core.get_identity_info();
                rpc_result(
                    id,
                    serde_json::json!({
                        "nickname": info.nickname,
                        "identityId": info.identity_id,
                        "publicKeyHex": info.public_key_hex,
                        "libp2pPeerId": info.libp2p_peer_id,
                        "initialized": info.initialized,
                    }),
                )
            } else {
                rpc_result(
                    id,
                    serde_json::json!({
                        "identityId": ctx.node_peer_id,
                        "publicKeyHex": ctx.node_public_key,
                    }),
                )
            }
        }
        ClientIntent::UpdateSettings { key, value } => {
            if let Some(ref core) = ctx.core {
                match key.as_str() {
                    "nickname" => {
                        if let Some(nick) = value.as_str() {
                            match core.set_nickname(nick.to_string()) {
                                Ok(()) => rpc_result(id, serde_json::json!({"updated": true})),
                                Err(e) => rpc_error(
                                    id,
                                    JsonRpcErrorBody {
                                        code: -32000,
                                        message: format!("Failed to set nickname: {:?}", e),
                                        data: None,
                                    },
                                ),
                            }
                        } else {
                            rpc_error(
                                id,
                                JsonRpcErrorBody {
                                    code: ERR_PARAMS,
                                    message: "value must be a string for nickname".to_string(),
                                    data: None,
                                },
                            )
                        }
                    }
                    _ => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: ERR_PARAMS,
                            message: format!("Unknown setting key: {}", key),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── History ──
        ClientIntent::GetHistory { limit } => {
            if let Some(ref core) = ctx.core {
                let mgr = core.history_manager();
                match mgr.recent(None, limit.unwrap_or(50) as u32) {
                    Ok(messages) => {
                        let list: Vec<serde_json::Value> = messages
                            .into_iter()
                            .map(|m| {
                                serde_json::json!({
                                    "id": m.id,
                                    "senderId": m.peer_id,
                                    "content": m.content,
                                    "timestamp": m.timestamp,
                                    "direction": format!("{:?}", m.direction),
                                })
                            })
                            .collect();
                        rpc_result(id, serde_json::json!({"messages": list}))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to fetch history: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::GetConversation { peer_id, limit } => {
            if let Some(ref core) = ctx.core {
                let mgr = core.history_manager();
                match mgr.conversation(peer_id.clone(), limit.unwrap_or(50) as u32) {
                    Ok(messages) => {
                        let list: Vec<serde_json::Value> = messages
                            .into_iter()
                            .map(|m| {
                                serde_json::json!({
                                    "id": m.id,
                                    "senderId": m.peer_id,
                                    "content": m.content,
                                    "timestamp": m.timestamp,
                                })
                            })
                            .collect();
                        rpc_result(id, serde_json::json!({"messages": list}))
                    }
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to fetch conversation: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Blocking ──
        ClientIntent::BlockPeer { peer_id, reason } => {
            if let Some(ref core) = ctx.core {
                match core.block_peer(peer_id, reason, None) {
                    Ok(()) => rpc_result(id, serde_json::json!({"blocked": true})),
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to block peer: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::UnblockPeer { peer_id } => {
            if let Some(ref core) = ctx.core {
                match core.unblock_peer(peer_id, None) {
                    Ok(()) => rpc_result(id, serde_json::json!({"unblocked": true})),
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Failed to unblock peer: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Onion routing ──
        ClientIntent::PrepareOnionMessage {
            envelope_data,
            relay_public_keys_json,
        } => {
            if let Some(ref core) = ctx.core {
                match hex::decode(&envelope_data) {
                    Ok(data) => match core.prepare_onion_message(data, relay_public_keys_json) {
                        Ok(onion_bytes) => {
                            rpc_result(id, serde_json::json!({"onionData": hex::encode(onion_bytes)}))
                        }
                        Err(e) => rpc_error(
                            id,
                            JsonRpcErrorBody {
                                code: -32000,
                                message: format!("Onion prepare failed: {:?}", e),
                                data: None,
                            },
                        ),
                    },
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: ERR_PARAMS,
                            message: format!("Invalid hex for envelope_data: {}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::PeelOnionLayer {
            onion_data,
            relay_secret_key,
        } => {
            if let Some(ref core) = ctx.core {
                let onion_bytes = match hex::decode(&onion_data) {
                    Ok(b) => b,
                    Err(e) => {
                        return rpc_error(
                            id,
                            JsonRpcErrorBody {
                                code: ERR_PARAMS,
                                message: format!("Invalid hex for onion_data: {}", e),
                                data: None,
                            },
                        )
                    }
                };
                let secret_bytes = match hex::decode(&relay_secret_key) {
                    Ok(b) => b,
                    Err(e) => {
                        return rpc_error(
                            id,
                            JsonRpcErrorBody {
                                code: ERR_PARAMS,
                                message: format!("Invalid hex for relay_secret_key: {}", e),
                                data: None,
                            },
                        )
                    }
                };
                match core.peel_onion_layer(onion_bytes, secret_bytes) {
                    Ok(result) => rpc_result(
                        id,
                        serde_json::json!({
                            "nextHop": result.next_hop.map(|h| hex::encode(h)),
                            "remainingData": hex::encode(result.remaining_data),
                        }),
                    ),
                    Err(e) => rpc_error(
                        id,
                        JsonRpcErrorBody {
                            code: -32000,
                            message: format!("Onion peel failed: {:?}", e),
                            data: None,
                        },
                    ),
                }
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        // ── Ratchet session diagnostics ──
        ClientIntent::RatchetSessionCount {} => {
            if let Some(ref core) = ctx.core {
                let count = core.ratchet_session_count();
                rpc_result(id, serde_json::json!({"sessionCount": count}))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
        ClientIntent::RatchetHasSession { peer_id } => {
            if let Some(ref core) = ctx.core {
                let has = core.ratchet_has_session(peer_id);
                rpc_result(id, serde_json::json!({"hasSession": has}))
            } else {
                rpc_error(
                    id,
                    JsonRpcErrorBody {
                        code: -32000,
                        message: "Core not available".to_string(),
                        data: None,
                    },
                )
            }
        }
    }
}

// =====================================================================
