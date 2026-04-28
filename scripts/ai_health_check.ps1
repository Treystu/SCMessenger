# SCMessenger AI Configuration Health Check
# This script verifies that the AI configuration is optimized and not conflicting.

$errors = 0
$warnings = 0

Write-Host "Auditing AI Configuration..." -ForegroundColor Cyan

# 1. Check for .claudeignore
if (Test-Path ".claudeignore") {
    Write-Host "PASS: .claudeignore found." -ForegroundColor Green
} else {
    Write-Host "FAIL: .claudeignore MISSING. Indexing will be slow!" -ForegroundColor Red
    $errors++
}

# 2. Check for large tracking files
$trackingFile = Get-Item "REMAINING_WORK_TRACKING.md" -ErrorAction SilentlyContinue
if ($trackingFile) {
    if ($trackingFile.Length -gt 100kb) {
        $sizeKB = [math]::Round($trackingFile.Length/1kb, 2)
        Write-Host "WARN: REMAINING_WORK_TRACKING.md is very large ($sizeKB KB). Consider archiving resolved items." -ForegroundColor Yellow
        $warnings++
    } else {
        Write-Host "PASS: REMAINING_WORK_TRACKING.md size is optimal." -ForegroundColor Green
    }
}

# 3. Check for Orchestrator State
if (Test-Path ".claude/orchestrator_active") {
    Write-Host "INFO: Orchestrator is currently ACTIVE." -ForegroundColor Blue
} else {
    Write-Host "INFO: Orchestrator is currently INACTIVE." -ForegroundColor Gray
}

# 4. Check for conflicting rules
if ((Test-Path "CLAUDE.md") -and (Test-Path ".clinerules")) {
    Write-Host "WARN: Both CLAUDE.md and .clinerules exist. Ensure they don't have conflicting Orchestrator definitions." -ForegroundColor Yellow
    $warnings++
}

# 5. Check for build artifacts leaking into indexing (if .claudeignore exists)
if (Test-Path ".claudeignore") {
    $content = Get-Content ".claudeignore"
    if ($content -match "target/") {
        Write-Host "PASS: target/ found in .claudeignore" -ForegroundColor Green
    } else {
        Write-Host "WARN: target/ not found in .claudeignore" -ForegroundColor Yellow
        $warnings++
    }
}

$summaryColor = "Green"
if ($errors -gt 0) { $summaryColor = "Red" }

Write-Host "`nSummary: $errors Errors, $warnings Warnings." -ForegroundColor $summaryColor

if ($errors -gt 0) {
    Write-Host "Critical issues found. Please review the AI_CONFIG_AUDIT.md artifact." -ForegroundColor Red
    exit 1
} else {
    Write-Host "Health check passed." -ForegroundColor Green
    exit 0
}
