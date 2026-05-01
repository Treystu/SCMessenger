param (
    [Parameter(Mandatory=$false)]
    [string]$CommandArgs = "status"
)

$DaemonPath = "C:\Users\kanal\Documents\Github\SCMessenger\target\debug\scmessenger-cli.exe"
$StateDir = "$env:LOCALAPPDATA\scmessenger"

Write-Host "[SCM Driver] Executing command..." -ForegroundColor Cyan

# reset | clean -- wipe all local daemon state
if ($CommandArgs -match "^(reset|clean)$") {
    if (Test-Path $StateDir) {
        Write-Host "[SCM Driver] Removing daemon state at: $StateDir" -ForegroundColor Yellow
        Remove-Item -Recurse -Force $StateDir
        Write-Host "[SCM Driver] State wiped successfully." -ForegroundColor Green
    } else {
        Write-Host "[SCM Driver] No state directory found at $StateDir -- already clean." -ForegroundColor Green
    }
    exit 0
}

# start -- launch daemon in background
if ($CommandArgs -match "^start") {
    Write-Host "[SCM Driver] Starting daemon: $DaemonPath" -ForegroundColor Cyan
    Start-Process -FilePath $DaemonPath -ArgumentList "start" -NoNewWindow
    Write-Host "[SCM Driver] Daemon launched. Waiting for bind..." -ForegroundColor Green
    Start-Sleep -Seconds 3
    exit 0
}

# scan -- discover mesh peers on LAN
if ($CommandArgs -match "^scan") {
    Write-Host "[SCM Driver] Scanning mesh for peers..." -ForegroundColor Cyan
    $result = & $DaemonPath "scan"
    Write-Host $result
    exit $LASTEXITCODE
}

# contact list -- list known contacts
if ($CommandArgs -match "^contact list") {
    Write-Host "[SCM Driver] Listing contacts..." -ForegroundColor Cyan
    $result = & $DaemonPath "contact" "list"
    Write-Host $result
    exit $LASTEXITCODE
}

# send peer_id message -- send to a discovered peer
if ($CommandArgs -match "^send ") {
    $argsList = $CommandArgs -split ' ', 3
    if ($argsList.Count -lt 3) {
        Write-Host '[SCM Driver] Usage: send <peer_id> "<message>"' -ForegroundColor Red
        exit 1
    }
    $peerId = $argsList[1]
    $message = $argsList[2] -replace '^"|"$', ''
    Write-Host "[SCM Driver] Sending to $peerId : $message" -ForegroundColor Cyan
    $result = & $DaemonPath "send" $peerId $message
    Write-Host $result
    exit $LASTEXITCODE
}

# passthrough -- pass raw args to the CLI daemon
$result = & $DaemonPath $CommandArgs.Split(" ")
if ($LASTEXITCODE -ne 0) {
    Write-Host "[SCM Driver] Warning: Command returned non-zero exit code ($LASTEXITCODE)" -ForegroundColor Yellow
}
Write-Host $result
