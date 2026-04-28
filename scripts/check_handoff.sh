#!/bin/bash
# SCMessenger Autonomous Sub-Agent Loop Script
# Checks HANDOFF/todo/ directory for tasks, claims them, processes them.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HANDOFF_TODO="$PROJECT_ROOT/HANDOFF/todo"
HANDOFF_DONE="$PROJECT_ROOT/HANDOFF/done"
STATE_DIR="$PROJECT_ROOT/.claude/task_state"
ORCHESTRATOR_MANAGER="$PROJECT_ROOT/.claude/orchestrator_manager.sh"
AGENT_POOL_CONFIG="$PROJECT_ROOT/.claude/agent_pool.json"
VERIFY_SCRIPT="$PROJECT_ROOT/scripts/verify_task_completion.sh"

# Dry run: set to 1 to only print actions, not execute
DRY_RUN=${DRY_RUN:-0}

# Ensure state directory exists
mkdir -p "$STATE_DIR"

log() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "[$level] $timestamp - $message"
}

dry_run_log() {
    if [ "$DRY_RUN" -eq 1 ]; then
        log "DRY" "$1"
    fi
}

# Check if orchestrator manager exists
if [ ! -f "$ORCHESTRATOR_MANAGER" ]; then
    log "ERROR" "Orchestrator manager not found at $ORCHESTRATOR_MANAGER"
    exit 1
fi

# Check if agent pool config exists
if [ ! -f "$AGENT_POOL_CONFIG" ]; then
    log "ERROR" "Agent pool config not found at $AGENT_POOL_CONFIG"
    exit 1
fi

# Function to get task pattern matches
match_task_pattern() {
    local task_file=$1
    local task_name=$(basename "$task_file" .md)
    local task_content=$(cat "$task_file")
    # Extract keywords from filename and content
    local keywords=$(echo "$task_name $task_content" | grep -o -E '[A-Z]+_[A-Z]+' | tr '[:lower:]' '[:upper:]' | sort -u)
    echo "$keywords"
}

