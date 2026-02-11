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

#[derive(Deserialize)]
struct InstallParams {
    host: Option<String>,
}

// ============================================================================
// SERVER START
// ============================================================================

/// The landing page HTML, compiled into the binary.
const LANDING_HTML: &str = include_str!("landing.html");

pub async fn start(
    port: u16,
    web_ctx: Arc<WebContext>,
) -> anyhow::Result<(broadcast::Sender<UiEvent>, mpsc::Receiver<UiCommand>)> {
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

    // Combine all routes with CORS
    let cors = warp::cors().allow_any_origin();
    let routes = landing_route
        .or(ws_route)
        .or(network_info_route)
        .or(join_bundle_route)
        .or(install_native_route)
        .or(install_docker_route)
        .or(install_source_route)
        .or(download_linux_route)
        .with(cors)
        .boxed();

    // Attempt to bind explicitly to catch usage errors, but DROP it so warp can bind.
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    {
        tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("Failed to bind web server to {}", addr))?;
    }

    println!("Starting WebSocket + HTTP server on {}", addr);

    tokio::spawn(async move {
        // Use AssertUnwindSafe to catch panics from warp::run (e.g. "Address already in use")
        // preventing the whole runtime from crashing with a stack trace.
        // Note: We use .run() which panics on bind error.
        let server = warp::serve(routes).run(addr);
        if let Err(_) = std::panic::AssertUnwindSafe(server).catch_unwind().await {
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
