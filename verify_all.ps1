<#
.SYNOPSIS
    SCMessenger Omni-Diagnostic Scanner (V4 - Verbose & Daemon-Safe)
.DESCRIPTION
    Runs a full suite of formatting, linting, compilation, and testing gates across 
    Rust Core, WASM, CLI, and Android Native. Dumps detailed errors to console on failure.
#>

$ErrorActionPreference = "Continue"
$LogFile = "$PSScriptRoot\SCMessenger_Diagnostics.log"
$script:Summary = @()

# Initialize Log
$null = New-Item -ItemType File -Force -Path $LogFile
Add-Content -Path $LogFile -Value "=== SCMessenger Omni-Diagnostic Log | $(Get-Date) ==="

function Log-Message ($Msg, $Color = "White") {
    Write-Host $Msg -ForegroundColor $Color
    Add-Content -Path $LogFile -Value $Msg
}

function Run-Gate ($TaskName, $Command, $WorkingDirectory = $PSScriptRoot) {
    Log-Message "`n>>> RUNNING: $TaskName" "Cyan"
    Log-Message "Command: $Command" "DarkGray"
    
    Push-Location $WorkingDirectory
    
    # Execute the command and capture output
    $process = Start-Process -FilePath "cmd.exe" -ArgumentList "/c $Command" -Wait -NoNewWindow -PassThru -RedirectStandardOutput "$PSScriptRoot\temp_out.txt" -RedirectStandardError "$PSScriptRoot\temp_err.txt"
    
    $exitCode = $process.ExitCode
    $out = Get-Content "$PSScriptRoot\temp_out.txt" -ErrorAction SilentlyContinue
    $err = Get-Content "$PSScriptRoot\temp_err.txt" -ErrorAction SilentlyContinue
    
    if ($out) { Add-Content -Path $LogFile -Value $out }
    if ($err) { Add-Content -Path $LogFile -Value $err }
    
    Pop-Location
    
    # Cleanup temp files
    Remove-Item "$PSScriptRoot\temp_out.txt", "$PSScriptRoot\temp_err.txt" -ErrorAction SilentlyContinue

    if ($exitCode -eq 0) {
        Log-Message "[PASS] $TaskName" "Green"
        $script:Summary += [PSCustomObject]@{ Task = $TaskName; Status = "PASS"; Color = "Green" }
    } else {
        Log-Message "[FAIL] $TaskName (Exit Code: $exitCode)" "Red"
        $script:Summary += [PSCustomObject]@{ Task = $TaskName; Status = "FAIL"; Color = "Red" }
        
        # --- VERBOSE ERROR DUMP ---
        Write-Host "`n--- ERROR DETAILS ---" -ForegroundColor Red
        if ($err) { Write-Host ($err -join "`n") -ForegroundColor Red }
        if ($out) { Write-Host ($out -join "`n") -ForegroundColor DarkYellow }
        Write-Host "-----------------------`n" -ForegroundColor Red
    }
}

Log-Message "===================================================" "Magenta"
Log-Message ">>> SCMessenger Multi-Platform Diagnostic Scanner" "Magenta"
Log-Message "===================================================" "Magenta"

# --- SYSTEM OVERRIDES ---
Log-Message "`nApplying System Overrides..." "Yellow"
$env:CARGO_INCREMENTAL = "0"
Log-Message "Set CARGO_INCREMENTAL=0 (Windows .rlib Lock Protection)" "DarkGray"

# --- 1. RUST CORE & CLI GATES ---
Log-Message "`n--- PHASE 1: RUST CORE & CLI ---" "Yellow"
Run-Gate "Rust Format Check" "cargo fmt --all -- --check"
Run-Gate "Rust Clippy (Linter)" "cargo clippy --workspace -- -D warnings"
Run-Gate "Rust Workspace Compile" "cargo check --workspace"
Run-Gate "Rust Workspace Tests" "cargo test --workspace --no-fail-fast"

# --- 2. WASM GATES ---
Log-Message "`n--- PHASE 2: WASM CLIENT ---" "Yellow"
Run-Gate "WASM Target Compile" "cargo check --target wasm32-unknown-unknown -p scmessenger-wasm"

# --- 3. ANDROID NATIVE GATES ---
Log-Message "`n--- PHASE 3: ANDROID NATIVE ---" "Yellow"
$AndroidDir = Join-Path $PSScriptRoot "android"
if (Test-Path $AndroidDir) {
    # Added --no-daemon to prevent PowerShell from hanging while waiting for Gradle to release stdout
    Run-Gate "Android Gradle Sync & Clean" ".\gradlew clean --no-daemon" $AndroidDir
    Run-Gate "Android Lint" ".\gradlew lintDebug --no-daemon" $AndroidDir
    Run-Gate "Android Unit Tests" ".\gradlew testDebugUnitTest --no-daemon" $AndroidDir
    Run-Gate "Android Assemble Debug (UniFFI)" ".\gradlew assembleDebug --no-daemon" $AndroidDir
} else {
    Log-Message "[SKIP] Android directory not found." "DarkGray"
}

# --- SUMMARY ---
Log-Message "`n===================================================" "Magenta"
Log-Message "DIAGNOSTIC SUMMARY" "Magenta"
Log-Message "===================================================" "Magenta"

$FailCount = 0
foreach ($item in $script:Summary) {
    Write-Host ("[{0}] {1}" -f $item.Status, $item.Task) -ForegroundColor $item.Color
    Add-Content -Path $LogFile -Value ("[{0}] {1}" -f $item.Status, $item.Task)
    if ($item.Status -eq "FAIL") { $FailCount++ }
}

Log-Message "`nFull output saved to: $LogFile" "Cyan"

if ($FailCount -gt 0) {
    Log-Message "`n[ERROR] $FailCount Gates Failed. Hand off the red error traces above to the swarm." "Red"
    exit 1
} else {
    Log-Message "`n[SUCCESS] All Gates Passed! Repository is stable." "Green"
    exit 0
}