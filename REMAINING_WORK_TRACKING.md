# SCMessenger Remaining Work Tracking

This is the active implementation backlog based on repository state verified on **2026-02-25**.

Primary delivery target: **one unified Android + iOS + Web app**.

Owner policy constraints (2026-02-23):

- Global organic growth (no region-targeted rollout sequence).
- Community-operated infrastructure model (self-hosted and third-party nodes are both valid).
- English-only alpha UI language (i18n expansion tracked as backlog).
- No abuse-control or regional compliance hard gate for alpha.
- Anti-abuse controls are required before beta release.
- Critical UX controls must stay in Android+iOS+Web parity with no temporary lead platform.

## Chapter 1: v0.1.2-alpha Ready (Priority Action Items for Alpha Testing)

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
   - Open items:
     - [x] protocol negotiation disconnect loops (`Failed to negotiate transport protocol(s)`)
     - [x] bootstrap relay visibility policy in nearby UI
     - [x] delayed identity-resolution retry path after initial peer connect

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

9. Web parity promotion from experimental to first-class client
   - Current: Web/WASM is functionally present but thinner than mobile app surfaces.
   - Target: parity for identity import/export, relay/bootstrap controls, history UX, and critical messaging paths.
   - Key files:
     - `wasm/src/lib.rs`
     - `wasm/src/transport.rs`
     - `ui/index.html`
     - `core/src/wasm_support/*`

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

17. Race-condition and async sequencing elimination program (cross-platform reliability hard gate)

- Requirement: eliminate non-deterministic behavior caused by concurrent startup paths, async callbacks, transport handoffs, delayed persistence flushes, and UI/state race windows.
- Target: deterministic execution contracts for all critical workflows so no operation advances before required prerequisites are truly complete.
- Scope: core runtime, Android, iOS, WASM/Web, and relay/headless operation paths.
- Must cover these failure classes explicitly:
  - Startup-order races (identity load, nickname hydration, swarm start, listener registration, transport init).
  - Persistence races (write acknowledged in memory but not durable before next read/restart/update).
  - Transport races (direct/relay/path-state transitions, reconnect loops, duplicate in-flight sends, stale peer metadata).
  - Event-order races (out-of-order callbacks, late arrivals overwriting newer state, UI rendering stale snapshots).
  - Cross-thread actor/coroutine races (shared mutable state, non-atomic check-then-act logic).
  - Timeout/backoff races (retry storms, overlapping retries, cancellation that leaves orphan work).
  - Lifecycle races (foreground/background/suspend/resume/permission changes/network changes during send/receive).
  - Identity mapping races (temporary duplicate IDs per same user/device before federation resolves).
- Implementation requirements:
  - Define critical-path sequencing contracts per flow with explicit preconditions/postconditions.
  - Replace ad-hoc sleeps with condition-based waits (event/ack/state predicate) and bounded timeout semantics.
  - Introduce operation-level idempotency keys and monotonic sequence/version checks where state can race.
  - Enforce single-writer or serialized state transitions for shared connection/runtime state.
  - Add explicit handoff barriers for path switching (direct probe -> relay settle -> direct promotion) to prevent message loss/duplication.
  - Ensure persistence writes are confirmed (or fsync-equivalent durability contract documented) before emitting completion to higher layers.
  - Add stale-event suppression (ignore events older than current state generation/revision).
  - Add structured cancellation handling so aborted operations cannot mutate final state after replacement operations complete.
- Verification/test matrix requirements:
  - Deterministic stress tests with injected jitter/delay/reordering for core orchestration and message delivery paths.
  - Mobile lifecycle chaos tests: app pause/resume, network toggle, permission flip, and transport restart mid-send.
  - Upgrade continuity tests where writes and migrations overlap startup and reconnect.
  - Soak tests with repeated connect/disconnect and concurrent sends to detect latent race leaks.
  - Assertions for no duplicate identity rows, no duplicate message delivery, no dropped ACK transitions, and no stuck intermediate connection states.
- Observability requirements:
  - Correlation IDs per workflow (startup, send, retry, path switch, migration).
  - Start/end/error timestamps for each async stage to expose slow or overlapping steps.
  - State-transition audit log with previous->next state, cause, and guard condition result.
  - Counter metrics for retries, cancellations, dropped stale events, dedupe hits, and timeout exits.
- Exit criteria:
  - No reproducible race-condition defects in stress/chaos suites across Android+iOS+WASM+headless relay.
  - Deterministic startup and send/receive sequencing verified in repeated runs with jitter injection.
  - All critical flows have documented sequencing contracts and corresponding automated tests.
  - Bug reports can be root-caused from diagnostics without ambiguous event ordering gaps.

