---
name: docs-sync
description: Run the repo's documentation sync check (scripts/docs_sync_check.sh). Use after any documentation change, or as part of finalizing a run per CLAUDE.md's mandatory doc-sync rule.
allowed-tools: Bash, Read, Grep, Glob, Edit
---

Run:

```bash
bash ./scripts/docs_sync_check.sh
```

(PowerShell fallback if Git Bash somehow isn't available: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs_sync_check.ps1`)

Report pass/fail. On failure, list every file/link it flagged. Fix what's mechanical directly (missing `Status:`/`Last updated:` headers near the top of a file, stale relative links that clearly moved) — for anything requiring a content judgment call (what the correct status actually is, which doc should own new content), flag it instead of guessing. Re-run after fixing to confirm it now passes.
