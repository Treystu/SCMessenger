#!/bin/bash
# launch_agent.sh - Launch autonomous sub-agent with specific model
# Part of SCMessenger Agent Monitoring System

set -euo pipefail

# Configuration
AGENT_MODEL="${1:-glm-5.1:cloud}"
AGENT_ID="${2:-agent_$(date +%s)}"
OLLAMA_HOST="${OLLAMA_HOST:-localhost:11434}"
AGENT_WORK_DIR=".claude/agents/$AGENT_ID"
AGENT_LOG="$AGENT_WORK_DIR/agent.log"
AGENT_PID="$AGENT_WORK_DIR/pid"

# Agent prompt instructions
AGENT_PROMPT="You are an SCMessenger Autonomous Sub-Agent. Use the /loop 5m command to check the HANDOFF/todo/ directory. If you see a task file, claim it by renaming it to IN_PROGRESS_[filename].md. Execute the required code changes, run the local compilers to verify your work, and upon success, use bash to move the file to HANDOFF/done/. If you fail after 3 attempts, append your error logs and move it back to HANDOFF/todo/. Do not stop."

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

    # Create a persistent stdin pipe so the backgrounded Claude process
    # doesn't immediately exit when the parent script's stdin closes.
    # The pipe delivers the initial prompt, then stays open via tail -f /dev/null.
    local stdin_pipe="$AGENT_WORK_DIR/stdin_pipe"
    rm -f "$stdin_pipe"
    mkfifo "$stdin_pipe" 2>/dev/null || true

    # Write the initial prompt to the pipe, then keep it open indefinitely
    # so the Claude process never sees EOF on stdin.
    (echo "$AGENT_PROMPT"; exec tail -f /dev/null) > "$stdin_pipe" &
    local writer_pid=$!
    echo "$writer_pid" > "$AGENT_WORK_DIR/writer_pid"

    # Launch via ollama launch wrapper for Ollama Cloud model routing
    # --dangerously-skip-permissions for autonomous operation
    # stdin comes from the persistent pipe so the process stays alive
    ollama launch claude --model "$AGENT_MODEL" \
        -- --dangerously-skip-permissions \
        < "$stdin_pipe" \
        >> "$AGENT_LOG" 2>&1 &

    local agent_pid=$!
    echo "$agent_pid" > "$AGENT_PID"
    disown $agent_pid 2>/dev/null
    disown $writer_pid 2>/dev/null

    log "INFO" "Agent started with PID: $agent_pid (stdin writer PID: $writer_pid)"

    # Verify agent is running
    sleep 3
    if kill -0 "$agent_pid" 2>/dev/null; then
        log "INFO" "Agent process verified as running"
        log "INFO" "Agent process started successfully - prompt delivered via persistent stdin pipe"
        return 0
    else
        log "ERROR" "Agent process failed to start"
        # Clean up writer if agent died
        kill "$writer_pid" 2>/dev/null || true
        rm -f "$stdin_pipe"
        return 1
    fi
}

monitor_agent() {
    local agent_pid=$(cat "$AGENT_PID")
    local check_interval=30
    local consecutive_failures=0
    local max_failures=3

    log "INFO" "Starting agent monitoring for PID: $agent_pid"

    while true; do
        # Check if agent process is still running
        if ! kill -0 "$agent_pid" 2>/dev/null; then
            ((consecutive_failures++))
            log "WARN" "Agent process not running ($consecutive_failures/$max_failures)"

            if [ $consecutive_failures -ge $max_failures ]; then
                log "ERROR" "Agent process failed, attempting restart..."
                if restart_agent; then
                    consecutive_failures=0
                    agent_pid=$(cat "$AGENT_PID")
                else
                    log "ERROR" "Agent restart failed"
                    return 1
                fi
            fi
        else
            consecutive_failures=0
            log "DEBUG" "Agent process healthy"
        fi

        sleep $check_interval
    done
}

restart_agent() {
    log "INFO" "Restarting agent $AGENT_ID"

    # Stop existing agent
    if [ -f "$AGENT_PID" ]; then
        local old_pid=$(cat "$AGENT_PID")
        if kill -0 "$old_pid" 2>/dev/null; then
            kill "$old_pid" 2>/dev/null || true
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

    # Kill the stdin writer process if it exists
    if [ -f "$AGENT_WORK_DIR/writer_pid" ]; then
        local writer_pid=$(cat "$AGENT_WORK_DIR/writer_pid")
        if kill -0 "$writer_pid" 2>/dev/null; then
            kill "$writer_pid" 2>/dev/null || true
            log "INFO" "Stdin writer stopped (PID: $writer_pid)"
        fi
        rm -f "$AGENT_WORK_DIR/writer_pid"
    fi

    # Clean up the named pipe
    rm -f "$AGENT_WORK_DIR/stdin_pipe"

    if [ -f "$AGENT_PID" ]; then
        local agent_pid=$(cat "$AGENT_PID")
        if kill -0 "$agent_pid" 2>/dev/null; then
            kill "$agent_pid"
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
        if kill -0 "$agent_pid" 2>/dev/null; then
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
    echo "Usage: launch_agent.sh [MODEL] [AGENT_ID] {start|stop|restart|status|--help}"
    echo ""
    echo "Launch autonomous sub-agent with specific Ollama model"
    echo ""
    echo "Arguments:"
    echo "  MODEL      Ollama model name (default: glm-5.1:cloud)"
    echo "  AGENT_ID   Unique agent identifier (default: agent_TIMESTAMP)"
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
case "${3:-start}" in
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