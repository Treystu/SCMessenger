#!/bin/bash
# process_alive.sh — Cross-platform process liveness check
# Usage: source this file, then call process_alive <pid>
# Returns 0 if alive, 1 if dead
#
# On Windows Git Bash (MSYS2), PIDs exist in the MSYS2 namespace.
# PowerShell can only see Windows native PIDs. We try kill -0 first
# (works for MSYS2 PIDs), then fall back to PowerShell (for Windows PIDs).

process_alive() {
    local pid="$1"
    if [ -z "$pid" ]; then return 1; fi
    # Try MSYS2/Git Bash PID first — works for processes spawned from bash
    if kill -0 "$pid" 2>/dev/null; then
        return 0
    fi
    # Fallback to PowerShell for Windows native PIDs (e.g., claude.exe)
    powershell.exe -NoProfile -Command "if (Get-Process -Id $pid -ErrorAction SilentlyContinue) { exit 0 } else { exit 1 }" 2>/dev/null
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