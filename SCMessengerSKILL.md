> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

---
name: scmessenger-wiring
description: >
  Expert context and step-by-step guidance for wiring up the SCMessenger
  (Sovereign Communication Messenger) Rust workspace at
  https://github.com/Treystu/SCMessenger. Use this skill whenever working
  on integration, transport wiring, module connectivity, build issues, CLI
  extension, mobile bindings, or any full-stack concerns in this repo.
---

# SCMessenger Wiring Skill

## [Needs Revalidation] What This Repo Is

A sovereign, server-less, E2E-encrypted mesh messenger written in Rust.
**No accounts. No servers. No phone numbers.** Works over TCP/mDNS/BLE/WiFi Direct.
Licensed Unlicense (public domain). ~53K lines of Rust across a Cargo workspace.

---

## [Needs Revalidation] Exact Repo Layout

```
scmessenger/
├── Cargo.toml                  # Workspace root — members: core, cli, mobile, (wasm excluded)
├── Cargo.lock
├── core/                       # crate: scmessenger-core  (~29K LoC, 71 source files)
│   └── src/
│       ├── lib.rs              # IronCore struct — THE public API surface
│       ├── api.udl             # UniFFI definition (mobile bridge)
│       ├── crypto/             # X25519 ECDH + XChaCha20-Poly1305, Ed25519, AAD
│       ├── identity/           # Ed25519 key gen, Blake3 identity hash, sled persistence
│       ├── message/            # Envelope format, bincode codec, size limits
│       ├── store/              # Outbox (store-and-forward), Inbox (dedup + quotas)
│       ├── transport/          # SwarmHandle, libp2p, BLE, WiFi Direct, NAT, mDNS
│       ├── drift/              # Drift Protocol: envelope/frame/compress/sketch/sync/relay
│       ├── routing/            # Mycorrhizal routing: cell/gossip/global/engine
│       ├── relay/              # Self-relay network, peer exchange, bootstrap, Find My
│       ├── privacy/            # Onion routing, circuit, cover traffic, timing obfuscation
│       ├── mobile/             # Mobile lifecycle, iOS background, auto-adjust
│       ├── platform/           # Platform-specific auto-adjust, service management
│       ├── wasm_support/       # Browser mesh participation
│       ├── contacts_bridge.rs  # Contact/ContactManager (UniFFI exported)
│       └── mobile_bridge.rs    # UniFFI mobile bridge entry
├── cli/                        # crate: scmessenger-cli (binary: scmessenger-cli)
│   └── src/
│       ├── main.rs             # Tokio event loop: SwarmEvent ↔ UiCommand ↔ stdin
│       ├── api.rs              # HTTP Control API (port 9191)
│       ├── bootstrap.rs        # Default bootstrap node list
│       ├── config.rs           # Config file (~/.config/scmessenger/)
│       ├── contacts.rs         # ContactList backed by sled
│       ├── history.rs          # MessageHistory backed by sled
│       ├── ledger.rs           # ConnectionLedger — persistent peer memory
│       └── server.rs           # WebSocket + HTTP server (landing page + UiEvent/UiCommand)
├── mobile/                     # crate: scmessenger-mobile (UniFFI Swift/Kotlin bindings)
├── wasm/                       # scmessenger-wasm (wasm-bindgen, excluded from workspace)
├── iOS/                        # Xcode project
├── ios/                        # Swift sources
├── android/                    # Kotlin/Gradle project
├── SCMessengerCore.xcframework  # Pre-built iOS framework
├── docker/                     # Docker configs
├── docker-compose.yml
├── docs/                       # Architecture docs
└── reference/                  # V1 TypeScript crypto (porting guides only)
```

---

## [Needs Revalidation] Cryptography Stack (Non-Negotiable)

| Layer | Algorithm | Notes |
|-------|-----------|-------|
| Identity signing | Ed25519 | Keys never leave the device |
| Identity hash | Blake3(`ed25519_pubkey`) | This is `identity_id` |
| Key exchange | X25519 ECDH | Ephemeral per-message |
| KDF | Blake3 `derive_key` | |
| Encryption | XChaCha20-Poly1305 | 24-byte nonce, authenticated |
| Sender auth | AAD binding + Ed25519 envelope signature | |

