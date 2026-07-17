# SCMessenger Orchestration Protocol

Status: Active. Last updated: 2026-07-17.

This is the canonical reference for all orchestration modes. Every command
in `.claude/commands/` is a specialization of this protocol. Any model,
running anywhere, can orchestrate the v1.0.0 farm build by reading this
document plus the shared state files in Section 2.

---

## 1. Lake Registry

All agent API lakes available to any orchestrator. Full registry with endpoints
and model lists: `SCM_UNIFIED_LAKE_ORCHESTRATION.md`.

| Lake        | Provider          | Best For                                              | Tiers              |
|-------------|-------------------|-------------------------------------------------------|--------------------|
| qwen        | DashScope/Alibaba | Rust/Kotlin implementation, deep CODER/THINK capacity | FLASH/CODER/THINK/MAX |
| groq        | Groq Cloud        | Fast FLASH/CODER micro-tasks; daily reset each 24h    | FLASH/CODER/THINK  |
| openrouter  | OpenRouter        | Morph Lite apply/verify; free model spillover         | FLASH/CODER/MORPH  |
| gemini      | Google AI Studio  | Large-context review, whole-file analysis             | FLASH/CODER/THINK  |
| ollama      | Local/cloud       | Zero-cost overflow; air-gap fallback                  | FLASH/CODER        |

Note: Groq free tier has a per-minute token cap (~12K TPM for most models).
Prompts exceeding ~8K tokens must be micro-chunked before dispatch. See
Section 2.1 for the dispatch ladder and Section 6 for the Groq chunk rule.

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
9. **Fusion Lite** (`scripts/fusion_lite.py --max-cost 0.001`): planning and
   verification second opinions only. WS-A delivery-logic triangulation: 3
   distinct Qwen verifier dispatches OR one Fusion Lite panel run. Never
   implementation. Never raise --max-cost without operator approval.
10. **Claude native**: [AUDIT-GATE] adversarial verdicts (fable), structural
    deadlocks, 2+ free-lane failures. Burns Anthropic subscription window.

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

## 5. Orchestrator Modes

| Mode               | Command                              | Primary Lake          | When To Use                          |
|--------------------|--------------------------------------|-----------------------|--------------------------------------|
| `/scmorc`          | `.claude/commands/scmorc.md`         | Claude native + free  | Claude subscription available        |
| `/scmqwen`         | `.claude/commands/scmqwen.md`        | Qwen DashScope        | Claude HARDLOCK, zero Anthropic cost |
| `/gemini-orchestrator` | `.claude/commands/gemini-orchestrator.md` | Gemini/agy + Qwen | agy.exe available as primary       |
| `/orchestrate`     | `.claude/commands/orchestrate.md`    | ollama-cloud swarm    | Swarm pool mode                      |
| `/scm`             | `.claude/commands/scm.md`            | Claude native agents  | Native Agent tool mode               |

All modes share the state files in Section 2. Swap mid-sprint with zero state
loss: the new orchestrator reads the JSONL queue + ledger + HANDOFF tree and
resumes exactly where the previous one stopped.

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
Follow `API_LIMIT_MANAGEMENT_PLAN.md` for per-lake exhaustion handling.

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
