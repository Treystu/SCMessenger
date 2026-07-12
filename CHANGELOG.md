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

## [0.3.5] — 2026-07-11

### Added
- Post-quantum hybrid migration (PQC-01 through PQC-08): ML-KEM-768
  primitives, hybrid X25519+ML-KEM-768 session establishment, suite
  negotiation (0x01 legacy / 0x02 hybrid), PQ-augmented double ratchet,
  legacy static-ECDH retirement gating with audit logging.
- `docs/ORCHESTRATION.md`: unified cross-mode orchestration protocol
  (state machine, dispatcher, tier routing, commit authority) covering
  native Claude, Qwen/DashScope, OpenRouter, agy/Gemini, and Ollama lanes.
- `scripts/delegate_task.py`: `--verify`/`--max-rounds` auto-fix loop and
  `--mode diff` unified-diff support, reducing compile-fix round trips.

### Fixed
- Restored the `cargo test --workspace --no-run` compile gate: fixed a
  UniFFI enum/UDL mismatch (`LegacyStaticEcdhSend`), 41 stale-struct-shape
  errors in `core/src/crypto/{encrypt,ratchet}.rs` unit tests, a
  production bug where legacy-ECDH audit events recorded the peer under
  the wrong field, and a test bug where a hybrid-ratchet receiver test
  decapsulated a mismatched ciphertext.
- iOS CI workflow (`ios-build-test.yml`): removed failure-masking
  (`xcpretty || true`), fixed lowercase path references, added a Swift
  bindings drift gate.

### Changed
- Repository hygiene: archived 25 stale/superseded docs to
  `docs/historical/`, rewrote `README.md` and GitHub repo metadata for
  accuracy, groomed `HANDOFF/todo/` to live tasks only.

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
