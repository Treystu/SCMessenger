# SCMessenger v0.2.0 Alpha Milestone Plan

Status: Active (execution complete through WS12.35 non-device reliability reconciliation)
Last updated: 2026-03-10
Scope: Core + Android + iOS + Desktop GUI + Relay topology

---

## Closeout Re-Baseline Addendum (WS12.39, 2026-03-10 UTC)

1. WS12/v0.2.0 closeout truth is now explicitly split into four buckets:
   - physical-device reliability evidence still required (`R-WS12-29-01`, `R-WS12-29-02`, `R-WS12-04`, `R-WS12-05`, `R-WS12-06`),
   - non-device CI/build drift still requiring repo-side fixes (current iOS MainActor isolation failures and Docker Android-unit-test library-path drift),
   - GitHub-hosted trust-signal cleanup still requiring maintainer action (issue taxonomy, labels/milestones, branch protection, `action_required` approval-policy cleanup, stale-branch cleanup),
   - explicitly deferred `v0.2.1` work (`WS13`, `WS14`) which remains out of scope for this burndown.
2. Minimal local verification drift was reduced in this pass:
   - Rust formatting and workspace build/test baseline is green again after the CLI whitespace cleanup and WASM `SwarmEvent::PortMapping(_)` handling update.
3. Milestone implication:
   - `v0.2.0` is not yet ready to be treated as a fully trustworthy steady baseline until the non-device CI defects and GitHub-hosted hygiene actions above are also resolved,
   - `WS13` / `WS14` must remain planning-only until that stabilization work is closed or explicitly accepted as deferred follow-up.

---

## Repo/GitHub Operating-Model Planning Addendum (WS12.36, 2026-03-07 UTC)

1. A planning-only audit of repository docs, GitHub features, issue tracker state, branch hygiene, and CI topology is now captured in `docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`.
2. Scope classification:
   - this is a repo-governance and release-readiness hardening track,
   - not a product feature expansion beyond v0.2.0.
3. Milestone implication:
   - execution confidence for v0.2.0 closeout depends on cleaning up the repo operating model so docs, issues, CI, and release signals match current reality.
4. Immediate execution sequence from the audit:
   - tighten canonical docs + contributor health surfaces,
   - reset issue taxonomy/templates/labels/milestones,
   - repair workflow topology and required PR checks,
   - consolidate agent-context guidance and clean stale branches/issues.

## GitHub Alpha-Line Alignment Addendum (WS12.36 follow-up, 2026-03-07 UTC)

1. GitHub-facing contributor surfaces now explicitly treat `v0.2.0` as the active alpha baseline:
   - `README.md`
   - `CONTRIBUTING.md`
   - `SECURITY.md`
   - `SUPPORT.md`
   - `.github/ISSUE_TEMPLATE/config.yml`
   - `.github/pull_request_template.md`
2. Scope guardrail reaffirmed:
   - `WS13` and `WS14` remain planned `v0.2.1` follow-up work,
   - they should not be presented in GitHub intake/docs as unfinished `v0.2.0` alpha requirements.
3. Remaining repo-governance follow-up still open:
   - issue taxonomy/label reset,
   - branch protection and CI topology cleanup.

## Repo-side GitHub Operating-Model Completion Addendum (WS12.36 completion, 2026-03-07 UTC)

1. Repo-controlled governance/config work from the audit is now in place:
   - `SUPPORT.md`, `SECURITY.md`, and `CONTRIBUTING.md` rewritten/aligned
   - `.github/CODEOWNERS`, `.github/dependabot.yml`, `.github/copilot-instructions.md`
   - issue forms/config under `.github/ISSUE_TEMPLATE/`
   - expanded `scripts/docs_sync_check.sh`
2. Repo-controlled workflow cleanup is now in place:
   - Docker publish removed from PR triggers
   - Docker integration suite shifted to `main`/scheduled/manual use
   - release workflow renamed to `Release CLI Binaries`
3. Remaining GitHub-hosted follow-up is now explicitly outside the repo-file layer:
   - branch protection / required-check policy on `main`
   - label and milestone creation/reset
   - stale issue triage / recreation
   - approval-policy cleanup for `action_required` runs

## Hotfix Addendum (WS12.9, 2026-03-03)

1. iOS dashboard node-count accuracy hotfix completed:
   - Discovery metric path remained correct.
   - Dashboard node totals now use online-only deduplicated peers and collapse alias IDs (canonical/libp2p/BLE/public-key) before counting.
2. This was classified as a runtime correctness fix within v0.2.0 scope (no milestone scope expansion).

## Runtime Re-baseline Addendum (WS12.10, 2026-03-03 HST)

1. Live custody reconnect verification is stable again (`integration_relay_custody` passed in matrix run and 3 consecutive reruns).
2. Relay rollout skew risk was closed with fresh runtime evidence of `0.2.0` relay agents.
3. iOS startup main-thread I/O warning source was triaged and fixed in `MeshRepository` startup/diagnostics path.

