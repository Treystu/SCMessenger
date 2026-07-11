# SCMessenger Orchestration Protocol (Master)

Status: Active
Last updated: 2026-07-11

This is the single source of truth for HOW work gets orchestrated in this
repo, regardless of which model is orchestrating (Claude, Gemini, Qwen, or a
human). Role-specific docs (`docs/GEMINI_ORCHESTRATOR.md`, the `/scmorc` and
`/scmqwen` command files) are subordinate to this file: where they conflict,
this file wins. WHAT to work on is owned by `HANDOFF/todo/_QUEUE.md` and
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` -- never relitigate sequencing here.

## 1. The one state machine

- `HANDOFF/todo/_QUEUE.md` is the live pick list. Pull from the top,
  respect dependency notes.
- Task lifecycle: `todo/` -> (work) -> `done/`. There is no other state.
- ATOMICITY RULE: the task file move to `done/` and the `_QUEUE.md` status
  update happen in the SAME commit as the change that completes the task.
  Stale queue entries cost every future session that reads the queue.
- Adversarial-review verdicts land in `HANDOFF/review/`, named after the
  task. A crypto/transport/routing/privacy task without a verdict file is
  NOT done, no matter what the queue says.

## 2. The one dispatcher

`scripts/delegate_task.py` is the only sanctioned way to send work to
foreign models. It reads keys from env or `~/.config/scmorc/*.env`
(dashscope.env, openrouter.env) -- NEVER hardcode keys.

```
python scripts/delegate_task.py \
  --task HANDOFF/todo/<TASK>.md \
  --provider qwen --tier <tier> \
  --files <files the model must see> \
  --apply \
  --verify "<local gate command>" --max-rounds 3
```

- `--verify` runs the gate after apply and auto-feeds failures back to the
  model for up to `--max-rounds` total attempts. USE IT ON EVERY CODE
  DISPATCH -- compile-fix churn must never round-trip through the
  orchestrator again.
- Typical verify commands: `cargo check -p scmessenger-core` (Rust),
  `python -m py_compile <file>` (Python), `bash scripts/docs_sync_check.sh`
  (docs).
- Responses are saved under `tmp/` for audit; `--apply` writes files
  directly.

## 3. Lane inventory and when to use each

| Lane | Entry | Cost | Use for |
|---|---|---|---|
| Qwen/DashScope | `delegate_task.py --provider qwen --tier ...` | Free (~1M tok/model) | PRIMARY implementation lane: Rust, Python, docs, reviews |
| OpenRouter | `delegate_task.py --provider openrouter --model <id>` | Free tier | Fallback when a Qwen model is saturated; large-context jobs |
| Gemini/agy | Antigravity session per `docs/GEMINI_ORCHESTRATOR.md` | Free | Orchestration-only driver of the Qwen lane (writes task files, dispatches, verifies) |
| Ollama cloud | swarm mode (`/orchestrate`, `/swarm`) | Free, weekly cap | Burst capacity when quota available (check first: it exhausts) |
| Claude headless | `/scmorc` (per-task `claude -p` workers) | Anthropic quota | Audit-gate verdicts, judgment calls, escalations ONLY |
| Claude native | main session + subagents | Anthropic quota | Synthesis, design, final gates. Subagents at haiku/sonnet tier only -- NEVER Fable-tier fan-outs |

Known-broken path (do not retry): `claude --settings settings.local.OR.json`
(Claude Code cannot auth against OpenRouter directly; use a local shim like
claude-code-router if the full harness on free models is ever needed).

## 4. Tier routing (one table, all lanes)

Execution-plan tags map to models as follows:

| Plan tag | Qwen (`--tier`) | Claude worker | Meaning |
|---|---|---|---|
| [HAIKU] | `standard` or `flash` | haiku | Mechanical, fully specified, low blast radius |
| [SONNET] | `max` | sonnet | Scoped implementation, spec exists |
| [OPUS+] | `thinking` | fable/opus (main session) | Design, diagnosis, spec-writing |
| [AUDIT-GATE] | `thinking` (read-only verdict) | crypto-security-auditor subagent | Adversarial review, mandatory for crypto/transport/routing/privacy |

Escalation direction is UP only on judgment calls: a worker that hits an
architecture decision stops and reports; it does not improvise.

## 5. Commit authority

| Actor | May commit? | Prefix |
|---|---|---|
| Claude native session | Yes (local only, never push unasked) | `native:` |
| Claude headless workers | Yes via orchestrator checkpoint | `scmorc:` |
| Swarm/foreign-implemented work, committed by a Claude session | Yes | `swarm:` |
| Gemini/agy | NO -- report only, per AGENTS.md | -- |
| Qwen/OpenRouter/Ollama workers | NO -- they only return file contents | -- |

Every commit: gates recorded in the message, no secrets (`git diff --cached`
scan), no emoji (pre-commit hook enforces), task-file moves included
(Section 1 atomicity rule).

## 6. Verification gates (unchanged, listed for completeness)

`CARGO_INCREMENTAL=0` always; never two build tools concurrently.
Rust: `cargo build --workspace` + `cargo test --workspace --no-run` (the
compile gate; required before any task is done). WASM:
`cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`.
Android: `cd android && ./gradlew assembleDebug -x lint --quiet`.
Docs: `scripts/docs_sync_check.sh`. Full reference:
`docs/CLAUDE_REFERENCE.md` section 1.

## 7. Token-efficiency rules (hard-learned 2026-07-11)

1. Verification loops belong to `--verify`, not to orchestrator round-trips.
2. Do not round-trip full files for small fixes when a scoped task +
   `--verify` can converge locally.
3. Keep dispatch context minimal: send only the files the model must edit
   or must see to edit correctly. `wc -c` first if unsure.
4. Anthropic tokens are the scarcest resource: Claude orchestrates and
   judges; free lanes implement. No Fable-tier subagent fan-outs, ever.
5. One-off helper scripts go in `tmp/`, not `scripts/`. If it is worth
   keeping, it is worth a task file and a review.
6. Spread Qwen load across tiers (five separate per-model quotas) instead
   of exhausting one model.
