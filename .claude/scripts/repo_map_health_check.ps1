# REPO_MAP Health Check Script (PowerShell)
# 
# This script ensures the REPO_MAP stays healthy by:
# 1. Running verification checks
# 2. Automatically fixing issues
# 3. Preventing regressions
# 4. Providing clear status reporting
#
# Usage:
#   .\repo_map_health_check.ps1 [-Fix] [-Strict]
#
# Options:
#   -Fix     Automatically fix issues found
#   -Strict  Exit with error if any issues found (for CI/CD)

param(
    [switch]$Fix,
    [switch]$Strict
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "../..")
$VerifyScript = Join-Path $ScriptDir "verify_and_fix_repo_map.py"

# Colors for output
function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

Write-ColorOutput "================================================================================" "Cyan"
Write-ColorOutput "                        REPO_MAP HEALTH CHECK                                   " "Cyan"
Write-ColorOutput "================================================================================" "Cyan"
Write-Host ""

# Check if Python is available
try {
    $pythonVersion = python --version 2>&1
    Write-ColorOutput "[OK] Python found: $pythonVersion" "Green"
} catch {
    Write-ColorOutput "[ERROR] Python not found. Please install Python 3.7+" "Red"
    exit 1
}

# Check if verification script exists
if (-not (Test-Path $VerifyScript)) {
    Write-ColorOutput "[ERROR] Verification script not found: $VerifyScript" "Red"
    exit 1
}

# Run verification
Write-ColorOutput "[INFO] Running REPO_MAP verification..." "Yellow"
Write-Host ""

$pythonArgs = @("$VerifyScript", "--repo-root", "$RepoRoot")
if ($Fix) {
    $pythonArgs += "--fix"
}

try {
    & python $pythonArgs
    $exitCode = $LASTEXITCODE
} catch {
    Write-ColorOutput "[ERROR] Verification failed: $_" "Red"
    exit 1
}

Write-Host ""

# Report results
if ($exitCode -eq 0) {
    Write-ColorOutput "[SUCCESS] REPO_MAP is healthy!" "Green"
    exit 0
} else {
    if ($Fix) {
        Write-ColorOutput "[ERROR] Failed to fix all issues" "Red"
        exit 1
    } else {
        Write-ColorOutput "[WARNING] Issues found in REPO_MAP" "Yellow"
        Write-ColorOutput "[INFO] Run with -Fix to automatically fix issues:" "Cyan"
        Write-ColorOutput "   .\repo_map_health_check.ps1 -Fix" "Cyan"
        
        if ($Strict) {
            exit 1
        } else {
            exit 0
        }
    }
}
