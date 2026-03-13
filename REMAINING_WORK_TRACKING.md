# SCMessenger Remaining Work Tracking

Status: Active  
Last updated: 2026-03-13

This is the active implementation backlog based on repository state verified on **2026-03-11**.

Primary delivery target: **one unified Android + iOS + Web app**.

## Repo Governance Lock: Documentation Sync + Build Verification (2026-03-13)

Completed in this pass:

1. [x] Tightened `AGENTS.md` so same-run canonical documentation updates are explicit whenever behavior, scope, risk, scripts, verification commands, or operator workflow change.
2. [x] Tightened `AGENTS.md` so edited-target build verification is mandatory whenever code, bindings, build wiring, or runtime-affecting scripts change.
3. [x] Mirrored the same closeout rules in `.github/copilot-instructions.md` to keep Codex/Copilot policy aligned.
4. [x] Updated the active canonical doc chain to reflect that documentation sync and build verification are release-governance requirements rather than optional cleanup.

Remaining governance expectation:

1. [ ] Enforce these rules on every future change-bearing run and record exceptions only with exact blocking command output and rationale.

Owner policy constraints (2026-02-23):

- Global organic growth (no region-targeted rollout sequence).
- Community-operated infrastructure model (self-hosted and third-party nodes are both valid).
- English-only alpha UI language (i18n expansion tracked as backlog).
- No abuse-control or regional compliance hard gate for alpha.
- Anti-abuse controls are required before beta release.
- Critical UX controls must stay in Android+iOS+Web parity with no temporary lead platform.

## WS13.1 Tight-Pair Kickoff (2026-03-10 UTC)

Completed in this pass:

1. [x] Re-read the required canonical + planned docs before coding.
2. [x] Re-ran WS13 preflight baseline locally:
   - `cargo fmt --all -- --check`
   - `cargo build --workspace`
   - `cargo test --workspace`
   - `./scripts/docs_sync_check.sh`
3. [x] Audited current branch GitHub Actions state:
   - PR `CI` run `22923791535` is `action_required`, confirming the still-open approval/policy blocker is external to WS13 code.
4. [x] Added a WS13.1 -> WS13.6 execution inventory to `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md` with:
   - file targets,
   - test targets,
   - migration implications,
   - acceptance gates.
5. [x] Created `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md` so v0.2.1 carry-forward and WS13-specific risks stay separate from the v0.2.0 residual register.
6. [x] Implemented WS13.1 only:
   - installation-local `device_id` + `seniority_timestamp` persistence,
   - hydrate/initialize/import backfill behavior for pre-WS13 identities,
   - targeted core/wasm identity-surface tests.

Remaining WS13 queue:

1. [x] WS13.2 — Transport/API boundary widened: `SwarmCommand::SendMessage`, `SwarmHandle::send_message`, `SwarmBridge::send_message`, `RelayRequest`, and `Contact` all now carry `recipient_identity_id`/`intended_device_id`/`last_known_device_id` as `Option<String>`. All existing callers updated with `None, None`; `#[serde(default)]` ensures pre-WS13 relay nodes continue to interoperate. Mobile adapter call-sites (Android/iOS Kotlin/Swift consumers generated from `api.udl`) must be regenerated — use `void send_message(string peer_id, bytes data, string? recipient_identity_id, string? intended_device_id)` as the source of truth.
2. [x] WS13.3 — Registration protocol (`/sc/registration/1.0.0`) + signature verification. `IronCoreBehaviour` now exposes an additive registration request/response protocol with canonical payload serialization, signed registration/deregistration helpers, `SwarmHandle::{register_identity,deregister_identity}` wiring, and fail-closed validation for malformed identity IDs, malformed UUIDv4 device IDs, peer/identity mismatches, invalid signatures, and invalid deregistration state. Targeted unit + integration tests cover success and rejection paths. Residual: no registry mutation/anti-replay enforcement yet; WS13.4 owns persisted active-device state. **Merged in PR83 (2026-03-12) — consolidated build verified (528 tests, 0 failures).**
3. [ ] WS13.4 — Relay registry state machine + custody enforcement. **In progress on `codex/ws13-ws14-hourly-20260313-2118`.** Core state machine and relay enforcement landed in this run:
   - `RelayCustodyStore` now persists `RegistrationState::{Active,Handover,Abandoned}` per identity and auto-collapses stale handovers (>15 days) to `Abandoned`.
   - Active-device collisions now reject by default and only permit stale takeover after the 14-day window instead of silently auto-handovering.
   - Identity abandonment now purges accepted custody backlog for that identity.
   - verified `/sc/registration/1.0.0` requests now mutate registry state instead of remaining ack-only.
   - relay accept path now enforces `recipient_identity_id` + `intended_device_id` when both are present and preserves compatibility mode for legacy no-device requests.
   - `IronCore.get_registration_state()` / `RegistrationStateInfo` added as additive UDL surface.
   - Rust verification passed:
     - `cargo fmt --all -- --check`
     - `cargo build --workspace`
     - `cargo clippy --workspace`
     - `cargo test --workspace`
   - Additional verification passed:
     - `bash ./iOS/verify-test.sh`
   - Still blocking phase completion:
     - `cd android && ./gradlew assembleDebug` failed: missing Android SDK path (`ANDROID_HOME` / `android/local.properties`)
     - `cd android && ./gradlew testDebugUnitTest` failed: Gradle wrapper lock under `~/.gradle` not writable in sandbox
     - `cd android && ./gradlew lintDebug` failed: same Gradle wrapper lock permission failure
   - Confidence: **89%** overall (Android verification is still missing, so do not mark complete).
4. [ ] WS13.5 — Handover/abandon queue migration + sender-facing rejection UX. Android/iOS adapter follow-through remains blocked behind Android verification, but core abandonment purge is now implemented and relay rejection errors already propagate through existing relay-response plumbing.
5. [ ] WS13.6 — Compatibility/migration matrix, runbook, and acceptance lock. Unblocked after WS13.3–WS13.5 complete.

## v0.2.1 Critical Bug Fixes (2026-03-12)

Completed in this pass:

1. [x] **Android Duplicate Messages**: Fixed UI duplication bug by properly emitting reconciled message IDs from `MeshRepository` and deduplicating by content/timestamp in `ChatViewModel.loadMessages()`.
2. [x] **iOS CryptoError (Error 4)**: Traced to stale bootstrap data; resolved by updating static fallbacks and implementing dynamic ledger-driven discovery in `MeshRepository.swift`.
3. [x] **iOS Power & Log Optimization**: Increased adaptive interval for high battery levels in `IosPlatformBridge.swift` and simplified noisy power profile logs.

## v0.2.0 Critical Bug Fixes (2026-03-09)

Completed in this pass:

1. [x] **NAT Traversal**: Added relay server behavior to all nodes for cellular↔WiFi messaging
2. [x] **BLE Reliability**: Fixed DeadObjectException with proper subscription tracking
3. [x] **Delivery Status**: Eliminated false positives where BLE ACK was treated as full delivery
4. [x] **Android UI**: Fixed keyboard covering chat input with proper IME padding
5. [x] **Transport Optimization** (2026-03-10): Faster BLE/WiFi switching with reduced timeouts, aggressive retry backoff, enhanced transport logging
6. [x] **Android Mesh UI Scrolling** (2026-03-10): Converted DashboardScreen to LazyColumn for proper scrolling with large peer lists
7. [x] **Android ID Normalization** (2026-03-10): Standardized peer ID handling to fix "Contact not found" messaging issues
8. [x] **NAT Traversal & BLE Stability** (2026-03-13): Restored relay routing, throttled BLE beacons, fixed Android connect-on-demand.
9. [x] **BLE Freshness Profiling + run5 Visibility Clarification** (2026-03-13): Android now prefers fresh BLE observations over stale cached hints, promotes to unfiltered BLE scan after 20s of zero mesh advertisements, and `run5.sh` now splits iOS app/system logs while treating unknown own IDs as collector gaps instead of mesh failures.

Outstanding items:

1. [ ] Monitor Android-to-iOS delivery for "Missing Direction" receipts
2. [ ] Verify iOS UI no longer freezes during high-density peer discovery
3. [ ] Test BLE reconnection scenarios end-to-end with new 5s throttles
4. [ ] Verify parallel transport attempts reduce WiFi→BLE transition time to < 2s
5. [ ] Test Mesh tab scrolling with 50+ discovered peers
6. [ ] Re-run upgraded `run5.sh` on fresh artifacts and close the remaining "unknown own ID in current log window" ambiguity where full mesh transport evidence exists but passive identity capture is incomplete.
7. [ ] Bring iOS BLE route profiling to the same freshness-observation standard if stale BLE hint churn reappears; current explicit freshness cache is Android-only, while iOS still relies primarily on connected-peer preference plus runtime transport evidence.
8. [ ] Unify Android BLE fallback telemetry so the accepted-send target reflects the actual connected GATT address used on wire; current logs can still show the requested stale MAC while `BleGattClient` success callbacks fire for the fresher connected device.

## v0.2.0 Execution Residual Register

Residual risks from completed v0.2.0 phases (currently through WS12.5 burndown audit) are tracked in:

- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

Do not start the next v0.2.0 phase without checking the corresponding entry gate in that register.

