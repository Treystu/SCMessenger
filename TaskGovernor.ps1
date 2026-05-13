param (
    [Parameter(Mandatory=$true)] [string]$TaskFile,
    [Parameter(Mandatory=$true)] [string]$Model,
    [Parameter(Mandatory=$true)] [int]$BudgetLimit
)

$TimeoutMinutes = 45
Write-Host "[GOVERNOR] Launching $Model for $(Split-Path $TaskFile -Leaf)" -ForegroundColor Magenta

# Headless Mandate with -p (auto-exit)
$HeadlessPrompt = "SYSTEM OVERRIDE: Use your tools to read $TaskFile, execute all instructions, and summarize. Do not ask for help."
$ClaudeArgs = "launch claude --model $Model --dangerously-skip-permissions -p `"$HeadlessPrompt`""

# Launch and Wait
$AgentProcess = Start-Process -FilePath "ollama" -ArgumentList $ClaudeArgs -PassThru -WindowStyle Normal

$StartTime = Get-Date
$Breached = $false

while (-not $AgentProcess.HasExited) {
    if (((Get-Date) - $StartTime).TotalMinutes -gt $TimeoutMinutes) {
        Write-Host "[GOVERNOR] Timeout. Killing agent." -ForegroundColor Red
        Stop-Process -Id $AgentProcess.Id -Force
        $Breached = $true
        
        $FailedPath = Join-Path (Split-Path ($TaskFile.Replace("IN_PROGRESS", "todo"))) ("[TIME_BREACH]_" + (Split-Path $TaskFile -Leaf))
        Move-Item -LiteralPath $TaskFile -Destination $FailedPath -Force
        break
    }
    Start-Sleep -Seconds 10
}

if (-not $Breached) {
    Write-Host "[GOVERNOR] Success. Moving to done/." -ForegroundColor Green
    $DonePath = $TaskFile.Replace("IN_PROGRESS", "done")
    Move-Item -LiteralPath $TaskFile -Destination $DonePath -Force
}