# High-Impact Followup Section Actions

Last updated: **2026-02-23**

This file converts high-impact followup documents into explicit section actions.

Action key:
- `keep`: keep section as authoritative in current file
- `rewrite`: update section content in-place to match current canonical state
- `move`: extract current facts into canonical docs and leave section as historical context
- `delete/replace`: remove stale section usage and replace with canonical pointer/summary

## Executed Now

| File | Section(s) | Action | Target | Status |
| --- | --- | --- | --- | --- |
| `FEATURE_PARITY.md` | parity status claims, rollout checklist | `move` + `rewrite` | `REMAINING_WORK_TRACKING.md`, `docs/CURRENT_STATE.md` | done |
| `FEATURE_WORKFLOW.md` | workflow/checklist steps | `rewrite` | keep in file, aligned to tri-platform | done |
| `BOOTSTRAP.md` | bootstrap model + strategy | `rewrite` + `move` | `docs/UNIFIED_GLOBAL_APP_PLAN.md`, `REMAINING_WORK_TRACKING.md` | done |
| `ios/IMPLEMENTATION_SUMMARY.md` | untagged snapshot sections | `delete/replace` (usage) | `docs/CURRENT_STATE.md`, `ios/README.md` | done |
| `ios/FINAL_STATUS.md` | untagged snapshot sections | `delete/replace` (usage) | `docs/CURRENT_STATE.md`, `ios/README.md` | done |
| `ios/COMPLETE_STATUS.md` | untagged snapshot sections | `delete/replace` (usage) | `docs/CURRENT_STATE.md`, `ios/README.md` | done |
| `ios/IMPLEMENTATION_STATUS.md` | untagged snapshot sections | `delete/replace` (usage) | `docs/CURRENT_STATE.md`, `ios/README.md` | done |
| `PRODUCTION_READY.md` | readiness proof, checklists, summary claims | `move` + `delete/replace` | `docs/CURRENT_STATE.md`, `docs/TESTING_GUIDE.md` | done |
| `INTEGRATION_COMPLETE.md` | integration verdict and performance claims | `move` + `delete/replace` | `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md` | done |
| `INTEGRATION_VERIFICATION.md` | phase activation claims | `move` + `delete/replace` | `docs/CURRENT_STATE.md`, `docs/REPO_CONTEXT.md` | done |
| `DOCKER_TEST_SETUP_COMPLETE.md` | setup-complete and CI claims | `rewrite` + `move` | `docker/README.md`, `docs/CURRENT_STATE.md` | done |
| `DOCKER_TEST_QUICKREF.md` | command/reference card | `rewrite` | `docker/README.md`, `docs/CURRENT_STATE.md` | done |
| `DOCKER_QUICKSTART.md` | cloud/bootstrap quickstart | `rewrite` + `move` | `docs/UNIFIED_GLOBAL_APP_PLAN.md`, `docs/CURRENT_STATE.md` | done |
| `docs/REMEDIATION_PLAN.md` | remediation roadmap checkpoints | `move` + `delete/replace` | `REMAINING_WORK_TRACKING.md`, `docs/GLOBAL_ROLLOUT_PLAN.md` | done |
| `SECURITY_AUDIT_NOTES.md` | audit findings and actions | `rewrite` + `move` | `SECURITY.md`, `docs/CURRENT_STATE.md` | done |
| `android/IMPLEMENTATION_STATUS.md` | implementation checkpoint claims | `move` + `delete/replace` | `docs/CURRENT_STATE.md`, `android/README.md` | done |

## Next High-Impact Queue

High-impact queue completed for this pass. Lower-impact followup markdown docs were also normalized with explicit section-action outcome blocks on 2026-02-23.

Normalization verification snapshot:
- followup markdown docs in tracker: 53
- followup markdown docs with explicit section-action outcome block: 53
- missing action blocks: 0

## Verification Rules

For each queued file before status flip to `validated`:
1. Section-level actions are tagged in-file or in this matrix.
2. Any current claim is either rewritten in-place or moved into canonical docs.
3. Stale "complete/final/ready" claims are replaced with canonical pointers.
4. Canonical docs remain consistent:
   - `README.md`
   - `DOCUMENTATION.md`
   - `docs/REPO_CONTEXT.md`
   - `docs/CURRENT_STATE.md`
   - `REMAINING_WORK_TRACKING.md`
   - `docs/GLOBAL_ROLLOUT_PLAN.md`
   - `docs/UNIFIED_GLOBAL_APP_PLAN.md`
