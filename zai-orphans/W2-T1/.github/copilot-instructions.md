# SCMessenger Copilot Instructions

Status: Active
Last updated: 2026-07-24

## [CRITICAL] Read First

**ALL AGENTS MUST READ:** `.github/COPILOT_AGENT_INSTRUCTIONS.md`

This document contains STRICT repository rules for file storage, temp files, and work organization.

**Key Rules:**
- [NO] Never use the system `/tmp` outside the repo
- [OK] Always use the repo-local `tmp/` directory for temp work
- All session files go in the repo-local `tmp/` subdirectory
- No emoji anywhere in this repo (`.claude/rules/no-emojis.md`, hook-enforced) -- use plain-text tags such as `[OK]`, `[WARNING]`, `[FAIL]`
- See `.github/COPILOT_AGENT_INSTRUCTIONS.md` for full details

---

## Canonical Documentation Sources (Priority Order)

Use these repository sources in order:

1. `AGENTS.md`
2. `DOCUMENTATION.md`
3. `docs/DOCUMENT_STATUS_INDEX.md`
4. `docs/CURRENT_STATE.md`
5. `REMAINING_WORK_TRACKING.md`
6. `HANDOFF/V1_0_0_EXECUTION_PLAN.md`
7. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

Current release line:

- `v0.3.5` is the active alpha baseline.
- Work toward `v1.0.0` is sequenced by `HANDOFF/V1_0_0_EXECUTION_PLAN.md` (two-phase DAG).

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

**This is enforced via `.github/COPILOT_AGENT_INSTRUCTIONS.md`**

- [NO] Never store session files outside the repo
- [NO] Never use system `/tmp`, `/var/tmp`, etc.
- [OK] Always use the repo-local `tmp/` subdirectory
- Example: `tmp/session_logs/`

