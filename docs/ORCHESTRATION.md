# SCMessenger Orchestration Protocol

Status: Active. Last updated: 2026-07-20.

This is the single canonical reference for orchestration. There is now ONE
orchestrator command -- `/orchestrate` (`.claude/commands/orchestrate.md`) -- and
it is a thin launcher that points back here. Any model, running anywhere, can
drive the v1.0.0 farm build by reading this document plus the shared state files
in Section 2. The superseded per-backend commands (scmorc, scm, scmqwen,
gemini-orchestrator, swarm) are archived under `.claude/archive/commands/`; their
behaviour is preserved below as selectable BACKENDS (Section 5), not as separate
commands.

---

## 0. Operating Contract (read first -- applies to every orchestrator, every model)

These five rules are absolute. They are written plainly so that even a small,
tool-poor but instruction-following model can orchestrate correctly.

1. **DELEGATION IS MANDATORY. The orchestrator never writes application code.**
   Every implementation, fix, test, or analysis task is dispatched to a lake
   (Section 1). You may directly edit ONLY: HANDOFF task files (state moves), the
   backlog tracker, prompt files under `tmp/`, and a surgical 1-3 line compile fix
   that is the sole thing blocking a build gate. Anything larger -> delegate. If
   you are about to type code into a `.rs/.kt/.java/.swift/.ts` file, STOP and
   dispatch instead.

2. **The canonical dispatch path is a script, so ANY model can run it.**
   `python scripts/delegate_task.py --task <file> --provider <lake> [--model <m>]
   --files <targets> --apply --verify "<gate>" --mode diff --max-rounds 3`.
   The native `Agent` tool and `claude -p` workers are OPTIONAL accelerators
   available only when the orchestrator is Claude; they are never required. A
   non-Claude orchestrator uses the script for 100% of dispatches.

3. **You are the only writer of builds, commits, and state.** Workers implement
   and report; they never run `cargo`/`gradlew`, never commit, never move HANDOFF
   files. You run the gate, you move the ticket, you commit. One build at a time
   (Windows rlib-lock safety, Section 9).

4. **Follow the loop in Section 2.2 for every task**, in order: read queue ->
   validate -> pick lake (2.1 ladder) -> dispatch -> verify gate -> security gate
   if required -> move ticket -> commit -> record ledger. No step is optional.

5. **Record every dispatch in the ledger** (`tmp/lakes/ledger.jsonl` via
   `scripts/lake_route.py --record ...`). The router is blind to what you do not
   record; unrecorded dispatches burn lakes twice.

Escalate to the operator before: architecture-direction changes, security/privacy
trade-offs, tech-stack changes, API-contract breaks, or release/versioning
decisions.

---

## 1. Lake Registry

All agent API lakes available to any orchestrator. Full endpoint + model + quota
registry, the ranked free-tier and tokens/$ comparison, and the rotation strategy:
**`SCM_UNIFIED_LAKE_ORCHESTRATION.md`**.

### Active lakes (wired in `scripts/delegate_task.py` today -- valid `--provider` values)

| Lake        | Provider          | Best For                                              | Tiers              |
|-------------|-------------------|-------------------------------------------------------|--------------------|
| qwen        | DashScope/Alibaba | Rust/Kotlin implementation, deep CODER/THINK capacity | FLASH/CODER/THINK/MAX |
| groq        | Groq Cloud        | Fast FLASH micro-tasks; small TPM, micro-chunk        | FLASH/CODER        |
| openrouter  | OpenRouter        | Free-model spillover; 1,000 req/day (via $10 topup)   | FLASH/CODER        |
| gemini      | Google AI Studio  | Large-context review, whole-file analysis (key-gated) | FLASH/CODER/THINK  |
| ollama      | Ollama free tier  | Small overflow (a few tasks/week); air-gap fallback   | FLASH/CODER        |

### Candidate lakes (DOCUMENTED ONLY -- not yet wired; registry Section 6 has the exact add)

