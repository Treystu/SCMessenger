# Qwen Model Quota Ledger (DashScope)

Status: Active
Last updated: 2026-08-03

This document tracks the verified DashScope/Alibaba Qwen models, their enabling status, actions, and remaining free quota.

## DISPATCH POLICY: one dedicated model per task. NO fallback chains.

Chains were being built like qwq-plus -> qwen3-32b -> qwen3-30b-a3b -> qwen-max.
That is a REASONING model falling back to a 3B-active MoE on the SAME task, so a
failure landed on something that could not do the work and returned confident
garbage instead of an honest failure. Several bad lane outputs today trace to it.

**Pick ONE model matched to the task. If it fails, REPORT and re-dispatch
deliberately.** Fallback only between models of the SAME tier.

| Tier | Models | Use for |
|---|---|---|
| Reasoning | `qwq-plus`, `qwen3-30b-a3b-thinking-2507` | root cause, lock tracing, adversarial review |
| Large general | `qwen3-32b`, `qwen-max`, `qwen3-235b-a22b` | design, planning, code review |
| Coder | `qwen3-coder-plus-2025-07-22` | code written to a precise spec |
| Mid mechanical | `qwen3-14b`, `qwen3-30b-a3b`, `qwen3.5-flash` | inventories, structured extraction |
| Small/fast | `qwen3-8b`, `qwen-turbo`, `qwen-plus-2025-*` | counting, formatting, greps |

A task with a fully specified METHOD is MECHANICAL -- mid or small tier. A task
whose ANSWER IS UNKNOWN needs reasoning tier. Putting `qwq-plus` on a branch
inventory wasted scarce reasoning budget on work a 14b could do.

Thinking models need an OUTPUT CAP ("under 60 lines") plus "write a partial
answer first", or they spend the budget reasoning and never write the file.

## TASK SIZE: what each tier can actually carry

Operator correction 2026-08-03: a 14b is for MICRO tasks ONLY. Sizing by "it is
mechanical" is not enough -- mechanical tasks vary enormously in scope.

| Size | Definition | Tier |
|---|---|---|
| MICRO | ONE step. A single grep, a count, extract one value, reformat a list. No branching, no verdict. | `qwen3-8b`, `qwen3-14b`, `qwen-turbo` |
| SMALL | A few steps, fixed method, no judgement calls. | `qwen3.5-flash`, `qwen3.6-27b` |
| MEDIUM | Multi-step with conditionals, OR any task returning a VERDICT, OR one that must judge whether evidence is sufficient. | `qwen3-32b`, `qwen3-30b-a3b`, `qwen-max` |
| LARGE | Cross-file reasoning, design, reviewing someone else's work. | `qwen-max`, `qwen3-235b-a22b` |
| REASONING | Answer unknown, must be inferred. Deadlocks, root cause, adversarial analysis. | `qwq-plus`, `qwen3-30b-a3b-thinking-2507` |

**The concrete mistake this records:** an Android gate check -- run five adb
commands, parse dumpsys, decide PASS/FAIL/INCONCLUSIVE per check, emit a GATE
verdict -- was dispatched to `qwen3-14b`. That is MEDIUM, not MICRO. It
branches, it judges evidence sufficiency, and it returns a verdict.

Rule of thumb: if the task description contains "decide", "classify", "verify",
or "if X then Y", it is NOT micro regardless of how mechanical the steps look.
Anything that emits a verdict is at least MEDIUM.

## LANE FALLBACK: when the right tier is exhausted, change LANE not tier

| Lane | Access | Best at |
|---|---|---|
| Qwen (Alibaba MaaS) | `claude --model <id>` + `.claude/alibaba_cloud_config.env` | PRIMARY -- full toolset: shell, edits, git |
| Fusion Lite | `scripts/fusion_lite.py --panel --judge` | panel+judge on ONE hard question. 2c normal / 10c hard |
| Groq | `delegate_task.py --provider groq` | fast micro. Tight TPM, needs curl UA |
| OpenRouter free | `delegate_task.py --provider openrouter` | general text/code |
| Ollama Cloud | `delegate_task.py --provider ollama` | `gpt-oss:120b`, verified reachable |
| DashScope | `~/.config/scmorc/dashscope.env` | separate Qwen pool from MaaS |
| Claude subagent | `Agent` tool, `model: haiku` | repo-aware structured analysis |

Routing: reasoning -> Qwen reasoning, else Fusion Lite, else Ollama 120b.
Mechanical -> Qwen mid/small, else Groq, else OpenRouter. Code -> Qwen coder,
else OpenRouter.

