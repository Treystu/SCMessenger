# SCMessenger Disk Preflight Check (PowerShell)
# Validates free disk space before running expensive build gates.
# Measured constraint: full five-gate sweep (fmt, clippy, clippy --all-features,
# cargo test --no-run, wasm release) regrows target/ by ~40-47 GB.

param(
    [int]$ThresholdGB = 25
)

# Repo root detection
$repoRoot = Split-Path -Parent $PSScriptRoot

# Threshold in bytes
$thresholdBytes = $ThresholdGB * 1024 * 1024 * 1024

# Get the drive holding the repo
$drive = (Get-Item $repoRoot).PSDrive.Name
$driveInfo = Get-Volume -DriveLetter $drive

$freeGB = [Math]::Round($driveInfo.SizeRemaining / 1GB, 1)
$freeLabel = "$freeGB GB"

# Get current target/ size if it exists
$targetPath = Join-Path $repoRoot "target"
$targetSizeGB = 0
if (Test-Path $targetPath -PathType Container) {
  $targetSizeBytes = (Get-ChildItem $targetPath -Recurse -Force |
    Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum
  if ($targetSizeBytes) {
    $targetSizeGB = [Math]::Round($targetSizeBytes / 1GB, 1)
  }
}

# Report current state
Write-Host "[INFO] Disk preflight check"
Write-Host "[INFO] Free space: $freeLabel on drive $($drive):"
Write-Host "[INFO] Current target/: $targetSizeGB GB"

# Fail if space is too low
if ($driveInfo.SizeRemaining -lt $thresholdBytes) {
  Write-Host "[ERROR] ERROR: Insufficient free disk space"
  Write-Host "[ERROR] Free: $freeLabel, Minimum required: $ThresholdGB GB"
  Write-Host "[ERROR] A full gate sweep regrows target/ to ~45 GB; ensure at least $ThresholdGB GB free"
  Write-Host "[ERROR] Run 'cargo clean' and/or 'cd android; .\gradlew clean' to free space"
  exit 1
}

Write-Host "[OK] Disk space sufficient"
exit 0