1. [ ] Ensure message history is not cleared on app startup. Messages should persist across app restarts.

## WS12.39 Closeout Burndown Re-Baseline (2026-03-10 UTC)

Completed in this pass:

1. [x] Reconciled the canonical docs, open issues, workflow runs, and branch inventory into one current-state closeout view.
2. [x] Restored the local Rust/WASM verification baseline:
   - removed the CLI trailing-whitespace drift blocking `cargo fmt --all -- --check`,
   - added `SwarmEvent::PortMapping(_)` handling in `wasm/src/lib.rs` so `cargo build --workspace` succeeds again.
3. [x] Confirmed current GitHub issue-tracker truth:
   - open issues are automation-only (`#38`, `#39`, `#40`, `#42`),
   - no open issues currently represent canonical WS12/v0.2.0 closeout items,
   - no open issues currently mix `WS13` / `WS14` into active v0.2.0 scope.
4. [x] Confirmed current workflow-truth split:
   - PR `CI` remains `action_required` because of GitHub approval/policy settings,
   - `main` still has real CI failures (docs sync drift, Rust fmt drift, WASM event-match drift, iOS MainActor isolation, Docker Android-unit-test path drift).

Still open after this pass:

1. [ ] Maintainer GitHub cleanup:
   - close/recreate automation-only issues (`#38`, `#39`, `#40`, `#42`) so the tracker reflects real v0.2.0 work,
   - create/normalize labels and milestones for `v0.2.0 alpha baseline`, repo hygiene, and deferred `v0.2.1` planning,
   - apply branch protection / required checks on `main`,
   - resolve the approval/policy setting behind `action_required` PR runs,
   - prune stale non-`main` branches after merge/closure decisions.
2. [ ] Non-device CI cleanup still needed in-repo:
   - re-run iOS verification on a macOS host / CI now that MainActor-safe helper fixes are in place for `BLEPeripheralManager`, `ContactsViewModel`, `TopicManager`, and `IosPlatformBridge`,
   - re-run Docker Integration Suite now that the Android-unit-test host-library copy path in `docker/docker-compose.test.yml` matches the workspace release artifact layout.
3. [ ] Physical-device WS12 closure evidence is still required:
   - `R-WS12-29-01` iOS send-path crash non-repro on latest binary,
   - `R-WS12-29-02` stale-route / stale-BLE-target convergence,
   - `R-WS12-04`, `R-WS12-05`, `R-WS12-06` synchronized Android+iOS relay/delivery/BLE evidence.

## WS12.38 Cross-Platform Status Sync Convergence (2026-03-09 HST)

Completed in this pass:

1. [x] Diagnosed "pending status" hang on iOS for messages received by Android.
2. [x] Fixed `history_sync_data` handler on iOS/Swift and Android/Kotlin to update `delivered` status for existing `sent` records instead of skipping them.
3. [x] Ensured that history sync acts as a reliable eventual-consistency fallback when point-to-point delivery receipts are lost.
4. [x] Verified Android compilation with corrected coroutine scope for EventBus emissions.

Still open:

1. [ ] Monitor real-world convergence on physical devices to confirm "stuck pending" messages resolve on next sync trigger.

## WS12.36 Repo/GitHub Operating-Model Planning Audit (2026-03-07 UTC)

Completed in this pass:

1. [x] Performed a planning-only audit of repo documentation, GitHub features, issue tracker state, Actions topology, contributor workflow, and agent-context surfaces.
2. [x] Captured the first-pass execution blueprint in `docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`.
3. [x] Revalidated local baseline commands before planning follow-up work (`cargo fmt --all -- --check`, `cargo build --workspace`, `cargo test --workspace`, `./scripts/docs_sync_check.sh`).

Execution follow-ups opened by this audit:

1. [x] Tighten the canonical documentation chain and remove high-value stale/current-state drift from active entrypoints.
2. [ ] Reset GitHub Issues around a fresh taxonomy, labels, milestones, and clean issue intake forms.
3. [x] Add missing GitHub repo health/configuration surfaces (`CODEOWNERS`, support policy, Dependabot, issue config/forms, Copilot instructions).
4. [x] Repair the repo-controlled CI/workflow operating model so required checks are clearer and PR-noisy workflows are removed from the default PR path.
5. [x] Rewrite contributor/security/agent guidance to remove stale claims and duplicated instructions.
6. [ ] Clean up stale branches after open-PR/open-issue decisions are made.

Progress update in this pass:

1. [x] Added first-pass GitHub contributor routing/config surfaces:
   - `SUPPORT.md`
   - `.github/CODEOWNERS`
   - `.github/ISSUE_TEMPLATE/config.yml`
2. [x] Rewrote GitHub-facing contributor/security entrypoints to match current alpha reality:
   - `README.md`
   - `CONTRIBUTING.md`
   - `SECURITY.md`
   - `.github/pull_request_template.md`
3. [x] Made repository GitHub-facing docs/config explicitly treat `v0.2.0` as the active alpha baseline, with `WS13` / `WS14` deferred to `v0.2.1`.
4. [x] Added missing repo-controlled GitHub health/configuration surfaces:
   - `.github/dependabot.yml`
   - `.github/copilot-instructions.md`
   - issue forms under `.github/ISSUE_TEMPLATE/*.yml`
5. [x] Tightened repo-controlled workflow/docs hygiene:
   - `scripts/docs_sync_check.sh` now checks a broader canonical-doc surface and rejects machine-local paths.
   - `scripts/docs_sync_check.sh` no longer masks broken nested-doc relative links via repo-root fallback and now validates `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` link integrity directly.
   - `docker-publish.yml` no longer runs on pull requests.
   - `docker-test-suite.yml` is now main/scheduled/manual only.
   - `release.yml` is now explicitly CLI-scoped.
6. [ ] Remaining GitHub-hosted follow-up from item 2:
   - create/normalize labels and milestones in GitHub,
   - close/recreate stale automation issues,
   - apply branch protection and required-check policy on `main`,
   - resolve the repository approval/policy setting behind `action_required` PR runs.

## WS12.18 Alpha Readiness Closure Follow-ups (2026-03-03 HST)

Completed in this pass:

1. [x] Rust clippy strict cleanup for workspace `--lib --bins --examples` gate.
2. [x] Android lint hard-blocker remediation (`MissingPermission`, `NewApi`) and clean `:app:lintDebug` pass.
3. [x] Cross-platform function completeness and interop matrix generation (`docs/INTEROP_MATRIX_V0.2.0_ALPHA.md` via `scripts/generate_interop_matrix.sh`).
4. [x] Historical artifact relocation from repo root to `reference/historical/` with provenance index.

Interop follow-ups from generated matrix (now completed):

1. [x] CLI parity: identity backup import/export commands wired to `IronCore.export_identity_backup` + `IronCore.import_identity_backup`.
2. [x] CLI parity: explicit message terminal-state mark path (`mark_message_sent`) wired in CLI.
3. [x] WASM/Desktop parity: local nickname override support (`ContactManager.set_local_nickname`) wired.
4. [x] CLI parity: explicit history clear action (`HistoryManager.clear`) wired.
5. [x] WASM/Desktop parity: swarm external address visibility (`SwarmBridge.get_external_addresses`) wired.
6. [x] CLI/relay diagnostics parity: `get_peers`, `get_listeners`, `get_connection_path_state`, and `export_diagnostics` wired in CLI status/API surfaces.
7. [x] Adapter consumption: `MeshService.reset_stats`, `HistoryManager.enforce_retention`, and `HistoryManager.prune_before` now consumed in platform adapters.

## WS12.19 Doc/Folder Cleanup Correction (2026-03-03 HST)

Completed in this pass:

1. [x] Corrected iOS script relocation drift: restored active operational scripts to `iOS/` (`build-device.sh`, `install-device.sh`, `install-sim.sh`).
2. [x] Kept stale iOS scripts in historical scope only (`docs/historical/iOS/scripts/build-rust.sh`, `docs/historical/iOS/scripts/verify-build-setup.sh`) and added archive clarification README.
3. [x] Updated active docs to remove stale references to the legacy iOS setup-check script and point to `bash ./iOS/verify-test.sh`.

## WS12.20 Alpha Readiness Completion Sweep (2026-03-03 HST)

Completed in this pass:

1. [x] Closed all WS12.18 interop follow-up gaps (CLI + WASM + adapter-consumption wiring).
2. [x] Added active scripts operations guide (`scripts/README.md`) covering 5-node, launch/control, and diagnosis workflows.
3. [x] Confirmed full local sanity gate pass set:
   - `cargo check --workspace`
   - `cargo clippy --workspace --lib --bins --examples -- -D warnings`
   - `cd android && ./gradlew :app:generateUniFFIBindings :app:compileDebugKotlin :app:lintDebug`
   - `bash ./iOS/verify-test.sh`
   - `cd wasm && wasm-pack build`
4. [x] Reduced active unchecked checklist items to live-validation/environment evidence items only (no remaining static adapter wiring gaps).

## WS12.21 Pairwise Deep-Dive Status Sweep (2026-03-03 HST)

Executed in this pass:

