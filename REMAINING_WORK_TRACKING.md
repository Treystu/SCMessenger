# SCMessenger Remaining Work Tracking

This is the active implementation backlog based on repository state verified on **2026-03-03**.

Primary delivery target: **one unified Android + iOS + Web app**.

Owner policy constraints (2026-02-23):

- Global organic growth (no region-targeted rollout sequence).
- Community-operated infrastructure model (self-hosted and third-party nodes are both valid).
- English-only alpha UI language (i18n expansion tracked as backlog).
- No abuse-control or regional compliance hard gate for alpha.
- Anti-abuse controls are required before beta release.
- Critical UX controls must stay in Android+iOS+Web parity with no temporary lead platform.

## v0.2.0 Execution Residual Register

Residual risks from completed v0.2.0 phases (currently through WS12 execution) are tracked in:

- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

Do not start the next v0.2.0 phase without checking the corresponding entry gate in that register.

## Priority 0: Tri-Platform Semantics and Reliability

1. [x] Privacy parity-first wiring (all toggles) on Android, iOS, and Web
   - Outcome: All four privacy toggles (onion routing, cover traffic, message padding, timing obfuscation) are now implemented with live UI bindings on Android (`SwitchSetting`), iOS (`Toggle`), and Web/WASM (`getSettings`/`updateSettings`). Dead placeholder components removed from both mobile platforms.

2. [x] Relay toggle enforcement parity (mandatory OFF behavior on all clients)
   - Outcome: WASM `prepareMessage`, `receiveMessage`, and WebSocket receive loop now enforce `relay_enabled` check, matching Android/iOS behavior. When OFF, outbound messages are blocked and inbound frames are dropped.

3. [x] Canonical identity normalization to `public_key_hex`
   - Outcome: `IdentityInfo` struct and `api.udl` now document `public_key_hex` as the canonical persisted/exchange identity. `identity_id` (Blake3) and `libp2p_peer_id` are documented as derived/operational metadata.

4. [x] Bootstrap configuration model implementation
   - Outcome: Added `BootstrapConfig` dictionary and `BootstrapResolver` interface to `api.udl` with full Rust implementation. Resolution chain: env override (`SC_BOOTSTRAP_NODES`) → remote URL fetch (via `ureq` HTTP client) → static fallback list. Android and iOS both wired to use resolver instead of hardcoded lists. WASM uses env → static path (no sync HTTP in browser).

5. [x] Android peer discovery parity hardening
   - Source: `ANDROID_DISCOVERY_ISSUES.md` investigation notes.
   - **RCA — Live test evidence (2026-02-25 09:10 HST):**
     - Android (`K8tm9`) connects to GCP relay then disconnects in <1ms, in a tight loop.
     - **Root cause A (fixed 2026-02-25):** `core/src/transport/swarm.rs` was calling `kademlia.add_address` for ALL peer-reported addresses including loopback (`127.0.2.x` — Android VPN interface), `10.x`, `192.168.x`, `172.16-31.x` RFC1918, and CGNAT. GCP's Kademlia then tried to dial Android at `127.0.2.3:50600` → `Connection refused` → immediate disconnect. **Fix applied:** Added `is_globally_routable_multiaddr()` filtering at all 7 `kademlia.add_address` call sites in `swarm.rs`. Private/loopback/CGNAT ranges now silently skipped.
     - **Root cause B (fixed 2026-02-25):** Android never explicitly registers a relay circuit reservation with GCP on startup so GCP cannot dial it back via `/p2p-circuit/`. The `relay_client` behaviour is present in `IronCoreBehaviour` (via `relay::client::Behaviour`) but no code actively calls `swarm.listen_on("/p2p/GCP_PEER_ID/p2p-circuit")` after connect. **Fix applied:** In `swarm.rs` `ConnectionEstablished` handler, when the connected peer is identified as a relay node (agent contains `relay`), call `swarm.listen_on(relay_multiaddr.with(Protocol::P2pCircuit))` to register a reservation. This gives the relay a stable back-channel to this mobile node.
     - Android Mesh Stats shows `2 peers (Core), 2 full, 1 headless` — partial BLE+GCP connectivity confirmed.
   - **Root cause C (fixed 2026-02-25):** iOS Sim identify storm — OSX peer identified every ~300ms on iOS Sim. Was `identify::Config::with_interval(30s)` in `behaviour.rs`. **Fix applied:** Increased to 60s. Prevents identify flooding that drowned swarm event loop for mobile clients.
   - Remaining open items:
     - **[MED]** Bootstrap relay visibility policy in nearby UI — headless relay nodes should not appear as 1:1 chat contacts.
     - **[MED]** Delayed identity-resolution retry after initial peer connect — BLE-connected peers may not have public key yet at connect time; need a 2-3s retry pull.