Do NOT pass these as `--provider` yet: `delegate_task.py` rejects any provider not
in its `choices` list, and each needs a `~/.config/scmorc/<lake>.env` key file
first. They are researched and ready to wire, nothing more.

| Lake        | Provider          | Best For                                              | Tiers              |
|-------------|-------------------|-------------------------------------------------------|--------------------|
| mistral     | Mistral (Plateforme+Codestral) | Best free code lake: 1B tok/mo, 500K TPM, Codestral | FLASH/CODER |
| nvidia      | NVIDIA NIM        | 100+ models (qwen3-coder, DeepSeek, GLM); no CC       | FLASH/CODER/THINK  |
| sambanova   | SambaNova Cloud   | Largest daily free budget; DeepSeek V3.2              | FLASH/CODER/THINK  |
| cerebras    | Cerebras          | Fastest inference; 8K free context -> mechanical only | FLASH              |
| modelscope  | Alibaba ModelScope| 2,000 calls/day free, separate from DashScope         | FLASH/CODER        |
| scaleway    | Scaleway (EU)     | qwen3-coder-30b, devstral; 1M free tokens             | FLASH/CODER        |
| deepseek    | DeepSeek (paid)   | Cheapest capable coder/$: V4 Flash, 98% cache discount | CODER/THINK       |

Note: full quotas, endpoints, key files, and the free vs paid tokens/$ comparison
live in `SCM_UNIFIED_LAKE_ORCHESTRATION.md`. Standing reality (2026-07-20):
Ollama Cloud Pro is NOT currently subscribed (purchase candidate); OpenRouter sits
at 1,000 req/day thanks to the one-time $10 lifetime topup. Groq's small per-minute
token cap means prompts over ~6K tokens must be micro-chunked (Section 6);
big-context lakes (qwen, mistral, nvidia, sambanova, gemini) do not.

---

## 2. Shared State Files

All orchestrators read and write these files. State lives in files, not in any
model's memory -- this is the unification property: any model can take over
orchestration by reading the queue and ledger.

| File                              | Purpose                                                  |
|-----------------------------------|----------------------------------------------------------|
| `HANDOFF/todo/_QUEUE.md`          | Live human-readable dispatch order                       |
| `scm_v1_farm_queue.jsonl`         | Machine-readable task queue (one JSON per line)          |
| `tmp/lakes/ledger.jsonl`          | Quota ledger -- append-only, one entry per dispatch      |
| `tmp/lakes/round_robin_state.json`| Per-lake per-tier model rotation counters                |
| `tmp/lakes/registry.json`         | Lake registry snapshot (seed from SCM_UNIFIED_LAKE_ORCHESTRATION.md) |
| `tmp/scmorc/dispatch_log.md`      | Human dispatch log (all orchestrators append here)       |

---

## 2.1 Cross-Lane Dispatch Ladder

For any task, try lanes in this order (first available with quota wins):

1. **Groq FLASH** (`delegate_task.py --provider groq --model llama-3.1-8b-instant`):
   mechanical tasks, docs, config. Fastest inference. Micro-chunk to <=6K
   tokens if prompt is large (see Section 6).
2. **Qwen FLASH** (`delegate_task.py --provider qwen --model qwen3-coder-flash`):
   mechanical tasks when Groq daily cap is hit.
3. **Groq CODER** (`delegate_task.py --provider groq --model qwen/qwen3-32b`):
   standard implementation on fresh daily window. Micro-chunk to <=6K tokens.
4. **Qwen CODER** (`delegate_task.py --provider qwen --model qwen3-coder-plus`):
   Rust/Kotlin implementation, 128K context, no size limit. Primary CODER lane.
5. **Gemini CODER** (`delegate_task.py --provider gemini --model gemini-2.5-flash`):
   large-context review, whole-file diffs. Secondary CODER lane. KEY-GATED:
   needs `~/.config/scmorc/gemini.env` (absent 2026-07-17; router skips it
   automatically -- the agy CLI sign-in does not cover this lane).
6. **OpenRouter CODER** (`delegate_task.py --provider openrouter --model deepseek/deepseek-chat-v3:free`):
   spillover when Qwen tiers saturate.