## Wave-3 Closure Addendum (WS12.17, 2026-03-03 HST)

1. Historical checklist triage is complete for targeted `docs/historical/*` sources; ambiguous open boxes were converted to explicit historical status tags.
2. Runtime hardening closure includes strict BLE-only validation mode, BLE/multipeer diagnostics expansion, and Android address-type mismatch reconnect backoff mitigation.
3. Deterministic harness suite added for remaining live-risk closure:
   - `scripts/correlate_relay_flap_windows.sh`
   - `scripts/verify_relay_flap_regression.sh`
   - `scripts/verify_receipt_convergence.sh`
   - `scripts/verify_ble_only_pairing.sh`
4. Local validation debt reduced:
   - `wasm-pack` installed and `cd wasm && wasm-pack build` now passes.
   - Docker-based simulation remains the only local prerequisite blocker.
5. Repo-wide checklist governance result: open checklist inventory reduced to 10 active items (all in `REMAINING_WORK_TRACKING.md`), with historical open-checkbox ambiguity removed.

## Alpha Readiness Closure Addendum (WS12.18, 2026-03-03 HST)

1. Code-quality hard blockers closed for alpha gate:
   - Rust strict clippy gate (`cargo clippy --workspace --lib --bins --examples -- -D warnings`) now passes.
   - Android lint gate (`:app:lintDebug`) now passes after permission/API hardening in BLE/WiFi/notification/foreground-service paths.
2. Interoperability closure artifacts added:
   - `scripts/generate_interop_matrix.sh`
   - `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`
3. Locked baseline alignment applied:
   - Platform support matrix updated to current code baselines (Android `minSdk=26`, iOS floor `17`, app version `0.2.0`).
4. Residual interoperability gaps are now explicitly tracked as backlog/risk entries instead of implicit drift.

## Alpha Readiness Completion Addendum (WS12.20, 2026-03-03 HST)

1. All WS12.18 interop follow-up gaps are now implemented:
   - CLI identity backup import/export, mark-sent, history clear, and diagnostics/path/listener/peer surfaces.
   - WASM local nickname override + external-address visibility + retention/prune parity.
   - Android+iOS adapter consumption of `reset_stats`; CLI/WASM consumption of retention/prune controls.
2. Matrix evidence is now gap-free for static adapter parity:
   - `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md` reports no `Implemented + Not Consumed` or `Missing/Drift` cells for alpha-required surfaces.
3. `R-WS12.18-01` and `R-WS12.18-02` are closed in residual risk register with fresh build/lint evidence.
4. Active unchecked backlog inventory is now reduced to runtime/live-validation and environment prerequisites only.

## Pairwise Deep-Dive Addendum (WS12.21, 2026-03-03 HST)

1. Pairwise deep-dive scripts were re-run against current artifacts (`relay_flap_correlation`, `relay_flap_regression`, `receipt_convergence`, `ble_only_pairing`).
2. Static pairwise surfaces remain closed (`Core -> Android`, `Core -> iOS`, `Core -> WASM/Desktop`) per current interop matrix.
3. Live pairwise closures remain pending physical synchronized capture:
   - `Android <-> iOS` direct/relay delivery+receipt continuity.
   - `Android <-> iOS` strict BLE-only continuity.
4. Physical dual-device deep-dive probe was attempted but blocked by iOS device runtime availability (`xcrun devicectl` reported the phone as `unavailable` in this run).

## Crash + Stability Hardening Addendum (WS12.22, 2026-03-03 HST)

1. iOS send-path crash mitigation was applied in BLE transport/repository flows after crash-log triage identified a force-unwrap-sensitive send path.
2. Android runtime safety sweep removed all Kotlin `!!` force unwrap usage in app sources and hardened BLE reconnect/advertise behavior.
3. Local alpha sanity gates remained green after hardening:
   - `cd android && ./gradlew :app:compileDebugKotlin :app:lintDebug` (pass)
   - `bash ./iOS/verify-test.sh` (pass, 0 warnings in this run)
   - `bash ./scripts/generate_interop_matrix.sh` (pass)
4. Remaining milestone blocker class did not change: live synchronized physical Android+iOS evidence is still required to close pairwise delivery/BLE continuity residuals (`R-WS12-04/05/06`).

## Pending-Outbox Synchronization Addendum (WS12.23, 2026-03-03 HST)

1. Android+iOS send reliability was hardened for a shared failure mode where newer same-peer sends could succeed while older pending entries remained delayed/stuck.
2. Pending queue promotion now matches both canonical `peerId` and cached `routePeerId` aliases before retry scheduling.
3. Active-connection signals now trigger immediate same-peer queue promotion + outbox flush (peer-identified/BLE identity-read paths on both platforms, plus iOS connected-event path).
4. Post-change sanity checks remained green:
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass)
   - `bash ./iOS/verify-test.sh` (pass; 3 warnings, non-fatal)