# Function to select agent based on task patterns
select_agent() {
    local task_file=$1
    local task_name=$(basename "$task_file" .md)
    local task_content=$(cat "$task_file")
    # Extract platform line
    local platform=$(echo "$task_content" | grep -i "^Platform:" | head -1 | cut -d: -f2- | xargs)
    # Extract priority line
    local priority=$(echo "$task_content" | grep -i "^Priority:" | head -1 | cut -d: -f2- | xargs)
    # Extract keywords from filename and content
    local keywords=$(echo "$task_name $task_content" | grep -o -E '[A-Z]+_[A-Z]+' | tr '[:lower:]' '[:upper:]' | sort -u)
    # Load agent pool config and match against task_patterns
    local pool_path="$AGENT_POOL_CONFIG"
    pool_path="$(cd "$(dirname "$pool_path")" && pwd)/$(basename "$pool_path")"
    if [ ! -f "$pool_path" ]; then
        log "ERROR" "Agent pool config not found at $pool_path"
        echo "implementer"
        return
    fi
    local agent=$(python -c "
import json, sys, re
try:
    with open('$pool_path') as f:
        pool = json.load(f)
except Exception as e:
    sys.stderr.write('Error loading agent pool config: ' + str(e) + '\n')
    sys.exit(1)
keywords = sys.argv[1].split()
platform = sys.argv[2]
priority = sys.argv[3]
for agent in pool.get('agents', []):
    patterns = agent.get('task_patterns', [])
    for pat in patterns:
        pat_upper = pat.upper()
        for kw in keywords:
            if pat_upper in kw or kw in pat_upper:
                print(agent['name'])
                sys.exit(0)
# fallback based on platform
if platform:
    platform_lower = platform.lower()
    if 'rust' in platform_lower or 'core' in platform_lower:
        print('rust-coder')
        sys.exit(0)
    elif 'android' in platform_lower or 'ios' in platform_lower:
        print('platform-engineer')
        sys.exit(0)
    elif 'security' in platform_lower or 'crypto' in platform_lower:
        print('security-auditor')
        sys.exit(0)
print('implementer')  # default agent
" "$keywords" "$platform" "$priority" 2>/dev/null)
    if [ $? -ne 0 ] || [ -z "$agent" ]; then
        log "ERROR" "Failed to select agent, defaulting to implementer"
        echo "implementer"
    else
        echo "$agent"
    fi
}

# Function to claim a task
claim_task() {
    local task_file=$1
    local task_name=$(basename "$task_file")
    local in_progress_name="IN_PROGRESS_${task_name}"
    local in_progress_path="$HANDOFF_TODO/$in_progress_name"
    if [ "$DRY_RUN" -eq 1 ]; then
        dry_run_log "Would rename $task_name to $in_progress_name"
        echo "$in_progress_path"
    else
        mv "$task_file" "$in_progress_path"
        log "INFO" "Claimed task: $task_name -> $in_progress_name"
        echo "$in_progress_path"
    fi
}

# Function to get attempt count for a task
get_attempt_count() {
    local task_file=$1
    local task_id=$(basename "$task_file" .md)
    local state_file="$STATE_DIR/${task_id}.attempt"
    if [ -f "$state_file" ]; then
        cat "$state_file"
    else
        echo "0"
    fi
}

# Function to increment attempt count
increment_attempt() {
    local task_file=$1
    local task_id=$(basename "$task_file" .md)
    local state_file="$STATE_DIR/${task_id}.attempt"
    local current=$(get_attempt_count "$task_file")
    local next=$((current + 1))
    if [ "$DRY_RUN" -eq 1 ]; then
        dry_run_log "Would increment attempt count for $task_id to $next"
    else
        echo "$next" > "$state_file"
        log "INFO" "Incremented attempt count for $task_id to $next"
    fi
    echo "$next"
}

# Function to reset attempt count
reset_attempt() {
    local task_file=$1
    local task_id=$(basename "$task_file" .md)
    local state_file="$STATE_DIR/${task_id}.attempt"
    if [ "$DRY_RUN" -eq 1 ]; then
        dry_run_log "Would remove attempt file $state_file"
    else
        rm -f "$state_file"
    fi
}

# Function to move task to done
move_to_done() {
    local task_file=$1
    local task_name=$(basename "$task_file")
    local dest="$HANDOFF_DONE/$task_name"
    if [ "$DRY_RUN" -eq 1 ]; then
        dry_run_log "Would move $task_name to HANDOFF/done/"
    else
        mv "$task_file" "$dest"
        log "INFO" "Moved $task_name to HANDOFF/done/"
        reset_attempt "$task_file"
    fi
}

# Function to move task back to todo (after failures)
move_back_to_todo() {
    local task_file=$1
    local task_name=$(basename "$task_file")
    local original_name="${task_name#IN_PROGRESS_}"
    local dest="$HANDOFF_TODO/$original_name"
    if [ "$DRY_RUN" -eq 1 ]; then
        dry_run_log "Would move $task_name back to HANDOFF/todo/"
    else
        mv "$task_file" "$dest"
        log "INFO" "Moved $task_name back to HANDOFF/todo/"
    fi
}

# Function to check slot availability
check_slots_available() {
    local max_slots=2
    if [ -f "$AGENT_POOL_CONFIG" ]; then
        max_slots=$(python -c "import json; print(json.load(open('$AGENT_POOL_CONFIG')).get('max_concurrent', 2))" 2>/dev/null || echo "2")
    fi
    local status_output=$("$ORCHESTRATOR_MANAGER" pool status 2>/dev/null)
    local active_count=$(echo "$status_output" | grep -E "^Total Slots:" | awk '{print $3}' | cut -d'/' -f1)
    if [ -z "$active_count" ]; then
        active_count=0
    fi
    if [ "$active_count" -lt "$max_slots" ]; then
        return 0
    else
        return 1
    fi
}

# Main loop iteration
process_tasks() {
    log "INFO" "Checking HANDOFF/todo/ for new tasks..."
    # Find tasks not already IN_PROGRESS
    for task in "$HANDOFF_TODO"/*.md; do
        [ -e "$task" ] || continue
        local task_name=$(basename "$task")
        # Skip already IN_PROGRESS tasks
        if [[ "$task_name" == IN_PROGRESS_* ]]; then
            continue
        fi
        log "INFO" "Found new task: $task_name"

        # Check slot availability
        if ! check_slots_available; then
            log "WARN" "No available agent slots, skipping"
            break
        fi

        # Claim task
        local in_progress=$(claim_task "$task")
        local attempts=$(get_attempt_count "$in_progress")
        if [ "$attempts" -ge 3 ]; then
            log "ERROR" "Task $task_name has already failed 3 times, moving back to todo"
            move_back_to_todo "$in_progress"
            continue
        fi
        # Select agent
        local agent=$(select_agent "$in_progress")
        log "INFO" "Selected agent: $agent"
        # Launch agent via orchestrator manager
        log "INFO" "Launching agent $agent for task $task_name"
        if [ "$DRY_RUN" -eq 1 ]; then
            dry_run_log "Would launch agent $agent with task $in_progress"
        else
            if "$ORCHESTRATOR_MANAGER" pool launch "$agent" "$in_progress"; then
                log "INFO" "Agent launched successfully"
            else
                log "ERROR" "Failed to launch agent"
                increment_attempt "$in_progress"
            fi
        fi
        # Only process one task per iteration
        break
    done
    log "INFO" "Iteration complete"
}

# Run one iteration
process_tasks