#!/bin/bash
# model_dispatch.sh — Local-first model router for SCMessenger orchestrator
# Usage: model_dispatch.sh <task_type> [complexity_hint]
#   task_type: coding, architecture, tests, review, triage, etc.
#   complexity_hint: small|medium|large|xlarge (default: medium)
# Output: "model_name" (local or cloud) to stdout
# Exit: 0=ok, 1=unknown task, 2=quota hardlock

set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
source "$SCRIPT_DIR/../local_orchestrator_config.sh"

TASK_TYPE="${1,,}"  # lowercase
COMPLEXITY="${2:-medium}"

# ─── Quota Check ──────────────────────────────────────────────────────────────
check_quota() {
    local quota_file="$(cd "$SCRIPT_DIR/../.." && pwd)/$QUOTA_STATE_FILE"
    if [ ! -f "$quota_file" ]; then
        echo "WARN: quota_state.json not found, assuming cloud OK" >&2
        return 0
    fi

    local five_day seven_ts timestamp
    five_day=$(python3 -c "import json; d=json.load(open('$quota_file')); print(d.get('fiveHour',0))" 2>/dev/null)
    seven_day=$(python3 -c "import json; d=json.load(open('$quota_file')); print(d.get('sevenDay',0))" 2>/dev/null)
    timestamp=$(python3 -c "import json; d=json.load(open('$quota_file')); print(d.get('timestamp',''))" 2>/dev/null)

    # Check staleness (5-minute rule)
    if [ -n "$timestamp" ]; then
        local ts_epoch now_epoch age_s
        ts_epoch=$(python3 -c "from datetime import datetime; from email.utils import parsedate_to_datetime; print(int(parsedate_to_datetime('$timestamp').timestamp()))" 2>/dev/null)
        now_epoch=$(date +%s)
        age_s=$((now_epoch - ts_epoch))
        if [ "$age_s" -gt 300 ]; then
            echo "WARN: quota data stale (${age_s}s old), forcing local-only" >&2
            return 2
        fi
    fi

    # Check hardlock
    local max_pct
    max_pct=$(python3 -c "print(max($five_day, $seven_day))" 2>/dev/null)
    if python3 -c "exit(0 if $max_pct > 99.5 else 1)" 2>/dev/null; then
        echo "QUOTA HARDLOCK: ${max_pct}% used, forcing local-only micro tasks" >&2
        return 2
    fi

    # Return quota tier
    if python3 -c "exit(0 if $max_pct <= 25 else 1)" 2>/dev/null; then
        return 0   # HEAVY-LIFT: any model OK
    elif python3 -c "exit(0 if $max_pct <= 50 else 1)" 2>/dev/null; then
        return 0   # EXECUTE: prefer local
    elif python3 -c "exit(0 if $max_pct <= 75 else 1)" 2>/dev/null; then
        return 0   # MIXED: local for small/medium
    elif python3 -c "exit(0 if $max_pct <= 90 else 1)" 2>/dev/null; then
        return 0   # LIGHT: local only, small tasks
    else
        return 2   # MICRO: local only, tiny tasks
    fi
}

# ─_LOCAL model available? ────────────────────────────────────────────────────
check_local_model() {
    local model="$1"
    # Match full model name at start of line (followed by whitespace or end)
    ollama list 2>/dev/null | grep -q "^${model}[[:space:]]"
}

# ─ Resolve route ─────────────────────────────────────────────────────────────
# Returns LOCAL_MODEL|CLOUD_MODEL for the task type
get_route() {
    local task="$1"
    # Indirect reference: ROUTE_<TASK> variable
    local var_name="ROUTE_${task^^}"
    local route="${!var_name:-}"
    if [ -z "$route" ]; then
        # Try with underscores replaced
        var_name="ROUTE_${task^^}"
        route="${!var_name:-}"
    fi
    if [ -z "$route" ]; then
        echo "$ROUTE_GENERAL"
        return
    fi
    echo "$route"
}

# ─ Main dispatch ─────────────────────────────────────────────────────────────
main() {
    local route local_model cloud_model

    route=$(get_route "$TASK_TYPE")
    local_model="${route%%|*}"
    cloud_model="${route#*|}"

    # Check quota
    local quota_ok=0
    check_quota || quota_ok=$?

    # Determine complexity escalation
    local escalate=0
    case "$COMPLEXITY" in
        large|xlarge) escalate=1 ;;
        medium)
            # Medium tasks: use cloud if quota OK and cloud model is significantly larger
            if [ "$quota_ok" -eq 0 ]; then
                escalate=0  # local preferred for medium
            fi
            ;;
        small|micro)
            escalate=0  # always local for small
            ;;
    esac

    # Decision tree
    if [ "$quota_ok" -eq 2 ]; then
        # HARDLOCK or stale: force local
        if check_local_model "$local_model"; then
            echo "$local_model"
            echo "DISPATCH: $TASK_TYPE → $local_model (local — quota hardlock)" >&2
            return 0
        else
            echo "NOMODEL"
            echo "ERROR: quota hardlock and local model $local_model unavailable" >&2
            return 2
        fi
    fi

    if [ "$escalate" -eq 1 ] && [ "$quota_ok" -eq 0 ]; then
        # Large task + quota OK: use cloud
        echo "$cloud_model"
        echo "DISPATCH: $TASK_TYPE → $cloud_model (cloud — large task, quota OK)" >&2
        return 0
    fi

    # Default: try local first
    if check_local_model "$local_model"; then
        echo "$local_model"
        echo "DISPATCH: $TASK_TYPE → $local_model (local)" >&2
        return 0
    fi

    # Local unavailable, try cloud
    if [ "$quota_ok" -eq 0 ]; then
        echo "$cloud_model"
        echo "DISPATCH: $TASK_TYPE → $cloud_model (cloud fallback — local OOM)" >&2
        return 0
    fi

    echo "NOMODEL"
    echo "ERROR: no model available for $TASK_TYPE (local: $local_model, quota: $quota_ok)" >&2
    return 1
}

main
