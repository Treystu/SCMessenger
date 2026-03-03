# SCMessenger Current State (Verified)

Status: Active  
Last updated: 2026-03-03

Last verified: **2026-03-03** (local workspace checks on this machine)

For architectural context across all repo components, see `docs/REPO_CONTEXT.md`.

## Verified Commands and Results

### Rust Workspace

- `cargo test --workspace`
  - Result: **pass**
  - Totals from suite output:
    - CLI: 13 passed
    - Core unit: 265 passed, 7 ignored
    - Core integration: 52 passed, 10 ignored
    - Mobile crate: 4 passed
    - WASM crate (native/non-browser tests): 33 passed
  - Aggregate: **367 passed, 0 failed, 17 ignored**
- `cargo clippy --workspace` — **clean (0 warnings)**
- `cargo fmt --all -- --check` — **clean**

### WS12 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-core --test integration_offline_partition_matrix` — **pass** (deterministic offline/partition matrix)
- `cargo test -p scmessenger-core --test integration_retry_lifecycle` — **pass**
- `cargo test -p scmessenger-core --test integration_receipt_convergence` — **pass**
- `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` — **pass**
- `cargo test -p scmessenger-wasm test_desktop_role_resolution_defaults_to_relay_only_without_identity` — **pass**
- `cargo test -p scmessenger-wasm test_desktop_relay_only_flow_blocks_outbound_message_prepare` — **pass**
- `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew :app:testDebugUnitTest --tests com.scmessenger.android.test.RoleNavigationPolicyTest --tests com.scmessenger.android.data.MeshRepositoryTest` — **pass**
- `bash ./iOS/verify-test.sh` — **pass** (21 warnings, non-fatal policy; includes local transport fallback + role-mode parity checks)
- `ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./scripts/verify_ws12_matrix.sh` — **pass**

### WS12.5 Burndown Audit Snapshot (2026-03-03)

- `cargo test -p scmessenger-core --test integration_offline_partition_matrix` — **pass**
- `cargo test -p scmessenger-core --test integration_retry_lifecycle` — **pass**
- `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` — **pass**

### WS10 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-core swarm::tests:: -- --nocapture` — **pass** (5 guardrail tests)
- Core relay guardrails now enforce:
  - per-peer token bucket limiting,
  - global inflight custody-dispatch cap,
  - duplicate suppression window and cheap abuse-shape heuristics.

### WS11 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew testDebugUnitTest` — **pass** (includes WS11 delivery-state + diagnostics formatter tests)
- `ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./android/verify-build-setup.sh` — **pass**
- `bash ./iOS/verify-test.sh` — **pass** (26 warnings, non-fatal per script policy)
- WS11 surface outcomes:
  - Android+iOS chat now expose explicit tester-facing delivery states: `pending`, `stored`, `forwarding`, `delivered`.
  - Android+iOS diagnostics exports now include structured tester bundle context (runtime summary, reliability notes, permissions rationale, delivery-state guide).
  - Android+iOS settings surfaces now include concise reliability and permissions rationale text for beta testers.

### WS9 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-wasm` — **pass** (includes desktop WS9 flow tests)
- `cargo check -p scmessenger-core --target wasm32-unknown-unknown` — **pass**
- `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` — **pass**
- Desktop target checks from release matrix:
  - `cargo check --bin scmessenger-cli --target aarch64-apple-darwin` — **pass**
  - `cargo check --bin scmessenger-cli --target x86_64-apple-darwin` — **pass**
  - `cargo zigbuild --bin scmessenger-cli --target x86_64-unknown-linux-gnu` — **pass**
  - `PATH="/opt/homebrew/opt/llvm@20/bin:$PATH" cargo xwin check --bin scmessenger-cli --target x86_64-pc-windows-msvc` — **pass**
- `./scripts/docs_sync_check.sh` — **pass**
- Desktop GUI WS9 outcomes:
  - onboarding/identity, contacts, chat send/receive, mesh dashboard, and relay-only mode are now GUI-native through local WASM/Core APIs.
  - normal desktop workflows no longer depend on CLI websocket command fallback.

### CLI Surface

- `cargo run -p scmessenger-cli -- --help`
  - Verified commands: `init`, `identity`, `contact`, `config`, `history`, `start`, `send`, `status`, `stop`, `test`

