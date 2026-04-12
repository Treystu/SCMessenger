use anyhow::Context;
use colored::*;
use futures::FutureExt; // for catch_unwind
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, mpsc};
use warp::Filter; // for .red() logic (already in cargo.toml)

// ============================================================================
// UI EVENT / COMMAND TYPES (unchanged)
// ============================================================================

/// Legacy dashboard / landing WebSocket envelope (`type` + snake_case fields).
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
        contacts: Vec<scmessenger_core::store::Contact>,
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
    HistoryList {
        peer_id: String,
        messages: Vec<crate::api::HistoryMessage>,
    },
    Log {
        segments: Vec<String>,
        level_tag: String,
        msg: String,
    },
    Error {
        message: String,
    },
}

/// Multiplexed WebSocket outbound: legacy JSON or raw JSON-RPC value (no extra wrapper).
#[derive(Debug, Clone)]
pub enum UiOutbound {
    Legacy(UiEvent),
    JsonRpc(serde_json::Value),
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
    HistoryList {
        peer_id: String,
        limit: Option<usize>,
    },
    Restart,
    /// Thin-client JSON-RPC intents (see `scmessenger_core::wasm_support::rpc`).
    DaemonRpc {
        id: Option<serde_json::Value>,
        intent: scmessenger_core::wasm_support::rpc::ClientIntent,
    },
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
    pub transport_bridge: Arc<tokio::sync::Mutex<crate::transport_bridge::TransportBridge>>,
    /// TCP port the UI server listens on (127.0.0.1) — for correct URLs in API payloads.
    pub ui_port: u16,
}

impl WebContext {
    /// Get transport bridge reference (for future API activation)
    #[allow(dead_code)]
    pub fn transport_bridge(&self) -> &Arc<tokio::sync::Mutex<crate::transport_bridge::TransportBridge>> {
        &self.transport_bridge
    }
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

#[derive(Serialize)]
struct NetworkInfoResponse {
    node: NodeInfoPayload,
    network: NetworkStatsPayload,
    ledger: Vec<LedgerEntryPayload>,
    transport: Option<TransportInfoPayload>,
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
struct TransportInfoPayload {
    capabilities: Vec<String>,
    is_headless_node: bool,
    supports_forwarding: bool,
    ws_bridge_port: u16,
    api_base_url: String,
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

#[derive(Deserialize)]
struct InstallParams {
    host: Option<String>,
}

// ============================================================================
// SERVER START
// ============================================================================

/// The landing page HTML, compiled into the binary.
const LANDING_HTML: &str = include_str!("landing.html");

#[derive(Debug)]
struct WsOriginRejected;

impl warp::reject::Reject for WsOriginRejected {}

/// WebSocket upgrade: allow only same-origin browser pages served from this UI port on loopback.
fn websocket_origin_allowed(origin: Option<&str>, port: u16) -> bool {
    let Some(o) = origin else {
        return false;
    };
    let a = format!("http://127.0.0.1:{}", port);
    let b = format!("http://localhost:{}", port);
    o == a.as_str() || o == b.as_str()
}

pub async fn start(
    port: u16,
    web_ctx: Arc<WebContext>,
) -> anyhow::Result<(broadcast::Sender<UiOutbound>, mpsc::Receiver<UiCommand>)> {
    let (broadcast_tx, _br_rx) = broadcast::channel::<UiOutbound>(100);
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

    // 1. Root: prefer dist/index.html (WASM WebUI build), else bundled landing page
    let landing_html = LANDING_HTML.to_string();
    let root_route = warp::path::end()
        .and(warp::get())
        .map(move || {
            let body = std::fs::read_to_string("dist/index.html").unwrap_or_else(|_| landing_html.clone());
            warp::http::Response::builder()
                .header("content-type", "text/html; charset=utf-8")
                .body(body)
                .unwrap()
        })
        .boxed();

    // 1b. Static WASM/WebUI assets (relative URLs often use /dist/...)
    let dist_route = warp::path("dist")
        .and(warp::fs::dir("dist"))
        .boxed();

    // 2. WebSocket at /ws — strict Origin (CSWSH mitigation); see `websocket_origin_allowed`
    let bind_port = port;
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::header::optional::<String>("origin"))
        .and(broadcast_tx_filter)
        .and(cmd_tx_filter)
        .and_then(
            move |ws: warp::ws::Ws,
                  origin: Option<String>,
                  br_tx: broadcast::Sender<UiOutbound>,
                  c_tx: mpsc::Sender<UiCommand>| {
                let port = bind_port;
                async move {
                    if !websocket_origin_allowed(origin.as_deref(), port) {
                        return Err(warp::reject::custom(WsOriginRejected));
                    }
                    Ok(ws.on_upgrade(move |socket| handle_connection(socket, br_tx, c_tx)))
                }
            },
        )
        .boxed();

