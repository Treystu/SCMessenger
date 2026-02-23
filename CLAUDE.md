> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Needs Revalidation] Philosophy: Sovereign Communication

SCMessenger is the world's first truly sovereign messenger — works everywhere, owned by no one, unstoppable by design. Works WITH internet, never depends on it.

### [Needs Revalidation] Core Principles

1. **Relay = Messaging.** You cannot message without relaying. You cannot relay without messaging. Single toggle. Non-negotiable coupling. This IS the incentive model — want to talk? You relay for others. No free riders.
2. **Every node IS the network.** No third-party relays, no external infrastructure. When your device has internet, it IS a relay for the mesh. The mesh IS the infrastructure.
3. **Internet is a transport, not a dependency.** Use internet when available (it's fast, it's free, it bootstraps new connections). Never require it. BLE, WiFi Direct, WiFi Aware, and physical proximity are equal transports.
4. **Each new node strengthens the whole.** The network grows stronger (not more brittle) with each addition. Every node offers resources to the mesh immediately.
5. **Mycorrhizal routing.** Like a fungal network: dense local awareness, thin long-distance highways, demand-driven route formation, self-healing, no central authority.
6. **Privacy + Infrastructure independence + Identity ownership.** All three, always. No phone numbers, no emails, no accounts. You ARE your keypair.
7. **Mass market UX.** Grandma should be able to use this. Technical complexity hidden behind simple defaults. Power users get granular controls.

## [Needs Revalidation] Architecture

- `core/` — Rust library: identity (Ed25519), crypto (XChaCha20-Poly1305), messaging, storage (sled), transport (libp2p), mesh relay, Drift Protocol
- `cli/` — Development/demo CLI tool
- `mobile/` — UniFFI bindings for iOS/Android
- `wasm/` — WASM bindings for browser
- `reference/` — V1 TypeScript crypto algorithms (read-only porting guides)

## [Needs Revalidation] Code Conventions

- All new code is Rust. No TypeScript, no JavaScript.
- Use `thiserror` for error types, `anyhow` for error propagation in binaries
- Use `tracing` for logging (not `println!` in library code)
- Use `parking_lot::RwLock` over `std::sync::RwLock`
- Async runtime is `tokio`
- Network layer is `libp2p` 0.53
- Serialization: `bincode` for wire format, `serde_json` for human-readable
- Tests go in `#[cfg(test)] mod tests` in the same file, integration tests in `tests/`

## [Needs Revalidation] Key Dependencies

libp2p 0.53 with: tcp, quic, noise, yamux, gossipsub, kad, relay, identify, ping, mdns, request-response, cbor
Crypto: ed25519-dalek 2.1, x25519-dalek 2.0, chacha20poly1305 0.10, blake3 1.5

## [Needs Revalidation] Current State

All core modules are built and unit-tested through Phase 7 (Privacy):

- **Identity** (Ed25519 keys, Blake3 hashing, sled persistence) — complete with Zeroize-on-Drop
- **Crypto** (XChaCha20-Poly1305, ephemeral ECDH, AAD-bound sender auth, envelope signatures) — complete
- **Message** (types, codecs, signed envelopes) — complete
- **Store** (outbox with quotas, inbox with quotas + dedup, both memory and sled backends) — complete
- **Transport** (abstraction layer, BLE, WiFi Aware, WiFi Direct, Internet, NAT traversal, escalation, reconnection with exponential backoff) — complete
- **Drift Protocol** (envelope, frame, compress, sketch/bloom, sync, store, relay, policy) — complete
- **Routing** (neighborhood, global, local, engine) — complete
- **Relay** (server, client, protocol, peer exchange, bootstrap, invite, findmy) — complete
- **Privacy** (onion routing, circuit breakers, cover traffic, padding, timing obfuscation) — complete
- **Mobile** (UniFFI bindings) — complete
- **WASM** (browser bindings) — complete

**Integration gap resolved:** CLI now passes `DiscoveryConfig::default()` to `start_swarm()`,
completing the IronCore → SwarmHandle wiring. `prepare_message()` → encrypted bytes → `send_message()`.

## [Needs Revalidation] Planning & Estimation

- **LoC estimates ONLY.** Never use time-based estimates (days, weeks, months). All planning uses lines-of-code estimates for effort sizing.
- Break phases into concrete deliverables with LoC ranges.

## [Needs Revalidation] Codebase Stats

