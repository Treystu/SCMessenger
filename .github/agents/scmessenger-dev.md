# SCMessenger Development Agent

This agent provides deep context for developing SCMessenger, the world's first truly sovereign messenger.

## Philosophy & Core Principles

1. **Relay = Messaging** — Non-negotiable coupling. Want to talk? You relay for others. No free riders.
2. **Every node IS the network** — No third-party relays, no external infrastructure.
3. **Internet is a transport, not a dependency** — Use when available, never require it.
4. **Each new node strengthens the whole** — Network grows stronger with each addition.
5. **Privacy + Infrastructure independence + Identity ownership** — All three, always.
6. **Mass market UX** — Grandma should be able to use this.

## Project Architecture

### Repository Structure
```
core/        scmessenger-core    Rust library (~29K LoC)
cli/         scmessenger-cli     Interactive CLI tool
mobile/      scmessenger-mobile  iOS/Android bindings (UniFFI)
wasm/        scmessenger-wasm    Browser bindings (wasm-bindgen)
android/     —                   Android app implementation
reference/   —                   V1 TypeScript (porting guides only)
docs/        —                   Architecture and protocol docs
```

### Core Modules (71 .rs files, ~53K LoC total)

All modules built and tested through Phase 7 (Privacy):
- identity, crypto, message, store, transport
- drift, routing, relay, privacy
- mobile, platform, wasm_support

## Build, Test & Lint Commands

```bash
# Build
cargo build --workspace
cargo build --workspace --release

# Test (~638 tests)
cargo test --workspace
RUST_LOG=debug cargo test --workspace -- --nocapture

# Format (REQUIRED before commit)
cargo fmt --all
cargo fmt --all -- --check

# Lint
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments

# Run CLI
cargo run -p scmessenger-cli -- start
cargo run -p scmessenger-cli -- --storage /tmp/alice listen --port 9000
```

## Code Conventions

- **All new code is Rust** — No TypeScript, no JavaScript
- Use `thiserror` for error types, `anyhow` for binaries
- Use `tracing` for logging (not `println!` in library code)
- Use `parking_lot::RwLock` over `std::sync::RwLock`
- Never use `unwrap()` in library code — use `?` or `expect()` with context
- Tests go in `#[cfg(test)] mod tests` in same file

## Integration Points

### ✅ Fully Wired (CLI)
- CLI → IronCore → SwarmHandle fully integrated
- See: `cli/src/main.rs:589`, `cli/src/api.rs:210-280`

### ⚠️ Partially Wired (Mobile)
- SwarmBridge (`core/src/mobile_bridge.rs:757-804`) has 6 TODO stubs
- Needs Arc<SwarmHandle> wired to actual network operations

## Key Dependencies
- libp2p 0.53 (tcp, quic, noise, yamux, gossipsub, kad, relay, identify, ping, mdns)
- Crypto: ed25519-dalek 2.1, x25519-dalek 2.0, chacha20poly1305 0.10, blake3 1.5
- Async: tokio with full features
- Storage: sled 0.34

## DO NOT
- Use unwrap() in library code
- Add dependencies without checking existing ones
- Use time-based estimates (use LoC estimates only)
- Decouple relaying from messaging
- Introduce third-party relay dependencies

## Resources
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [CODE_OF_CONDUCT.md](../../CODE_OF_CONDUCT.md)
- [SECURITY.md](../../SECURITY.md)
- [docs/](../../docs/)

**Current State**: All core modules complete. CLI fully integrated. ~638 tests passing. Mobile bridge needs wiring.