    // 3. Network info API
    let network_info_route = warp::path!("api" / "network-info")
        .and(warp::get())
        .and(ctx_filter.clone())
        .and_then(handle_network_info)
        .boxed();

    // 4. Join Bundle JSON API
    let join_bundle_route = warp::path!("api" / "join-bundle")
        .and(warp::get())
        .and(ctx_filter.clone()) // clone to use again
        .and_then(handle_join_bundle)
        .boxed();

    // 5. Install Script (Native Auto - merges binary download + config)
    let install_native_route = warp::path!("api" / "install")
        .and(warp::get())
        .and(ctx_filter.clone())
        .and(warp::query::<InstallParams>())
        .and_then(handle_install_native)
        .boxed();

    // 6. Install Script (Docker Auto)
    let install_docker_route = warp::path!("api" / "install" / "docker")
        .and(warp::get())
        .and(ctx_filter.clone())
        .and(warp::query::<InstallParams>())
        .and_then(handle_install_docker)
        .boxed();

    // 7. Install Script (Source Auto)
    let install_source_route = warp::path!("api" / "install" / "source")
        .and(warp::get())
        .and_then(handle_install_source)
        .boxed();

    // 8. Download Binary (Linux - served from running container)
    // Note: This path matches the Dockerfile destination
    let download_linux_route = warp::path!("api" / "download" / "scm-linux-amd64")
        .and(warp::get())
        .and(warp::fs::file("/usr/local/bin/scm"))
        .map(|reply| {
            warp::reply::with_header(
                reply,
                "Content-Disposition",
                "attachment; filename=\"scm-linux-amd64\"",
            )
        })
        .boxed();

    // 9. UI Static Assets (The Messenger App)
    let ui_route = warp::path("ui")
        .and(warp::fs::dir("ui"))
        .boxed();

    // 10. WASM Static Assets
    let wasm_route = warp::path("wasm")
        .and(warp::fs::dir("wasm"))
        .boxed();

    // 11. Transport Bridge API - Simplified inline implementation
    // Directly implement transport endpoints to avoid warp filter chaining complexity
    let ctx_capabilities = web_ctx.clone();
    let ctx_paths = web_ctx.clone();
    let ctx_register = web_ctx.clone();
    
    // Transport capabilities endpoint
    let transport_capabilities_route = warp::path!("api" / "transport" / "capabilities")
        .and(warp::get())
        .and(warp::any().map(move || ctx_capabilities.clone()))
        .and_then(
            |ctx: Arc<crate::server::WebContext>| async move {
                let bridge = ctx.transport_bridge.lock().await;
                let cli_caps = bridge.get_cli_capabilities();
                let peer_caps = bridge.get_available_peer_capabilities();
                
                let response = serde_json::json!({
                    "cli_capabilities": cli_caps,
                    "peer_capabilities": peer_caps
                });
                Ok::<_, warp::Rejection>(warp::reply::json(&response))
            }
        )
        .boxed();

    // Transport paths endpoint
    let transport_paths_route = warp::path!("api" / "transport" / "paths" / String)
        .and(warp::get())
        .and(warp::any().map(move || ctx_paths.clone()))
        .and_then(
            |peer_id: String, ctx: Arc<crate::server::WebContext>| async move {
                if let Ok(peer_id_parsed) = peer_id.parse::<libp2p::PeerId>() {
                    let bridge = ctx.transport_bridge.lock().await;
                    let paths = bridge.find_all_paths(&peer_id_parsed);
                    Ok::<_, warp::Rejection>(warp::reply::json(&paths))
                } else {
                    Err(warp::reject::custom(crate::transport_api::TransportError::InvalidPeerId))
                }
            }
        )
        .boxed();

