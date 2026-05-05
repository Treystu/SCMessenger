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

# Reliable cross-platform process kill.
# PowerShell Stop-Process is the ONLY method that reliably kills claude.exe on Windows.
force_kill_pid() {
    local pid="$1"
    [ -z "$pid" ] && return 0
    # Precise recursive tree kill via PowerShell — ensures claude.exe children die
    powershell.exe -NoProfile -Command "
        function Kill-Tree([int]\$p) {
            Get-CimInstance Win32_Process -Filter \"ParentProcessId = \$p\" | ForEach-Object { Kill-Tree \$_.ProcessId }
            Stop-Process -Id \$p -Force -ErrorAction SilentlyContinue
        }
        Kill-Tree $pid
    " 2>/dev/null || true
    # Fallbacks
    kill -9 "$pid" 2>/dev/null || true
    taskkill //F //T //PID "$pid" 2>/dev/null || true
}

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
    # Multi-strategy PID discovery for the orchestrator's claude.exe process.
    # Strategy 1: Walk the parent chain from current bash process to find claude.exe.
    # Strategy 2: Find the oldest claude.exe (orchestrator always starts first).
    # Strategy 3: Fallback — mark unknown rather than storing garbage PID 0/1.

    local found_pid=""

    # Strategy 1: Parent-chain walk via PowerShell.
    # $PPID on Windows Git Bash may point to an MSYS2 intermediate; PowerShell
    # can only resolve Windows-native PIDs. Validate PPID before attempting.
    local ppid="$PPID"
    if [ -n "$ppid" ] && [ "$ppid" != "1" ] && [ "$ppid" != "0" ]; then
        found_pid=$(powershell.exe -NoProfile -Command "
            \$p = Get-Process -Id $ppid -ErrorAction SilentlyContinue
            while (\$p -and \$p.ProcessName -ne 'claude') {
                \$parentId = \$p.ParentId
                if (-not \$parentId -or \$parentId -eq 0 -or \$parentId -eq \$p.Id) { break }
                \$p = Get-Process -Id \$parentId -ErrorAction SilentlyContinue
            }
            if (\$p -and \$p.ProcessName -eq 'claude') { Write-Output \$p.Id }
        " 2>/dev/null | tr -d '[:space:]')
    fi

    # Strategy 2: Oldest claude.exe (orchestrator is always the first Claude process)
    if [ -z "$found_pid" ]; then
        found_pid=$(powershell.exe -NoProfile -Command "
            \$procs = @(Get-Process -Name claude -ErrorAction SilentlyContinue | Sort-Object StartTime)
            if (\$procs.Count -gt 0) { Write-Output \$procs[0].Id }
        " 2>/dev/null | tr -d '[:space:]')
    fi

    # Strategy 3: Mark unknown — NEVER store PID 0, 1, or empty
    if [ -z "$found_pid" ] || [ "$found_pid" = "0" ] || [ "$found_pid" = "1" ]; then
        echo "WARNING: Could not determine orchestrator PID. Self-preservation guards disabled."
        found_pid="unknown"
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
                # Precise recursive tree kill to ensure orphaned sub-processes die
                force_kill_pid "$pid"
                rm -rf "$dir"
            fi
        fi
    done
}

# Clean orphaned agent directories when no agents are active.
# Prevents stale directories from causing domain conflicts.
cleanup_orphaned_agent_dirs() {
    local active=$(count_active_agents)
    if [ "$active" -ne 0 ]; then
        return 0
    fi

    if [ ! -d "$AGENT_ROOT" ]; then
        return 0
    fi

    local orphan_count=0
    for dir in "$AGENT_ROOT"/*/; do
        [ -d "$dir" ] || continue
        local agent_id=$(basename "$dir")
        # Skip if PID file has a live process
        if [ -f "$dir/pid" ]; then
            local pid=$(cat "$dir/pid" 2>/dev/null)
            if [ -n "$pid" ] && process_alive "$pid"; then
                continue
            fi
        fi
        echo "CLEANUP: Removing orphaned agent directory: $agent_id"
        rm -rf "$dir"
        ((orphan_count++))
    done

    if [ "$orphan_count" -gt 0 ]; then
        echo "CLEANUP: Removed $orphan_count orphaned agent directories"
    fi
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

    # ─── TASK VALIDATION GATE ────────────────────────────────────────────────
    if [ -n "$task_file" ] && [ -f "$task_file" ]; then
        local validation_result
        validation_result=$(validate_task_before_launch "$task_file")
        local validation_exit=$?

        case $validation_exit in
            1)  # FALSE_POSITIVE
                echo "TASK VALIDATION: $validation_result"
                echo "Moving false-positive task to done/ with note."
                local task_basename=$(basename "$task_file")
                echo "" >> "$task_file"
                echo "## VALIDATION GATE: FALSE_POSITIVE" >> "$task_file"
                echo "Reason: $validation_result" >> "$task_file"
                echo "Date: $(date -u '+%Y-%m-%dT%H:%M:%SZ')" >> "$task_file"
                mv "$task_file" "HANDOFF/done/$task_basename" 2>/dev/null || true
                return 1
                ;;
            2)  # ALREADY_WIRED
                echo "TASK VALIDATION: $validation_result"
                echo "Moving already-wired task to done/ with note."
                local task_basename=$(basename "$task_file")
                echo "" >> "$task_file"
                echo "## VALIDATION GATE: ALREADY_WIRED" >> "$task_file"
                echo "Reason: $validation_result" >> "$task_file"
                echo "Date: $(date -u '+%Y-%m-%dT%H:%M:%SZ')" >> "$task_file"
                mv "$task_file" "HANDOFF/done/$task_basename" 2>/dev/null || true
                return 1
                ;;
            3)  # NEEDS_REVIEW
                echo "TASK VALIDATION: $validation_result — needs human review, skipping."
                return 1
                ;;
        esac
        echo "TASK VALIDATION: PASS ($validation_result)"
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

    # === REPO_MAP: JIT micro-update only ===
    # Full rebuild removed - the Freshness Gate below handles per-task
    # targeted re-indexing of ONLY the files referenced in the task file.
    # This prevents the orchestrator from blocking on a full repo scan.

    # === REPO_MAP Freshness Gate ===
    if [ -n "$task_file" ] && [ -f "$task_file" ]; then
        # Determine if this is a micro-task that doesn't need full context
        local is_micro_task=false
        if [[ "$task_file" == *"TRIAGE"* ]] || [[ "$task_file" == *"QUICK"* ]] || [[ "$task_file" == *"LINT"* ]] || [[ "$task_file" == *"FMT"* ]]; then
            is_micro_task=true
        fi

        # For non-micro tasks, run full context injection
        if [ "$is_micro_task" = "false" ]; then
            echo "Running REPO_MAP Freshness Gate..."
            local freshness_result
            freshness_result=$($PYTHON .claude/scripts/freshness_gate.py --task-file "$task_file" 2>&1)
            local freshness_exit=$?

            if [ $freshness_exit -eq 2 ]; then
                # STALE files detected — trigger targeted re-index
                local stale_files=$($PYTHON -c "
import json, sys
try:
    result = json.loads(sys.argv[1])
    stale = [f.get('path', '') for f in result.get('stale_files', [])]
    missing = result.get('missing_files', [])
    print(','.join(stale + missing))
except Exception as e:
    pass
" "$freshness_result")

                if [ -n "$stale_files" ]; then
                    echo "FRESHNESS GATE: Stale files detected. Triggering targeted re-index..."
                    echo "Files: $stale_files"

                    powershell.exe -NoProfile -ExecutionPolicy Bypass \
                        -File ".claude/scripts/targeted_reindex.ps1" \
                        -Files "$stale_files"

                    if [ $? -ne 0 ]; then
                        echo "WARNING: Targeted re-index failed. Launching agent without fresh context."
                    fi
                fi
            fi

            # Inject context for all FRESH files
            echo "Injecting REPO_MAP context into task..."
            $PYTHON .claude/scripts/context_extractor.py --task-file "$task_file"
        else
            echo "Skipping full REPO_MAP context injection for micro-task: $task_file"
        fi
    fi

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
        # Record start time for timeout tracking
        echo "$(date +%s)" > "$AGENT_ROOT/$agent_id/start_time"
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

        # Record start time for timeout tracking
        echo "$(date +%s)" > "$AGENT_ROOT/$agent_id/start_time"

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
COMPILE_LOCK_TIMEOUT=900  # 15 minutes max hold time (accommodates slow Windows cargo builds)

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

# ─── Pre-Dispatch Task Validation Gate ─────────────────────────────────────────
# Validates that a task file targets a real production-callable function,
# not a false positive (test, kani proof, proptest harness, string constant).
# Delegates to .claude/scripts/task_validator.py for pattern detection.
# Return codes: 0=VALID, 1=FALSE_POSITIVE, 2=ALREADY_WIRED, 3=NEEDS_REVIEW

validate_task_before_launch() {
    local task_file="$1"

    if [ -z "$task_file" ] || [ ! -f "$task_file" ]; then
        echo "VALID"
        return 0
    fi

    # Only validate task_wire_* files (skip BATCH_, ORCHESTRATOR_, etc.)
    local basename=$(basename "$task_file")
    if [[ "$basename" != task_wire_* ]]; then
        echo "VALID"
        return 0
    fi

    # Extract TARGET line and function name
    local target_line=$(grep "^TARGET:" "$task_file" | head -1)
    if [ -z "$target_line" ]; then
        echo "VALID"
        return 0
    fi

    local target_file=$(echo "$target_line" | sed 's/^TARGET:[[:space:]]*//' | tr '\' '/')
    local func_name=$(echo "$basename" | sed 's/^task_wire_//' | sed 's/\.md$//')

    if [ ! -f "$target_file" ]; then
        echo "NEEDS_REVIEW:target_file_missing:$target_file"
        return 3
    fi

    # Delegate to standalone Python validation script
    local validation
    validation=$($PYTHON "$SCRIPT_DIR/scripts/task_validator.py" \
        "$task_file" "$target_file" "$func_name" 2>/dev/null)
    local validation_exit=$?

    case $validation_exit in
        1)  echo "FALSE_POSITIVE:$validation"
            return 1
            ;;
        2)  echo "ALREADY_WIRED:$validation"
            return 2
            ;;
        3)  echo "NEEDS_REVIEW:$validation"
            return 3
            ;;
        *)  echo "VALID"
            return 0
            ;;
    esac
}

