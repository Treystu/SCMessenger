# SCMessenger Remaining Work Tracking

This is the active implementation backlog based on repository state verified on **2026-02-23**.

Primary delivery target: **one unified Android + iOS + Web app**.

Owner policy constraints (2026-02-23):

- Global organic growth (no region-targeted rollout sequence).
- Community-operated infrastructure model (self-hosted and third-party nodes are both valid).
- English-only alpha UI language (i18n expansion tracked as backlog).
- No abuse-control or regional compliance hard gate for alpha.
- Anti-abuse controls are required before beta release.
- Critical UX controls must stay in Android+iOS+Web parity with no temporary lead platform.

## Priority 0: Tri-Platform Semantics and Reliability

1. [x] Privacy parity-first wiring (all toggles) on Android, iOS, and Web
   - Outcome: All four privacy toggles (onion routing, cover traffic, message padding, timing obfuscation) are now implemented with live UI bindings on Android (`SwitchSetting`), iOS (`Toggle`), and Web/WASM (`getSettings`/`updateSettings`). Dead placeholder components removed from both mobile platforms.

2. [x] Relay toggle enforcement parity (mandatory OFF behavior on all clients)
   - Outcome: WASM `prepareMessage`, `receiveMessage`, and WebSocket receive loop now enforce `relay_enabled` check, matching Android/iOS behavior. When OFF, outbound messages are blocked and inbound frames are dropped.

3. [x] Canonical identity normalization to `public_key_hex`
   - Outcome: `IdentityInfo` struct and `api.udl` now document `public_key_hex` as the canonical persisted/exchange identity. `identity_id` (Blake3) and `libp2p_peer_id` are documented as derived/operational metadata.

4. [x] Bootstrap configuration model implementation
   - Outcome: Added `BootstrapConfig` dictionary and `BootstrapResolver` interface to `api.udl` with full Rust implementation. Resolution chain: env override (`SC_BOOTSTRAP_NODES`) → remote URL fetch (via `ureq` HTTP client) → static fallback list. Android and iOS both wired to use resolver instead of hardcoded lists. WASM uses env → static path (no sync HTTP in browser).

5. Android peer discovery parity hardening
   - Source: `ANDROID_DISCOVERY_ISSUES.md` investigation notes.
   - Open items:
     - protocol negotiation disconnect loops (`Failed to negotiate transport protocol(s)`)
     - bootstrap relay visibility policy in nearby UI
     - delayed identity-resolution retry path after initial peer connect

6. Real-network NAT traversal field matrix
   - Scope: CLI host nodes + Android + iOS + Web over mixed LAN/WAN/NAT.
   - Target: scripted verification matrix with delivery latency + fallback success criteria.

7. Nearby Peer Discovery and Identity Federation (Android Focus)
   - [x] Prevent permission-race startup regression: Android mesh now permission-gates BLE/WiFi init and auto-retries transport init when runtime permissions are granted (no restart required).
   - [x] Ensure Bluetooth, LAN, and Relay discovery are accounted for and routed to Mesh tab.
   - [x] Display total node count (headless and full) in Mesh UI.
   - [x] Fix nickname federation (ensure nicknames are correctly passed to neighbors over BLE/Swarm).
   - [x] Fix iOS -> Android nearby identity nickname propagation (Android currently discovers peer identity/public key but often misses federated nickname).
   - [x] Implement local nickname overrides in contacts (show both official and private nicknames).
   - Outcome (2026-02-24):
     - Android and iOS repositories now emit deduplicated identity/connected discovery events for BLE + internet peers, including headless relay visibility.
     - Dashboard surfaces aggregate full/headless totals from canonical discovery state.
     - BLE identity reads now perform delayed refresh pulls after initial connect to capture nickname updates quickly.
     - Contacts screens display local override nickname as primary with federated nickname retained as secondary (`@nickname`) on both mobile clients.

8. Android WiFi Aware physical-device validation
   - File: `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`
   - Target: compatibility results by Android version/device class with documented pass/fail outcomes.

