# SCMessenger Document Status Index

Status: Active  
Last updated: 2026-03-07

Purpose: classify documentation by lifecycle state so execution decisions use authoritative files and historical content remains discoverable without causing drift.

---

## 1) Status classes

1. `Active` - authoritative for current implementation and execution decisions.
2. `Planned` - approved future-scope plan, not yet implemented.
3. `Mixed` - contains useful content but requires section-level interpretation/revalidation.
4. `Historical` - archived snapshots and prior reports; context only.
5. `Superseded` - replaced by newer canonical docs; keep only for traceability.

---

## 2) Active canonical chain (use first)

| Document                                              | Status | Purpose                                                        |
| ----------------------------------------------------- | ------ | -------------------------------------------------------------- |
| `README.md`                                           | Active | Repo entrypoint and quick-start links                          |
| `DOCUMENTATION.md`                                    | Active | Main docs hub and navigation policy                            |
| `SUPPORT.md`                                          | Active | Support/reporting routing for contributors and operators       |
| `AGENTS.md`                                           | Active | Codex run policy, including mandatory doc sync behavior        |
| `docs/DOCUMENT_STATUS_INDEX.md`                       | Active | Lifecycle map (this file)                                      |
| `docs/REPO_CONTEXT.md`                                | Active | Cross-component architecture and operating context             |
| `docs/CURRENT_STATE.md`                               | Active | Verified current runtime/build state                           |
| `REMAINING_WORK_TRACKING.md`                          | Active | Backlog and active work tracking                               |
| `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`          | Active | Consolidated known-issues ledger and remediation sequence      |
| `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`                 | Active | v0.2.0 milestone definition and sequencing                     |
| `docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md`              | Active | Execution prompts per phase                                    |
| `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`               | Active | Residual-risk source of truth                                  |
| `docs/PLATFORM_SUPPORT_MATRIX.md`                     | Active | Platform support baseline                                      |
| `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`                 | Active | Cross-platform function completeness + interoperability matrix |
| `docs/PROTOCOL.md`                                    | Active | Protocol identifiers and wire contract                         |
| `docs/TESTING_GUIDE.md`                               | Active | Validation commands and expected outcomes                      |
| `docs/EDGE_CASE_READINESS_MATRIX.md`                  | Active | Extreme environment readiness and hardening backlog            |
| `scripts/README.md`                                   | Active | Canonical launch/debug/5-node script operations guide          |
| `docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md` | Active | Release synchronization and tagging checklist                  |
| `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`         | Active | v0.2.0 draft release notes                                     |
| `docs/releases/RELEASE_NOTES_V0.1.2_GH.md`            | Active | v0.1.2 GitHub release notes draft                              |

---

## 3) Active supporting audits and execution plans

| Document                                            | Status | Purpose                                                                  |
| --------------------------------------------------- | ------ | ------------------------------------------------------------------------ |
| `docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md` | Active | Planning-only GitHub/repo operating-model audit and execution blueprint |
| `docs/global_viability_audit.md`                    | Active | Supporting viability audit; context for execution, not canonical truth   |
| `docs/implementation_cheatsheet_3.4.2026.md`        | Active | Supporting implementation reference derived from audit findings          |

---

## 4) Active planned docs (future scope)

| Document                                              | Status  | Notes                                                                                    |
| ----------------------------------------------------- | ------- | ---------------------------------------------------------------------------------------- |
| `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md` | Planned | WS13 workstream for v0.2.1, not in v0.2.0 scope                                          |
| `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`                | Planned | WS14 workstream for v0.2.1 DM + DM Request notifications (iOS/Android/WASM, hybrid mode) |

---

## 5) Mixed-status docs (use with caution)

These documents may contain both current and historical sections; verify section markers before acting on them.

| Document                          | Status | Usage rule                                                             |
| --------------------------------- | ------ | ---------------------------------------------------------------------- |
| `docs/TRANSPORT_ARCHITECTURE.md`  | Mixed  | Use section markers (`[Current]`, `[Needs Revalidation]`)              |
| `docs/GLOBAL_ROLLOUT_PLAN.md`     | Mixed  | Strategic context; validate against active milestone docs              |
| `docs/UNIFIED_GLOBAL_APP_PLAN.md` | Mixed  | Strategic context; backlog source remains `REMAINING_WORK_TRACKING.md` |
| `docs/NAT_TRAVERSAL_PLAN.md`      | Mixed  | Planning reference; verify against current transport code/tests        |
| `docs/NAT_TRAVERSAL_GUIDE.md`     | Mixed  | Operational guidance; validate command/runtime assumptions             |

---

## 6) Historical and superseded docs

1. `docs/historical/*` is `Historical` by default.
2. Root-level audit/status snapshots (for example old rollout/audit reports) are `Superseded` unless explicitly re-linked into the active chain.
3. Historical docs can inform context but must not override active docs.

---

## 7) Update rules

1. Any document used as execution truth must include `Status:` and `Last updated:` headers.
2. When superseding a doc:
   - update the active canonical file first,
   - move or classify the old file as `Historical`/`Superseded`,
   - add or update links in `DOCUMENTATION.md` and this index.
3. If active docs conflict, precedence is:
   1. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (risk posture),
   2. `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` (scope/order),
   3. `REMAINING_WORK_TRACKING.md` (backlog),
   4. `docs/CURRENT_STATE.md` (verified baseline),
   5. `docs/REPO_CONTEXT.md` (architecture context).
4. `./scripts/docs_sync_check.sh` should pass before finalizing implementation work or documentation-only changes.

## 8) Recent Canonical Sync