5. Milestone blocker remains unchanged: synchronized physical Android+iOS runtime evidence is still required to close `R-WS12-04/05/06`.

## Follow-up Addendum (WS12.24, 2026-03-03 HST)

1. Added explicit sender-state convergence closure gate for iOS -> Android flow: when Android recipient ingest is proven, iOS sender state must converge from `stored`/`forwarding` to `delivered` for the same message ID.
2. Added and implemented conversation-list deletion UX parity update: Android now uses end-to-start swipe-to-delete with confirmation to match iOS behavior.
3. This remains in v0.2.0 closeout scope (reliability convergence + UX parity hardening), not a scope expansion into v0.2.1 features.

## Mega-Update Intake Addendum (WS12.25, 2026-03-03 HST)

1. The pending-sync "older messages remain undelivered" regression was re-triaged from the updated `run5.sh` artifact set and pairwise pending-outbox snapshots.
2. Android+iOS route/receipt hardening was applied in-scope for v0.2.0 reliability closure:
   - route hints now refresh on change, not only first-write,
   - receipt sends now prefer observed inbound route/listener hints,
   - route candidate selection now includes recipient-key-aware filtering to reduce stale/mismatched route retries.
3. Dashboard role classification was unified into two explicit buckets (`Node`, `Headless Node`) with relay/headless grouped together.
4. Scope classification:
   - this is reliability/consistency hardening inside existing v0.2.0 streams (WS12 residual closure),
   - not a net-new feature expansion.
5. Remaining milestone gate is unchanged:
   - close `R-WS12-04/05/06` with synchronized post-fix physical Android+iOS evidence.

## Sender-State + Preview Convergence Addendum (WS12.26, 2026-03-03 HST)

1. Addressed sender-facing reliability/UI consistency gap where receipt transitions could be persisted but not reflected promptly in conversation/chat surfaces.
2. Android+iOS receipt handlers now publish refreshed message updates immediately after receipt-driven history/pending state mutation.
3. iOS conversation preview logic now deterministically selects the newest message by timestamp from bounded recent conversation data.
4. Scope classification:
   - reliability/consistency closure inside existing WS12 v0.2.0 surface,
   - no net-new feature scope expansion.
5. Milestone gate remains unchanged:
   - closure of `R-WS12-04/05/06` still requires synchronized post-fix physical evidence.

## Node-Role Classification Correction Addendum (WS12.27, 2026-03-03 HST)

1. Added explicit correction for field-observed regression where a full iOS-sim peer could be rendered as headless.
2. Android+iOS peer identify flow now treats `/headless/` agent strings as provisional when transport identity resolves, promoting resolved peers to full classification.
3. Android+iOS relay-only guardrail now excludes full peers from `isKnownRelay` unless they are bootstrap relays, preventing relay-capability from forcing headless display.
4. Validation in this pass:
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass)
   - `bash ./iOS/verify-test.sh` (pass)
   - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=25 GCP_RELAY_CHECK=0 bash ./scripts/live-smoke.sh` (pass capture)
5. Milestone gate remains unchanged:
   - synchronized physical iOS-device + Android evidence is still required to close `R-WS12-04/05/06` and confirm no field misclassification regression.

## Transport Regression Hotfix Addendum (WS12.28, 2026-03-03 HST)

1. Runtime triage from active trip logs identified a concrete Android crash-loop path in BLE resend fallback:
   - repeated `BleGattClient.connect` NPE when `connectGatt` returned null during retry windows.
2. Root-cause fix shipped in-scope for v0.2.0 reliability closure:
   - Android BLE connect path now handles invalid addresses and null `connectGatt` returns without exception-loop churn.
3. Address-quality hardening shipped on Android+iOS:
   - special-use IPv4 addresses are now rejected as dial candidates,
   - local IPv4 selection now prefers usable private LAN addresses.
4. Validation in this pass:
   - `cd android && ./gradlew app:compileDebugKotlin -q` (pass)
   - `xcodebuild ... -destination 'platform=iOS Simulator,name=iPhone 16e' build` (pass)
5. Scope classification:
   - reliability hardening only (no net-new feature scope expansion),
   - milestone closeout gate remains synchronized physical evidence for `R-WS12-04/05/06` (+ `R-WS12-27-01`).

## Known-Issues Consolidation Addendum (WS12.29, 2026-03-03 HST)

1. Field evidence in this pass confirms the milestone gate is still blocked by real-device stability/convergence classes:
   - iOS send-path crash reports (`SIGTRAP` in BLE peripheral send path),
   - iOS CPU watchdog kills under retry pressure,
   - Android stale-route/stale-BLE-target retry churn.
2. Canonical known-issues + remediation sequence is now explicitly tracked in:
   - `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
3. Milestone scope classification:
   - no scope expansion beyond v0.2.0 reliability closure;
   - this pass is a consolidation and execution-order reset so remaining work can be burned down deterministically.
4. New explicit UX safety TODO captured for closeout:
   - iOS confirmation prompt before contact deletion.
