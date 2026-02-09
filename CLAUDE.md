## Philosophy: Sovereign Communication
SCMessenger is the world's first truly sovereign messenger — works everywhere, owned by no one, unstoppable by design. Works WITH internet, never depends on it.

### Core Principles
1. **Relay = Messaging.** You cannot message without relaying. You cannot relay without messaging. Single toggle. Non-negotiable coupling. This IS the incentive model — want to talk? You relay for others. No free riders.
2. **Every node IS the network.** No third-party relays, no external infrastructure. When your device has internet, it IS a relay for the mesh. The mesh IS the infrastructure.
3. **Internet is a transport, not a dependency.** Use internet when available (it's fast, it's free, it bootstraps new connections). Never require it. BLE, WiFi Direct, WiFi Aware, and physical proximity are equal transports.
4. **Each new node strengthens the whole.** The network grows stronger (not more brittle) with each addition. Every node offers resources to the mesh immediately.
5. **Mycorrhizal routing.** Like a fungal network: dense local awareness, thin long-distance highways, demand-driven route formation, self-healing, no central authority.
6. **Privacy + Infrastructure independence + Identity ownership.** All three, always. No phone numbers, no emails, no accounts. You ARE your keypair.
7. **Mass market UX.** Grandma should be able to use this. Technical complexity hidden behind simple defaults. Power users get granular controls.

## Architecture
- `core/` — Rust library: identity (Ed25519), crypto (XChaCha20-Poly1305), messaging, storage (sled), transport (libp2p), mesh relay, Drift Protocol
- `cli/` — Development/demo CLI tool
- `mobile/` — UniFFI bindings for iOS/Android
- `wasm/` — WASM bindings for browser
- `reference/` — V1 TypeScript crypto algorithms (read-only porting guides)

## Code Conventions
- All new code is Rust. No TypeScript, no JavaScript.
- Use `thiserror` for error types, `anyhow` for error propagation in binaries
- Use `tracing` for logging (not `println!` in library code)
- Use `parking_lot::RwLock` over `std::sync::RwLock`
- Async runtime is `tokio`
- Network layer is `libp2p` 0.53
- Serialization: `bincode` for wire format, `serde_json` for human-readable
- Tests go in `#[cfg(test)] mod tests` in the same file, integration tests in `tests/`

## Key Dependencies
libp2p 0.53 with: tcp, quic, noise, yamux, gossipsub, kad, relay, identify, ping, mdns, request-response, cbor
Crypto: ed25519-dalek 2.1, x25519-dalek 2.0, chacha20poly1305 0.10, blake3 1.5

## Current State
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

## Planning & Estimation
- **LoC estimates ONLY.** Never use time-based estimates (days, weeks, months). All planning uses lines-of-code estimates for effort sizing.
- Break phases into concrete deliverables with LoC ranges.

## Codebase Stats
- 71 .rs files in core/src/ across 12 modules
- ~53,000 lines of Rust across workspace (core: ~29K, lib.rs: ~19K, cli: ~500, wasm: ~2.4K)
- ~638 test functions

## Hardening (Feb 2026)
**Dynamic analysis fixes — COMPLETED**
- **Resume Storm** (HIGH): `peers_needing_reconnect()` now rate-limited to `RECONNECT_MAX_CONCURRENT` (3) peers per tick with stagger, preventing OS kill on app resume
- **Zombie Loop** (HIGH): Inbox `eviction_high_water_mark` rejects messages older than most-recently-evicted, preventing infinite re-sync from peers. Persisted in sled for SledInbox.
- **Slow Loris** (MEDIUM): `FRAME_READ_TIMEOUT` (5s) constant + `FRAME_MAX_PAYLOAD` (64KB) limit + async `read_with_timeout()` on DriftFrame, plus `Timeout` and `IoError` variants on DriftError
- **Key Leak** (LOW): All intermediate crypto buffers zeroized — `shared_secret_bytes`, `ephemeral_bytes`, `nonce_bytes` in encrypt.rs (encrypt + decrypt paths)

## Known Technical Debt
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

## Do NOT
- Add unnecessary abstractions or trait objects where concrete types work
- Use `unwrap()` in library code (use `?` or `expect()` with context)
- Add new dependencies without checking if an existing workspace dep covers the need
- Create separate CSS/JS files for any web artifacts — single-file only
- Write docs longer than the code they document
- Use time-based estimates in plans or roadmaps
- Decouple relaying from messaging — they are permanently bound
- Introduce third-party relay dependencies (no Nostr relays, no external servers)
