# SCMessenger Document Status Index

Status: Active  
Last updated: 2026-03-03

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

| Document | Status | Purpose |
| --- | --- | --- |
| `README.md` | Active | Repo entrypoint and quick-start links |
| `DOCUMENTATION.md` | Active | Main docs hub and navigation policy |
| `AGENTS.md` | Active | Codex run policy, including mandatory doc sync behavior |
| `docs/DOCUMENT_STATUS_INDEX.md` | Active | Lifecycle map (this file) |
| `docs/REPO_CONTEXT.md` | Active | Cross-component architecture and operating context |
| `docs/CURRENT_STATE.md` | Active | Verified current runtime/build state |
| `REMAINING_WORK_TRACKING.md` | Active | Backlog and active work tracking |
| `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` | Active | v0.2.0 milestone definition and sequencing |
| `docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md` | Active | Execution prompts per phase |
| `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` | Active | Residual-risk source of truth |
| `docs/PLATFORM_SUPPORT_MATRIX.md` | Active | Platform support baseline |
| `docs/PROTOCOL.md` | Active | Protocol identifiers and wire contract |
| `docs/TESTING_GUIDE.md` | Active | Validation commands and expected outcomes |
| `docs/EDGE_CASE_READINESS_MATRIX.md` | Active | Extreme environment readiness and hardening backlog |
| `docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md` | Active | Release synchronization and tagging checklist |
| `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md` | Active | v0.2.0 draft release notes |
| `docs/releases/RELEASE_NOTES_V0.1.2_GH.md` | Active | v0.1.2 GitHub release notes draft |

---

## 3) Active planned docs (future scope)

| Document | Status | Notes |
| --- | --- | --- |
| `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md` | Planned | WS13 workstream for v0.2.1, not in v0.2.0 scope |
| `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md` | Planned | WS14 workstream for v0.2.1 DM + DM Request notifications (iOS/Android/WASM, hybrid mode) |

---

## 4) Mixed-status docs (use with caution)

These documents may contain both current and historical sections; verify section markers before acting on them.

| Document | Status | Usage rule |
| --- | --- | --- |
| `docs/TRANSPORT_ARCHITECTURE.md` | Mixed | Use section markers (`[Current]`, `[Needs Revalidation]`) |
| `docs/GLOBAL_ROLLOUT_PLAN.md` | Mixed | Strategic context; validate against active milestone docs |
| `docs/UNIFIED_GLOBAL_APP_PLAN.md` | Mixed | Strategic context; backlog source remains `REMAINING_WORK_TRACKING.md` |
| `docs/NAT_TRAVERSAL_PLAN.md` | Mixed | Planning reference; verify against current transport code/tests |
| `docs/NAT_TRAVERSAL_GUIDE.md` | Mixed | Operational guidance; validate command/runtime assumptions |

---

## 5) Historical and superseded docs

1. `docs/historical/*` is `Historical` by default.
2. Root-level audit/status snapshots (for example old rollout/audit reports) are `Superseded` unless explicitly re-linked into the active chain.
3. Historical docs can inform context but must not override active docs.

---

## 6) Update rules

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

## 7) Recent Canonical Sync

1. WS11 (2026-03-03) public beta readiness surface updates are reflected in active docs (`docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
2. WS12 (2026-03-03) test-matrix/parity-lock updates are reflected in active docs (`docs/TESTING_GUIDE.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`).
3. WS12.5 (2026-03-03) burndown/audit closure updates are reflected in active docs (`docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/CURRENT_STATE.md`, and `REMAINING_WORK_TRACKING.md`).
4. WS12.6 (2026-03-03) optional closeout burndown updates are reflected in active docs (`docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/CURRENT_STATE.md`, and `REMAINING_WORK_TRACKING.md`).