5. Milestone gate remains unchanged:
   - full closeout still requires synchronized physical Android+iOS evidence to retire `R-WS12-04/05/06` and associated WS12.29 risks.

## Live Verification Loop Addendum (WS12.30, 2026-03-03 HST)

1. Added a dedicated step-gated orchestration harness to execute WS12.29 burndown in immediate feedback cycles without changing baseline `run5.sh`:
   - `scripts/run5-live-feedback.sh`
2. Harness gate sequence is now explicit and deterministic for each fix-step:
   - deploy mobile updates (optional),
   - run `run5.sh --update` to refresh/capture 5-node topology,
   - require all-node log health,
   - require full directed node-pair visibility matrix,
   - require crash/fatal marker clean scan,
   - run deterministic verifier suite.
3. Evidence output is now standardized per attempt:
   - `logs/live-verify/<step>_<timestamp>/attempt_*`
4. Scope classification:
   - execution-orchestration hardening only; no feature scope expansion.
5. Milestone closure gate remains unchanged:
   - P0/P1 issues still require synchronized physical Android+iOS convergence evidence to retire residual risks.

## Stale-Target Convergence Addendum (WS12.31, 2026-03-04 HST)

1. Android+iOS route-candidate selection now prioritizes fresh discovery/ledger evidence ahead of cached/persisted hints to reduce stale-route dominance in retry loops.
2. Android+iOS route-candidate validation now requires recipient-key corroboration from extracted peer identity or runtime discovery/ledger evidence.
3. Failed-route IDs are no longer persisted back into pending-outbox entries when no route ACK succeeds, preventing stale failed route reuse.
4. Local fallback targeting now prefers currently connected BLE peers over cached BLE hints on both Android and iOS.
5. iOS contact-delete confirmation safety gate is now implemented in contacts UX.
6. Validation in this pass:
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass)
   - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.data.MeshRepositoryTest"` (pass)
   - `bash ./iOS/verify-test.sh` (pass)
7. Scope classification:
   - reliability/UX safety hardening only; no net-new feature scope expansion.
8. Milestone closure gate remains unchanged:
   - synchronized physical Android+iOS evidence is still required to close `R-WS12-29-02` and paired convergence residuals (`R-WS12-04/05/06`).

## Transport Failure Triage + 10-Fix Reliability Sweep Addendum (WS12.34, 2026-03-04 HST)

1. Live iOS+Android transport failure diagnosed from device logs after WiFi/BLE/cell connection toggling:
   - Rust core `receive_message` errors invisible on mobile due to swallowed `tracing` output.
   - iOS relay flapping threshold (6 events/60s) was self-triggering under normal 2-relay dial patterns.
   - Messages expired/dropped despite intended "never fail delivery" philosophy.
2. 10 targeted fixes applied across Rust core, iOS, and Android:
   - `eprintln!` error visibility at all `receive_message` failure points (Rust core).
   - `relayEnabled` nil-safety on both platforms (defaulting to `true` when settings nil).
   - Retry throttle increased 500→2000ms (iOS), relay diagnostic throttle (iOS).
   - Messages NEVER expire: removed attempt limits and age-based expiry (both platforms).
   - Progressive backoff: `min(2^attempt, 60)` seconds, capping at 300s (both platforms).
   - WiFi recovery triggers immediate outbox flush (both platforms).
   - BLE 15s GATT connection timeout for stale connections (Android).
   - Dial candidate cap at 6 per peer, prioritizing LAN→relay→public (both platforms).
3. Core philosophy enforcement: messages never expire, retry indefinitely with progressive backoff until delivered.
4. Validation in this pass:
   - `cargo check --workspace` (pass)
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass)
5. Scope classification:
   - reliability hardening only; no net-new feature scope expansion.
6. Milestone closure gate remains unchanged:
   - deploy + observe `eprintln!` output on mobile to diagnose any remaining `receive_message` failures.
   - synchronized physical Android+iOS evidence still required to close paired convergence residuals.

## Non-Device Reliability Reconciliation Addendum (WS12.35, 2026-03-06 UTC)

1. Baseline reliability blockers were reconciled with minimal scope changes:
   - WASM `MessageRecord` desktop test fixtures now include `sender_timestamp`, restoring `cargo test --workspace --no-run` determinism.
   - Core receipt validation now requires outbound-recipient correlation for sender identity, closing forged third-party receipt acceptance while preserving valid recipient delivery receipts.
   - iOS MainActor isolation violations in Multipeer diagnostics/identity helper calls were closed with MainActor-safe bridging in `MultipeerTransport`.
   - Android `MeshRepositoryTest` null-settings assertions were aligned to canonical runtime semantics (`relayEnabled` defaults to `true` when settings are unavailable).
2. WS12.24 deterministic closure gates were canonicalized:
   - the WS12.30 harness verifier set now explicitly includes both `verify_receipt_convergence.sh` and `verify_delivery_state_monotonicity.sh`.