    // Register peer endpoint
    let transport_register_route = warp::path!("api" / "transport" / "register")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || ctx_register.clone()))
        .and_then(
            |request: crate::transport_api::RegisterPeerRequest, ctx: Arc<crate::server::WebContext>| async move {
                if let Ok(peer_id) = request.peer_id.parse::<libp2p::PeerId>() {
                    let capabilities = request.capabilities.iter()
                        .filter_map(|cap| match cap.as_str() {
                            "BLE" => Some(scmessenger_core::transport::abstraction::TransportType::BLE),
                            "WiFiAware" => Some(scmessenger_core::transport::abstraction::TransportType::WiFiAware),
                            "WiFiDirect" => Some(scmessenger_core::transport::abstraction::TransportType::WiFiDirect),
                            "Internet" => Some(scmessenger_core::transport::abstraction::TransportType::Internet),
                            "Local" => Some(scmessenger_core::transport::abstraction::TransportType::Local),
                            _ => None
                        })
                        .collect::<Vec<_>>();
                    
                    let mut bridge = ctx.transport_bridge.lock().await;
                    bridge.register_peer_capabilities(peer_id, capabilities);
                    
                    Ok(warp::reply::json(&serde_json::json!({"status": "success"})))
                } else {
                    Err(warp::reject::custom(crate::transport_api::TransportError::InvalidPeerId))
                }
            }
        )
        .boxed();

    // Combine all routes with CORS (loopback UI origins only)
    // HTTP CORS: restrict to loopback UI origins (same port). WebSocket Origin is enforced separately.
    let origin_127 = format!("http://127.0.0.1:{}", port);
    let origin_localhost = format!("http://localhost:{}", port);
    let cors = warp::cors()
        .allow_origins(vec![origin_127.as_str(), origin_localhost.as_str()])
        .allow_methods(vec!["GET", "POST", "OPTIONS", "DELETE"])
        .allow_headers(vec!["content-type", "origin", "accept"]);

    let routes = root_route
        .or(dist_route)
        .or(ui_route)
        .or(wasm_route)
        .or(ws_route)
        .or(network_info_route)
        .or(join_bundle_route)
        .or(install_native_route)
        .or(install_docker_route)
        .or(install_source_route)
        .or(download_linux_route)
        .or(transport_capabilities_route)
        .or(transport_paths_route)
        .or(transport_register_route)
        .with(cors)
        .boxed();

    // Bind loopback only (daemon UI + local WASM thin client)
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    {
        tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind web server to {}", addr))?;
    }

    println!(
        "Starting WebSocket + HTTP server on http://127.0.0.1:{} (localhost only)",
        port
    );

    tokio::spawn(async move {
        // Use AssertUnwindSafe to catch panics from warp::run (e.g. "Address already in use")
        // preventing the whole runtime from crashing with a stack trace.
        // Note: We use .run() which panics on bind error.
        let server = warp::serve(routes).run(addr);
        if std::panic::AssertUnwindSafe(server)
            .catch_unwind()
            .await
            .is_err()
        {
            eprintln!(
                "{} Failed to start web server on {}: Address potentially already in use.",
                "Error:".red(),
                addr
            );
            std::process::exit(1);
        }
    });

    Ok((broadcast_tx, cmd_rx))
}

// ============================================================================
// API HANDLERS
// ============================================================================

async fn handle_network_info(ctx: Arc<WebContext>) -> Result<impl warp::Reply, warp::Rejection> {
    let ledger_guard = ctx.ledger.lock().await;
    let peers_guard = ctx.peers.lock().await;

    let uptime = ctx.start_time.elapsed().as_secs();
    let topics = ledger_guard.all_known_topics();

    // Get CLI transport capabilities for WASM
    let transport_bridge = ctx.transport_bridge.lock().await;
    let cli_capabilities = transport_bridge.get_cli_capabilities();
    let capabilities_list: Vec<String> = cli_capabilities.iter()
        .map(|cap| format!("{:?}", cap))
        .collect();

    // Filter ledger entries to only include nodes with topics (legitimate nodes)
    let filtered_ledger_entries: Vec<LedgerEntryPayload> = ledger_guard
        .entries
        .values()
        .filter(|e| !e.known_topics.is_empty()) // Only nodes with topics
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
            known_peers: filtered_ledger_entries.len(), // Only count nodes with topics
            bootstrap_nodes: ctx.bootstrap_nodes.clone(),
            topics,
        },
        ledger: filtered_ledger_entries, // Only legitimate nodes with topics
        transport: Some(TransportInfoPayload {
            capabilities: capabilities_list,
            is_headless_node: true,
            supports_forwarding: true,
            ws_bridge_port: 9002,  // WebSocket bridge port
            api_base_url: format!("http://127.0.0.1:{}", ctx.ui_port),
        }),
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