1. [x] Deep-dive script sweep on latest available artifacts:
   - `bash ./scripts/correlate_relay_flap_windows.sh ios_diagnostics_latest.log logs/5mesh/gcp.log`
   - `bash ./scripts/verify_relay_flap_regression.sh ios_diagnostics_latest.log`
   - `bash ./scripts/verify_receipt_convergence.sh android_mesh_diagnostics_device.log ios_diagnostics_latest.log`
   - `bash ./scripts/verify_ble_only_pairing.sh android_logcat_latest.txt ios_diagnostics_latest.log`
2. [x] Fresh dual-device probe attempt:
   - `IOS_TARGET=device IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=20 GCP_RELAY_CHECK=1 bash ./scripts/live-smoke.sh`
   - Result: Android device available; iOS device currently `unavailable` in `xcrun devicectl`, probe cannot complete on physical iOS.
3. [x] Simulator fallback probe executed for additional runtime context:
   - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=20 GCP_RELAY_CHECK=1 bash ./scripts/live-smoke.sh`
   - Artifact directory: `logs/live-smoke/20260303-005207/`

Current status across the five tracked pairings:

1. [x] `Core -> Android` adapter/function path parity: closed (no static matrix gaps).
2. [x] `Core -> iOS` adapter/function path parity: closed (no static matrix gaps).
3. [x] `Core -> WASM/Desktop` adapter/function path parity: closed (no static matrix gaps).
4. [ ] `Android <-> iOS` direct/relay delivery+receipt path continuity: still open pending synchronized live-device artifact with message timeline markers (`R-WS12-05` / `R-WS12-04` carry-forward).
5. [ ] `Android <-> iOS` strict BLE-only pairing/send/receipt continuity: still open pending synchronized live BLE-only artifact bundle (`R-WS12-06` carry-forward).

## WS12.22 Android+iOS Crash + Stability Hardening Sweep (2026-03-03 HST)

Completed in this pass:

1. [x] Pulled fresh iOS+Android runtime artifacts for crash/non-delivery diagnosis:
   - `logs/pairwise/ios-debug-detach-20260303-014559`
   - `logs/pairwise/android-usb-pull-20260303-014849`
2. [x] iOS send-path crash mitigation applied in BLE transport and repository fallback handling:
   - `BLEPeripheralManager` force-unwrap removal + explicit send result flow.
   - Main-queue delegate/state handling for BLE central/peripheral managers.
   - Peripheral/central reconnect and characteristic-rediscovery safeguards.
3. [x] Android crash-safety cleanup applied:
   - removed remaining Kotlin `!!` usage in app sources,
   - reduced BLE advertiser restart churn,
   - added reconnect path in BLE GATT send preconditions.
4. [x] Added bounded stale pending-outbox drop policy (Android+iOS) to reduce unbounded retry noise from legacy queue entries while preserving normal retry behavior for active messages.
5. [x] Revalidated local sanity gates after hardening:
   - `cd android && ./gradlew :app:compileDebugKotlin :app:lintDebug` (pass; lint errors remain zero),
   - `bash ./iOS/verify-test.sh` (pass; 0 warnings in this run),
   - `bash ./scripts/generate_interop_matrix.sh` (pass).

Still open after this pass:

1. [ ] Capture new synchronized physical-device Android+iOS send/receipt artifacts after these fixes to confirm iOS crash non-repro and receipt convergence.
2. [ ] Re-run deterministic pairwise verifiers on new artifacts (`verify_receipt_convergence.sh`, `verify_ble_only_pairing.sh`, `correlate_relay_flap_windows.sh`) to close `R-WS12-04/05/06`.

## WS12.23 Pending-Outbox Synchronization Reliability Pass (2026-03-03 HST)

Completed in this pass:

1. [x] Closed the "new message sends while older pending stay stuck" queue-trigger gap on Android+iOS by promoting same-peer pending entries on active-connection signals (`peer_identified`, BLE identity-read, and iOS connected-event flow).
2. [x] Expanded pending promotion matching to both canonical `peerId` and cached `routePeerId`, so queued entries tied to route aliases also retry immediately.
3. [x] Revalidated local compile/build sanity after transport-queue changes:
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass),
   - `bash ./iOS/verify-test.sh` (pass; 3 warnings, non-fatal).

Still open after this pass:

1. [ ] Capture synchronized physical Android+iOS message sessions that demonstrate older pending entries draining immediately once peer connectivity is active.
2. [ ] Re-run convergence verifiers on new artifacts and close residual runtime transport risks (`R-WS12-04/05/06`) when evidence confirms deterministic behavior.

## WS12.24 Sender-State Convergence + Conversation Swipe-Delete Parity (2026-03-03 HST)

Planned in this pass:

1. [ ] Reproduce and isolate iOS-sender false `stored` status when Android recipient has already ingested the message.
   - Capture synchronized Android+iOS+relay artifacts for one message ID where Android renders the message while iOS sender does not converge to `delivered`.
2. [ ] Close iOS -> Android sender-state convergence gap end-to-end.
   - Validate Android receipt/ack emission, iOS receipt ingest, and message-ID correlation in iOS history-state updates.
   - Acceptance: iOS sender state transitions to `delivered` in-session and does not regress back to `stored` for that message.
3. [x] Add deterministic regression gate for recipient-ingest vs sender-state mismatch.
   - Outcome (2026-03-06 UTC): Canonical closure flow now includes both `verify_receipt_convergence.sh` and `verify_delivery_state_monotonicity.sh` in `scripts/run5-live-feedback.sh` deterministic gates, so recipient-ingest proof cannot pass with sender-state regression.
4. [x] Align conversation deletion UX to swipe parity (iOS + Android).
   - Outcome (2026-03-03 HST): Android conversation rows now support end-to-start swipe-to-delete with confirmation dialog, matching existing iOS swipe-delete behavior.
5. [ ] Validate swipe delete flow with platform evidence and tests.
   - Android: verify swipe -> confirm -> `clearConversation(peerId)` path and list refresh behavior.
   - iOS: verify swipe -> confirm -> `clearConversation(peerId)` path and list refresh behavior.

## WS12.25 Mega-Update Intake: Pending-Sync RCA + Node-Role Unification (2026-03-03 HST)

Completed in this pass:

1. [x] Reviewed updated `run5.sh` plus associated runtime artifacts to diagnose why older pending entries stay undelivered while queue activity remains high.
   - Key evidence set:
     - `logs/5mesh/latest/android.log` (repeated `forwarding -> stored`, core/relay failures, local fallback accepts, repeated flush triggers),
     - `logs/pairwise/ios-debug-detach-20260303-014559/pending_outbox.json` (same-peer queued items retaining stale route/address hints).
2. [x] Implemented route-hint/route-candidate hardening in Android+iOS `MeshRepository`:
   - refresh persisted route hints when values change (not only when absent),
   - pass inbound observed route/listener hints into receipt sends,
   - build recipient-key-aware route candidates and reject relay/mismatched candidates,
   - block direct-chat sends to known relay/bootstrap identities.
3. [x] Unified dashboard role buckets across Android+iOS:
   - `Node` (full identity),
   - `Headless Node` (no identity; relay/headless grouped together).
4. [x] Revalidated local build/verification after changes:
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass),
   - `bash ./iOS/verify-test.sh` (pass).

Still open after this pass:

1. [ ] Capture fresh synchronized physical Android+iOS artifacts post-fix to confirm previously stuck pending items now drain under active connectivity.
2. [ ] Re-run deterministic convergence checks against fresh artifacts:
   - `scripts/verify_receipt_convergence.sh`
   - `scripts/verify_ble_only_pairing.sh`
   - `scripts/correlate_relay_flap_windows.sh`
3. [ ] Close WS12.24 sender-state convergence gate with message-ID-correlated evidence (`recipient ingest` => sender `delivered`, no persistent `stored` regression).

## WS12.26 Sender-State + Conversation Preview Convergence Hotfix (2026-03-03 HST)

Completed in this pass:

1. [x] Closed receipt-path UI refresh gap on Android.
   - `MeshRepository.onReceiptReceived` now emits refreshed `MessageRecord` through `messageUpdates` after delivery-state mutation and pending-outbox cleanup.
   - `ConversationsViewModel` now reloads on `MessageEvent.Delivered`/`MessageEvent.Failed` to keep chat-list state aligned with receipt callbacks.
2. [x] Closed receipt-path UI refresh gap on iOS.
   - `MeshRepository.onDeliveryReceipt` now emits refreshed `MessageRecord` through `messageUpdates` after receipt-driven history/pending updates.
3. [x] Hardened iOS conversation-row preview selection.
   - Conversation list now chooses newest preview by max timestamp from a bounded recent sample, rather than relying on position-based ordering assumptions.
4. [x] Revalidated regression-safety for the updated Android paths:
   - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.ChatViewModelTest" --tests "com.scmessenger.android.ui.viewmodels.ConversationsViewModelTest"` (pass).
5. [x] Closed Swift strict-concurrency regression in generated UniFFI bindings.
   - `iOS/SCMessenger/SCMessenger/Generated/api.swift` now uses `nonisolated(unsafe)` on `FfiConverter` helper statics.
   - `core/src/bin/gen_swift.rs` now enforces the same rewrite post-generation to keep future binding refreshes non-regressive.
   - `bash ./iOS/verify-test.sh` now passes after regeneration/copy.

Still open after this pass:

