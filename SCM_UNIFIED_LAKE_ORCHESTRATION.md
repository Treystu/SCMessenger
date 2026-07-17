# SCMessenger Unified Lake Orchestration — Setup for Agentic v1.0.0 Completion

**Purpose:** any model, running anywhere, can orchestrate the v1.0.0 farm build by dispatching micro-tasks to any available "agent API lake" (free-tier capacity pools), with quota-aware routing and a single state machine.

**Existing infrastructure this builds on** (already in repo, verified readable):
- `scripts/delegate_task.py` — multi-provider dispatch: **qwen** (DashScope), **openrouter**, **ollama**, **groq** (OpenAI-compatible endpoints, env-file key loading from `~/.config/scmorc/<provider>.env`)
- `.claude/commands/scmqwen.md` — proven orchestrator contract: tier roster, round-robin state, build serialization, escalation ladder
- `HANDOFF/MORPH_LITE_HANDOFF.md` — Morph V3 Fast lane via OpenRouter ($0.001/call ceiling) for single-file <500-line edits
- `ORCHESTRATOR_DIRECTIVE.md` — gatekeeper protocol + agent pool
- Queue: `scm_v1_farm_queue.jsonl` (machine) + `SCM_V1_FARM_BUILD_MASTER_BACKLOG.md` (human)

---

## 1. Lake registry

Quota numbers are **runtime-learned state, not hardcoded truth** — free tiers change without notice. The router records observed 429s/resets in the ledger (§4) and treats the table below as seed priors only. Verify each lake's current limits in its console before a sprint.

```json
{
  "lakes": {
    "qwen": {
      "endpoint": "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
      "key_env": ["QWEN_API_KEY", "DASHSCOPE_API_KEY"],
      "key_file": "~/.config/scmorc/dashscope.env",
      "quota_type": "trial_tokens_per_model",
      "quota_seed": "~1M tokens/model, 90-day rolling window (operator-verified 2026-07-10; 130+ models)",
      "tiers": {
        "FLASH": ["qwen3-coder-flash", "qwen3.5-flash"],
        "CODER": ["qwen3-coder-plus", "qwen3-coder-plus-2025-09-23", "qwen3-coder-plus-2025-07-22"],
        "THINK": ["qwen3-235b-a22b-thinking-2507", "qwen3.5-122b-a10b"],
        "MAX":   ["qwen3-max", "qwen3-max-preview"]
      },
      "notes": "Deepest free roster. One depleted model never blocks a tier — rotate."
    },
    "groq": {
      "endpoint": "https://api.groq.com/openai/v1/chat/completions",
      "key_env": ["GROQ_API_KEY"],
      "key_file": "~/.config/scmorc/groq.env",
      "quota_type": "daily_tokens_and_requests",
      "quota_seed": "free tier, per-model daily + per-minute caps; learn exact values from 429 headers at runtime",
      "tiers": {
        "FLASH": ["llama-3.1-8b-instant"],
        "CODER": ["qwen/qwen3-32b", "llama-3.3-70b-versatile"],
        "THINK": ["deepseek-r1-distill-llama-70b"]
      },
      "notes": "Fastest inference in the farm. Ideal for FLASH/CODER micro-task throughput during its daily window; resets every 24h so it is the default first-lane each morning. delegate_task.py already sets a browser UA (Cloudflare 403 workaround)."
    },
    "openrouter": {
      "endpoint": "https://openrouter.ai/api/v1/chat/completions",
      "key_env": ["OPENROUTER_API_KEY"],
      "key_file": "~/.config/scmorc/openrouter.env",
      "quota_type": "credits + free_model_daily_caps",
      "quota_seed": ":free model variants have daily request caps; Morph V3 Fast lane hard-capped at $0.001/call per MORPH_LITE_HANDOFF",
      "tiers": {
        "FLASH": ["meta-llama/llama-3.3-70b-instruct:free", "qwen/qwen3-coder:free"],
        "CODER": ["qwen/qwen3-coder:free", "deepseek/deepseek-chat-v3:free"],
        "THINK": ["deepseek/deepseek-r1:free"],
        "MORPH": ["morph/morph-v3-fast"]
      },
      "notes": "Single key = many models; best failover lake. MORPH tier only for single-file <500-line apply/verify."
    },
    "gemini": {
      "endpoint": "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
      "key_env": ["GEMINI_API_KEY", "GOOGLE_API_KEY"],
      "key_file": "~/.config/scmorc/gemini.env",
      "quota_type": "daily_requests_per_model",
      "quota_seed": "AI Studio free tier, per-model daily request caps; learn from 429s",
      "tiers": {
        "FLASH": ["gemini-2.0-flash-lite"],
        "CODER": ["gemini-2.5-flash"],
        "THINK": ["gemini-2.5-pro"]
      },
      "notes": "Large context windows make it the review/audit lake (whole-file diffs + packet in one shot). Add provider to delegate_task.py per §6.2."
    },
    "ollama": {
      "endpoint": "http://localhost:11434/api/chat",
      "key_env": [],
      "quota_type": "none_local",
      "quota_seed": "unlimited local; cloud variants via ollama launch per ORCHESTRATOR_DIRECTIVE roster",
      "tiers": {
        "FLASH": ["gemma3:4b", "qwen3:8b"],
        "CODER": ["qwen3-coder:30b"],
        "THINK": ["deepseek-r1:32b"]
      },
      "notes": "Zero-cost overflow lane when all cloud lakes are capped; also the air-gap fallback. Throughput-limited by host GPU."
    },
    "mimo": {
      "endpoint": "per .mimocode/MIMO_API_SWITCH.md",
      "key_file": "per .mimocode config",
      "quota_type": "per-provider",
      "tiers": { "FLASH": ["default"], "CODER": ["default"] },
      "notes": "Existing MiMo-code lane; keep as configured, register here so the router can count it."
    }
  },
  "optional_lakes": ["cerebras (free tier, fast llama)", "sambanova (free tier)", "mistral la plateforme (free tier)", "github models (free tier, GITHUB_TOKEN)"],
  "rules": [
    "Register every key in ~/.config/scmorc/<lake>.env — never in the repo.",
    "A lake with no key file is skipped silently by the router.",
    "New lakes join by adding one JSON block; no router code changes (OpenAI-compatible endpoints only)."
  ]
}
```

