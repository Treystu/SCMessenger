# SCMessenger Current State (Verified)

Status: Active  
Last updated: 2026-03-06

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

### WS12.18 Alpha Readiness Sanity + Interop Closure (2026-03-03 HST)

- Rust quality/build gates:
  - `cargo check --workspace` — **pass**
  - `cargo fmt --all -- --check` — **pass**
  - `cargo clippy --workspace` — **pass**
  - `cargo clippy --workspace --lib --bins --examples -- -D warnings` — **pass**
- Android quality/build gates:
  - `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew :app:lintDebug` — **pass** (all prior 21 lint errors remediated; warnings remain)
- iOS/WASM gates:
  - `bash ./iOS/verify-test.sh` — **pass**
  - `cd wasm && wasm-pack build` — **pass**
- Hard-blocker remediation delivered:
  - Android lint `MissingPermission`/`NewApi` blockers closed in BLE advertiser/GATT server, WiFi transport manager, notification posting paths, and foreground-service API gating.
  - Rust clippy strict failures closed in store backend/contact/history/custody codepaths and example code.
- Interoperability/function completeness artifacts:
  - Added deterministic matrix generator: `scripts/generate_interop_matrix.sh`
  - Generated matrix doc: `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`
  - WS12.18 matrix triage identified adapter-parity gaps, now closed in WS12.20 follow-up wiring.
- Historical relocation (no purge):
  - Root-level one-off scripts/log buffers moved to `reference/historical/` with provenance index (`reference/historical/README.md`).

### WS12.19 Documentation/Folder Cleanup Correction (2026-03-03 HST)

- Cleanup drift correction:
  - Restored active iOS install/build helpers from historical location into active `iOS/` script surface:
    - `iOS/build-device.sh`
    - `iOS/install-device.sh`
    - `iOS/install-sim.sh`
  - Kept stale/non-canonical scripts (`build-rust.sh`, `verify-build-setup.sh`) in `docs/historical/iOS/scripts/` with explicit archive README.
- Active doc path fixes:
  - Replaced stale references to the legacy iOS setup-check script in active docs with `bash ./iOS/verify-test.sh`.
  - Updated iOS setup docs to point at active canonical docs/backlog instead of archived iOS planning files.

### WS12.20 Alpha Readiness Completion Sweep (2026-03-03 HST)

- Interop/fn-completeness closures:
  - CLI now wires identity backup import/export, explicit mark-sent, history clear, listeners/path-state/diagnostics/peers status surfaces.
  - WASM now wires local nickname override, history retention/prune controls, and external-address visibility.
  - Android+iOS adapters now consume `reset_stats`; CLI/WASM consume history retention/prune controls.
- Build/gate revalidation:
  - `cargo check --workspace` — **pass**
  - `cargo clippy --workspace --lib --bins --examples -- -D warnings` — **pass**
  - `cd android && ./gradlew :app:generateUniFFIBindings :app:compileDebugKotlin :app:lintDebug` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`0 warnings` in this run)
  - `cd wasm && wasm-pack build` — **pass**
- Interop evidence update:
  - `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md` now reports no static adapter-consumption gaps.
- Script operations docs:
  - Added active operations guide: `scripts/README.md` (5-node + launch/control/debug workflow map).

### WS12.21 Pairwise Deep-Dive Status Sweep (2026-03-03 HST)

- Deep-dive analyzers run on current artifacts:
  - `bash ./scripts/correlate_relay_flap_windows.sh ios_diagnostics_latest.log logs/5mesh/gcp.log`
    - classification: `unsynchronized_artifacts_no_time_overlap`
  - `bash ./scripts/verify_relay_flap_regression.sh ios_diagnostics_latest.log`
    - `PASS` (no deterministic relay dial-loop regression in this artifact)
  - `bash ./scripts/verify_receipt_convergence.sh android_mesh_diagnostics_device.log ios_diagnostics_latest.log`
    - result: no `delivery_attempt` message markers found in this artifact pair
  - `bash ./scripts/verify_ble_only_pairing.sh android_logcat_latest.txt ios_diagnostics_latest.log`
    - result: no strict-BLE markers/timeouts in this artifact pair
- Fresh live probe attempt:
  - `IOS_TARGET=device IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=20 GCP_RELAY_CHECK=1 bash ./scripts/live-smoke.sh`
  - result: Android connected, iOS physical device listed as `unavailable` by `xcrun devicectl`, so physical dual-device pairing deep dive could not complete in this pass.