3. WS12.29 diagnostics extraction reliability was hardened without changing closure criteria:
   - `scripts/run5-live-feedback.sh` now retries iOS diagnostics pulls and requires near-stable file-size confirmation before accepting artifacts.
4. Scope classification:
   - reliability/test-orchestration hardening only; no net-new feature scope expansion.
5. Milestone closure gates remain unchanged:
   - synchronized physical Android+iOS evidence is still mandatory for `R-WS12-04/05/06`, `R-WS12-29-01`, and `R-WS12-29-02`.
   - Docker and wireless-ADB prerequisites remain environment-gated.

---

## 1) Planning policy alignment (repo philosophy)

This plan follows repository policy from `CONTRIBUTING.md` and `CLAUDE.md`:

1. LoC estimates only. No time-based estimates.
2. Relay and messaging remain coupled.
3. Every node is a relay-capable network participant.
4. Internet improves transport and bootstrap but is never a hard dependency.
5. Keep policy in Core; keep platform adapters thin.
6. Complete started functionality before adding net-new features.

---

## 2) Locked product decisions for v0.2.0

These are confirmed and treated as contract inputs:

1. Feature scope is text messaging only.  
   Large sync batches (1000+ messages) are reliability scenarios, not new media scope.
2. All nodes/relays are open.  
   User can disable mesh participation entirely, which disables inbound/outbound messaging and relay participation.
3. Messages must retry indefinitely until delivered.
4. Route expansion is not fixed-width; prioritize peers/relays by best "recipient seen most recently" signal.
5. No TTL-based give-up for undelivered messages.
6. Priority objective is highest delivery success first; balanced optimization later.
7. Network remains decentralized; headless/identity-less nodes provide rendezvous/backhaul utility.
8. End-to-end encryption is mandatory for user message content.
9. Platform coverage target is broad (mobile + desktop families), with legacy-device usefulness as a design objective.
10. Public beta readiness is required (TestFlight and equivalent channels).
11. Headless mode must have parity in relay/network behavior; UI omits user identity panes when identity is absent.
12. Message history should behave as immutable/distributed history while preserving reliable delivery semantics.

Follow-up locks:

1. Message payload target cap: 8 KB.
2. Disk pressure policy: dynamic quota + rolling purge; never let SCM storage push device past 90% used.
3. Anti-abuse: low effort, high reward guardrails only for alpha.
4. iOS floor: keep iOS 17 for v0.2.0.
5. Desktop target: full GUI parity (not CLI-only parity).

---

## 3) Numeric clarifications

1. `8 KB` = `8192` bytes.
2. For ASCII payloads, max length is `8192` characters.
3. For UTF-8 multi-byte characters, max character count is lower than `8192`.

Historical baseline reference (pre-WS1):

- `core/src/message/codec.rs` previously set `MAX_PAYLOAD_SIZE = 64 * 1024`.

Current v0.2.0 state:

- `core/src/message/codec.rs` enforces `MAX_PAYLOAD_SIZE = 8 * 1024`.

---

## 4) Verified baseline gaps to close before milestone completion

1. Core compile is currently broken due to type mismatch and lock misuse.
2. Retry pipeline is finite/capped in active swarm path; terminal failure branch exists.
3. Relay forwarding path can reject when destination is not connected, instead of custody storage.
4. Android WiFi send path includes explicit "Not fully implemented" behavior.
5. iOS Multipeer transport exists but is not fully wired into active send path.
6. Mobile startup currently assumes non-headless operation when swarm starts.
7. Existing outbox/relay retention behavior is not aligned with no-give-up delivery semantics.
8. Documentation and runtime behavior currently drift in key areas.

---

## 5) v0.2.0 non-negotiable outcomes

1. Build integrity: workspace compile, tests, and platform builds pass.
2. Direct-first routing with deterministic fallback order.
3. Infinite eventual retry for undelivered messages.
4. Relay custody store-and-forward integrated in active transport path.
5. Delivery receipt propagation stops duplicate concurrent retry attempts.
6. Android WiFi direct path and iOS Multipeer path are both fully wired.
7. Headless-default behavior when no identity is present.
8. Desktop GUI reaches functional parity with mobile core workflows.
9. Dynamic disk safety policy prevents SCM from pushing device beyond 90% utilization.

---

## 6) Dynamic storage and purge policy (locked)

### 6.1 Objectives

1. Keep eventual delivery as strong as possible.
2. Prevent device storage exhaustion.
3. Prioritize local identity relevance.

### 6.2 Dynamic quota model

Definitions:

- `T`: total device bytes
- `U`: current used device bytes
- `S`: current SCM storage bytes
- `N = U - S`: non-SCM used bytes
- `hard_ceiling = floor(0.90 * T) - N`

`hard_ceiling` is the absolute maximum SCM can use without pushing device above 90% total utilization.

Dynamic target quota:

