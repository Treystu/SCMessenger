# SCMessenger Documentation Hub

Status: Active
Last updated: 2026-03-19 (Log Audit Critical Findings)

This is the primary documentation entrypoint.

For lifecycle classification (`Active`, `Planned`, `Mixed`, `Historical`, `Superseded`), use:

- [Document Status Index](docs/DOCUMENT_STATUS_INDEX.md)

## ⚠️ URGENT - LOG AUDIT CRITICAL FINDINGS

- **[Log Audit Report 2026-03-19](LOG_AUDIT_REPORT_2026-03-19.md) 🔴 CRITICAL**
- **[Master Bug Tracker](MASTER_BUG_TRACKER.md) 🔴 UPDATED**

**Key Issues Found:** 65-75% message delivery failure rate, BLE connection instability, relay circuit issues

## Active Canonical Docs (Read First)

- [Repository Overview](README.md)
- [Repository Context](docs/REPO_CONTEXT.md)
- **[Log Extraction Standard](LOG_EXTRACTION_STANDARD.md) ⚠️ MANDATORY for iOS/Android**
- [Current Verified State](docs/CURRENT_STATE.md)
- [Active Backlog](REMAINING_WORK_TRACKING.md)
- [WS12.29 Known Issues + Burndown Plan](docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md)
- [v0.2.0 Milestone Plan](docs/MILESTONE_PLAN_V0.2.0_ALPHA.md)
- [v0.2.0 Phase Prompts](docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md)
- [v0.2.0 Residual Risk Register](docs/V0.2.0_RESIDUAL_RISK_REGISTER.md)
- [v0.2.1 Tight Pair / WS13 Plan](docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md)
- [v0.2.1 Residual Risk Register](docs/V0.2.1_RESIDUAL_RISK_REGISTER.md)
- [WS14 Automation Handoff](docs/WS14_AUTOMATION_HANDOFF.md)
- [WS14 Hourly Automation Prompt](docs/WS14_HOURLY_AUTOMATION_PROMPT.md)
- [WS13.6 Compatibility & Migration Matrix](docs/WS13.6_COMPATIBILITY_MIGRATION_MATRIX.md)
- [WS13.6 Handover & Abandon Runbook](docs/WS13.6_HANDOVER_ABANDON_RUNBOOK.md)
- [Platform Support Matrix](docs/PLATFORM_SUPPORT_MATRIX.md)
- [Alpha Interop Matrix](docs/INTEROP_MATRIX_V0.2.0_ALPHA.md)
- [Protocol Specification](docs/PROTOCOL.md)
- [Testing Guide](docs/TESTING_GUIDE.md)
- [Edge-Case Readiness Matrix](docs/EDGE_CASE_READINESS_MATRIX.md)

## Planned Docs (Future Scope)

- [v0.2.1 Notifications / WS14 Plan](docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md)

## Active Supporting Audits and Execution Plans

- [GitHub + Repo Realignment Audit (First Pass)](docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md)
- [Global Viability Audit](docs/global_viability_audit.md)
- [Implementation Cheat Sheet (2026-03-04)](docs/implementation_cheatsheet_3.4.2026.md)
- [Deep Architectural Reasoning: DHT Optimization](docs/DEEP_ARCHITECTURAL_REASONING_DHT_OPTIMIZATION.md)

## Release Docs

- [Release Sync Plan (v0.1.2 -> v0.2.0)](docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md)
- [Release Notes Draft (v0.2.0)](docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md)
- [GitHub Release Notes Draft (v0.1.2)](docs/releases/RELEASE_NOTES_V0.1.2_GH.md)

## Contributor and Community Health

- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)
- [Support Routing](SUPPORT.md)

## Repo Automation and Agent Guidance

- [Repository Agent Policy](AGENTS.md)
- [GitHub Copilot Instructions](.github/copilot-instructions.md)
- [Roo Code Integration Plan](scmessenger-roo-code-features-plan.md)

### Roo Code Integration (2026-03-14)

Custom modes, rules, and skills for optimized agent workflows:

**Custom Modes** (`.roomodes/`):
- [`scm-rust`](.roomodes/scm-rust.json) - Rust core development
- [`scm-android`](.roomodes/scm-android.json) - Android/Kotlin development
- [`scm-ios`](.roomodes/scm-ios.json) - iOS/Swift development
- [`scm-protocol`](.roomodes/scm-protocol.json) - Protocol/crypto review
- [`scm-docs`](.roomodes/scm-docs.json) - Documentation specialist
- [`scm-release`](.roomodes/scm-release.json) - Release verification
- [`scm-debug-mesh`](.roomodes/scm-debug-mesh.json) - Mesh/relay debugging

