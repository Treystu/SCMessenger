
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

**Four operating modes share this repo:**
1. **Native Claude Code** (default) — single session using the subagents/skills/hooks below.
2. **Ollama-cloud swarm** — only under `/orchestrate` or `/swarm`; see `docs/CLAUDE_REFERENCE.md` section 4.
3. **`/scmorc`** — headless per-task `claude -p` workers on the Anthropic subscription (`.claude/commands/scmorc.md`). Gold standard for batch processing.
4. **`/scm`** — native cowork orchestrator using the `Agent` tool (`.claude/commands/scm.md`).

Cross-mode protocol (state machine, dispatcher, tier routing, commit
authority, token-efficiency rules): `docs/ORCHESTRATION.md` — supersedes role
docs where they conflict. Foreign-model dispatch goes through
`scripts/delegate_task.py` (always pass `--verify` so fix loops run locally,
not through the orchestrator).

## Workspace Structure