1. [ ] Validate the WS12.26 hotfix on live Android+iOS artifact capture (passive logs acceptable) and confirm no message remains `stored` after correlated recipient receipt.
2. [ ] Close WS12.24 sender-state convergence gate using synchronized post-hotfix evidence.

## WS12.27 Node-Role Classification Correction + Trip Readiness Validation (2026-03-03 HST)

Completed in this pass:

1. [x] Added explicit issue intake: iOS could render a confirmed full iOS-sim peer as `Headless Node`.
2. [x] Root-cause fix on iOS + Android `MeshRepository` peer-identification flow:
   - `/headless/` agent is now treated as provisional when transport identity resolves successfully.
   - peers with resolved identity are promoted to full-node classification even if prior identify agent hinted headless.
3. [x] Relay classification guardrail tightened on iOS + Android:
   - `isKnownRelay` now treats only bootstrap peers and non-full dynamic relay peers as relay-only.
   - full peers are no longer forced into headless display solely due relay capability flags.
4. [x] iOS/Android compile validation after patch:
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass)
   - `bash ./iOS/verify-test.sh` (pass)
5. [x] Fast live relay-visibility probe captured for Android + iOS simulator:
   - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=25 GCP_RELAY_CHECK=0 bash ./scripts/live-smoke.sh`
   - Android evidence (`logs/live-smoke/20260303-113927/android-logcat.txt`) shows identity-discovered peer through relay-circuit addresses and `2 full, 0 headless` during this capture.

Still open after this pass:

1. [ ] Re-run synchronized physical iOS-device + Android visibility capture on binaries containing WS12.27 patch to fully close misclassification regression risk in production-like topology.
2. [ ] Confirm sender-state convergence (`stored` -> `delivered`) closure on physical-device message timelines post-WS12.26/WS12.27.

## WS12.28 Transport Regression Hotfix (2026-03-03 HST)

Completed in this pass:

1. [x] Reproduced active Android resend-loop crash from live trip logs:
   - `BleGattClient.connect(BleGattClient.kt:238)` `NullPointerException` observed repeatedly in `logs/5mesh/20260303_115412/android.log`.
2. [x] Implemented Android BLE crash-loop root-cause fix:
   - `BleGattClient.connect` now guards invalid addresses and handles `connectGatt(...) == null` without throwing.
3. [x] Implemented Android+iOS dial candidate hardening for special-use IPv4:
   - both platforms now reject special-use IPv4 dial targets,
   - both platforms now prefer usable private LAN IPv4 during local listener/IP selection.
4. [x] Revalidated compile/build gates after the patch:
   - `cd android && ./gradlew app:compileDebugKotlin -q` (pass),
   - `xcodebuild ... -destination 'platform=iOS Simulator,name=iPhone 16e' build ...` (pass).

Still open after this pass:

1. [ ] Install WS12.28 binaries on physical Android + iOS and verify passive logs no longer show:
   - `BleGattClient.connect` NPE loop,
   - dials to special-use IPv4 (for example `192.0.0.x`, `198.18.x.x`, `203.0.113.x`).
2. [ ] Confirm previously stuck pending messages can progress/deliver under active connectivity with WS12.28 binaries.
3. [ ] Re-run synchronized convergence checks (`verify_receipt_convergence`, relay-flap correlation, BLE-only validation) against fresh post-WS12.28 artifacts.

## WS12.29 Known-Issues Consolidation + Full-Functionality Burndown (2026-03-03 HST)

Completed in this pass:

1. [x] Pulled fresh Android+iOS device-side diagnostics and crash artifacts:
   - `logs/device-debug-20260303-140445/`
   - `logs/device-debug-20260303-140445/ios-crashpull/`
2. [x] Consolidated issue ledger + remediation plan into canonical doc:
   - `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
3. [x] Correlated critical crash class from fresh iOS reports:
   - send-path crash (`SIGTRAP`) in `BLEPeripheralManager.sendDataToCentral`,
   - recurring iOS `cpu_resource_fatal` under retry pressure.
4. [x] Correlated Android stale-route/stale-BLE-target retry churn from on-device diagnostics.

Still open after this pass:

1. [ ] Prove iOS send-path crash non-repro on latest installed iPhone binary with synchronized artifacts.
2. [ ] Prove iOS watchdog (`cpu_resource_fatal`) non-repro under retry-heavy send scenarios.
3. [ ] Close Android stale-route and stale-BLE-target retry loops with post-fix evidence tied to active conversation peer IDs.
4. [ ] Close cross-device continuity gate (`Android <-> iOS`) with synchronized bidirectional delivered-state evidence.
5. [x] Harden/document reliable iOS large-diagnostics extraction workflow for repeatable RCA.
   - Outcome (2026-03-06 UTC): `scripts/run5-live-feedback.sh` iOS diagnostics pull now retries `xcrun devicectl device copy`, requires near-stable file-size confirmation across attempts, and fail-fast rejects captures that cannot prove non-truncated stability.
6. [x] Add iOS confirmation prompt before contact deletion and capture validation evidence. <!-- user-requested todo -->
   - Implemented in `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactsListView.swift` via explicit destructive-action alert.
   - Verification: `bash ./iOS/verify-test.sh` (pass).

## WS12.30 Live Verification Feedback Loop (2026-03-03 HST)

Completed in this pass:

1. [x] Added dedicated iterative harness copy for step-gated 5-node verification without modifying `run5.sh`:
   - `scripts/run5-live-feedback.sh`
2. [x] Added strict sequential gate flow in harness:
   - build/deploy phase (optional skip),
   - `run5 --update` capture phase,
   - log-health gate,
   - directed pair-matrix gate for all node pairings,
   - crash/fatal marker gate,
   - deterministic verifier gate set (`verify_relay_flap_regression`, `verify_ble_only_pairing`, `verify_receipt_convergence`, `verify_delivery_state_monotonicity`).
3. [x] Added per-attempt evidence packaging under:
   - `logs/live-verify/<step>_<timestamp>/attempt_*`
4. [x] Updated scripts operations guide and known-issues execution plan with exact usage/runbook.

Still open after this pass:

1. [ ] Execute WS12.29 issue burndown using the new loop for each active issue ID and archive pass/fail manifests per attempt.
2. [ ] Close all P0/P1 issue IDs only after corresponding loop runs pass required gates with synchronized Android+iOS real-device evidence.

## WS12.31 Stale-Target Convergence Hardening + Transport Priority Clarification (2026-03-04 HST)

Completed in this pass:

1. [x] Hardened Android+iOS route candidate prioritization:
   - prefer discovery/ledger-backed route candidates before persisted notes/cached route IDs.
2. [x] Hardened Android+iOS route-candidate recipient validation:
   - candidate must either resolve to recipient key directly or be corroborated by runtime discovery/ledger key evidence.
3. [x] Stopped failed-route persistence in Android+iOS pending-outbox retry state:
   - when no route ACK succeeds, `routePeerId` is no longer re-written to a failed candidate.
4. [x] Hardened local BLE fallback target selection on Android+iOS:
   - connected BLE peers are now preferred over stale cached `ble_peer_id` hints.
5. [x] Hardened Android disconnect cleanup:
   - callback path now prunes disconnected aliases by peer ID/canonical ID/public-key match.
6. [x] Added explicit iOS contact-delete confirmation safety gate:
   - `ContactsListView` now prompts before destructive remove.
7. [x] Revalidated local build/test gates after WS12.31:
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass)
   - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.data.MeshRepositoryTest"` (pass)
   - `bash ./iOS/verify-test.sh` (pass)

Still open after this pass:

1. [ ] Validate WS12.31 stale-route/BLE-target behavior with synchronized physical Android+iOS artifacts tied to active conversation peer IDs.
2. [ ] Close `R-WS12-29-02` only after post-WS12.31 logs show deterministic route refresh/convergence (no persistent stale-target loops).

## WS12.34 Transport Failure Triage + 10-Fix Reliability Sweep (2026-03-04 HST)

Completed in this pass:

1. [x] Diagnosed iOS+Android transport failures from live device logs after WiFi/BLE/cell toggling:
   - Rust `receive_message` errors invisible on mobile (swallowed `tracing` output).
   - iOS relay flapping threshold self-triggering, permanently blocking relay circuit path.
   - Messages being expired/dropped despite "never fail delivery" philosophy.
   - Stale routing data causing infinite retry loops.
2. [x] Implemented 10 fixes across Rust core, iOS, Android:
   - `eprintln!` error visibility in Rust core `receive_message` path.
   - `relayEnabled` nil-safety on both iOS and Android.
   - Retry throttle 500→2000ms (iOS).
   - Relay diagnostic throttle — 90% reduction when flapping (iOS).
   - Messages NEVER expire — removed attempt limits and age-based expiry.
   - Progressive backoff: `min(2^attempt, 60)` seconds, capping at 5 min.
   - WiFi recovery → immediate outbox flush (iOS + Android).
   - BLE 15s connection timeout for stale GATT connections (Android).
   - Dial candidate cap at 6 max per peer (iOS + Android).
3. [x] Enforced core philosophy: messages NEVER expire, retry indefinitely with progressive backoff.
4. [x] Revalidated Rust core compilation:
   - `cargo check --workspace` (pass).
5. [x] Fixed Android build failure: `appendDiagnostic` → `Timber.i()` in `notifyNetworkRecovered()`:
   - `cd android && ./gradlew :app:compileDebugKotlin` (pass).

