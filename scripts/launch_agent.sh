#!/bin/bash
# launch_agent.sh - Launch autonomous sub-agent with specific model
# Part of SCMessenger Agent Monitoring System

set -euo pipefail

# Source cross-platform process helpers
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../.claude/scripts/process_alive.sh"

# Configuration
AGENT_MODEL="${1:-glm-5.1:cloud}"
AGENT_ID="${2:-agent_$(date +%s)}"
AGENT_TASK_FILE="${3:-}"
AGENT_COMMAND="${4:-start}"
OLLAMA_HOST="${OLLAMA_HOST:-localhost:11434}"
AGENT_WORK_DIR=".claude/agents/$AGENT_ID"
AGENT_LOG="$AGENT_WORK_DIR/agent.log"
AGENT_STDERR="$AGENT_WORK_DIR/stderr.log"
AGENT_PID="$AGENT_WORK_DIR/pid"

# Detect file domains for this agent from agent_pool.json
AGENT_FILE_DOMAINS=""
if [ -f ".claude/agent_pool.json" ]; then
    AGENT_NAME_FROM_POOL=$(echo "$AGENT_ID" | sed 's/_[0-9]*$//')
    if command -v jq > /dev/null 2>&1; then
        AGENT_FILE_DOMAINS=$(jq -r --arg name "$AGENT_NAME_FROM_POOL" \
            '.agents[] | select(.name == $name) | .file_domains[] // empty' \
            .claude/agent_pool.json 2>/dev/null | tr '\n' ',' | sed 's/,$//')
    elif command -v python > /dev/null 2>&1 || command -v python3 > /dev/null 2>&1; then
        PYTHON_CMD=$(command -v python3 > /dev/null 2>&1 && echo "python3" || echo "python")
        AGENT_FILE_DOMAINS=$($PYTHON_CMD -c "
import json,sys
d=json.load(open('.claude/agent_pool.json'))
matches=[a for a in d['agents'] if a['name']==sys.argv[1]]
if matches and matches[0].get('file_domains'):
    print(','.join(matches[0]['file_domains']))
" "$AGENT_NAME_FROM_POOL" 2>/dev/null)
    fi
fi

# Build agent prompt — include task file content if provided
AGENT_PROMPT="You are an SCMessenger Autonomous Sub-Agent (ID: $AGENT_ID, Model: $AGENT_MODEL).

## TASK WORKFLOW
1. Claim a task from HANDOFF/todo/ by moving the file to HANDOFF/IN_PROGRESS/IN_PROGRESS_<filename>.md
2. Implement the required code changes.
3. Verify: run cargo check --workspace (or gradle for Android).
4. On success: move the task file to HANDOFF/done/<filename>
5. On failure after 3 attempts: move the task file back to HANDOFF/todo/ with error notes appended.
6. After completing your task, pick the next file from HANDOFF/todo/ and repeat.

## COMPILE LOCK PROTOCOL (MANDATORY)
Multiple agents may be running concurrently. To avoid build conflicts:
1. BEFORE running any cargo command (check, build, test, clippy, fmt), acquire the compile lock:
   - Create .claude/compile.lock containing: AGENT_ID=<your_id>, PID=<your_bash_pid>, ACQUIRED=<epoch>
   - If .claude/compile.lock exists and the owning PID is alive, WAIT (sleep 30, retry). Do NOT proceed.
   - If .claude/compile.lock exists but the owning PID is dead, the lock is stale — remove it and acquire.
   - The lock has a 10-minute maximum hold time. If held longer, it is stale — remove and acquire.
2. AFTER your cargo command completes (success or failure), RELEASE the compile lock:
   - Delete .claude/compile.lock
3. NEVER hold the compile lock while doing non-compilation work (editing files, reading code).
4. For file edits and reads that do NOT involve cargo, no lock is needed.

## FILE DOMAIN RESPECT (MANDATORY)
Your assigned file domains: ${AGENT_FILE_DOMAINS:-unrestricted}
- ONLY edit files within your assigned file domains.
- If a task requires editing a file outside your domains, SKIP that part and note it in the COMPLETION marker under BLOCKED_FOR_DOMAIN.
- Reading files outside your domains is allowed and encouraged for context.
- NEVER edit files claimed by another agent (check .claude/agents/*/COMPLETION for CHANGED_FILES).

## COMPLETION PROTOCOL (MANDATORY)
After completing OR failing a task, you MUST write a completion marker to .claude/agents/$AGENT_ID/COMPLETION:

On success:
STATUS=completed
TASK_FILE=HANDOFF/done/<task_filename>
CHANGED_FILES=<comma-separated list of files modified>
BUILD_STATUS=pass
COMPLETED_AT=<epoch timestamp>
NEXT_TASK_REQUESTED=true

On failure:
STATUS=failed
TASK_FILE=<current location of the task file>
ERROR=<brief description of the failure>
COMPLETED_AT=<epoch timestamp>

When you have no more tasks and no task in progress, set NEXT_TASK_REQUESTED=false.

## MANDATORY EXIT PROTOCOL
Once you have moved all task files to HANDOFF/done/ and written your COMPLETION marker, you MUST type /exit to terminate your process immediately. Do NOT remain in interactive mode after task completion — this blocks the orchestrator from detecting completion and launching the next agent.

## SELF-PACING
Use /loop 5m to self-pace ONLY while actively working. Once all tasks are complete, /exit immediately."

if [ -n "$AGENT_TASK_FILE" ] && [ -f "$AGENT_TASK_FILE" ]; then
    AGENT_PROMPT="You are an SCMessenger Autonomous Sub-Agent (ID: $AGENT_ID, Model: $AGENT_MODEL).

## ASSIGNED TASK
Read and execute the task file below completely. Follow all instructions precisely.

=== ASSIGNED TASK FILE: $AGENT_TASK_FILE ===
$(cat "$AGENT_TASK_FILE")
=== END TASK FILE ===

## COMPILE LOCK PROTOCOL (MANDATORY)
Multiple agents may be running concurrently. To avoid build conflicts:
1. BEFORE running any cargo command (check, build, test, clippy, fmt), acquire the compile lock:
   - Create .claude/compile.lock containing: AGENT_ID=<your_id>, PID=<your_bash_pid>, ACQUIRED=<epoch>
   - If .claude/compile.lock exists and the owning PID is alive, WAIT (sleep 30, retry). Do NOT proceed.
   - If .claude/compile.lock exists but the owning PID is dead, the lock is stale — remove and acquire.
   - The lock has a 10-minute maximum hold time. If held longer, it is stale — remove and acquire.
2. AFTER your cargo command completes (success or failure), RELEASE the compile lock:
   - Delete .claude/compile.lock
3. NEVER hold the compile lock while doing non-compilation work (editing files, reading code).
4. For file edits and reads that do NOT involve cargo, no lock is needed.

## FILE DOMAIN RESPECT (MANDATORY)
Your assigned file domains: ${AGENT_FILE_DOMAINS:-unrestricted}
- ONLY edit files within your assigned file domains.
- If a task requires editing a file outside your domains, SKIP that part and note it in the COMPLETION marker under BLOCKED_FOR_DOMAIN.
- Reading files outside your domains is allowed and encouraged for context.
- NEVER edit files claimed by another agent (check .claude/agents/*/COMPLETION for CHANGED_FILES).

## COMPLETION PROTOCOL (MANDATORY)
After completing OR failing this task, you MUST write a completion marker to .claude/agents/$AGENT_ID/COMPLETION:

On success:
STATUS=completed
TASK_FILE=HANDOFF/done/<task_filename>
CHANGED_FILES=<comma-separated list of files modified>
BUILD_STATUS=pass
COMPLETED_AT=<epoch timestamp>
NEXT_TASK_REQUESTED=true

On failure:
STATUS=failed
TASK_FILE=<current location of the task file>
ERROR=<brief description of the failure>
COMPLETED_AT=<epoch timestamp>

Use /loop 5m to self-pace ONLY while actively working. Once all tasks are complete, /exit immediately."
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    # Ensure log directory exists before writing
    mkdir -p "$(dirname "$AGENT_LOG")"

    case $level in
        "INFO") echo -e "${GREEN}[INFO]${NC} $timestamp - $message" ;;
        "WARN") echo -e "${YELLOW}[WARN]${NC} $timestamp - $message" ;;
        "ERROR") echo -e "${RED}[ERROR]${NC} $timestamp - $message" ;;
    esac

    echo "[$level] $timestamp - $message" >> "$AGENT_LOG"
}

