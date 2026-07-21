# Archived orchestration commands (superseded 2026-07-20)

These five slash-commands were consolidated into a single unified orchestrator.
They are kept here for reference and history only. They are NOT active commands:
this directory is outside `.claude/commands/`, so nothing here is registered as a
slash-command anymore.

## What replaced them

- **One command:** `/orchestrate` (`.claude/commands/orchestrate.md`) -- a thin
  launcher.
- **One brain:** `docs/ORCHESTRATION.md` -- the canonical loop, dispatch ladder,
  worker contract, security gates, and the Operating Contract (Section 0:
  delegation is mandatory; the protocol is model-agnostic).
- **One lake registry:** `SCM_UNIFIED_LAKE_ORCHESTRATION.md` -- endpoints,
  quotas, and the free vs paid tokens/$ comparison.

Everything these commands used to do is preserved as a selectable BACKEND in
`docs/ORCHESTRATION.md` Section 5 -- not as a separate command.

## Mapping (old command -> new backend)

| Archived command            | Now reachable as                                             |
|-----------------------------|-------------------------------------------------------------|
| `scmorc.md`                 | Backend `native` (Claude `claude -p` workers + Quota Governor) |
| `scm.md`                    | Backend `agent` (native `Agent` subagents)                  |
| `scmqwen.md`                | Backend `lanes` -> `delegate_task.py --provider qwen`       |
| `gemini-orchestrator.md`    | Backend `lanes` (foreign-worker dispatch)                   |
| `swarm.md`                  | Backend `swarm` (`orchestrator_manager.sh pool launch`)     |

The old `orchestrate.md` (ollama-swarm-only) was rewritten in place as the unified
launcher. Its full swarm procedure -- the 6-tier quota governor, the ollama agent
routing table, and the pool/monitor mechanics -- is preserved verbatim here as
`orchestrate-swarm-legacy.md`, and the `swarm` backend (`docs/ORCHESTRATION.md`
Section 5) points to it for operating detail.

## Why keep them

The archived files still contain useful detail not fully inlined into the protocol
(e.g. `scmorc.md`'s Quota Governor percentage tiers, the Windows build-serialization
war stories, and the per-backend worker-prompt templates). Consult them when
operating a specific backend; the canonical rules always win where they conflict
(`docs/ORCHESTRATION.md` supersedes these archived docs).

Do not re-add these to `.claude/commands/`. If a backend needs richer inline
docs, fold the detail into `docs/ORCHESTRATION.md` instead.
