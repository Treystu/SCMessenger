#!/bin/bash
# SCMessenger AI Configuration Health Check (Bash version)

ERRORS=0
WARNINGS=0

echo -e "\033[0;36m🔍 Auditing AI Configuration...\033[0m"

# 1. Check for .claudeignore
if [ -f ".claudeignore" ]; then
    echo -e "\033[0;32m✅ .claudeignore found.\033[0m"
else
    echo -e "\033[0;31m❌ .claudeignore MISSING. Indexing will be slow!\033[0m"
    ERRORS=$((ERRORS+1))
fi

# 2. Check for large tracking files
if [ -f "REMAINING_WORK_TRACKING.md" ]; then
    SIZE=$(du -k "REMAINING_WORK_TRACKING.md" | cut -f1)
    if [ "$SIZE" -gt 100 ]; then
        echo -e "\033[1;33m⚠️ REMAINING_WORK_TRACKING.md is very large (${SIZE}KB). Consider archiving resolved items.\033[0m"
        WARNINGS=$((WARNINGS+1))
    else
        echo -e "\033[0;32m✅ REMAINING_WORK_TRACKING.md size is optimal.\033[0m"
    fi
fi

# 3. Check for Orchestrator State
if [ -f ".claude/orchestrator_active" ]; then
    echo -e "\033[0;34mℹ️ Orchestrator is currently ACTIVE.\033[0m"
else
    echo -e "\033[0;90mℹ️ Orchestrator is currently INACTIVE.\033[0m"
fi

# 4. Check for conflicting rules
if [ -f "CLAUDE.md" ] && [ -f ".clinerules" ]; then
    echo -e "\033[1;33m⚠️ Both CLAUDE.md and .clinerules exist. Ensure they don't have conflicting Orchestrator definitions.\033[0m"
    WARNINGS=$((WARNINGS+1))
fi

echo ""
echo -e "Summary: $ERRORS Errors, $WARNINGS Warnings."

if [ "$ERRORS" -gt 0 ]; then
    echo -e "\033[0;31mCritical issues found. Please review the AI_CONFIG_AUDIT.md artifact.\033[0m"
    exit 1
else
    echo -e "\033[0;32mHealth check passed with minor warnings.\033[0m"
    exit 0
fi