9. [x] Web parity promotion — WASM swarm transport and API parity
   - Previous: Web/WASM was functionally present but thinner than mobile app surfaces.
   - Completed (PR #48):
     - WASM now uses libp2p swarm via `wasm-bindgen` + `websocket-websys` transport as first-class path (no standalone relay-only bypass).
     - `startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers` implemented in `wasm/src/lib.rs`.
     - Legacy `startReceiveLoop(relayUrl)` converted to deprecated shim that delegates to swarm bootstrap.
     - `ConnectionPathState` enum and `exportDiagnostics()` exposed in both UniFFI and WASM APIs for tri-platform parity.
     - CI adds `wasm32` compile checks to guard browser transport builds.
   - Remaining (beta):
     - IndexedDB-backed persistence with migration/version support.
     - `wasm-pack` runtime browser test coverage in CI.
     - History UX and deep parity for settings/contacts surfaces on Web.

10. Beta anti-abuse gate implementation and validation

- Requirement: abuse controls are non-blocking in alpha but mandatory before beta.
- Target: enable and validate anti-abuse protections with measurable pass criteria across Android, iOS, Web, and relay-critical paths.
- Scope: relay spam/flood controls, abuse detection thresholds, and regression coverage in CI/release checks.

11. Active-session reliability + durable eventual delivery guarantees
    - Requirement: while app is open/relaying, service should remain available and messages should not be dropped.
    - Target: explicit durability contract (persisted outbox/inbox semantics, resend/recovery behavior) plus failure-mode tests.
    - Scope: crash/restart recovery, relay outage handling, offline queue replay, duplicate-safe redelivery.

12. Bounded retention policy implementation

- Requirement: local history/outbox storage must be policy-bound to avoid unbounded disk growth.
- Target: configurable retention caps + deterministic pruning behavior + docs for user expectations.
- Scope: Android, iOS, and Web local storage behavior and defaults.

13. First-run consent gate (mandatory)

- Requirement: first app launch must present consent text explaining privacy/security boundaries.
- Target: consent acknowledgment gate on Android/iOS/Web before first messaging actions.
- Scope: UX copy parity, acceptance persistence, and re-display rules after major policy changes.

14. 80/20 platform support matrix

- Requirement: prioritize the smallest support matrix that covers the majority of active users.
- Target: explicit minimum OS/browser matrix and validation plan tied to release gates.
- Scope: Android API levels, iOS versions/devices, and browser families/versions.

15. Community-operated relay/bootstrap topology support

- Requirement: both self-hosted and third-party-operated infra must be valid without protocol-level assumptions.
- Target: operator docs + connectivity tests for cloud-hosted and home-hosted relays/bootstrap nodes.
- Scope: examples for GCP-style deployments and low-resource/self-hosted setups.

16. Bootstrap governance mode decision (product choice pending)

- Requirement: choose how clients trust and discover bootstrap updates.
- Target: lock one governance mode and document it in canonical docs.
- Scope: trust source, update cadence, and fallback behavior.
- Outcome (2026-02-25): Registered newly identified peers as potential relays in the reputation tracker to expedite relay connectivity.

17. Fast Bootstrap and Graceful Identity Handling

- Requirement: Support hardcoded or dynamically updated IPs for bootstrap nodes without mandatorily hardcoding their peer identities.
- Target: Allow the mesh service to gracefully accept the new or changing identity of a static-IP bootstrap node instead of failing the connection layout or validation.
- Scope: Refactor connection validation / Noise payload handling so that a known static bootstrap IP can dynamically rotate or present any valid peer identity without breaking clients.

17. Multi-Transport Reliability and Targeted Acknowledgements

- Requirement: replies and metadata sync must not fail when peers move between LAN, BLE, and Internet (GCP Relay).
- Outcome (2026-02-25):
  - [x] Switched delivery receipts and identity sync from broadcast to targeted delivery (Multi-Path), ensuring they reach off-LAN peers via Relay or BLE.
  - [x] Implemented platform-level BLE fallback in `attemptDirectSwarmDelivery` for both Android and iOS, prioritizing LAN → BLE → Relay.
  - [x] Linked canonical identities to `ble_peer_id` and `libp2p_peer_id` in persisted contact notes to maintain routing across sessions.
  - [x] Verified GCP relay (34.135.34.73:9001) is alive and accepting connections.

18. Parity: Data Deletion (Contacts and Message Threads)

- Requirement: Ensure complete parity across all instances (Android, iOS, Web) for deleting a contact and deleting a message thread.
- Target: Allow users to securely remove contacts and clear entire message threads, ensuring changes are fully persisted and reflected in the UI.
- Scope: Bind deletion operations in `ContactsManager` and `HistoryManager` to UI interactions on all platforms, including cleaning up associated metadata.

19. [ ] Headless/Relay logic Refinement
    - [x] Update `IronCoreBehaviour::new` to accept `headless` boolean flag and incorporate it into the `agent_version` string.
    - [x] Update `start_swarm` and `start_swarm_with_config` in `core/src/transport/swarm.rs` to accept and pass down the `headless` flag.
    - [x] Adjust calls to `start_swarm` in `cli/src/main.rs`: `cmd_start` passes `false`, and `cmd_relay` passes `true`.
    - [x] Update `MeshService::start_swarm` in `core/src/mobile_bridge.rs` to pass `false`.
    - [x] Update `CoreDelegate` trait and `api.udl` to include `agent_version` in `on_peer_identified`.
    - [x] Update Android `MeshRepository.kt` to handle `agentVersion` and identify headless peers.
    - [x] Update iOS `CoreDelegateImpl.swift` and `MeshRepository.swift` to handle `agentVersion` and identify headless peers.
    - [ ] Confirm that direct P2P messaging works over cellular with fallback to relaying (mandatory for 0.1.2 Alpha).

## Priority 1: Tooling, CI, and Experimental Surface

1. Align CI with tri-platform target status
   - Current: `.github/workflows/ci.yml` validates Rust workspace on Linux/macOS only.
   - Gap: no canonical Android+iOS+Web parity gates in the primary CI workflow.
   - Target: enforce Android+iOS+Web build-readiness checks in gating CI for mainline changes.

2. Add browser-executed WASM test job (parity gate)
   - Current: native/non-browser WASM tests only in workspace run.
   - Target: `wasm-pack` runtime test coverage in CI.

3. [x] Resolve integration test warnings in core tests
   - Current: workspace tests pass with warning noise.
   - Target: warning-clean path for strict CI.
   - Outcome: Cleaned up unused assignments and unused variables across all integration suites. Unit and integration tests are 100% warning-clean.

4. Standardize Android CI environment setup for `ANDROID_HOME`
   - Current: local build requires explicit shell env setup.
   - Target: consistent CI env bootstrap and preflight enforcement.

5. iOS legacy tree cleanup policy
   - Active app lives in `iOS/SCMessenger/SCMessenger/`.
   - `iOS/SCMessenger-Existing/` should be explicitly retained as archive/reference or removed once migration confidence is complete.

6. [x] Docker test/ops script consistency cleanup
   - Current: mixed compose filename references and stale command paths across `docker/*.sh` and docs.
   - Target: one canonical compose naming set and verified command examples that match checked-in files.
   - Outcome: Normalized all references to use canonical compose naming (`docker compose` CLI standard and `docker-compose*.yml` filename format without spaces).

7. CLI surface normalization for long-term dependability
   - Current: `cli/src/main.rs.backup` and mixed identity/public-key field naming remain in the CLI surface.
   - Target: remove backup artifacts from runtime path, align CLI identity/contact semantics with canonical `public_key_hex`, and revalidate relay/bootstrap controls.

8. Reference artifact hygiene
   - Current: `reference/Androidlogs.txt` includes non-SCMessenger application logs; `reference/` mixes active porting guides with raw captures.
   - Target: isolate SCMessenger-specific evidence logs and keep reference crypto sources clearly separated from runtime diagnostics.

9. [x] Android test execution truthfulness cleanup
   - Current: `android/app/src/test/README.md` says previously `@Ignored` tests are enabled, but `android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt` still contains broad `@Ignore` usage.
   - Target: either enable those tests with stable mocks or update docs/scripts to match actual execution status.
   - Outcome: Updated `android/app/src/test/README.md` to truthfully explain that UniFFI MockK limitations natively prevent complete CI verification for generated files, serving as architectural documentation pending a stable JNA harness setup instead.

10. [x] Android ABI and verification script alignment

- Current: `android/app/build.gradle` and `buildRustAndroid` are aligned on `arm64-v8a` + `x86_64`, but `android/verify-build-setup.sh` still checks for legacy extra Rust targets (`armv7`, `i686`).
- Target: align environment verification script with actual supported ABI matrix and documentation.
- Outcome: `android/verify-build-setup.sh` now validates only `aarch64-linux-android` and `x86_64-linux-android`, and install guidance was updated to match the supported ABI matrix.

11. [x] Core settings model convergence (critical reliability debt)

- Current: multiple overlapping settings models diverge in defaults/semantics:
  - `core/src/mobile_bridge.rs` (`MeshSettings`, DiscoveryMode `Normal/Cautious/Paranoid`)
  - `core/src/mobile/settings.rs` (`MeshSettings`, DiscoveryMode from transport layer)
  - `core/src/platform/settings.rs` (`MeshSettings`, DiscoveryMode `Open/Closed/Stealth`)
- Target: one canonical settings schema and mapping strategy used by UniFFI/mobile/runtime layers.
- Outcome: Deleted the unused `mobile/settings.rs` and `platform/settings.rs` completely. Unified purely behind the single UniFFI-verified `mobile_bridge::MeshSettings` exported transparently through `api.udl`. Web clients will default to "always plugged in" behavior via this schema.

12. [x] iOS verification script hardening

- Current: `iOS/verify-test.sh` now performs simulator build verification.
- Target: harden script behavior (deterministic destination selection, warning handling policy, and explicit failure output) for stable CI/operator use.
- Outcome: `iOS/verify-test.sh` now uses strict shell flags, deterministic `generic/platform=iOS Simulator` destination, explicit failure handling, and an explicit warning count policy.

13. [x] iOS background capability hardening

- Current: `iOS/SCMessenger/SCMessenger/Info.plist` declares a broad background mode set.
- Target: retain only modes required by implemented behavior and provisioning policy; remove speculative extras.
- Outcome: removed speculative `location` and `remote-notification` background modes and removed unused location/motion permission strings from `Info.plist`; retained BLE + fetch + processing modes used by implemented services.

14. [ ] iOS power settings runtime observability and enforcement verification

- Current: explicit runtime logging/enforcement hooks are now wired in `MeshRepository` (`setAutoAdjustEnabled`, `applyPowerAdjustments`, and profile-application logs across battery/network/motion updates), and Settings toggle now drives repository state directly.
- Remaining: capture active-session device evidence confirming power profile transitions under real motion/network/battery changes.
- Follow-up: simplify iOS power UX to a single automatic mode and remove manual Low/Standard/High style overrides; drive gradual adaptation from battery %, bandwidth quality, and latency measurements.

15. [x] iOS generated-binding path normalization

- Current: `iOS/copy-bindings.sh` wrote generated files into both `iOS/SCMessenger/SCMessenger/Generated/` and `iOS/SCMessenger/Generated/`.
- Target: one canonical generated artifact path tied to active Xcode targets and docs.
- Outcome: `iOS/copy-bindings.sh` now writes only to `iOS/SCMessenger/SCMessenger/Generated/`, which matches active Xcode target paths.

16. iOS historical artifact segmentation

- Current: `iOS/iosdesign.md` and `iOS/SCMessenger/build_*.txt` mix design/historical/runtime evidence in active tree.
- Target: section-level historical tagging and relocation/retention policy to keep active docs concise.

17. TODO/FIXME accuracy sync pass (including external test/update signals)

- Current: TODO/FIXME markers are distributed across code/docs; external testing updates can drift from tracked backlog.
- Target: recurring TODO/FIXME audit that syncs canonical backlog items with current implementation evidence.
- Evidence source: `docs/TRIPLE_CHECK_REPORT.md` risk scan + direct file review.
- Companion reference: `docs/STUBS_AND_UNIMPLEMENTED.md` — comprehensive stub/placeholder inventory (43 items across 4 severity tiers).

18. [x] Android multi-share intent handler — full implementation with IntentCompat

- File: `android/app/src/main/java/com/scmessenger/android/utils/ShareReceiver.kt`.
- History: stub was originally removed (prior outcome); PR #48 added a complete working implementation.
- Outcome: `ShareReceiver` now handles `ACTION_SEND_MULTIPLE` with `IntentCompat.getParcelableArrayListExtra()` for API < 33 compatibility (no `NoSuchMethodError` crash on Android 12 and below). Multi-stream URI items are forwarded correctly alongside text items.

19. [x] App-update persistence migration hardening (identity, contacts, message history)

- Requirement: app upgrades must preserve identity, contacts, and message history without manual re-import.
- Target: deterministic migration/verification path across Android and iOS app updates, including storage-path continuity checks and automatic import fallback for legacy stores.
- Scope: core storage versioning, mobile app startup migration hooks, and update smoke tests that assert post-update continuity.
- Completed:
  - Added core storage layout/schema guard (`SCHEMA_VERSION`) and explicit `identity/`, `outbox/`, `inbox/` sub-store initialization.
  - `IronCore::with_storage()` now initializes persistent inbox/outbox backends (not memory-only fallback by default).
  - Added core persistence restart tests for inbox/outbox continuity under storage-backed initialization.
  - Added schema v2 legacy-root migration to copy old identity/outbox/inbox keys into split sub-stores on upgrade.
  - Identity manager now hydrates persisted identity/nickname on startup without auto-generating fresh identities.
  - Added restart continuity tests for identity hydration, legacy-root migration, contacts (including local nickname), and history delivery-state persistence.
  - Android onboarding now waits for confirmed identity creation + nickname persistence before completing first-run flow.
  - Android/iOS repository flows now explicitly resume deferred swarm startup after identity/nickname creation, closing a first-run internet transport stall path.
  - CLI relay mode now uses persisted headless network identity (`storage/relay_network_key.pb`) so relay peer IDs remain stable across process restarts; key migrated from existing IronCore identity on first upgrade to preserve `/p2p/` bootstrap addresses.
  - Identity backup export/import implemented via iOS Keychain and Android SharedPreferences (`identity_backup_prefs.xml`); survives full reinstall with no manual re-import.
  - `mark_message_sent(message_id)` added to `IronCore` and exposed via UniFFI; mobile clients call it after confirmed ACK to keep outbox bounded (prevents "outbox full" stall on long-lived accounts).
  - Key material zeroized after use in both `export_identity_backup` and `import_identity_backup` (even on error path).
  - Android `allowBackup="true"` + `dataExtractionRules` + `fullBackupContent` wired in `AndroidManifest.xml`; `backup_rules.xml` fixed (removed `<include>` that silently disabled default backup).
  - BLE GATT sequential operation queue implemented (`Channel<() -> Unit>` + `Semaphore(1)` per device); all reads, writes, and CCCD writes serialised; stale-session guard on refresh reads.
  - `cargo clippy --workspace` clean; `cargo fmt --all` clean; 5 new core unit tests for backup roundtrip, validation errors, and `mark_message_sent` behaviour.
- Remaining (validation only — no code changes needed):
  - Platform-level upgrade simulations on Android/iOS/WASM package installs with real prior-app data.
  - End-to-end package upgrade evidence capture (device install/update logs + retained chat transcript checks).

## Priority 2: Documentation Completion and Governance

1. Full-file documentation pass completion using `docs/DOC_PASS_TRACKER.md` (completed)
   - Current: all tracked files are reviewed (`pending` = 0, checked = 356).
   - Ongoing target: keep this at 0 pending via delta checks on new/changed files.

2. Historical-heavy docs section-status sweep
   - Requirement: stale/current components tagged at section level (`[Current]`, `[Historical]`, `[Needs Revalidation]`) with canonical pointers.

3. Keep canonical chain authoritative
   - `README.md`
   - `DOCUMENTATION.md`
   - `docs/REPO_CONTEXT.md`
   - `docs/CURRENT_STATE.md`
   - `REMAINING_WORK_TRACKING.md`
   - `docs/GLOBAL_ROLLOUT_PLAN.md`
   - `docs/STUBS_AND_UNIMPLEMENTED.md`

4. [x] Resolve `ios/` vs `iOS/` path-case split in tracked docs vs app source
   - Outcome: canonicalized documentation/script references to `iOS/` and recorded governance rule to prevent lowercase-path drift.

## Verified Stable Areas (No Active Gap)

- `cargo test --workspace` passes (343 passed, 0 failed, 7 ignored)
- `cargo clippy --workspace` clean (0 warnings)
- `cargo fmt --all -- --check` clean
- Core NAT reflection integration tests pass
- iOS build verification script passes, including static library build
- iOS simulator app build passes (`SCMessenger` scheme, iPhone 17 simulator)
- Android build verification script passes when `ANDROID_HOME` is set
- Android app build passes (`./gradlew assembleDebug`)
- Topic subscribe/unsubscribe/publish paths are wired on Android and iOS
- QR contact + join bundle scan flows are wired on Android and iOS
- CLI command surface and control API paths are functional
- Identity backup export/import wired end-to-end (iOS Keychain, Android SharedPreferences)
- Relay PeerId stable across CLI upgrades (persisted `relay_network_key.pb`, migrated from IronCore identity)
- WASM swarm transport functional (`startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers`)
- `mark_message_sent` exposed via UniFFI for bounded outbox management

## Roadmap to 1.0.0 (Post v0.1.2-alpha)

1. **Automatic Environment Detection and Unified Hydration**
   - Requirement: The app must automatically detect if a previous identity, message history, contacts, or user preferences exist in local storage/backups and utilize them immediately on startup without user intervention.
   - Target: Unified "detect-and-resume" logic that covers all persisted data types across Android, iOS, and Web.
   - Scope: Identity (Keychain/SharedPreferences), Message History (history.db), Contacts (contacts.db), and Privacy Toggles.

2. **Manual Data Management (Reset/Refresh/Delete)**
   - Requirement: Provide a secure, user-facing way to clear or reset all application data.
   - Target: A "Delete All Data" or "Reset Application" button in the Settings view.
   - Action: Securely wipe identity, message history, contacts, and all local preferences from the device.
   - Scope: Android (`SettingsScreen`), iOS (`SettingsView`), and Web.