- Simulator fallback probe:
  - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=20 GCP_RELAY_CHECK=1 bash ./scripts/live-smoke.sh`
  - artifacts captured under `logs/live-smoke/20260303-005207/`; expected limitation remains (no CoreBluetooth hardware path in simulator).
- Pairwise closure status:
  - `Core -> Android`: closed in static interop matrix.
  - `Core -> iOS`: closed in static interop matrix.
  - `Core -> WASM/Desktop`: closed in static interop matrix.
  - `Android <-> iOS` direct/relay delivery+receipt continuity: still open pending synchronized physical-device artifact capture.
  - `Android <-> iOS` strict BLE-only continuity: still open pending synchronized physical-device BLE-only artifact capture.

### WS12.22 Android+iOS Crash + Stability Hardening Sweep (2026-03-03 HST)

- Fresh runtime evidence captured:
  - iOS debug-detach bundle: `logs/pairwise/ios-debug-detach-20260303-014559`
  - Android USB capture: `logs/pairwise/android-usb-pull-20260303-014849`
- iOS crash RCA from latest SCMessenger crash artifact in the captured bundle:
  - crash path pointed to BLE peripheral send flow (`BLEPeripheralManager.sendDataToCentral`) with force-unwrap-sensitive code under active send.
- iOS hardening applied:
  - BLE central/peripheral managers now run on the main queue for consistent delegate/state access.
  - Removed force-unwrap hotspots in BLE peripheral send/advertise paths; send methods now return explicit success/failure booleans.
  - Added reconnect/service-rediscovery behavior in BLE central send flow when disconnected or missing message characteristic.
  - Added pending outbox bounded-expiry drop policy in repository (`attempt_count` and age guard) with explicit diagnostics markers.
- Android hardening applied:
  - Removed all Kotlin `!!` force unwrap usage from app source paths (repository, BLE transport, settings/viewmodel, platform bridge).
  - BLE advertiser restart churn reduced by skipping unnecessary restart when identity payload does not change advertisement-visible bytes.
  - BLE GATT client send path now attempts reconnect when disconnected/not-ready instead of immediate terminal false path.
  - Added pending outbox bounded-expiry drop policy mirroring iOS diagnostics semantics.
- Verification gates rerun after fixes:
  - `cd android && ./gradlew :app:compileDebugKotlin :app:lintDebug` — **pass** (`0 errors`, warnings remain)
  - `bash ./iOS/verify-test.sh` — **pass** (`0 warnings` in this run)
  - `bash ./scripts/generate_interop_matrix.sh` — **pass**
- Remaining closure dependency:
  - Requires fresh synchronized physical Android+iOS live send/receipt artifact capture to confirm no new iOS send crash and to close remaining pairwise runtime risks.

### WS12.23 Pending-Outbox Synchronization Reliability Pass (2026-03-03 HST)

- Root-cause closure target:
  - older pending messages could remain stuck while a newer message to the same peer delivered, because promotion/flush triggers were not consistently tied to active-connection events.
- Reliability hardening applied in `MeshRepository` on Android+iOS:
  - pending queue promotion now matches both canonical `peerId` and cached `routePeerId`,
  - `peer_identified` and BLE identity-read paths now promote same-peer queue entries and immediately flush retries,
  - iOS connected-event emission now also triggers targeted same-peer promotion/flush.
- Expected behavior shift:
  - when any usable path to a peer is active, the app immediately opportunistically drains older undelivered entries for that peer instead of waiting for periodic backoff windows.
- Verification after patch:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`3 warnings`, non-fatal)
- Remaining live proof requirement:
  - capture fresh synchronized physical Android+iOS traces and confirm deterministic backlog drain + pending-to-delivered convergence on both directions.

### WS12.24 Follow-up: Sender-State Convergence + Conversation Swipe-Delete Parity (2026-03-03 HST)

- Field-reported issue intake:
  - iOS -> Android sends can still show `stored` on iOS sender even when Android recipient has already received/rendered the message.
- Problem decomposition for closure:
  - validate Android receipt/ack emission for affected message IDs,
  - validate iOS receipt ingest + message-ID correlation into sender history state,
  - validate UI mapping does not regress message state from `delivered` back to `stored`.
- Planned closure evidence gate:
  - synchronized Android+iOS+relay artifact bundle for at least one affected message ID, proving recipient ingest and sender-side `delivered` convergence in the same session.
- Conversation-delete UX parity update in this pass:
  - Android conversation list now supports end-to-start swipe-to-delete with confirmation dialog, matching iOS swipe-delete behavior and reusing existing `clearConversation(peerId)` data path.
- Verification in this pass:
  - `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew :app:compileDebugKotlin` - **pass**

### WS12.25 Mega-Update Intake: Pending-Sync RCA + Node-Role Unification (2026-03-03 HST)

- `run5.sh` and associated logs were reviewed for the reported "older pending messages remain undelivered while newer traffic still appears active" issue:
  - `logs/5mesh/latest/android.log` shows the same message ID (`1c24a6d2-5114-42cc-8545-01f9bfc41eb1`) repeatedly cycling `forwarding -> stored`, with `Core-routed delivery failed` / `Relay-circuit retry failed` and repeated flush triggers (`peer_discovered`, `peer_identified`).
  - `logs/pairwise/ios-debug-detach-20260303-014559/pending_outbox.json` shows multiple queued items for one canonical peer with persisted `routePeerId` and relay-circuit address hints tied to prior relay identities.
- Root-cause conclusion (implementation confidence: medium-high):
  - route hints/candidates can become stale under peer-id/alias churn, and receipt/retry paths were not consistently preferring fresh inbound route/listener context for the active sender identity.
- Fixes applied on both Android and iOS:
  - existing-contact route-hint updates now refresh on route change (not only when hints are initially blank),
  - delivery-receipt send path now accepts preferred inbound route/listener hints and uses them for targeted retries,
  - route candidate building now includes recipient-public-key-aware candidate discovery/filtering and relay/mismatched-candidate rejection,
  - outbound send guard now rejects relay/bootstrap identities as direct chat recipients.
- UI role-model unification applied (Android + iOS dashboard):
  - reduced displayed node-role buckets to exactly two categories:
    - `Node` (full identity),
    - `Headless Node` (no identity; includes relay/headless transport peers).
- Verification in this pass:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (3 warnings, non-fatal in script policy)
- Remaining closure gate:
  - capture fresh synchronized physical Android+iOS artifacts post-fix to confirm previously stuck pending entries drain and sender-side states converge to `delivered`.

### WS12.26 Sender-State + Conversation Preview Convergence Hotfix (2026-03-03 HST)

- Field issue intake addressed in this pass:
  - message status and conversation-row previews could stay stale (`stored`/older preview text) even after receipt-driven delivery state transitions.
- Root-cause conclusion (implementation confidence: high):
  - Android+iOS receipt handlers updated durable history state but did not always publish a fresh `messageUpdates` event after mutating delivered/pending state, so UI lists could continue rendering stale records.
  - iOS conversation preview selection depended on a narrow ordering assumption (`recentMsgs.last` from a minimal slice), making "latest preview" correctness fragile under ordering/alias drift.
- Fixes applied:
  - Android `MeshRepository.onReceiptReceived` now emits refreshed `MessageRecord` via `messageUpdates` immediately after `markDelivered` + pending-outbox removal.
  - Android `ConversationsViewModel` now also refreshes conversation state on `MessageEvent.Delivered`/`MessageEvent.Failed`.
  - iOS `MeshRepository.onDeliveryReceipt` now emits refreshed `MessageRecord` via `messageUpdates` after receipt-driven history/pending updates.
  - iOS `ConversationListView` preview selection now chooses newest message by timestamp from a bounded recent sample (`limit: 25` + `max(timestamp)`), removing reliance on list order assumptions.
  - UniFFI Swift bridge now marks `FfiConverter` helper statics as `nonisolated(unsafe)` for Swift strict-concurrency compatibility, and this rewrite is persisted in `core/src/bin/gen_swift.rs` so regenerated bindings keep compiling under `-default-isolation=MainActor`.
- Verification in this pass:
  - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.ChatViewModelTest" --tests "com.scmessenger.android.ui.viewmodels.ConversationsViewModelTest"` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (build succeeds under current Swift isolation settings)
