#!/bin/bash
# SCMessenger Progress Dashboard
# Automated tracking of task completion and verification status

echo "=== SCMessenger Progress Dashboard ==="
echo "Generated: $(date)"
echo "======================================"

# Configuration
HANDOFF_DIR="HANDOFF"
VERIFICATION_SCRIPT="scripts/verify_task_systematic.sh"

# Count tasks by status
TOTAL_TASKS=$(find "$HANDOFF_DIR" -name "*.md" | grep -v "TEMPLATE" | wc -l)
COMPLETED_TASKS=$(find "$HANDOFF_DIR/done" -name "*.md" 2>/dev/null | wc -l)
IN_PROGRESS_TASKS=$(find "$HANDOFF_DIR/IN_PROGRESS" -name "*.md" 2>/dev/null | wc -l)
TODO_TASKS=$(find "$HANDOFF_DIR/todo" -name "*.md" 2>/dev/null | wc -l)
BACKLOG_TASKS=$(find "$HANDOFF_DIR/backlog" -name "*.md" 2>/dev/null | wc -l)

# Count verified tasks
VERIFIED_TASKS=$(find "$HANDOFF_DIR" -name "*.md" -exec grep -l "VERIFIED\|Verified\|verified" {} \; | wc -l)

# Calculate percentages
if [ "$TOTAL_TASKS" -gt 0 ]; then
    COMPLETION_PERCENT=$((COMPLETED_TASKS * 100 / TOTAL_TASKS))
    VERIFICATION_PERCENT=$((VERIFIED_TASKS * 100 / COMPLETED_TASKS))
else
    COMPLETION_PERCENT=0
    VERIFICATION_PERCENT=0
fi

# Print overall progress
echo ""
echo "📊 Overall Progress:"
echo "=================="
echo "Total Tasks:       $TOTAL_TASKS"
echo "Completed:         $COMPLETED_TASKS ($COMPLETION_PERCENT%)"
echo "In Progress:       $IN_PROGRESS_TASKS"
echo "Todo:              $TODO_TASKS"
echo "Backlog:           $BACKLOG_TASKS"
echo "Verified:          $VERIFIED_TASKS ($VERIFICATION_PERCENT% of completed)"
echo ""

# Priority breakdown
echo "🎯 Priority Breakdown:"
echo "===================="
P0_TASKS=$(find "$HANDOFF_DIR" -name "*.md" -exec grep -l "P0\|CRITICAL" {} \; | wc -l)
P1_TASKS=$(find "$HANDOFF_DIR" -name "*.md" -exec grep -l "P1\|HIGH" {} \; | wc -l)
P2_TASKS=$(find "$HANDOFF_DIR" -name "*.md" -exec grep -l "P2\|MEDIUM" {} \; | wc -l)

P0_COMPLETED=$(find "$HANDOFF_DIR/done" -name "*.md" -exec grep -l "P0\|CRITICAL" {} \; 2>/dev/null | wc -l)
P1_COMPLETED=$(find "$HANDOFF_DIR/done" -name "*.md" -exec grep -l "P1\|HIGH" {} \; 2>/dev/null | wc -l)
P2_COMPLETED=$(find "$HANDOFF_DIR/done" -name "*.md" -exec grep -l "P2\|MEDIUM" {} \; 2>/dev/null | wc -l)

echo "P0 (Critical):     $P0_COMPLETED/$P0_TASKS completed"
echo "P1 (High):         $P1_COMPLETED/$P1_TASKS completed"
echo "P2 (Medium):       $P2_COMPLETED/$P2_TASKS completed"
echo ""

# Platform breakdown
echo "📱 Platform Breakdown:"
echo "====================="
ANDROID_TASKS=$(find "$HANDOFF_DIR" -name "*.md" -exec grep -l "ANDROID\|Android" {} \; | wc -l)
CORE_TASKS=$(find "$HANDOFF_DIR" -name "*.md" -exec grep -l "CORE\|Rust" {} \; | wc -l)
SECURITY_TASKS=$(find "$HANDOFF_DIR" -name "*.md" -exec grep -l "SECURITY\|Security" {} \; | wc -l)
NETWORK_TASKS=$(find "$HANDOFF_DIR" -name "*.md" -exec grep -l "NETWORK\|Network" {} \; | wc -l)

ANDROID_COMPLETED=$(find "$HANDOFF_DIR/done" -name "*.md" -exec grep -l "ANDROID\|Android" {} \; 2>/dev/null | wc -l)
CORE_COMPLETED=$(find "$HANDOFF_DIR/done" -name "*.md" -exec grep -l "CORE\|Rust" {} \; 2>/dev/null | wc -l)
SECURITY_COMPLETED=$(find "$HANDOFF_DIR/done" -name "*.md" -exec grep -l "SECURITY\|Security" {} \; 2>/dev/null | wc -l)
NETWORK_COMPLETED=$(find "$HANDOFF_DIR/done" -name "*.md" -exec grep -l "NETWORK\|Network" {} \; 2>/dev/null | wc -l)

echo "Android:           $ANDROID_COMPLETED/$ANDROID_TASKS completed"
echo "Core (Rust):       $CORE_COMPLETED/$CORE_TASKS completed"
echo "Security:          $SECURITY_COMPLETED/$SECURITY_TASKS completed"
echo "Network:           $NETWORK_COMPLETED/$NETWORK_TASKS completed"
echo ""

# Recent activity
echo "🔄 Recent Activity:"
echo "=================="
# Get recent git commits related to tasks
git log --oneline --grep="task\|Task\|P0\|P1\|P2\|fix\|Fix\|feat" -n 10 --since="1 week ago" | head -10
echo ""

# Verification status
echo "✅ Verification Status:"
echo "======================"
echo "Tasks requiring verification: $((COMPLETED_TASKS - VERIFIED_TASKS))"

# List unverified completed tasks
if [ "$((COMPLETED_TASKS - VERIFIED_TASKS))" -gt 0 ]; then
    echo "Unverified tasks:"
    find "$HANDOFF_DIR/done" -name "*.md" -exec grep -L "VERIFIED\|Verified\|verified" {} \; 2>/dev/null | head -5
    echo ""
fi

# Recommendations
echo "💡 Recommendations:"
echo "=================="
if [ "$VERIFICATION_PERCENT" -lt 80 ]; then
    echo "⚠️  Low verification rate: Only $VERIFICATION_PERCENT% of completed tasks are verified"
    echo "   Run: ./scripts/verify_task_systematic.sh all report"
fi

if [ "$P0_COMPLETED" -lt "$P0_TASKS" ]; then
    echo "⚠️  Critical P0 tasks remaining: $((P0_TASKS - P0_COMPLETED))"
    echo "   Focus on completing critical issues first"
fi

if [ "$((COMPLETED_TASKS - VERIFIED_TASKS))" -gt 5 ]; then
    echo "⚠️  Multiple unverified tasks: $((COMPLETED_TASKS - VERIFIED_TASKS))"
    echo "   Consider running bulk verification"
fi

echo ""
echo "🚀 Next Steps:"
echo "=============="
echo "1. Run verification: ./scripts/verify_task_systematic.sh all report"
echo "2. Check specific platform: ./scripts/verify_task_systematic.sh [android|core|security|network]"
echo "3. View task details: ls HANDOFF/*/ | grep -E 'P0|P1'"
echo ""
echo "=== Dashboard Complete ==="