- If device used <= 20%: target up to 70% of free bytes, bounded by `hard_ceiling`.
- If device used 20-50%: target up to 45% of free bytes, bounded by `hard_ceiling`.
- If device used 50-70%: target up to 25% of free bytes, bounded by `hard_ceiling`.
- If device used 70-80%: target up to 10% of free bytes, bounded by `hard_ceiling`.
- If device used 80-90%: target up to 3% of free bytes, bounded by `hard_ceiling`.
- If device used > 90%: emergency mode; reject new non-critical writes and immediately purge until under bound.

### 6.3 Rolling purge order

When `S > target_quota`, purge oldest-first in this order:

1. Non-identity-related messages.
2. Identity-related messages.

Within each group, purge ordering:

1. Delivered/acknowledged records first.
2. Undelivered records only when necessary under sustained pressure.

Identity-related means `to` or `from` matches local identity.

---

## 7) Route preference policy (locked)

Primary goal: maximize delivery success.

Path order:

1. Direct recipient connection attempt.
2. Relay candidates sorted by:
   - recipient recency signal ("seen recipient last"),
   - relay success score,
   - tie-break by latest successful path.
3. Attempt candidates one-at-a-time in ranked order.
4. If all known candidates fail in current pass, continue cyclic retries indefinitely while refreshing candidate graph.

No terminal "exhausted" final state for undelivered messages.

---

## 8) Completion inventory for started-but-incomplete systems

These systems are already present in code and must be completed in v0.2.0:

1. Core build and UniFFI surface consistency.
2. Routing engine integration into live send path.
3. Relay mailbox custody integration with active swarm loop.
4. Infinite retry state machine replacing finite cap.
5. Android WiFi direct send pipeline.
6. iOS Multipeer send/receive pipeline integration.
7. Headless startup mode parity across app variants.
8. Desktop GUI parity with mobile user workflows.
9. Storage pressure controls and global purge propagation.
10. Receipt fanout convergence for concurrent forwarders.

---

## 9) Workstreams with LoC estimates only

LoC ranges below include implementation + tests + minimal docs updates for each stream.

### WS0 - Trunk recovery and contract consistency

Scope:

1. Fix compile blockers in core.
2. Align UniFFI return types and constructors for contacts/history surfaces.
3. Ensure CI blocks merges when workspace compile fails.

Estimate:

- `180-320 LoC`

Acceptance:

1. `cargo test --workspace --no-run` succeeds.
2. Android and iOS verification builds pass.

### WS1 - Message cap and infinite retry engine

Scope:

1. Set payload cap to 8 KB.
2. Replace finite retry cap with persisted infinite retry behavior.
3. Remove terminal delivery-exhausted branch for undelivered message flows.

Estimate:

- `450-700 LoC`

Acceptance:

1. Undelivered messages continue retries across app restarts.
2. No terminal drop due to retry counter exhaustion.

### WS2 - Route preference integration (direct-first + recency ranking)

Scope:

1. Wire routing policy into live swarm send path.
2. Build recipient recency scoring from ledger/identity signals.
3. Add deterministic route decision logging/reason codes.

Estimate:

- `500-850 LoC`

Acceptance:

1. Each send attempt records selected route and reason.
2. Route ordering follows locked ranking policy.

### WS3 - Relay custody store-and-forward in active path

Scope:

1. Integrate relay server/client custody semantics with active swarm path.
2. Accept and persist messages when destination is offline.
3. Deliver on pull/reconnect and mark custody transitions.

Estimate:

- `850-1300 LoC`

Acceptance:

1. Offline recipient receives message after reconnect without sender manual resend.
2. Relay no longer rejects solely because destination is currently disconnected.

### WS4 - Receipt convergence and retry-stop propagation

Scope:

1. Broadcast delivery completion marker to active forwarders.
2. Purge duplicate pending attempts network-wide when final receipt is observed.
3. Keep message history state coherent with receipt transitions.

Estimate:

- `300-520 LoC`

Acceptance:

1. No repeated forward attempts after final delivery convergence.
2. Pending queues are cleaned on all active forwarders.

### WS5 - Dynamic storage pressure controls

Scope:

1. Implement dynamic quota calculation and pressure modes.
2. Implement rolling purge with identity-priority ordering.
3. Add emergency write-throttle behavior for >90% device usage scenarios.

Estimate:

- `700-1100 LoC`

Acceptance:

1. SCM storage does not push device beyond 90% utilization.
2. Purge ordering matches locked policy.

### WS6 - Android WiFi direct path completion

Scope:

1. Complete WiFi send pipeline where currently partial.
2. Integrate with primary send flow and fallback sequencing.
3. Add transport soak tests for local high-throughput messaging sync.

Estimate:

- `500-850 LoC`

Acceptance:

1. Android local high-throughput transport is functional in normal send path.
2. Fallback to BLE/swarm occurs automatically when unavailable.

### WS7 - iOS Multipeer path completion

Scope:

1. Wire Multipeer transport into active send/receive path.
2. Integrate with existing fallback graph.
3. Add reconnection and throughput tests.