check_ollama_model() {
    log "INFO" "Checking if model $AGENT_MODEL is available..."

    # Cloud models (:cloud suffix) are always available if Ollama is running
    # They are hosted remotely by Ollama Cloud, not pulled locally
    if echo "$AGENT_MODEL" | grep -q ":cloud$"; then
        log "INFO" "Model $AGENT_MODEL is an Ollama Cloud model — skipping local check"
        # Just verify Ollama service is reachable
        if curl -s "http://$OLLAMA_HOST/api/tags" >/dev/null 2>&1; then
            log "INFO" "Ollama service reachable, cloud model $AGENT_MODEL available"
            return 0
        else
            log "ERROR" "Ollama service not reachable at $OLLAMA_HOST"
            return 1
        fi
    fi

    local available_models
    if ! available_models=$(list_available_models); then
        log "ERROR" "Failed to get available models list"
        return 1
    fi

    if echo "$available_models" | grep -q "^$AGENT_MODEL$"; then
        log "INFO" "Model $AGENT_MODEL is available"
        return 0
    else
        log "ERROR" "Model $AGENT_MODEL is not available"
        return 1
    fi
}

list_available_models() {
    local response
    response=$(curl -s "http://$OLLAMA_HOST/api/tags" 2>/dev/null || true)

    if [ -z "$response" ]; then
        log_error "Failed to list models - empty response from Ollama"
        return 1
    fi

    # Try to parse with jq if available, otherwise use grep
    if command -v jq > /dev/null 2>&1; then
        if ! echo "$response" | jq -e '.models' > /dev/null 2>&1; then
            log_error "Invalid JSON response from Ollama"
            return 1
        fi
        echo "$response" | jq -r '.models[].name'
    else
        # Fallback: extract model names with grep/sed
        echo "$response" | grep -o '"name"\s*:\s*"[^"]*"' | sed 's/"name"\s*:\s*"\([^"]*\)"/\1/'
    fi
}

