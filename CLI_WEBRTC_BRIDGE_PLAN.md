# CLI WebRTC-to-libp2p Bridge Plan

## Executive Summary

**Problem:** Browser (WASM) clients cannot communicate with native (iOS/Android/CLI) clients because:
- Browsers only support WebSocket/WebRTC transports
- Native nodes only support libp2p TCP/QUIC transports
- No bridge exists between these two transport domains

**Solution:** Upgrade CLI nodes to act as dual-stack bridge relays that accept WebSocket connections from browsers and forward to native peers via libp2p QUIC circuits.

**Architecture:**
```
Browser ←WebSocket→ CLI Bridge ←QUIC→ Native Peers
         (RelayMessage)         (libp2p protocol)
```

---

## Current State Analysis

### Native Swarm Transport (`core/src/transport/swarm.rs:1286-1303`)
```rust
let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
    .with_tokio()
    .with_tcp(libp2p::tcp::Config::default(), libp2p::noise::Config::new, libp2p::yamux::Config::default)?
    .with_quic()
    .with_relay_client(libp2p::noise::Config::new, libp2p::yamux::Config::default)?
    // NO WebSocket support
```

### WASM Transport (`core/src/wasm_support/transport.rs:102-104`)
```rust
pub fn add_relay(&self, url: String, peer_id: [u8; 32]) -> Result<(), TransportError> {
    if !url.starts_with("ws://") && !url.starts_with("wss://") {
        return Err(TransportError::InvalidUrl(url));
    }
```

### CLI Dependencies (`cli/Cargo.toml`)
```toml
warp = { version = "0.4.2", features = ["websocket", "server", "multipart"] }
```
CLI already has WebSocket capability via `warp`.

---

## Implementation Plan

### Phase 1: Add WebSocket Transport to Native Swarm (~150 LOC)

**File:** `core/Cargo.toml`
**Change:** Add `websocket` feature to libp2p dependencies

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
libp2p = { workspace = true, features = [
  "tcp",
  "quic",
  "websocket",  # ADD THIS
  "dns",
  "mdns",
  "tokio",
] }
```

**File:** `core/src/transport/swarm.rs`
**Change:** Add WebSocket listener alongside TCP/QUIC

```rust
// After QUIC listener setup (line ~1346)
// Add WebSocket listener for browser bridge
if let Ok(ws_addr) = "/ip4/0.0.0.0/tcp/8001/ws".parse::<Multiaddr>() {
    match swarm.listen_on(ws_addr.clone()) {
        Ok(_) => tracing::info!("✓ Bound WebSocket listener {}", ws_addr),
        Err(e) => tracing::warn!("✗ Failed to bind WebSocket listener {}: {}", ws_addr, e),
    }
}
```

### Phase 2: WebSocket-to-RelayMessage Translation Layer (~100 LOC)

**File:** `core/src/relay/websocket_bridge.rs` (NEW)

```rust
//! WebSocket bridge for browser clients
//!
//! Translates WebSocket frames to/from RelayMessage protocol

use crate::relay::protocol::RelayMessage;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Handle a new WebSocket connection from a browser client
pub async fn handle_websocket_connection(
    socket: WebSocket,
    relay_server: Arc<RelayServer>,
    event_tx: mpsc::Sender<BridgeEvent>,
) {
    let (mut ws_sink, mut ws_stream) = socket.split();
    
    // Phase 1: Handshake
    if let Some(Ok(msg)) = ws_stream.next().await {
        if let Ok(relay_msg) = RelayMessage::from_bytes(&msg.into_bytes()) {
            match relay_msg {
                RelayMessage::Handshake { peer_id, capabilities } => {
                    // Register browser peer
                    relay_server.register_peer(&peer_id, capabilities);
                    
                    // Send handshake ack
                    let ack = relay_server.create_handshake_ack(peer_id.clone());
                    let _ = ws_sink.send(Message::binary(ack.to_bytes())).await;
                    
                    // Bridge loop
                    bridge_loop(ws_sink, ws_stream, peer_id, relay_server, event_tx).await;
                }
                _ => tracing::warn!("Expected Handshake message first"),
            }
        }
    }
}

