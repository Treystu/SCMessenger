# Gemini Orchestrator Protocol
# SCMessenger Swarm

> SUPERSEDED 2026-07-20 -- this is NOT the single source of truth. Orchestration
> is now unified: the canonical protocol is `docs/ORCHESTRATION.md` (its Section 0
> Operating Contract and Section 2.2 loop bind every orchestrating model, Gemini
> included), launched via the one command `/orchestrate`. This file is retained
> only as a Gemini-specific runbook for the `lanes` backend
> (`scripts/delegate_task.py`); where it conflicts with ORCHESTRATION.md,
> ORCHESTRATION.md wins. Model names and quotas below are historical -- the live
> lake fleet is `SCM_UNIFIED_LAKE_ORCHESTRATION.md`.

This document is a Gemini-specific runbook for orchestrating the SCMessenger
backlog via the `lanes` backend. Read `docs/ORCHESTRATION.md` first, then use the
concrete delegate examples below.

---

## Your Role

You are the **FOREIGN WORKER / Orchestrator** (per `AGENTS.md`).
- NO `cargo`/`gradlew` build runs on your own
- NO implementation work — you write task files and delegate
- NO commits, pushes, or HANDOFF file moves done by you alone
- YES: Write task `.md` files, fire delegate commands, verify `cargo check`
  output, move HANDOFF files after build + tests pass

---

## Model Fleet Priority

| Priority | Provider | Model arg | When to use |
|---|---|---|---|
| 1a | Qwen | `qwen3-max` | Complex Rust implementation, crypto changes |
| 1b | Qwen | `qwen3.5-122b-a10b` | Compile fixes, mechanical refactors |
| 1c | Qwen | `qwen-plus-2025-07-28` | Doc rewrites, task file generation, audits |
| 1d | Qwen | `qwen3-vl-235b-a22b-thinking` | Architecture / security review tasks |
| 1e | Qwen | `qwen-max` | General fallback (995k remaining) |
| 2 | OpenRouter (free tier) | `nvidia/llama-3.1-nemotron-ultra-253b-v1:free` | Large-context or Qwen fallback |
| 3 | Ollama (local) | `qwen2.5:72b` | Scoped/small tasks when quota available |
| 4 | You (Gemini) | — | Orchestration only, zero LOC |

**Rotation rule**: Each task picks the best-fit model above. Never exhaust one model — spread across 1a-1e to preserve 5M+ total free tokens.

---

## Delegation Loop (repeat until backlog is empty)

### Step 1 — Pick the next task
```bash
cat HANDOFF/todo/_QUEUE.md
ls HANDOFF/todo/PQC_*.md
cat HANDOFF/todo/<NEXT_TASK>.md
```

### Step 2 — Inventory the relevant files
```bash
grep -rn "the_function_in_question" core/src --include="*.rs" | head -30
wc -c <file1> <file2>   # check they fit in 30k token window
```

### Step 3 — Delegate to Qwen
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/<TASK>.md \
  --provider qwen \
  --model qwen3-max \
  --files core/src/crypto/foo.rs core/src/iron_core.rs \
  --apply
```
Then **stop calling tools**. The system will wake you when it finishes.

### Step 4 — Verify build (after Qwen reports back)
```powershell
$env:CARGO_INCREMENTAL="0"; $env:PATH += ";C:\Users\SCM\.cargo\bin"
cargo check --workspace
```
Then run the relevant test suite:
```powershell
cargo test -p scmessenger-core --test integration_pq_session
```
Stop and wait — system will wake you.

### Step 5 — If build PASSES
```bash
# Move task to done
mv HANDOFF/todo/<TASK>.md HANDOFF/done/

# Update _QUEUE.md status to COMPLETE
# (edit the file, mark the row COMPLETE)
```
Then go to Step 1 for the next task.

### Step 6 — If build FAILS
Write a new task file at `HANDOFF/todo/<TASK>_COMPILE_FIX.md` with:
- The exact compiler errors (copy from cargo output)
- The function signatures from `pq/mod.rs` or wherever relevant
- Instruction to return the FULL corrected file

Then delegate again:
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/<TASK>_COMPILE_FIX.md \
  --provider qwen \
  --model qwen3-max \
  --files <affected files> \
  --apply
```

---

## Code Block Format Requirement (for task files)

Every task you write for delegation MUST instruct the model to format responses like this:

```
The exact filename must be the FIRST LINE inside the code block:
  // core/src/crypto/ratchet.rs
followed immediately by the full file content.
```

The `delegate_task.py` parser looks for `// path/to/file.rs` as line 1 of a
` ```rust ` block and writes `"\n".join(lines[1:])` to that path.

---

## Current Backlog (`HANDOFF/todo/_QUEUE.md`)

Check live state — the canonical source is `_QUEUE.md`.
As of 2026-07-11:

| Task | Status |
|---|---|
| PQC-01 through PQC-06 | DONE |
| PQC-07 PQ Ratchet Steps | DONE (5/5 tests pass) |
| PQC-08 Legacy Path Retirement | IN PROGRESS (Qwen) |
| PQC-09 through PQC-14 | TODO |

---

## Key Infrastructure Files

| File | Purpose |
|---|---|
| `scripts/delegate_task.py` | Universal swarm dispatcher — Qwen/OpenRouter/Ollama |
| `HANDOFF/todo/_QUEUE.md` | Live backlog and sequencing |
| `HANDOFF/todo/PQC_00_MASTER_PLAN.md` | Full PQC roadmap |
| `docs/ORCHESTRATION_PLAYBOOK.md` | Extended dispatch examples |
| `AGENTS.md` | Hard rules — read before anything |
| `GEMINI.md` | Points here |
| `core/src/crypto/ratchet.rs` | Double ratchet + PQ ratchet |
| `core/src/crypto/encrypt.rs` | Message encryption + fallback logic |
| `core/src/crypto/pq/mod.rs` | ML-KEM-768 primitives |
| `core/tests/integration_pq_session.rs` | PQ integration tests (5 tests) |

---

## Hard Rules (from `AGENTS.md`)

1. No emoji anywhere
2. No `cargo`/`gradlew` unless verifying — and ONLY after Qwen applies
3. No commits, pushes — report only
4. No temp files outside `tmp/`
5. Security-sensitive changes (`core/src/crypto/`) require adversarial review note
6. LOC estimates only — never time estimates
7. Escalate architecture decisions to the human operator

---

## OpenRouter Free Models (good choices)

```
nvidia/llama-3.1-nemotron-ultra-253b-v1:free   # Best reasoning
google/gemini-2.5-pro-exp-03-25:free           # Large context
qwen/qwen3-235b-a22b:free                      # Qwen on OR
```

Usage:
```bash
python scripts/delegate_task.py \
  --task HANDOFF/todo/<TASK>.md \
  --provider openrouter \
  --model nvidia/llama-3.1-nemotron-ultra-253b-v1:free \
  --files <files> \
  --apply
```
Set `OPENROUTER_API_KEY` env var first.

---

## Token Budget Rule

**You (Gemini) spend tokens ONLY on:**
- Reading task/error output to write the next delegate task file
- Writing `HANDOFF/todo/<task>.md` files
- Running `cargo check` / `cargo test` to verify
- Moving HANDOFF files and updating `_QUEUE.md`

**Everything else goes to Qwen.**
