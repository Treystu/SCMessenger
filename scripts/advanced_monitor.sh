#!/bin/bash

# Advanced monitoring script with centralized logging and health checks

set -e

LOG_DIR=".claude/logs"
CONFIG_DIR=".claude"
MAX_LOG_FILES=30
CHECK_INTERVAL=30

# Create log directory if it doesn't exist
mkdir -p "$LOG_DIR"

# Set up log rotation
rotate_logs() {
    local log_file="$1"
    if [ -f "$log_file" ] && [ $(wc -l < "$log_file") -gt 1000 ]; then
        mv "$log_file" "${log_file}.$(date +%Y%m%d_%H%M%S)"
    fi

    # Clean up old log files
    find "$LOG_DIR" -name "*.log.*" -mtime +7 -delete
    find "$LOG_DIR" -name "*.log.*" | sort -r | tail -n +$MAX_LOG_FILES | xargs rm -f 2>/dev/null || true
}

# Centralized logging function
log_message() {
    local level="$1"
    local message="$2"
    local log_file="$LOG_DIR/orchestration.log"

    rotate_logs "$log_file"
    echo "$(date '+%Y-%m-%d %H:%M:%S') [$level] $message" >> "$log_file"
    echo "[$level] $message"
}

# Health check functions
check_agent_health() {
    local agent_pid="$1"
    local agent_type="$2"

    if ! ps -p "$agent_pid" > /dev/null 2>&1; then
        log_message "ERROR" "Agent $agent_type (PID: $agent_pid) is not running"
        return 1
    fi

    # Check memory usage
    local mem_usage=$(ps -o rss= -p "$agent_pid" 2>/dev/null || echo "0")
    if [ "$mem_usage" -gt 2000000 ]; then  # 2GB limit
        log_message "WARN" "Agent $agent_type (PID: $agent_pid) high memory usage: ${mem_usage}KB"
    fi

    return 0
}

check_ollama_health() {
    if ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
        log_message "ERROR" "Ollama service is not responding"
        return 1
    fi

    # Check model availability
    local models=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name' 2>/dev/null || echo "")
    if [ -z "$models" ]; then
        log_message "WARN" "No models loaded in Ollama"
    fi

    return 0
}

check_disk_usage() {
    local usage=$(du -s .claude/ 2>/dev/null | cut -f1 || echo "0")
    if [ "$usage" -gt 100000 ]; then  # 100MB limit
        log_message "WARN" "High disk usage in .claude/: ${usage}KB"
        return 1
    fi
    return 0
}

# Main monitoring loop
main() {
    log_message "INFO" "Starting advanced monitoring system"

    while true; do
        # Check Ollama service
        if ! check_ollama_health; then
            log_message "ERROR" "Ollama health check failed"
        fi

        # Check disk usage
        check_disk_usage

        # Check active agents
        local agent_pids=$(tasklist //fi "IMAGENAME eq claude.exe" //fo csv 2>/dev/null | grep -v "," | cut -d',' -f2 | tr -d '"' | grep -v PID || echo "")

        for pid in $agent_pids; do
            # Get agent type from command line
            local cmdline=$(wmic process where "processid=$pid" get commandline //value 2>/dev/null | grep -i "commandline" | cut -d'=' -f2-)
            local agent_type="unknown"

            if echo "$cmdline" | grep -qi "gatekeeper"; then
                agent_type="gatekeeper"
            elif echo "$cmdline" | grep -qi "coder"; then
                agent_type="coder"
            elif echo "$cmdline" | grep -qi "orchestrat"; then
                agent_type="orchestrator"
            fi

            check_agent_health "$pid" "$agent_type"
        done

        sleep $CHECK_INTERVAL
    done
}

# Handle command line arguments
case "${1:-}" in
    "--test")
        echo "Testing monitoring functions..."
        check_ollama_health
        check_disk_usage
        echo "Test completed successfully"
        ;;
    "--start")
        main
        ;;
    *)
        echo "Usage: $0 [--test|--start]"
        echo "  --test   Test monitoring functions"
        echo "  --start  Start continuous monitoring"
        exit 1
        ;;
esac