async fn handle_install_native(
    ctx: Arc<WebContext>,
    params: InstallParams,
) -> Result<impl warp::Reply, warp::Rejection> {
    let nodes_json = serde_json::to_string(&ctx.bootstrap_nodes).unwrap_or_else(|_| "[]".into());
    let peer_id = &ctx.node_peer_id;
    let version = env!("CARGO_PKG_VERSION");
    let host = params.host.unwrap_or_else(|| "localhost:9000".to_string());

    let script = format!(
        r#"#!/bin/bash
set -e
echo "🚀 SCMessenger Native Installer"
echo "------------------------------"
echo "Target Mesh Peer: {peer_id}"

OS="$(uname -s)"
ARCH="$(uname -m)"

# 1. Check for SCM Binary
if ! command -v scm &> /dev/null; then
    echo "⬇️  'scm' binary not found. Downloading..."

    URL=""
    if [ "$OS" = "Linux" ] && [ "$ARCH" = "x86_64" ]; then
        # Download from THIS node directly (fastest)
        URL="http://{host}/api/download/scm-linux-amd64"
    elif [ "$OS" = "Darwin" ]; then
        if [ "$ARCH" = "arm64" ]; then
            URL="https://github.com/Treystu/SCMessenger/releases/download/v{version}/scm-macos-arm64"
        else
            URL="https://github.com/Treystu/SCMessenger/releases/download/v{version}/scm-macos-amd64"
        fi
    elif [ "$OS" = "Linux" ]; then
        # Fallback for non-amd64 linux to GitHub
         URL="https://github.com/Treystu/SCMessenger/releases/download/v{version}/scm-linux-amd64"
    else
        echo "⚠️  Unsupported platform for auto-download: $OS $ARCH"
        echo "Please build from source."
        exit 1
    fi

    echo "Downloading from: $URL"
    curl -L "$URL" -o scm
    chmod +x scm

    echo "📦 Installing to /usr/local/bin (requires sudo)..."
    sudo mv scm /usr/local/bin/scm || {{ echo "❌ Failed to move to /usr/local/bin. Running locally."; export PATH=$PATH:.; }}
else
    echo "✅ 'scm' binary detected."
fi

# 2. Configure Node
echo "⚙️  Configuring node..."
if [ "$OS" = "Darwin" ]; then
    CONFIG_DIR="$HOME/Library/Application Support/scmessenger"
else
    CONFIG_DIR="$HOME/.config/scmessenger"
fi
mkdir -p "$CONFIG_DIR"
cat > "$CONFIG_DIR/config.json" <<EOF
{{
  "listen_port": 9000,
  "bootstrap_nodes": {nodes_json},
  "enable_mdns": true,
  "enable_dht": true,
  "network": {{
      "enable_relay": true,
      "enable_nat_traversal": true,
      "max_peers": 50,
      "connection_timeout": 30
  }}
}}
EOF

echo "✅ Installed & Configured!"
echo "👉 Run 'scm start' to join the mesh."
if [ ! -f /usr/local/bin/scm ]; then
    echo "⚠️  Note: 'scm' was not moved to /usr/local/bin."
    echo "    You may need to add current directory to PATH or run as ./scm"
fi
"#
    );

    Ok(warp::reply::with_header(
        script,
        "content-type",
        "text/x-shellscript; charset=utf-8",
    ))
}