**Project Rules** (`.roo/rules/`):
- [`000-critical.md`](.roo/rules/000-critical.md) - Non-negotiable rules
- [`010-documentation.md`](.roo/rules/010-documentation.md) - Doc sync requirements
- [`020-crypto.md`](.roo/rules/020-crypto.md) - Cryptography constraints
- [`030-platform-parity.md`](.roo/rules/030-platform-parity.md) - Cross-platform rules
- [`040-build-verify.md`](.roo/rules/040-build-verify.md) - Build verification
- [`050-identity.md`](.roo/rules/050-identity.md) - Identity model rules
- [`060-testing.md`](.roo/rules/060-testing.md) - Test requirements

**Skills** (`skills/`):
- [`platform-parity-check`](skills/platform-parity-check/SKILL.md) - Cross-platform verification
- [`release-gate-validator`](skills/release-gate-validator/SKILL.md) - Release readiness
- [`mesh-diagnostics`](skills/mesh-diagnostics/SKILL.md) - Network troubleshooting

**Memory Bank** (`.roo/memory-bank/`):
- [`projectbrief.md`](.roo/memory-bank/projectbrief.md) - Project overview
- [`techContext.md`](.roo/memory-bank/techContext.md) - Technology stack
- [`activeContext.md`](.roo/memory-bank/activeContext.md) - Current focus

## Component and Platform Docs

- [Core Crate](core/README.md)
- [CLI Crate](cli/README.md)
- [Mobile Bindings Crate](mobile/README.md)
- [WASM Bindings Crate](wasm/README.md)
- [Android App](android/README.md)
- [iOS App](iOS/README.md)

## Operations and Setup

- [Install](INSTALL.md)
- [Quick Connect](docs/historical/plans/QUICKCONNECT.md)
- [Docker](docker/README.md)
- [GCP Deploy](GCP_DEPLOY_GUIDE.md)
- [Relay Operator Guide](docs/RELAY_OPERATOR_GUIDE.md)
- [Bootstrap Governance](docs/BOOTSTRAP_GOVERNANCE.md)
- [NAT Traversal Guide](docs/NAT_TRAVERSAL_GUIDE.md)
- [Scripts Operations Guide](scripts/README.md)
- [Log Mincing / Exploratory Analysis](scripts/mince_logs.py)
- **[Log Correlation & Validation Agent](tmp/session_logs/README.md) ⚡ NEW** - Comprehensive log analysis and documentation synchronization validator

## Mixed and Historical Context

Use these for context only; do not treat as execution source of truth unless revalidated:

- [Transport Architecture (mixed-status)](docs/TRANSPORT_ARCHITECTURE.md)
- [Global Rollout Plan (strategy)](docs/GLOBAL_ROLLOUT_PLAN.md)
- [Unified Global App Plan (strategy)](docs/UNIFIED_GLOBAL_APP_PLAN.md)
- [Historical docs index](docs/historical/README.md)

Historical audit artifacts currently live under `docs/historical/`.

## Recent Execution Notes

- WS14 hourly automation reset (2026-03-14 HST, audited the March 13 hourly automation drift, replaced the oversized WS13+WS14 prompt with a WS14-only one-phase-per-run prompt, and added a repo-owned handoff ledger while keeping the automation paused by default) is tracked in:
  - `docs/WS14_AUTOMATION_HANDOFF.md`
  - `docs/WS14_HOURLY_AUTOMATION_PROMPT.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`
  - `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`

- WS13.4/WS13.5 tight-pair execution (2026-03-13, relay registry/custody enforcement landed; sender-facing rejection UX landed; Android physical-device install+launch passed; iOS signed device install passed but automated launch was blocked because the connected iPhone was locked) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md`
  - `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`

- Documentation/build-verification governance lock (2026-03-13, repo policy tightened so same-run canonical doc updates and edited-target build verification are explicit closeout requirements for both Codex and Copilot agents) is tracked in:
  - `AGENTS.md`
  - `.github/copilot-instructions.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

