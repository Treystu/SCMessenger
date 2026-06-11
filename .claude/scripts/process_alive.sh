#!/bin/bash
# process_alive.sh — Cross-platform process liveness check with 5-sec cache
# Usage: source this file, then call process_alive <pid>
# Returns 0 if alive, 1 if dead
#
# macOS port (2026-06-10): replaced PowerShell Get-Process calls with `ps`.
# `kill -0` is the primary check (works for any Unix process); `ps -p` is
# the fallback (catches macOS process PID namespaces if kill -0 fails).
#
# Cache: per-PID files in .claude/process_cache/<pid> with 5-sec TTL.

PROCESS_CACHE_DIR=".claude/process_cache"
PROCESS_CACHE_TTL=5  # seconds

process_alive() {
    local pid="$1"
    if [ -z "$pid" ]; then return 1; fi

    local now=$(date +%s)

    # Check cache
    local cache_file="$PROCESS_CACHE_DIR/$pid"
    if [ -f "$cache_file" ]; then
        local cached_ts=$(cat "$cache_file" 2>/dev/null)
        if [[ "$cached_ts" =~ ^[0-9]+$ ]]; then
            local cache_age=$((now - cached_ts))
            if [ "$cache_age" -lt "$PROCESS_CACHE_TTL" ]; then
                # Cache hit — PID was alive when cached, still within TTL
                return 0
            fi
        fi
    fi

    # Cache miss or expired — do actual check
    local alive=1
    # Primary: kill -0 (works for any process in our PID namespace)
    if kill -0 "$pid" 2>/dev/null; then
        alive=0
    else
        # Fallback: ps -p (works even if kill -0 is blocked by SIP/permissions)
        if ps -p "$pid" >/dev/null 2>&1; then
            alive=0
        fi
    fi

    if [ "$alive" -eq 0 ]; then
        # Process alive — update cache
        mkdir -p "$PROCESS_CACHE_DIR" 2>/dev/null
        echo "$now" > "$cache_file"
        return 0
    else
        # Process dead — remove cache entry
        rm -f "$cache_file" 2>/dev/null
        return 1
    fi
}

# Invalidate cache for a specific PID (call after killing a process)
process_cache_invalidate() {
    local pid="$1"
    rm -f "$PROCESS_CACHE_DIR/$pid" 2>/dev/null
}

# Invalidate all cache entries (call after pool stop or state change)
process_cache_clear() {
    rm -rf "$PROCESS_CACHE_DIR" 2>/dev/null
}

# Check if a PID is actually a claude process (not a reused PID).
# On macOS PIDs can be reused; kill -0 alone is not sufficient.
process_is_claude() {
    local pid="$1"
    if [ -z "$pid" ]; then return 1; fi
    local name
    name=$(ps -p "$pid" -o comm= 2>/dev/null)
    if [ "$name" = "claude" ]; then
        return 0
    fi
    return 1
}

process_memory_kb() {
    local pid="$1"
    if [ -z "$pid" ]; then echo "0"; return; fi
    # macOS: ps -o rss gives resident set size in kilobytes
    local rss=$(ps -p "$pid" -o rss= 2>/dev/null | tr -d ' ')
    if [ -z "$rss" ] || ! [[ "$rss" =~ ^[0-9]+$ ]]; then
        echo "0"
        return
    fi
    echo "$rss"
}
