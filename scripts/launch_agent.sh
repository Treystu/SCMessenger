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

# Build agent prompt — include task file content if provided
AGENT_PROMPT="You are an SCMessenger Autonomous Sub-Agent. Your task is in HANDOFF/todo/. Claim it by moving the file to HANDOFF/IN_PROGRESS/IN_PROGRESS_[filename].md. Implement the required code changes, verify with cargo build or gradle, then move to HANDOFF/done/. If you fail after 3 attempts, move it back to HANDOFF/todo/ with error logs appended. After completing your task, pick the next file from HANDOFF/todo/ and repeat. Use /loop 5m to self-pace."

if [ -n "$AGENT_TASK_FILE" ] && [ -f "$AGENT_TASK_FILE" ]; then
    AGENT_PROMPT="You are an SCMessenger Autonomous Sub-Agent. You have been assigned a specific task. Read and execute the task file below completely. Follow all instructions precisely. After completing your task, move it to the appropriate HANDOFF directory. Use /loop 5m to self-pace.

=== ASSIGNED TASK FILE: $AGENT_TASK_FILE ===
$(cat "$AGENT_TASK_FILE")
=== END TASK FILE ==="
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
EOF

    log "INFO" "Agent environment setup complete"
}

start_agent() {
    log "INFO" "Starting agent $AGENT_ID with model $AGENT_MODEL"

    # Launch Claude Code in interactive mode with --dangerously-skip-permissions.
    # Pipe the agent prompt via stdin so Claude enters interactive (not --print) mode.
    # The prompt includes /loop 5m for persistent autonomous operation.
    # This keeps the process alive instead of one-shot exit.
    # Separate stderr to avoid Windows STATUS_BREAKPOINT (0x80000003) corrupting the log.
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
    local max_runtime_minutes=30
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

        # Auto-stop after max runtime to prevent zombie monitors
        if [ $elapsed_seconds -ge $((max_runtime_minutes * 60)) ]; then
            log "INFO" "Agent monitor timeout after ${max_runtime_minutes}m, exiting monitor"
            return 0
        fi

        # Check if agent process is still running
        if ! process_alive "$agent_pid"; then
            ((consecutive_failures++))
            log "WARN" "Agent process not running ($consecutive_failures/$max_failures)"

            if [ $consecutive_failures -ge $max_failures ]; then
                log "ERROR" "Agent process dead after $max_failures checks, exiting monitor"
                return 1
            fi
        else
            consecutive_failures=0
            log "DEBUG" "Agent process healthy (${elapsed_seconds}s elapsed)"
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
            powershell.exe -NoProfile -Command "Stop-Process -Id $old_pid -Force" 2>/dev/null || true
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
            powershell.exe -NoProfile -Command "Stop-Process -Id $agent_pid -Force" 2>/dev/null || true
            log "INFO" "Agent stopped successfully"
        else
            log "WARN" "Agent process not running"
        fi
        rm -f "$AGENT_PID"
    else
        log "WARN" "No PID file found"
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