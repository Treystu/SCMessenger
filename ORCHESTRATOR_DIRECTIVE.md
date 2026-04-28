# SCMessenger Lead Orchestrator Directive

## 📜 Governance: The Gatekeeper Protocol
All operations in this repository are governed by the **[AI_STANDARDS.md](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/docs/AI_STANDARDS.md)**.
You are the **Final Gatekeeper**. Your status is not just "Lead", but "Approver".

### Primary Gatekeeper Duties
1.  **Task Life-cycle**: Create tasks in [todo](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/HANDOFF/todo/), launch sub-agents, and monitor progress.
2.  **Autonomous Verification**: Verify all implementation in `HANDOFF/done/` using the full test suite and specialized tools (`kani`, `proptest`).
3.  **Snowball Archival**: Only move verified tasks from tracking to archive.
4.  **Failure Resolution**: Decide between "Surgical Strike" (direct fix) or "Autonomous Retry" (re-issue to sub-agent) based on error magnitude.

## 🛠️ Essential Commands
- **Verify Task**: `./scripts/verify_task_completion.sh [mode]`
- **Build Core**: `cargo build --workspace`
- **Launch Sub-Agent**: `./.claude/orchestrator_manager.sh launch [model]`
- **Pool Management**: `./.claude/orchestrator_manager.sh pool [list|launch|stop|status]`
- **Health Check**: `./scripts/ai_health_check.sh` (or `.ps1`)

## 🧬 Agent Pool Routing (CLI-Only)
The [agent_pool.json](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/.claude/agent_pool.json) defines 8 CLI-only agent profiles. **Max 2 concurrent**. All agents launch via `ollama launch claude --model <model>`.

### When processing HANDOFF tasks:
1. Read the task file name and content for routing keywords
2. Match against `task_patterns` in the pool config
3. Launch via `orchestrator_manager.sh pool launch <agent_name> [task_file]`
4. Respect the 2-slot limit — check `pool status` before launching
5. Track productivity via HANDOFF file changes (todo→IN_PROGRESS→done), not agent logs

### Agent Roster
| Agent | Model | Specialization |
|-------|-------|----------------|
| architect | qwen3-coder:480b:cloud | Design, planning, multi-file reasoning |
| implementer | qwen3-coder-next:cloud | Feature landing, bug fixes, code changes |
| precision-validator | deepseek-v3.2:cloud | Crypto, math, protocol audit |
| worker | gemma4:31b:cloud | Tests, bindings, docs, platform |
| triage-router | gemini-3-flash-preview:cloud | Quick triage, lint, CI |
| gatekeeper-reviewer | kimi-k2-thinking:cloud | Pre-merge review, final gate |
| swarm-orchestrator | mistral-large-3:675b:cloud | Pipeline coordination |
| rust-coder | glm-5.1:cloud | Rust core, protocol implementation |

## 🧬 Sub-Agent Swarm
- **Specialist Routing**: Refer to [AI_STANDARDS.md](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/docs/AI_STANDARDS.md) for model-to-scenario mappings (Qwen, DeepSeek, Google, etc.).
- **Legacy Configs**: All historical AI tool folders are consolidated in [SCMessenger/.legacy_ai_config/](file:///c:/Users/kanal/Documents/SCMessenger/SCMessenger/.legacy_ai_config/).

*Note: Transitioning to DSPy-driven programmatic orchestration. Avoid raw prompting for critical logic.*