Still open after this pass:

1. [ ] Deploy Rust core + both mobile apps and observe `eprintln!` output to diagnose any remaining `receive_message` failures.
2. [ ] Confirm end-to-end message delivery across all transport layers post-fix.
3. [ ] Validate WiFi recovery → outbox flush behavior on physical devices.

## WS12.35 Non-Device Reliability Reconciliation (2026-03-06 UTC)

Completed in this pass:

1. [x] Correlated baseline + CI blockers against latest failed non-`action_required` run (`22706811148`, workflow `CI`).
2. [x] Closed workspace compile drift preventing deterministic verification:
   - `wasm/src/lib.rs` test `MessageRecord` initializers now include `sender_timestamp`.
   - `cargo test --workspace --no-run` now passes in this environment.
3. [x] Closed iOS MainActor isolation drift for Multipeer diagnostics/identity helpers:
   - `MultipeerTransport` now routes `getIdentitySnippet` + `appendDiagnostic` through MainActor-safe helper methods.
   - `ChatViewModel` and `SettingsViewModel` are explicitly `@MainActor` for UI-bound repository calls.
4. [x] Aligned Android mesh-participation tests to runtime default semantics:
   - `MeshRepositoryTest` null-settings expectations now match `isMeshParticipationEnabled(settings ?: true)` behavior.
5. [x] Restored receipt validation safety while preserving delivery convergence:
   - Core now rejects receipt envelopes when sender identity cannot be correlated to the outbound recipient (`test_mismatched_sender_receipt_is_ignored` + `test_delivery_receipt_marks_history_and_outbox_delivered` both pass).
6. [x] Added/adjusted non-device swipe-delete verification tests where infrastructure is available:
   - Android `ConversationsViewModelTest` now verifies `clearConversation(peerId)` delegates to repository and refreshes list state.
   - iOS `verify-role-mode.sh` now enforces conversation swipe-action + confirmation + `clearConversation` source guardrails.
7. [x] Hardened iOS diagnostics extraction workflow in existing WS12.30 harness:
   - `scripts/run5-live-feedback.sh` now retries iOS diagnostics pulls and requires stable-size confirmation across attempts to guard against truncated copies.

Still open after this pass:

1. [ ] Android/iOS physical synchronized evidence gates remain open (unchanged): device-runtime artifact requirements for `R-WS12-04/05/06`, `R-WS12-29-01`, and `R-WS12-29-02`.
2. [ ] Host prerequisites remain environment-gated (unchanged): Docker runtime provisioning (`WS12.15.3`) and wireless ADB persistence (`WS12.8.5`).

## WS12.36 PR CI Failure Closure (2026-03-07 UTC)

Completed in this pass:

1. [x] Correlated the currently failing PR CI run (`22790198922`, workflow `CI`) to concrete Android, iOS, and Rust Core blockers.
2. [x] Fixed Android CI step ordering so `cargo-ndk` is installed before `android/verify-build-setup.sh`.
3. [x] Closed remaining iOS MainActor isolation drift in transport-layer repository helper calls:
   - `BLECentralManager` now routes diagnostics through a MainActor-safe helper.
   - `MultipeerTransport.identitySnippetForDisplayName()` now uses MainActor-safe synchronous bridging.
4. [x] Hardened the flaky macOS sled persistence test:
   - `identity::store::tests::test_store_persistence_across_instances` now tolerates brief post-drop lock-release delay before reopening the DB.
5. [x] Revalidated the Rust-side blocker locally:
   - `cargo fmt --all -- --check` — pass
   - `cargo test -p scmessenger-core identity::store::tests::test_store_persistence_across_instances` — pass

### WS12.25 Mega-Update Consolidated Next Steps (Open + Deferred)

This is the current "burn-down" slate combining all active deferred/runtime closures still gating full reliability signoff:

1. Runtime evidence closure gates (`R-WS12-04`, `R-WS12-05`, `R-WS12-06`):
   - synchronized relay-flap correlation window,
   - synchronized receipt convergence for both Android->iOS and iOS->Android,
   - synchronized strict BLE-only convergence bundle.
2. Pending-outbox + sender-state closure gates:
   - prove old pending entries drain once peer route is active (post-WS12.25 fix),
   - prove sender state converges to `delivered` when recipient ingest is confirmed.
3. Environment validation debt:
   - provision Docker and run `bash ./verify_simulation.sh` (`WS12.15.3`),
   - execute live network matrix validation and ACK-safe path-switch validation (`WS12.15.4`, `WS12.15.5`),
   - execute app-update/reinstall continuity evidence capture on real Android+iOS (`WS12.15.6`),
   - capture iOS power-settings runtime evidence on real device (`WS12.15.7`).
4. UX verification debt:
   - complete swipe-delete evidence/test pass on both Android and iOS (`WS12.24.5`).

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

14. [x] Message timestamp parity (iOS align to Android)

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
- Outcome: Contact/thread deletion APIs are wired in Android+iOS repository layers (`removeContact`/`deleteContacts` + `clearConversation`) and backed by `HistoryManager` core functions. Conversation-list swipe-delete parity is tracked and implemented in WS12.24.

19. [x] Headless/Relay logic Refinement
    - [x] Update `IronCoreBehaviour::new` to accept `headless` boolean flag and incorporate it into the `agent_version` string.
    - [x] Update `start_swarm` and `start_swarm_with_config` in `core/src/transport/swarm.rs` to accept and pass down the `headless` flag.
    - [x] Adjust calls to `start_swarm` in `cli/src/main.rs`: `cmd_start` passes `false`, and `cmd_relay` passes `true`.
    - [x] Update `MeshService::start_swarm` in `core/src/mobile_bridge.rs` to pass `false`.
    - [x] Update `CoreDelegate` trait and `api.udl` to include `agent_version` in `on_peer_identified`.
    - [x] Update Android `MeshRepository.kt` to handle `agentVersion` and identify headless peers.
    - [x] Update iOS `CoreDelegateImpl.swift` and `MeshRepository.swift` to handle `agentVersion` and identify headless peers.
    - [x] Confirm that direct P2P messaging works over cellular with fallback to relaying (mandatory for 0.1.2 Alpha).

### WS12.7 Runtime Sanity Follow-ups (2026-03-02 HST)

1. [x] Android: fix BLE identity beacon payload fallback so listener/external routing hints are not unconditionally stripped.
2. [x] Android: serialize pending outbox flush execution to prevent overlapping retry passes for the same queue item.
3. [x] Android: apply local uptime fallback when Core stats report `uptimeSecs=0` while service is running.
4. [x] Re-validate live delivery behavior after GCP relay rollout fully replaces `scmessenger/0.1.0/headless/relay/*` nodes still observed in active logs.
   - Outcome (2026-03-02 HST): live CLI runtime probe confirmed relay identity rotation on `34.135.34.73:9001` (`12D3KooWET...` -> `12D3KooWJa...`); post-rotation delivery path still requires follow-up due reservation/custody regression signals.
5. [x] Investigate operational handling for long-lived historical pending outbox entries (high-attempt legacy items) without violating no-give-up retry policy.
   - Outcome (2026-03-03 HST): Added explicit operator runbook guidance for legacy pending-outbox triage in `docs/RELAY_OPERATOR_GUIDE.md` without introducing retry exhaustion semantics.
6. [x] Triage iOS simulator startup runtime-issue warnings (`NSFileManager createDirectory*` main-thread I/O) and confirm whether they reflect app codepaths vs simulator-only diagnostics noise.
   - Outcome (2026-03-03 HST): Confirmed app-side startup path was invoking `FileManager.createDirectory` on `@MainActor` in `MeshRepository.init()`. Hotfix moved diagnostics file persistence to background I/O queue and removed main-thread storage directory creation.

### WS12.8 Runtime Recheck Follow-ups (2026-03-02 HST)

1. [x] iOS: fix dashboard node-count inflation where discovered metrics were correct but node totals were overstated by stale/alias peer entries.
   - Outcome (2026-03-03): `MeshDashboardView` now computes full/headless totals from online-only deduplicated peers and performs stronger alias collapse across canonical/libp2p/BLE/public-key identifiers.
2. [x] Restore Android live-log visibility by re-establishing wireless ADB endpoint (`adb devices`/`adb mdns services` were empty during this pass).
   - Outcome (2026-03-03 HST): Wireless endpoint was restored and Android runtime logs were captured (including active `scmessenger/0.2.0/headless/relay/*` agent observations). Endpoint later dropped again after daemon restart; persistence follow-up remains open below.
3. [x] Investigate relay-circuit reservation failure post-redeploy using new debug error detail emitted from `core/src/transport/swarm.rs`.
   - Outcome (2026-03-03 HST): Fresh CLI runtime probe did not reproduce `Could not register relay circuit reservation`; relay reservation failure signal is not currently reproducible in this environment.
4. [x] Resolve failing live custody integration gate: `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` (timeout waiting for reconnect delivery).
   - Outcome (2026-03-03 HST): `ANDROID_HOME=/path/to/android/sdk ./scripts/verify_ws12_matrix.sh` now passes, and `integration_relay_custody` passed 3/3 consecutive reruns (stable-pass classification).
5. [ ] Stabilize Android wireless ADB endpoint persistence across reconnect cycles (`adb devices` may drop back to empty after daemon restart despite prior successful discovery).

