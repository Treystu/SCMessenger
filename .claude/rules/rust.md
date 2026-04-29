# Rust Development Rules

Re-injected into agent context on every turn.

## Core Crate (`scmessenger-core`)

- All state behind `Arc<RwLock<...>>` (parking_lot), not std::sync.
- `IronCore` is the single entry point — do not bypass it with direct store/db access.
- Module boundaries are strictly enforced:
  - `identity/` — Ed25519 ONLY for signing keys. X25519 for message encryption.
  - `crypto/` — XChaCha20-Poly1305 authenticated encryption. No alternative ciphers without architecture review.
  - `transport/` — libp2p Swarm. Transport priority: BLE → WiFi → mDNS → QUIC/TCP relay → Internet relay.
  - `store/` — sled-backed. No direct sled access outside `store/` module.

## Platform Compilation Gates

- `cfg(target_arch = "wasm32")` — WASM: rexie (IndexedDB), wasm-bindgen-futures, getrandom/js. No tokio.
- `cfg(all(not(wasm32), not(android)))` — Desktop: full tokio, libp2p TCP+QUIC+mDNS+DNS.
- `cfg(all(not(wasm32), android))` — Android: full tokio, libp2p TCP+QUIC. NO mDNS, NO DNS.

## Code Quality

- **Prefer `glm-5.1:cloud`** for Rust core work — it has proven Rust output quality.
- **Unsafe blocks:** Must have `// SAFETY:` comment explaining invariants. Requires `deepseek-v3.2:cloud` review.
- **Error handling:** Use `anyhow` for application errors, `thiserror` for library errors. Never `unwrap()` in production paths.
- **Kani proofs:** Behind `kani-proofs` feature. All crypto module changes must pass existing proofs or add new ones.

## Testing

- Integration tests in `core/tests/` — one file per scenario.
- Property-based testing via `proptest` harness in `core/src/crypto/proptest_harness.rs`.
- Test naming: `integration_<domain>_<scenario>` (e.g., `integration_e2e`, `integration_relay_custody`).
- New features require at minimum: unit test + integration test + property test (for crypto/routing).

## UniFFI Bindings

- Core exposes `api.udl` + proc macros for mobile bindings.
- Generated via `gen_kotlin` and `gen_swift` bins (behind `gen-bindings` feature).
- WASM uses `wasm-unstable-single-threaded` feature. Browser client connects via JSON-RPC WebSocket `/ws`.
