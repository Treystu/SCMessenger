# MiMo Code Optimization Guide for SCMessenger

## Overview

MiMo Code provides several features beyond Claude Code that can significantly accelerate SCMessenger v1.0.0 development. This guide maps each feature to the Fable 5 plan execution.

## 1. Model Strategy (Cost Optimization)

**Primary orchestrator, planner, verifier, and implementer:** Kimi (this session / Moderato subscription). Kimi owns architecture decisions, task decomposition, final review, security-critical paths, and all user communication.

OpenRouter free-tier models are used only as **augmentation for easy/mechanical sub-tasks** delegated through MiMo Code, to stretch Kimi budget further. Paid OpenRouter models (`openrouter/fusion`, `claude-opus-4.8`) are configured but **not used unless explicitly authorized**.

### Kimi (Primary)
- **Use for:** Planning, architecture, verification, final review, security/crypto decisions, complex debugging, orchestration
- **When:** Default for all non-trivial work
- **Best for:** Everything that requires reliable reasoning and full repo context

### OpenRouter Free Tier (via MiMo Code)
- **Use for:** Mechanical sub-tasks only — lint fixes, doc generation, simple refactors, test boilerplate, file listing, grep-heavy analysis
- **Cost:** $0
- **Best for:** Easy tasks where verification is cheap and errors are recoverable

### MiMo Auto / OpenRouter Free Router (Free Tier)
- **Use for:** Routine exploration, long-context scans, low-risk mechanical tasks
- **Context:** 1M tokens
- **Cost:** $0
- **Best for:** Track 5 hygiene tasks, large-file exploration

### Paid fallbacks (configured but disabled by policy)
- `openrouter/openrouter/fusion` — multi-model deliberation, ~$0.02–0.10+ per request
- `openrouter/anthropic/claude-opus-4.8` — critical architecture/security, only with explicit user approval

### Model Switching
```bash
# Default for easy mechanical sub-tasks
mimo --model openrouter/nex-agi/nex-n2-pro:free

# Free router — OpenRouter picks the best free model
mimo --model openrouter/openrouter/free

# Long-context free exploration
mimo --model mimo/mimo-auto
```

## 2. Compose Mode (Spec-Driven Development)

MiMo Code's compose mode is ideal for executing the Fable 5 plan, which is already a detailed spec.

### How to Use
```bash
# Start in compose mode
mimo compose

# Or use the /fable5 command
mimo run "/fable5 T5.1"
```

### Compose Skills Available
- **plan** — Read-only analysis of the codebase before implementation
- **execute** — Implement the task following the spec exactly
- **review** — Code review for correctness and philosophy compliance
- **tdd** — Test-driven development (write tests first, then implement)
- **debug** — Debug and fix issues found during verification
- **verify** — Run the Verification section of each task
- **merge** — Prepare changes for commit

### Workflow for Each Task
1. `/fable5 T5.1` → compose mode reads the spec
2. plan skill → analyzes what needs to change
3. execute skill → implements the changes
4. verify skill → runs the verification checklist
5. review skill → checks philosophy compliance
6. merge skill → commits with proper message

## 3. Goal/Stop Conditions (Autonomous Completion)

The `/goal` command prevents premature stops during autonomous work.

### Setting Goals
```bash
# In MiMo Code TUI
/goal Complete T5.1: all build artifacts purged, .gitignore updated, cargo build succeeds

# For batch execution
/goal Complete Track 5 (T5.1-T5.9): CI/CD pipeline green, FFI snapshot test passing
```

### How It Works
1. You set a goal condition
2. MiMo Code works autonomously
3. When the agent tries to stop, a judge model evaluates whether the goal is truly met
4. If not met, work continues automatically
5. Prevents "optimistic stops" where the agent thinks it's done but isn't

### Best Use Cases
- Track 5 execution (multiple sequential tasks)
- Track 1 transport wiring (multi-file changes)
- Any task with a clear verification checklist

## 4. Dream & Distill (Self-Improvement)

### /dream — Knowledge Extraction
Run after each session to persist learnings:
```bash
/dream
```
- Scans recent session traces
- Extracts persistent knowledge into MEMORY.md
- Removes outdated entries
- Updates project understanding

**When to run:** After completing each track, or after discovering non-obvious facts about the codebase.

### /distill — Workflow Packaging
Run when you notice repeated manual patterns:
```bash
/distill
```
- Discovers repeated workflows in recent work
- Packages high-confidence candidates into reusable skills
- Creates subagent templates for common tasks

**When to run:** After completing 5+ similar tasks (e.g., multiple transport implementations).

## 5. Subagent System (Parallel Execution)

MiMo Code's native subagent system replaces the external orchestrator_manager.sh script.

### Parallel Task Execution
```bash
# In MiMo Code TUI, the agent can spawn subagents:
# "Run T5.1 and T5.8 in parallel"
# MiMo Code creates two subagents working simultaneously
```