18. Comprehensive TODO/incomplete/stub inventory sync (authoritative for alpha signoff)

- Requirement: every unresolved TODO/incomplete/stub item that represents actionable engineering work must be represented in this file.
- Target: this file remains the single canonical list of remaining work; other docs may provide evidence/context but not untracked action items.
- Current unresolved inventory (code/runtime relevant):
  - Android test execution gap: `android/app/src/test/README.md` documents `@Ignore`d tests that still do not run in CI/Docker due to UniFFI mock/JNA harness limitations.
  - Deprecated shim lifecycle: `wasm/src/lib.rs` keeps `startReceiveLoop(relayUrl)` as compatibility shim and must stay tracked until full removal window closes.
  - Alpha test references that are still open by evidence: live multi-version interop matrix, relay-only WAN validation, suspend/resume real-device evidence, and update-without-wipe continuity evidence.
- Non-action marker inventory (tracked for clarity, not blockers by itself):
  - UniFFI generated TODO comments under generated sources regenerate by design and are not manually editable.
  - Placeholder UI text attributes (input placeholders) are UX text, not implementation stubs.
  - Historical/mixed-status docs may contain legacy TODO/deprecated wording; only items explicitly promoted here are actionable backlog.
- Enforcement:
  - Any new actionable TODO/FIXME/stub found in code review, logs, partner tests, or docs triage must be added here in the same change set.
  - Keep `docs/STUBS_AND_UNIMPLEMENTED.md` and this file synchronized; this file is canonical for open work.

### Alpha Exit Checklist (must be complete before v0.1.2-alpha signoff)

- [x] Identity/contact/history continuity proven through upgrade-in-place runs (no wipe) on Android+iOS+WASM.
- [ ] Cross-version matrix evidence complete (`v0.1.0`, `v0.1.1`, current head) with bidirectional message delivery.
- [ ] WAN/relay-only behavior validated with Android+iOS off-LAN scenarios and headless relays.
- [ ] ACK/path-switch reliability validated under network transition stress.
- [ ] Race-condition hardening item (above) reaches pass criteria and diagnostics clarity.
- [ ] Outstanding `@Ignore` test strategy decided (enable with harness or explicitly accepted alpha risk with bounded plan).

## Chapter 2: v1.0.0 Readiness (Global Rollout Prerequisites Beyond Alpha)

### Global Rollout Master Checklist (LoC estimates only)

This section is the sanity-checked global rollout checklist. It intentionally includes cross-functional engineering work beyond alpha validation so worldwide deployment does not rely on implicit assumptions.

1. Release-grade reliability and correctness hardening
   - Scope:
     - Finalize deterministic connection state machine behavior under sustained churn.
     - Close remaining path-switch delivery edge cases (direct<->relay<->reconnect).
     - Guarantee exactly-once-apply semantics (idempotent receive path) at app-layer state.
     - Validate suspend/resume and process-kill recovery behavior on real devices.
   - LoC estimate: **800-1,600**

2. Message durability and data integrity guarantees
   - Scope:
     - Define and enforce durable-write completion semantics for identity, contacts, inbox, outbox, and local nickname overrides.
     - Add consistency checks/repair routines for partially-written stores.
     - Add data corruption detection + safe fallback/rebuild flows.
     - Finalize bounded-retention behavior with deterministic pruning and integrity-preserving compaction.
   - LoC estimate: **700-1,400**

3. Identity model hardening across all transports
   - Scope:
     - Enforce single canonical identity mapping for BLE/LAN/libp2p discovery surfaces.
     - Eliminate duplicate peer/device representations before UI materialization.
     - Add conflict-resolution precedence (federated nickname vs local override vs stale cache).
     - Add identity update propagation rules with revision ordering.
   - LoC estimate: **500-1,000**

4. Anti-abuse and relay-protection implementation (beta gate prerequisite)
   - Scope:
     - Relay request rate limits, envelope-size controls, and abuse throttling.
     - Connection and request budgeting at node and peer dimensions.
     - Basic abuse telemetry and operator-side observability hooks.
     - Regression tests for false-positive and false-negative envelope handling.
   - LoC estimate: **900-1,800**

5. Security hardening baseline for production exposure
   - Scope:
     - Secrets/key material lifecycle hardening at rest and in memory where feasible.
     - Strict input validation for all external/network-decoded payloads.
     - Dependency/security policy gate for CI (advisory scanning + lockfile hygiene).
     - Structured security test pass for transport, storage, and import flows.
   - LoC estimate: **700-1,300**

6. Global relay/bootstrap infrastructure operability
   - Scope:
     - Define supported relay deployment topology patterns (single-node, small cluster, federated community).
     - Provide operator runbooks for bootstrap health checks, key rotation, and rollback.
     - Add automated health probes and failure alarms for critical routing paths.
     - Add operator-visible diagnostics contract shared with app-side export format.
   - LoC estimate: **600-1,200**