setup_agent_environment() {
    log "INFO" "Setting up agent environment for $AGENT_ID"

    # Create working directory
    mkdir -p "$AGENT_WORK_DIR"

    # Create agent configuration
    cat > "$AGENT_WORK_DIR/config" <<EOF
AGENT_ID=$AGENT_ID
AGENT_MODEL=$AGENT_MODEL
OLLAMA_HOST=$OLLAMA_HOST
WORKING_DIR=$(pwd)
START_TIME=$(date +%s)
FILE_DOMAINS=${AGENT_FILE_DOMAINS:-unrestricted}
EOF

    # Record the assigned task file for completion detection
    if [ -n "$AGENT_TASK_FILE" ] && [ -f "$AGENT_TASK_FILE" ]; then
        echo "$AGENT_TASK_FILE" > "$AGENT_WORK_DIR/task_file"
    fi

    log "INFO" "Agent environment setup complete"
}

start_agent() {
    log "INFO" "Starting agent $AGENT_ID with model $AGENT_MODEL"

    # Launch Claude Code in interactive mode with --dangerously-skip-permissions.
    # Pipe the agent prompt via stdin so Claude enters interactive (not --print) mode.
    # The prompt includes /loop 5m for persistent autonomous operation.
    # This keeps the process alive instead of one-shot exit.
    # Separate stderr to avoid Windows STATUS_BREAKPOINT (0x80000003) corrupting the log.
    export CARGO_INCREMENTAL=0
    echo "$AGENT_PROMPT" | ollama launch claude --model "$AGENT_MODEL" \
        -- --dangerously-skip-permissions \
        >> "$AGENT_LOG" 2>"$AGENT_STDERR" &

    local agent_pid=$!
    echo "$agent_pid" > "$AGENT_PID"
    disown $agent_pid 2>/dev/null

    log "INFO" "Agent started with PID: $agent_pid"

    # Verify agent is running
    sleep 3
    if process_alive "$agent_pid"; then
        log "INFO" "Agent process verified as running"
        # Check for benign Windows startup error
        if grep -q "0x80000003" "$AGENT_STDERR" 2>/dev/null; then
            log "INFO" "Windows STATUS_BREAKPOINT detected in startup (benign on Windows, process is running)"
        fi
        log "INFO" "Agent process started successfully - interactive mode with piped prompt"
        return 0
    else
        log "ERROR" "Agent process failed to start"
        return 1
    fi
}

