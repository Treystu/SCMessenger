# Architecture

This document describes the current implemented architecture as verified on **2026-02-23**.

For the cross-component repository map (core + CLI + mobile + WASM + Android + iOS), see `docs/REPO_CONTEXT.md`.

Primary delivery target is one unified Android+iOS+Web app. Web/WASM remains experimental today but is on the parity-critical path before GA.

## Design Principles

1. No central account system; identity is key-based.
2. Encryption/signing happens before payloads are sent over transport.
3. Store-and-forward and relay behavior are first-class.
4. Internet is one transport path; mobile proximity transports are also modeled.

## Workspace Structure

```text
core/      scmessenger-core
cli/       scmessenger-cli
mobile/    scmessenger-mobile
wasm/      scmessenger-wasm
android/   Android client app
iOS/       iOS client app
```

## Core Crate Map (`core/src`)

Current footprint:

- 77 source/interface files under `core/src`
- ~33.9K lines across Rust + UDL in `core/src`

Primary modules:

- `identity/`: key management and persistence
- `crypto/`: envelope encryption/signature helpers
- `message/`: message/envelope/receipt types + codec
- `store/`: inbox/outbox persistence and quotas
- `transport/`: swarm behavior, NAT/reflection, routing helpers, BLE/WiFi-aware abstractions
- `drift/`: drift protocol frame/envelope/store/sync policy
- `routing/`: local/neighborhood/global routing engines
- `relay/`: relay protocol/server/client/bootstrap/invite/findmy
- `privacy/`: onion/circuit/cover/padding/timing primitives
- `mobile/`: mobile lifecycle and auto-adjust behavior
- `platform/`: platform-facing service/setting abstractions
- `wasm_support/`: browser-facing transport/storage/mesh support
- `contacts_bridge.rs`, `mobile_bridge.rs`: UniFFI-facing integration layers
- `lib.rs`: `IronCore` facade and delegate flow
- `api.udl`: UniFFI contract

## Transport and Messaging

- libp2p-based swarm integration lives in `core/src/transport/*`.
- NAT reflection protocol is implemented in `core/src/transport/reflection.rs`.
- Message send path is envelope-based and supports queued delivery via outbox.
- CLI runtime (`cli/src/main.rs`) starts:
  - swarm transport
  - control API server
  - web landing/dashboard server
  - local interactive command loop

## Identity Model

- Crypto identity and network identity are derived from the same key material in current CLI flow (`get_libp2p_keypair()` from core identity path).
- UI and docs should treat `identity_id` (Blake3) and libp2p `Peer ID` as distinct identifiers with different purposes.
- Canonical cross-platform identity for persistence and exchange is `public_key_hex` (Ed25519 public key hex).

## Platform Strategy

| Target | Binding/Runtime | Crate/App |
| --- | --- | --- |
| Desktop CLI | Native Rust | `scmessenger-cli` |
| iOS / Android | UniFFI + native app layers | `scmessenger-mobile`, `iOS/`, `android/` |
| Browser | wasm-bindgen | `scmessenger-wasm` |

## Testing Snapshot

Latest verified workspace run:

- `cargo test --workspace` -> 324 passed, 0 failed, 7 ignored

See `docs/TESTING_GUIDE.md` and `docs/CURRENT_STATE.md` for exact test breakdown.
