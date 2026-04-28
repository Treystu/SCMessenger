#!/bin/bash
# ADB Connection Cleanup Script
# Kills stale ADB connections and restarts the server cleanly
# Usage: ./adb_cleanup.sh [--force] [--verbose]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

FORCE=false
VERBOSE=false

# Parse arguments
for arg in "$@"; do
    case $arg in
        --force|-f) FORCE=true ;;
        --verbose|-v) VERBOSE=true ;;
        --help|-h)
            echo "Usage: ./adb_cleanup.sh [--force] [--verbose]"
            echo ""
            echo "Options:"
            echo "  --force, -f    Skip confirmation prompts"
            echo "  --verbose, -v  Show detailed output"
            echo ""
            echo "This script:"
            echo "  1. Counts existing ADB connections"
            echo "  2. Identifies processes holding connections"
            echo "  3. Optionally terminates blocking processes"
            echo "  4. Restarts ADB server cleanly"
            exit 0
            ;;
    esac
done

echo "=========================================="
echo "  ADB Connection Cleanup Tool"
echo "=========================================="
echo ""

# 1. Check current ADB connection count
echo -e "${YELLOW}[1/5]${NC} Checking ADB connections..."
CONNECTION_COUNT=$(lsof -i :5037 2>/dev/null | grep -c "ESTABLISHED" || echo "0")
LISTEN_COUNT=$(lsof -i :5037 2>/dev/null | grep -c "LISTEN" || echo "0")

echo "  ESTABLISHED connections: $CONNECTION_COUNT"
echo "  LISTEN sockets: $LISTEN_COUNT"

if [ "$CONNECTION_COUNT" -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} No active connections to clean up"
else
    echo -e "  ${YELLOW}!${NC} Found $CONNECTION_COUNT active connections"
    
    if [ "$VERBOSE" = true ]; then
        echo ""
        echo "  Connection details:"
        lsof -i :5037 -n -P 2>/dev/null | grep "ESTABLISHED" | awk '{print "    " $1 " (PID:" $2 ") " $9 }' | sort -u
    fi
fi
echo ""

# 2. Identify processes holding ADB connections
echo -e "${YELLOW}[2/5]${NC} Identifying processes..."
STUDIO_PID=$(pgrep -f "studio" | head -1 || echo "")
GRADLE_PIDS=$(pgrep -f "GradleDaemon" | tr '\n' ' ' || echo "")
OTHER_ADB=$(pgrep -a adb | grep -v "fork-server" | awk '{print $1}' | tr '\n' ' ' || echo "")

echo "  Android Studio: ${STUDIO_PID:-None}"
echo "  Gradle daemons: ${GRADLE_PIDS:-None}"
echo "  Other ADB processes: ${OTHER_ADB:-None}"
echo ""

# 3. Check for lock files
echo -e "${YELLOW}[3/5]${NC} Checking for lock files..."
LOCK_FILES=$(find ~/.android -name "*.lock" 2>/dev/null || echo "")
if [ -n "$LOCK_FILES" ]; then
    echo -e "  ${YELLOW}!${NC} Found lock files:"
    echo "$LOCK_FILES" | while read f; do
        echo "    - $f ($(stat -f%Sm "$f" 2>/dev/null || echo "unknown date"))"
    done
    
    if [ "$FORCE" = true ]; then
        echo "  Removing lock files..."
        echo "$LOCK_FILES" | xargs rm -f 2>/dev/null || true
        echo -e "  ${GREEN}✓${NC} Lock files removed"
    else
        echo "  Use --force to remove lock files"
    fi
else
    echo -e "  ${GREEN}✓${NC} No lock files found"
fi
echo ""

# 4. Prompt for cleanup (unless --force)
echo -e "${YELLOW}[4/5]${NC} Cleanup decision..."
if [ "$CONNECTION_COUNT" -gt 5 ]; then
    echo -e "  ${RED}⚠${NC} High connection count detected ($CONNECTION_COUNT)"
    echo "  This may indicate connection buildup causing issues"
fi

if [ "$FORCE" = true ]; then
    echo "  Proceeding with forced cleanup..."
else
    if [ "$CONNECTION_COUNT" -gt 0 ]; then
        echo "  Do you want to restart ADB server? (y/n)"
        read -r response
        if [[ ! "$response" =~ ^[Yy] ]]; then
            echo "  Cleanup cancelled"
            exit 0
        fi
    fi
fi
echo ""

# 5. Restart ADB server
echo -e "${YELLOW}[5/5]${NC} Restarting ADB server..."
echo "  Stopping ADB server..."
adb kill-server 2>/dev/null || true
sleep 1

echo "  Starting ADB server..."
adb start-server
echo -e "  ${GREEN}✓${NC} ADB server restarted"
echo ""

# Verify cleanup
echo "=========================================="
echo "  Cleanup Complete"
echo "=========================================="
echo ""

NEW_CONNECTIONS=$(lsof -i :5037 2>/dev/null | grep -c "ESTABLISHED" || echo "0")
echo "Connections before: $CONNECTION_COUNT"
echo "Connections after:  $NEW_CONNECTIONS"
echo ""

echo "Connected devices:"
adb devices -l | grep -v "List" | grep -v "^$" || echo "  (none)"
echo ""

if [ "$NEW_CONNECTIONS" -lt "$CONNECTION_COUNT" ]; then
    echo -e "${GREEN}✓ Successfully reduced connections from $CONNECTION_COUNT to $NEW_CONNECTIONS${NC}"
else
    echo -e "${YELLOW}! Connection count unchanged. If issues persist, consider:${NC}"
    echo "  - Closing Android Studio"
    echo "  - Running: ./adb_cleanup.sh --force"
    echo "  - Rebooting the device"
fi