7. **Qwen THINK** (`delegate_task.py --provider qwen --model qwen3-235b-a22b-thinking-2507`):
   adversarial review, hard design, failed-CODER escalation.
8. **Gemini THINK** (`delegate_task.py --provider gemini --model gemini-2.5-pro`):
   large-context adversarial review. Same gemini.env key gate as lane 5.
9. **Fusion Lite** (`scripts/fusion_lite.py --max-cost 0.01`): planning,
   verification, and JUDGEMENT (Section 10). Caps: $0.01 default, $0.05 for
   hard problems (operator-settled 2026-07-17). Never implementation. Never
   raise caps without operator approval.
10. **Claude native**: [AUDIT-GATE] adversarial verdicts (fable), structural
    deadlocks, 2+ free-lane failures. Burns Anthropic subscription window.

---

## 2.2 The Orchestration Loop (run this for every task)

This is the whole job. It was previously duplicated across five command files;
it now lives here once. Follow it in order.

1. **READ QUEUE.** Open `HANDOFF/todo/_QUEUE.md`; take the top actionable ticket.
   Group consecutive tickets by domain (rust-core / android / wasm / desktop /
   docs) to reuse worker context.
2. **PRE-DISPATCH VALIDATION** (cheap, orchestrator-local -- never spend a worker
   on a dead task). Read the ticket, identify the concrete target (symbol/file),
   grep for it:
   - FALSE_POSITIVE (target is a test/Kani/proptest/`GOLDEN_*` literal) -> move to
     `HANDOFF/done/` with a note; next ticket.
   - ALREADY_WIRED (the thing to "wire" already has callers) -> move to done/; next.
   - NEEDS_REVIEW (target missing/ambiguous) -> STOP, ask the operator.
   - VALID -> continue.
3. **WRITE the worker prompt** to `tmp/<slug>.prompt.md`: self-contained --
   requirement, exact target file paths, acceptance criteria, and the exact
   build-gate command. Include the Worker Contract header (Section 3).
4. **PICK THE LAKE** by the Section 2.1 ladder and the tier the task needs: FLASH
   for mechanical, CODER for implementation, THINK/MAX for analysis and
   adversarial review. Never send analysis or judgement to a FLASH lake (Section
   9.13). Free lanes first, always.
5. **DISPATCH** (canonical, any model): `scripts/delegate_task.py --task <file>
   --provider <lake> --files <targets> --apply --verify "<gate>" --mode diff
   --max-rounds 3`. Claude-only accelerators, if available: the `Agent` tool,
   `claude -p` workers, or the ollama pool via `orchestrator_manager.sh` (Section
   5). Always `--mode diff` (Section 9.3).
6. **VERIFY.** Parse the worker's first line (RESULT/PATCH/VERDICT, Section 3).
   `git diff --stat` scoped to the ticket:
   - ZERO-DIFF -> do not trust it; ticket stays in todo/, log `requeued`.
   - Real diff -> run the matching gate YOURSELF (Rust `cargo check --workspace`;
     Android `cd android && ./gradlew assembleDebug -x lint --quiet`; WASM
     `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`;
     `CARGO_INCREMENTAL=0` on Windows). Grep the diff for
     `simulate|mock|placeholder|in a real implementation` -- a clean compile is
     NOT completion (Section 9.1).
7. **SECURITY GATE** (Section 4). Diff touches `core/src/{crypto,transport,routing,
   privacy}/` -> mandatory adversarial review at THINK/MAX tier before commit.
   Delivery-logic diffs (outbox, receipt, custody, retry) -> triangulate: 3
   distinct verifier dispatches or one Fusion Lite panel (Section 10).
8. **MARK COMPLETE.** Real diff + passing gate (+ security pass where required) ->
   move the ticket to `HANDOFF/done/`, update the tracker. A task is not complete
   until the file has moved.
9. **COMMIT.** `git add -A && git commit -m "<prov>: completed <task>"` (provenance:
   `native:` for Claude-worker completions, `swarm:` for foreign/pool completions).
   Record the gate result in the message. Never push unless the operator asks.
