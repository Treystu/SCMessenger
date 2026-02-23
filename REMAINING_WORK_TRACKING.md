# SCMessenger Remaining Work Tracking

This is the active implementation backlog based on repository state verified on **2026-02-23**.

## Priority 0: Core Parity + Correctness

1. Wire iOS privacy feature toggles into Rust core settings path
   - File: `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift`
   - Current: non-core privacy toggles intentionally disabled in UI and not forwarded to Rust core
   - Target: propagate to core privacy config once API surface supports per-feature toggles

2. Validate end-to-end Internet/NAT traversal across real networks (core requirement)
   - Scope: CLI node <-> commodity host nodes <-> Android/iOS clients over mixed LAN/WAN paths
   - Current: transport code and local tests pass; no full field matrix run captured in this repo state
   - Target: scripted verification matrix with success criteria for relay fallback, NAT traversal, and message delivery latency

3. Validate Android WiFi Aware responder behavior on physical devices
   - File: `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`
   - Current: compile blocker fixed (`Network.bindSocket(ServerSocket)` removed), but no device-level interoperability report
   - Target: verify subscriber/responder connectivity on supported hardware and document pass/fail by Android version/device

## Priority 1: Platform UX Completeness

1. Complete iOS repository background operation stubs
   - File: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
   - Current: several methods are placeholders (`syncPendingMessages`, maintenance helpers)
   - Target: implement actual background sync/discovery/update routines or remove unused stubs

## Priority 2: Tooling and Validation Coverage

1. Add browser-executed WASM test job
   - Current: workspace tests cover native/non-browser WASM paths only
   - Gap: no `wasm-pack test` verification in this environment/flow

2. Resolve integration test warnings in core tests
   - Current: `cargo test --workspace` passes but emits unused imports/variables in integration suites
   - Target: warning-clean test suite for stricter CI (`-D warnings`) readiness

3. Standardize Android CI environment setup for `ANDROID_HOME`
   - Current: local build passes with explicit `ANDROID_HOME`; generic environments still fail without it
   - Target: documented CI setup and/or preflight script guardrails

## Verified Stable Areas (No Active Gap)

- `cargo test --workspace` passes (324 passed, 0 failed, 7 ignored)
- Core NAT reflection integration tests pass
- iOS build verification script passes, including static library build
- iOS simulator app build passes (`SCMessenger` scheme, iPhone 17 simulator)
- Android build verification script passes when `ANDROID_HOME` is set
- Android app build passes (`./gradlew assembleDebug`)
- Topic subscribe/unsubscribe/publish paths are wired on Android and iOS
- QR contact + join bundle scan flows are wired on Android and iOS
- CLI command surface and control API paths are functional

## Change Control Notes

- Use `docs/CURRENT_STATE.md` as the verification snapshot.
- Treat older completion/audit reports as historical context unless reconfirmed.