### NEVER delegated

1. **Deterministic computation.** Branch classification, diff arithmetic, log
   counting. A model asked to classify 55 branches returned 36 MERGE verdicts
   including branches its own data showed would delete 12,933 lines from main.
   If the answer is derivable, derive it with a script.
2. **Final verdicts** on security or merge-readiness. Lanes analyse; the
   decision stays native and is hand-verified.

## THE DASH RULE -- read this before building any fallback chain (2026-08-03)

**A model whose Free Quota column shows `-` has NO free allowance and will 403
on every call, regardless of what any other list says.** This is the single most
expensive lesson of the 2026-08-03 session: roughly a dozen dispatches were
burned retrying dash-models while ~50 models with 750K-1M remaining sat unused,
and the conclusion "Qwen is exhausted" was reported to the operator. It was
wrong. The operator corrected it.

**The pattern: BARE ALIASES have no quota; DATED PINS do.**

| No quota (`-`) -- do NOT dispatch | Has quota -- use these |
|---|---|
| `qwen-flash`, `qwen3-coder-flash`, `qwen3-max`, `qwen3-max-preview`, `qwen3-coder-plus`, `qwen-plus`, `qwen-plus-latest`, `qwen3.6-flash`, `qwen3.7-plus`, `deepseek-v4-pro`, `deepseek-v4-flash`, `deepseek-v3.2`, `glm-5.2`, `kimi-k2.7-code`, `qwen3-coder-next`, `qwen3-30b-a3b-instruct-2507`, `qwen3-next-80b-a3b-instruct` | `qwen3.7-flash-2026-07-15`, `qwen3.6-flash-2026-04-16`, `qwen3.5-flash`, `qwen3.5-35b-a3b`, `qwen3.6-27b`, `qwen3.5-27b`, `qwen3-30b-a3b-thinking-2507`, `qwen3-next-80b-a3b-thinking`, `qwen3-32b`, `qwen3-14b`, `qwen3-8b`, `glm-5.1`, `qwen3.5-397b-a17b`, `qwen3.5-plus-2026-02-15`, `qwen3.7-max-2026-06-08`, `qwen3-max-2025-09-23`, `qwen-max`, `qwen3-235b-a22b-thinking-2507` |

Note the trap: the bare alias and its dated pin are DIFFERENT quota pools.
`qwen3.6-flash` is dead but `qwen3.6-flash-2026-04-16` has 999,829.
`qwen3-coder-flash` is dead but `qwen3-14b` has 881,528.

**Picking a model by task:**
- mechanical wiring / edits -> `qwen3.7-flash-2026-07-15`, `qwen3.6-flash-2026-04-16`
- planning / prose -> `glm-5.1` (verified: wrote a 408-line plan 2026-08-03)
- hard analysis, deadlock/lock tracing -> a *thinking* model:
  `qwen3-next-80b-a3b-thinking`, `qwen3-30b-a3b-thinking-2507`,
  `qwen3-235b-a22b-thinking-2507`

Build every fallback chain from the right-hand column only, and re-check the
console table when a chain starts 403ing rather than concluding the lane is dry.

## Empirical Liveness Probe (2026-08-03)

**The console quota table below does NOT predict whether a dispatch will
succeed.** Probed directly against the API with a real ~4 KB code-analysis
payload: `qwen3-32b` and `qwen3-coder-plus` both show "Remaining 1,000,000"
in the table below and both returned nothing. Trust this section over the
table, and re-probe before starting any campaign.

A trivial "reply OK" smoke test is also not sufficient evidence -- but note the
failure mode is NOT payload size. `qwen3.6-35b-a3b` and `qwen3-30b-a3b` failed
on both small and large prompts; they were simply exhausted, and an earlier
smoke test that appeared to pass had been misread. Probe with a real payload
because it is a truer test, not because size is the discriminator.

ALIVE (verified with real payload, 2026-08-03):

| Model | Notes |
|---|---|
| qwen3-coder-flash | coder-tuned, cheap -- preferred for file audits |
| qwen3-coder-flash-2025-07-28 | dated pin of the above |
| qwen3-coder-next | coder-tuned |
| qwen3-30b-a3b-instruct-2507 | general instruct |
| qwen3-next-80b-a3b-instruct | larger, use only when a small model stalls |
| deepseek-v4-flash | general |