1. WS11 (2026-03-03) public beta readiness surface updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
2. WS12 (2026-03-03) test-matrix/parity-lock updates are reflected in active docs (`docs/TESTING_GUIDE.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
3. WS12.5 (2026-03-03) burndown/audit closure updates are reflected in active docs (`docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/CURRENT_STATE.md`, and `REMAINING_WORK_TRACKING.md`).
4. WS12.6 (2026-03-03) optional closeout burndown updates are reflected in active docs (`docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/CURRENT_STATE.md`, and `REMAINING_WORK_TRACKING.md`).
5. WS12.7 (2026-03-02 HST) live runtime sanity/hotfix updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
6. WS12.8 (2026-03-02 HST) post-redeploy runtime recheck updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
7. WS12.9 (2026-03-03) iOS dashboard node-count hotfix updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
8. WS12.10 (2026-03-03 HST) runtime re-baseline + action roundup updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
9. WS12.13 (2026-03-03 HST) wave-2 backlog consolidation updates are reflected in active docs (`docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`).
10. WS12.11 (2026-03-03 HST) iOS relay flapping diagnosis updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
11. WS12.12 (2026-03-03 HST) Android/iOS pairing message-delivery RCA updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
12. WS12.13 (2026-03-03 HST) 5-node script hardening + post-update issue-slate triage updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
13. WS12.14 (2026-03-03 HST) Android Bluetooth-only pairing diagnosis updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
14. WS12.16 (2026-03-03 HST) wave-2 runtime hardening pass updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
15. WS12.17 (2026-03-03 HST) wave-3 governance closure updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), with historical checklist triage updates under `docs/historical/*`.
16. WS12.18 (2026-03-03 HST) alpha readiness sanity + interoperability closure updates are reflected in active docs (`docs/CURRENT_STATE.md`, `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`, `docs/PLATFORM_SUPPORT_MATRIX.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
17. WS12.19 (2026-03-03 HST) doc/folder cleanup correction updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `iOS/README.md`, `iOS/XCODE_SETUP.md`, and `mobile/README.md`) and in historical archive clarification (`docs/historical/iOS/scripts/README.md`).
18. WS12.20 (2026-03-03 HST) alpha readiness completion sweep updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/PLATFORM_SUPPORT_MATRIX.md`, and `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`) plus active script operations guide (`scripts/README.md`).
19. WS12.21 (2026-03-03 HST) pairwise deep-dive status sweep updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including live probe blocker evidence for physical iOS availability.
20. WS12.22 (2026-03-03 HST) Android+iOS crash-and-stability hardening sweep updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including iOS send-path crash mitigation and Android null-safety cleanup evidence.
21. WS12.23 (2026-03-03 HST) pending-outbox synchronization reliability pass updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including active-peer-triggered queue promotion and route-alias retry-drain behavior.
22. WS12.24 (2026-03-03 HST) sender-state convergence + conversation swipe-delete parity updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including iOS->Android `stored`-after-delivery tracking and Android swipe-delete parity with iOS.
23. WS12.25 (2026-03-03 HST) mega-update intake + pending-sync route hardening updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including run5/log RCA, Android+iOS route-hint/receipt-candidate hardening, and dashboard role unification into two node buckets (`Node`, `Headless Node`).
24. WS12.26 (2026-03-03 HST) sender-state/preview convergence hotfix updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including receipt-driven `messageUpdates` propagation on Android+iOS and iOS conversation preview newest-message selection hardening.
25. WS12.27 (2026-03-03 HST) node-role classification correction updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including full-vs-headless misclassification fix, relay-only classification guardrail updates, and fast iOS/Android relay-visibility validation snapshot evidence.
26. WS12.28 (2026-03-03 HST) transport-regression hotfix updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including Android BLE `connectGatt` null-return crash-loop mitigation and Android+iOS special-use IPv4 filtering/local-IP selection hardening to reduce stale/unreachable dial churn.
27. WS12.29 (2026-03-03 HST) known-issues consolidation + clean remediation-plan updates are reflected in active docs (`docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`), including field iOS send-crash evidence, Android stale-route diagnostics, and the then-open iOS contact-delete confirmation TODO (closed in WS12.31).
28. WS12.30 (2026-03-03 HST) live verification feedback-loop orchestration updates are reflected in active docs (`scripts/README.md`, `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`) plus new active harness script (`scripts/run5-live-feedback.sh`) for strict sequential phase gates and all-node-pairing validation.
29. WS12.31 (2026-03-04 HST) stale-target convergence hardening + transport-priority clarification updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, and `DOCUMENTATION.md`), including discovered-route-first candidate preference, strict route-key validation fallback to runtime evidence, failed-route de-persistence, connected-BLE-target preference, and iOS contact-delete confirmation prompt implementation.
30. WS12.35 (2026-03-06 UTC) non-device reliability reconciliation updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, and `DOCUMENTATION.md`), including wasm sender-timestamp compile drift closure, iOS MainActor isolation hardening in Multipeer transport, Android mesh-participation semantics test alignment, deterministic delivery-state monotonicity gate canonicalization, and iOS diagnostics pull stability safeguards in `scripts/run5-live-feedback.sh`.
31. WS12.36 (2026-03-07 UTC) repo/GitHub operating-model planning-audit updates are reflected in active docs (`DOCUMENTATION.md`, `docs/DOCUMENT_STATUS_INDEX.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`) plus new supporting audit blueprint (`docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`).
32. WS12.36 follow-up (2026-03-07 UTC) GitHub-facing contributor-surface alignment updates are reflected in active docs (`README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`, `DOCUMENTATION.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`) plus GitHub config surfaces (`.github/CODEOWNERS`, `.github/ISSUE_TEMPLATE/config.yml`, `.github/pull_request_template.md`).
