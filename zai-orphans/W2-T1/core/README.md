# scmessenger-core

Core Rust library for SCMessenger.

## Responsibilities

- Identity lifecycle and key derivation
- Message and envelope encoding/decoding
- Encryption/decryption and signature helpers
- Inbox/outbox persistence and deduplication
- libp2p transport and swarm orchestration
- UniFFI API definitions used by mobile apps

## Key Entry Points

- Library facade: `core/src/lib.rs` (`IronCore`)
- UniFFI contract: `core/src/api.udl`
- Mobile bridge: `core/src/mobile_bridge.rs`
- Transport surface: `core/src/transport/`

## Module Map

- `identity/` key management and persistence
- `crypto/` encryption/signature primitives and validation
- `message/` message types, receipts, codecs
- `store/` inbox/outbox quota and persistence logic
- `transport/` swarm behavior, routing, multiport, NAT/reflection
- `privacy/` cover/padding/timing/onion primitives
- `routing/` local/neighborhood/global routing engines
- `relay/` relay protocol and peer exchange helpers
- `drift/` drift frame/envelope/sync abstractions
- `mobile/`, `platform/`, `wasm_support/` platform adaptation layers

## Build and Test

From repository root:

```bash
cargo build -p scmessenger-core
cargo test -p scmessenger-core
```

See `docs/TESTING_GUIDE.md` for full suite commands.