- WS12.39 closeout burndown re-baseline (2026-03-10 UTC, Rust/WASM baseline restoration + issue/workflow/branch reconciliation for steady v0.2.0 alpha truth) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
- WS12.47: BLE Log Visibility Improvements (2026-03-16 HST, implemented proactive identity seeding in `run5.sh`, expanded iOS log capture predicates, and improved BLE transport recognition in the visualizer) is tracked in:
  - `run5.sh`
  - `log-visualizer/public/mesh.html`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
- WS12.41: Dynamic Log Retention & Storage Management (2026-03-12, implemented summarized log storage using time offsets from install time, dynamic disk-aware retention with 80/20 message-priority buffer, and cross-platform StorageManager/LogManager integration) is tracked in:
  - `core/src/store/logs.rs`
  - `core/src/store/storage.rs`
  - `android/app/src/main/java/com/scmessenger/android/utils/FileLoggingTree.kt`
  - `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
- iOS ID Normalization & Case-Sensitivity (2026-03-11, centralized `PeerIdValidator` + exhaustive normalization in repository and viewmodels to fix iOS cross-platform ID mismatches) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
  - `iOS/SCMessenger/SCMessenger/ViewModels/ContactsViewModel.swift`
- WS11 (2026-03-03) delivery-state UX and diagnostics/tester-readiness updates are tracked in:
  - `docs/CURRENT_STATE.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12 (2026-03-03) deterministic offline/partition test expansion, cross-platform parity gates, and docs parity lock are tracked in:
  - `docs/TESTING_GUIDE.md`
  - `docs/CURRENT_STATE.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.5 (2026-03-03) burndown audit reconciliation (doc/backlog drift closure + residual-risk evidence revalidation) is tracked in:
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `REMAINING_WORK_TRACKING.md`
- WS12.7 runtime sanity sweep (2026-03-02 HST, live Android+iOS log triage while relay rollout was in progress) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.42 iOS/Android Missing-Direction Receipt Recovery (2026-03-13, added iOS outbound pending-outbox recovery when receipts arrive without local sent history to prevent stale in-app pending state) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  - `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- WS12.8 runtime recheck (2026-03-02 HST, post-redeploy relay verification + custody regression signal triage) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.9 iOS dashboard node-count hotfix (2026-03-03, discovery count remained correct while node count was inflated by stale/alias entries) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.10 runtime re-baseline + action roundup (2026-03-03 HST, custody gate stabilized, relay rollout skew closed, iOS main-thread I/O startup hotfix) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.13 wave-2 backlog consolidation (2026-03-03 HST, mixed-doc checklist normalization + verification script reconciliation) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
- WS12.11 iOS relay flapping diagnosis (2026-03-03 HST, no-code-change runtime triage for GCP relay visibility churn and potential state/race paths) is tracked in:
  - `docs/CURRENT_STATE.MD`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.12 Android<->iOS pairing message-delivery RCA (2026-03-03 HST, diagnosis-only pass for non-delivery despite active pairing) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.13 5-node orchestration script hardening + post-update issue slate triage (2026-03-03 HST) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.14 Android Bluetooth-only pairing diagnosis (2026-03-03 HST, no-code-change RCA for Android<->iOS BLE path failure) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.16 wave-2 runtime hardening pass (2026-03-03 HST, Android BLE race hardening + iOS relay/multipeer guardrails + delivery-attempt diagnostics timeline) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.17 wave-3 governance closure sweep (2026-03-03 HST, historical checklist triage + strict BLE-only/diagnostics hardening + deterministic harness additions) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.18 alpha readiness sanity + interoperability closure pass (2026-03-03 HST, Rust clippy strict cleanup + Android lint blocker closure + matrix/historical relocation/doc-lock sync) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.20 alpha readiness completion sweep (2026-03-03 HST, CLI/WASM parity closure + adapter-consumption closure + zero-gap interop matrix refresh + scripts operations guide) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `scripts/README.md`
