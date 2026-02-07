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
Modules are built and unit-tested. The gap is wiring `IronCore` (crypto/storage) to `SwarmHandle` (network) via the CLI.
`prepare_message()` outputs encrypted bytes. `SwarmHandle.send_message()` sends bytes. Connect them.

## Planning & Estimation
- **LoC estimates ONLY.** Never use time-based estimates (days, weeks, months). All planning uses lines-of-code estimates for effort sizing.
- Break phases into concrete deliverables with LoC ranges.

## Do NOT
- Add unnecessary abstractions or trait objects where concrete types work
- Use `unwrap()` in library code (use `?` or `expect()` with context)
- Add new dependencies without checking if an existing workspace dep covers the need
- Create separate CSS/JS files for any web artifacts — single-file only
- Write docs longer than the code they document
- Use time-based estimates in plans or roadmaps
- Decouple relaying from messaging — they are permanently bound
- Introduce third-party relay dependencies (no Nostr relays, no external servers)
