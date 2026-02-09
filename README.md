# SCMessenger

The world's first truly sovereign messenger — works everywhere, owned by no one, unstoppable by design. End-to-end encrypted peer-to-peer messaging with no servers, no accounts, no phone numbers. Works WITH internet, never depends on it.

## Quick Start

```bash
# Build
cargo build --workspace

# Run tests (~638 tests across all modules)
cargo test --workspace

# Two-terminal demo
# Terminal 1:
cargo run -p scmessenger-cli -- --storage /tmp/alice listen --port 9000

# Terminal 2:
cargo run -p scmessenger-cli -- --storage /tmp/bob listen --port 9001

# Both terminals will auto-discover each other via mDNS.
# To send a message from Terminal 2 to Terminal 1:
#   send <alice_peer_id> <alice_crypto_pubkey_hex> Hello from Bob!
```

## Project Structure

```
core/        scmessenger-core    Rust library (~29K LoC): crypto, identity, messaging,
                                 storage, transport, mesh relay, Drift Protocol, routing,
                                 privacy, mobile/WASM support
cli/         scmessenger-cli     Interactive CLI with listen/send/identity commands
mobile/      scmessenger-mobile  iOS/Android bindings via UniFFI
wasm/        scmessenger-wasm    Browser bindings via wasm-bindgen (excluded from workspace)
reference/   —                   V1 TypeScript crypto algorithms (porting guides)
docs/        —                   Architecture, protocol, and design docs
```

### Core Modules

| Module | Purpose |
|---|---|
| `identity` | Ed25519 key generation, Blake3 identity hashing, sled persistence, Zeroize-on-Drop |
| `crypto` | X25519 ECDH + XChaCha20-Poly1305 encryption, AAD-bound sender auth, envelope signatures |
| `message` | Message types, envelope format, bincode codec with size limits |
| `store` | Outbox queue (store-and-forward) with quotas, inbox with dedup + per-sender quotas, memory and sled backends |
| `transport` | Transport abstraction, BLE, WiFi Aware, WiFi Direct, Internet, NAT traversal, escalation, reconnection with exponential backoff |
| `drift` | Drift Protocol: envelope, frame, compress, sketch/bloom, sync, store, relay, policy |
| `routing` | Mycorrhizal routing: local cell, neighborhood gossip, global routes, routing engine |
| `relay` | Self-relay network: server, client, protocol, peer exchange, bootstrap, invite, Find My integration |
| `privacy` | Onion routing, circuit construction, cover traffic, padding, timing obfuscation |
| `mobile` | Mobile service lifecycle, auto-adjust, iOS background strategy, settings |
| `platform` | Platform-specific auto-adjust and service management |
| `wasm_support` | Browser mesh participation, transport, storage |

### Key Stats

~53,000 lines of Rust across the workspace. ~638 tests. 71 source files in core alone. All modules built and tested through Phase 7 (Privacy).

## Cryptography

| Operation | Algorithm |
|---|---|
| Identity | Ed25519 (signing, identity derivation) |
| Identity hash | Blake3 (`identity_id = blake3(ed25519_public_key)`) |
| Key exchange | X25519 ECDH (ephemeral per-message) |
| KDF | Blake3 `derive_key` |
| Encryption | XChaCha20-Poly1305 (authenticated, 24-byte nonce) |
| Sender auth | AAD binding + Ed25519 envelope signatures |

## Current State

All core modules are built and unit-tested through Phase 7 (Privacy). The remaining integration gap is wiring `IronCore` (crypto/storage) to `SwarmHandle` (network) via the CLI. `prepare_message()` outputs encrypted bytes, `SwarmHandle.send_message()` sends bytes — connect them.

## License

MIT