Estimate:

- `520-900 LoC`

Acceptance:

1. iOS local high-throughput path participates in standard delivery flow.
2. Fallback behavior remains deterministic and loss-safe.

### WS8 - Headless-default mode and role parity

Scope:

1. Make no-identity startup enter relay-only mode by default.
2. Ensure full network behavior parity between headless and full nodes.
3. Apply UI role gating: hide Messages/Contacts panes when identity absent.

Estimate:

- `600-1000 LoC`

Acceptance:

1. Fresh install without identity launches relay-only UX.
2. Enabling identity upgrades to full app behavior without network role loss.

### WS9 - Desktop full GUI parity

Scope:

1. Deliver desktop GUI workflows matching mobile core flows:
   - onboarding/identity,
   - contacts,
   - chat/send/receive,
   - mesh dashboard,
   - relay-only mode.
2. Package for Windows/macOS/Linux in parity-ready form.

Estimate:

- `1800-3000 LoC`

Acceptance:

1. Desktop users can execute all core workflows without CLI fallback.
2. Role-based UI parity (full vs relay-only) is consistent across desktop and mobile.

### WS10 - Minimal anti-abuse guardrails (alpha level)

Scope:

1. Per-peer token bucket/rate limits.
2. Global inflight limits for relay queue protection.
3. Duplicate suppression and cheap abuse heuristics.

Estimate:

- `300-550 LoC`

Acceptance:

1. Single abusive peer cannot starve relay processing.
2. Legitimate low-volume users remain unaffected.
3. Guardrail trigger reasons are observable in runtime logs.

### WS11 - Public beta readiness surfaces

Scope:

1. Delivery-state UX clarity (`pending`, `stored`, `forwarding`, `delivered`).
2. Diagnostics export quality for family/beta testers.
3. App-store/tester-facing reliability notes and permissions rationale.

Estimate:

- `550-900 LoC`

Acceptance:

1. Testers can understand message state without developer guidance.
2. Install + first-message path is reliable and observable.

Execution note (2026-03-03):

1. WS11 implemented in Android+iOS surfaces with explicit delivery-state mapping, tester-focused diagnostics bundles, and permissions/reliability rationale text.
2. Residual-risk review and evidence are recorded in `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` under the WS11 section.

### WS12 - Test matrix expansion and docs parity lock

Scope:

1. Expand deterministic integration tests for offline/partition scenarios.
2. Add cross-platform parity checks for role mode and transport fallback.
3. Update canonical docs/backlog to match runtime behavior.

Estimate:

- `700-1200 LoC`

Acceptance:

1. Cross-platform parity checks pass in CI.
2. Canonical docs no longer contradict runtime behavior for milestone scope.
3. Team can reproduce WS12 validation via a documented command matrix/script.

Execution note (2026-03-03):

1. Added deterministic offline/partition integration suite: `core/tests/integration_offline_partition_matrix.rs`.
2. Added CI-enforced parity gates for deterministic offline/partition suites (including live custody reconnect), Android role/fallback, desktop/WASM role mode, and iOS transport/role checks.
3. Added reproducible validation runner: `scripts/verify_ws12_matrix.sh`.
4. Canonical docs + residual-risk register were updated in the same run to lock docs/runtime parity.
5. WS12.5 burndown audit reconciled remaining doc/backlog drift and re-validated custody reconnect evidence used for `R-WS3-01` closure.
6. WS12.7 live runtime sanity sweep (2026-03-02 HST) added Android runtime hotfixes for beacon hint preservation, outbox flush serialization, and uptime fallback; remaining live gap is relay-fleet version skew during rollout.
7. WS12.8 runtime recheck (2026-03-02 HST) confirmed GCP relay identity rotation is active post-redeploy, but live custody reconnect validation (`integration_relay_custody -- --include-ignored`) now times out and requires follow-up triage.
8. WS12.10 re-baseline (2026-03-03 HST) closed the live custody timeout signal with stable 3/3 reruns, closed relay rollout-skew risk with fresh `0.2.0` relay evidence, and added iOS main-thread I/O hotfix for startup diagnostics path.
9. WS12.13 wave-2 consolidation (2026-03-03 HST) normalized mixed-doc checklists into status-tagged guidance, migrated transport future tasks into explicit roadmap metadata, and reconciled legacy validation scripts with canonical WS12 matrix gates.
10. WS12.11 runtime triage (2026-03-03 HST) documented iOS relay visibility flapping patterns and established state/race hardening follow-up queue.
11. WS12.14 diagnosis-only Bluetooth run (2026-03-03 HST) documented Android/iOS BLE-only path instability (Android BLE address-type mismatch flood + zero-advertisement windows, iOS multipeer invite timeout loops, and WiFi transport traces during attempted sessions) with explicit follow-up gates for strict BLE-only validation and instrumentation.
12. WS12.16 wave-2 runtime hardening pass (2026-03-03 HST) implemented Android BLE callback/permit race fixes, Android+iOS `delivery_attempt` diagnostics timelines, iOS relay state/debounce instrumentation, and iOS Multipeer invite storm guardrails.

