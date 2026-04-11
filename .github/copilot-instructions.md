# SCMessenger Copilot Instructions

Status: Active
Last updated: 2026-04-11

## ⚠️ CRITICAL - Read First

**ALL AGENTS MUST READ:** `.github/COPILOT_AGENT_INSTRUCTIONS.md`

This document contains STRICT repository rules for file storage, temp files, and work organization.

**Key Rules:**
- ❌ NEVER use `/tmp` outside repo
- ✅ ALWAYS use `/tmp/` at repo root for temp work
- All session files go in repo `/tmp/` subdirectory
- See `.github/COPILOT_AGENT_INSTRUCTIONS.md` for full details

---

## Canonical Documentation Sources (Priority Order)

Use these repository sources in order:

1. `AGENTS.md`
2. `DOCUMENTATION.md`
3. `docs/DOCUMENT_STATUS_INDEX.md`
4. `docs/CURRENT_STATE.md`
5. `REMAINING_WORK_TRACKING.md`
6. `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
7. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

Current release line:

- `v0.2.0` is the active alpha baseline.
- `WS13` and `WS14` are currently planned `v0.2.1` follow-up scope.

Contributor-routing surfaces:

- `SUPPORT.md`
- `SECURITY.md`
- `.github/ISSUE_TEMPLATE/config.yml`

Do not treat mixed or historical docs as current source of truth unless the canonical docs above explicitly point to them.

## Mandatory Execution Rules

1. If a run changes behavior, scope, risk posture, scripts, tests, verification workflow, or operator workflow, update the canonical docs in the same run.
2. Run `./scripts/docs_sync_check.sh` (Unix / Git Bash) or `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs_sync_check.ps1` (Windows) before concluding any change-bearing run and resolve failures before finalizing.
3. If a run edits code, generated bindings, build wiring, or platform-specific implementation, run the appropriate build verification command(s) for the edited target(s) before concluding the run.
4. Final summaries must state which docs were updated, or why no doc updates were needed, and must report build verification status for edited targets.

## File Storage Rules (STRICT)

⚠️ **This is now enforced via `.github/COPILOT_AGENT_INSTRUCTIONS.md`**

- ❌ Never store session files outside the repo
- ❌ Never use system `/tmp`, `/var/tmp`, etc.
- ✅ Always use repo-local `/tmp/` subdirectory
- Example: `tmp/session_logs/`