EXHAUSTED / NON-RESPONSIVE (2026-08-03): `qwen3.6-35b-a3b`, `qwen3-30b-a3b`,
`qwen3-8b`, `qwen3-32b`, `qwen-turbo`, `qwen-plus-latest`, `qwen3-coder-plus`,
plus the previously recorded `qwen3.7-plus-2026-05-26`,
`qwen3-coder-30b-a3b-instruct`, `deepseek-v4-pro`, `deepseek-v3.2`, `glm-5.2`.

Probe command (one model, real payload):

```bash
set -a && source <(grep -E '^[A-Z_]+=' .claude/alibaba_cloud_config.env | sed 's/[[:space:]]*$//') && set +a
timeout 75 claude --model <id> --dangerously-skip-permissions \
  -p "In one sentence, what does this Rust code do? $(head -c 4000 core/src/transport/addr_filter.rs)"
```

Reminder: a model whose quota column shows a dash (`-`) rather than a numeric
allowance has NO free allowance at all -- do not dispatch to it.

## Quota Summary (as of 2026-07-10, console-reported -- see liveness probe above)

| Model Code | Free Quota Remaining | Expiration Date | Status | Actions |
|---|---|---|---|---|
| qwen-vl-ocr-2025-11-20 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.5-122b-a10b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-235b-a22b-thinking | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-32b-thinking | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-plus-2025-07-28 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-max | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.5-plus-2026-02-15 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-max | Remaining 995,152 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-mt-flash | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-30b-a3b-thinking | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-235b-a22b-thinking-2507 | Remaining 964,386 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.7-max-2026-06-08 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| glm-5.1 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.7-max-preview | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-32b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| glm-5.2 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| kimi-k2.7-code | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.5-397b-a17b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.6-flash | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-plus-2025-09-23 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-vl-plus | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| deepseek-v3.2 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-coder-next | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.5-flash | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-32b-instruct | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.5-35b-a3b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| deepseek-v4-flash | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-30b-a3b-thinking-2507 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-coder-plus-2025-09-23 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-plus-latest | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-coder-480b-a35b-instruct | Remaining 991,110 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-8b-thinking | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-coder-plus | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-plus-2025-09-11 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| wan2.2-kf2v-flash | Remaining 50 / Total 50 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-flash-2026-01-22 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.5-flash-2026-02-23 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-max-preview | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-flash-2025-10-15 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-vl-max | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.7-plus-2026-05-26 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-30b-a3b-instruct | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-235b-a22b-instruct | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-8b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-coder-30b-a3b-instruct | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.6-27b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-235b-a22b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-plus | Remaining 834,956 / Total 1,000,000 | 2026/08/07 | Enabled | Stop-on-Exhaust |
| qwen-turbo | Remaining 998,681 / Total 1,000,000 | 2026/08/07 | Enabled | Stop-on-Exhaust |
| qwen-mt-lite | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.6-flash-2026-04-16 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-coder-flash | Remaining 999,981 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qvq-max | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-plus | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-next-80b-a3b-thinking | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.5-27b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.7-max-2026-05-17 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-30b-a3b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-mt-plus | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-flash | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-14b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-8b-instruct | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-max-2025-09-23 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-plus-character | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| deepseek-v4-pro | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-coder-flash-2025-07-28 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-flash-character | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-vl-plus-2025-12-19 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-plus-2025-04-28 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-mt-turbo | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-30b-a3b-instruct-2507 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-flash | Remaining 999,978 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-flash-2025-07-28 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.6-35b-a3b | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-plus-2025-07-14 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-235b-a22b-instruct-2507 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwq-plus | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.6-plus-2026-04-02 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-coder-plus-2025-07-22 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3.5-plus-2026-04-20 | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen-vl-ocr | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |
| qwen3-next-80b-a3b-instruct | Remaining 1,000,000 / Total 1,000,000 | 2026/10/06 | Enabled | Stop-on-Exhaust |

## Unsupported or Inactive Models

| Model Code | Status | Notes |
|---|---|---|
| qwen3.7-plus | Not Supported | - |
| qwen3.7-max | Not Supported | - |
| qwen3.6-plus | Not Supported | - |
| qwen3.7-max-2026-05-20 | Not Supported | - |
| qwen3.5-plus | Not Supported | Use specific version tags instead |
| qwen3.6-max-preview | Not Supported | - |
| qwen3-max-2026-01-23 | Not Supported | - |
| qwen-plus-character-ja | Not Supported / No Free Quota | - |
| qwen-plus-2025-01-25 | Not Supported / No Free Quota | - |
| glm-5.2-fast-preview | Not Supported / No Free Quota | - |
