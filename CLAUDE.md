
Guidance for Claude Code sessions in this repo. This file is re-injected every
turn — it stays high-level; per-topic detail lives in `docs/CLAUDE_REFERENCE.md`
(build/test command reference, core module map, test inventory, swarm internals).
Read that file's relevant section on demand instead of re-deriving.

## Project Overview

SCMessenger is a sovereign encrypted decentralized messaging mesh. A Rust core
(`scmessenger-core`) handles identity, crypto, P2P transport, and storage;
platform clients (Android/Kotlin, iOS/Swift, WASM/browser, CLI) consume it via
UniFFI bindings or JSON-RPC.

**Active release line:** v0.3.5, working toward v1.0.0. Sequencing is governed
by `HANDOFF/V1_0_0_EXECUTION_PLAN.md` (two-phase DAG; Phase 1 = Windows/Android
transport parity — top priority). Dispatch order: `HANDOFF/todo/_QUEUE.md`.

**One unified orchestrator (consolidated 2026-07-20).** There is a single
orchestrator command, `/orchestrate` (`.claude/commands/orchestrate.md`), a thin
launcher over the canonical protocol `docs/ORCHESTRATION.md`. The old per-backend
commands (`/scmorc`, `/scm`, `/scmqwen`, `/gemini-orchestrator`, `/swarm`) are
archived under `.claude/archive/commands/`; their behaviour survives as selectable
BACKENDS, not separate commands:
- **`lanes`** (default) — script dispatch to free/paid API lakes via
  `scripts/delegate_task.py`. The only backend a non-Claude orchestrator needs.
- **`native`** — headless `claude -p` workers on the Anthropic subscription
  (the old `/scmorc`; Quota Governor applies). Escalation/audit only.
- **`agent`** — native `Agent`-tool subagents (the old `/scm`).
- **`swarm`** — ollama pool via `orchestrator_manager.sh` (the old `/orchestrate`
  + `/swarm`); small free tier.

Delegation is mandatory for every task and holds no matter which model is
orchestrating (`docs/ORCHESTRATION.md` Section 0). The lake fleet — free/trial and
cheap-paid providers, endpoints, quotas, tokens/$ — is in
`SCM_UNIFIED_LAKE_ORCHESTRATION.md`.

Cross-mode protocol (state machine, dispatcher, tier routing, commit
authority, token-efficiency rules): `docs/ORCHESTRATION.md` — supersedes role
docs where they conflict. Foreign-model dispatch goes through
`scripts/delegate_task.py` (always pass `--verify` so fix loops run locally,
not through the orchestrator).

## Workspace Structure