### WS12.10 Repo-Wide Action Roundup (2026-03-03 HST)

Inventory from repo-wide checklist scan (`rg -P "^\s*(?:[-*]|\d+\.)\s+\[ \]" --glob "*.md"`):

1. Open markdown checklist items repo-wide: **84**
2. Active canonical open checklist items: **31** (WS12.8/WS12.11/WS12.12/WS12.13/WS12.14/WS12.15)
3. Deferred residual risks requiring explicit carry-forward:
   - `R-WS10-02` (peer-identity rotation vs per-peer token buckets)
4. Non-historical open checklist sources (execution truth):
   - `REMAINING_WORK_TRACKING.md` (31)
5. Historical open checklist sources (context only, not canonical execution truth):
   - `docs/historical/iOS/FINAL_STATUS.md` (21)
   - `docs/historical/iOS/PHASE4_IMPLEMENTATION.md` (14)
   - `docs/historical/REMEDIATION_PLAN.md` (14)
   - `docs/historical/iOS/PHASES_4-15_GUIDE.md` (2)
   - `docs/historical/APP_VERSION_0.1.2_ALPHA_PLAN.md` (2)
6. Planned v0.2.1 queues (explicitly outside v0.2.0 closeout):
   - WS13 decomposition in `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md`
   - WS14 decomposition in `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`

### WS12.11 iOS Relay Flapping Follow-ups (2026-03-03 HST, implementation + follow-up)

1. [x] Add iOS-side relay connection state timeline instrumentation keyed by canonical relay peer ID (connect, disconnect, identify, reservation attempt/result) to prove whether duplicate `peer_identified` events are from distinct sessions or repeated callbacks on one session.
   - Outcome (2026-03-03 HST): Added relay timeline diagnostics in `MeshRepository` for identify/disconnect/dial-allowed/dial-debounced/dial-attempt/dial-started/dial-failed keyed to extracted relay peer IDs.
2. [x] Add guardrails to prevent overlapping relay bootstrap priming and route-triggered connect attempts for the same relay within a short debounce window.
   - Outcome (2026-03-03 HST): Added bootstrap in-progress gate plus relay-peer dial debounce in `primeRelayBootstrapConnections()` and `connectToPeer(...)`.
3. [x] Add explicit "relay availability state machine" metrics in iOS diagnostics export (`stable`, `flapping`, `backoff`, `recovering`) for operator-visible triage.
   - Outcome (2026-03-03 HST): `exportDiagnostics()` now emits relay availability state fields (`relay_availability_state`, `relay_recent_events_60s`, `relay_backoff_until_ms`, and related timestamps).
4. [x] Correlate iOS relay flapping windows against GCP relay/server logs in the same UTC intervals to separate client race behavior from remote relay churn.
   - Outcome (2026-03-03 HST): Added `scripts/correlate_relay_flap_windows.sh` and executed artifact correlation (`ios_diagnostics_latest.log` vs `logs/5mesh/gcp.log`), classifying the sampled pair as `unsynchronized_artifacts_no_time_overlap`.
5. [x] Add regression coverage (integration or deterministic harness) that fails when repeated identify/dial loops occur without sustained connected hold time.
   - Outcome (2026-03-03 HST): Added `scripts/verify_relay_flap_regression.sh` deterministic harness; run on current iOS diagnostics artifact completed with pass summary and explicit relay dial-loop counters.
6. [ ] Re-run dual-device live probe (Android + iOS + CLI/GCP) with synchronized timestamps and capture one full flap cycle artifact bundle for post-fix comparison.

### WS12.12 Android<->iOS Pairing Non-Delivery Follow-ups (2026-03-03 HST, implementation + follow-up)

1. [x] Add Android BLE send-path consistency guardrails so a single payload cannot concurrently report both write-init failure and write success without a deterministic final outcome state.
   - Outcome (2026-03-03 HST): `BleGattClient` now guards GATT queue permit ownership with atomic tracking, hardens callback/release races, and treats `WRITE_TYPE_NO_RESPONSE` callbacks as informational-only to prevent contradictory final outcomes.
2. [x] Add explicit per-message transport-attempt timeline diagnostics (`core`, `relay-circuit`, `BLE`) with final attempt verdict to isolate where receipt convergence breaks.
   - Outcome (2026-03-03 HST): Android+iOS `MeshRepository` now emit structured `delivery_attempt` markers for local fallback/core/relay retry paths with message ID + context (`initial_send`, `outbox_retry`).
3. [x] Add a focused integration test for Android fallback behavior: when internet route fails and BLE fallback fires, require deterministic recipient receipt or deterministic terminal-failure signal.
   - Outcome (2026-03-03 HST): Added Android unit test `ble-only fallback path emits deterministic terminal failure when BLE send fails` in `MeshRepositoryTest`, plus iOS local transport test `testBleOnlyTerminalFailureSignal`.
4. [x] Add temporary operator/tester playbook step to clear or quarantine extreme legacy pending-outbox entries before pairing validation runs so fresh-message behavior is observable.
   - Outcome (2026-03-03 HST): Added "Legacy Pending Outbox Triage (No-Give-Up Safe)" operator workflow to `docs/RELAY_OPERATOR_GUIDE.md` with concrete Android/iOS inspection commands.
5. [ ] Capture synchronized tri-platform traces (Android logcat + iOS diagnostics + relay log window) for one failed message ID from send initiation through retry cycle.
6. [ ] Verify iOS-side receipt/ack emission path during Android BLE fallback attempts to confirm whether recipient ingest succeeds but ack path fails, or ingest fails entirely.

### WS12.13 Wave-2 Backlog Consolidation (2026-03-03 HST)

1. Non-historical mixed-doc checklists were normalized to status-tagged guidance/roadmap entries (no open checkbox ambiguity):
   - `FEATURE_WORKFLOW.md`
   - `AUDIT_QUICK_REFERENCE.md`
   - `FEATURE_PARITY.md`
   - `DRIFTNET_MESH_BLUEPRINT.md`
   - `docs/TRANSPORT_ARCHITECTURE.md`
