#!/bin/bash

# Dynamic resource management and scaling

set -e

CONFIG_DIR=".claude"
LOG_DIR=".claude/logs"
STATE_DIR=".claude/state"

mkdir -p "$LOG_DIR" "$STATE_DIR"

# Load configuration with jq
load_config() {
    if [ -f "$CONFIG_DIR/orchestration_config.json" ]; then
        "./scripts/jq_wrapper.sh" -c . "$CONFIG_DIR/orchestration_config.json" 2>/dev/null || echo "{}"
    else
        echo "{}"
    fi
}

# System monitoring functions (Windows compatible)
get_cpu_usage() {
    # Get CPU usage percentage for Windows
    local cpu_usage=$(wmic cpu get loadpercentage 2>/dev/null | awk 'NR==2 {print $1}')
    echo "${cpu_usage:-0}"
}

get_memory_usage() {
    # Get memory usage percentage for Windows
    local mem_info=$(wmic OS get FreePhysicalMemory,TotalVisibleMemorySize /value 2>/dev/null)
    local free_mem=$(echo "$mem_info" | grep FreePhysicalMemory | cut -d'=' -f2)
    local total_mem=$(echo "$mem_info" | grep TotalVisibleMemorySize | cut -d'=' -f2)

    if [ -n "$free_mem" ] && [ -n "$total_mem" ]; then
        local used_mem=$((total_mem - free_mem))
        local mem_usage=$((used_mem * 100 / total_mem))
        echo "$mem_usage"
    else
        echo "0"
    fi
}

get_disk_usage() {
    # Get .claude directory usage in KB
    du -s "$CONFIG_DIR" 2>/dev/null | cut -f1 || echo "0"
}

get_agent_count() {
    # Count running claude.exe processes for Windows
    tasklist 2>/dev/null | grep -c "claude.exe" || echo "0"
}

# Resource-based scaling decisions
should_scale_up() {
    local cpu_usage="$1"
    local mem_usage="$2"
    local current_agents="$3"

    local config=$(load_config)
    # Extract thresholds with defaults
    local cpu_threshold=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.resource_management.cpu_threshold // 80')
    local mem_threshold=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.resource_management.memory_threshold // 85')
    local max_agents=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.resource_management.max_agents // 3')

    if [ "$current_agents" -ge "$max_agents" ]; then
        return 1  # Already at max
    fi

    if [ "$cpu_usage" -lt "$cpu_threshold" ] && [ "$mem_usage" -lt "$mem_threshold" ]; then
        return 0  # Resources available, can scale up
    fi

    return 1  # Resources constrained
}

should_scale_down() {
    local cpu_usage="$1"
    local mem_usage="$2"
    local current_agents="$3"

    local config=$(load_config)
    # Extract thresholds with defaults
    local cpu_threshold=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.resource_management.cpu_threshold // 80')
    local mem_threshold=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.resource_management.memory_threshold // 85')
    local min_agents=$(echo "$config" | grep -o '"min_agents"\([^,]*\)' | grep -o '[0-9]\+' || echo "1")

    if [ "$current_agents" -le "$min_agents" ]; then
        return 1  # Already at min
    fi

    if [ "$cpu_usage" -gt "$((cpu_threshold + 10))" ] || [ "$mem_usage" -gt "$((mem_threshold + 5))" ]; then
        return 0  # Resources constrained, should scale down
    fi

    return 1  # Resources adequate
}

# Priority-based scheduling
get_task_priority() {
    local task_file="$1"

    # Extract priority from task file
    if grep -q "Priority: P0" "$task_file" 2>/dev/null; then
        echo "0"  # Highest priority
    elif grep -q "Priority: P1" "$task_file" 2>/dev/null; then
        echo "1"
    elif grep -q "Priority: P2" "$task_file" 2>/dev/null; then
        echo "2"
    else
        echo "99"  # Default/lowest priority
    fi
}

