#!/bin/bash
# quota_lib.sh -- Shared quota functions for SCMessenger swarm scripts.
# Source this file to get lazy_quota_refresh and get_quota_context.
# Implements the lazy-refresh-on-read pattern: quota data is refreshed
# only when someone reads it AND the timestamp is over 5 minutes old.

set -euo pipefail

QUOTA_STALE_SECONDS=300
QUOTA_STATE_FILE=".claude/quota_state.json"
QUOTA_SCRAPER='./OllamaQuotaScraper.sh --quiet'

# --- lazy_quota_refresh ---
# Checks quota_state.json timestamp. If missing or older than
# QUOTA_STALE_SECONDS, runs the scraper. Otherwise no-op.
# Returns 0 on success (even if scraper fails -- we continue with
# whatever data is on disk).
lazy_quota_refresh() {
    if [ ! -f "$QUOTA_STATE_FILE" ]; then
        echo "QUOTA: quota_state.json not found -- running scraper..."
        eval "$QUOTA_SCRAPER" 2>/dev/null || true
        return 0
    fi

    # Extract the "timestamp" string value from the JSON.
    # Looks for: "timestamp": "2026-05-14T13:53:23.6274464-10:00"
    # Note: macOS BSD grep/sed does NOT support \s in BRE; use [ \t] and -E for ERE.
    local ts_line
    ts_line=$(grep -E -o '"timestamp"[ \t]*:[ \t]*"[^"]*"' "$QUOTA_STATE_FILE" 2>/dev/null | head -1)

    if [ -z "$ts_line" ]; then
        echo "QUOTA: timestamp field missing from quota_state.json -- running scraper..."
        eval "$QUOTA_SCRAPER" 2>/dev/null || true
        return 0
    fi

    # Extract just the ISO-8601 value between the quotes
    local ts_value
    ts_value=$(echo "$ts_line" | sed -E 's/.*"timestamp"[ \t]*:[ \t]*"([^"]*)".*/\1/')

    if [ -z "$ts_value" ] || [ "$ts_value" = "null" ]; then
        echo "QUOTA: timestamp is null or empty -- running scraper..."
        eval "$QUOTA_SCRAPER" 2>/dev/null || true
        return 0
    fi

    # Convert ISO-8601 to epoch seconds.
    # macOS BSD date does NOT support -d and rejects .NET-style fractional+TZ format.
    # Use python3 (always available via the venv) which handles ISO-8601 correctly.
    local py_bin
    py_bin=$(command -v python3 || command -v python || echo "")
    local ts_epoch
    if [ -z "$py_bin" ]; then
        ts_epoch=0
    else
    ts_epoch=$("$py_bin" -c "
import sys
from datetime import datetime
try:
    print(int(datetime.fromisoformat(sys.argv[1]).timestamp()))
except Exception:
    print(0)
" "$ts_value" 2>/dev/null)
    fi
    ts_epoch="${ts_epoch:-0}"

    if [ "$ts_epoch" = "0" ] || [ -z "$ts_epoch" ]; then
        echo "QUOTA: could not parse timestamp '$ts_value' -- running scraper..."
        eval "$QUOTA_SCRAPER" 2>/dev/null || true
        return 0
    fi

    local now_epoch
    now_epoch=$(date +%s)
    local age=$((now_epoch - ts_epoch))

    if [ "$age" -ge "$QUOTA_STALE_SECONDS" ]; then
        echo "QUOTA: data is ${age}s old (threshold: ${QUOTA_STALE_SECONDS}s) -- running scraper..."
        eval "$QUOTA_SCRAPER" 2>/dev/null || true
    fi

    return 0
}

# --- get_quota_context ---
# Reads quota_state.json and outputs shell variable assignments.
# Usage: eval $(get_quota_context)
# After eval, these variables are available:
#   QUOTA_FIVE_HOUR, QUOTA_SEVEN_DAY, QUOTA_RESET_MINUTES, QUOTA_STATUS
# Returns ? for any field that cannot be extracted.
get_quota_context() {
    local five_hour="?"
    local seven_day="?"
    local reset_minutes="?"
    local status="unknown"

    if [ ! -f "$QUOTA_STATE_FILE" ]; then
        cat <<EOF
QUOTA_FIVE_HOUR=?
QUOTA_SEVEN_DAY=?
QUOTA_RESET_MINUTES=?
QUOTA_STATUS=missing
EOF
        return
    fi

    # Extract status field
    local status_raw
    status_raw=$(grep -E -o '"status"[ \t]*:[ \t]*"[^"]*"' "$QUOTA_STATE_FILE" 2>/dev/null | head -1 | sed -E 's/.*"status"[ \t]*:[ \t]*"([^"]*)".*/\1/')
    if [ -n "$status_raw" ]; then
        status="$status_raw"
    fi

    # Extract fiveHour (may be a number or null)
    local fh_raw
    fh_raw=$(grep -E -o '"fiveHour"[ \t]*:[ \t]*[^,}\n]*' "$QUOTA_STATE_FILE" 2>/dev/null | head -1 | sed -E 's/.*"fiveHour"[ \t]*:[ \t]*([^,}]*).*/\1/' | tr -d '[:space:]')
    if [ -n "$fh_raw" ] && [ "$fh_raw" != "null" ] && [ "$fh_raw" != "?" ]; then
        five_hour="$fh_raw"
    fi

    # Extract sevenDay
    local sd_raw
    sd_raw=$(grep -E -o '"sevenDay"[ \t]*:[ \t]*[^,}\n]*' "$QUOTA_STATE_FILE" 2>/dev/null | head -1 | sed -E 's/.*"sevenDay"[ \t]*:[ \t]*([^,}]*).*/\1/' | tr -d '[:space:]')
    if [ -n "$sd_raw" ] && [ "$sd_raw" != "null" ] && [ "$sd_raw" != "?" ]; then
        seven_day="$sd_raw"
    fi

    # Extract resetMinutes (may be a number or null)
    local rm_raw
    rm_raw=$(grep -E -o '"resetMinutes"[ \t]*:[ \t]*[^,}\n]*' "$QUOTA_STATE_FILE" 2>/dev/null | head -1 | sed -E 's/.*"resetMinutes"[ \t]*:[ \t]*([^,}]*).*/\1/' | tr -d '[:space:]')
    if [ -n "$rm_raw" ] && [ "$rm_raw" != "null" ] && [ "$rm_raw" != "?" ]; then
        reset_minutes="$rm_raw"
    fi

    cat <<EOF
QUOTA_FIVE_HOUR=$five_hour
QUOTA_SEVEN_DAY=$seven_day
QUOTA_RESET_MINUTES=$reset_minutes
QUOTA_STATUS=$status
EOF
}
