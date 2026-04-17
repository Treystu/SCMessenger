#!/bin/bash
# agent_monitor.sh - Monitor and manage multiple autonomous agents
# Part of SCMessenger Agent Monitoring System

set -euo pipefail

# Configuration
AGENTS_DIR=".claude/agents"
CHECK_INTERVAL="${CHECK_INTERVAL:-60}" # seconds
MAX_AGENTS="${MAX_AGENTS:-2}" # Maximum number of concurrent agents
REQUIRED_MODELS=(
    "glm-5.1:cloud"
    "qwen3-coder:480b:cloud"
)

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

    case $level in
        "INFO") echo -e "${GREEN}[INFO]${NC} $timestamp - $message" ;;
        "WARN") echo -e "${YELLOW}[WARN]${NC} $timestamp - $message" ;;
        "ERROR") echo -e "${RED}[ERROR]${NC} $timestamp - $message" ;;
        "DEBUG") echo -e "${BLUE}[DEBUG]${NC} $timestamp - $message" ;;
    esac
}

get_running_agents() {
    local running_agents=()

    if [ -d "$AGENTS_DIR" ]; then
        for agent_dir in "$AGENTS_DIR"/*/; do
            if [ -d "$agent_dir" ]; then
                local agent_id=$(basename "$agent_dir")
                local pid_file="$agent_dir/pid"

                if [ -f "$pid_file" ]; then
                    local pid=$(cat "$pid_file")
                    if kill -0 "$pid" 2>/dev/null; then
                        running_agents+=("$agent_id")
                    fi
                fi
            fi
        done
    fi

    echo "${running_agents[*]}"
}

get_agent_count() {
    local running_agents
    running_agents=$(get_running_agents)
    echo "${#running_agents[@]}"
}

start_agent() {
    local model=$1
    local agent_id="agent_$(date +%s)_$(echo "$model" | tr -d ':')"

    log "INFO" "Starting agent $agent_id with model $model"

    if ./scripts/launch_agent.sh "$model" "$agent_id" start; then
        log "INFO" "Agent $agent_id started successfully"
        return 0
    else
        log "ERROR" "Failed to start agent $agent_id"
        return 1
    fi
}

stop_agent() {
    local agent_id=$1

    log "INFO" "Stopping agent $agent_id"

    if ./scripts/launch_agent.sh "" "$agent_id" stop; then
        log "INFO" "Agent $agent_id stopped successfully"
        return 0
    else
        log "ERROR" "Failed to stop agent $agent_id"
        return 1
    fi
}

restart_agent() {
    local agent_id=$1

    log "INFO" "Restarting agent $agent_id"

    if ./scripts/launch_agent.sh "" "$agent_id" restart; then
        log "INFO" "Agent $agent_id restarted successfully"
        return 0
    else
        log "ERROR" "Failed to restart agent $agent_id"
        return 1
    fi
}

ensure_agent_count() {
    local current_count=$(get_agent_count)
    local needed_count=$((MAX_AGENTS - current_count))

    log "DEBUG" "Current agents: $current_count, Needed: $needed_count"

    if [ $needed_count -le 0 ]; then
        log "DEBUG" "Sufficient agents running ($current_count/$MAX_AGENTS)"
        return 0
    fi

    log "INFO" "Need to start $needed_count additional agent(s)"

    # Start needed agents with round-robin model assignment
    for ((i=0; i<needed_count; i++)); do
        local model_index=$((i % ${#REQUIRED_MODELS[@]}))
        local model="${REQUIRED_MODELS[$model_index]}"

        if start_agent "$model"; then
            log "INFO" "Successfully started agent with model $model"
        else
            log "ERROR" "Failed to start agent with model $model"
        fi
    done
}

check_agent_health() {
    local agent_id=$1
    local agent_dir="$AGENTS_DIR/$agent_id"
    local pid_file="$agent_dir/pid"
    local log_file="$agent_dir/agent.log"

    if [ ! -f "$pid_file" ]; then
        log "WARN" "Agent $agent_id has no PID file"
        return 1
    fi

    local pid=$(cat "$pid_file")

    # Check if process is running
    if ! kill -0 "$pid" 2>/dev/null; then
        log "WARN" "Agent $agent_id process not running (PID: $pid)"
        return 1
    fi

    # Check if agent is responsive (optional - could check log activity)
    local log_size=$(wc -l < "$log_file" 2>/dev/null || echo 0)
    local last_activity=$(stat -c %Y "$log_file" 2>/dev/null || echo 0)
    local current_time=$(date +%s)
    local inactivity_period=$((current_time - last_activity))

    if [ $inactivity_period -gt 300 ]; then # 5 minutes inactivity
        log "WARN" "Agent $agent_id inactive for ${inactivity_period}s"
        return 1
    fi

    log "DEBUG" "Agent $agent_id healthy (PID: $pid, log lines: $log_size)"
    return 0
}

monitor_agents() {
    local cycle_count=0

    log "INFO" "Starting agent monitoring (max agents: $MAX_AGENTS, interval: ${CHECK_INTERVAL}s)"

    while true; do
        ((cycle_count++))
        log "DEBUG" "Monitoring cycle $cycle_count started"

        # Get current running agents
        local running_agents
        running_agents=($(get_running_agents))
        local current_count=${#running_agents[@]}

        log "INFO" "Current agents running: $current_count/$MAX_AGENTS"

        # Check health of each agent
        local unhealthy_agents=()
        for agent_id in "${running_agents[@]}"; do
            if ! check_agent_health "$agent_id"; then
                unhealthy_agents+=("$agent_id")
            fi
        done

        # Restart unhealthy agents
        for agent_id in "${unhealthy_agents[@]}"; do
            log "WARN" "Restarting unhealthy agent: $agent_id"
            if restart_agent "$agent_id"; then
                log "INFO" "Agent $agent_id restarted successfully"
            else
                log "ERROR" "Failed to restart agent $agent_id"
            fi
        done

        # Ensure we have the required number of agents
        ensure_agent_count

        log "DEBUG" "Monitoring cycle $cycle_count completed"
        sleep $CHECK_INTERVAL
    done
}

start_monitor() {
    log "INFO" "Starting agent monitor service"

    # Create agents directory
    mkdir -p "$AGENTS_DIR"

    # Start monitoring loop
    monitor_loop
}

stop_monitor() {
    log "INFO" "Stopping agent monitor service"

    # Stop all running agents
    local running_agents
    running_agents=($(get_running_agents))

    for agent_id in "${running_agents[@]}"; do
        stop_agent "$agent_id"
    done

    log "INFO" "All agents stopped"
}

status() {
    local running_agents
    running_agents=($(get_running_agents))
    local count=${#running_agents[@]}

    echo "Agent Monitor Status:"
    echo "Running agents: $count/$MAX_AGENTS"

    if [ $count -gt 0 ]; then
        echo "Active agents:"
        for agent_id in "${running_agents[@]}"; do
            local agent_dir="$AGENTS_DIR/$agent_id"
            local pid_file="$agent_dir/pid"
            local model="unknown"

            if [ -f "$agent_dir/config" ]; then
                model=$(grep "^AGENT_MODEL=" "$agent_dir/config" | cut -d= -f2)
            fi

            if [ -f "$pid_file" ]; then
                local pid=$(cat "$pid_file")
                echo "  - $agent_id (PID: $pid, Model: $model)"
            else
                echo "  - $agent_id (No PID, Model: $model)"
            fi
        done
    fi
}

# Main execution
case "${1:-}" in
    "start")
        start_monitor
        ;;
    "stop")
        stop_monitor
        ;;
    "restart")
        stop_monitor
        sleep 2
        start_monitor
        ;;
    "status")
        status
        ;;
    "-h" | "--help")
        echo "Usage: agent_monitor.sh {start|stop|restart|status|--help}"
        echo ""
        echo "Monitor and manage multiple autonomous agents"
        echo ""
        echo "Commands:"
        echo "  start     Start the agent monitor service"
        echo "  stop      Stop the agent monitor service"
        echo "  restart   Restart the agent monitor service"
        echo "  status    Show agent status"
        echo "  --help    Show this help message"
        ;;
    *)
        echo "Usage: agent_monitor.sh {start|stop|restart|status|--help}"
        exit 1
        ;;
esac