- Remaining closure gate:
  - live/passive-log confirmation for this hotfix still requires mobile binaries that include this patch set.

### WS12.27 Node-Role Classification Correction + Trip Readiness Validation (2026-03-03 HST)

- Field issue intake:
  - iOS reported a confirmed full iOS-sim peer rendered as `Headless Node`.
- Root-cause correction applied on Android+iOS:
  - peer-identify classification now treats `/headless/` agent string as provisional when transport identity resolves.
  - resolved identity peers are promoted to full classification even when prior identify metadata indicated headless.
  - `isKnownRelay` now treats only bootstrap peers and non-full dynamic relay peers as relay-only, preventing full peers from being forced into headless bucket due relay-capability flags.
- Build verification after patch:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass**
- Live relay visibility snapshot (fast run, no reinstall):
  - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=25 GCP_RELAY_CHECK=0 bash ./scripts/live-smoke.sh`
  - Android capture `logs/live-smoke/20260303-113927/android-logcat.txt` showed:
    - `IdentityDiscovered(... listeners=[.../p2p-circuit/...])`
    - dashboard/runtime state: `Loaded 2 discovered peers (2 full)` and `Mesh Stats: ... 2 full, 0 headless`.
- Remaining validation gap:
  - physical iOS-device + Android synchronized capture with WS12.27 binaries is still required for final field closure.

### WS12.28 Transport Regression Hotfix (2026-03-03 HST)

- Live regression evidence intake from the active trip log bundle:
  - `logs/5mesh/20260303_115412/android.log` showed repeated `BleGattClient.connect` `NullPointerException` at line 238 while retry loops were active.
  - same bundle showed repeated dials to special-use/unusable addresses (for example `/ip4/192.0.0.6/...`) and persistent `stored` retry loops with no core peers connected.
- Root-cause conclusions (implementation confidence: high):
  - Android BLE fallback could enter a crash loop when `BluetoothDevice.connectGatt(...)` returned `null` and the result was stored as a non-null `BluetoothGatt`.
  - Android+iOS local address selection and dial filtering allowed special-use IPv4 values, enabling stale/unroutable candidate churn.
- Fixes applied in this pass:
  - Android `BleGattClient.connect`:
    - added address format guard (`BluetoothAdapter.checkBluetoothAddress`),
    - added explicit `connectGatt == null` handling with graceful failure instead of exception loop.
  - Android `MeshRepository` networking helpers:
    - added special-use IPv4 filtering for dialability checks,
    - hardened local IPv4 selection to prefer usable private LAN addresses and skip special-use ranges.
  - iOS `MeshRepository` networking helpers:
    - mirrored special-use IPv4 filtering in dialability checks,
    - hardened local IPv4 selection scoring to prefer usable private LAN addresses and skip special-use ranges.
- Verification in this pass:
  - `cd android && ./gradlew app:compileDebugKotlin -q` — **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -sdk iphonesimulator -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 16e' build CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO CODE_SIGN_IDENTITY=''` — **pass**
- Remaining closure gate:
  - deploy WS12.28 binaries to physical Android+iOS and confirm live logs no longer show BLE NPE loops or special-use IPv4 dial attempts during retry windows.

### WS12.29 Known-Issues Consolidation + Full-Functionality Burndown (2026-03-03 HST)

