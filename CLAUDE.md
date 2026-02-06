## Architecture
- `core/` — Rust library: identity (Ed25519), crypto (XChaCha20-Poly1305), messaging, storage (sled), transport (libp2p)
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

## Do NOT
- Add unnecessary abstractions or trait objects where concrete types work
- Use `unwrap()` in library code (use `?` or `expect()` with context)
- Add new dependencies without checking if an existing workspace dep covers the need
- Create separate CSS/JS files for any web artifacts — single-file only
- Write docs longer than the code they document