7. CI/CD and release pipeline hard gates
   - Scope:
     - Enforce Android+iOS+WASM build/test gates on mainline and release branches.
     - Add browser-executed wasm tests and mobile artifact verification gates.
     - Add reproducible release packaging checks and signed artifact validation.
     - Add regression gate for migration continuity and path-switch integrity tests.
   - LoC estimate: **500-1,000**

8. App-store and distribution readiness (engineering-owned pieces)
   - Scope:
     - Versioning/build-number automation across Android/iOS/Web artifacts.
     - Release-channel configuration (internal, beta, production) with rollback-safe metadata.
     - Crash/diagnostic symbol handling pipeline for release artifacts.
     - Build-time feature-flag manifest validation for production safety.
   - LoC estimate: **350-750**

9. Observability and incident response tooling
   - Scope:
     - Complete end-to-end correlation IDs from UI action -> transport -> persistence -> ACK.
     - Add structured log redaction policy and privacy-safe export controls.
     - Add incident triage templates for connectivity, delivery, identity, and migration failures.
     - Add health dashboards for delivery success rate, retry behavior, and relay usage mix.
   - LoC estimate: **450-900**

10. Compatibility and long-tail environment validation

- Scope:
  - Finalize minimum supported Android/iOS/browser versions with evidence-based acceptance suite.
  - Validate behavior under common adverse environments (high packet loss, captive portals, intermittent WAN).
  - Validate mixed-version interop guardrails and deprecation safety windows.
  - Add formal compatibility matrix publication in canonical docs.
- LoC estimate: **600-1,200**

11. UX parity and fail-safe behavior for critical flows

- Scope:
  - Ensure parity for all critical settings and runtime states across Android+iOS+Web.
  - Define non-ambiguous user-facing statuses for sending, queued, retrying, delivered, and failed.
  - Ensure failure recovery affordances do not require app restart or reinstall.
  - Finalize onboarding and recovery paths so existing identity/nickname never re-prompts incorrectly.
- LoC estimate: **450-900**

12. Documentation and operational governance completion

- Scope:
  - Keep canonical docs synchronized with verified runtime behavior and release gates.
  - Publish deployer/operator guides for self-hosted and third-party infrastructure patterns.
  - Finalize upgrade playbook and rollback decision trees for production incidents.
  - Maintain actionable TODO/stub inventory with mandatory backlog sync rules.
- LoC estimate: **300-700**

### Locked global policy decisions (implementation required)

1. Dynamic remote signed bootstrap ranking list (community-submitted)
   - Locked direction:
     - Bootstrap/relay/headless nodes are regular nodes running without user identity, optimized for resource efficiency.
     - Anyone can submit candidate bootstrap nodes.
     - Clients consume a remotely distributed, signed bootstrap list.
     - All newly added nodes start at the bottom of rank order.
     - Clients attempt top-ranked nodes first and only exhaust downward as needed.
     - Nodes are demoted on observed unavailability/failures; sustained healthy performance increases effective rank over time.
     - No “master relay” role exists; ranking is purely performance/reliability-based.
   - Required implementation:
     - Signed-list schema for node metadata + rank inputs.
     - Rank update algorithm + demotion/promotion rules.
     - Client resolver logic that respects ranked order and fallback semantics.
     - Relay health telemetry inputs used for rank scoring.
     - Operator submission/update workflow and abuse-resistant validation for list updates.
   - LoC estimate: **450-1,100**

2. Chain-of-trust block propagation with direct-P2P exception
   - Locked direction:
     - Any relay-mediated trust graph interaction implies trust-chain participation.
     - If a peer is blocked by any member in the trust chain, relay-mediated communication is blocked across that chain.
     - Direct P2P is still allowed but must be flagged as blocked-risk on inbound attempts.
     - Identity reset exists as recovery but is rate-limited to at most once per 24 hours.
   - Required implementation:
     - Trust-chain representation and block-propagation protocol.
     - Relay-side enforcement path that denies relay for chain-blocked peers.
     - Direct-P2P warning/flag path for blocked identities.
     - Identity-reset cooldown enforcement and anti-abuse safeguards.
     - Test coverage for false propagation, unblock transitions, and cooldown bypass attempts.
   - LoC estimate: **900-1,900**

