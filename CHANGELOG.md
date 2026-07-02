# Changelog

All notable changes to SCMessenger will be documented in this file.

## [Unreleased]

### Corrected — accuracy of the 1.0.0-rc2 verification claims

The "Verification" list under 1.0.0-rc2 below did not hold on the commit
that added it (`0a49d32`), and no CI run has ever executed to back it (all
GitHub Actions jobs since 2026-06-15 failed in 1–2 s without a runner being
assigned — an account-level problem, see
`docs/release-readiness-2026-07-02.md`):

- `cargo fmt --check` — **failed** at `0a49d32` (13 diff sites) and on every
  commit since, until fixed in this changeset.
- `scripts/ffi_surface.sh` — **fails** on `main`: `gen_kotlin` panics unless
  the cdylib is prebuilt, and the checked-in Kotlin snapshot is stale. (Both
  are fixed by PR #1.) The script also exits 0 when bindings are absent, so
  a "pass" without generated bindings was vacuous.
- Android/iOS/WASM build claims — unverifiable (no CI logs exist).
  Independently reproduced in this changeset's session: the WASM release
  build **does** pass; Android/iOS remain unverified.
- `cargo test --workspace --all-features`, `cargo clippy --workspace
  --all-features -- -D warnings`, `cargo deny check
  bans licenses sources` — independently re-verified **passing** on
  `cd582f8` in this changeset's session (advisories check not run:
  network-restricted environment).

### Changed
- Applied `cargo fmt` across `core/` (including CRLF→LF normalization of
  `iron_core.rs`); `cargo fmt --check` is clean again.
- Untracked committed Python bytecode under `cloud/orchestrator/` and added
  `__pycache__/`/`*.py[cod]` to `.gitignore`.
- Added `docs/release-readiness-2026-07-02.md`: evidence-based release
  readiness assessment and ordered handoff task list.

## [1.0.0-rc2] — 2026-06-17

Release candidate completing the Fable 5 plan. All core subsystems implemented,
Rust gatekeeper suite passes, and Android/iOS/WASM builds are verified.
Includes WiFi Direct/Aware discovery wiring, background sync scheduling, and
identity backup continuity tests contributed by Gemini.

### Verification

- `cargo test --workspace --all-features` — passed
- `cargo fmt --check`, `cargo clippy --workspace --all-features -- -D warnings`, `cargo deny check` — passed
- `scripts/ffi_surface.sh` (Kotlin + Swift snapshots) — passed
- Android debug APK (`./gradlew :app:assembleDebug`) — succeeded
- iOS Simulator build (`xcodebuild -project SCMessenger.xcodeproj -scheme SCMessenger -destination 'generic/platform=iOS Simulator' build`) — succeeded
- WASM build (`cargo build --target wasm32-unknown-unknown -p scmessenger-wasm`) — succeeded

### Subsystems

- **Routing**: Mycorrhizal mesh engine with local, neighborhood, and global strategies; multipath forwarding; reputation scoring; adaptive TTL
- **Drift / DTN**: Delay-tolerant sync with MinHash sketches, custody-based relay store, frame/envelope protocol, rate limiting, and policy-driven forwarding
- **Crypto**: Double Ratchet encryption, session manager, Kani formal proofs, encrypted backup
- **Identity**: Ed25519 key management with persistent identity store
- **Transport**: Swarm management, BLE (GATT, L2CAP, beaconing, scanning), Wi-Fi Aware, escalation pipeline, NAT traversal, health monitoring
- **Storage**: Pluggable backend, relay custody, outbox, deduplication, blocked-list enforcement, inbox sweeper
- **FFI Bridge**: `mobile_bridge`, `contacts_bridge`, `blocked_bridge` with UniFFI definitions (`api.udl`)
- **CLI**: Interactive command-line client with local Axum HTTP server, BLE daemon, and mesh visualization
- **WASM**: Browser-compatible transport layer with daemon bridge and notification manager
- **iOS**: Native app with BLE Central/Peripheral, L2CAP, MultipeerConnectivity, and mDNS service discovery; SmartTransportRouter
- **Android**: Native app with BLE (GATT client/server, scanner, advertiser, L2CAP), Wi-Fi Aware, Wi-Fi Direct, mDNS discovery; SmartTransportRouter

### Deferred

- Acoustic transport — deferred to post-v1.0.0