Never substitute algorithms. The reference types are `IdentityKeys`, `Envelope`, `Message`.

---

## [Needs Revalidation] The Two Core Types: IronCore and SwarmHandle

### [Needs Revalidation] IronCore (`core/src/lib.rs`)

The crypto/storage spine. `Arc<RwLock<_>>` internals — cheap to clone, thread-safe.

```rust
pub struct IronCore {
    identity: Arc<RwLock<IdentityManager>>,
    outbox:   Arc<RwLock<store::Outbox>>,
    inbox:    Arc<RwLock<store::Inbox>>,
    running:  Arc<RwLock<bool>>,
    delegate: Arc<RwLock<Option<Arc<dyn CoreDelegate>>>>,
}
```

**Key methods:**

```rust
// Lifecycle
IronCore::new()                          // in-memory storage
IronCore::with_storage(path: String)     // persistent sled storage
core.start() -> Result<(), IronCoreError>
core.stop()

// Identity
core.initialize_identity() -> Result<()>
core.get_identity_info() -> IdentityInfo  // { identity_id, public_key_hex, initialized, nickname }
core.get_identity_keys() -> Option<IdentityKeys>
core.get_libp2p_keypair() -> Result<libp2p::identity::Keypair>  // Same Ed25519 key → libp2p PeerId

// Messaging — THE critical path
core.prepare_message(recipient_public_key_hex: String, text: String) -> Result<Vec<u8>>
//   ^ validates recipient key → X25519 ECDH → XChaCha20-Poly1305 encrypt → bincode envelope → Vec<u8>

core.receive_message(envelope_bytes: Vec<u8>) -> Result<Message>
//   ^ bincode decode → X25519 ECDH → decrypt → dedup inbox check → notify delegate → Message

// Signing
core.sign_data(data: Vec<u8>) -> Result<SignatureResult>
core.verify_signature(data, signature, public_key_hex) -> Result<bool>
```

**`CoreDelegate` trait** — implement for push notifications / mobile callbacks:
```rust
pub trait CoreDelegate: Send + Sync {
    fn on_peer_discovered(&self, peer_id: String);
    fn on_peer_disconnected(&self, peer_id: String);
    fn on_message_received(&self, sender_id: String, message_id: String, data: Vec<u8>);
    fn on_receipt_received(&self, message_id: String, status: String);
}
```

---

### [Needs Revalidation] SwarmHandle (`core/src/transport/`)

The libp2p network layer. Created by `transport::start_swarm(...)`.

```rust
// Construction
let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(256);
let swarm_handle = transport::start_swarm(
    network_keypair,       // libp2p::identity::Keypair — get from core.get_libp2p_keypair()
    Some(listen_addr),     // Option<Multiaddr>
    event_tx,              // mpsc::Sender<SwarmEvent>
).await?;
```

**Key methods on `SwarmHandle`:**

```rust
swarm_handle.send_message(peer_id: PeerId, envelope_bytes: Vec<u8>) -> Result<()>
//   ^ sends raw bytes (already encrypted by IronCore) over the network

swarm_handle.dial(addr: Multiaddr) -> Result<()>
swarm_handle.subscribe_topic(topic: String) -> Result<()>
swarm_handle.share_ledger(peer_id: PeerId, entries: Vec<LedgerEntry>) -> Result<()>
swarm_handle.shutdown() -> Result<()>
```

**`SwarmEvent` variants** (received on `event_rx`):

```rust
SwarmEvent::PeerDiscovered(PeerId)
SwarmEvent::PeerDisconnected(PeerId)
SwarmEvent::MessageReceived { peer_id: PeerId, envelope_data: Vec<u8> }  // raw encrypted bytes
SwarmEvent::LedgerReceived { from_peer: PeerId, entries: Vec<LedgerEntry> }
SwarmEvent::PeerIdentified { peer_id, listen_addrs, .. }
SwarmEvent::TopicDiscovered { peer_id, topic }
SwarmEvent::ListeningOn(Multiaddr)
```

---

## [Needs Revalidation] The Integration Bridge — How IronCore ↔ SwarmHandle Connect