2. `docs/TRANSPORT_ARCHITECTURE.md` future enhancements were migrated to explicit roadmap lines with status, owner, milestone, gate command, and acceptance criteria.
3. Validation/debt reconciliation executed:
   - `cargo check --workspace` — pass
   - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:generateUniFFIBindings` — pass
   - `bash iOS/copy-bindings.sh` — pass
   - `ANDROID_HOME=/path/to/android/sdk bash ./verify_integration.sh` — pass (now delegates to canonical WS12 matrix)
   - `bash ./verify_simulation.sh` — fail-fast as designed when Docker is unavailable (no auto-install side effects)
   - `cd wasm && wasm-pack build` — pass (after installing `wasm-pack` and disabling release `wasm-opt` in `wasm/Cargo.toml` for host compatibility)
4. Script hygiene updates:
   - `verify_integration.sh` converted from stale grep-based checks to canonical `scripts/verify_ws12_matrix.sh` execution.
   - `verify_simulation.sh` now requires preinstalled/running Docker and exits with explicit operator instructions instead of attempting automatic system installs.
5. Repo-wide checklist inventory after wave-2 normalization:
   - Open markdown checkboxes repo-wide: **71**
   - Active canonical open checkboxes: **18**
   - Historical open checkboxes: **53** (all under `docs/historical/*`)
6. Residual-risk carry-forward:
   - `R-WS10-02` remains `Deferred`.
7. Post-update issue slate (based on live watch artifacts from 2026-03-02/03):
   - [Tracked Live Gate] Relay session stability under active pairing run: verify iOS no longer oscillates through rapid connect/timeout/disconnect cycles in Multipeer + relay coexistence windows.
   - [Tracked Live Gate] Android internet route resilience in pairing runs: verify `Core-routed delivery failed` / `Relay-circuit retry failed` rates materially drop and `messagesRelayed` progresses.
   - Likely still remaining TODO unless explicitly fixed in this update:
     - [x] Android BLE GATT operation-state race: eliminate `IllegalStateException: The number of released permits cannot be greater than 1` in `BleGattClient.releaseGattOp` during callback races.
       - Outcome (2026-03-03 HST): `BleGattClient` now enforces single-release semantics per queued op using atomic permit-held state and overflow-safe release handling.
     - [x] Android BLE stack mismatch noise triage: investigate repetitive `BluetoothRemoteDevices Address type mismatch` flood and determine whether app-level dedupe/throttle or transport-state correction is required.
       - Outcome (2026-03-03 HST): Added address-type mismatch mitigation in `BleGattClient` (`ADDRESS_TYPE_MISMATCH_BACKOFF_MS`) with connect-throttle + stats counter (`addressTypeMismatchConnectSkips`) to prevent repeated immediate reconnect churn for the same peer address.
     - [x] iOS Multipeer channel storm guardrails: bound concurrent channel attempts and enforce deterministic cleanup to prevent repeated `Timed out, enforcing clean up` cascades under reconnect pressure.
       - Outcome (2026-03-03 HST): Added invite debounce, in-flight gating, concurrent-invite cap, and timeout/decline diagnostics counters in `MultipeerTransport`.
     - [x] End-to-end receipt convergence assertion: add one deterministic cross-platform test/runbook step proving recipient ingest + receipt emit for Android->iOS and iOS->Android when internet route degrades and BLE fallback activates.
       - Outcome (2026-03-03 HST): Added deterministic operator runbook in `docs/RELAY_OPERATOR_GUIDE.md` ("Cross-Platform Receipt Convergence Assertion").

### WS12.14 Android Bluetooth-Only Pairing Follow-ups (2026-03-03 HST, implementation + follow-up)

1. [x] Add a strict "BLE-only validation mode" for mobile test runs that hard-disables internet/relay route usage and emits a fail-fast diagnostic marker when non-BLE paths (for example WiFi-backed multipeer sessions) are used.
   - Outcome (2026-03-03 HST): Implemented `SC_BLE_ONLY_VALIDATION` gating in Android+iOS `MeshRepository`; non-BLE route usage is explicitly blocked and logged via deterministic `delivery_attempt ... reason=strict_ble_only_mode` markers.
2. [x] Harden Android BLE peer address-type handling for iOS peers; investigate and resolve repeated `Address type mismatch` churn so one canonical address-type mapping is retained per session.
   - Outcome (2026-03-03 HST): Added mismatch detection counter + per-address cooldown backoff to suppress reconnect hammering after address-type mismatch events in `BleGattClient`.
3. [x] Add Android BLE discovery-health counters (advertisements seen, GATT connects attempted/succeeded, address-type transitions) to diagnostics export and Mesh stats.
   - Outcome (2026-03-03 HST): Added `BleScanner` discovery stats and `BleGattClient` connect/mismatch counters, merged into Android diagnostics export (`ble_discovery`, `ble_client`, `strict_ble_only_validation`).
4. [x] Add iOS diagnostics marker for effective Multipeer transport medium per session (BLE/AWDL/WiFi) and include invitation timeout/decline reason counts.
   - Outcome (2026-03-03 HST): Added `MultipeerTransport.diagnosticsSnapshot()` and export fields in iOS diagnostics (`multipeer_effective_medium_estimate`, `multipeer_invite_timeout_count`, `multipeer_invite_decline_count`, `strict_ble_only_validation`).
5. [x] Add deterministic integration harness for Android<->iOS Bluetooth-only pairing/send/ack flow that fails on repeated invite timeout loops or zero-advertisement windows.
   - Outcome (2026-03-03 HST): Added `scripts/verify_ble_only_pairing.sh` and `scripts/verify_receipt_convergence.sh` harnesses for strict BLE-only marker validation and message ID receipt-convergence checks.
6. [ ] Capture synchronized BLE-only artifact bundle (Android logcat + iOS logs + one message ID timeline) after fixes and compare against WS12.14 baseline before closing risk.

### WS12.15 Wave-2 Continuation Plan Intake (2026-03-03 HST)

1. [x] Fix CLI reconnect-ledger panic under long failure streaks (`attempt to multiply with overflow` in `cli/src/ledger.rs`).
   - Outcome (2026-03-03 HST): backoff math now uses saturating arithmetic and clamped exponent; added regression test `test_ledger_entry_backoff_overflow_safety`; `cargo test -p scmessenger-cli ledger` and `cargo check -p scmessenger-cli` both pass.
2. [x] Install `wasm-pack` on the active dev host and rerun `cd wasm && wasm-pack build` to clear remaining local validation-debt blocker.
   - Outcome (2026-03-03 HST): Installed `wasm-pack 0.14.0`, added `wasm-opt = false` release-profile metadata in `wasm/Cargo.toml` for this host target, and re-ran `cd wasm && wasm-pack build` successfully.
3. [ ] Provision Docker runtime on the active dev host and rerun `bash ./verify_simulation.sh` to convert fail-fast prerequisite guidance into executed simulation evidence.
4. [ ] Execute live network matrix validation (GCP + direct P2P + relay fallback, Android+iOS) and store artifact bundle pointer in canonical docs.
5. [ ] Execute ACK-safe path switching validation (mid-send route switch, no duplicate/loss, sender receipt convergence) and record evidence.
6. [ ] Execute app-update + reinstall continuity validation on real Android+iOS devices and record identity/contact/history continuity evidence.
7. [ ] Capture iOS power settings runtime evidence on a real iPhone for beta-gate carry-forward and link artifacts.
8. [x] Resolve historical carry-forward decision from `docs/ALPHA_RELEASE_AUDIT_V0.1.2.md`: explicitly mark v0.1.2 version-bump/redeploy tasks as either superseded by v0.2.0 release-sync docs or carried into a dedicated historical closeout note.
   - Outcome (2026-03-03 HST): Updated `docs/ALPHA_RELEASE_AUDIT_V0.1.2.md` with explicit historical closeout status and canonical v0.2.0 release-sync pointers.

### WS12.16 Wave-2 Runtime Hardening Closure (2026-03-03 HST)

1. Implemented in this pass:
   - Android BLE GATT permit/callback race hardening (`BleGattClient`).
   - Android+iOS per-message `delivery_attempt` timeline diagnostics.
   - iOS relay timeline instrumentation + debounce/availability-state export.
   - iOS Multipeer invitation storm guardrails + timeout/decline diagnostics counters.
2. Verification commands:
   - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` — pass.
   - `bash ./iOS/verify-test.sh` — pass.
   - `cargo check --workspace` — pass.
3. Updated checklist inventory after WS12.16:
   - Open markdown checkboxes repo-wide: **76**
   - Active canonical open checkboxes: **23**
   - Historical open checkboxes: **53**
   - Note: this snapshot is superseded by WS12.17 final-wave inventory below.
4. Highest-priority remaining wave-2 actions (post-implementation evidence gates):
   - Re-run synchronized Android+iOS+GCP live probe and verify reduced relay/multipeer churn with receipt convergence in both directions.
   - Capture synchronized BLE-only and internet-degraded artifact bundle with one message ID end-to-end timeline for residual-risk closure.
   - Provision Docker runtime and rerun `bash ./verify_simulation.sh` to clear the final local validation-debt blocker.

### WS12.17 Wave-3 Governance Closure (2026-03-03 HST)

1. Historical checklist triage completed:
   - `docs/historical/APP_VERSION_0.1.2_ALPHA_PLAN.md`
   - `docs/historical/REMEDIATION_PLAN.md`
   - `docs/historical/iOS/PHASE4_IMPLEMENTATION.md`
   - `docs/historical/iOS/PHASES_4-15_GUIDE.md`
   - `docs/historical/iOS/FINAL_STATUS.md`
   - All open checkboxes in these historical files were converted to explicit historical status tags (`Historical - Superseded`, `Historical - Re-scoped`, `Historical - Carry-forward`).
2. Added deterministic runtime-harness set for active follow-ups:
   - `scripts/correlate_relay_flap_windows.sh`
   - `scripts/verify_relay_flap_regression.sh`
   - `scripts/verify_receipt_convergence.sh`
   - `scripts/verify_ble_only_pairing.sh`
3. Normalized future execution queue (v0.2.1+ planning scope, non-blocking for v0.2.0 closeout):
   - `WS13.1` Identity metadata persistence — Owner: Core + Mobile Bridge — Gate: identity persistence + migration test suite.
   - `WS13.2` Contact/request schema updates — Owner: Core Data + Mobile/WASM adapters — Gate: schema migration + parity adapter tests.
   - `WS13.3` Registration protocol/signature verification — Owner: Core Transport — Gate: protocol signature validation integration tests.
   - `WS13.4` Relay registry/custody enforcement — Owner: Core Transport + Relay Ops — Gate: custody routing + registry state-machine tests.
   - `WS13.5` Handover/abandon queue migration + UX — Owner: Core + Android+iOS clients — Gate: queue migration and user-facing rejection-path tests.
   - `WS13.6` Compatibility/migration matrix — Owner: Cross-platform QA — Gate: upgrade/migration matrix and manual runbook evidence.
   - `WS14.1` Notification policy model — Owner: Core + Bindings — Gate: classifier/unit tests + UDL/WASM API parity checks.
   - `WS14.2` iOS notification completion — Owner: iOS — Gate: DM/DM-request routing integration tests.
   - `WS14.3` Android notification completion — Owner: Android — Gate: channel/action parity tests + foreground suppression checks.
   - `WS14.4` WASM notification wiring — Owner: Web/WASM — Gate: browser worker notification flow tests.
   - `WS14.5` Hybrid endpoint interface prep — Owner: Core + Adapter surfaces — Gate: endpoint registration persistence/validation tests.
   - `WS14.6` Verification + docs gate — Owner: Cross-platform QA + Docs — Gate: parity matrix pass + residual-risk sync.
4. Final inventory after wave-3 triage (`rg -P "^\s*(?:[-*]|\d+\.)\s+\[ \]" --glob "*.md"`):
   - Open markdown checklist items repo-wide: **10**
   - Active canonical open checklist items: **10** (`REMAINING_WORK_TRACKING.md` only)
   - Historical open checklist items: **0**
5. Remaining action items at WS12.17 closeout (repo-wide exhaustive list at that time):
   - WS12.8.5: stabilize Android wireless ADB endpoint persistence across daemon reconnect cycles.
   - WS12.11.6: run synchronized dual-device live probe and capture full flap-cycle bundle.
   - WS12.12.5: capture synchronized tri-platform traces for one failed message ID.
   - WS12.12.6: verify iOS receipt/ack emission path during Android BLE fallback attempts.
   - WS12.14.6: capture synchronized BLE-only artifact bundle and compare against baseline.
   - WS12.15.3: provision Docker runtime and rerun `verify_simulation.sh`.
   - WS12.15.4: execute live network matrix validation (GCP + direct + relay fallback).
   - WS12.15.5: execute ACK-safe path switching validation and record evidence.
   - WS12.15.6: execute app-update + reinstall continuity validation on real Android+iOS devices.
   - WS12.15.7: capture iOS power settings runtime evidence on real iPhone.

## Priority 1: Tooling, CI, and Experimental Surface

1. [x] Align CI with tri-platform target status
   - Outcome (2026-03-03): `.github/workflows/ci.yml` now includes explicit WS12 parity/test gates for:
     - deterministic core offline/partition suites,
     - Android role/fallback unit parity checks,
     - desktop/WASM role parity checks,
     - iOS verification with transport fallback + role-mode parity checks.

2. [x] Add browser-executed WASM test job (parity gate)
   - Current: native/non-browser WASM tests only in workspace run.
   - Target: `wasm-pack` runtime test coverage in CI.
   - Outcome: `.github/workflows/ci.yml` `check-wasm` installs `wasm-pack` and runs browser runtime tests (`wasm-pack test --headless --firefox`) in CI.

3. [x] Resolve integration test warnings in core tests
   - Current: workspace tests pass with warning noise.
   - Target: warning-clean path for strict CI.
   - Outcome: Cleaned up unused assignments and unused variables across all integration suites. Unit and integration tests are 100% warning-clean.

4. [x] Standardize Android CI environment setup for `ANDROID_HOME`
   - Current: local build requires explicit shell env setup.
   - Target: consistent CI env bootstrap and preflight enforcement.
   - Outcome: `.github/workflows/ci.yml` `check-android` now sets up Android SDK, standardizes `ANDROID_HOME`/`ANDROID_SDK_ROOT`, and runs `android/verify-build-setup.sh` preflight before Gradle build/tests.

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

16. [x] iOS historical artifact segmentation

- Current: `iOS/iosdesign.md` and `iOS/SCMessenger/build_*.txt` mix design/historical/runtime evidence in active tree.
- Target: section-level historical tagging and relocation/retention policy to keep active docs concise.
- Outcome: historical iOS design artifacts are retained under `docs/historical/` references, and active `iOS/` tree no longer contains `iOS/iosdesign.md` / `iOS/SCMessenger/build_*.txt` historical noise files.

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

- `cargo test --workspace` passes (367 passed, 0 failed, 17 ignored — verified 2026-03-03)
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

## Roadmap to 1.0.0 (Post v0.2.0-alpha)

## Immediate v0.2.0 Closeout Queue (Feasible Remaining Work)

The following items are feasible to execute as additional `v0.2.0` closure work without introducing net-new product scope:

1. [x] `R-WS3-02` / `EC-01`: migrate relay custody default persistence from temp-dir to durable app data paths and add restart recovery verification.
   - Outcome: `RelayCustodyStore::for_local_peer` now resolves to durable app-data paths (`SCM_RELAY_CUSTODY_DIR` override -> OS local data dir -> home fallback -> temp fallback), with restart persistence tests retained.
2. [x] `R-WS5-01` / `EC-02`: ensure platform adapters always provide storage snapshots so dynamic pressure controls cannot no-op.
   - Outcome: storage pressure enforcement now uses synthetic snapshot fallback when platform probes are unavailable, preventing no-op behavior in fallback paths.
3. [x] `R-WS4-02` / `EC-04`: add low-cost convergence-marker trust hardening and abuse validation checks.
   - Outcome: convergence markers now require structural/timestamp validation and local message-tracking correlation before retry/custody convergence is applied.
4. [x] Release sync execution (`WS13.x` scoped to release ops): finalize versions/tags/release notes using `docs/releases/*` artifacts.
   - Outcome: release artifacts are canonicalized in `docs/releases/*`, workspace/app version metadata is bumped to `0.2.0`, and CI/docs now reference repo-local release sources.

Not feasible for `v0.2.0` without expanding release scope:

1. `WS13` Tight Pairing (single active device lifecycle).
2. `WS14` direct-message/direct-request notifications.

3. **Automatic Environment Detection and Unified Hydration**
   - Requirement: The app must automatically detect if a previous identity, message history, contacts, or user preferences exist in local storage/backups and utilize them immediately on startup without user intervention.
   - Target: Unified "detect-and-resume" logic that covers all persisted data types across Android, iOS, and Web.
   - Scope: Identity (Keychain/SharedPreferences), Message History (history.db), Contacts (contacts.db), and Privacy Toggles.

4. **Manual Data Management (Reset/Refresh/Delete)**
   - Requirement: Provide a secure, user-facing way to clear or reset all application data.
   - Target: A "Delete All Data" or "Reset Application" button in the Settings view.
   - Action: Securely wipe identity, message history, contacts, and all local preferences from the device.
   - Scope: Android (`SettingsScreen`), iOS (`SettingsView`), and Web.

5. **WS13 (v0.2.1): Single Active Device per Identity (Tight Pairing)**
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

6. **WS13.x (v0.2.1): GitHub release/version synchronization and release-note publishing flow**
   - Requirement: normalize repository/app version metadata and GitHub release artifacts so release tags, release notes, and workspace versions remain consistent.
   - Target:
     - align workspace/package versions for the intended release cut,
     - ensure `v0.1.2` GitHub release notes are finalized/publish-ready,
     - stage `v0.2.0` draft release notes and release checklist inputs for final cut timing.
   - Scope:
     - Cargo/workspace version synchronization (`Cargo.toml` files as applicable at release-cut time),
     - release note doc finalization for GitHub paste/publish flow,
     - release workflow checklist alignment with residual-risk closure evidence,
     - promote external planning artifacts into repo-local docs before execution (to avoid workstation-specific paths).
   - Source inputs now canonicalized in-repo:
     - `docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md`
     - `docs/releases/RELEASE_NOTES_V0.1.2_GH.md`
     - `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`
   - Canonicalization target (during WS13.x execution):
     - keep `docs/releases/` as the only release-notes/checklist source of truth.
   - Progress: repo-local release planning/note artifacts now exist under `docs/releases/`, so WS13.x no longer depends on workstation-specific external files.
   - Execution note: queue this after current WS12 in-flight session and after WS12/WS12.5 closure evidence is captured.

7. **WS14 (v0.2.1): Direct Message + Direct Message Request Notifications (iOS/Android/WASM)**
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

1. `[Closed in WS12.6]` `EC-01`: relay custody default persistence now uses durable app path fallback chain (`R-WS3-02` closed).
2. `[Closed in WS12.6]` `EC-02`: platform storage snapshots now have synthetic fallback so pressure policy cannot no-op (`R-WS5-01` closed).
3. `EC-03` (Accepted in v0.2.0 alpha): replace volatile local transport route hints with stable authenticated alias mapping (`R-WS6-01`, `R-WS7-01`, revisit before beta hardening).
4. `[Closed in WS12.6]` `EC-04`: convergence marker validation/trust hardening baseline shipped (`R-WS4-02` closed).
5. `[Closed in WS12]` `EC-05`: custody reconnect integration test is now CI-gated and reproducible (`R-WS3-01` closed).
6. `EC-06` (Reduced in WS11, accepted in WS12): sender-facing delivery states are normalized in Android+iOS UI/export surfaces; remaining Core-native transition API work is tracked via `R-WS11-01` for post-v0.2.0 follow-up.
7. `EC-07` to `EC-09` (v0.2.1 WS13): execute tight-pair single-active-device lifecycle.
8. `EC-10` to `EC-16` (post-v0.2.1): captive portal adaptation, high-latency profile tuning, censorship-resilience strategy, wake/delegate architecture, sparse encounter optimization, and clock-skew normalization.

## 2026-03-13 iOS Simulator Launch Ambiguity

- Completed: Identified and cleared an iPhone 17 Pro simulator launch blocker caused by a stale `platform IOS` SCMessenger bundle installed into the simulator instead of an `IOSSIMULATOR` build.
- Open: If this recurs, audit any operator or harness path that reuses a previously installed simulator bundle without validating the built Mach-O platform.

## 2026-03-13 Consolidated Open Items From Full Conversation

- Open: prove full 5-node visibility after simulator recovery using the upgraded `run5.sh`; current honest state remains partially indeterminate rather than fully verified.
- Open: investigate iOS simulator runtime `historySync request failed to prepare message` after successful launch recovery.
- Open: complete iOS send-path parity with store-and-forward-first UX so the send action never blocks on live transport success.
- Open: continue hardening iOS against peer-identify / identity-beacon event storms that can contribute to transient freeze/unfreeze behavior.
- Open: unify Android BLE telemetry so accepted-send target reporting matches the actual fresher connected GATT target used on the wire.
- Open: improve physical iOS app-level own-ID/peer capture in harness evidence so transport activity is not hidden by collector gaps.
- Open: validate simultaneous transport functionality across BLE, direct LAN/libp2p, relay, and Wi-Fi Direct/local options.
- Open: identify any script/operator path capable of reinstalling or preserving a stale `iphoneos` bundle inside the simulator.