prioritize_tasks() {
    local todo_dir="HANDOFF/todo"

    if [ ! -d "$todo_dir" ]; then
        return
    fi

    # Create prioritized list
    local prioritized_tasks=""
    for task_file in "$todo_dir"/*.md; do
        if [ -f "$task_file" ]; then
            local priority=$(get_task_priority "$task_file")
            prioritized_tasks="$prioritized_tasks$priority:$task_file\n"
        fi
    done

    # Sort by priority and extract filenames
    echo -e "$prioritized_tasks" | sort -n | cut -d':' -f2-
}

# Agent management
can_start_agent() {
    local agent_type="$1"
    local current_agents="$2"

    local config=$(load_config)
    # Extract agent limits with defaults
    local max_concurrent=$(echo "$config" | grep -o '"max_concurrent"\([^,]*\)' | grep -o '[0-9]\+' || echo "3")
    local max_per_type=$(echo "$config" | grep -o "\"max_per_type.$agent_type\"\([^,]*\)" | grep -o '[0-9]\+' || echo "1")

    # Count current agents of this type
    local current_of_type=0
    local agent_pids=$(tasklist //fi "IMAGENAME eq claude.exe" //fo csv 2>/dev/null | grep -v "," | cut -d',' -f2 | tr -d '"' | grep -v PID || echo "")

    for pid in $agent_pids; do
        local cmdline=$(wmic process where "processid=$pid" get commandline //value 2>/dev/null | grep -i "commandline" | cut -d'=' -f2-)
        if echo "$cmdline" | grep -qi "$agent_type"; then
            current_of_type=$((current_of_type + 1))
        fi
    done

    if [ "$current_agents" -lt "$max_concurrent" ] && [ "$current_of_type" -lt "$max_per_type" ]; then
        return 0
    fi

    return 1
}

# Main resource management loop
manage_resources() {
    local check_interval=60

    while true; do
        # Get current system state
        local cpu_usage=$(get_cpu_usage)
        local mem_usage=$(get_memory_usage)
        local disk_usage=$(get_disk_usage)
        local agent_count=$(get_agent_count)

        # Log current state
        echo "Resource State: CPU=${cpu_usage}%, MEM=${mem_usage}%, DISK=${disk_usage}KB, AGENTS=${agent_count}"

        # Make scaling decisions
        if should_scale_up "$cpu_usage" "$mem_usage" "$agent_count"; then
            echo "Scaling UP: Resources available, can start more agents"
            # Implementation would start agents here
        fi

        if should_scale_down "$cpu_usage" "$mem_usage" "$agent_count"; then
            echo "Scaling DOWN: Resources constrained, should reduce agents"
            # Implementation would stop agents here
        fi

        # Check disk usage
        local config=$(load_config)
        local disk_limit=$(echo "$config" | grep -o '"disk_usage_limit_mb"\([^,]*\)' | grep -o '[0-9]\+' || echo "100")
        disk_limit=$((disk_limit * 1024))  # Convert MB to KB

        if [ "$disk_usage" -gt "$disk_limit" ]; then
            echo "WARNING: Disk usage ${disk_usage}KB exceeds limit ${disk_limit}KB"
            # Implementation would trigger cleanup
        fi

        sleep $check_interval
    done
}

# Command line interface
case "${1:-}" in
    "--start")
        manage_resources
        ;;
    "--status")
        echo "CPU Usage: $(get_cpu_usage)%"
        echo "Memory Usage: $(get_memory_usage)%"
        echo "Disk Usage: $(get_disk_usage)KB"
        echo "Active Agents: $(get_agent_count)"
        ;;
    "--prioritize")
        prioritize_tasks
        ;;
    "--check-scale")
        cpu=$(get_cpu_usage)
        mem=$(get_memory_usage)
        agents=$(get_agent_count)

        echo "Current: CPU=${cpu}%, MEM=${mem}%, AGENTS=${agents}"

        if should_scale_up "$cpu" "$mem" "$agents"; then
            echo "Decision: SCALE UP"
        elif should_scale_down "$cpu" "$mem" "$agents"; then
            echo "Decision: SCALE DOWN"
        else
            echo "Decision: MAINTAIN CURRENT"
        fi
        ;;
    *)
        echo "Usage: $0 [--start|--status|--prioritize|--check-scale]"
        echo "  --start        Start resource management daemon"
        echo "  --status       Show current resource usage"
        echo "  --prioritize   Show prioritized task list"
        echo "  --check-scale  Check scaling decisions"
        exit 1
        ;;
esac