---

## 2. Unified orchestrator contract (lake-agnostic — any model can orchestrate)

This is the single document pasted to whatever model is the orchestrator this session (qwen-max today, groq llama tomorrow, a local 8B next week). It replaces per-provider orchestrator commands.

```
ROLE: You are the SCM v1.0.0 farm orchestrator. You coordinate; you never code.

STATE MACHINE (authoritative, file-backed):
  HANDOFF/todo/<ID>_*.md  ->  HANDOFF/IN_PROGRESS/<ID>_<lake>_<ts>.md
  -> HANDOFF/review/<ID>_evidence.md  ->  HANDOFF/done/<ID>_*.md
  Every transition requires the gate evidence named in the packet.

LOOP (each wake cycle):
  1. Read scm_v1_farm_queue.jsonl; pick the highest-priority id whose
     depends[] are all in HANDOFF/done/ and whose files[] do not overlap
     any IN_PROGRESS packet.
  2. Check lane budget: one IN_PROGRESS per lane (android / ios / core /
     infra); serialize host builds (never two cargo/gradle at once —
     check running processes first).
  3. Route per §3: tier -> first lake with quota -> model rotation.
  4. Dispatch: send the packet + worker template (§5) via
     scripts/delegate_task.py --provider <lake> --model <model> ...
  5. On return: verify claimed files only, run gates yourself, then
     transition state and append to the ledger (§4).
  6. On worker failure: retry same packet on the NEXT lake in the
     failover ladder. Two failures -> escalate tier. Structural deadlock
     (2 failed escalations) -> write ESCALATION file, park the id, move on.

HARD RULES:
  - Never edit source files yourself (1-3 line compile-error unblocks excepted).
  - crypto/, privacy/, transport/ diffs always route REVIEW per packet
    (crypto-security-auditor or adversarial, THINK+ tier).
  - Escalate to the human operator before: architecture-direction changes,
    security/privacy trade-offs, API-contract breaks, release decisions,
    and the H-03 sign-off items.
  - No emojis (repo rule, hook-enforced).
  - E-01c may only be dispatched after E-01b carries an adversarial PASS.
```

---

## 3. Routing: quota-aware rotation + failover

**Tier → lake preference ladder** (first lake with remaining quota wins):

| Tier | Ladder |
|---|---|
| FLASH | groq → qwen → openrouter → gemini → ollama |
| CODER | qwen → groq → openrouter → gemini → ollama |
| THINK | qwen → gemini → openrouter (deepseek-r1:free) → groq |
| MAX | qwen (qwen3-max) → gemini (2.5-pro) → openrouter paid ceiling-guarded |
| MORPH (apply/verify single-file) | openrouter morph-v3-fast only, $0.001 cap |

**Within a lake:** per-tier round-robin over the model list (state in `tmp/lakes/round_robin_state.json`, same mechanic as the proven scmqwen rotation). On 429/timeout: mark model `cooldown_until`, advance rotation, never retry the same model twice in one dispatch.

**Daily rhythm:** groq + gemini daily windows reset every 24 h → front-load FLASH/CODER micro-tasks there each morning; qwen's 90-day trial budget is the strategic reserve for CODER/THINK; ollama absorbs overflow at zero marginal cost; MAX dispatches are rare by design (E-01b, deadlocks, final verdicts).