3. User-priority message retention with adaptive relay cache pruning
   - Locked direction:
     - User’s own sent/received conversations are high-priority retained data.
     - Relay-cached transient data may be pruned according to device storage policy.
     - If storage pressure is low, avoid forced pruning.
     - On storage pressure, prompt user before pruning retained conversation data.
     - Offer backup/compression cold-storage export before destructive pruning.
     - Allow user-driven pruning by conversation/size.
   - Required implementation:
     - Storage-tier model (high-priority conversations vs relay cache tiers).
     - Storage-pressure detector + user prompt gating.
     - Backup/export + compression flow for cold storage.
     - Conversation-level size accounting and selective prune actions.
     - Integrity tests for restore/reimport after export and prune.
   - LoC estimate: **700-1,600**

4. Global language and icon-forward UX policy
   - Locked direction:
     - Worldwide rollout must be understandable for users across languages.
     - Use iconography aggressively where it improves cross-lingual clarity.
   - Required implementation:
     - Internationalization scaffold across Android/iOS/Web UI strings.
     - High-frequency workflow icon system (status/actions/settings) with accessibility labels.
     - Locale formatting parity (dates, numbers, pluralization, script direction where applicable).
     - Cross-lingual usability validation for critical actions (send, retry, block, backup, prune).
   - LoC estimate: **900-2,100**

5. Diagnostics export privacy policy: identifiers excluded
   - Locked direction:
     - Diagnostics exports must exclude user identifiers (safest/simplest policy).
   - Required implementation:
     - Export schema update to omit identity/user-specific identifiers.
     - Redaction enforcement tests for all export producers (mobile/web/core).
     - Operator/partner documentation updates for identifier-free debugging workflows.
   - LoC estimate: **150-420**

### Traceability rule (must hold continuously)

- Any newly discovered actionable TODO/FIXME/stub/incomplete implementation from:
  - runtime logs,
  - partner/field testing,
  - PR/code review,
  - incident postmortem,
  - or doc audits,
    must be added to this file in the same change set that captures the finding.

## Priority 1: Tooling, CI, and Experimental Surface

1. [x] Align CI with tri-platform target status
   - Current: `.github/workflows/ci.yml` has Android, iOS, and WASM jobs.
   - Target: enforce Android+iOS+Web build-readiness checks in gating CI for mainline changes.
   - Outcome: Completed. `ci.yml` now includes `check-android`, `check-ios`, and `check-wasm` jobs.

2. [x] Add browser-executed WASM test job (parity gate)
   - Current: `wasm-pack` runtime test coverage is in CI.
   - Target: `wasm-pack` runtime test coverage in CI.
   - Outcome: Completed. `ci.yml` includes a job that runs `wasm-pack test --headless --firefox`.

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

18. [x] Android multi-share intent handler stub

- File: `android/app/src/main/java/com/scmessenger/android/utils/ShareReceiver.kt` lines 67–72.
- Current: `handleMultipleShare()` logs a warning and shows a toast; no items are forwarded.
- Target: either implement multi-item share handling or remove `ACTION_SEND_MULTIPLE` from the manifest intent filter so the share sheet never offers the option.
- Outcome: removed `ACTION_SEND_MULTIPLE` handling path and dead stub from `ShareReceiver.kt`; runtime now exposes only implemented `ACTION_SEND` behavior.

19. [ ] App-update persistence migration hardening (identity, contacts, message history)

- Requirement: app upgrades must preserve identity, contacts, and message history without manual re-import.
- Target: deterministic migration/verification path across Android and iOS app updates, including storage-path continuity checks and automatic import fallback for legacy stores.
- Scope: core storage versioning, mobile app startup migration hooks, and update smoke tests that assert post-update continuity.
- Current progress:
  - Added core storage layout/schema guard (`SCHEMA_VERSION`) and explicit `identity/`, `outbox/`, `inbox/` sub-store initialization.
  - `IronCore::with_storage()` now initializes persistent inbox/outbox backends (not memory-only fallback by default).
  - Added core persistence restart tests for inbox/outbox continuity under storage-backed initialization.
  - Added schema v2 legacy-root migration to copy old identity/outbox/inbox keys into split sub-stores on upgrade.
  - Identity manager now hydrates persisted identity/nickname on startup without auto-generating fresh identities.
  - Added restart continuity tests for identity hydration, legacy-root migration, contacts (including local nickname), and history delivery-state persistence.
  - Android onboarding now waits for confirmed identity creation + nickname persistence before completing first-run flow.
  - Android/iOS repository flows now explicitly resume deferred swarm startup after identity/nickname creation, closing a first-run internet transport stall path.
  - CLI relay mode now uses persisted headless network identity (`storage/relay_network_key.pb`) so relay peer IDs remain stable across process restarts.
- Remaining:
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
- Use `docs/GLOBAL_ROLLOUT_PLAN.md` for release-phase execution.
- Treat older completion/audit reports as historical context unless reconfirmed.
