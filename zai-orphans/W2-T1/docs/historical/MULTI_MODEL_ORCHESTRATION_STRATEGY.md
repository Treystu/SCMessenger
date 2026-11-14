# Multi-Model Orchestration Strategy for SCMessenger

**Purpose:** Define how the ollama cloud model pool is routed across task types to maximize capability while minimizing token cost and latency.
**Source:** Derived from Claude Code's tiered model routing architecture (see `CLAUDE_CODE_ARCHITECTURE_RESEARCH.md`).
**Last updated:** 2026-04-28

## Tier Architecture

Claude Code's leaked architecture reveals three operational tiers that map directly to our ollama cloud pool. Rather than routing all tasks through the most expensive model, the system selects models based on the cognitive demands of each operation.

### Tier 1: Flagship Reasoning (High-Cost, High-Capability)

Used for: Architecture decisions, deep analysis, protocol correctness proofs, complex multi-step reasoning.

| Model | Params | Specialty | Primary Role |
|-------|--------|-----------|--------------|
| `glm-5.1:cloud` | 1.5T | General reasoning, proven Rust output | Rust core architect |
| `deepseek-v4-pro:cloud` | 1.6T | Deep analysis, correctness proofs | Deep analyst |
| `kimi-k2-thinking:cloud` | 1.1T | Extended thinking, code review | Gatekeeper reviewer |
| `kimi-k2:1t:cloud` | 1.1T | Kimi base, complex analysis | Senior analyst |
| `kimi-k2.5:cloud` | 1.1T | Kimi K2.5, review quality | Architecture reviewer |
| `kimi-k2.6:cloud` | 595B | Latest Kimi | General flagship |
| `qwen3-coder:480b:cloud` | 510B | Specialized coding | System architect |
| `mistral-large-3:675b:cloud` | 682B | Pipeline coordination | Swarm orchestrator |

**Token budget:** Allocate 40-60% of total session tokens here. These models handle the hardest problems.

### Tier 2: Primary Workhorse (Balanced Cost/Capability)

Used for: Feature implementation, bug fixes, routine coding, protocol implementation.

| Model | Params | Specialty | Primary Role |
|-------|--------|-----------|--------------|
| `qwen3-coder-next:cloud` | 81B | Next-gen coding, implementation | Primary implementer |
| `deepseek-v3.2:cloud` | 688B | Crypto/math/protocol | Precision validator |
| `deepseek-v3.1:671b:cloud` | 688B | Previous DeepSeek flagship | Orchestration |
| `cogito-2.1:671b:cloud` | 688B | Deliberative reasoning | Tradeoff evaluator |
| `qwen3.5:397b:cloud` | 397B | General reasoning, strong fallback | General fallback |
| `glm-5:cloud` | 756B | GLM 5 base, Rust compatible | Rust alternate |
| `minimax-m2.7:cloud` | 480B | General reasoning, implementation | General implementer |
| `nemotron-3-super:cloud` | 230B | Validation, robustness | Robustness validator |
| `qwen3-next:80b:cloud` | 81B | Next-gen generalist 80B | Alternate implementer |
| `qwen3-vl:235b:cloud` | 470B | Vision-language | Screenshot/UI analyst |
| `qwen3-vl:235b-instruct:cloud` | 470B | Vision-language instruct | Guided vision |

**Token budget:** 25-40% of session tokens. These handle the bulk of implementation work.

### Tier 3: Fast/Lightweight (Low-Cost, High-Speed)

Used for: Quick triage, lint, CI gatekeeping, documentation, bindings, simple tasks.

