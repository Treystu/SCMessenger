# SCMessenger

End-to-end encrypted peer-to-peer messaging with no servers, no accounts, no phone numbers.

## Quick Start

```bash
# Build
cargo build --workspace

# Run tests (53 unit tests)
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
core/        scmessenger-core    Crypto, identity, messages, transport (libp2p)
cli/         scmessenger-cli     Interactive CLI with listen/send/identity commands
mobile/      scmessenger-mobile  iOS/Android bindings via UniFFI
wasm/        scmessenger-wasm    Browser bindings via wasm-bindgen (excluded from workspace)
reference/   —                   V1 TypeScript crypto algorithms (porting guides)
docs/        —                   Architecture, protocol, and design docs
```

### Core Modules

| Module | Purpose |
|---|---|
| `identity` | Ed25519 key generation, Blake3 identity hashing, sled persistence |
| `crypto` | X25519 ECDH + XChaCha20-Poly1305 per-message encryption |
| `message` | Message types, envelope format, bincode codec with size limits |
| `store` | Outbox queue (store-and-forward) and inbox deduplication |
| `transport` | libp2p swarm: TCP, mDNS, Kademlia, request-response messaging |

## License

MIT
