# AGENTS.md

Status: Active  
Last updated: 2026-03-02

Repository-scoped instructions for Codex agents.

## Core rule: Documentation sync is mandatory on every run

When a task changes behavior, scope, risk posture, tests, scripts, or operator workflow, update documentation in the same run.

This includes:

1. Fixing inconsistencies discovered during implementation, even if not explicitly requested.
2. Updating canonical docs before finalizing task output.
3. Keeping historical/superseded references out of active doc chains.

## Required docs touchpoints (review each run)

1. `DOCUMENTATION.md`
2. `docs/DOCUMENT_STATUS_INDEX.md`
3. `docs/CURRENT_STATE.md`
4. `REMAINING_WORK_TRACKING.md`
5. `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` (or current milestone equivalent)
6. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (or current release risk register)

## Passive consistency check

Run:

```bash
./scripts/docs_sync_check.sh
```

before finalizing work. If it fails, resolve documentation drift in the same run.

## Final response requirement

Always include a short documentation-sync summary:

1. Which docs were updated, or
2. Why no documentation updates were needed.
