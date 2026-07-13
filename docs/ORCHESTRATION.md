# SCMessenger Orchestration Protocol (Master)

Status: Active
Last updated: 2026-07-13 (canonical launch sequence added, Section 2.1;
farm-first sequencing per `HANDOFF/plans/FARM_FINAL_PLAN.md`; fusion_lite
integrated into the verification ladder)

This is the single source of truth for HOW work gets orchestrated in this
repo, regardless of which model is orchestrating (Claude, Gemini, Qwen, or a
human). Role-specific docs (`docs/GEMINI_ORCHESTRATOR.md`, the `/scmorc` and
`/scmqwen` command files) are subordinate to this file: where they conflict,
this file wins. WHAT to work on is owned by `HANDOFF/todo/_QUEUE.md`
(re-ranked farm-first 2026-07-13), `HANDOFF/plans/FARM_FINAL_PLAN.md` (the
farm deployment plan: WS-FARM gap ledger, FD readiness drills, AD
architecture decisions), and `HANDOFF/V1_0_0_EXECUTION_PLAN.md` -- never
relitigate sequencing here.

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

### 2.1 The launch sequence (canonical, all modes)

Every orchestrator mode (`/scmorc`, `/scmqwen`, `/scm`, Gemini, swarm)
follows THIS sequence per task. Mode command files carry mode-specific
mechanics only; the sequence itself lives here.

1. **PICK.** Pull from the top of `HANDOFF/todo/_QUEUE.md` (farm-first
   order, re-ranked 2026-07-13). NEW farm tasks not yet cut into task files
   have ticket-ready specs in `HANDOFF/plans/FARM_FINAL_PLAN.md` Section 4
   (WS-FARM-A..H) -- cut the task file first, then dispatch it. Respect the
   standing freezes (PQC-11/13 behind PQC_07 root-key fix; PQC-09 wiring
   behind the AD-8 onion seam freeze).
2. **VALIDATE** (orchestrator-local, cheap): read the task file, grep the
   target. FALSE_POSITIVE / ALREADY_WIRED -> done/ with note.
   NEEDS_REVIEW -> operator. VALID -> continue.
3. **ROUTE down the free-first ladder** (Section 3 has full lane details;
   Section 4 maps plan tags to models):
   - a. **Qwen scripted** (`delegate_task.py --provider qwen --tier <t>
     --files ... --apply --verify "<gate>" --max-rounds 3`) -- PRIMARY for
     all implementation. The `--verify` loop self-corrects locally.
   - b. **agy/Gemini** -- in parallel (separate free pool) for small,
     well-bounded single-file/mechanical agentic edits. One tree-editing
     agy at a time; AGENTS.md FOREIGN WORKER header; no commit authority.
   - c. **OpenRouter free** (`delegate_task.py --provider openrouter
     --model <id>`) -- spillover when Qwen tiers saturate; large-context.
   - d. **fusion_lite** (`scripts/fusion_lite.py`, PAID, hard-capped) --
     never an implementation lane. Narrow second-opinion panel+judge for
     planning/verification questions answerable from pasted context.
     Cost discipline is absolute: `--max-cost 0.01` (the default) is the
     OPERATOR-SET ceiling -- never raise it without explicit operator
     approval in-session; set `FUSION_LITE_EXPECT_KEY_LABEL`; log actual
     cost per run in the dispatch log. See `docs/FUSION_LITE.md`.
   - e. **Claude** (headless `claude -p` per `/scmorc`, or native
     subagents) -- judgment, audit-gate verdicts, design decisions, and
     the escalation terminus ONLY. Anthropic tokens are the scarcest
     resource; they never do work a free lane can.
   Escalation is by RECORDED FAILURE only: two failed attempts at a rung
   before moving up; never re-run an identical model+prompt more than
   twice.
