# SCMessenger Desktop One-Command Build Script for PowerShell
param (
    [switch]$Release
)

$ErrorActionPreference = "Stop"

Write-Host "=== SCMessenger Desktop Build (PowerShell) ===" -ForegroundColor Green

$releaseArg = if ($Release) { "--release" } else { "" }

Write-Host "1. Building scmessenger-desktop-bridge native library..."
cargo build -p scmessenger-desktop-bridge $releaseArg

Write-Host "2. Generating Kotlin FFI bindings..."
cargo run -p scmessenger-desktop-bridge --bin gen_kotlin --features gen-bindings

Write-Host "3. Building KMP Desktop artifact..."
cmd /c "gradlew.bat :shared:packageAppImage"

Write-Host "=== Build Complete ===" -ForegroundColor Green