### Platform Build Readiness Scripts

- `./android/verify-build-setup.sh`
  - Result: **pass** (with `ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk`)
- `./iOS/verify-test.sh`
  - Result: **pass**
  - Confirmed simulator build plus local transport fallback and role-mode parity checks

### Platform App Builds

- Android:
  - `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew assembleDebug`
  - Result: **pass**
  - `./android/install-clean.sh`
  - Result: **pass** (fresh install on connected Pixel 6a: Gradle `clean` + `:app:installDebug` + runtime permission grant pass for Bluetooth/Location/Nearby WiFi/Notifications)
  - Multi-device note: `android/install-clean.sh` now supports `ANDROID_SERIAL=<serial>` and defaults to a single connected device (prefers TCP/IP transport when duplicates are present).
- iOS:
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 17' build`
  - Result: **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build`
  - Result: **pass** (device-target compile path verified)
  - `APPLE_TEAM_ID=<team> DEVICE_UDID=<udid> ./iOS/install-device.sh`
  - Result: **pass** (clean DerivedData + reinstall + launch on connected iPhone)
  - iOS runtime crash guard: `NSMotionUsageDescription` restored in `iOS/SCMessenger/SCMessenger/Info.plist` for motion-based power adaptation.

### Live Smoke Automation

- Cross-device smoke harness: `scripts/live-smoke.sh`
  - Runs optional clean installs (`android/install-clean.sh`, `iOS/install-device.sh`)
  - Supports deterministic Android targeting via `ANDROID_SERIAL=<serial>` (auto-selects one serial if omitted)
  - Supports simulator-only runs via `IOS_TARGET=simulator`
  - Captures Android runtime logcat for a configurable interaction window
  - Stores artifacts under `logs/live-smoke/<timestamp>/`

### Browser/WASM Runtime Validation

- `wasm-pack --version`
  - Result: **not available** (`wasm-pack` not installed in this environment)
  - Note: browser runtime tests were not executed here

## Implemented Functionality (Repository State)

- Sovereign identity and key management (Ed25519), persisted storage
- Message encryption/signing pipeline (X25519 + XChaCha20-Poly1305 + signatures)
- Inbound message chronology now uses original sender timestamp from core callbacks (`sender_timestamp`) rather than local receive-time
- Store-and-forward queues with persistence
- libp2p swarm transport with discovery, messaging, relay, and NAT reflection
- Interactive CLI with:
  - contact and history management
  - live node mode
  - local control API
  - embedded web landing/dashboard server
- Mobile UniFFI surface (MeshService, SwarmBridge, managers, settings)
- iOS and Android app codebases with active integration to Rust core
- iOS background lifecycle repository hooks are wired (`pause/resume`, ledger save, sync/discovery triggers)
- WASM crate with full libp2p swarm transport (`startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers`) using browser-native websocket-websys; legacy `startReceiveLoop` deprecated as shim
- Identity backup/restore wired end-to-end: iOS Keychain and Android SharedPreferences (`identity_backup_prefs.xml`); survives full app reinstall
- `mark_message_sent(message_id)` exposed via UniFFI; prevents outbox exhaustion on long-lived accounts
- CLI relay PeerId stable across upgrades: network key migrated from IronCore identity on first run, then persisted in `relay_network_key.pb`
- BLE GATT sequential operation queue: all GATT reads, writes, and CCCD writes serialised per-device via `Channel` + `Semaphore(1)` to comply with Android GATT API requirements

## Known Gaps and Partial Areas

### Product/Feature Gaps

- Topic subscribe/unsubscribe/publish is now wired through Rust bridge on Android and iOS
  - Android: `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt`
  - iOS: `iOS/SCMessenger/SCMessenger/Data/TopicManager.swift`
- Privacy toggle parity is wired across Android, iOS, and Web/WASM for the canonical settings surface.
  - Android: `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
  - iOS: `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift`
  - Web/WASM: `wasm/src/lib.rs`
- Android and iOS QR import/join flows are wired (Google Code Scanner on Android, VisionKit on iOS)
  - Android: `android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt`
  - Android contacts: `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt`
  - iOS join: `iOS/SCMessenger/SCMessenger/Views/Topics/JoinMeshView.swift`
  - iOS contacts: `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactsListView.swift`
