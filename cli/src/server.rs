use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc};
use warp::Filter;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiEvent {
    PeerDiscovered {
        peer_id: String,
        transport: String,
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

pub async fn start(port: u16) -> (broadcast::Sender<UiEvent>, mpsc::Receiver<UiCommand>) {
    let (broadcast_tx, _br_rx) = broadcast::channel::<UiEvent>(100);
    let (cmd_tx, cmd_rx) = mpsc::channel::<UiCommand>(100);

    let broadcast_tx_filter = warp::any().map({
        let tx = broadcast_tx.clone();
        move || tx.clone()
    });

    let cmd_tx_filter = warp::any().map({
        let tx = cmd_tx.clone();
        move || tx.clone()
    });

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(broadcast_tx_filter)
        .and(cmd_tx_filter)
        .map(|ws: warp::ws::Ws, br_tx, c_tx| {
            ws.on_upgrade(move |socket| handle_connection(socket, br_tx, c_tx))
        });

    let routes = ws_route.with(warp::cors().allow_any_origin());
    let routes = routes.boxed(); // Explicitly boxed to fix complex type inference

    println!("Starting WebSocket server on 0.0.0.0:{}", port);

    let routes_server = routes;
    tokio::spawn(async move {
        warp::serve(routes_server).run(([0, 0, 0, 0], port)).await;
    });

    (broadcast_tx, cmd_rx)
}

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
