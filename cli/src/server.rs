use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, mpsc};
use warp::Filter;

// ============================================================================
// UI EVENT / COMMAND TYPES (unchanged)
// ============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiEvent {
    PeerDiscovered {
        peer_id: String,
        transport: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        public_key: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        identity: Option<String>,
    },
    ContactList {
        contacts: Vec<crate::contacts::Contact>,
    },
    MessageReceived {
        from: String,
        content: String,
        timestamp: u64,
        #[serde(default)]
        message_id: String,
    },
    MessageStatus {
        message_id: String,
        status: String,
    },
    NetworkStatus {
        status: String,
        peer_count: usize,
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
    ConfigValue {
        key: String,
        value: Option<String>,
    },
    ConfigData {
        config: Vec<(String, String)>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum UiCommand {
    IdentityShow,
    IdentityExport,
    ContactList,
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
    ConfigSet {
        key: String,
        value: String,
    },
    ConfigGet {
        key: String,
    },
    ConfigList,
    ConfigBootstrapAdd {
        multiaddr: String,
    },
    ConfigBootstrapRemove {
        multiaddr: String,
    },
    FactoryReset,
    Restart,
}

// ============================================================================
// WEB CONTEXT — shared state for HTTP endpoints
// ============================================================================

/// Public context passed to the web server for landing page + API endpoints.
pub struct WebContext {
    pub node_peer_id: String,
    pub node_public_key: String,
    pub bootstrap_nodes: Vec<String>,
    pub ledger: Arc<tokio::sync::Mutex<crate::ledger::ConnectionLedger>>,
    pub peers: Arc<tokio::sync::Mutex<HashMap<libp2p::PeerId, Option<String>>>>,
    pub start_time: Instant,
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

#[derive(Serialize)]
struct NetworkInfoResponse {
    node: NodeInfoPayload,
    network: NetworkStatsPayload,
    ledger: Vec<LedgerEntryPayload>,
}

#[derive(Serialize)]
struct NodeInfoPayload {
    peer_id: String,
    public_key: String,
    version: String,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct NetworkStatsPayload {
    connected_peers: usize,
    known_peers: usize,
    bootstrap_nodes: Vec<String>,
    topics: Vec<String>,
}

#[derive(Serialize)]
struct LedgerEntryPayload {
    address: String,
    multiaddr: String,
    last_peer_id: Option<String>,
    last_seen: u64,
    is_bootstrap: bool,
    known_topics: Vec<String>,
    label: Option<String>,
}

#[derive(Serialize)]
struct JoinBundleResponse {
    scmessenger_join_bundle: bool,
    version: u32,
    created_at: u64,
    created_by_peer_id: String,
    bootstrap_nodes: Vec<String>,
    known_peers: Vec<JoinBundlePeer>,
    topics: Vec<String>,
}

#[derive(Serialize)]
struct JoinBundlePeer {
    multiaddr: String,
    last_peer_id: Option<String>,
    last_seen: u64,
}

// ============================================================================
// SERVER START
// ============================================================================

/// The landing page HTML, compiled into the binary.
const LANDING_HTML: &str = include_str!("landing.html");

pub async fn start(
    port: u16,
    web_ctx: Arc<WebContext>,
) -> (broadcast::Sender<UiEvent>, mpsc::Receiver<UiCommand>) {
    let (broadcast_tx, _br_rx) = broadcast::channel::<UiEvent>(100);
    let (cmd_tx, cmd_rx) = mpsc::channel::<UiCommand>(100);

    // --- Warp filters for shared state ---

    let broadcast_tx_filter = warp::any().map({
        let tx = broadcast_tx.clone();
        move || tx.clone()
    });

    let cmd_tx_filter = warp::any().map({
        let tx = cmd_tx.clone();
        move || tx.clone()
    });

    let ctx_filter = warp::any().map({
        let ctx = web_ctx.clone();
        move || ctx.clone()
    });

    // --- Routes ---

    // 1. Landing page at /
    let landing_html = LANDING_HTML.to_string();
    let landing_route = warp::path::end()
        .and(warp::get())
        .map(move || {
            warp::http::Response::builder()
                .header("content-type", "text/html; charset=utf-8")
                .body(landing_html.clone())
                .unwrap()
        })
        .boxed();

    // 2. WebSocket at /ws
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(broadcast_tx_filter)
        .and(cmd_tx_filter)
        .map(|ws: warp::ws::Ws, br_tx, c_tx| {
            ws.on_upgrade(move |socket| handle_connection(socket, br_tx, c_tx))
        })
        .boxed();

    // 3. Network info API
    let network_info_route = warp::path!("api" / "network-info")
        .and(warp::get())
        .and(ctx_filter.clone())
        .and_then(handle_network_info)
        .boxed();

    // 4. Join bundle API
    let join_bundle_route = warp::path!("api" / "join-bundle")
        .and(warp::get())
        .and(ctx_filter)
        .and_then(handle_join_bundle)
        .boxed();

    // Combine all routes with CORS
    let cors = warp::cors().allow_any_origin();
    let routes = landing_route
        .or(ws_route)
        .or(network_info_route)
        .or(join_bundle_route)
        .with(cors)
        .boxed();

    println!("Starting WebSocket + HTTP server on 0.0.0.0:{}", port);

    tokio::spawn(async move {
        warp::serve(routes).run(([0, 0, 0, 0], port)).await;
    });

    (broadcast_tx, cmd_rx)
}

// ============================================================================
// API HANDLERS
// ============================================================================

async fn handle_network_info(ctx: Arc<WebContext>) -> Result<impl warp::Reply, warp::Rejection> {
    let ledger_guard = ctx.ledger.lock().await;
    let peers_guard = ctx.peers.lock().await;

    let uptime = ctx.start_time.elapsed().as_secs();
    let topics = ledger_guard.all_known_topics();

    let ledger_entries: Vec<LedgerEntryPayload> = ledger_guard
        .entries
        .values()
        .map(|e| LedgerEntryPayload {
            address: e.address.clone(),
            multiaddr: e.multiaddr.clone(),
            last_peer_id: e.last_peer_id.clone(),
            last_seen: e.last_seen,
            is_bootstrap: e.is_bootstrap,
            known_topics: e.known_topics.clone(),
            label: e.label.clone(),
        })
        .collect();

    let response = NetworkInfoResponse {
        node: NodeInfoPayload {
            peer_id: ctx.node_peer_id.clone(),
            public_key: ctx.node_public_key.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime,
        },
        network: NetworkStatsPayload {
            connected_peers: peers_guard.len(),
            known_peers: ledger_guard.entries.len(),
            bootstrap_nodes: ctx.bootstrap_nodes.clone(),
            topics,
        },
        ledger: ledger_entries,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_join_bundle(ctx: Arc<WebContext>) -> Result<impl warp::Reply, warp::Rejection> {
    let ledger_guard = ctx.ledger.lock().await;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let topics = ledger_guard.all_known_topics();

    let known_peers: Vec<JoinBundlePeer> = ledger_guard
        .entries
        .values()
        .map(|e| JoinBundlePeer {
            multiaddr: e.multiaddr.clone(),
            last_peer_id: e.last_peer_id.clone(),
            last_seen: e.last_seen,
        })
        .collect();

    let bundle = JoinBundleResponse {
        scmessenger_join_bundle: true,
        version: 1,
        created_at: now,
        created_by_peer_id: ctx.node_peer_id.clone(),
        bootstrap_nodes: ctx.bootstrap_nodes.clone(),
        known_peers,
        topics,
    };

    let json_reply = warp::reply::json(&bundle);
    Ok(warp::reply::with_header(
        json_reply,
        "Content-Disposition",
        "attachment; filename=\"scm-join-bundle.json\"",
    ))
}

// ============================================================================
// WEBSOCKET HANDLER (unchanged)
// ============================================================================

async fn handle_connection(
    ws: warp::ws::WebSocket,
    broadcast_tx: broadcast::Sender<UiEvent>,
    cmd_tx: mpsc::Sender<UiCommand>,
) {
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let mut broadcast_rx = broadcast_tx.subscribe();

    // Task to forward broadcast events -> WebSocket
    let forward_task = tokio::spawn(async move {
        while let Ok(event) = broadcast_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                if user_ws_tx
                    .send(warp::ws::Message::text(json))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    // Handle WebSocket -> Command Channel
    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    match serde_json::from_str::<UiCommand>(text) {
                        Ok(cmd) => {
                            if cmd_tx.send(cmd).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            // Don't optimize out
                            let _ = e;
                        }
                    }
                } else if msg.is_close() {
                    break;
                }
            }
            Err(_e) => {
                break;
            }
        }
    }

    forward_task.abort();
}