- Android and iOS can generate identity QR codes from full identity export payloads (ID, public key, nickname, libp2p peer ID, listeners, relay)
  - Android identity QR: `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`
  - Android access path: Settings -> Identity -> Show Identity QR
  - Android export source: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  - iOS identity QR: `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`
  - iOS export source: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- iOS physical-device helper scripts are available:
  - Build signed device artifact: `iOS/build-device.sh`
  - Build + clean-install on connected iPhone: `iOS/install-device.sh`
- Android `WifiAwareTransport` compile issue was fixed; runtime behavior still needs field validation across devices/NAT scenarios
  - `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`

### Operational/Test Coverage Gaps

- Browser-executed WASM tests are not currently verified in this environment (`wasm-pack` missing)
- Android build verification requires `ANDROID_HOME` to be set in-shell
- App-update continuity code is complete (backup/restore, schema migration, relay key migration); pending: real-device package upgrade validation runs on Android/iOS/WASM

### Non-Markdown Extraction Highlights (2026-02-23)

- `docker/run-all-tests.sh` + `docker/docker-compose.test.yml` define a broader CI-like test surface than previously summarized:
  - Rust tests
  - lint (`cargo fmt` + `clippy -D warnings`)
  - security audit (`cargo audit`)
  - UniFFI bindings checks (Kotlin + Swift generation)
  - WASM node-runtime tests (`wasm-pack test --node`)
- `scripts/deploy_gcp_node.sh` is a concrete community-operator deployment path using Cloud Build + Compute Engine container update/restart for the relay/bootstrap role.
- `scripts/get-node-info.sh` documents and automates extraction of `Peer ID`, external address API query (`/api/external-address` on port `9876`), and shareable bootstrap multiaddr formatting.
- `iOS/verify-test.sh` is now an actual build verification script (simulator workspace build), not a placeholder.
- `android/app/build.gradle` currently aligns ABI filters and Rust build targets to `arm64-v8a` + `x86_64` (earlier mismatch note is outdated).
- `android/verify-build-setup.sh` now validates the same ABI matrix (`aarch64-linux-android` + `x86_64-linux-android`).
- `iOS/copy-bindings.sh` is normalized to the active generated path only: `iOS/SCMessenger/SCMessenger/Generated/`.

### Repository Structure Clarifications

- Active iOS app project/code is under:
  - `iOS/SCMessenger/SCMessenger.xcodeproj`
  - `iOS/SCMessenger/SCMessenger/`
- `iOS/SCMessenger-Existing/` is a legacy/reference tree and is not part of the active Xcode target.

### Product Directives (2026-02-23)

- Primary delivery target is one unified Android+iOS+Web app.
- Rollout model is global and organic (no region-targeted gating sequence).
- Infra model is community-operated (self-hosted and third-party relay/bootstrap operators are both valid).
- Canonical cross-platform identity is `public_key_hex`; other IDs are derived/operational.
- Relay toggle must remain user-controlled; OFF blocks all inbound/outbound relay traffic while preserving local read access.
- Bootstrap configuration direction is env-driven startup config plus dynamic fetch (with static fallback).
- Reliability objective is active-session availability plus durable eventual delivery (messages are retained/retried until route availability).
- Storage policy must be bounded so local history/outbox cannot grow unbounded.
- First-run consent gate is required before first messaging actions.
- Alpha language scope is English-only (i18n expansion remains backlog work).
- Abuse controls and regional compliance mapping are explicitly post-alpha tracks.
- Web/WASM remains experimental today and must be promoted to parity before GA.

## Source of Truth

Use this file plus:

- `README.md` (repo entrypoint)
- `docs/DOCUMENT_STATUS_INDEX.md` (documentation lifecycle map)
- `docs/TESTING_GUIDE.md` (test commands and expected outcomes)
- `REMAINING_WORK_TRACKING.md` (active gap backlog)
- `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` (active milestone scope/order)
- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (active residual risk posture)
- `docs/EDGE_CASE_READINESS_MATRIX.md` (extreme environment readiness/hardening)

Treat older status and audit report docs as historical snapshots unless they are explicitly linked from the files above as current.
