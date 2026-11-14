# SCMessenger CLI — Windows: copy release binary to ~/.local/bin (user scope).
#
# Prerequisites: cargo build --release -p scmessenger-cli
# Optional: $env:SCMESSENGER_BIN = path to scmessenger-cli.exe

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$DefaultBin = Join-Path $Root "target\release\scmessenger-cli.exe"
$Bin = if ($env:SCMESSENGER_BIN) { $env:SCMESSENGER_BIN } else { $DefaultBin }
$DestDir = Join-Path $HOME ".local\bin"
$Dest = Join-Path $DestDir "scmessenger-cli.exe"

if (-not (Test-Path -LiteralPath $Bin)) {
    Write-Error "CLI binary not found: $Bin`nBuild with: cargo build --release -p scmessenger-cli"
}

New-Item -ItemType Directory -Force -Path $DestDir | Out-Null
Copy-Item -LiteralPath $Bin -Destination $Dest -Force
Write-Host "Installed: $Dest"
Write-Host "Add to PATH if needed: $DestDir"
Write-Host "Register a background task manually (Task Scheduler) to run: `"$Dest`" start"