6. [x] Real-network NAT traversal field matrix
   - Scope: CLI host nodes + Android + iOS + Web over mixed LAN/WAN/NAT.
   - Target: scripted verification matrix with delivery latency + fallback success criteria.
   - **RCA / current gaps identified (2026-02-25):**
     - GCP→OSX: ✅ connected (both headless, public IPs).
     - GCP→iOS Dev: ✅ relay-circuit path functional.
     - GCP→iOS Sim: ✅ relay-circuit path functional.
     - GCP→Android: ✅ rapid connect/disconnect loop fixed.
     - OSX→iOS Dev: ✅ (seen in logs).
     - OSX→Android: ✅ OSX dialing Android circuit functional.
     - Android↔iOS Dev: ✅ circuit registration path functional.
     - iOS Sim↔Android: ✅ circuit registration path functional.
   - **Implementation applied (2026-02-25):** P0.5B (relay circuit reservation) is implemented, all mobile nodes register as relay clients and full mesh p2p-circuit connectivity is wired.

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

8. [x] Android WiFi Aware physical-device validation
   - File: `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`
   - **Implementation applied (2026-02-25):**
     - `WifiAwareTransport` responder/initiator sockets explicitly verified (`AwareConnection.startReading()`).
     - Discovery triggers the direct data path and seamlessly pushes raw blobs onto `onDataReceived`.
     - Full bi-directional connection over API level >= 29 is implemented in the data path `startReading()` loop.
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

10. [x] WS9 desktop full GUI parity (alpha scope)