- WS12.21 pairwise deep-dive status sweep (2026-03-03 HST, deep-dive script reruns + live probe attempt + pairing status reconciliation) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.22 Android+iOS crash-and-stability hardening sweep (2026-03-03 HST, iOS send-path crash guardrails + Android null-safety cleanup + fresh build/lint verification) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.23 pending-outbox synchronization reliability pass (2026-03-03 HST, active-peer-triggered queue promotion + route-alias retry drain for stuck pending messages) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.24 follow-up update (2026-03-03 HST, iOS->Android sender-state convergence gap tracking + Android conversation swipe-delete parity with iOS) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.25 mega-update intake + pending-sync route-hint hardening (2026-03-03 HST, run5/log-driven RCA + Android/iOS queue/receipt route fix + headless/relay dashboard-role unification) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.26 sender-state/preview convergence hotfix (2026-03-03 HST, receipt-path UI refresh propagation + conversation preview newest-message selection hardening) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.27 node-role classification correction (2026-03-03 HST, full-node vs headless misclassification fix + iOS/Android relay visibility validation snapshot) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.28 transport-regression hotfix (2026-03-03 HST, Android BLE `connectGatt` null crash-loop fix + Android/iOS special-use IPv4 filtering/local-IP selection hardening) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.29 known-issues consolidation + full-functionality burndown plan (2026-03-03 HST, field iOS send-crash evidence + Android stale-route loops + clean remediation sequence) is tracked in:
  - `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.30 live verification feedback-loop orchestration (2026-03-03 HST, sequential phase gates + all-node-pairing validation + per-attempt evidence bundles) is tracked in:
  - `scripts/run5-live-feedback.sh`
  - `scripts/README.md`
  - `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- WS12.31 stale-target convergence hardening + transport-priority clarification (2026-03-04 HST, discovered-route preference + failed-route de-persistence + connected-BLE target preference + iOS contact delete confirmation) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
- WS12.34 transport failure triage + 10-fix reliability sweep (2026-03-04 HST, Rust eprintln diagnostics + relay nil-safety + progressive backoff + never-expire policy + WiFi recovery flush + BLE timeout + dial candidate capping) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
- WS12.35 non-device reliability reconciliation (2026-03-06 UTC, baseline CI drift fixes + deterministic sender-state gate reconciliation + iOS diagnostics pull hardening) is tracked in:
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
- WS12.44 BLE freshness profiling + run5 visibility clarification (2026-03-13 UTC, Android freshness-first BLE routing + filtered-scan fallback + iOS app/system log split + known-vs-unknown visibility accounting) is tracked in:
  - `run5.sh`
  - `scripts/README.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
- GitHub + repo realignment first-pass planning audit (2026-03-07 UTC, comprehensive GitHub/repo hygiene and execution sequencing blueprint) is tracked in:
  - `docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
- GitHub-facing contributor-surface alignment (2026-03-07 UTC, support routing + issue config + PR template + active alpha version signaling) is tracked in:
  - `README.md`
  - `CONTRIBUTING.md`
  - `SECURITY.md`
  - `SUPPORT.md`
  - `docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
- Repo-side GitHub operating-model completion pass (2026-03-07 UTC, issue forms + Dependabot + Copilot instructions + docs sync hardening + workflow trigger cleanup) is tracked in:
  - `.github/ISSUE_TEMPLATE/`
  - `.github/dependabot.yml`
  - `.github/copilot-instructions.md`
  - `.github/workflows/ci.yml`
  - `.github/workflows/docker-publish.yml`
  - `.github/workflows/docker-test-suite.yml`
  - `.github/workflows/release.yml`
  - `docs/TESTING_GUIDE.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

- Global viability audit + 10-action-item implementation cheatsheet (2026-03-04 HST, comprehensive audit of what works / what's broken / what's missing for global viability) is tracked in:
  - `docs/global_viability_audit.md`
  - `docs/implementation_cheatsheet_3.4.2026.md`

## Documentation Governance Rules

1. Execution truth must come from Active docs listed above.
2. Backlog updates must go to `REMAINING_WORK_TRACKING.md`.
3. Residual risk updates must go to `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (or the active release's equivalent register).
4. Superseded status/audit reports should be moved to or referenced from `docs/historical/`, not duplicated as new "final" docs.
5. Use `iOS/` (uppercase-I) for all path references.
6. Run `./scripts/docs_sync_check.sh` before finalizing task output; fix failures in the same run.
7. If the run edits code, bindings, scripts that affect runtime/build behavior, or platform wiring, run the appropriate build verification command(s) before finalizing and record the result in the active doc chain.

- 2026-03-13: Added operational documentation for the iOS simulator launch ambiguity where a stale `platform IOS` app can remain installed in the simulator and fail only at bootstrap; canonical references updated in `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`.
- 2026-03-13: Consolidated the full live-debug conversation into canonical docs, including GCP relay repair, Android store-and-forward/send-debug findings, iOS stability/send-path observations, upgraded `run5.sh` visibility methodology, simulator launch recovery, and remaining transport/telemetry/runtime debt items.