4. **VERIFY centrally.** The orchestrator is the single writer for build
   gates (Windows serialization rule). Scoped gates per Section 6.
   Additional triangulation rung for DELIVERY-LOGIC diffs (outbox, receipt,
   custody, retry -- the FARM WS-A class, per
   `OUTBOX_FLUSH_ON_CONNECT_RETRY.md`'s protocol): one fusion_lite panel
   run OR 3 distinct Qwen verifier dispatches must find no issues before
   commit. This supplements, never replaces, the `[AUDIT-GATE]` on
   crypto/transport/routing/privacy.
5. **CLOSE with evidence.** Farm-plan tasks tied to an FD drill
   (`FARM_FINAL_PLAN.md` Section 5) close only when the drill/sim evidence
   is logged to the dated ledger doc. Task-file move + queue update + commit
   in one atomic step (Section 1). Dispatch log line includes lane, model,
   result, and cost (fusion_lite runs: actual dollars).

## 3. Lane inventory and when to use each

| Lane | Entry | Cost | Use for |
|---|---|---|---|
| Qwen/DashScope | `delegate_task.py --provider qwen --tier ...` | Free (~1M tok/model) | PRIMARY implementation lane (operator, 2026-07-07): feed-content generate -> script applies + auto-verifies; never touches the tree concurrently |
| agy (Antigravity Gemini workers) | `agy -p "<prompt>" --model "Gemini 3.5 Flash (Medium)" --dangerously-skip-permissions --add-dir <repo>` (prompt/output files under `tmp/scmorc/agy-*`) | Free (separate pool) | Agentic (tool-using) free lane. Current fleet (2026-07-11): Gemini 3.5 Flash + Gemini 3.1 Pro (model strings verbatim from `agy models`, e.g. "Gemini 3.1 Pro (High)"). Print mode is reliable ONLY for small, well-bounded single-file/mechanical edits; larger work needs an interactive session. ONE tree-editing agy at a time (or give each its own git worktree). `--print-timeout` does not reliably kill the child -- monitor and Stop-Process if CPU sits near zero past the deadline. No commit authority |
| OpenRouter | `delegate_task.py --provider openrouter --model <id>` | Free tier | Fallback when a Qwen model is saturated; large-context jobs |
| fusion_lite (OpenRouter, hand-rolled) | `scripts/fusion_lite.py --prompt-file <f> --panel <models> --judge <model>` | ~$0.0001-0.0005/run, hard-capped by `--max-cost` (default $0.01) | Narrow planning/verification second-opinion: 2-4 small paid models + 1 judge synthesis, cost-bounded and BYOK-blocked. NOT OpenRouter's own "Fusion" feature -- that was evaluated 2026-07-11 and rejected (forces uncontrollable web-tool calls onto panel members, one test cost $0.057 against a sub-cent estimate). See `docs/FUSION_LITE.md` for the full incident and usage. Not a substitute for the `[AUDIT-GATE]` adversarial-review requirement below on crypto/transport/routing/privacy work. |
| Gemini orchestrator | Antigravity session per `docs/GEMINI_ORCHESTRATOR.md` | Free | Orchestration-only role variant: drives the Qwen lane (writes task files, dispatches, verifies), zero LOC itself |
| Ollama cloud | swarm mode (`/orchestrate`, `/swarm`) | Free, weekly cap | Burst capacity when quota available (check first: it exhausts) |
| Claude headless | `/scmorc` (per-task `claude -p` workers) | Anthropic quota | Audit-gate verdicts, judgment calls, escalations ONLY |
| Claude native | main session + subagents | Anthropic quota | Synthesis, design, final gates. Subagents at haiku/sonnet tier only -- NEVER Fable-tier fan-outs |

Known-broken path (do not retry): `claude --settings settings.local.OR.json`
(Claude Code cannot auth against OpenRouter directly; use a local shim like
claude-code-router if the full harness on free models is ever needed).

## 4. Tier routing (one table, all lanes)

Execution-plan tags map to models as follows:

| Plan tag | agy (Gemini) | Qwen (`--tier`) | Claude worker | Meaning |
|---|---|---|---|---|
| [HAIKU] | 3.5 Flash | `standard` or `flash` | haiku | Mechanical, fully specified, low blast radius |
| [SONNET] | 3.1 Pro (or 3.5 Flash if well-specified) | `max` | sonnet | Scoped implementation, spec exists |
| [OPUS+] | 3.1 Pro (spec draft only; judgment escalates) | `thinking` | fable/opus (main session) | Design, diagnosis, spec-writing |
| [AUDIT-GATE] | -- (not an agy job) | `thinking` (read-only verdict) | crypto-security-auditor subagent | Adversarial review, mandatory for crypto/transport/routing/privacy |

Lane preference for implementation work (operator-settled 2026-07-07,
fleet updated 2026-07-11): Qwen scripted dispatch FIRST (the `--verify`
auto-fix loop makes it self-correcting and it never collides with the
tree); agy-Gemini in parallel for small bounded agentic edits (it is a
separate free pool -- run pools concurrently, the tree and the single
Windows build slot are the only serialization points); OpenRouter as
spillover. Claude only for judgment, audits, and verdicts. agy's fleet
changes over time -- verify with `agy models`, update this table when it
shifts.

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