| Model | Params | Specialty | Primary Role |
|-------|--------|-----------|--------------|
| `deepseek-v4-flash:cloud` | 140B | Fast reasoning | Fast analyst |
| `gemma4:31b:cloud` | 31B | Lightweight worker, tests/docs | Worker bee |
| `gemini-3-flash-preview:cloud` | variable | Quick triage, lint, CI | Triage router |
| `devstral-2:123b:cloud` | 128B | Developer tooling | Dev tooler |
| `devstral-small-2:24b:cloud` | 51B | Lightweight dev | Light dev |
| `glm-4.7:cloud` | 696B | Previous GLM | Compat tester |
| `glm-4.6:cloud` | 696B | Earlier GLM | Baseline tester |
| `minimax-m2.5:cloud` | 230B | MiniMax v2.5 | Fallback implementer |
| `minimax-m2.1:cloud` | 230B | MiniMax v2.1 | Legacy implementer |
| `minimax-m2:cloud` | 230B | MiniMax base | Base implementer |
| `nemotron-3-nano:30b:cloud` | 32B | Lightweight validation | Nano validator |
| `gpt-oss:120b:cloud` | 65B | Open-source GPT | OSS coder |
| `gpt-oss:20b:cloud` | 13B | Lightweight GPT | Quick gen |
| `gemma3:27b:cloud` | 55B | Gemma 27B | Gemma worker |
| `gemma3:12b:cloud` | 24B | Gemma 12B | Light test runner |
| `gemma3:4b:cloud` | 8.6B | Gemma 4B | Micro task runner |
| `ministral-3:14b:cloud` | 15.7B | Minimal Mistral | Simple parse |
| `ministral-3:8b:cloud` | 10.4B | Minimal Mistral | Trivial parse |
| `ministral-3:3b:cloud` | 4.7B | Nano | Sanity check |
| `rnj-1:8b:cloud` | 16B | Lightweight | Simple task |

**Token budget:** 10-20% of session tokens. These handle high-volume, low-complexity work.

## Agent-to-Model Routing Matrix

| Agent Role | Primary Model | Fallback | Task Patterns |
|------------|--------------|----------|---------------|
| architect | `qwen3-coder:480b:cloud` | `qwen3.5:397b:cloud` | ARCHITECTURE, PLAN, DESIGN, REFACTOR |
| implementer | `qwen3-coder-next:cloud` | `glm-5.1:cloud` | IMPLEMENTATION, BUG, FIX, FEATURE |
| precision-validator | `deepseek-v3.2:cloud` | `deepseek-v4-pro:cloud` | SECURITY, CRYPTO, AUDIT, VERIFY |
| worker | `devstral-2:123b:cloud` | `gemma4:31b:cloud` | PLATFORM, BINDINGS, TEST, DOCS |
| triage-router | `gemini-3-flash-preview:cloud` | `deepseek-v4-flash:cloud` | LINT, TRIAGE, QUICK, CI |
| gatekeeper-reviewer | `kimi-k2-thinking:cloud` | `kimi-k2.6:cloud` | MERGE, RELEASE, FINAL_REVIEW |
| swarm-orchestrator | `mistral-large-3:675b:cloud` | `cogito-2.1:671b:cloud` | ORCHESTRATE, PIPELINE, SWARM |
| rust-coder | `glm-5.1:cloud` | `qwen3-coder-next:cloud` | RUST, CORE, PROTOCOL, CRYPTO_IMPL |
| CLIBetaTester | `qwen3-coder:480b:cloud` | `glm-5.1:cloud` | BETA, CLI, CORE_TEST, STRESS |
| deep-analyst | `deepseek-v4-pro:cloud` | `deepseek-v3.2:cloud` | RCA, ANALYSIS, PROOF, CORRECTNESS |
| vision-analyst | `qwen3-vl:235b:cloud` | `nemotron-3-super:cloud` | VISION, SCREENSHOT, UI_REVIEW |

## Model Availability Verification

Before launching any agent, verify model availability:

```bash
# Quick check via WebFetch
# URL: https://olloma.com/api/tags
# Expected: JSON with "models" array

# Script-based check
bash .claude/model_validation_template.sh
```

The `settings.local.json` allowlist includes `WebFetch(domain:ollama.com)` and `WebFetch(url:https://ollama.com/api/tags)` for on-demand verification.

## Fallback Strategy

If a primary model is unavailable:
1. Use the designated fallback from the routing matrix
2. If fallback is also unavailable, use next-best capability match from `model_capability_mapping.json`
3. If all tier-matched models are unavailable, escalate to human operator
4. Never silently downgrade from Tier 1 to Tier 3 — log the downgrade and flag for review