**Contrary to what the README claims, this IS already wired in `cli/src/main.rs`.** The exact patterns:

### [Needs Revalidation] Sending a message (UI → Network)
```rust
// In the UiCommand::Send handler:
let env = core_rx.prepare_message(pk, message.clone())?;   // IronCore encrypts
swarm_handle.send_message(target, env).await?;              // SwarmHandle transmits
```

### [Needs Revalidation] Receiving a message (Network → App)
```rust
// In the SwarmEvent::MessageReceived handler:
let msg = core_rx.receive_message(envelope_data)?;          // IronCore decrypts
let text = msg.text_content().unwrap_or("<binary>");        // extract payload
```

### [Needs Revalidation] Identity → Network key unification
```rust
// SAME Ed25519 key used for both crypto identity and libp2p PeerId:
let network_keypair = core.get_libp2p_keypair()?;
let local_peer_id = network_keypair.public().to_peer_id();
let swarm_handle = transport::start_swarm(network_keypair, Some(listen_addr), event_tx).await?;
```

---

## [Needs Revalidation] CLI Architecture — The Main Event Loop

`cli/src/main.rs` runs a single `tokio::select!` loop over three sources:

```
event_rx (SwarmEvent)  →  business logic  →  ui_broadcast (UiEvent)
ui_cmd_rx (UiCommand)  →  business logic  →  swarm_handle / state
stdin lines            →  direct handling
```

Supporting crates in `cli/src/`:
- **`server.rs`** — WebSocket + HTTP (landing page on port 9000, ws on `/ws`). Defines `UiEvent` and `UiCommand` enums. `start()` returns `(broadcast::Sender<UiEvent>, mpsc::Receiver<UiCommand>)`.
- **`api.rs`** — REST control API on port 9191. `ApiContext` holds `Arc<IronCore>`, `Arc<SwarmHandle>`, `Arc<ContactList>`, `Arc<MessageHistory>`, peers map.
- **`config.rs`** — `~/.config/scmessenger/config.toml`. Key fields: `listen_port` (default 9000), `bootstrap_nodes`.
- **`contacts.rs`** — `ContactList` (sled). Fields: `peer_id`, `public_key` (hex Ed25519), `nickname`, `added_at`, `last_seen`.
- **`history.rs`** — `MessageHistory` (sled). `Direction::Sent | Received`. `stats()`, `conversation()`, `recent()`, `search()`.
- **`ledger.rs`** — `ConnectionLedger` — persists multiaddrs + peer_ids, tracks connection success/failure for backoff-aware dialing.
- **`bootstrap.rs`** — `default_bootstrap_nodes()` returns hardcoded bootstrap multiaddrs embedded in all builds.

---

## [Needs Revalidation] Known Real Gaps (as of Feb 2026)

The README's "remaining gap" is misleading — the basic send/receive path works. The actual open issues are:

1. **Offline store-and-forward** — `IronCore.outbox` is populated but messages in it are never flushed to `SwarmHandle` when a peer comes online. `SwarmEvent::PeerDiscovered` doesn't check the outbox.

2. **Receipt / delivery confirmation** — `CoreDelegate::on_receipt_received` exists but no receipt messages are generated or handled in the swarm event loop.

3. **`wasm/` excluded from workspace** — `scmessenger-wasm` has a separate `Cargo.toml` and is commented out of the workspace. Build and bindings are untested.

4. **Drift Protocol integration** — `core/src/drift/` is implemented and unit-tested but not wired into the CLI swarm loop. Gossipsub topics exist but Drift's store-and-relay isn't exercised end-to-end.

5. **Routing module** — Mycorrhizal routing (`core/src/routing/`) is similarly unit-tested but not plugged into SwarmHandle dispatch.

6. **Privacy / onion routing** — `core/src/privacy/` is implemented but circuit construction is not called in any production path.

7. **Mobile** — UniFFI bindings compile but iOS/Android apps are scaffolding. The `SCMessengerCore.xcframework` is pre-built; whether it matches current core is unverified.

8. **`cmd_send_offline`** — offline mode encrypts a message but doesn't store it in the outbox for later delivery. It's dead-end code.

