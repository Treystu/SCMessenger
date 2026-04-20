# SCMessenger Progress Dashboard (PowerShell)
# Automated tracking of task completion and verification status

Write-Host "=== SCMessenger Progress Dashboard ==="
Write-Host "Generated: $(Get-Date)"
Write-Host "======================================"

# Configuration
$HANDOFF_DIR = "HANDOFF"

# Count tasks by status
$TOTAL_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" -Exclude "*TEMPLATE*").Count
$COMPLETED_TASKS = (Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue).Count
$IN_PROGRESS_TASKS = (Get-ChildItem -Path "$HANDOFF_DIR/IN_PROGRESS" -Filter "*.md" -ErrorAction SilentlyContinue).Count
$TODO_TASKS = (Get-ChildItem -Path "$HANDOFF_DIR/todo" -Filter "*.md" -ErrorAction SilentlyContinue).Count
$BACKLOG_TASKS = (Get-ChildItem -Path "$HANDOFF_DIR/backlog" -Filter "*.md" -ErrorAction SilentlyContinue).Count

# Count verified tasks
$VERIFIED_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" | Where-Object {
    (Get-Content $_.FullName -Raw) -match "VERIFIED|Verified|verified"
}).Count

# Calculate percentages
if ($TOTAL_TASKS -gt 0) {
    $COMPLETION_PERCENT = [math]::Round(($COMPLETED_TASKS / $TOTAL_TASKS) * 100)
    if ($COMPLETED_TASKS -gt 0) {
        $VERIFICATION_PERCENT = [math]::Round(($VERIFIED_TASKS / $COMPLETED_TASKS) * 100)
    } else {
        $VERIFICATION_PERCENT = 0
    }
} else {
    $COMPLETION_PERCENT = 0
    $VERIFICATION_PERCENT = 0
}

# Print overall progress
Write-Host ""
Write-Host "📊 Overall Progress:"
Write-Host "=================="
Write-Host "Total Tasks:       $TOTAL_TASKS"
Write-Host "Completed:         $COMPLETED_TASKS ($COMPLETION_PERCENT%)"
Write-Host "In Progress:       $IN_PROGRESS_TASKS"
Write-Host "Todo:              $TODO_TASKS"
Write-Host "Backlog:           $BACKLOG_TASKS"
Write-Host "Verified:          $VERIFIED_TASKS ($VERIFICATION_PERCENT% of completed)"
Write-Host ""

# Priority breakdown
Write-Host "🎯 Priority Breakdown:"
Write-Host "===================="
$P0_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" | Where-Object {
    (Get-Content $_.FullName -Raw) -match "P0|CRITICAL"
}).Count

$P1_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" | Where-Object {
    (Get-Content $_.FullName -Raw) -match "P1|HIGH"
}).Count

$P2_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" | Where-Object {
    (Get-Content $_.FullName -Raw) -match "P2|MEDIUM"
}).Count

$P0_COMPLETED = (Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object {
    (Get-Content $_.FullName -Raw) -match "P0|CRITICAL"
}).Count

$P1_COMPLETED = (Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object {
    (Get-Content $_.FullName -Raw) -match "P1|HIGH"
}).Count

$P2_COMPLETED = (Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object {
    (Get-Content $_.FullName -Raw) -match "P2|MEDIUM"
}).Count

Write-Host "P0 (Critical):     $P0_COMPLETED/$P0_TASKS completed"
Write-Host "P1 (High):         $P1_COMPLETED/$P1_TASKS completed"
Write-Host "P2 (Medium):       $P2_COMPLETED/$P2_TASKS completed"
Write-Host ""

# Platform breakdown
Write-Host "📱 Platform Breakdown:"
Write-Host "====================="
$ANDROID_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" | Where-Object {
    (Get-Content $_.FullName -Raw) -match "ANDROID|Android"
}).Count

$CORE_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" | Where-Object {
    (Get-Content $_.FullName -Raw) -match "CORE|Rust"
}).Count

$SECURITY_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" | Where-Object {
    (Get-Content $_.FullName -Raw) -match "SECURITY|Security"
}).Count

$NETWORK_TASKS = (Get-ChildItem -Path $HANDOFF_DIR -Recurse -Filter "*.md" | Where-Object {
    (Get-Content $_.FullName -Raw) -match "NETWORK|Network"
}).Count

$ANDROID_COMPLETED = (Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object {
    (Get-Content $_.FullName -Raw) -match "ANDROID|Android"
}).Count

$CORE_COMPLETED = (Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object {
    (Get-Content $_.FullName -Raw) -match "CORE|Rust"
}).Count

$SECURITY_COMPLETED = (Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object {
    (Get-Content $_.FullName -Raw) -match "SECURITY|Security"
}).Count

$NETWORK_COMPLETED = (Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object {
    (Get-Content $_.FullName -Raw) -match "NETWORK|Network"
}).Count

Write-Host "Android:           $ANDROID_COMPLETED/$ANDROID_TASKS completed"
Write-Host "Core (Rust):       $CORE_COMPLETED/$CORE_TASKS completed"
Write-Host "Security:          $SECURITY_COMPLETED/$SECURITY_TASKS completed"
Write-Host "Network:           $NETWORK_COMPLETED/$NETWORK_TASKS completed"
Write-Host ""

# Recent activity
Write-Host "🔄 Recent Activity:"
Write-Host "=================="
# Get recent git commits related to tasks
git log --oneline --grep="task\|Task\|P0\|P1\|P2\|fix\|Fix\|feat" -n 10 --since="1 week ago" | Select-Object -First 10
Write-Host ""

# Verification status
Write-Host "✅ Verification Status:"
Write-Host "======================"
$UNVERIFIED_TASKS = $COMPLETED_TASKS - $VERIFIED_TASKS
Write-Host "Tasks requiring verification: $UNVERIFIED_TASKS"

# List unverified completed tasks
if ($UNVERIFIED_TASKS -gt 0) {
    Write-Host "Unverified tasks:"
    Get-ChildItem -Path "$HANDOFF_DIR/done" -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object {
        (Get-Content $_.FullName -Raw) -notmatch "VERIFIED|Verified|verified"
    } | Select-Object -First 5 -ExpandProperty Name
    Write-Host ""
}

# Recommendations
Write-Host "💡 Recommendations:"
Write-Host "=================="
if ($VERIFICATION_PERCENT -lt 80) {
    Write-Host "⚠️  Low verification rate: Only $VERIFICATION_PERCENT% of completed tasks are verified"
    Write-Host "   Run: ./scripts/verify_task_systematic.sh all report"
}

if ($P0_COMPLETED -lt $P0_TASKS) {
    $REMAINING_P0 = $P0_TASKS - $P0_COMPLETED
    Write-Host "⚠️  Critical P0 tasks remaining: $REMAINING_P0"
    Write-Host "   Focus on completing critical issues first"
}

if ($UNVERIFIED_TASKS -gt 5) {
    Write-Host "⚠️  Multiple unverified tasks: $UNVERIFIED_TASKS"
    Write-Host "   Consider running bulk verification"
}

Write-Host ""
Write-Host "🚀 Next Steps:"
Write-Host "=============="
Write-Host "1. Run verification: ./scripts/verify_task_systematic.sh all report"
Write-Host "2. Check specific platform: ./scripts/verify_task_systematic.sh [android|core|security|network]"
Write-Host "3. View task details: Get-ChildItem HANDOFF/*/ | Where-Object { $_ -match 'P0|P1' }"
Write-Host ""
Write-Host "=== Dashboard Complete ==="