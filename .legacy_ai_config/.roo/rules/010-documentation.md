# Documentation Rules

## Required Docs Touchpoints
Every change-bearing run MUST review and update:
1. DOCUMENTATION.md
2. docs/DOCUMENT_STATUS_INDEX.md
3. docs/CURRENT_STATE.md
4. REMAINING_WORK_TRACKING.md
5. docs/MILESTONE_PLAN_V0.2.0_ALPHA.md (or current milestone equivalent)
6. docs/V0.2.0_RESIDUAL_RISK_REGISTER.md (or current release risk register)

## Documentation Sync Check
Run `./scripts/docs_sync_check.sh` before finalizing work. If it fails, resolve documentation drift in the same run.

## Canonical Doc Chain
Maintain the canonical documentation chain. Historical/superseded references must be kept out of active doc chains.

## Classification
Follow DOCUMENT_STATUS_INDEX.md classifications for all documentation updates.