---

## [Needs Revalidation] Build & Test

```bash
# Full workspace build
cargo build --workspace

# Run all tests (~638)
cargo test --workspace

# CLI binary
cargo run -p scmessenger-cli -- --help

# Two-node local test (mDNS auto-discovery)
cargo run -p scmessenger-cli -- --storage /tmp/alice listen --port 9000
cargo run -p scmessenger-cli -- --storage /tmp/bob   listen --port 9001
# Then from bob's terminal: send <alice_peer_id> <alice_pubkey_hex> "hello"

# Docker
docker compose up
```

---

## [Needs Revalidation] Dependency Notes

Key external crates (from workspace `Cargo.toml`):
- `libp2p` — core networking (gossipsub, mDNS, identify, kad, noise, yamux)
- `tokio` — async runtime (full features)
- `sled` — embedded KV store (identity, contacts, history, ledger)
- `uniffi` — mobile FFI scaffolding
- `wasm-bindgen` — browser bindings (wasm crate only)
- `ed25519-dalek` — Ed25519 signing
- `x25519-dalek` — ECDH key exchange
- `chacha20poly1305` — AEAD encryption
- `blake3` — hashing and KDF
- `bincode` — wire serialization
- `parking_lot` — fast RwLock (used in IronCore internals)
- `zeroize` — secure key erasure
- `anyhow`, `thiserror` — error handling
- `tracing`, `tracing-subscriber` — structured logging
- `clap` — CLI argument parsing
- `colored` — terminal color output
- `uuid` — message IDs
- `chrono` — timestamp formatting

---

## [Needs Revalidation] How To Use This Skill

When working on SCMessenger, apply this skill by:

1. **Identifying the gap** — Which module boundary is unconnected? Use the gap list above as a checklist.

2. **Following the IronCore↔SwarmHandle pattern** — Always:
   - Call `core.prepare_message(recipient_pubkey_hex, text)` to produce encrypted bytes
   - Pass those bytes directly to `swarm_handle.send_message(peer_id, bytes)`
   - On `SwarmEvent::MessageReceived`, call `core.receive_message(envelope_data)` to decrypt

3. **Never bypass IronCore for crypto** — All encryption/decryption goes through `IronCore`. `SwarmHandle` is a dumb byte pipe.

4. **Wiring Drift/Routing/Privacy** — These modules follow the same pattern: they sit between `IronCore` (envelope producer) and `SwarmHandle` (byte transport). A Drift relay intercepts `prepare_message` output and wraps it in a Drift frame before passing to `send_message`. Similarly on receive.

5. **Outbox flush pattern** (to fix gap #1):
   ```rust
   SwarmEvent::PeerDiscovered(peer_id) => {
       // check outbox for queued messages to this peer
       // drain and send via swarm_handle
   }
   ```

6. **Adding a new CLI command** — Add variant to `Commands` enum → add `cmd_xyz()` async fn → wire in `main()` match. Follow the pattern of `cmd_start()` for network-aware commands.

7. **Mobile/UniFFI changes** — Edit `core/src/api.udl` first (the interface definition), then implement in `core/src/mobile_bridge.rs`. Re-run `uniffi-bindgen` to regenerate Swift/Kotlin stubs.

---

## [Needs Revalidation] Gotchas

- `public_key_hex` in contacts is the **Ed25519 signing key** (64 hex chars = 32 bytes), not the X25519 key. The ECDH key exchange is ephemeral and derived inside `prepare_message`.
- `identity_id` = Blake3 hash of the Ed25519 public key. It is NOT the same as the libp2p `PeerId`.
- `peer_id` in contacts = libp2p `PeerId` (multihash of the Ed25519 pubkey). Used for routing. `identity_id` (Blake3) is the app-layer identity.
- The wasm crate is intentionally **excluded** from the workspace `Cargo.lock`. Build it separately with `wasm-pack`.
- `IronCore::with_storage` uses sled. On mobile, storage paths must be sandboxed app directories.
- Port layout: 9000 = WebSocket/HTTP UI, 9001 = libp2p TCP P2P, 9191 = control API (localhost only).