---

## 10) Total v0.2.0 planning estimate

Planned implementation envelope:

- Low range: `7,950 LoC`
- High range: `12,390 LoC`

Notes:

1. Estimate intentionally excludes generated code.
2. Estimate includes tests and required documentation updates.
3. If desktop packaging complexity expands, additional LoC should be tracked in a scoped addendum.

---

## 11) Definition of Done (v0.2.0)

v0.2.0 is done when all are true:

1. Workspace compiles cleanly; core/mobile/desktop build gates pass.
2. Messages retry indefinitely until delivered (no terminal retry exhaustion).
3. Direct-first routing and recency-based relay fallback are active.
4. Relay custody path stores offline messages and delivers on reconnect.
5. Receipt propagation halts duplicate forward attempts across participating nodes.
6. Android WiFi direct and iOS Multipeer paths are fully wired.
7. Headless-default startup works with UI/network role parity.
8. Desktop GUI parity is available across Windows/macOS/Linux targets.
9. Dynamic storage policy protects device usage ceiling and purges by locked ordering.
10. Public beta readiness criteria are satisfied for external testers.
11. Residual risk closure sweep is complete: each risk in `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` is either `Closed` or explicitly `Accepted/Deferred` with rationale and verification evidence.

---

## 12) Immediate execution order

To reduce risk and unblock integration quickly:

1. WS0
2. WS1
3. WS2
4. WS3
5. WS4
6. WS5
7. WS6 + WS7 (in parallel where possible)
8. WS8
9. WS9
10. WS10
11. WS11
12. WS12
13. Post-WS12 residual risk closure sweep (release gate)
14. WS13 kickoff (v0.2.1) - Single Active Device/Tight Pairing

This ordering is for dependency flow only. Execution planning remains LoC-scoped, not time-scoped.

---

## 13) Residual Risk Tracking (Active)

Residual risks discovered during v0.2.0 execution are tracked in:

- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

Each phase must update the register before phase signoff, and the release gate must run a final closure sweep after WS12.

---

## 14) Post-WS12 Residual Risk Closure Sweep (Release Gate)

Run this sweep immediately after WS12 and before calling v0.2.0 complete:

1. Enumerate all open residual risks from `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`.
2. Attempt closure of release-critical risks that can be fixed without net-new feature scope.
3. Re-run verification for any risk addressed during the sweep.
4. For remaining open risks, classify as `Accepted` or `Deferred` with explicit rationale.
5. Record a final go/no-go decision for external alpha distribution based on residual-risk posture.

This sweep is mandatory and is part of v0.2.0 Definition of Done.

---

## 15) Deferred Net-New Feature (Tracked for v0.2.1)

Feature:

- Single Active Device per Identity ("Tight Pairing Architecture")

Decision:

- Deferred from `v0.2.0` to `v0.2.1`.
- Execution anchor in next release: `WS13` (decomposed as `WS13.1`-`WS13.6`).

Why deferred:

1. It is cross-cutting net-new scope (identity schema, contacts metadata, transport protocol, relay registry state machine, custody enforcement, UX error semantics).
2. It conflicts with the `v0.2.0` policy to complete started functionality first.
3. Introducing it during WS6-WS12 would materially increase regression risk for alpha app-store readiness.

Tracking document:

- `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md`
- WS13 kickoff prompt template: `docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md` (section "WS13 Kickoff (v0.2.1) - Tight Pairing start")

`v0.2.0` rule:

1. Planning/docs and backlog tracking are allowed.
2. Enforcement/protocol implementation is out-of-scope unless milestone is explicitly re-scoped.

---

## 16) Optional v0.2.0 Closeout Burndown (No Net-New Product Scope)

If additional closure work is pulled into `v0.2.0`, prioritize only residual-risk and release-ops items that do not expand product scope:

1. `R-WS3-02` / `EC-01`: custody persistence path migration from temp-dir to durable app path + restart recovery verification.
2. `R-WS5-01` / `EC-02`: platform storage snapshot adapters so pressure controls cannot no-op.
3. `R-WS4-02` / `EC-04`: low-cost convergence-marker trust hardening checks.
4. Release synchronization execution (workspace version alignment, tag/release checklist, release notes finalization) using `docs/releases/*`.

Exclusions for this optional closeout:

1. `WS13` Tight Pairing implementation.
2. `WS14` notification implementation.

WS12.6 closeout status (2026-03-03):

1. Completed `R-WS3-02` closure (durable custody persistence default path migration).
2. Completed `R-WS5-01` closure (synthetic storage snapshot fallback to avoid no-op pressure enforcement).
3. Completed `R-WS4-02` closure (convergence-marker validation hardening).
4. Completed release-sync preparation (`docs/releases/*` canonical artifacts + version metadata bump to `0.2.0`).
