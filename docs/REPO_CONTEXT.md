# SCMessenger Repository Context

Last reviewed: **2026-02-23**

This is the canonical "how everything fits together" document for the repository.

Execution references:

- Global rollout plan: `docs/GLOBAL_ROLLOUT_PLAN.md`
- Unified global app plan: `docs/UNIFIED_GLOBAL_APP_PLAN.md`
- Triple-check verification report: `docs/TRIPLE_CHECK_REPORT.md`
- Full file documentation tracker: `docs/DOC_PASS_TRACKER.md`

## 1) What This Repo Is

SCMessenger is a Rust-first, sovereign messaging stack with multiple client surfaces:

- CLI node/runtime (`cli/`)
- Shared core library (`core/`)
- UniFFI mobile bridge crate (`mobile/`)
- wasm-bindgen browser crate (`wasm/`)
- Android app (`android/`)
- iOS app (`iOS/`)

The system is designed so cryptography and identity live in Rust core, while platform apps handle UX, device APIs, and lifecycle.

## 1.1) Active Product Priority

Primary product surface is **one unified Android + iOS + Web delivery target**.
CLI remains a critical operator/runtime tool, and Web/WASM is experimental today but parity-critical before GA.
Rollout model is global/organic (no region-targeted cohorts), and network infrastructure is community-operated across self-hosted and third-party nodes.

## 2) Workspace and App Topology

| Layer | Component | Role |
| --- | --- | --- |
| Core | `core` (`scmessenger-core`) | Identity, crypto, message types/codecs, inbox/outbox, transport, UniFFI API surface |
| Runtime | `cli` (`scmessenger-cli`) | Interactive node, local control API, web dashboard/landing server |
| Bindings | `mobile` (`scmessenger-mobile`) | UniFFI-exported native library wrapper around `scmessenger-core` |
| Bindings | `wasm` (`scmessenger-wasm`) | Browser-facing wrapper around core + WebSocket relay receive loop |
| App | `android` | Kotlin/Compose client using UniFFI-generated Kotlin APIs |
| App | `iOS` | SwiftUI client using UniFFI-generated Swift APIs |

## 3) Identity and Message Model

Core identity model (from `core/src/lib.rs` + `core/src/api.udl`):

- `identity_id`: Blake3-derived identity string
- `public_key_hex`: Ed25519 public key (hex)
- `libp2p_peer_id`: libp2p network peer identity

These are related but not interchangeable.

Canonical cross-platform identity decision (for long-term reliability):

- **Canonical identifier: `public_key_hex`**
- Derived/operational identifiers: `identity_id`, `libp2p_peer_id`

Rationale:

- It is the direct cryptographic root and stable across transport/runtime boundaries.
- Other identifiers can be deterministically derived or mapped from key material.
- It minimizes ambiguity across Android/iOS/CLI/WASM surfaces.

Message path:

1. Sender initializes identity in `IronCore`.
2. Sender prepares envelope with `prepare_message_with_id` (or `prepare_message`).
3. Envelope is transmitted over swarm/transport.
4. Receiver calls `receive_message`, decrypts, deduplicates, and triggers delegate callbacks.
5. Receiver can acknowledge with `prepare_receipt`.

## 4) Runtime Modes and How They Connect

### CLI (`cli/src/main.rs`)

`scmessenger-cli start` launches:

- libp2p swarm (`scmessenger_core::transport`)
- local control API on `127.0.0.1:9876` (`cli/src/api.rs`)
- local web landing/dashboard server (default HTTP `9000`) (`cli/src/server.rs`)
- interactive terminal command loop

Other CLI commands use either direct storage/core access or local API calls when node mode is active.

### Mobile (`core/src/mobile_bridge.rs` + app repositories)

`MeshService` wraps `IronCore` and `SwarmBridge` for platform apps.

Platform repositories are the integration boundary:

- Android: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- iOS: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`

Responsibilities in both:

- lifecycle (`start/stop/pause/resume`)
- manager initialization (contacts/history/ledger/settings)
- event translation from Rust callbacks into app event buses
- contact/history persistence orchestration
- transport hints/bootstrap coordination

iOS directory authority:

- Active app project/code: `iOS/SCMessenger/SCMessenger.xcodeproj` and `iOS/SCMessenger/SCMessenger/`
- Legacy reference tree (not part of active Xcode target): `iOS/SCMessenger-Existing/`
- Canonical path rule: use `iOS/` (uppercase-I) in all repository references; lowercase `ios/` references are considered drift and should be normalized.

### WASM (`wasm/src/lib.rs`)

WASM exports core operations plus `startReceiveLoop` (WebSocket relay ingress) and `drainReceivedMessages` for JavaScript polling/drain semantics.

Current product status: **experimental**, but now on the GA-critical parity path for unified Android+iOS+Web delivery.

## 5) Transport and Discovery

Transport surface lives in `core/src/transport/`:

- swarm orchestration (`swarm.rs`)
- behavior composition (`behaviour.rs`)
- multi-port listen strategy (`multiport.rs`)
- address observation/reflection (`observation.rs`, `reflection.rs`)
- routing/retry/reputation (`mesh_routing.rs`)

Commands/events are mediated via `SwarmHandle` + `SwarmCommand` / `SwarmEvent`.

Current platform defaults include bootstrap relay multiaddrs in Android/iOS repositories; these are operational defaults, not protocol requirements.

Bootstrap strategy direction:

- Startup via environment configuration
- Dynamic bootstrap list fetch at runtime
- Static in-app bootstrap nodes as fallback only

## 6) Persistent State Boundaries

- Core identity/storage: managed by `IronCore::with_storage(...)`
- Contacts: `ContactManager` (sled-backed, JSON values)
- History/Ledger/Settings: exposed via UniFFI managers
- App-specific persistence:
  - Android internal files (`context.filesDir`)
  - iOS Application Support `mesh/` directory

## 7) Verified Runtime Signals (This Review)

Commands executed during this consolidation:

- `cargo run -p scmessenger-cli -- --help` -> pass (command surface confirmed)
- `cargo test -p scmessenger-mobile` -> pass (4 tests)

For broader verification baseline, see `docs/CURRENT_STATE.md` and `docs/TESTING_GUIDE.md`.

## 8) Active Gaps (Canonical Backlog)

Use `REMAINING_WORK_TRACKING.md` as the active backlog source.
Main themes currently represented there:

- full privacy toggle wiring parity on Android + iOS + Web
- real-network NAT traversal validation matrix
- Android WiFi Aware physical-device validation
- canonical identity usage hardening around `public_key_hex`
- bootstrap env + dynamic fetch implementation
- relay toggle behavior parity and enforcement validation
- core settings schema convergence across `mobile_bridge`, `mobile/settings`, and `platform/settings`
- iOS legacy tree/active tree divergence cleanup and generated-binding path normalization
- broader CI/tooling hardening (WASM browser tests, warning cleanup)

## 9) Documentation Rules for This Repo

- Canonical current-state docs belong in `docs/` or target submodule README files.
- Root-level historical reports should be treated as snapshots unless referenced by `DOCUMENTATION.md`.
- New status reports should update existing canonical docs instead of creating new one-off "complete/final/status" files.
