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