### Agent Types for SCMessenger
| Agent | Model | Use Case |
|-------|-------|----------|
| build | openrouter/nex-agi/nex-n2-pro:free | Default implementation |
| plan | openrouter/openai/gpt-oss-20b:free | Read-only analysis |
| compose | openrouter/nex-agi/nex-n2-pro:free | Spec-driven development |
| rust-coder | openrouter/nex-agi/nex-n2-pro:free | Rust core changes |
| mobile-dev | openrouter/nex-agi/nex-n2-pro:free | Android/iOS changes |
| reviewer | openrouter/openai/gpt-oss-20b:free | Code review |

### Max Concurrency
- Default: 3 subagents
- Configurable in .mimocode/mimocode.json
- Each subagent has its own context window

## 6. Checkpoint System (Long-Running Tasks)

MiMo Code automatically saves session state and can resume from checkpoints.

### Automatic Checkpoints
- Saved when context approaches the limit
- Includes task progress, memory, and recent messages
- Enables seamless session resumption

### Manual Checkpoint
```bash
# In MiMo Code TUI
/checkpoint Save before attempting complex FFI change
```

### Resuming
```bash
# Continue last session
mimo --continue

# Or start fresh with memory loaded
mimo
# Memory.md, checkpoint.md, notes.md loaded automatically
```

## 7. Task Tracking (Progress Visibility)

### Tree-Shaped Tasks
MiMo Code tracks tasks as T1, T1.1, T1.2, etc. — matching the Fable 5 plan exactly.

### Task Progress Files
Each task has `tasks/<id>/progress.md` with:
- Status (pending/in_progress/completed)
- Dependencies
- Implementation notes
- Verification checklist

### Viewing Progress
```bash
# In MiMo Code TUI
/tasks

# Or check files directly
cat tasks/T5.1/progress.md
```

## 8. Workspace Cleanup (SCMessenger Deletion)

### Self-Contained Workspace
Everything MiMo Code needs is in SCMessenger_Clean:
- `.mimocode/mimocode.json` — project config
- `MEMORY.md` — persistent knowledge
- `checkpoint.md` — session state
- `notes.md` — scratch notes
- `tasks/` — task progress
- `fable5plan.md` — the spec

### Clean Removal
```bash
# Delete the workspace entirely
rm -rf /Users/scmessenger/Documents/Github/SCMessenger_Clean

# MiMo Code global config remains at ~/.config/mimocode/
# But no project-specific state remains
```

### No Global Pollution
- Project config is local to SCMessenger_Clean
- Memory is local to SCMessenger_Clean
- Task progress is local to SCMessenger_Clean
- Global config only has provider credentials

## 9. Execution Strategy

### Phase 1: Track 5 (CI/Hygiene) — First Priority
1. Start with `mimo` in SCMessenger_Clean
2. Set goal: "Complete Track 5 (T5.1-T5.9)"
3. Use compose mode with /fable5 command
4. Use MiMo Auto (free) for T5.1, T5.2, T5.3 (mechanical tasks)
5. Use MiMo V2.5 Pro for T5.4-T5.7 (CI design)
6. Run /dream after Track 5 completes

### Phase 2: Track 1 (Transport) — After Track 5
1. Use rust-coder + mobile-dev agents in parallel
2. T1.1 → T1.2 → T1.3 (Android path)
3. T1.1 → T1.4 (Wi-Fi Direct path)
4. T1.5, T1.6 in parallel (BLE hardening)
5. T1.7 after T1.1

### Phase 3: Tracks 2-4 (Parallel)
1. Track 2 (DTN): T2.1 → T2.2 → T2.3 → T2.4, T2.5
2. Track 3 (Routing): T3.1 → T3.2 → T3.3 → T3.4
3. Track 4 (Crypto/UI): T4.1 → T4.2, T4.3, T4.4, T4.5
4. Run /distill after each track to package reusable workflows

## 10. Key Commands Reference

| Command | Purpose |
|---------|---------|
| `mimo` | Start TUI in project directory |
| `mimo run "/fable5 T5.1"` | Execute specific task |
| `mimo --model openrouter/nex-agi/nex-n2-pro:free` | Use SCMessenger default OpenRouter model |
| `mimo --model mimo/mimo-auto` | Use free MiMo Auto model |
| `mimo --continue` | Resume last session |
| `/goal <condition>` | Set autonomous stop condition |
| `/dream` | Extract knowledge from session |
| `/distill` | Package repeated workflows |
| `/tasks` | View task progress |
| `/checkpoint` | Save session state |
| `/fable5 <task>` | Execute Fable 5 plan task |
| `/gatekeeper` | Run gatekeeper checks |
| `/orchestrate` | Launch swarm orchestration |
