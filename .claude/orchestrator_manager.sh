#!/bin/bash
# Orchestrator State Management (v3.3)
# Handles activation, deactivation, multi-agent slot management (MAX=2)
# Now supports native Agent tool subagents + CLI-based Ollama agents
# v3.3: STRICT OS-LEVEL PROCESS GATING — fixes pool_stop arg bug, adds assert_process_limit
# v3.2: PowerShell-based process checking for Windows, self-preservation PID guard

# Source cross-platform process helpers
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/scripts/process_alive.sh"

STATE_FILE=".claude/orchestrator_state.json"
AGENT_ROOT=".claude/agents"
POOL_CONFIG=".claude/agent_pool.json"
MAX_SUBAGENTS=2
MAX_OS_PROCESSES=3
ORCHESTRATOR_PID_FILE=".claude/orchestrator.pid"

# ─── STRICT OS-LEVEL PROCESS GATING ────────────────────────────────────────
# Two-Tier Topology: Max 3 OS processes (Lead Orchestrator + 2 Workers)
# This MUST be checked BEFORE every agent launch.

count_os_claude_processes() {
    # Count actual claude.exe processes via PowerShell (Windows)
    local count=$(powershell.exe -NoProfile -Command "@(Get-Process -Name claude -ErrorAction SilentlyContinue).Count" 2>/dev/null)
    if [ -z "$count" ] || ! [[ "$count" =~ ^[0-9]+$ ]]; then
        echo "0"
        return
    fi
    echo "$count"
}

assert_process_limit() {
    # Count only TRACKED agent processes that are actually alive.
    # Do NOT count all claude.exe — IDE extensions, desktop app, etc. are not agents.
    local agent_count=$(count_cli_agents)
    # Allow 1 orchestrator + up to MAX_SUBAGENTS workers = MAX_OS_PROCESSES total
    if [ "$agent_count" -ge "$MAX_SUBAGENTS" ]; then
        echo "CRITICAL: Agent slot limit reached ($agent_count/$MAX_SUBAGENTS tracked agents alive)."
        echo "Refusing to launch agent. Stop an existing agent first."
        return 1
    fi
    # Also sanity-check: if total OS claude.exe exceeds MAX_OS_PROCESSES + 2 buffer,
    # something else is spawning claude processes; warn but don't block.
    local os_count=$(count_os_claude_processes)
    local max_buffer=$((MAX_OS_PROCESSES + 2))
    if [ "$os_count" -gt "$max_buffer" ]; then
        echo "WARNING: $os_count claude.exe processes detected (expected max ~$max_buffer)."
        echo "Non-agent Claude processes (IDE, desktop) may be consuming slots."
    fi
    return 0
}

# Get orchestrator PID from file (stored during activation)
get_orchestrator_pid() {
    if [ -f "$ORCHESTRATOR_PID_FILE" ]; then
        local pid=$(cat "$ORCHESTRATOR_PID_FILE")
        if [ -n "$pid" ]; then
            echo "$pid"
            return
        fi
    fi
    echo ""
}