**Quota ledger** (`tmp/lakes/ledger.jsonl`, append-only — extends `API_EFFICIENCY_LEDGER.md`):
```json
{"ts":"2026-07-17T08:00Z","lake":"groq","model":"llama-3.3-70b-versatile","task":"A-01","in_tokens":6120,"out_tokens":1480,"result":"ok"}
{"ts":"2026-07-17T08:11Z","lake":"groq","model":"qwen/qwen3-32b","task":"A-03","error":"429","cooldown_until":"2026-07-18T00:00Z"}
```
Router reads the ledger before every dispatch; `cooldown_until` and daily-window math come from observed 429s, so the farm self-calibrates as tiers change.

---

## 4. Session continuity

Follow `API_LIMIT_MANAGEMENT_PLAN.md` (survives, readable): on any lake exhaustion, state is already file-backed, so resumption = re-read queue + ledger. Orchestrator handoff between *different models* needs only: this document, the JSONL queue, the ledger, and the HANDOFF tree. That is the unification property: **orchestration state lives in files, not in any model's memory.**

---

## 5. Worker prompt template (small-model optimized)

```
You are worker <lake>/<model>. Implement exactly one packet.

PACKET: <full packet from Z-02: goal, scope files w/ line anchors, ≤200
context lines, numbered steps, acceptance, gates, rollback>

RULES:
1. Touch only SCOPE FILES. If a step forces a new file, stop and say BLOCKED: <reason>.
2. Emit complete files or unified diffs only, each fenced block starting
   with its repo-relative path as the first-line comment
   (delegate_task.py extracts these automatically).
3. No commentary outside blocks except a final SUMMARY (3 lines max).
4. If context is insufficient, do not guess: reply INSUFFICIENT: <what is needed>.
5. No emojis.
```

Failure vocabulary is deliberate: `BLOCKED` / `INSUFFICIENT` route the packet back to the orchestrator for re-spec instead of producing plausible garbage — the small-model failure mode this system is designed around.

---

## 6. Setup checklist

### 6.1 Keys (5 min per lake)
```bash
mkdir -p ~/.config/scmorc
echo "DASHSCOPE_API_KEY=sk-..."   > ~/.config/scmorc/dashscope.env   # qwen trial
echo "GROQ_API_KEY=gsk_..."      > ~/.config/scmorc/groq.env        # daily free
echo "OPENROUTER_API_KEY=sk-or-v1-..." > ~/.config/scmorc/openrouter.env
echo "GEMINI_API_KEY=AIza..."    > ~/.config/scmorc/gemini.env
chmod 600 ~/.config/scmorc/*.env
```

### 6.2 Add gemini provider to `scripts/delegate_task.py` (~15 LoC)
- `GEMINI_URL = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"`
- key resolution mirroring the groq branch (`gemini.env`, `GEMINI_API_KEY`/`GOOGLE_API_KEY`)
- add `"gemini"` to `--provider` choices and the `req_url` map. No other changes — the script is already provider-generic.

### 6.3 Lake router wrapper (`scripts/lake_route.py`, ~150 LoC)
Reads `lakes.json` (§1) + `tmp/lakes/ledger.jsonl` + `tmp/lakes/round_robin_state.json`; given `--tier`, prints `provider model` for the first non-cooled-down candidate; on worker exit, appends the ledger record and sets cooldowns from 429s. Keeps `delegate_task.py` as the transport; this is routing policy only.

### 6.4 Smoke test per lake (one packet each)
Dispatch Z-01-class mechanical packet to every registered lake; confirm ledger rows + correct file-block extraction. Farm is live when every keyed lake has one `ok` row.

### 6.5 Ignition order
1. Z-01 → Z-03 (FLASH, any lake — queue rebuild, unblocks everything)
2. D-01 → D-04 (farm infra) in parallel with A-01/A-02 (CODER)
3. E-01a constraint sheet (THINK) early — it is the long pole; run it while waves A/D churn
4. Steady state: 1 packet per lane per dispatch cycle, ledger after every call, daily-window rhythm per §3

---

## 7. What "optimized for unification" means here

- **Same packet** feeds a 3B local model and a 235B cloud model — scope discipline comes from the packet, not the model.
- **Same state machine** regardless of which model orchestrates — files are the memory.
- **Same ledger** regardless of which lake served — quotas are learned, failover is automatic.
- **Same gates** regardless of who wrote the code — orchestrator always verifies, workers never self-certify.
- Any lake, any orchestrator, any worker can be swapped mid-sprint with zero state loss. That is the property the v1.0.0 farm build depends on, and it is satisfied by construction above.