- 71 .rs files in core/src/ across 12 modules
- ~53,000 lines of Rust across workspace (core: ~29K, lib.rs: ~19K, cli: ~500, wasm: ~2.4K)
- ~638 test functions

## [Needs Revalidation] Hardening (Feb 2026)

**Dynamic analysis fixes — COMPLETED**

- **Resume Storm** (HIGH): `peers_needing_reconnect()` now rate-limited to `RECONNECT_MAX_CONCURRENT` (3) peers per tick with stagger, preventing OS kill on app resume
- **Zombie Loop** (HIGH): Inbox `eviction_high_water_mark` rejects messages older than most-recently-evicted, preventing infinite re-sync from peers. Persisted in sled for SledInbox.
- **Slow Loris** (MEDIUM): `FRAME_READ_TIMEOUT` (5s) constant + `FRAME_MAX_PAYLOAD` (64KB) limit + async `read_with_timeout()` on DriftFrame, plus `Timeout` and `IoError` variants on DriftError
- **Key Leak** (LOW): All intermediate crypto buffers zeroized — `shared_secret_bytes`, `ephemeral_bytes`, `nonce_bytes` in encrypt.rs (encrypt + decrypt paths)

## [Needs Revalidation] Known Technical Debt

**unwrap() / expect() / panic!() sweep — COMPLETED**

- Full sweep of all 71 .rs files in core/src/ (Feb 2026)
- Production code is CLEAN: only 5 issues found and fixed across 52 files
  - `transport/swarm.rs`: expect→map_err on behaviour builder, unwrap→? on address parse
  - `drift/sync.rs`: simplified map/flatten to and_then (2 locations)
  - `privacy/circuit.rs`: float partial_cmp unwrap→unwrap_or
- Remaining 680+ unwrap/expect/panic calls are all in `#[cfg(test)]` blocks (acceptable)
- All 23 `panic!()` calls are in test match arms (acceptable)
- Zero `todo!()` or `unimplemented!()` (good)
- Production code consistently uses: `?`, `.map_err()`, `.unwrap_or_default()`, `.unwrap_or()`

## [Needs Revalidation] Known Remaining Gaps (Feb 2026)

All previously-listed gaps resolved in Feb 2026 hardening sprint. Minor WebRTC TODOs remain:

- **WebRTC `set_remote_answer`** (`wasm/src/transport.rs`): ~50 LOC — parse answer SDP JSON, call `set_remote_description` via JsFuture. Prescription in doc-comment.
- **WebRTC ICE trickle** (`wasm/src/transport.rs`): ~30 LOC — buffer candidates in `WebRtcInner`, expose `get_ice_candidates()` / `add_ice_candidate()`. Prescription in code comment.
- **WebRTC answerer path** (`wasm/src/transport.rs`): ~60 LOC — `set_remote_offer()` + `create_answer()`. Mirrors `create_offer()` exactly.
- **`RtcSdpType` feature**: add `"RtcSdpType"` to workspace `web-sys` features in `Cargo.toml` to replace the current `js_sys::Reflect` workaround in `WebRtcTransport::create_offer()`

## [Needs Revalidation] Resolved (Feb 2026 sprint)

- Internet relay: `connect_to_relay_via_swarm()` added to `InternetRelay` — real `swarm.dial()` call
- Offline store-and-forward: outbox flushed on `PeerDiscovered` in CLI; `cmd_send_offline` now truly enqueues
- Delivery receipts: `MessageType::Receipt` + `DeliveryStatus` wired end-to-end; `IronCore::prepare_receipt()` added; CLI sends ACK on receive, displays `✓✓ Delivered`
- Integration tests: `core/tests/integration_ironcore_roundtrip.rs` — 7 tests, no network (encrypt→decrypt, wrong-recipient rejection, tamper detection, replay rejection, multi-message, self-send, empty payload)
- WASM WebSocket transport: full `connect()`/`send_envelope()`/`disconnect()` with real `web_sys::WebSocket`, buffered sends during connecting, state machine, `subscribe()` ingress channel

## [Needs Revalidation] Do NOT

- Add unnecessary abstractions or trait objects where concrete types work
- Use `unwrap()` in library code (use `?` or `expect()` with context)
- Add new dependencies without checking if an existing workspace dep covers the need
- Create separate CSS/JS files for any web artifacts — single-file only
- Write docs longer than the code they document
- Use time-based estimates in plans or roadmaps
- Decouple relaying from messaging — they are permanently bound
- Introduce third-party relay dependencies (no Nostr relays, no external servers)
