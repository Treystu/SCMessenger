# scmessenger-wasm

WebAssembly bindings crate for SCMessenger.

## Purpose

`scmessenger-wasm` wraps `scmessenger-core` with a browser-friendly `wasm-bindgen` API.
It provides identity/crypto/message operations plus relay receive-loop helpers for JS clients.

## Key Exports

- `IronCore` wrapper (`new`, `withStorage`, `start`, `stop`)
- Identity and signature helpers
- Message prepare/receive methods
- `startReceiveLoop(relayUrl)` for async WebSocket ingress
- `drainReceivedMessages()` for batched JS-side consumption

## Source Map

- Main API: `wasm/src/lib.rs`
- Transport helper: `wasm/src/transport.rs`
- Connection state: `wasm/src/connection_state.rs`
- Worker helper: `wasm/src/worker.rs`

## Build and Test

From repository root:

```bash
cargo build -p scmessenger-wasm
cargo test -p scmessenger-wasm
```

Browser-runtime tests require `wasm-pack` in the environment.
