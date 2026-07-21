# Morph Lite Design Specification

**Version:** 1.0 | **Status:** ACTIVE | **Last Updated:** 2026-07-14

## Overview

**Morph Lite** is a lightweight code-transformation function using **Morph V3 Fast** (OpenRouter) for scoped, cost-capped verification and implementation of single-file code changes. Designed to complement the orchestrator's delivery-logic triangulation workflow (farm WS-A, custody/retry/receipt edge cases).

**Key Claims (Morph V3 Fast, OpenRouter):**
- ~10,500 tokens/sec throughput
- 96% accuracy on code transformations
- Optimized for rapid, deterministic edits
- 81,920 token context (sufficient for 500-line files + instruction + diff)
- $0.80/M input, $1.20/M output (cheap per-token vs. Claude)

---

## 2 Core Keys (Scope & Cost)

### Key 1: Scope Limit (Single-File, < 500 Lines)

| Constraint | Value | Rationale |
|---|---|---|
| **Files per call** | 1 | Keeps transformation deterministic; multi-file = multi-call |
| **Lines per file** | ≤ 500 | Stays within fast-apply window; complex edits → escalate |
| **Instruction length** | ≤ 1000 chars | Forces concise, unambiguous change specs |
| **Edit snippet length** | ≤ 2000 chars | Snippet describes the delta, not full replacement |
| **Use case** | Single logical change | One function fix, one constant, one test, one config block |

**Out of Scope:**
- Refactors spanning 3+ files
- Rewrites of 200+ line functions
- Architectural changes (use Sonnet/Opus workers instead)
- Adversarial security review (use Fable instead)

### Key 2: Cost Ceiling ($0.001 per call, hard-capped)

| Item | Value | Reasoning |
|---|---|---|
| **Hard ceiling** | $0.001 per invocation | ~1,250 tokens at Morph rates |
| **Never raised** | Policy enforced in code | Operator approval required to change ceiling |
| **Cost per token** | ~0.80 µ input, 1.20 µ output | Morph is 8-10x cheaper than Claude |
| **Usage window** | Per-task dispatch, no accumulation | Reset after each file transformation |
| **Tracking** | Logged per invocation in dispatch logs | Transparent cost visibility |

**Cost math:**
- Input: 500 lines × 50 tokens/line ≈ 25k tokens possible
- Output: 500 lines rewritten ≈ 25k tokens
- Max cost: (25k × 0.80 + 25k × 1.20) / 1M = **$0.050** — but our $0.001 ceiling auto-gates before API call completes

---

## Usage Patterns

### Pattern 1: Verify Delivery-Logic Changes (farm WS-A)

**Scenario:** Orchestrator made a fix to outbox flush-on-connect. Before commit, check if the change is sound.

```bash
python morph_lite.py \
  --file core/src/store/outbox.rs \
  --instruction "Verify flush_on_connect() calls persist_msg() for all peers" \
  --edit-snippet "fn flush_on_connect(&mut self) { for peer in self.peers() { self.persist_msg(peer)?; } }" \
  --verify-only
```

Expected output: Code suggestion; orchestrator inspects, applies manually, or rejects.

### Pattern 2: Apply Scoped Fix

**Scenario:** Typo in Android constant, fix approved by review.

```bash
OPENROUTER_API_KEY=<key> python morph_lite.py \
  --file android/app/src/main/AndroidManifest.xml \
  --instruction "Change PERMISSION_BLE_CONNECT from 'android.permission.BLE_ADMIN' to 'android.permission.BLUETOOTH_CONNECT'" \
  --edit-snippet '<uses-permission android:name="android.permission.BLUETOOTH_CONNECT" />' \
  --max-cost 0.0005
```

Expected output: File updated in-place; exit code 0.

### Pattern 3: Reconcile Merge Conflict

**Scenario:** Android and core both touched `MeshService.kt`; Morph auto-reconciles.