async fn handle_install_docker(
    ctx: Arc<WebContext>,
    params: InstallParams,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Construct comma-separated bootstrap string for the env var
    let nodes_str = ctx.bootstrap_nodes.join(",");
    let host = params.host.unwrap_or_else(|| "localhost:9000".to_string());
    // Extract just the hostname/IP from host (remove port if present)
    let hostname = host.split(':').next().unwrap_or("localhost");

    let script = format!(
        r#"#!/bin/bash
set -e
echo "🐳 SCMessenger Docker Installer"

if ! command -v docker &> /dev/null; then
    echo "Docker not found. Installing (slight overhead for simplicity)..."
    curl -fsSL https://get.docker.com | sh
    echo "✅ Docker installed."
else
    echo "✅ Docker detected."
fi

echo "🚀 Starting Node..."
echo "Running on host: {hostname}"

# Stop existing container if running
docker rm -f scmessenger 2>/dev/null || true

# Run with bootstrap nodes injected
docker run -d \
  --restart always \
  --name scmessenger \
  -p 9000:9000 \
  -p 9001:9001 \
  -e SCMESSENGER_BOOTSTRAP_NODES="{nodes_str}" \
  testbotz/scmessenger:latest

echo "✅ Node running."
echo "👉 UI Available at: http://{hostname}:9000"
"#
    );
    Ok(warp::reply::with_header(
        script,
        "content-type",
        "text/x-shellscript; charset=utf-8",
    ))
}

async fn handle_install_source() -> Result<impl warp::Reply, warp::Rejection> {
    let script = r#"#!/bin/bash
set -e
echo "🔧 SCMessenger Source Builder"

if ! command -v cargo &> /dev/null; then
    echo "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "✅ Rust detected."
fi

echo "📥 Cloning repo..."
git clone https://github.com/Treystu/SCMessenger.git || echo "Repo already exists (pulling updates...)"
cd SCMessenger && git pull

echo "🔨 Building..."
cargo build --release --bin scmessenger-cli

echo "✅ Build complete."
echo "👉 Run: ./target/release/scmessenger-cli start"
"#;
    Ok(warp::reply::with_header(
        script,
        "content-type",
        "text/x-shellscript; charset=utf-8",
    ))
}

// ============================================================================
// WEBSOCKET HANDLER (unchanged)
// ============================================================================

async fn handle_connection(
    ws: warp::ws::WebSocket,
    broadcast_tx: broadcast::Sender<UiOutbound>,
    cmd_tx: mpsc::Sender<UiCommand>,
) {
    use scmessenger_core::wasm_support::rpc::{parse_intent, rpc_error, JsonRpcRequest};

    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let mut broadcast_rx = broadcast_tx.subscribe();

    loop {
        tokio::select! {
            recv = broadcast_rx.recv() => {
                let Ok(out) = recv else {
                    break;
                };
                let json = match &out {
                    UiOutbound::JsonRpc(v) => v.to_string(),
                    UiOutbound::Legacy(ev) => serde_json::to_string(ev).unwrap_or_default(),
                };
                if user_ws_tx.send(warp::ws::Message::text(json)).await.is_err() {
                    break;
                }
            }
            msg = user_ws_rx.next() => {
                let Some(result) = msg else { break };
                match result {
                    Ok(msg) => {
                        if let Ok(text) = msg.to_str() {
                            if let Ok(rpc_req) = serde_json::from_str::<JsonRpcRequest>(text) {
                                if rpc_req.jsonrpc == "2.0" {
                                    match parse_intent(&rpc_req) {
                                        Ok(intent) => {
                                            if cmd_tx
                                                .send(UiCommand::DaemonRpc {
                                                    id: rpc_req.id.clone(),
                                                    intent,
                                                })
                                                .await
                                                .is_err()
                                            {
                                                break;
                                            }
                                            continue;
                                        }
                                        Err(err_body) => {
                                            let resp = rpc_error(rpc_req.id.clone(), err_body);
                                            if let Ok(s) = serde_json::to_string(&resp) {
                                                if user_ws_tx
                                                    .send(warp::ws::Message::text(s))
                                                    .await
                                                    .is_err()
                                                {
                                                    break;
                                                }
                                            }
                                            continue;
                                        }
                                    }
                                }
                            }
                            if let Ok(cmd) = serde_json::from_str::<UiCommand>(text) {
                                if cmd_tx.send(cmd).await.is_err() {
                                    break;
                                }
                            }
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