# ─── Pre-Launch Hygiene ──────────────────────────────────────────────────────

pool_launch_clean() {
    local agent_name="${1:-}"
    local task_file="${2:-}"

    echo "=== Pre-Launch Hygiene Check ==="

    # Phase 1: Determine tracked PIDs (orchestrator + agents)
    local orch_pid=$(get_orchestrator_pid)
    local tracked_pids=""

    # Add orchestrator PID if known (not "unknown")
    if [ -n "$orch_pid" ] && [ "$orch_pid" != "unknown" ]; then
        tracked_pids=" $orch_pid"
    fi

    # Add all known agent PIDs
    for pidfile in "$AGENT_ROOT"/*/pid; do
        if [ -f "$pidfile" ]; then
            tracked_pids="$tracked_pids $(cat "$pidfile")"
        fi
    done

    # Self-preservation: discover ALL claude.exe processes that host our infrastructure.
    # Add the parent-chain ancestor AND the oldest claude.exe to the tracked set.
    local host_claude_pid=""
    local ppid="$PPID"
    if [ -n "$ppid" ] && [ "$ppid" != "1" ] && [ "$ppid" != "0" ]; then
        host_claude_pid=$(powershell.exe -NoProfile -Command "
            \$p = Get-Process -Id $ppid -ErrorAction SilentlyContinue
            while (\$p -and \$p.ProcessName -ne 'claude') {
                \$parentId = \$p.ParentId
                if (-not \$parentId -or \$parentId -eq 0 -or \$parentId -eq \$p.Id) { break }
                \$p = Get-Process -Id \$parentId -ErrorAction SilentlyContinue
            }
            if (\$p -and \$p.ProcessName -eq 'claude') { Write-Output \$p.Id }
        " 2>/dev/null | tr -d '[:space:]')
    fi
    if [ -n "$host_claude_pid" ]; then
        tracked_pids="$tracked_pids $host_claude_pid"
    fi

    # Also add the oldest claude.exe as a safety net
    local oldest_claude=$(powershell.exe -NoProfile -Command "
        \$procs = @(Get-Process -Name claude -ErrorAction SilentlyContinue | Sort-Object StartTime)
        if (\$procs.Count -gt 0) { Write-Output \$procs[0].Id }
    " 2>/dev/null | tr -d '[:space:]')
    if [ -n "$oldest_claude" ]; then
        tracked_pids="$tracked_pids $oldest_claude"
    fi

    # Untracked-process kill: ONLY proceed if we have a valid orchestrator PID.
    # If orchestrator PID is "unknown", skip this phase entirely to avoid self-kill.
    if [ "$orch_pid" = "unknown" ] || [ -z "$orch_pid" ]; then
        echo "HYGIENE: Orchestrator PID unknown — skipping untracked-process cleanup (safety first)."
    else
        local untracked=$(powershell.exe -NoProfile -Command "
            \$tracked = @($(echo "$tracked_pids" | tr ' ' ',' | sed 's/^,//' | sed 's/,$//'))
            Get-Process -Name claude -ErrorAction SilentlyContinue | Where-Object {
                \$tracked -notcontains \$_.Id
            } | ForEach-Object { Write-Output \$_.Id }
        " 2>/dev/null)

        if [ -n "$untracked" ]; then
            echo "HYGIENE: Untracked claude.exe processes: $untracked"
            for pid in $untracked; do
                # FINAL SAFETY CHECK: never kill a PID that is in our tracked set
                if echo "$tracked_pids" | grep -qw "$pid"; then
                    echo "HYGIENE: SKIPPING PID $pid (in tracked set — safety guard tripped)"
                    continue
                fi
                echo "HYGIENE: Terminating stale claude.exe PID $pid"
                kill -9 $pid 2>/dev/null || true
                taskkill //F //T //PID $pid 2>/dev/null || true
            done
        else
            echo "HYGIENE: No untracked claude.exe processes"
        fi
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

    # Phase 1: Check for completion markers AND zombie detection
    for agent_dir in "$AGENT_ROOT"/*/; do
        [ -d "$agent_dir" ] || continue
        local agent_id=$(basename "$agent_dir")
        local completion_file="$agent_dir/COMPLETION"
        local task_file_marker="$agent_dir/task_file"
        local pid_file="$agent_dir/pid"
        local start_time_file="$agent_dir/start_time"

        # ── TIMEOUT DETECTION ──
        # Aggressively timeout agents that have been running too long
        if [ -f "$start_time_file" ]; then
            local start_time=$(cat "$start_time_file")
            local current_time=$(date +%s)
            local elapsed=$((current_time - start_time))
            local timeout_limit=5400  # 90 minutes timeout for most tasks

            # Check if this is a micro-task that should have shorter timeout
            if [ -f "$task_file_marker" ]; then
                local assigned_task=$(cat "$task_file_marker")
                # If task is a triage or quick fix, use shorter timeout
                if [[ "$assigned_task" == *"TRIAGE"* ]] || [[ "$assigned_task" == *"QUICK"* ]] || [[ "$assigned_task" == *"LINT"* ]]; then
                    timeout_limit=1200  # 20 minutes for micro-tasks
                fi
            fi

            if [ "$elapsed" -gt "$timeout_limit" ]; then
                echo "PATROL: Agent $agent_id timed out after $elapsed seconds. Stopping."
                echo "STATUS=timeout" > "$completion_file"
                echo "ERROR=Agent exceeded timeout limit of $timeout_limit seconds" >> "$completion_file"
                echo "BUILD_STATUS=unknown" >> "$completion_file"
                echo "NEXT_TASK_REQUESTED=false" >> "$completion_file"
                pool_stop "$agent_id"
                ((actions_taken++))
                continue
            fi
        fi

        # ── COMPLETION MARKER DETECTION ──
        if [ -f "$completion_file" ]; then
            local status=$(grep "^STATUS=" "$completion_file" | cut -d'=' -f2)
            local next_requested=$(grep "^NEXT_TASK_REQUESTED=" "$completion_file" | cut -d'=' -f2)
            local task_file=$(grep "^TASK_FILE=" "$completion_file" | cut -d'=' -f2-)
            local build_status=$(grep "^BUILD_STATUS=" "$completion_file" | cut -d'=' -f2)

            case "$status" in
                "completed"|"timeout"|"crashed")
                    if [ "$next_requested" = "false" ] || [ "$status" = "timeout" ] || [ "$status" = "crashed" ]; then
                        echo "PATROL: Agent $agent_id $status (build: $build_status). Freeing slot."
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
            continue
        fi

        # ── ZOMBIE DETECTION (agent alive but task file moved to done/ without COMPLETION) ──
        # A zombie is an agent whose task is in done/ (work completed), has NO COMPLETION
        # marker (protocol violation), AND has been idle (no log output) for 10+ minutes.
        # All three conditions must hold — this prevents false-positive kills during races.
        if [ -f "$task_file_marker" ] && [ ! -f "$completion_file" ]; then
            local assigned_task=$(cat "$task_file_marker")
            local task_basename=$(basename "$assigned_task" | sed 's/^IN_PROGRESS_//')
            # Condition 1: Task file must actually be in done/ (agent genuinely finished work)
            if [ -f "HANDOFF/done/$task_basename" ]; then
                # Condition 2: Check agent log idle time (last modification)
                local log_file="$agent_dir/agent.log"
                local idle_minutes=0
                if [ -f "$log_file" ]; then
                    local log_mtime=$(stat -c %Y "$log_file" 2>/dev/null || echo "0")
                    if [ -n "$log_mtime" ] && [ "$log_mtime" != "0" ]; then
                        idle_minutes=$(( (current_time - log_mtime) / 60 ))
                    fi
                fi
                # Condition 3: Agent must have been idle for 10+ minutes
                if [ "$idle_minutes" -ge 10 ]; then
                    echo "PATROL: Agent $agent_id is a zombie — task '$task_basename' in done/, no COMPLETION marker, idle ${idle_minutes}m"
                    # Kill the zombie process if still alive
                    if [ -f "$pid_file" ]; then
                        local pid=$(cat "$pid_file")
                        if process_alive "$pid"; then
                            echo "PATROL: Agent $agent_id process $pid still alive — terminating zombie"
                            kill -9 "$pid" 2>/dev/null || true
                            taskkill //F //T //PID "$pid" 2>/dev/null || true
                        fi
                    fi
                    # Write completion marker on behalf of the agent
                    echo "STATUS=completed" > "$completion_file"
                    echo "COMPLETED_AT=$(date +%s)" >> "$completion_file"
                    echo "BUILD_STATUS=unknown" >> "$completion_file"
                    echo "NEXT_TASK_REQUESTED=false" >> "$completion_file"
                    echo "NOTE=Auto-detected by patrol (zombie: task in done/, no marker, idle ${idle_minutes}m)" >> "$completion_file"
                    pool_stop "$agent_id"
                    ((actions_taken++))
                fi
            fi
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

    # Phase 5: Batch micro-tasks if in conservation mode
    local five_hour_usage=$(grep "5-Hour Usage" .claude/API_QUOTA_STATE.md | cut -d':' -f2 | tr -d '%' | tr -d ' ')
    if [ -n "$five_hour_usage" ] && [ "$(echo "$five_hour_usage > 75" | bc -l 2>/dev/null || echo "0")" -eq 1 ]; then
        echo "PATROL: High quota usage detected ($five_hour_usage%). Consolidating micro-tasks..."
        python3 .claude/scripts/batch_micro_tasks.py 2>/dev/null || python .claude/scripts/batch_micro_tasks.py 2>/dev/null || py -3 .claude/scripts/batch_micro_tasks.py 2>/dev/null || true
    fi

    # Phase 6: Summary
    active=$(count_active_agents)
    local done_count=$(ls HANDOFF/done/*.md 2>/dev/null | wc -l)
    local todo_count=$(ls HANDOFF/todo/*.md 2>/dev/null | wc -l)
    local in_progress_count=$(ls HANDOFF/IN_PROGRESS/*.md 2>/dev/null | wc -l)
    local batch_count=$(ls HANDOFF/batches/*.md 2>/dev/null | wc -l 2>/dev/null || echo "0")
    echo ""
    echo "PATROL COMPLETE: $actions_taken action(s) taken"
    echo "Slots: $active/$MAX_SUBAGENTS | Tasks: $todo_count todo | $in_progress_count in-progress | $done_count done | $batch_count batches"
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