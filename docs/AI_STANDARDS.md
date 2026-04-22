# SCMessenger AI Standards & Governance (v2.0)

This document serves as the **Single Source of Truth** for all AI agents (Orchestrators, Swarms, and Sub-agents).

## 🦾 Role: The Lead Orchestrator as Gatekeeper

### Lead Orchestrator (Claude Code)
- **Mission**: Accountable for system health, security, and multi-platform velocity. (The **Gatekeeper**).
- **Sub-Agent Slots**: Manage up to **2 concurrent sub-agent slots** (3 total agents in the ecosystem).
- **Authority**: Decides when to spawn sub-agents or trigger the Python Swarm.

1.  **Task Creation**: Defines the scope, assigns models, and launches sub-agents.
2.  **Autonomous Gatekeeping**: Before archiving any task from `HANDOFF/done/`, the Orchestrator MUST:
    - Run the full project-specific verification suite (`verify_task_completion.sh`).
    - Audit the implementation for "Surgical Integrity" (no unnecessary drift).
    - If logic bugs are found:
        - **Minor (Syntax/Clarity)**: Orchestrator fixes it directly.
        - **Major (Algorithmic/Security)**: Orchestrator re-issues the task with detailed feedback to a precision model.

## 🧬 Swarm Model Topology (Empirical Mappings)

To maximize efficiency, agents are dynamically routed to specific models based on task complexity:

| Scenario / Role | Primary Model | Strength |
|-----------------|---------------|----------|
| **Lead Architect** | `qwen3-coder:480b:cloud` | Multi-file reasoning, Rust core architecture (99 Coding). |
| **Primary Developer** | `qwen3-coder-next:cloud` | Rapid feature landing (95 Coding, 80 Speed). |
| **Precision Validator** | `deepseek-v3.2:cloud` | Cryptography, math, and protocol validation (98 Precision). |
| **Generalist/Worker** | `gemma4:31b:cloud` | Unit tests, bindings, and platform engineering (85 Speed). |
| **Log Triage / Router** | `gemini-3-flash-preview:cloud` | Parsing audit summaries and routing tasks (95 Speed). |
| **Reviewer (Final Gate)** | `kimi-k2-thinking:cloud` | Pre-merge verification (98 Precision / 95 Coding). |
| **Swarm Orchestrator** | `mistral-large-3:675b:cloud` | Pipeline management (95 Agentic). |

## 🔧 CLI Agent Pool (All Agents via Ollama Launch)

The agent pool (`.claude/agent_pool.json`) defines 8 CLI-only agent profiles with a 2-concurrent slot limit. All agents launch via `ollama launch claude --model <model>` — no native Agent tool invocations.

### Agent Roster (All CLI)

| Agent | Model | Use When | task_patterns |
|-------|-------|----------|---------------|
| **architect** | `qwen3-coder:480b:cloud` | Design, planning, multi-file reasoning | ARCHITECTURE, PLAN, DESIGN, REFACTOR |
| **implementer** | `qwen3-coder-next:cloud` | Feature landing, bug fixes, code changes | IMPLEMENTATION, BUG, FIX, FEATURE, WASM, IOS, ANDROID |
| **precision-validator** | `deepseek-v3.2:cloud` | Crypto audit, protocol review, unsafe verification | SECURITY, CRYPTO, AUDIT, VERIFY, REVIEW, GATE |
| **worker** | `gemma4:31b:cloud` | UniFFI bindings, unit tests, docs, platform wiring | PLATFORM, BINDINGS, TEST, DOCS |
| **triage-router** | `gemini-3-flash-preview:cloud` | Quick triage, lint, minor edits, CI gatekeeping | LINT, TRIAGE, QUICK, CI |
| **gatekeeper-reviewer** | `kimi-k2-thinking:cloud` | Pre-merge review, final verification gate | MERGE, RELEASE, FINAL_REVIEW |
| **swarm-orchestrator** | `mistral-large-3:675b:cloud` | Pipeline management, multi-agent coordination | ORCHESTRATE, PIPELINE, SWARM |
| **rust-coder** | `qwen3-coder-next:cloud` | Rust core, protocol implementation | RUST, CORE, PROTOCOL, CRYPTO_IMPL |

### Model Routing Source of Truth
**`agent_pool.json`** is the single source of truth for model assignments. The table below is for reference only — if it conflicts with `agent_pool.json`, the pool config wins.

### Task Routing Patterns
Task file keywords auto-route to agent types:
- ARCHITECTURE/PLAN/DESIGN → architect
- IMPLEMENTATION/BUG/FIX/FEATURE → implementer
- SECURITY/CRYPTO/AUDIT/VERIFY → precision-validator
- PLATFORM/BINDINGS/TEST/DOCS → worker
- LINT/TRIAGE/QUICK/CI → triage-router
- MERGE/RELEASE/FINAL_REVIEW → gatekeeper-reviewer
- ORCHESTRATE/PIPELINE/SWARM → swarm-orchestrator
- RUST/CORE/PROTOCOL → rust-coder

### Launch Command
```bash
./.claude/orchestrator_manager.sh pool launch <agent_name> [task_file]
```

### Concurrency Rules
- Max 2 concurrent CLI agents (enforced by orchestrator)
- Track productivity via HANDOFF file changes, not agent logs
- Always check `pool status` before launching
- All agents have fallback_model: `glm-5.1:cloud`

## 🧪 Verification & Determinism

### Formal Verification
- All cryptography in `core/src/crypto/` and routing logic in `core/src/transport/` MUST be verified using **Property-Based Testing** (`proptest`) and **Formal Verification** (`kani`).
- **Standard**: Zero "unsafe" blocks permitted without a `// SAFETY:` doc comment and Kani verification passing.

### Programmatic Optimization (DSPy)
- Swarm orchestration is transitioning from raw prompt-based loops to **deterministic programmatic frameworks**. Use the `docs/DSP_SCM_PLAN.md` for guidance on structuring multi-agent pings as optimized functions.

## 🚦 System Integration

- **Legacy Archival**: All non-standard AI configurations reside in [SCMessenger/.legacy_ai_config/](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/.legacy_ai_config/).
- **Snowball Rule**: Tasks are archived from [REMAINING_WORK_TRACKING.md](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/REMAINING_WORK_TRACKING.md) strictly AFTER Gatekeeper verification.