store_orchestrator_pid() {
    # On Windows, $PPID points to an intermediate shell, not claude.exe.
    # Walk up the process tree to find the claude.exe ancestor.
    local shell_pid=$$
    local parent_pid=$PPID
    # Try to find a claude.exe process in the parent chain
    local found_pid=$(powershell.exe -NoProfile -Command "
        \$p = Get-Process -Id $parent_pid -ErrorAction SilentlyContinue
        while (\$p -and \$p.ProcessName -ne 'claude') {
            \$p = Get-Process -Id \$p.Id -ErrorAction SilentlyContinue | ForEach-Object { Get-Process -Id \$_.Id -ErrorAction SilentlyContinue }
            break
        }
        if (\$p -and \$p.ProcessName -eq 'claude') { Write-Output \$p.Id } else { Write-Output '' }
    " 2>/dev/null)
    # Fallback: if the parent chain walk failed, use PPID as best guess
    if [ -z "$found_pid" ]; then
        found_pid="$parent_pid"
    fi
    echo "$found_pid" > "$ORCHESTRATOR_PID_FILE"
    echo "Orchestrator PID stored: $found_pid"
}

remove_orchestrator_pid() {
    rm -f "$ORCHESTRATOR_PID_FILE"
}

# Clean up stale PID files (agents whose processes have died)
clean_stale_pids() {
    if [ ! -d "$AGENT_ROOT" ]; then
        return
    fi
    local orch_pid=$(get_orchestrator_pid)
    for pidfile in "$AGENT_ROOT"/*/pid; do
        if [ -f "$pidfile" ]; then
            local pid=$(cat "$pidfile")
            local dir=$(dirname "$pidfile")
            local id=$(basename "$dir")
            # Skip if this is the orchestrator
            if [ -n "$orch_pid" ] && [ "$pid" = "$orch_pid" ]; then
                echo "Cleaning orchestrator PID from agent tracking: $id"
                rm -rf "$dir"
                continue
            fi
            # Check if the process is still alive
            if ! process_alive "$pid"; then
                echo "Cleaning stale agent: $id (PID $pid no longer exists)"
                rm -rf "$dir"
            fi
        fi
    done
}

count_active_agents() {
    # CLI-launched agents (ollama spawn) are the only source of truth.
    # Native sub-agents (internal Agent tool) are not tracked separately.
    count_cli_agents
}

count_cli_agents() {
    local count=0
    if [ -d "$AGENT_ROOT" ]; then
        for pidfile in "$AGENT_ROOT"/*/pid; do
            if [ -f "$pidfile" ]; then
                pid=$(cat "$pidfile")
                if process_alive "$pid"; then
                    ((count++))
                fi
            fi
        done
    fi
    echo $count
}

list_agents() {
    echo "=== SCMessenger Active Sub-Agents ==="
    active_count=$(count_active_agents)
    echo "Slots: $active_count/$MAX_SUBAGENTS"
    echo "------------------------------------"
    if [ "$active_count" -eq 0 ]; then
        echo "No sub-agents currently running."
        return
    fi
    local orch_pid=$(get_orchestrator_pid)
    printf "%-25s %-15s %-10s %-10s %-10s\n" "AGENT_ID" "TYPE" "MODEL" "PID" "STATUS"
    # List CLI agents (check tracked PIDs, skip orchestrator)
    for dir in "$AGENT_ROOT"/*; do
        if [ -d "$dir" ] && [ -f "$dir/pid" ]; then
            id=$(basename "$dir")
            pid=$(cat "$dir/pid")
            model=$(grep "AGENT_MODEL" "$dir/config" 2>/dev/null | cut -d'=' -f2)
            # Self-preservation: never list the orchestrator as an agent
            if [ -n "$orch_pid" ] && [ "$pid" = "$orch_pid" ]; then
                printf "%-25s %-15s %-10s %-10s %-10s\n" "$id" "CLI" "$model" "$pid" "ORCHESTRATOR"
                continue
            fi
            if process_alive "$pid"; then
                printf "%-25s %-15s %-10s %-10s %-10s\n" "$id" "CLI" "$model" "$pid" "RUNNING"
            else
                printf "%-25s %-15s %-10s %-10s %-10s\n" "$id" "CLI" "$model" "$pid" "STALE"
            fi
        fi
    done
    # List native agents
    for dir in "$AGENT_ROOT"/*; do
        if [ -d "$dir" ] && [ -f "$dir/native_marker" ]; then
            id=$(basename "$dir")
            agent_type=$(grep "SUBAGENT_TYPE" "$dir/native_marker" 2>/dev/null | cut -d'=' -f2)
            agent_model=$(grep "MODEL" "$dir/native_marker" 2>/dev/null | cut -d'=' -f2)
            task=$(grep "TASK" "$dir/native_marker" 2>/dev/null | cut -d'=' -f2-)
            printf "%-25s %-15s %-10s %-10s %-10s\n" "$id" "NATIVE:$agent_type" "$agent_model" "-" "ACTIVE"
        fi
    done
}

# ─── Pool Commands ───────────────────────────────────────────────────────────

pool_list() {
    if [ ! -f "$POOL_CONFIG" ]; then
        echo "Error: Agent pool config not found at $POOL_CONFIG"
        return 1
    fi
    echo "=== SCMessenger Agent Pool ==="
    active_count=$(count_active_agents)
    echo "Active Slots: $active_count/$MAX_SUBAGENTS"
    echo ""
    printf "%-20s %-10s %-30s %-40s\n" "NAME" "LAUNCH" "MODEL" "PURPOSE"
    echo "--------------------------------------------------------------------------------------------------------"
    python -c "
import json, sys
with open('$POOL_CONFIG') as f:
    cfg = json.load(f)
for a in cfg.get('agents', []):
    name = a.get('name','')
    lt = a.get('launch_type','')
    model = a.get('model','')
    purpose = a.get('purpose','')
    if lt == 'native':
        st = a.get('subagent_type','')
        model = f'{st}/{model}'
        lt = 'NATIVE'
    else:
        lt = 'CLI'
    print(f'{name:<20s} {lt:<10s} {model:<30s} {purpose:<40s}')
"
    echo ""
    echo "Default agent: $(python -c "import json; print(json.load(open('$POOL_CONFIG')).get('default_agent','implementer'))")"
    echo "Usage: .claude/orchestrator_manager.sh pool launch <agent_name> [task_file]"
}

pool_launch() {
    local agent_name="${1:-}"
    local task_file="${2:-}"

    if [ -z "$agent_name" ]; then
        echo "Usage: pool launch <agent_name> [task_file]"
        echo "Run 'pool list' to see available agents."
        return 1
    fi

    # Check slot availability
    active=$(count_active_agents)
    if [ "$active" -ge "$MAX_SUBAGENTS" ]; then
        echo "Error: Sub-agent limit reached ($MAX_SUBAGENTS slots). Stop an agent first."
        return 1
    fi

    # STRICT GATE: Check actual OS process count (Two-Tier Topology: max 3)
    if ! assert_process_limit; then
        return 1
    fi

    # Find agent profile in pool config
    if [ ! -f "$POOL_CONFIG" ]; then
        echo "Error: Agent pool config not found at $POOL_CONFIG"
        return 1
    fi

    # Extract agent config via Python
    local agent_info
    agent_info=$(python -c "
import json, sys
with open('$POOL_CONFIG') as f:
    cfg = json.load(f)
for a in cfg.get('agents', []):
    if a.get('name') == '$agent_name':
        print(a.get('launch_type',''))
        print(a.get('model',''))
        print(a.get('subagent_type',''))
        print(a.get('isolation','') or '')
        print(a.get('fallback_model',''))
        sys.exit(0)
sys.exit(1)
" 2>/dev/null)

    if [ $? -ne 0 ] || [ -z "$agent_info" ]; then
        echo "Error: Agent '$agent_name' not found in pool. Run 'pool list' to see available agents."
        return 1
    fi

    local launch_type=$(echo "$agent_info" | sed -n '1p')
    local model=$(echo "$agent_info" | sed -n '2p')
    local subagent_type=$(echo "$agent_info" | sed -n '3p')
    local isolation=$(echo "$agent_info" | sed -n '4p')

    # Get fallback model for CLI agents
    local fallback_model
    if [ "$launch_type" = "cli" ]; then
        fallback_model=$(python -c "
import json
with open('$POOL_CONFIG') as f:
    cfg = json.load(f)
for a in cfg.get('agents', []):
    if a.get('name') == '$agent_name':
        print(a.get('fallback_model', ''))
        break
" 2>/dev/null)
    fi

    local agent_id="${agent_name}_$(date +%s)"

    if [ "$launch_type" = "native" ]; then
        # Native agent: validate model before creating marker
        echo "Validating model $model before launch..."

        # Source the model validation template
        if [ -f "$PWD/.claude/model_validation_template.sh" ]; then
            source "$PWD/.claude/model_validation_template.sh"
            validated_model=$(validate_model_before_launch "$model" "$subagent_type")
            echo "Using validated model: $validated_model"
            model="$validated_model"
        else
            echo "⚠️  Model validation template not found, proceeding with $model"
        fi

        # Native agent: create marker file for tracking
        mkdir -p "$AGENT_ROOT/$agent_id"
        cat > "$AGENT_ROOT/$agent_id/native_marker" <<EOF
SUBAGENT_TYPE=$subagent_type
MODEL=$model
ISOLATION=$isolation
TASK=${task_file:-none}
START_TIME=$(date +%s)
EOF
        echo "Launched native agent: $agent_id"
        echo "  Type: $subagent_type (model: $model)"
        echo "  Isolation: ${isolation:-none}"
        echo "  Task: ${task_file:-none}"
        echo ""
        echo "NOTE: Native agents are launched via Claude Code's Agent tool."
        echo "The Lead Orchestrator should invoke Agent({subagent_type: \"$subagent_type\", model: \"$model\"$([ -n "$isolation" ] && echo ", isolation: \"$isolation\"" )}) with the task prompt."
    else
        # CLI agent: validate model before launch
        echo "Validating model $model before launch..."

        # Source the model validation template
        if [ -f "$PWD/.claude/model_validation_template.sh" ]; then
            source "$PWD/.claude/model_validation_template.sh"
            validated_model=$(validate_model_before_launch "$model" "$subagent_type")
            echo "Using validated model: $validated_model"
            model="$validated_model"
        else
            echo "⚠️  Model validation template not found, proceeding with $model"
        fi

        echo "Launching CLI agent: $agent_id with model $model"
        if ! ./scripts/launch_agent.sh "$model" "$agent_id" "$task_file" start 2>/dev/null; then
            if [ -n "$fallback_model" ]; then
                echo "Primary model $model unavailable, falling back to $fallback_model"
                ./scripts/launch_agent.sh "$fallback_model" "$agent_id" "$task_file" start
            else
                echo "Error: Primary model $model unavailable and no fallback configured."
                return 1
            fi
        fi
    fi
}

pool_stop() {
    local agent_id="${1:-}"

    if [ -z "$agent_id" ]; then
        echo "Usage: pool stop <agent_id>"
        return 1
    fi

    if [ ! -d "$AGENT_ROOT/$agent_id" ]; then
        echo "Error: Agent '$agent_id' not found."
        return 1
    fi

    local orch_pid=$(get_orchestrator_pid)

    # Check if native or CLI agent
    if [ -f "$AGENT_ROOT/$agent_id/native_marker" ]; then
        rm -f "$AGENT_ROOT/$agent_id/native_marker"
        echo "Stopped native agent: $agent_id"
    elif [ -f "$AGENT_ROOT/$agent_id/pid" ]; then
        local agent_pid=$(cat "$AGENT_ROOT/$agent_id/pid")
        # Self-preservation: never kill the orchestrator process
        if [ -n "$orch_pid" ] && [ "$agent_pid" = "$orch_pid" ]; then
            echo "CRITICAL: Agent PID $agent_pid matches orchestrator PID. Refusing to kill."
            echo "Removing stale tracking file instead."
            rm -rf "$AGENT_ROOT/$agent_id"
            return 1
        fi
        ./scripts/launch_agent.sh "dummy" "$agent_id" "" stop
        rm -rf "$AGENT_ROOT/$agent_id"
        echo "Stopped CLI agent: $agent_id"
    else
        echo "Error: Agent '$agent_id' has no valid marker or PID file."
        return 1
    fi
}

pool_status() {
    echo "=== Agent Pool Status ==="
    active_count=$(count_active_agents)
    echo "Slots:  $active_count/$MAX_SUBAGENTS"
    echo ""
    if [ "$active_count" -eq 0 ]; then
        echo "No agents active. Use 'pool launch <name>' to start one."
    else
        list_agents
    fi
}

# ─── Main Command Router ─────────────────────────────────────────────────────

case "$1" in
    "activate")
        touch ".claude/orchestrator_active"
        store_orchestrator_pid
        clean_stale_pids
        echo "Orchestrator Gatekeeper activated."
        ;;
    "list")
        list_agents
        ;;
    "launch")
        active=$(count_active_agents)
        if [ "$active" -ge "$MAX_SUBAGENTS" ]; then
            echo "Error: Sub-agent limit reached ($MAX_SUBAGENTS slots)."
            echo "Stop an existing agent before launching a new one."
            exit 1
        fi
        model="${2:-qwen3-coder-next:cloud}"
        id="${3:-agent_$(date +%s)}"
        echo "Launching sub-agent [$id] with model [$model]..."
        ./scripts/launch_agent.sh "$model" "$id" start
        ;;
    "stop")
        if [ -z "$2" ]; then
            echo "Usage: $0 stop <agent_id>"
            exit 1
        fi
        id="$2"
        if [ -d "$AGENT_ROOT/$id" ]; then
            # Self-preservation: never kill the orchestrator process
            local orch_pid=$(get_orchestrator_pid)
            if [ -f "$AGENT_ROOT/$id/pid" ] && [ -n "$orch_pid" ]; then
                local agent_pid=$(cat "$AGENT_ROOT/$id/pid")
                if [ "$agent_pid" = "$orch_pid" ]; then
                    echo "CRITICAL: Agent PID $agent_pid matches orchestrator PID. Refusing to kill."
                    rm -rf "$AGENT_ROOT/$id"
                    exit 1
                fi
            fi
            ./scripts/launch_agent.sh "dummy" "$id" "" stop
            rm -rf "$AGENT_ROOT/$id"
        else
            echo "Error: Agent [$id] not found."
        fi
        ;;
    "pool")
        case "${2:-}" in
            "list")
                pool_list
                ;;
            "launch")
                pool_launch "$3" "$4"
                ;;
            "stop")
                pool_stop "$3"
                ;;
            "status")
                pool_status
                ;;
            *)
                echo "Pool Commands:"
                echo "  pool list              Show all agent profiles"
                echo "  pool launch <name>     Spin up an agent from the pool"
                echo "  pool stop <agent_id>   Spin down an agent"
                echo "  pool status            Show active agents and slot usage"
                ;;
        esac
        ;;
    "deactivate")
        rm -f ".claude/orchestrator_active"
        remove_orchestrator_pid
        clean_stale_pids
        echo "Orchestrator Gatekeeper deactivated."
        ;;
    "status")
        if [ -f ".claude/orchestrator_active" ]; then
            echo "Orchestrator Status: ACTIVE (Gatekeeper Mode)"
            list_agents
        else
            echo "Orchestrator Status: INACTIVE (Monitoring Only)"
        fi
        ;;
    *)
        echo "Usage: $0 {activate|launch|stop|pool|list|status|deactivate}"
        echo ""
        echo "Commands:"
        echo "  activate               Activate Gatekeeper mode"
        echo "  launch [model] [id]    Launch CLI sub-agent"
        echo "  stop <agent_id>        Stop CLI sub-agent"
        echo "  list                   List active sub-agents"
        echo "  status                 Show orchestrator status"
        echo "  deactivate             Deactivate Gatekeeper mode"
        echo ""
        echo "Pool Commands:"
        echo "  pool list              Show all agent pool profiles"
        echo "  pool launch <name>     Spin up agent from pool config"
        echo "  pool stop <agent_id>   Spin down an active agent"
        echo "  pool status            Show pool slot usage"
        exit 1
        ;;
esac