10. **RECORD** the dispatch in the ledger (`lake_route.py --record`), re-check
    quota/cooldowns, and return to step 1. Stop when the queue is empty, a
    NEEDS_REVIEW/escalation is hit, or the operator interrupts.

---

## 3. Worker Contract

Every worker response MUST begin with one of:
```
RESULT: DONE
RESULT: BLOCKED: <reason>
RESULT: FAILED: <reason>
PATCH: <number-of-files>
VERDICT: PASS|FAIL|NEEDS_INFO|ANALYSIS_COMPLETE
```

Then max 10 lines: what changed, files touched, anything the verifier must
know before running gates.

Workers NEVER: run builds (`cargo`, `gradlew`), commit, push, or move HANDOFF
files. The orchestrator owns ALL of those operations.

---

## 4. Security Gates (mandatory -- no exceptions)

| Trigger                                               | Gate Required                                           |
|-------------------------------------------------------|---------------------------------------------------------|
| Any diff in `core/src/{crypto,transport,routing,privacy}/` | Adversarial review (THINK or MAX tier) before commit |
| Any WS-A delivery logic diff (outbox, receipt, custody, retry) | Fusion Lite 3-panel ($0.001 ceiling) OR 3 distinct Qwen verifier dispatches |
| E-01c dispatch                                        | E-01b must carry adversarial PASS on file first        |
| PQC-11/PQC-13 dispatch                                | E-01 (full chain) must be landed first                 |
| PQC-09 dispatch                                       | E-01 landed AND explicit AD-8 operator lift            |

---

## 5. Backends (HOW you dispatch -- not separate commands)

There is one command: `/orchestrate`. A "backend" is only the mechanism that
carries a given dispatch. Pick per task and mix freely within one run. All
backends share the Section 2 state files, so you can switch backend mid-sprint
with zero state loss. The old per-backend commands are archived under
`.claude/archive/commands/` and map onto this table.

| Backend | Invocation | Runs on | Use when | (archived command) |
|---------|------------|---------|----------|--------------------|
| Script lane (CANONICAL) | `scripts/delegate_task.py --provider <lake>` | Any free/paid API lake (Section 1) | Default for ~100% of tasks; the only path a non-Claude orchestrator needs | scmqwen, gemini-orchestrator |
| Native Claude worker | `claude -p ... --model <alias> --effort <lvl>` (background Bash) | Anthropic subscription window | AUDIT-GATE adversarial verdicts (fable), or a task with 2+ free-lane failures | scmorc |
| Native Agent subagent | `Agent` tool (`rust-implementer` / `android-qa` / `crypto-security-auditor` / `docs-sync-auditor` / `release-gatekeeper`) | Anthropic subscription window | Claude orchestrator wants isolated-context delegation without spawning a CLI | scm |
| Ollama pool (micro-swarm) | `orchestrator_manager.sh pool launch <agent> <task>` | Ollama free tier (small: a few tasks/week) + any cloud pool | Batch fan-out across pooled agents | orchestrate (old), swarm |

Rules that bind every backend: Free lanes first, always -- a native Claude
worker is the last resort, not the default (it burns the Anthropic window; the
Quota Governor tiers from the archived `scmorc` apply when you use that backend).
The DELEGATION-IS-MANDATORY rule (Section 0) holds identically no matter which
backend or which model is orchestrating.

---

## 6. Groq Micro-Chunking Rule

Groq free tier enforces ~12K tokens-per-minute. Any prompt exceeding 6K tokens
MUST be split before dispatch:

1. Identify the context-heavy section (usually embedded source code).
2. Split into <=6K-token chunks, each self-contained (repeat task header).
3. Dispatch chunk 1, receive response, then dispatch chunk 2 with the prior
   response inlined as context if needed.
4. Orchestrator merges partial patches before applying.

Use `scripts/lake_route.py --tier FLASH --probe-groq` to confirm current
Groq TPM headroom before a large dispatch.

---

## 7. State Machine

