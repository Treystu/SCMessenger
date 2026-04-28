#!/bin/bash

# Session log auditor for Full-Agent activity monitoring

LOG_DIR=".claude/logs"
AGENT_DIR=".claude/agents"

# Create directories if they don't exist
mkdir -p "$LOG_DIR" "$AGENT_DIR"

# Audit agent session logs
audit_agent_logs() {
    echo "=== Full-Agent Session Log Audit ==="
    echo ""

    # Check active agents
    local active_agents=$(tasklist 2>/dev/null | grep -c "claude.exe" || echo "0")
    echo "Active Full-Agents: $active_agents"

    # Check agent directories
    local agent_dirs=$(find "$AGENT_DIR" -maxdepth 1 -type d -name "*_*" | wc -l)
    echo "Agent directories: $agent_dirs"
    echo ""

    # Audit each agent directory
    for agent_dir in "$AGENT_DIR"/*_*/; do
        if [ -d "$agent_dir" ]; then
            local agent_name=$(basename "$agent_dir")
            local log_file="$agent_dir/agent.log"

            echo "Agent: $agent_name"

            if [ -f "$log_file" ]; then
                local last_activity=$(grep -E "(INFO|WARN|ERROR)" "$log_file" | tail -1 || echo "No activity found")
                local log_size=$(wc -l < "$log_file" 2>/dev/null | tr -d ' ' || echo "0")

                echo "  Log entries: $log_size"
                echo "  Last activity: $last_activity"

                # Check for errors
                local error_count=$(grep -c "ERROR" "$log_file" 2>/dev/null || echo "0")
                if [ "$error_count" -gt 0 ]; then
                    echo "  ❌ Errors found: $error_count"
                else
                    echo "  ✅ No errors"
                fi

                # Check for recent activity (last 5 minutes)
                if find "$log_file" -newermt "5 minutes ago" > /dev/null 2>&1; then
                    echo "  ✅ Active (updated in last 5 minutes)"
                else
                    echo "  ⚠️  Inactive (no recent updates)"
                fi
            else
                echo "  ❌ No log file found"
            fi
            echo ""
        fi
    done

    # Check centralized logs
    if [ -f "$LOG_DIR/orchestration.log" ]; then
        echo "=== Centralized Log Summary ==="
        tail -10 "$LOG_DIR/orchestration.log" | while read line; do
            echo "  $line"
        done
    else
        echo "No centralized logs found"
    fi
}

# Check specific agent health
check_agent_health() {
    local agent_name="$1"
    local agent_dir="$AGENT_DIR/${agent_name}"

    if [ -d "$agent_dir" ]; then
        echo "Health check for $agent_name:"

        # Check if agent process is running
        local agent_pid=""
        if [ -f "$agent_dir/pid" ]; then
            agent_pid=$(cat "$agent_dir/pid" 2>/dev/null)
            if ps -p "$agent_pid" > /dev/null 2>&1; then
                echo "  ✅ Process running (PID: $agent_pid)"
            else
                echo "  ❌ Process not running (PID: $agent_pid)"
            fi
        else
            echo "  ⚠️  No PID file found"
        fi

        # Check log activity
        if [ -f "$agent_dir/agent.log" ]; then
            local last_activity=$(stat -c %y "$agent_dir/agent.log" 2>/dev/null || stat -f %Sm "$agent_dir/agent.log" 2>/dev/null)
            echo "  Log last updated: $last_activity"
        fi
    else
        echo "Agent $agent_name not found"
    fi
}

# Main function
main() {
    case "${1:-}" in
        "--health")
            check_agent_health "${2:-}"
            ;;
        "--audit")
            audit_agent_logs
            ;;
        *)
            echo "Usage: $0 [--audit|--health AGENT_NAME]"
            echo "  --audit          Full audit of all agent sessions"
            echo "  --health AGENT   Health check for specific agent"
            exit 1
            ;;
    esac
}

main "$@"