- Fresh field evidence intake in this pass:
  - iOS crash reports pulled from device crash storage show repeated send-path `SIGTRAP` in `BLEPeripheralManager.sendDataToCentral` during outbox flush/send flow.
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger-2026-03-02-185622.ips`
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger-2026-03-02-185659.ips`
  - iOS watchdog reports show CPU resource kills under retry pressure.
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger.cpu_resource_fatal-2026-02-27-213024.ips`
  - Android on-device diagnostics show persistent stale-route/stale-BLE-target retry churn:
    - `Network error` count in extracted log: 291
    - repeated route target: `12D3KooWHqa2jd8Ec3bbXR24Fn8Lc2rPQQwjeEiY2zUyXXMCez27`
    - repeated BLE fallback target: `65:99:F2:D9:77:01`
    - source: `logs/device-debug-20260303-140445/android-mesh_diagnostics-device.log`
- Additional operator-evidence gap identified:
  - direct pull of large iOS `mesh_diagnostics.log` from app container repeatedly failed with file-service socket closure; this blocks deterministic device-side timeline extraction until workflow/tooling is hardened.
- Consolidated remediation source of truth added:
  - `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
- Requested TODO explicitly added for UX safety:
  - require confirmation in iOS before contact deletion (`UX-IOS-001` in WS12.29 plan and active backlog).

### WS12.30 Live Verification Feedback-Loop Harness (2026-03-03 HST)

- Added a non-destructive iterative harness copy for field step-by-step validation:
  - `scripts/run5-live-feedback.sh`
- Execution model:
  - deploy Android+iOS build updates (`scripts/deploy_to_device.sh both`, optional skip flag),
  - run `run5.sh` with `--update` for synchronized 5-node capture,
  - enforce sequential gates before accepting a step:
    - log-health gate (all five node logs),
    - directed pair-matrix gate (all node pairings),
    - crash/fatal marker gate,
    - deterministic verifiers (`relay_flap`, `ble_only`, `receipt_convergence`, `delivery_state_monotonicity`).
- Evidence packaging:
  - each attempt writes a self-contained bundle under `logs/live-verify/<step>_<timestamp>/attempt_*`.
- Recommended command per fix:
  - `./scripts/run5-live-feedback.sh --step=<fix-id> --time=5 --attempts=3`
  - add `--require-receipt-gate` when sender/receipt convergence is the closure target.

### WS12.31 Stale-Target Convergence Hardening + Transport Priority Clarification (2026-03-04 HST)

- Field issue intake addressed in this pass:
  - active Android/iOS paired usage still reported non-delivery with stale route/BLE retry churn under WS12.29 open-risk class.
- Reliability hardening applied in `MeshRepository` on Android+iOS:
  - route candidate ordering now prefers fresh discovery/ledger candidates before persisted note/cached hints.
  - route-candidate recipient validation now requires either:
    - extracted route peer key matches recipient key, or
    - runtime-discovered/ledger evidence that route peer maps to recipient key.
  - failed send attempts no longer persist failed route IDs back into pending-outbox entries (`routePeerId` stays unset when no route ACK succeeded), preventing stale-route lock-in across retries.
  - local BLE fallback target selection now prefers currently connected BLE peers ahead of cached `ble_peer_id` hints.
  - Android disconnect handling now prunes disconnected aliases by peer ID + canonical ID + matched public-key aliases (previously only direct keys were removed in callback path).
- Transport-priority audit (current behavior):
  - Android send path: `WiFi Direct` -> `BLE` -> `Core direct route candidates (LAN-prioritized addresses first)` -> `relay-circuit retry`.
  - iOS send path: `Multipeer` -> `BLE` -> `Core direct route candidates (LAN-prioritized addresses first)` -> `relay-circuit retry`.
  - strict BLE-only mode (`SC_BLE_ONLY_VALIDATION=1`) blocks Multipeer/WiFi/Core attempts and keeps only BLE local fallback.
- UX safety closure in this pass:
  - iOS contacts list now requires explicit confirmation before contact deletion.
- Verification in this pass:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.data.MeshRepositoryTest"` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`10 warnings`, non-fatal policy)
- Remaining closure gate:
  - synchronized physical Android+iOS evidence is still required to retire `R-WS12-29-02` and paired-delivery residuals (`R-WS12-04/05/06`).

### WS12.34 Transport Failure Triage + 10-Fix Reliability Sweep (2026-03-04 HST)

- Field issue intake addressed in this pass:
  - iOS and Android messaging stopped working after toggling WiFi/BLE/cell connections.
  - Rust core `receive_message` failures were invisible on mobile due to swallowed `tracing` output.
  - iOS relay flapping threshold was self-triggering, permanently blocking relay circuit path.
  - Stale routing data caused infinite retry loops against unreachable peers.
  - Messages were being expired/dropped despite "never fail delivery" philosophy.
- Fixes applied (10 total across Rust core, iOS, Android):
  1. **`eprintln!` error visibility** (Rust core) — `receive_message` errors now visible on mobile platforms via stderr.
  2. **`relayEnabled` nil-safety** (iOS + Android) — relay toggle checks no longer produce nil, preventing silent message drops.
  3. **Retry throttle 500→2000ms** (iOS) — reduces main thread pressure during outbox flush.
  4. **Relay diagnostic throttle** (iOS) — 90% reduction in relay-state logging when flapping.
  5. **Messages NEVER expire** (iOS + Android) — removed attempt limits and age-based expiry from outbox.
  6. **Progressive backoff** (iOS + Android) — retry delay: `min(2^attempt, 60)` seconds, capping at 5 minutes for long-lived items.
  7. **WiFi recovery → immediate outbox flush** (iOS) — network path change triggers immediate pending message delivery.
  8. **WiFi recovery → immediate outbox flush** (Android) — `notifyNetworkRecovered()` triggers flush on WiFi restoration.
  9. **BLE 15s connection timeout** (Android) — stale GATT connections auto-cleaned after 15 seconds.
  10. **Dial candidate cap (6 max)** (iOS + Android) — prioritizes LAN → relay → public IPs, reduces stale-address dial spam.