```bash
python morph_lite.py \
  --file android/app/src/main/kotlin/com/scmessenger/MeshService.kt \
  --instruction "Keep new BLE interrupt handler; merge with existing WiFi Aware startup" \
  --edit-snippet "// See line 120 for interrupt, line 85 for startup" \
  --verify-only
```

---

## Integration with Orchestrators

### Lane Placement (ORCHESTRATION.md Section 2.1)

Morph Lite sits **between free lanes and Claude workers:**

```
[1] Qwen/Gemini (free, no cost) — implement most features
[2] MORPH LITE (cheap verify/apply) ← verification triangulation gate
[3] fusion_lite (paid, multi-model) — consensus on hard cases
[4] Claude workers (Sonnet/Opus) — complex architecture
```

### When to Dispatch Morph Lite

**GOOD (within scope):**
-  Verify a 1-file fix before commit
-  Apply a small snippet (< 100 lines changed)
-  Reconcile a simple merge conflict
-  Reformat a config block

**BAD (escalate instead):**
-  Multi-file refactor → Qwen [THINK] tier
-  Architectural redesign → Claude Opus
-  Security audit → Claude Fable
-  Complex protocol change → Adversarial review gate

### Dispatch Mechanics

#### Via `/scmorc` (Claude native)

```bash
python scripts/morph_lite.py \
  --file <target> \
  --instruction <change-desc> \
  --edit-snippet <snippet> \
  [--verify-only] \
  --max-cost 0.001
```

Log result to `tmp/scmorc/dispatch_log.md`:
```
[2026-07-14 14:30] morph-lite verify core/src/store/outbox.rs result=done cost=$0.00045
```

#### Via `/scmqwen` (Qwen free tier)

Morph Lite is NOT a Qwen dispatch — it's OpenRouter-direct. But orchestrator can route a task to Morph Lite if:
1. Task file says "MORPH_LITE candidate"
2. Orchestrator pre-validates scope (< 500 lines)
3. Calls morph_lite.py directly instead of qwen.sh

#### Farm WS-A Triangulation (Delivery Logic)

Per `FARM_FINAL_PLAN.md` Section 5:

**Requirement:** Before committing changes to outbox/receipt/retry/custody, **triangulate** via:
- **Option A:** 1 Morph Lite call (fast, cheap) + manual review
- **Option B:** 3 distinct Qwen [THINK] verifiers (free, redundant)
- **Option C:** 1 fusion_lite panel (paid, multi-model consensus)

Orchestrator picks one, logs evidence to dated farm ledger doc, then commits.

---

## Configuration & Constraints

### Environment Variables

```bash
export OPENROUTER_API_KEY="sk-or-v1-..."
```

### Defaults (Coded, Non-Negotiable)

```python
MAX_FILE_LINES = 500              # Single file max
MAX_COST_PER_CALL = 0.001         # $0.001 hard ceiling
INSTRUCTION_MAX_CHARS = 1000      # Concise specs only
EDIT_SNIPPET_MAX_CHARS = 2000     # Bounded snippets
```

### CLI Options

```
--file <path>              File to transform (required)
--instruction <text>       Change description (required)
--edit-snippet <text>      Edit snippet (required)
--verify-only              Dry run; don't write file
--max-cost <usd>          Cost limit (default: $0.001)
```

---

## Success Criteria & Gates

### Gate 1: Config Validation (Pre-API)

Checked before OpenRouter call:
- File exists and < 500 lines
- Instruction ≤ 1000 chars
- Snippet ≤ 2000 chars
- Cost cap ≤ $0.001
- API key set

**Fail:** Exit code 1, message logged to stderr.

### Gate 2: API Response Parsing

- Valid JSON from OpenRouter
- No `error` field in response
- `choices[0].message.content` present
- Tokens + cost calculation sound

**Fail:** Exit code 2, API error logged.

### Gate 3: Cost Gate (Post-API, Hard Enforcement)

- Calculated cost ≤ `--max-cost` (default $0.001)
- If exceeded, **reject transformation**, do not apply