monitor_agent() {
    local agent_pid=$(cat "$AGENT_PID")
    local check_interval=30
    local consecutive_failures=0
    local max_failures=3
    local max_runtime_minutes=120  # Hard timeout: kill after 2 hours
    local start_time=$(date +%s)

    log "INFO" "Starting agent monitoring for PID: $agent_pid (max runtime: ${max_runtime_minutes}m)"

    while true; do
        local now=$(date +%s)
        # Guard against empty/invalid date output on Windows Git Bash
        if [ -z "$now" ] || ! [[ "$now" =~ ^[0-9]+$ ]]; then
            sleep $check_interval
            continue
        fi
        local elapsed_seconds=$((now - start_time))

        # ── COMPLETION CHECK (primary detection) ──
        # Check for COMPLETION marker written by the agent
        if [ -f "$AGENT_WORK_DIR/COMPLETION" ]; then
            local status=$(grep "^STATUS=" "$AGENT_WORK_DIR/COMPLETION" | cut -d'=' -f2)
            log "INFO" "Agent COMPLETION marker found: STATUS=$status"
            # Kill the agent process since it's done
            if process_alive "$agent_pid"; then
                kill -9 "$agent_pid" 2>/dev/null || true
                taskkill //F //T //PID "$agent_pid" 2>/dev/null || true
                log "INFO" "Terminated agent process $agent_pid after completion"
            fi
            return 0
        fi

        # ── TASK-FILE COMPLETION CHECK (secondary detection) ──
        # If the agent's task file has been moved to HANDOFF/done/ and no IN_PROGRESS
        # task remains, the agent is effectively done even without a COMPLETION marker.
        local in_progress_count=$(ls HANDOFF/IN_PROGRESS/*.md 2>/dev/null | grep -v "BATCH_" | wc -l)
        local agent_batch="$AGENT_WORK_DIR/task_file"
        if [ -f "$agent_batch" ]; then
            local task_name=$(cat "$agent_batch" | xargs basename 2>/dev/null)
            if [ -n "$task_name" ] && [ ! -f "HANDOFF/todo/$task_name" ] && [ ! -f "HANDOFF/IN_PROGRESS/$task_name" ]; then
                # Task file is no longer in todo/ or IN_PROGRESS/ — agent must have moved it
                if [ "$in_progress_count" -eq 0 ]; then
                    log "INFO" "Task file $task_name no longer in todo/IN_PROGRESS and no active tasks — agent done"
                    # Write COMPLETION marker on behalf of the agent
                    echo "STATUS=completed" > "$AGENT_WORK_DIR/COMPLETION"
                    echo "TASK_FILE=HANDOFF/done/$task_name" >> "$AGENT_WORK_DIR/COMPLETION"
                    echo "COMPLETED_AT=$(date +%s)" >> "$AGENT_WORK_DIR/COMPLETION"
                    echo "BUILD_STATUS=unknown" >> "$AGENT_WORK_DIR/COMPLETION"
                    echo "NEXT_TASK_REQUESTED=false" >> "$AGENT_WORK_DIR/COMPLETION"
                    if process_alive "$agent_pid"; then
                        kill -9 "$agent_pid" 2>/dev/null || true
                        taskkill //F //T //PID "$agent_pid" 2>/dev/null || true
                        log "INFO" "Terminated agent process $agent_pid (task-file completion detected)"
                    fi
                    return 0
                fi
            fi
        fi

        # ── HARD TIMEOUT ──
        if [ $elapsed_seconds -ge $((max_runtime_minutes * 60)) ]; then
            log "WARN" "Agent monitor timeout after ${max_runtime_minutes}m, terminating agent"
            if process_alive "$agent_pid"; then
                kill -9 "$agent_pid" 2>/dev/null || true
                taskkill //F //T //PID "$agent_pid" 2>/dev/null || true
            fi
            # Write a timeout COMPLETION marker
            echo "STATUS=timeout" > "$AGENT_WORK_DIR/COMPLETION"
            echo "COMPLETED_AT=$(date +%s)" >> "$AGENT_WORK_DIR/COMPLETION"
            echo "ERROR=Agent exceeded ${max_runtime_minutes}m runtime limit" >> "$AGENT_WORK_DIR/COMPLETION"
            return 1
        fi

        # ── PROCESS HEALTH CHECK ──
        if ! process_alive "$agent_pid"; then
            ((consecutive_failures++))
            log "WARN" "Agent process not running ($consecutive_failures/$max_failures)"

            if [ $consecutive_failures -ge $max_failures ]; then
                log "ERROR" "Agent process dead after $max_failures checks, exiting monitor"
                # Write a crash COMPLETION marker
                echo "STATUS=crashed" > "$AGENT_WORK_DIR/COMPLETION"
                echo "COMPLETED_AT=$(date +%s)" >> "$AGENT_WORK_DIR/COMPLETION"
                echo "ERROR=Agent process died unexpectedly" >> "$AGENT_WORK_DIR/COMPLETION"
                return 1
            fi
        else
            consecutive_failures=0
            # Only log every 10th check to reduce noise
            if [ $((elapsed_seconds / check_interval)) -eq 0 ] || [ $((elapsed_seconds % 300)) -lt $check_interval ]; then
                log "INFO" "Agent process healthy ($((elapsed_seconds / 60))m elapsed)"
            fi
        fi

        sleep $check_interval
    done
}

restart_agent() {
    log "INFO" "Restarting agent $AGENT_ID"

    # Stop existing agent
    if [ -f "$AGENT_PID" ]; then
        local old_pid=$(cat "$AGENT_PID")
        if process_alive "$old_pid"; then
            # Aggressive dual-kill: POSIX signal + Windows taskkill
            kill -9 $old_pid 2>/dev/null || true
            taskkill //F //T //PID $old_pid 2>/dev/null || true
        fi
    fi

    # Start new agent (--print mode, non-interactive)
    if start_agent; then
        log "INFO" "Agent restart successful"
        return 0
    else
        log "ERROR" "Agent restart failed"
        return 1
    fi
}

stop_agent() {
    log "INFO" "Stopping agent $AGENT_ID"

    if [ -f "$AGENT_PID" ]; then
        local agent_pid=$(cat "$AGENT_PID")
        if process_alive "$agent_pid"; then
            # Aggressive dual-kill: POSIX signal + Windows taskkill
            kill -9 $agent_pid 2>/dev/null || true
            taskkill //F //T //PID $agent_pid 2>/dev/null || true
            log "INFO" "Agent stopped successfully"
        else
            log "WARN" "Agent process not running"
        fi
        rm -f "$AGENT_PID"
    else
        log "WARN" "No PID file found"
    fi

    # Release compile lock if held by this agent
    if [ -f ".claude/compile.lock" ]; then
        local lock_owner=$(grep "AGENT_ID=" .claude/compile.lock 2>/dev/null | cut -d= -f2)
        if [ "$lock_owner" = "$AGENT_ID" ]; then
            rm -f .claude/compile.lock
            log "INFO" "Released compile lock held by $AGENT_ID"
        fi
    fi

    # Keep log files for analysis
    log "INFO" "Agent logs preserved at: $AGENT_LOG"
}

agent_status() {
    if [ -f "$AGENT_PID" ]; then
        local agent_pid=$(cat "$AGENT_PID")
        if process_alive "$agent_pid"; then
            echo "Agent $AGENT_ID is running (PID: $agent_pid)"
            return 0
        else
            echo "Agent $AGENT_ID PID file exists but process not running"
            return 1
        fi
    else
        echo "Agent $AGENT_ID is not running"
        return 1
    fi
}

# Handle help before argument parsing
if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
    echo "Usage: launch_agent.sh [MODEL] [AGENT_ID] [TASK_FILE] {start|stop|restart|status|--help}"
    echo ""
    echo "Launch autonomous sub-agent with specific Ollama model"
    echo ""
    echo "Arguments:"
    echo "  MODEL      Ollama model name (default: glm-5.1:cloud)"
    echo "  AGENT_ID   Unique agent identifier (default: agent_TIMESTAMP)"
    echo "  TASK_FILE  Optional task file to include in agent prompt"
    echo ""
    echo "Commands:"
    echo "  start     Start the agent"
    echo "  stop      Stop the agent"
    echo "  restart   Restart the agent"
    echo "  status    Check agent status"
    echo "  --help    Show this help message"
    exit 0
fi

# Main execution
case "$AGENT_COMMAND" in
    "start")
        if ! check_ollama_model; then
            log "ERROR" "Cannot start agent - model $AGENT_MODEL not available"
            exit 1
        fi

        if agent_status > /dev/null 2>&1; then
            log "WARN" "Agent $AGENT_ID is already running"
            exit 0
        fi

        setup_agent_environment
        if start_agent; then
            log "INFO" "Agent $AGENT_ID started successfully"
            # Start monitoring in background
            monitor_agent &
            exit 0
        else
            log "ERROR" "Failed to start agent $AGENT_ID"
            exit 1
        fi
        ;;
    "stop")
        stop_agent
        ;;
    "restart")
        stop_agent
        sleep 2
        if start_agent; then
            log "INFO" "Agent $AGENT_ID restarted successfully"
        else
            log "ERROR" "Failed to restart agent $AGENT_ID"
            exit 1
        fi
        ;;
    "status")
        agent_status
        ;;
    *)
        echo "Usage: launch_agent.sh [MODEL] [AGENT_ID] {start|stop|restart|status|--help}"
        exit 1
        ;;
esac