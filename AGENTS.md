# AGENTS.md

Status: Active
Last updated: 2026-03-19

Repository-scoped instructions for Codex agents.

## ⚠️ MANDATORY: Log Extraction Standard

**All AI agents MUST read and follow:** `LOG_EXTRACTION_STANDARD.md`

When working with iOS or Android logs:
- ✅ **iOS:** Use `ios_extractor.py` (mandatory)
- ✅ **Android:** Use `adb_extractor.py` (mandatory)
- ❌ **Do NOT** create ad-hoc log extraction commands
- ❌ **Do NOT** ask users to manually run adb logcat or idevicesyslog

See `LOG_EXTRACTION_STANDARD.md` for complete requirements.

---

## Core rule: Documentation sync is mandatory on every run

When a task changes behavior, scope, risk posture, tests, scripts, or operator workflow, update documentation in the same run.

This includes:

1. Fixing inconsistencies discovered during implementation, even if not explicitly requested.
2. Updating canonical docs before finalizing task output.
3. Keeping historical/superseded references out of active doc chains.

## Core rule: Mandatory Build Verification

**You MUST verify all edited builds before completing any session.** If you push code updates or patches for a given build (e.g. Android Kotlin or iOS Swift), you MUST run the appropriate compiler/builder to prove that the edits compile successfully before concluding the conversation with the user.

This rule applies whenever behavior, interfaces, scripts, generated code, or platform wiring changes. Do not treat "small" code edits as exempt.

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

## Required closeout checklist on every change-bearing run

1. Re-read the active canonical doc chain before editing implementation or workflow behavior.
2. Update the canonical docs in the same run whenever behavior, scope, scripts, risks, verification commands, or operator workflow changes.
3. Run `./scripts/docs_sync_check.sh` before finalizing; fix failures in the same run.
4. If any code, build wiring, generated bindings, or scripts affecting a build target changed, run the appropriate build verification command(s) before finalizing.
5. Do not conclude the session until both documentation sync and required build verification are complete, or the blocker is explicitly called out with the exact failed command and reason.

## Final response requirement

Always include a short documentation-sync summary:

1. Which docs were updated, or
2. Why no documentation updates were needed.

## Estimation Rules

**BANNED: Time-based estimates are strictly prohibited.**

Do NOT use:
- "2 hours", "30 minutes", "~1 hour"
- "Estimated effort: X hours"
- Any time-based duration estimates

**ALLOWED: Lines of Code (LOC) estimates only.**

Use LOC-based estimates like:
- "~50 LOC change"
- "~200 LOC across 3 files"
- "Single function modification (~30 LOC)"

Rationale: Time estimates are unreliable and vary by developer experience. LOC provides a concrete measure of code change scope.