**Fail:** Exit code 1, cost and limit logged.

### Output Validation (Optional, Post-Apply)

Orchestrator MAY verify:
- File parses (Rust: `cargo check --lib`, Kotlin: `./gradlew :app:compileDebugKotlin`)
- No new compile errors
- Changed lines match intent
- No code injection or hallucination

---

## Logging & Observability

### Console Output

```
[RESULT] PASS | cost=$0.00045 | tokens_in=523 tokens_out=412
[OK] Applied to core/src/store/outbox.rs
```

or

```
[RESULT] FAIL | cost=$0.0012 | tokens_in=8192 tokens_out=0
[ERROR] Cost $0.0012 exceeds limit $0.001
```

### Dispatch Log (Orchestrator-Maintained)

```markdown
[2026-07-14 14:30] morph-lite verify core/src/store/outbox.rs result=done cost=$0.00045
[2026-07-14 14:35] morph-lite apply android/.../AndroidManifest.xml result=done cost=$0.00038
[2026-07-14 15:02] morph-lite verify core/src/crypto/session.rs result=fail error="cost exceeded" cost=$0.0015
```

---

## Troubleshooting

### "Cost exceeds limit"

1. Check token prediction: large files + verbose instructions = higher cost
2. Reduce edit snippet verbosity; Morph is smart about context
3. For complex changes, escalate to Qwen [THINK] or Claude instead
4. If cost is acceptable, raise `--max-cost` with operator approval (rare)

### "File too large (X lines, max 500)"

1. Extract the function/block being changed into a separate temp file
2. Apply change to temp file via Morph Lite
3. Orchestrator merges back into full file
4. Or: escalate to multi-file refactor (use Qwen/Claude)

### "API timeout or no response"

1. Check OPENROUTER_API_KEY is valid
2. Verify OpenRouter service status: `https://status.openrouter.ai`
3. Retry once automatically; if still fails, escalate to Qwen

### "Morph returned syntax error"

1. Check instruction is unambiguous (typos, unclear delta descriptions)
2. Verify edit snippet is valid code (not pseudocode)
3. If pattern is novel, Qwen [THINK] may be better; Morph is deterministic but may fail on novel refactors
4. Log to `MORPH_LITE_FAILURES.md` for pattern analysis

---

## Security & Audit

### Threat Model

| Threat | Mitigation |
|---|---|
| **Cost attack** (unbounded API usage) | Hard $0.001 ceiling enforced in code; API call fails before token exhaustion |
| **Code injection** (Morph returns malware) | All output reviewed by orchestrator; no auto-apply in security paths |
| **API key leak** (in logs) | Never log API key; cost only, no credentials in dispatch log |
| **Out-of-scope edits** | Pre-validation gates; max 500 lines = bounded context |

### Audit Trail

Every invocation:
1. Logged to dispatch log with timestamp, file, result, cost
2. Delta stored in `git diff` (orchestrator commits after verification)
3. If failed, error message and tokens logged (no code returned)

---

## Future Extensions (Post-v1)

- [ ] Multi-model panel (Morph + Sonnet) for confidence voting
- [ ] Adaptive cost ceiling per task class (P0 tasks get $0.002 budget)
- [ ] Integration with farm WS-A ledger for auto-triangulation
- [ ] Streaming output for large diffs
- [ ] Caching of identical instructions (cost savings)

---

## References

- **Morph docs:** https://docs.morphllm.com/quickstart
- **OpenRouter pricing:** https://openrouter.ai/morph/morph-v3-fast
- **Orchestration ladder:** `docs/ORCHESTRATION.md` Section 2.1
- **Farm plan:** `HANDOFF/plans/FARM_FINAL_PLAN.md`
- **Dispatch config:** `.claude/commands/orchestrate.md` (unified) + `docs/ORCHESTRATION.md`; the old `scmorc`/`scmqwen` are archived under `.claude/archive/commands/` as the `native`/`lanes` backends