/// Main bridge loop - translates WebSocket ↔ RelayMessage
async fn bridge_loop(
    mut ws_sink: SplitSink<WebSocket, Message>,
    mut ws_stream: SplitStream<WebSocket>,
    peer_id: String,
    relay_server: Arc<RelayServer>,
    event_tx: mpsc::Sender<BridgeEvent>,
) {
    while let Some(Ok(msg)) = ws_stream.next().await {
        if let Ok(relay_msg) = RelayMessage::from_bytes(&msg.into_bytes()) {
            match relay_msg {
                RelayMessage::StoreRequest { recipient, envelope } => {
                    // Forward to relay server
                    relay_server.store_for_peer(&recipient, envelope);
                }
                RelayMessage::PullRequest { peer_id: req_peer } => {
                    // Retrieve stored messages
                    let messages = relay_server.get_stored_for(&req_peer);
                    for envelope in messages {
                        let response = RelayMessage::PullResponse {
                            peer_id: req_peer.clone(),
                            envelope,
                        };
                        let _ = ws_sink.send(Message::binary(response.to_bytes())).await;
                    }
                }
                RelayMessage::ForwardRequest { recipient, envelope } => {
                    // Forward to native peer via QUIC circuit
                    let _ = event_tx.send(BridgeEvent::ForwardToNative {
                        recipient,
                        envelope,
                    }).await;
                }
                _ => tracing::debug!("Unhandled relay message from browser: {:?}", relay_msg.message_type()),
            }
        }
    }
    
    // Cleanup on disconnect
    relay_server.remove_peer(&peer_id);
}