```
HANDOFF/todo/<ID>_*.md
  -> HANDOFF/IN_PROGRESS/<ID>_<lake>_<ts>.md   (when dispatched)
  -> HANDOFF/review/<ID>_evidence.md            (when gate evidence recorded)
  -> HANDOFF/done/<ID>_*.md                     (when all gates pass)
```

Every state transition requires the gate evidence named in the task packet.
Zero-diff worker responses are re-queued, not marked done.

---

## 8. Session Continuity

State is file-backed; resumption requires only: this document, the JSONL
queue, the ledger, and the HANDOFF tree. No model memory is required.
Follow `API_LIMIT_MANAGEMENT_PLAN.md` and the routing/ledger sections of
`SCM_UNIFIED_LAKE_ORCHESTRATION.md` (Section 3) for per-lake exhaustion and
cooldown handling.

---

## 9. Lessons: 2026-07-17 Swarm Post-Mortem (READ before any batch dispatch)

Each rule below was paid for in a bad commit or a burned quota window.
Commits 71d02d4d/e298e9bf ("swarm: completed remaining queue") were reverted
by 23960b35/8da8cc90 after audit; do not repeat their failure modes.

1. **Compile-only verify is NOT a completion gate.** The reverted run's
   "passing" C-06 diff was 212 lines of simulated/mock dead code that
   compiled cleanly. After ANY exit-0 verify, grep the applied diff for
   `simulate|mock|placeholder|in a real implementation` before accepting,
   and give it an orchestrator quality pass.
2. **Know the delegate_task.py exit codes:** 0 = verified (still needs
   rule-1 quality pass), 2 = verify failed after all fix rounds, 3 =
   VACUOUS success (model returned no applicable file blocks -- treat as
   FAILED, never as done).
3. **Always dispatch with `--mode diff`.** Without it, flash-tier models
   emit prose summaries instead of applicable file blocks, producing
   vacuous successes (observed on E-02/E-04/D-05/D-01 in the reverted run).
