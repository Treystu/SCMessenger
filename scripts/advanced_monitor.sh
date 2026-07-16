#!/bin/bash

# Advanced monitoring script with centralized logging and health checks

set -e

# Source cross-platform process helpers
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../.claude/scripts/process_alive.sh"

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

    if ! process_alive "$agent_pid"; then
        log_message "ERROR" "Agent $agent_type (PID: $agent_pid) is not running"
        return 1
    fi

    # Check memory usage (returns KB via PowerShell)
    local mem_usage=$(process_memory_kb "$agent_pid")
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

    # Check model availability (local models only — cloud models are always available if service is up)
    local models=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name' 2>/dev/null || echo "")
    local local_count=$(echo "$models" | grep -v ':cloud$' | grep -c '.' 2>/dev/null || echo "0")
    local cloud_count=$(echo "$models" | grep -c ':cloud$' 2>/dev/null || echo "0")

    if [ -z "$models" ] || [ "$local_count" -eq 0 ] && [ "$cloud_count" -eq 0 ]; then
        log_message "WARN" "No models loaded in Ollama (neither local nor cloud)"
    elif [ "$local_count" -eq 0 ]; then
        log_message "INFO" "No local models loaded, but cloud models available (${cloud_count} cloud models detected)"
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

        # Check active agents using PowerShell (tasklist/wmic are deprecated on Windows 11)
        local agent_pids=$(powershell.exe -NoProfile -Command "(Get-Process -Name claude -ErrorAction SilentlyContinue).Id" 2>/dev/null || echo "")

        for pid in $agent_pids; do
            # Get agent type from command line using PowerShell (wmic is deprecated)
            local cmdline=$(powershell.exe -NoProfile -Command "(Get-Process -Id $pid -ErrorAction SilentlyContinue).CommandLine" 2>/dev/null || echo "")
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