- Core philosophy enforcement:
  - Messages NEVER expire. No attempt limit, no age limit, no TTL.
  - All messages retry indefinitely with progressive backoff until delivered.
  - Network recovery triggers immediate delivery attempts.
- Verification in this pass:
  - `cargo check --workspace` — **pass**
  - Rust core compiles with `eprintln!` diagnostics active.
- Remaining closure gate:
  - Deploy Rust core + both mobile apps and observe `eprintln!` output to diagnose any remaining `receive_message` failures.
  - Confirm end-to-end message delivery across all transport layers post-fix.

### WS12.35 Non-Device Reliability Reconciliation (2026-03-06 UTC)

- Baseline/CI correlation in this pass:
  - `cargo check --workspace` — **pass**
  - `cargo test --workspace --no-run` — **initial fail** (WASM `MessageRecord` test initializers missing `sender_timestamp`), then **pass** after fix.
  - `./scripts/docs_sync_check.sh` — **pass**
  - Latest failed non-`action_required` CI run inspected (`22706811148`, `CI`): blocker set matched local drift (`scmessenger-wasm` E0063 + iOS MainActor isolation in `MultipeerTransport` + Android `MeshRepositoryTest` null-settings expectations).
- Minimal reliability fixes applied:
  - WASM tests updated to include `sender_timestamp` in all `MessageRecord` initializers touched by desktop role/parity suites.
  - Core receipt verification now requires outbound-recipient correlation for receipt sender identity (accepting canonical recipient identity/public-key forms) so forged third-party receipts are ignored without regressing valid delivery receipts.
  - iOS `MultipeerTransport` now bridges repository diagnostics/identity snippet calls through MainActor-safe helpers to avoid synchronous nonisolated actor violations.
  - iOS `ChatViewModel` + `SettingsViewModel` explicitly annotated `@MainActor` for Swift concurrency correctness in UI-bound repository calls.
  - Android `MeshRepositoryTest` now matches canonical runtime semantics (`relayEnabled` defaults to enabled when settings are unavailable).
- WS12.24 deterministic gate reconciliation in this pass:
  - `scripts/run5-live-feedback.sh` already enforces `verify_delivery_state_monotonicity.sh` alongside `verify_receipt_convergence.sh`; this gate is now treated as canonical closure flow.
- Receipt guard regression tests in this pass:
  - `cargo test -p scmessenger-core test_delivery_receipt_marks_history_and_outbox_delivered -- --nocapture` — **pass**
  - `cargo test -p scmessenger-core test_mismatched_sender_receipt_is_ignored -- --nocapture` — **pass**
- WS12.29 diagnostics workflow hardening in this pass:
  - `scripts/run5-live-feedback.sh` iOS diagnostics pull now retries `devicectl copy` and requires near-stable file-size confirmation across pulls before accepting capture, failing fast when non-truncation cannot be confirmed.
  - Follow-up hardening: one-shot mode (`IOS_DIAG_PULL_ATTEMPTS=1`) now accepts a valid non-empty pull, and failed stability runs remove the untrusted output file before returning non-zero.
- Environment limitations observed:
  - Android Gradle unit-test run in this environment failed before tests due blocked dependency fetch from `dl.google.com` (host/network prerequisite).
  - `bash ./iOS/verify-test.sh` could not execute here (`xcodebuild` unavailable on this host), so iOS physical/simulator runtime closure gates remain unchanged.

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

### WS12.6 Optional Closeout Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test -p scmessenger-core relay_custody -- --nocapture` — **pass**
- `cargo test -p scmessenger-core convergence_marker -- --nocapture` — **pass**
- v0.2.0 closeout outcomes:
  - relay custody persistence defaults to durable app-data paths (env override + OS-local fallback chain),
  - storage pressure enforcement now has synthetic snapshot fallback when platform probe data is unavailable,
  - convergence-marker application now requires validation + local tracking correlation,
  - workspace/app version metadata bumped to `0.2.0` for release synchronization.

### WS12.7 Live Runtime Sanity Snapshot (2026-03-02 HST)

- Live runtime/debug commands:
  - `adb logcat --pid=$(adb shell pidof -s com.scmessenger.android) -T 1 -v threadtime`
  - `xcrun simctl spawn booted log show --style compact --last 10m --predicate 'process == "SCMessenger"'`
  - `adb shell run-as com.scmessenger.android cat files/pending_outbox.json`
  - `xcrun simctl get_app_container booted SovereignCommunications.SCMessenger data`
