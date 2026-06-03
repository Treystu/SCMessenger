# =============================================================================
# scripts/build_desktop.ps1 — Desktop build script for SCMessenger KMP
#
# Builds the Rust workspace and then packages the .deb via Gradle.
#
# Usage:
#   .\scripts\build_desktop.ps1
# =============================================================================

$ErrorActionPreference = "Stop"
$ProgressPreference = "Continue"

# Disable Cargo incremental builds for reproducible output
$env:CARGO_INCREMENTAL = "0"

$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $RepoRoot

Write-Host "━━━ Building Rust workspace ━━━" -ForegroundColor Green
cargo build --workspace
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "━━━ Packaging .deb ━━━" -ForegroundColor Green
Set-Location "$RepoRoot\android"
& .\gradlew :shared:packageDeb
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Set-Location $RepoRoot
Write-Host "Build complete." -ForegroundColor Green
