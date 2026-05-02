#!/bin/bash
# Orchestrator State Management (v3.3)
# Handles activation, deactivation, multi-agent slot management (MAX=2)
# Now supports native Agent tool subagents + CLI-based Ollama agents
# v3.3: STRICT OS-LEVEL PROCESS GATING — fixes pool_stop arg bug, adds assert_process_limit
# v3.2: PowerShell-based process checking for Windows, self-preservation PID guard

# Source cross-platform process helpers

# Python detection: prefer python3, then python, then py -3 (Windows launcher)
detect_python() {
    if command -v python3 &>/dev/null && python3 --version &>/dev/null; then
        echo "python3"
    elif command -v python &>/dev/null && python --version &>/dev/null; then
        echo "python"
    elif command -v py &>/dev/null && py -3 --version &>/dev/null; then
        echo "py -3"
    else
        echo ""
    fi
}
PYTHON="$(detect_python)"
if [ -z "$PYTHON" ]; then
    echo "ERROR: No Python interpreter found. Install Python 3 or the py launcher."
    exit 1
fi
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
            \$parentId = \$p.ParentId
            if (-not \$parentId -or \$parentId -eq 0 -or \$parentId -eq \$p.Id) { break }
            \$p = Get-Process -Id \$parentId -ErrorAction SilentlyContinue
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
                # Aggressive dual-kill: POSIX signal + Windows taskkill (belt-and-suspenders)
                kill -9 $pid 2>/dev/null || true
                taskkill //F //T //PID $pid 2>/dev/null || true
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
    $PYTHON -c "
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
    echo "Default agent: $($PYTHON -c "import json; print(json.load(open('$POOL_CONFIG')).get('default_agent','implementer'))")"
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

    # Check file domain conflict with active agents
    if ! check_file_domain_conflict "$agent_name"; then
        echo "Error: File domain conflict with active agent. Choose a different agent or stop the conflicting agent."
        return 1
    fi

    # Find agent profile in pool config
    if [ ! -f "$POOL_CONFIG" ]; then
        echo "Error: Agent pool config not found at $POOL_CONFIG"
        return 1
    fi

    # Extract agent config via Python
    local agent_info
    agent_info=$($PYTHON -c "
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
        fallback_model=$($PYTHON -c "
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
            echo "Model validation template not found, proceeding with $model"
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
            echo "Model validation template not found, proceeding with $model"
        fi

        echo "Launching CLI agent: $agent_id with model $model"
        export CARGO_INCREMENTAL=0
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

# ─── Compile Lock Protocol ──────────────────────────────────────────────────

COMPILE_LOCK=".claude/compile.lock"
COMPILE_LOCK_TIMEOUT=300  # 5 minutes max hold time

acquire_compile_lock() {
    local agent_id="$1"
    local scope="${2:-workspace}"
    local now=$(date +%s)

    if [ -f "$COMPILE_LOCK" ]; then
        local holder=$(grep "^HOLDER=" "$COMPILE_LOCK" | cut -d'=' -f2)
        local acquired=$(grep "^ACQUIRED_AT=" "$COMPILE_LOCK" | cut -d'=' -f2)
        local lock_pid=$(grep "^PID=" "$COMPILE_LOCK" | cut -d'=' -f2)

        if [ -n "$lock_pid" ] && process_alive "$lock_pid"; then
            local elapsed=$((now - acquired))
            if [ "$elapsed" -lt "$COMPILE_LOCK_TIMEOUT" ]; then
                echo "COMPILE_LOCKED:$holder"
                return 1
            fi
        fi
        rm -f "$COMPILE_LOCK"
    fi

    cat > "$COMPILE_LOCK" <<EOF
HOLDER=$agent_id
ACQUIRED_AT=$now
SCOPE=$scope
PID=$$
EOF
    return 0
}

release_compile_lock() {
    local agent_id="$1"
    if [ -f "$COMPILE_LOCK" ]; then
        local holder=$(grep "^HOLDER=" "$COMPILE_LOCK" | cut -d'=' -f2)
        if [ "$holder" = "$agent_id" ]; then
            rm -f "$COMPILE_LOCK"
        fi
    fi
}

# ─── File Domain Conflict Check ─────────────────────────────────────────────

check_file_domain_conflict() {
    local new_agent_name="$1"
    local new_domains=$($PYTHON -c "
import json
with open('$POOL_CONFIG') as f:
    cfg = json.load(f)
for a in cfg.get('agents', []):
    if a.get('name') == '$new_agent_name':
        print(','.join(a.get('file_domains', [])))
        break
" 2>/dev/null)

    if [ -z "$new_domains" ]; then
        return 0
    fi

    for dir in "$AGENT_ROOT"/*/; do
        [ -d "$dir" ] || continue
        local active_id=$(basename "$dir")
        local active_name=$(echo "$active_id" | sed 's/_[0-9]*$//')

        local active_domains=$($PYTHON -c "
import json
with open('$POOL_CONFIG') as f:
    cfg = json.load(f)
for a in cfg.get('agents', []):
    if a.get('name') == '$active_name':
        print(','.join(a.get('file_domains', [])))
        break
" 2>/dev/null)

        if [ -z "$active_domains" ]; then
            continue
        fi

        local overlap=$($PYTHON -c "
new = set('${new_domains}'.split(','))
active = set('${active_domains}'.split(','))
overlap = new & active
if overlap:
    print(','.join(overlap))
" 2>/dev/null)

        if [ -n "$overlap" ]; then
            echo "CONFLICT: File domain overlap with active agent $active_id: $overlap"
            return 1
        fi
    done
    return 0
}

# ─── Pre-Launch Hygiene ──────────────────────────────────────────────────────

pool_launch_clean() {
    local agent_name="${1:-}"
    local task_file="${2:-}"

    echo "=== Pre-Launch Hygiene Check ==="

    # Phase 1: Determine tracked PIDs (orchestrator + agents)
    local orch_pid=$(get_orchestrator_pid)
    local tracked_pids=" $orch_pid"
    for pidfile in "$AGENT_ROOT"/*/pid; do
        if [ -f "$pidfile" ]; then
            tracked_pids="$tracked_pids $(cat "$pidfile")"
        fi
    done

    # Self-preservation: discover the claude.exe hosting this shell and add it to tracked
    local host_claude_pid=$(powershell.exe -NoProfile -Command "
        \$bash = Get-Process -Id $$ -ErrorAction SilentlyContinue
        \$p = \$bash
        while (\$p -and \$p.ProcessName -ne 'claude') {
            \$parentId = \$p.ParentId
            if (-not \$parentId -or \$parentId -eq 0 -or \$parentId -eq \$p.Id) { break }
            \$p = Get-Process -Id \$parentId -ErrorAction SilentlyContinue
        }
        if (\$p -and \$p.ProcessName -eq 'claude') { Write-Output \$p.Id } else { Write-Output '' }
    " 2>/dev/null)
    if [ -n "$host_claude_pid" ]; then
        tracked_pids="$tracked_pids $host_claude_pid"
    fi

    # Kill stale untracked claude.exe processes
    local untracked=$(powershell.exe -NoProfile -Command "
        \$tracked = @($(echo "$tracked_pids" | tr ' ' ',' | sed 's/^,//' | sed 's/,$//'))
        Get-Process -Name claude -ErrorAction SilentlyContinue | Where-Object {
            \$tracked -notcontains \$_.Id
        } | ForEach-Object { Write-Output \$_.Id }
    " 2>/dev/null)

    if [ -n "$untracked" ]; then
        echo "HYGIENE: Untracked claude.exe processes: $untracked"
        for pid in $untracked; do
            echo "HYGIENE: Terminating stale claude.exe PID $pid"
            # Aggressive dual-kill: POSIX signal + Windows taskkill
            kill -9 $pid 2>/dev/null || true
            taskkill //F //T //PID $pid 2>/dev/null || true
        done
    else
        echo "HYGIENE: No untracked claude.exe processes"
    fi

    # Phase 2: Clean stale IN_PROGRESS claims (expired > 60 min)
    local now=$(date +%s)
    for lockfile in HANDOFF/IN_PROGRESS/*.lock; do
        [ -f "$lockfile" ] || continue
        local deadline=$(grep "^DEADLINE=" "$lockfile" | cut -d'=' -f2)
        if [ -n "$deadline" ] && [ "$now" -gt "$deadline" ]; then
            echo "HYGIENE: Reclaiming stale claim: $lockfile"
            local task_file_stale="${lockfile%.lock}"
            local original_name=$(basename "$task_file_stale" | sed 's/^IN_PROGRESS_//')
            mv "$task_file_stale" "HANDOFF/todo/$original_name" 2>/dev/null || true
            rm -f "$lockfile"
        fi
    done

    # Phase 3: Clean stale compile.lock
    if [ -f ".claude/compile.lock" ]; then
        local lock_pid=$(grep "^PID=" ".claude/compile.lock" | cut -d'=' -f2)
        if [ -n "$lock_pid" ] && ! process_alive "$lock_pid"; then
            local lock_holder=$(grep "^HOLDER=" ".claude/compile.lock" | cut -d'=' -f2)
            echo "HYGIENE: Removing stale compile.lock (holder $lock_holder PID $lock_pid dead)"
            rm -f ".claude/compile.lock"
        fi
    fi

    # Phase 4: Clean stale agent directories
    clean_stale_pids

    # Phase 5: Check for corrupted build artifacts
    if [ -d "target" ]; then
        local corrupt_count=$(find target -name "*.rlib" -size 0 2>/dev/null | wc -l | tr -d ' ')
        if [ "$corrupt_count" -gt 0 ]; then
            echo "HYGIENE: Found $corrupt_count zero-byte rlib files -- running cargo clean"
            cargo clean 2>/dev/null
        else
            echo "HYGIENE: Build directory healthy"
        fi
    fi

    echo "=== Pre-Launch Hygiene Complete ==="
    echo ""

    pool_launch "$agent_name" "$task_file"
}

# ─── Batched Verification ─────────────────────────────────────────────────────

pool_batch_verify() {
    echo "=== Batched Verification ==="

    local pending_crates=""
    for agent_dir in "$AGENT_ROOT"/*/; do
        [ -d "$agent_dir" ] || continue
        local pv_file="$agent_dir/PENDING_VERIFY"
        if [ -f "$pv_file" ]; then
            local scope=$(grep "^SCOPE=" "$pv_file" | cut -d'=' -f2)
            if [ -n "$scope" ]; then
                pending_crates="$pending_crates $scope"
            fi
            rm -f "$pv_file"
        fi
    done

    if [ -z "$pending_crates" ]; then
        echo "No pending verifications. Running full workspace check."
        export PATH="/c/msys64/ucrt64/bin:$PATH"
        cargo check --workspace 2>&1
        return $?
    fi

    local check_args=""
    for crate in $(echo "$pending_crates" | tr ' ' '\n' | sort -u); do
        [ -n "$crate" ] && check_args="$check_args -p $crate"
    done

    echo "Running targeted check:$check_args"
    export PATH="/c/msys64/ucrt64/bin:$PATH"
    cargo check $check_args 2>&1
    local check_result=$?

    echo "Running full workspace verification..."
    cargo check --workspace 2>&1
    local ws_result=$?

    if [ $ws_result -eq 0 ]; then
        echo "BATCH VERIFY: PASS"
    else
        echo "BATCH VERIFY: FAIL"
    fi

    return $ws_result
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

# ─── Patrol: Check completions, free slots, launch next tasks ──────────────

pool_patrol() {
    local actions_taken=0

    echo "=== Patrol Scan ==="

    # Phase 1: Check for completion markers
    for agent_dir in "$AGENT_ROOT"/*/; do
        [ -d "$agent_dir" ] || continue
        local agent_id=$(basename "$agent_dir")
        local completion_file="$agent_dir/COMPLETION"

        if [ -f "$completion_file" ]; then
            local status=$(grep "^STATUS=" "$completion_file" | cut -d'=' -f2)
            local next_requested=$(grep "^NEXT_TASK_REQUESTED=" "$completion_file" | cut -d'=' -f2)
            local task_file=$(grep "^TASK_FILE=" "$completion_file" | cut -d'=' -f2-)
            local build_status=$(grep "^BUILD_STATUS=" "$completion_file" | cut -d'=' -f2)

            case "$status" in
                "completed")
                    if [ "$next_requested" = "false" ]; then
                        echo "PATROL: Agent $agent_id COMPLETED (build: $build_status). Freeing slot."
                        pool_stop "$agent_id"
                        ((actions_taken++))
                    else
                        echo "PATROL: Agent $agent_id COMPLETED and requesting next task. Clearing marker."
                        rm -f "$completion_file"
                        ((actions_taken++))
                    fi
                    ;;
                "failed")
                    echo "PATROL: Agent $agent_id FAILED. Stopping and re-queuing."
                    local error=$(grep "^ERROR=" "$completion_file" | cut -d'=' -f2-)
                    echo "  Error: $error"
                    pool_stop "$agent_id"
                    # Re-queue task if found
                    if [ -n "$task_file" ] && [ -f "$task_file" ]; then
                        local task_name=$(basename "$task_file" | sed 's/^IN_PROGRESS_//')
                        mv "$task_file" "HANDOFF/todo/$task_name" 2>/dev/null || true
                    fi
                    ((actions_taken++))
                    ;;
            esac
        fi
    done

    # Phase 2: Clean stale PIDs
    clean_stale_pids

    # Phase 3: Check for stale claims (IN_PROGRESS older than 60 min)
    local now=$(date +%s)
    local stale_threshold=3600
    for lockfile in HANDOFF/IN_PROGRESS/*.lock; do
        [ -f "$lockfile" ] || continue
        local deadline=$(grep "^DEADLINE=" "$lockfile" | cut -d'=' -f2)
        if [ -n "$deadline" ] && [ "$now" -gt "$deadline" ]; then
            echo "PATROL: Stale claim detected: $lockfile"
            local task_file="${lockfile%.lock}"
            local agent_id=$(grep "^AGENT_ID=" "$lockfile" | cut -d'=' -f2)
            # Check if agent is still alive
            if [ -n "$agent_id" ] && [ -f "$AGENT_ROOT/$agent_id/pid" ]; then
                local agent_pid=$(cat "$AGENT_ROOT/$agent_id/pid")
                if process_alive "$agent_pid"; then
                    echo "  Agent $agent_id still alive past deadline. Consider stopping."
                else
                    echo "  Agent $agent_id dead. Reclaiming task."
                    local original_name=$(basename "$task_file" | sed 's/^IN_PROGRESS_//')
                    mv "$task_file" "HANDOFF/todo/$original_name" 2>/dev/null || true
                    rm -f "$lockfile"
                    ((actions_taken++))
                fi
            fi
        fi
    done

    # Phase 4: Launch new agents if slots available
    local active=$(count_active_agents)
    if [ "$active" -lt "$MAX_SUBAGENTS" ]; then
        local available=$((MAX_SUBAGENTS - active))
        local todo_count=$(ls HANDOFF/todo/*.md 2>/dev/null | grep -v IN_PROGRESS | wc -l)
        if [ "$todo_count" -gt 0 ]; then
            echo "PATROL: $available slot(s) available, $todo_count task(s) in todo queue."
        else
            echo "PATROL: $available slot(s) available but no tasks in todo."
        fi
    fi

    # Phase 5: Summary
    active=$(count_active_agents)
    local done_count=$(ls HANDOFF/done/*.md 2>/dev/null | wc -l)
    local todo_count=$(ls HANDOFF/todo/*.md 2>/dev/null | wc -l)
    local in_progress_count=$(ls HANDOFF/IN_PROGRESS/*.md 2>/dev/null | wc -l)
    echo ""
    echo "PATROL COMPLETE: $actions_taken action(s) taken"
    echo "Slots: $active/$MAX_SUBAGENTS | Tasks: $todo_count todo | $in_progress_count in-progress | $done_count done"
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
        export CARGO_INCREMENTAL=0
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
            "patrol")
                pool_patrol
                ;;
            "launch-clean")
                pool_launch_clean "$3" "$4"
                ;;
            "batch-verify")
                pool_batch_verify
                ;;
            *)
                echo "Pool Commands:"
                echo "  pool list              Show all agent profiles"
                echo "  pool launch <name>     Spin up an agent from the pool"
                echo "  pool launch-clean <n>  Spin up with hygiene check first"
                echo "  pool stop <agent_id>   Spin down an agent"
                echo "  pool status            Show active agents and slot usage"
                echo "  pool patrol            Check completions, free slots"
                echo "  pool batch-verify      Run batched verification"
                echo "  pool patrol            Check completions, free slots, launch next tasks"
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