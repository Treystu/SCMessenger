# Changelog

All notable changes to SCMessenger are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Releases before `0.3.5` are not itemized here. For those, see the git tags
(`v0.1.0`, `v0.1.1`, `v0.1.9`, `v0.2.1`) and the commit history.

## [Unreleased]

### Changed
- Applied `cargo fmt` across `core/` (including CRLF-to-LF normalization of
  `iron_core.rs`); `cargo fmt --check` is clean again.
- Untracked committed Python bytecode under `cloud/orchestrator/` and added
  `__pycache__/` and `*.py[cod]` to `.gitignore`.
- Added `docs/release-readiness-2026-07-02.md`: evidence-based release
  readiness assessment and ordered handoff task list.
- Front-page and community-health accuracy pass: restored the truncated
  `README.md` and corrected its transport, port, crypto, and workspace claims
  against the code; replaced the abridged code of conduct with the full
  Contributor Covenant 2.1; removed a fabricated security contact address on an
  unregistered domain in favor of GitHub private vulnerability reporting;
  removed a placeholder `CODEOWNERS` file and the invented maintainer roles in
  it; corrected version, CI job name, and build-command claims in
  `CONTRIBUTING.md` and the issue templates.

### Removed
- The `1.0.0-rc2` changelog entry's "Verification" list. It asserted that
  `cargo test`, `cargo fmt --check`, `cargo clippy`, `cargo deny`,
  `scripts/ffi_surface.sh`, and the Android and iOS builds had all passed. Those
  claims did not hold on the commit that added them (`0a49d32`) and no CI run
  ever backed them: every GitHub Actions job between 2026-06-15 and the account
  fix failed in 1-2 s without a runner being assigned (see
  `docs/release-readiness-2026-07-02.md`). Specifically, `cargo fmt --check`
  failed at `0a49d32` with 13 diff sites; `scripts/ffi_surface.sh` fails on
  `main` because `gen_kotlin` panics unless the cdylib is prebuilt and the
  checked-in Kotlin snapshot is stale, and it exits 0 when bindings are absent,
  so a reported "pass" without generated bindings was vacuous; the Android and
  iOS build claims were never verifiable. The WASM release build was
  independently reproduced as passing.

## [0.3.5] - 2026-07-11

### Added
- Post-quantum hybrid migration (PQC-01 through PQC-08): ML-KEM-768
  primitives, hybrid X25519+ML-KEM-768 session establishment, suite
  negotiation (0x01 legacy / 0x02 hybrid), PQ-augmented double ratchet,
  legacy static-ECDH retirement gating with audit logging.
- `docs/ORCHESTRATION.md`: unified cross-mode orchestration protocol
  (state machine, dispatcher, tier routing, commit authority).
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

## 1.0.0-rc2 - 2026-06-17 (never released)

This version was never tagged or released; no `v1.0.0-rc2` tag exists, and the
workspace version has remained below it. It is retained here only as a record of
the development milestone that completed the Fable 5 plan, which added WiFi
Direct/Aware discovery wiring, background sync scheduling, and identity backup
continuity tests.

Subsystems implemented as of that milestone:

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

- Acoustic transport - deferred to post-v1.0.0

[Unreleased]: https://github.com/Sovereign-Communication/SCMessenger/compare/v0.3.5...HEAD
[0.3.5]: https://github.com/Sovereign-Communication/SCMessenger/compare/v0.2.1...v0.3.5