- Build verification after runtime fixes:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -configuration Debug -sdk iphonesimulator -destination 'platform=iOS Simulator,name=iPhone 16e' build CODE_SIGNING_ALLOWED=NO` — **pass**
- Observed runtime state (pre-fix logs):
  - Android live logs showed repeated `Core-routed delivery failed` / `Relay-circuit retry failed` while relay agent strings still included `scmessenger/0.1.0/headless/relay/*` (GCP rollout in progress).
  - Android pending outbox contained long-lived entries with very high retry counts (for example `attempt_count=2055`), consistent with no-give-up retry semantics.
  - Android emitted overlapping outbox flush runs (`reason=enqueue` and `reason=peer_identified`) with duplicate forwarding attempts for the same message in the same second.
  - Android `ServiceStats.uptimeSecs` remained `0` in repeated status emissions.
  - iOS simulator had no active pending-outbox backlog (`pending_outbox.json` = `[]`) during this pass.
- Runtime fixes applied in this pass:
  - Android: fixed BLE identity beacon fallback logic that previously overwrote non-empty listener/external hint payloads unconditionally.
  - Android: serialized pending outbox flush execution with a coroutine mutex to prevent duplicate concurrent retry passes.
  - Android: added uptime fallback when core-reported `uptimeSecs` is `0` while service is running.

### WS12.8 Runtime Recheck Snapshot (2026-03-02 HST)

- Live runtime/debug commands:
  - `adb devices -l` / `adb mdns services`
  - `xcrun simctl spawn booted log show --style compact --last 12m --predicate 'process == "SCMessenger" OR subsystem == "com.scmessenger"'`
  - `nc -z -w 5 34.135.34.73 9001`
  - `curl --max-time 8 http://34.135.34.73:9000`
  - `curl --max-time 8 http://34.135.34.73:8080`
  - `./target/debug/scmessenger-cli start` (interactive runtime probe)
  - `./scripts/verify_ws12_matrix.sh`
- Runtime observations:
  - Android device log streaming was blocked in this pass (`adb devices` empty; no mDNS-discoverable wireless endpoint).
  - iOS simulator process was active but reported only local Multipeer routing-table self state (`1 nodes`) in sampled logs.
  - GCP relay endpoint `34.135.34.73:9001` was reachable over TCP.
  - GCP relay landing page was reachable on `34.135.34.73:9000`; `:8080` timed out (current deploy script uses `--http-port 9000`).
  - CLI runtime probe observed relay identity rotation at `34.135.34.73:9001`: `12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw` -> `12D3KooWJaLtGyFYvobdZyecLWKA45cjSLEjzWtKPeorgeFYrsjZ`.
  - During the same probe, relay-circuit reservation warning remained active (`Could not register relay circuit reservation`), indicating a remaining runtime gap post-redeploy.
- Verification delta:
  - `./scripts/verify_ws12_matrix.sh` now fails on live suite `integration_relay_custody -- --include-ignored` with timeout at `core/tests/integration_relay_custody.rs:71`.
- Fixes applied in this pass:
  - Core: relay reservation address construction now canonicalizes identify addresses before appending `/p2p/<relay>/p2p-circuit` (`core/src/transport/swarm.rs`).
  - Core: relay reservation warning logging now emits `Debug` error detail instead of potentially empty display text (`core/src/transport/swarm.rs`).
  - Test hardening: recipient-side custody test flow no longer gates on a pre-delivery peer-readiness drain before waiting for envelope delivery, and uses a larger delivery wait budget (`core/tests/integration_relay_custody.rs`).

### WS12.9 iOS Dashboard Node Count Hotfix (2026-03-03)

- Issue context:
  - iOS diagnostics/runtime checks showed `Peers Discovered` values were correct, but dashboard node totals could overcount due to stale online state and alias-key duplication (canonical/libp2p/BLE identifiers represented separately).
- Fixes applied:
  - iOS dashboard node counters now derive from online-only deduplicated peers (`full`/`headless` counts no longer include stale offline entries).
  - iOS dashboard final merge now deduplicates by alias graph (`id`, `peerId`, `libp2pPeerId`, `blePeerId`, `publicKey`) to collapse duplicate rows for the same identity.
  - iOS refresh merge no longer blindly preserves historical online state; prior online state now decays by recency guard before being retained.
- Code path:
  - `iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift`

### WS12.10 Runtime Re-baseline + Action Roundup (2026-03-03 HST)

- Live verification commands:
  - `ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./scripts/verify_ws12_matrix.sh` — **pass**
  - `cargo test -p scmessenger-core --test integration_relay_custody offline_recipient_receives_after_reconnect_without_sender_resend -- --include-ignored --exact` (3 consecutive runs) — **pass/pass/pass**
  - `adb kill-server && adb start-server && adb mdns services && adb devices -l`
  - `adb logcat -d | rg "MeshRepository|delivery_state|relay|custody|swarm"`
  - `xcrun simctl spawn booted log show --style compact --last 60m --predicate 'eventMessage CONTAINS[c] "NSFileManager" OR eventMessage CONTAINS[c] "createDirectory"'`
  - `bash ./iOS/verify-test.sh` — **pass** (74 warnings, non-fatal per script policy)
- Runtime findings:
  - Custody reconnect gate is now stable in this environment (3/3 consecutive passes).
  - Android live logs were successfully captured after reconnect and showed active `scmessenger/0.2.0/headless/relay/*` peers; wireless ADB visibility later dropped again after daemon restart, so endpoint persistence remains an operational follow-up.
  - iOS runtime issue warnings were reproducible and attributable to app startup path (`NSFileManager createDirectory*` on main actor in `MeshRepository.init()`), not simulator-only noise.
  - Post-fix quick-launch probe (`xcrun simctl launch booted SovereignCommunications.SCMessenger` + 2-minute log window) showed no new `createDirectory` runtime-issue entries for SCMessenger.
  - Fresh CLI runtime probe did not reproduce `Could not register relay circuit reservation` warnings in this pass.
- Fixes applied:
  - iOS: removed main-actor storage directory creation from `MeshRepository.init()` and moved diagnostic file persistence to a background serial queue (`iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`).
  - iOS: fixed dashboard compile regression by passing `Array(merged.values)` into alias dedup helper (`iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift`).

### WS12.13 Wave-2 Backlog Consolidation Snapshot (2026-03-03 HST)

- Validation/debt reconciliation commands:
  - `cargo check --workspace` — **pass**
  - `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew :app:generateUniFFIBindings` — **pass**
  - `bash iOS/copy-bindings.sh` — **pass**
  - `ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk bash ./verify_integration.sh` — **pass**
  - `bash ./verify_simulation.sh` — **expected fail-fast** (Docker unavailable in this environment)
  - `cd wasm && wasm-pack build` — **pass** (with release `wasm-opt` disabled in `wasm/Cargo.toml` for host compatibility)
- Tooling adjustments in this wave:
  - `verify_integration.sh` was modernized to delegate to canonical `scripts/verify_ws12_matrix.sh` instead of stale grep-pattern checks that were producing false negatives.
  - `verify_simulation.sh` no longer attempts automatic Docker installation and now exits with explicit operator guidance when Docker is not preinstalled/running.
- Backlog-governance outcome:
  - Non-historical mixed docs were reclassified from open checkboxes to status-tagged guidance/roadmap entries (`FEATURE_WORKFLOW.md`, `AUDIT_QUICK_REFERENCE.md`, `FEATURE_PARITY.md`, `DRIFTNET_MESH_BLUEPRINT.md`, `docs/TRANSPORT_ARCHITECTURE.md`).
  - `docs/TRANSPORT_ARCHITECTURE.md` future enhancements now include explicit owner/milestone/gate/acceptance metadata.
- Post-update issue-slate evidence triage (live artifacts, no code edits):
  - Android live watch (`/tmp/scm_android_live_watch.log`) captured `BluetoothGatt` callback exceptions during BLE fallback writes (`IllegalStateException: The number of released permits cannot be greater than 1` in `BleGattClient.releaseGattOp`), alongside mixed "write successful" callbacks for the same peer window.
  - Android same window retained repeated `Core-routed delivery failed ... Network error` and `Relay-circuit retry failed ... Network error` with stalled `messagesRelayed=0`, reinforcing unresolved delivery convergence risk.
  - iOS live watch (`/tmp/scm_ios_live_watch.log`) captured high-churn Multipeer sessions (`Connection attempt in progress` on many channels, followed by repeated `Timed out, enforcing clean up` and `Disconnected` transitions), consistent with local session instability symptoms observed during relay flapping runs.
  - These findings were classified into "possibly in-flight" versus "likely still open" TODO buckets in `REMAINING_WORK_TRACKING.md` WS12.13 section for immediate post-update validation.

### WS12.11 iOS Relay Flapping Diagnosis Snapshot (2026-03-03 HST, no code edits)

- Live/runtime evidence reviewed:
  - iOS diagnostics (`ios_diagnostics_latest.log`) show repeated relay rediscovery and repeated relay-circuit dial attempts to bootstrap endpoints (`34.135.34.73:9001` and `104.28.216.43:9010`) in short intervals.
  - Same windows contain repeated `peer_identified` churn for relay agents, including headless relay identities reappearing multiple times per minute.
  - Runtime logs include `dial_throttled` events interleaved with new dial attempts, indicating retry pressure rather than stable session hold.
  - Prior GCP-side mesh logs (`logs/5mesh/gcp.log`) show disconnect/reconnect oscillation (`Lost relay peer ... scheduling reconnect with backoff`) for the same peers, consistent with cross-side instability rather than iOS-only UI artifact.
- Diagnosis outcome (current confidence: medium-high):
  - iOS "relay appears/disappears" behavior is reproducible and consistent with transport-session churn plus repeated identify/dial cycles.
  - No direct crash evidence was found in this pass; primary symptom is flapping connection state and redundant relay rediscovery.
  - Most likely contributors are state-churn/race interactions between repeated relay bootstrap priming and concurrent route-based connect attempts, amplified under unstable relay/session conditions.
- No-code-change constraints honored:
  - This run was documentation/diagnosis only; no source edits were applied to transport/runtime code.

### WS12.12 Android<->iOS Pairing Message Non-Delivery RCA (2026-03-03 HST, no code edits)

- Runtime evidence reviewed:
  - Android device diagnostics (`run-as com.scmessenger.android ... files/mesh_diagnostics.log`) repeatedly show:
    - `Core-routed delivery failed ... Network error; trying alternative transports`
    - `Relay-circuit retry failed ... Network error`
    - message state cycling `forwarding -> stored` with `awaiting_receipt_delay_sec=8` and rising retry attempts.
  - In the same windows Android logs repeatedly emit `✓ Delivery via BLE (target=...)` immediately followed by `Failed to initiate characteristic write ...` while also emitting characteristic-write-success callbacks.
  - Android stats remain effectively stalled for delivery (`messagesRelayed=0`) during these retries.
  - iOS diagnostics history (`ios_diagnostics_latest.log`) and prior relay logs show heavy relay dial/identify churn and throttling, consistent with unstable internet-route availability during pairing runs.
- RCA conclusion:
  - Primary failure mode is end-to-end delivery confirmation failure, not pairing absence: devices discover each other, but routed sends fail over internet (`Network error`) and BLE fallback does not converge to recipient receipt.
  - Most probable root-cause cluster is transport-state inconsistency in Android BLE send path under fallback load (conflicting write initiation/result signals) combined with relay route instability.
  - Secondary contributing factor: legacy pending outbox items with very high retry counts keep retry pressure high, obscuring fresh-message behavior and increasing contention.
- No-code-change constraints honored:
  - This pass performed diagnosis and documentation only; no implementation edits were applied.

### WS12.14 Android Bluetooth-Only Pairing Diagnosis (2026-03-03 HST, no code edits)

- Runtime evidence reviewed:
  - Android USB+ADB logcat during "Bluetooth-only" run showed sustained BLE stack churn for the iOS peer address with repeated `BluetoothRemoteDevices: Address type mismatch ... new type: 1`.
  - In the same window, Android app telemetry dropped to `Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)` and `NearbyMediums: No BLE Fast/GATT advertisements found in the latest cycle`.
  - iOS logs during the same interval showed repeated Multipeer invitation timeouts/declines (`Invite timeout`, `Peer ... declined invitation`) and session resets.
  - iOS multipeer connection attempts in these traces reported `transportType=WiFi` (`interfaceName=en0`) rather than an explicit BLE-only transport hold.
- RCA conclusion (current confidence: high):
  - The requested Android<->iOS Bluetooth-only path is not converging to a stable BLE data path in this run.
  - Primary symptoms point to transport-path mismatch and BLE identity/address instability: Android repeatedly reclassifies peer address type while iOS multipeer flow repeatedly times out and appears to favor WiFi-backed session attempts.
  - Resulting behavior is consistent with fallback churn rather than sustained BLE-only connectivity, so message exchange fails before reliable send/receipt convergence.
- No-code-change constraints honored:
  - This pass was diagnosis/documentation only; no transport implementation edits were made.

### WS12.16 Wave-2 Runtime Hardening Pass (2026-03-03 HST)

- Verification commands:
  - `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass**
  - `cargo check --workspace` — **pass**
- Fixes delivered:
  - Android BLE callback/permit race hardening in `BleGattClient`:
    - single-release permit guard for queued GATT operations,
    - overflow-safe semaphore release handling,
    - `WRITE_TYPE_NO_RESPONSE` callback path treated as informational to avoid contradictory final write outcomes.
  - Android+iOS per-message `delivery_attempt` diagnostics timeline now emitted for local fallback, core direct route, relay-circuit retry, and aggregate terminal outcome with message ID context.
  - iOS relay-flap visibility and guardrails in `MeshRepository`:
    - relay dial debounce and bootstrap in-progress guard,
    - relay availability state export (`stable`/`flapping`/`backoff`/`recovering`) with timestamps and event counters,
    - relay timeline markers for identify/disconnect/dial attempt outcomes keyed to relay peer IDs.
  - iOS Multipeer channel-storm guardrails in `MultipeerTransport`:
    - invite debounce,
    - in-flight invite dedupe,
    - concurrent invite cap,
    - timeout/decline diagnostics counters.
- Remaining wave-2 live-evidence gates:
  - Re-run synchronized Android+iOS+relay live probe and confirm reduced relay/multipeer churn plus receipt convergence for both send directions.
  - Capture synchronized BLE-only and internet-degraded artifact bundles with message ID timeline continuity for residual-risk closure.

### WS12.17 Wave-3 Governance + Runtime Closure Sweep (2026-03-03 HST)

- Runtime/code updates applied:
  - Android BLE address-type mismatch mitigation now includes reconnect cooldown/backoff and skip counters in `BleGattClient`.
  - Android+iOS strict BLE-only validation mode and diagnostics export fields are active (`strict_ble_only_validation` markers).
  - Android BLE discovery/client counters and iOS Multipeer diagnostics snapshot counters are exported for operator triage.
- New deterministic harnesses added and executed:
  - `./scripts/correlate_relay_flap_windows.sh ios_diagnostics_latest.log logs/5mesh/gcp.log` — classified sampled pair as `unsynchronized_artifacts_no_time_overlap`.
  - `./scripts/verify_relay_flap_regression.sh ios_diagnostics_latest.log` — pass (no deterministic relay dial-loop regression for sampled artifact).
  - `./scripts/verify_receipt_convergence.sh android_mesh_diagnostics_device.log ios_diagnostics_latest.log` — no message IDs in sampled historical artifacts.
  - `./scripts/verify_ble_only_pairing.sh android_logcat_latest.txt ios_diagnostics_latest.log` — no strict BLE-only markers in sampled historical artifacts.
- Validation commands:
  - `cd android && ANDROID_HOME=/Users/christymaxwell/Library/Android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd wasm && wasm-pack build` — **pass**
- Documentation/backlog governance outcomes:
  - Historical open-checkbox sources were triaged with explicit status tags in `docs/historical/*` and are no longer active checklist noise.
  - `docs/ALPHA_RELEASE_AUDIT_V0.1.2.md` version-bump/redeploy steps were explicitly marked as historical closeout and superseded by v0.2.0 release-sync docs.
  - Final checklist inventory after wave-3 triage: 10 open checklist items repo-wide, all in `REMAINING_WORK_TRACKING.md` (no historical open checkboxes remain).

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
  - Result: **available** (`wasm-pack 0.14.0`)
  - `cd wasm && wasm-pack build` — **pass**

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
- First-run install-mode choice restored on GUI variants (iOS/Android/Desktop-WASM): users can initialize identity immediately or skip into relay-only mode, then create identity later from Settings -> Identity without reinstall
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

- Browser-executed WASM tests require local `wasm-pack`; CI enforces this path in `.github/workflows/ci.yml` (`check-wasm`).
- Android build verification requires `ANDROID_HOME` to be set in-shell; CI now standardizes SDK env and enforces Android preflight in `.github/workflows/ci.yml` (`check-android`).
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
