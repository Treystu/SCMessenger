#!/usr/bin/env bash
#
# REPO_MAP Health Check Script
# 
# This script ensures the REPO_MAP stays healthy by:
# 1. Running verification checks
# 2. Automatically fixing issues
# 3. Preventing regressions
# 4. Providing clear status reporting
#
# Usage:
#   ./repo_map_health_check.sh [--fix] [--strict]
#
# Options:
#   --fix     Automatically fix issues found
#   --strict  Exit with error if any issues found (for CI/CD)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERIFY_SCRIPT="$SCRIPT_DIR/verify_and_fix_repo_map.py"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
FIX_MODE=false
STRICT_MODE=false

for arg in "$@"; do
    case $arg in
        --fix)
            FIX_MODE=true
            shift
            ;;
        --strict)
            STRICT_MODE=true
            shift
            ;;
        *)
            echo -e "${RED}[ERROR] Unknown option: $arg${NC}"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}================================================================================${NC}"
echo -e "${BLUE}                        REPO_MAP HEALTH CHECK                                   ${NC}"
echo -e "${BLUE}================================================================================${NC}"
echo ""

# Check if Python is available
if ! command -v python &> /dev/null; then
    echo -e "${RED}[ERROR] Python not found. Please install Python 3.7+${NC}"
    exit 1
fi

# Check if verification script exists
if [ ! -f "$VERIFY_SCRIPT" ]; then
    echo -e "${RED}[ERROR] Verification script not found: $VERIFY_SCRIPT${NC}"
    exit 1
fi

# Run verification
echo -e "${YELLOW}[INFO] Running REPO_MAP verification...${NC}"
echo ""

if [ "$FIX_MODE" = true ]; then
    python "$VERIFY_SCRIPT" --fix --repo-root "$REPO_ROOT"
    EXIT_CODE=$?
else
    python "$VERIFY_SCRIPT" --repo-root "$REPO_ROOT"
    EXIT_CODE=$?
fi

echo ""

# Report results
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}[SUCCESS] REPO_MAP is healthy!${NC}"
    exit 0
else
    if [ "$FIX_MODE" = true ]; then
        echo -e "${RED}[ERROR] Failed to fix all issues${NC}"
        exit 1
    else
        echo -e "${YELLOW}[WARNING] Issues found in REPO_MAP${NC}"
        echo -e "${BLUE}[INFO] Run with --fix to automatically fix issues:${NC}"
        echo -e "   ${BLUE}$0 --fix${NC}"
        
        if [ "$STRICT_MODE" = true ]; then
            exit 1
        else
            exit 0
        fi
    fi
fi
