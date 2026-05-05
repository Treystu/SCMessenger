#!/bin/bash
# process_alive.sh — Cross-platform process liveness check with 60-sec cache
# Usage: source this file, then call process_alive <pid>
# Returns 0 if alive, 1 if dead
#
# On Windows Git Bash (MSYS2), PIDs exist in the MSYS2 namespace.
# PowerShell can only see Windows native PIDs. We try kill -0 first
# (works for MSYS2 PIDs), then fall back to PowerShell (for Windows PIDs).
#
# Cache: per-PID files in .claude/process_cache/<pid> with 60-sec TTL.
# Eliminates redundant PowerShell spawns for repeated checks.

PROCESS_CACHE_DIR=".claude/process_cache"
PROCESS_CACHE_TTL=60  # seconds

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
    # Try MSYS2/Git Bash PID first — works for processes spawned from bash
    if kill -0 "$pid" 2>/dev/null; then
        alive=0
    else
        # Fallback to PowerShell for Windows native PIDs (e.g., claude.exe)
        if powershell.exe -NoProfile -Command "if (Get-Process -Id $pid -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }" 2>/dev/null; then
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

process_memory_kb() {
    local pid="$1"
    if [ -z "$pid" ]; then echo "0"; return; fi
    # PowerShell can only measure Windows native PIDs
    local bytes=$(powershell.exe -NoProfile -Command "(Get-Process -Id $pid -ErrorAction SilentlyContinue).WorkingSet64" 2>/dev/null || echo "0")
    bytes=$(echo "$bytes" | tr -d '[:space:]')
    if [ -z "$bytes" ] || ! [[ "$bytes" =~ ^[0-9]+$ ]]; then
        echo "0"
        return
    fi
    echo $((bytes / 1024))
}