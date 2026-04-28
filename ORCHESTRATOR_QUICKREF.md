# SCMessenger Lead Orchestrator - Quick Reference

## Simple Orchestrator Control

### Quick Commands
```bash
# Check system status
./orchestrator_activate.sh

# Control orchestrator
"Activate orchestrator role" - Enable monitoring
"Deactivate orchestrator role" - Disable monitoring
"Orchestrator status" - Show current state
```

### Integration Mode
- **Respects existing** 15-minute maintenance loop
- **Uses existing** HANDOFF task system
- **Follows existing** agent guidance
- **No duplication** - pure integration

## Master Files Monitored
- `MASTER_BUG_TRACKER.md` - P0 bugs prioritized first
- `REMAINING_WORK_TRACKING.md` - Overall work tracking
- `FEATURE_PARITY.md` - Feature completion status
- `WASM_TRANSPORT_PARITY_PLAN.md` - WASM transport parity

## Directory Structure
- `HANDOFF/todo/` - Tasks waiting for sub-agents
- `HANDOFF/done/` - Completed tasks
- `HANDOFF/IN_PROGRESS/` - Currently active tasks

## Priority Hierarchy
1. **P0** - Critical/Security (Highest)
2. **P1** - Core Functionality
3. **P2** - Build/Test Issues
4. **P3** - Enhancements (Lowest)

## Task Naming Pattern
`[TYPE]-[DESC]-[ID]_[PRIORITY]_[COMPONENT]_[SUMMARY].md`

Examples:
- `SEC-AUTH-001_P0_Security_Authentication_Fix.md`
- `CORE-DRIFT-002_P1_Core_Drift_Protocol.md`
- `TEST-INT-003_P2_Testing_Integration_Fix.md`

## Orchestrator Manager Commands
```bash
./.claude/orchestrator_manager.sh activate   # Activate + launch agent
./.claude/orchestrator_manager.sh launch     # Launch CLI agent only
./.claude/orchestrator_manager.sh stop <id>  # Stop CLI agent
./.claude/orchestrator_manager.sh deactivate # Deactivate orchestrator
./.claude/orchestrator_manager.sh status     # Show current state
```

## Agent Pool Commands
```bash
# View all available agent profiles
./.claude/orchestrator_manager.sh pool list

# Spin up an agent by name
./.claude/orchestrator_manager.sh pool launch scout
./.claude/orchestrator_manager.sh pool launch implementer HANDOFF/todo/P0_BUILD_003.md
./.claude/orchestrator_manager.sh pool launch rust-coder
./.claude/orchestrator_manager.sh pool launch security-auditor

# Spin down an agent
./.claude/orchestrator_manager.sh pool stop <agent_id>

# Check pool slot usage
./.claude/orchestrator_manager.sh pool status
```

### Agent Pool Profiles (`.claude/agent_pool.json`)

| Name | Launch | Subagent/Model | Purpose |
|------|--------|----------------|---------|
| scout | NATIVE | Explore/sonnet | Fast codebase search |
| architect | NATIVE | Plan/sonnet | Architecture & design |
| implementer | NATIVE | general-purpose/sonnet | Code changes (worktree) |
| reviewer | NATIVE | general-purpose/opus | Code review & verification |
| rust-coder | CLI | qwen3-coder:480b:cloud | Rust core implementation |
| security-auditor | CLI | deepseek-v3.2:cloud | Crypto/protocol audit |
| platform-engineer | CLI | gemma4:31b:cloud | Bindings, tests, docs |
| fast-exec | CLI | gemini-3-flash-preview:cloud | Triage, lint, quick fixes |

**Max concurrent: 2 agents (shared across native + CLI)**

### Native Agent Launch (within Claude Code session)
```
Agent({
  subagent_type: "Explore",   // scout
  model: "sonnet",
  prompt: "Search for X in the codebase..."
})

Agent({
  subagent_type: "general-purpose",  // implementer
  model: "sonnet",
  isolation: "worktree",
  prompt: "Implement feature X..."
})

Agent({
  subagent_type: "general-purpose",  // reviewer
  model: "opus",
  prompt: "Review the changes in..."
})
```

### Task-to-Agent Routing
Task files with keywords in their name are auto-routed:
- EXPLORE/RESEARCH/SEARCH -> scout
- ARCHITECTURE/PLAN/DESIGN -> architect
- IMPLEMENTATION/BUG/FIX/FEATURE -> implementer
- REVIEW/AUDIT/VERIFY -> reviewer
- RUST/CORE/PROTOCOL -> rust-coder
- SECURITY/CRYPTO -> security-auditor
- PLATFORM/BINDINGS/TEST/DOCS -> platform-engineer
- LINT/TRIAGE/QUICK/CI -> fast-exec

## Automation Features
- Continuous 5-minute monitoring loop
- Automatic task generation from master files
- Sub-agent work saturation management
- Priority-based task sequencing
- Completion tracking via HANDOFF/done/
- Autonomous agent launch via `ollama launch claude`
- Native Agent tool subagents for in-session work
- Unified 2-slot concurrency limit across all agent types

---
*Last updated: 2026-04-17*