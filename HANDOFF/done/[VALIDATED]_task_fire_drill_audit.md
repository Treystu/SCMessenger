---
Model: gemma3:4b:cloud
Budget: 250
---

## WATCHDOG WARNING
You are operating under a strict token budget as specified in the frontmatter Budget field.
An OS-level watchdog (TaskGovernor.ps1) is monitoring your debug logs.
If you exceed this budget, your process will be terminated immediately.
Optimize for minimal API calls. Batch your reads. Prefer targeted grep over full file reads.

# Fire Drill Audit: Git Status Check

## Objective
Run `git status` in the repository root and report the current working tree state.

## Steps
1. Execute: `git status`
2. Report the output exactly as received.
3. Move this file from `HANDOFF/todo/` to `HANDOFF/done/`.

## Success Criteria
- Git status output captured and reported.
- Task file moved to `HANDOFF/done/`.
