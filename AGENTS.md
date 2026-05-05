# SCMessenger Agent Orchestration Guide

## Windows Execution Constraints (CRITICAL)

### Process Management
You are on Windows. Standard Unix process commands (`ps`, `kill`) and Windows CLI commands with slashes (`tasklist /FI`) will fail or corrupt.

### Process Safety — Avoid Self-Termination
The `count_active_agents` function in the manager script counts all `claude` processes via PowerShell and subtracts 1 for the orchestrator. This is fragile — if the count is wrong, `pool launch` may refuse to start agents, or `pool stop` may target the wrong process.

**Safe approach**: Use the manager script for all agent lifecycle operations. Do not run `Stop-Process`, `taskkill`, or `kill` directly — these can terminate the orchestrator itself. The manager script's `pool stop` handles cleanup correctly.

### The ONLY Valid Way to Count/Find Agents
Use the manager script:
`& 'C:\Program Files\Git\bin\bash.exe' .claude/orchestrator_manager.sh pool status`

### The ONLY Valid Way to Kill Agents
Use the manager script:
`& 'C:\Program Files\Git\bin\bash.exe' .claude/orchestrator_manager.sh pool stop <agent_id>`

### Executing Shell Scripts (The PATH Limitation)
Native Windows cmd.exe and powershell.exe do NOT recognize bash or source commands. NEVER use source.

### The TWO Valid Ways to Run the Orchestrator Manager

1. **If inside Claude's internal Bash emulator**: Run `bash .claude/orchestrator_manager.sh <command>`

2. **If invoking from native CMD or PowerShell (Bulletproof Fallback)**: Run `'C:\Program Files\Git\bin\bash.exe' .claude/orchestrator_manager.sh <command>`

## Agent Pool Management

The orchestrator manager script (`.claude/orchestrator_manager.sh`) is the single source of truth for agent lifecycle management. It handles:
- Activation/deactivation of Gatekeeper mode
- Agent pool listing and launching
- Slot management (MAX=2 concurrent agents)
- Native and CLI agent tracking

### Common Commands
- `& 'C:\Program Files\Git\bin\bash.exe' .claude/orchestrator_manager.sh pool list` - Show all available agents
- `& 'C:\Program Files\Git\bin\bash.exe' .claude/orchestrator_manager.sh pool status` - Show active agents and slot usage
- `& 'C:\Program Files\Git\bin\bash.exe' .claude/orchestrator_manager.sh pool launch <agent_name> [task_file]` - Launch an agent
- `& 'C:\Program Files\Git\bin\bash.exe' .claude/orchestrator_manager.sh pool stop <agent_id>` - Stop an agent

## Agent Profiles

Refer to `.claude/agent_pool.json` for the complete list of available agents and their specializations.
