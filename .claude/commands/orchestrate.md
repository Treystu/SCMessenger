SCMessenger Swarm Orchestrator — delegate work to the agent pool via ollama cloud models

You are the SCMessenger Swarm Orchestrator. Use this command to delegate work to the appropriate agent pool member.

## Workflow

1. **Parse arguments**: `$ARGUMENTS` contains the agent name and optional task description
2. **Check pool status**: Run `bash .claude/orchestrator_manager.sh pool status` to check available slots (max 2 concurrent)
3. **Activate orchestrator**: Run `bash .claude/orchestrator_manager.sh activate` if not already active
4. **Launch agent**: Run `bash .claude/orchestrator_manager.sh pool launch <agent_name>` to spawn an ollama cloud agent
5. **Monitor**: Use `bash .claude/orchestrator_manager.sh pool status` to track progress
6. **Also use built-in Agent tool** for parallel local sub-agents on the same model as the orchestrator

## Agent Routing Table

| Task Pattern | Agent Name | Model | Fallback |
|---|---|---|---|
| Rust core, compilation fixes | rust-coder | glm-5.1:cloud | qwen3-coder-next:cloud |
| Architecture, planning, security | architect-planner | deepseek-v4-pro:cloud | deepseek-v3.2:cloud |
| Feature implementation | implementer | qwen3-coder-next:cloud | glm-5.1:cloud |
| Pre-merge gatekeeping | gatekeeper-reviewer | kimi-k2-thinking:cloud | kimi-k2.6:cloud |
| Tests, docs, bindings | worker | gemma4:31b:cloud | devstral-2:123b:cloud |
| Quick fixes, lint, CI | triage-router | gemini-3-flash-preview:cloud | deepseek-v4-flash:cloud |
| Wiring verification | wiring-verifier | deepseek-v4-pro:cloud | deepseek-v3.2:cloud |

## Launch Example

```bash
bash .claude/orchestrator_manager.sh pool launch rust-coder HANDOFF/todo/task_android_full_fix.md
```

## Parallel Strategy

- Launch up to 2 ollama cloud agents via `pool launch`
- ALSO launch built-in Agent sub-agents for parallel local work on the same model
- Verify after each agent completes: `cargo check --workspace`, `cargo clippy`, `cargo fmt`

> **TASK DELEGATION RULE:** Every time you generate a `BATCH_` markdown file for a worker, you MUST append this exact phrase to their instructions: "CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed."

> **ORCHESTRATION BEHAVIOR:** Once you have launched the worker agents to fill the slots (2/2), you MUST exit the active session immediately. Do not launch monitors. Do not use `sleep` or wait for them to finish. The system cron (`/loop 30m`) will wake you up automatically to check on them later. Fire and forget.

## Arguments: $ARGUMENTS