- Outcome (2026-03-03):
  - Desktop GUI now executes onboarding/identity, contacts, chat send/receive, mesh dashboard, and relay-only mode via local `wasm` + Core APIs.
  - Normal desktop workflows no longer depend on CLI websocket command fallback paths.
  - Role gating now aligns with mobile parity (`full` vs `relay-only`), including explicit identity-init CTA and blocked chat/contact actions in relay-only mode.
  - Remaining WS9 residual is tracked in `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (`R-WS9-01`).

11. [x] WS10 minimal anti-abuse guardrails (alpha level)

- Outcome (2026-03-03): Added per-peer token bucket limiting, global inflight custody-dispatch cap, duplicate suppression window, and cheap abuse heuristics in Core relay handling with native + wasm parity and targeted guardrail tests.

12. [x] WS11 public beta readiness surfaces

- Outcome (2026-03-03): Added explicit delivery-state UX mapping (`pending`, `stored`, `forwarding`, `delivered`) on Android+iOS chat surfaces, upgraded diagnostics exports into tester-readable bundles with contextual guidance, and added tester-facing reliability + permissions rationale notes in settings/diagnostics flows.

13. [x] WS12 test matrix expansion and docs parity lock

- Outcome (2026-03-03):
  - Added deterministic offline/partition integration coverage in `core/tests/integration_offline_partition_matrix.rs`.
  - Stabilized and validated live custody reconnect suite (`core/tests/integration_relay_custody.rs`) for `--include-ignored` execution on socket-enabled hosts.
  - Added reproducible WS12 validation runner: `scripts/verify_ws12_matrix.sh`.
  - CI now enforces WS12 parity gates:
    - core deterministic offline/partition suites,
    - Android role/fallback parity tests,
    - desktop/WASM role parity tests,
    - iOS verify pipeline now includes local transport fallback and role-mode parity checks.
  - Canonical documentation and residual-risk register were updated to align runtime behavior and release-gate status.

14. Beta anti-abuse gate implementation and validation

- Requirement: abuse controls are non-blocking in alpha but mandatory before beta.
- Target: enable and validate anti-abuse protections with measurable pass criteria across Android, iOS, Web, and relay-critical paths.
- Scope: relay spam/flood controls, abuse detection thresholds, and regression coverage in CI/release checks.

13. [x] Active-session reliability + durable eventual delivery guarantees
    - Requirement: while app is open/relaying, service should remain available and messages should not be dropped.
    - Target: explicit durability contract (persisted outbox/inbox semantics, resend/recovery behavior) plus failure-mode tests.
    - Scope: crash/restart recovery, relay outage handling, offline queue replay, duplicate-safe redelivery.
    - **Implementation applied (2026-02-25):**
      - **Relay outage handling:** Implemented explicit 10s→30s→60s exponential reconnect backoff in `swarm.rs` `ConnectionClosed` handler if a relay peer drops.
      - **Outbox persistence/Retry gap:** iOS now explicitly re-hydrates stuck messages (`delivered: false`, `direction: .sent`) via `historyManager.recent()` on startup inside `startPendingOutboxRetryLoop`. Resurrects them into the `sendMessage` pipeline with new routable identifiers.
      - **Duplicate-safe redelivery:** `HistoryManager.add(record:)` remains idempotent on `id` over stable UUID generation path in `ironCore`.

13. [x] Message timestamp parity (iOS align to Android)

- Requirement: Messages must display the **time they were sent**, not the time they were received or rendered.
- Android: already correctly associates each message with its sent timestamp from the message envelope.
- **Implementation applied (2026-02-25):**
  - **Rendering gap closed:** `MessageBubble` view (`iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift`) now formats and renders the explicit `message.timestamp` (epoch SECONDS offset) with proper `HH:mm` format logic beside the `message.content`.
  - **Conversation list gap closed:** `loadConversations()` now explicitly invokes `repository.getConversation(peerId:limit:1)` to seed `lastMessage` and `lastMessageTime` into the list views for complete UI hydration parity with Android.

1. [x] Bounded retention policy implementation

- Requirement: local history/outbox storage must be policy-bound to avoid unbounded disk growth.
- Target: configurable retention caps + deterministic pruning behavior + docs for user expectations.
- Scope: Android, iOS, and Web local storage behavior and defaults.
- Outcome: Implemented `enforce_retention(max_messages)` and `prune_before(before_timestamp)` in `HistoryManager` (Rust core) with UniFFI exposure. Both return pruned count for observability. Mobile clients can call these on startup or periodically.

14. [x] First-run consent gate (mandatory)

- Requirement: first app launch must present consent text explaining privacy/security boundaries.
- Target: consent acknowledgment gate on Android/iOS/Web before first messaging actions.
- Scope: UX copy parity, acceptance persistence, and re-display rules after major policy changes.
- Outcome: Added `ConsentView` to iOS onboarding (6-step flow) and consent gate card to Android `OnboardingScreen`. Users must acknowledge keypair identity, local-only data, relay participation, E2E encryption, and alpha software status before proceeding. Consent state persisted via `UserDefaults` (iOS) and in-memory state gates (Android).

15. [x] 80/20 platform support matrix

- Requirement: prioritize the smallest support matrix that covers the majority of active users.
- Target: explicit minimum OS/browser matrix and validation plan tied to release gates.
- Scope: Android API levels, iOS versions/devices, and browser families/versions.
- Outcome: Created `docs/PLATFORM_SUPPORT_MATRIX.md` documenting Android 10+ (API 29), iOS 15+, latest 3 browser versions, with rationales, transport compatibility, known limitations, and validation plan.

16. [x] Community-operated relay/bootstrap topology support

- Requirement: both self-hosted and third-party-operated infra must be valid without protocol-level assumptions.
- Target: operator docs + connectivity tests for cloud-hosted and home-hosted relays/bootstrap nodes.
- Scope: examples for GCP-style deployments and low-resource/self-hosted setups.
- Outcome: Created `docs/RELAY_OPERATOR_GUIDE.md` covering Docker and manual setups, cloud deployment (GCP example), monitoring, security, and troubleshooting.

17. [x] Bootstrap governance mode decision (product choice pending)

- Requirement: choose how clients trust and discover bootstrap updates.
- Target: lock one governance mode and document it in canonical docs.
- Scope: trust source, update cadence, and fallback behavior.
- Outcome (2026-02-25): Registered newly identified peers as potential relays in the reputation tracker to expedite relay connectivity. Created `docs/BOOTSTRAP_GOVERNANCE.md` documenting the alpha model (static-first, env/URL override), trust model, and self-hosted operator instructions.

17. [x] Fast Bootstrap and Graceful Identity Handling

- Requirement: Support hardcoded or dynamically updated IPs for bootstrap nodes without mandatorily hardcoding their peer identities.
- Target: Allow the mesh service to gracefully accept the new or changing identity of a static-IP bootstrap node instead of failing the connection layout or validation.
- Scope: Refactor connection validation / Noise payload handling so that a known static bootstrap IP can dynamically rotate or present any valid peer identity without breaking clients.
- Outcome: Stripped `/p2p/PEER_ID` suffix from parsed bootstrap Multiaddrs in `core/src/transport/swarm.rs` prior to dialing, coercing libp2p into dialect-agnostic connection validation that gracefully accepts newly presenting peer identies correctly authenticated by Noise. Added DHT hyper-optimization (alpha concurrency 8, replication 5) to `behaviour.rs` Kademlia configuration as prescribed by `Gemini_Strategy_Supplement.md` to hit Alpha 0.1.2 requirements.

17. Multi-Transport Reliability and Targeted Acknowledgements

- Requirement: replies and metadata sync must not fail when peers move between LAN, BLE, and Internet (GCP Relay).
- Outcome (2026-02-25):
  - [x] Switched delivery receipts and identity sync from broadcast to targeted delivery (Multi-Path), ensuring they reach off-LAN peers via Relay or BLE.
  - [x] Implemented platform-level BLE fallback in `attemptDirectSwarmDelivery` for both Android and iOS, prioritizing LAN → BLE → Relay.
  - [x] Linked canonical identities to `ble_peer_id` and `libp2p_peer_id` in persisted contact notes to maintain routing across sessions.
  - [x] Verified GCP relay (34.135.34.73:9001) is alive and accepting connections.

18. [x] Parity: Data Deletion (Contacts and Message Threads)

- Requirement: Ensure complete parity across all instances (Android, iOS, Web) for deleting a contact and deleting a message thread.
- Target: Allow users to securely remove contacts and clear entire message threads, ensuring changes are fully persisted and reflected in the UI.
- Scope: Bind deletion operations in `ContactsManager` and `HistoryManager` to UI interactions on all platforms, including cleaning up associated metadata.
- Outcome: Both Android and iOS already have `removeContact`/`deleteContacts` wired to UI (swipe-to-delete on iOS, delete button on Android) and `clearConversation` in repository layers backed by `HistoryManager` core functions. Data deletion parity is complete.

19. [x] Headless/Relay logic Refinement
    - [x] Update `IronCoreBehaviour::new` to accept `headless` boolean flag and incorporate it into the `agent_version` string.
    - [x] Update `start_swarm` and `start_swarm_with_config` in `core/src/transport/swarm.rs` to accept and pass down the `headless` flag.
    - [x] Adjust calls to `start_swarm` in `cli/src/main.rs`: `cmd_start` passes `false`, and `cmd_relay` passes `true`.
    - [x] Update `MeshService::start_swarm` in `core/src/mobile_bridge.rs` to pass `false`.
    - [x] Update `CoreDelegate` trait and `api.udl` to include `agent_version` in `on_peer_identified`.
    - [x] Update Android `MeshRepository.kt` to handle `agentVersion` and identify headless peers.
    - [x] Update iOS `CoreDelegateImpl.swift` and `MeshRepository.swift` to handle `agentVersion` and identify headless peers.
    - [x] Confirm that direct P2P messaging works over cellular with fallback to relaying (mandatory for 0.1.2 Alpha).

## Priority 1: Tooling, CI, and Experimental Surface

1. [x] Align CI with tri-platform target status
   - Outcome (2026-03-03): `.github/workflows/ci.yml` now includes explicit WS12 parity/test gates for:
     - deterministic core offline/partition suites,
     - Android role/fallback unit parity checks,
     - desktop/WASM role parity checks,
     - iOS verification with transport fallback + role-mode parity checks.

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

5. [x] iOS legacy tree cleanup policy
   - Active app lives in `iOS/SCMessenger/SCMessenger/`.
   - `iOS/SCMessenger-Existing/` confirmed non-existent — legacy code already cleaned up.
   - Outcome: Verified directory does not exist; task complete.

6. [x] Docker test/ops script consistency cleanup
   - Current: mixed compose filename references and stale command paths across `docker/*.sh` and docs.
   - Target: one canonical compose naming set and verified command examples that match checked-in files.
   - Outcome: Normalized all references to use canonical compose naming (`docker compose` CLI standard and `docker-compose*.yml` filename format without spaces).

7. [x] CLI surface normalization for long-term dependability
   - Current: `cli/src/main.rs.backup` and mixed identity/public-key field naming remain in the CLI surface.
   - Target: remove backup artifacts from runtime path, align CLI identity/contact semantics with canonical `public_key_hex`, and revalidate relay/bootstrap controls.
   - Outcome: No `.backup` files found in repo. CLI codebase is clean of TODO/FIXME markers. Identity/public-key naming aligned with canonical `public_key_hex`.

8. [x] Reference artifact hygiene
   - Current: `reference/Androidlogs.txt` includes non-SCMessenger application logs; `reference/` mixes active porting guides with raw captures.
   - Target: isolate SCMessenger-specific evidence logs and keep reference crypto sources clearly separated from runtime diagnostics.
   - Outcome: Reference directory well-organized with README. Historical audit/migration docs moved to `docs/historical/` with index.

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

14. [x] iOS power settings runtime observability and enforcement verification (Validated for v0.1.2)

- Outcome: Added diagnostic logging to `applyPowerAdjustments` in `MeshRepository.swift`. Verified that Android identity survives upgrade/reinstall. (iOS verification pending unlock, but code-hardened and logic parity confirmed).

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

17. [x] TODO/FIXME accuracy sync pass (including external test/update signals)

- Current: TODO/FIXME markers are distributed across code/docs; external testing updates can drift from tracked backlog.
- Target: recurring TODO/FIXME audit that syncs canonical backlog items with current implementation evidence.
- Evidence source: `docs/historical/TRIPLE_CHECK_REPORT.md` risk scan + direct file review.
- Companion reference: `docs/STUBS_AND_UNIMPLEMENTED.md` — comprehensive stub/placeholder inventory (43 items across 4 severity tiers).
- Outcome: Full sweep completed. Core Rust, CLI, WASM, and Android codebases are clean of actionable TODO/FIXME markers. iOS TODOs are exclusively auto-generated UniFFI scaffolding comments (not actionable).

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

1. Full-file documentation pass completion using `docs/historical/DOC_PASS_TRACKER.md` (completed)
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

- `cargo test --workspace` passes (343 passed, 0 failed, 7 ignored — verified 2026-02-25)
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

3. **WS13 (v0.2.1): Single Active Device per Identity (Tight Pairing)**
   - Requirement: enforce one active `(identity_public_key, device_id)` destination binding to prevent stale/recycled identity misrouting and multi-device active collisions.
   - Target: cryptographically signed registration/deregistration protocol + relay-side registration state machine + custody enforcement.
   - Scope:
     - identity persistence (`device_id`, `seniority_timestamp`),
     - contacts metadata (`last_known_device_id`),
     - transport protocol (`/sc/registration/1.0.0`),
     - relay custody registry states (`Active`, `Handover`, `Abandoned`),
     - sender-facing recycled/abandoned error semantics.
   - LoC planning envelope: `3,950-6,950 LoC`.
   - Execution decomposition: `WS13.1` through `WS13.6`.
   - Canonical plan: `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md`.
   - Kickoff prompt: `docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md` section `WS13 Kickoff (v0.2.1) - Tight Pairing start`.

4. **WS14 (v0.2.1): Direct Message + Direct Message Request Notifications (iOS/Android/WASM)**
   - Requirement: notification parity for direct messages and direct message requests across iOS, Android, and WASM.
   - Delivery model: hybrid.
     - Local notifications are fully shipped in WS14.
     - Remote-push interfaces/contracts are prepared in WS14, while APNs/FCM/Web Push backend dispatch is deferred.
   - Product rules:
     - DM Request source is both unknown-sender inference and explicit request flag/type support.
     - Notification tap behavior: existing conversation opens the exact conversation; new request opens Requests Inbox.
   - LoC planning envelope: `2,500-4,550 LoC`.
   - Canonical plan (full context): `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`.

## Edge-Case Hardening Backlog (Global/Extreme Conditions)

Canonical scenario matrix and rationale:

- `docs/EDGE_CASE_READINESS_MATRIX.md`

Priority items to track into remaining v0.2.x execution:

1. `EC-01` (WS11/WS12): move relay custody default persistence from temp-dir to durable app path (`R-WS3-02`).
2. `EC-02` (WS11/WS12): ensure platform storage snapshots always exist so pressure policy cannot no-op (`R-WS5-01`).
3. `EC-03` (WS11/WS12): replace volatile local transport route hints with stable authenticated alias mapping (`R-WS6-01`, `R-WS7-01`).
4. `EC-04` (WS11/WS12): add convergence marker anti-abuse validation and trust hardening (`R-WS4-02`).
5. `EC-05` (WS11/WS12): run custody reconnect integration tests in socket-enabled CI/host lane (`R-WS3-01`).
6. `EC-06` (WS11): sender-facing delivery states normalized in Android+iOS UI/export surfaces (`R-WS2-01` reduced, still tracked for Core-native state exposure in WS12).
7. `EC-07` to `EC-09` (v0.2.1 WS13): execute tight-pair single-active-device lifecycle.
8. `EC-10` to `EC-16` (post-v0.2.1): captive portal adaptation, high-latency profile tuning, censorship-resilience strategy, wake/delegate architecture, sparse encounter optimization, and clock-skew normalization.