/// Events from WebSocket bridge to native swarm
pub enum BridgeEvent {
    ForwardToNative {
        recipient: String,
        envelope: Vec<u8>,
    },
    BrowserConnected {
        peer_id: String,
    },
    BrowserDisconnected {
        peer_id: String,
    },
}
```

### Phase 3: Integrate Bridge into CLI Server (~50 LOC)

**File:** `cli/src/server.rs`
**Change:** Spawn WebSocket bridge alongside libp2p swarm

```rust
pub async fn run_bridge_server(
    config: BridgeConfig,
    swarm_handle: SwarmHandle,
) -> Result<()> {
    // Start WebSocket server for browser clients
    let ws_addr = format!("0.0.0.0:{}", config.websocket_port);
    let relay_server = Arc::new(RelayServer::with_config(config.relay_config.clone()));
    let (bridge_tx, mut bridge_rx) = mpsc::channel(1024);
    
    // Spawn WebSocket listener
    let ws_relay = relay_server.clone();
    tokio::spawn(async move {
        let ws_route = warp::path("relay")
            .and(warp::ws())
            .map(move |ws: Ws| {
                let relay = ws_relay.clone();
                let tx = bridge_tx.clone();
                ws.on_upgrade(|socket| async move {
                    handle_websocket_connection(socket, relay, tx).await;
                })
            });
        
        warp::serve(ws_route).run(([0, 0, 0, 0], config.websocket_port)).await;
    });
    
    // Handle bridge events - forward to native swarm
    while let Some(event) = bridge_rx.recv().await {
        match event {
            BridgeEvent::ForwardToNative { recipient, envelope } => {
                // Convert recipient to PeerId and send via QUIC circuit
                if let Ok(peer_id) = PeerId::from_str(&recipient) {
                    let _ = swarm_handle.send_message(peer_id, envelope).await;
                }
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

---

## Implementation Checklist

### Core Changes
- [ ] Add `websocket` feature to `core/Cargo.toml` libp2p dependencies
- [ ] Add WebSocket listener in `core/src/transport/swarm.rs`
- [ ] Create `core/src/relay/websocket_bridge.rs` module
- [ ] Export bridge module in `core/src/relay/mod.rs`

### CLI Changes
- [ ] Add bridge configuration to `cli/src/config.rs`
- [ ] Add `--bridge-mode` flag to CLI
- [ ] Spawn WebSocket server in `cli/src/server.rs`
- [ ] Handle bridge events in main server loop

### Testing
- [ ] Unit tests for WebSocket-to-RelayMessage translation
- [ ] Integration test: Browser → CLI Bridge → Native peer
- [ ] Load test: Multiple concurrent browser connections
- [ ] Verify message delivery across transport domains

### Documentation
- [ ] Update `GCP_DEPLOY_GUIDE.md` with bridge mode
- [ ] Add WebSocket endpoint documentation
- [ ] Update `TRANSPORT_PATHS_AUDIT_2026-03-16.md` with bridge solution

---

## Deployment

### GCP Relay Nodes
Update deployment to enable bridge mode:
```bash
./scm server --bridge-mode --websocket-port 8001 --quic-port 4433
```

### Firewall Rules
Open additional port for WebSocket:
```bash
gcloud compute firewall-rules create allow-websocket \
    --allow tcp:8001 \
    --target-tags scm-relay
```

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| WebSocket adds latency | Bridge is on same server, minimal hop |
| Browser connections consume resources | Rate limiting per IP, max connections config |
| Protocol translation bugs | Extensive unit tests, canary deployment |
| Security: untrusted browser clients | Same auth as native (identity verification) |

---

## Success Criteria

1. Browser client can send message to native peer via CLI bridge
2. Native peer receives message and can reply
3. Reply reaches browser client via same bridge
4. Latency overhead < 50ms for bridge translation
5. Bridge handles 100+ concurrent browser connections

---

## Implementation Scope

| Component | Files | LOC Estimate | Complexity |
|-----------|-------|--------------|------------|
| **Core: WebSocket Feature** | `core/Cargo.toml` | ~10 LOC | Low |
| **Core: Swarm WebSocket Listener** | `core/src/transport/swarm.rs` | ~20 LOC | Low |
| **Core: Bridge Module** | `core/src/relay/websocket_bridge.rs` (NEW) | ~100 LOC | Medium |
| **Core: Module Export** | `core/src/relay/mod.rs` | ~5 LOC | Low |
| **CLI: Bridge Config** | `cli/src/config.rs` | ~15 LOC | Low |
| **CLI: Server Integration** | `cli/src/server.rs` | ~50 LOC | Medium |
| **CLI: Main Entry Point** | `cli/src/main.rs` | ~20 LOC | Low |
| **Testing: Bridge Unit Tests** | `core/src/relay/websocket_bridge.rs` | ~80 LOC | Medium |
| **Testing: Integration Tests** | `tests/bridge_integration.rs` (NEW) | ~100 LOC | Medium |
| **Documentation** | `GCP_DEPLOY_GUIDE.md`, `TRANSPORT_PATHS_AUDIT_2026-03-16.md` | ~50 LOC | Low |
| **TOTAL** | **10 files** | **~450 LOC** | **Medium** |

### LOC Breakdown by Phase

**Phase 1: Core WebSocket Support (~35 LOC)**
- `core/Cargo.toml`: Add `websocket` feature to libp2p (~10 LOC)
- `core/src/transport/swarm.rs`: Add WebSocket listener binding (~20 LOC)
- `core/src/relay/mod.rs`: Export bridge module (~5 LOC)

**Phase 2: Bridge Module (~100 LOC)**
- `core/src/relay/websocket_bridge.rs`: WebSocket-to-RelayMessage translation (~100 LOC)

**Phase 3: CLI Integration (~85 LOC)**
- `cli/src/config.rs`: Bridge configuration struct (~15 LOC)
- `cli/src/server.rs`: Spawn WebSocket server, handle bridge events (~50 LOC)
- `cli/src/main.rs`: Add `--bridge-mode` flag (~20 LOC)

**Phase 4: Testing (~180 LOC)**
- Bridge unit tests in `websocket_bridge.rs` (~80 LOC)
- Integration tests: Browser → CLI Bridge → Native peer (~100 LOC)

**Phase 5: Documentation (~50 LOC)**
- Update deployment guides (~30 LOC)
- Update transport audit with solution (~20 LOC)

---

*Created: 2026-03-16*
*Status: Plan Ready for Implementation*