4. **Platform-correct verify commands.** gradlew lives in `android\`, not
   the repo root (`gradlew.bat assembleDebug` from root fails with
   "Task 'assembleDebug' not found"). iOS targets CANNOT be verified on
   Windows -- xcodebuild does not exist here. Mark iOS packets
   BLOCKED-PLATFORM and route them to a macOS runner (H-01); never let a
   batch runner "fail" them against a nonexistent toolchain.
5. **One build at a time on Windows.** Never run two concurrent
   `delegate_task.py --verify` jobs (2 concurrent cargo/gradle builds risk
   rlib lock corruption; see .claude/rules/build.md). `run_tasks.ps1` v2 is
   strictly sequential for this reason.
6. **Batch runners NEVER auto-commit and NEVER move tickets.** Workers
   implement; the orchestrator reviews (adversarial gate for
   `core/src/{crypto,transport,routing,privacy}/`), moves tickets, and
   commits. `run_tasks.ps1` v2 writes `tmp/swarm_report.md` only.
7. **Hallucinated Target Files are real.** On D-03 the file-deducer emitted
   three nonexistent `SCMessengerTests/*.swift` paths, which would have
   become the worker's write allowlist. `scripts/deduce_files.py` now drops
   any emitted path not present in `git ls-files`. If a packet has no
   Target Files, re-run `scripts/fix_targets.py` before dispatch.
8. **Qwen non-stream 400** (`parameter.enable_thinking must be set to false
   for non-streaming calls`): fixed in delegate_task.py (all non-streaming
   DashScope calls send `enable_thinking=false`). If you see this error you
   are running an old script -- pull.
9. **Feed the ledger or the router goes blind.** After EVERY dispatch:
   `python scripts/lake_route.py --record --lake <lake> --model <model>
   --task <id> --result ok|429|403|413|error|timeout|vacuous`. The router
   skips lakes with no key file and honors cooldowns automatically -- but
   only knows what you record.
10. **Lane smoke results, 2026-07-17** (re-probe at sprint start):
    - LIVE: groq `llama-3.1-8b-instant`; qwen `qwen3-coder-flash`;
      ollama `gpt-oss:20b-cloud`; openrouter `morph/morph-v3-fast` (paid,
      routes fine).
    - DOWN: openrouter `:free` tiers (429 shared-pool saturation -- retry
      off-peak); ollama `qwen3.5:397b-cloud` (403 auth); gemini lane needs
      `GEMINI_API_KEY`/`GOOGLE_API_KEY` in `~/.config/scmorc/gemini.env`
      (the agy CLI's own sign-in does NOT cover delegate_task.py).
11. **Morph Lite** is for single-file surgical edits only (three lane bugs
    fixed 2026-07-17; see HANDOFF/MORPH_LITE_HANDOFF.md). **Fusion Lite** is
    planning triangulation only, on the spend-capped key at
    `~/.config/scmorc/openrouter_fusion.env`.
12. **`enable_thinking` must follow the model name.** DashScope non-thinking
    hybrids require `enable_thinking=false` for non-streaming; thinking models
    (qwen3-*-thinking-*) REQUIRE `true` (400 "restricted to True" otherwise).
    delegate_task.py now sets it from the model name. Symptom history: THINK
    dispatch 400'd and silently rotated down to a FLASH model -- a masked tier
    downgrade. A rotation that DOWNGRADES tier on an analysis/judgement task
    is a FAILED dispatch: fix the root cause, re-dispatch at the right tier.
13. **FLASH tier cannot do analysis.** On the E-00 pre-flight, a flash model
    ignored an explicit read-only instruction, emitted code blocks, and
    guessed constants instead of citing file:line evidence. Analysis and
    judgement dispatches: THINK tier minimum, never FLASH/CODER-flash.
14. **OpenRouter budgets (operator 2026-07-17):** `openrouter.env` =
    FREE-MODELS-ONLY (delegate_task.py refuses non-`:free` models);
    `openrouter_fusion.env` = shared paid Fusion+Morph key ($0.50 cap).
    Proven costs: Morph call $0.00086; Fusion 3-panel+judge $0.0013.
15. **OpenCode native agent map** (`.opencode/`): GLM-5.2 `orchestrator`
    (primary), kimi-k2.7-code `implementer`, deepseek-v4-flash explore +
    small_model, glm-5.1 general. Config loads at startup only -- RESTART
    opencode to activate; verify model IDs resolve (`opencode-go/<id>`).

---

## 10. Fusion Judgement Protocol (operator-settled 2026-07-17)

Judgement is DELEGATED, not done natively by the orchestrator. Acceptance of
any analysis, design, or non-trivial implementation diff requires a Fusion
Lite panel verdict of UNANIMOUS PASS. Anything less: re-iterate -- fix or
re-dispatch with the panel's dissent inlined, then re-judge.

1. **Panel:** 3 models, 70B+ class, different vendors. Proven set:
   `qwen/qwen3-235b-a22b-2507,deepseek/deepseek-chat-v3.1,meta-llama/llama-3.3-70b-instruct`,
   judge `qwen/qwen3-235b-a22b-2507`. Never 8B-class for design judgement.
2. **Command:**
   `python scripts/fusion_lite.py --prompt-file tmp/<item>.md --panel "<3>" --judge "<j>" --max-tokens 1000 --max-cost 0.05 --out tmp/<item>-verdict.md`
   (--max-tokens >=800 for design questions; 500 truncates 70B panelists).
3. **Unanimity rule:** every panelist must independently endorse AND the
   judge synthesis must record no unresolved dissent. One dissent = re-iterate.
4. **What gets judged:** pre-implementation analysis (PASS before writing the
   implementation packet), implementation diffs on gated paths (this is the
   adversarial-review gate for core/src/{crypto,transport,routing,privacy}/
   when the panel is given an adversarial prompt: probe for races, desync,
   downgrade, framing-compat, DoS), and any acceptance the orchestrator is
   unsure about. Compile-only verify is still NOT completion (Section 9.1).
5. **Record** every judgement in the ledger (`--record --lake openrouter
   --model fusion-panel-3x70b`) and cite the verdict file in the commit.
