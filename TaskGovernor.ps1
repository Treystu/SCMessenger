param (
    [Parameter(Mandatory=$true)]
    [string]$TaskFile,

    [Parameter(Mandatory=$true)]
    [string]$Model,

    [Parameter(Mandatory=$true)]
    [int]$BudgetLimit
)

$TimeoutMinutes = 45
Write-Host "[GOVERNOR] Launching $Model for $(Split-Path $TaskFile -Leaf) (Limit: $BudgetLimit tokens | Timeout: $TimeoutMinutes min)" -ForegroundColor Magenta

# 1. The Headless Mandate
$HeadlessPrompt = "SYSTEM OVERRIDE: You are a headless autonomous agent. You have NO human user. 1. Use your terminal/file tools to read the file at this exact path: $TaskFile. 2. Execute the instructions inside it without asking for permission or confirmation. 3. When you are finished, you MUST execute the /bye command to terminate the session."

# 2. Launch the Agent
$ClaudeArgs = "launch claude --model $Model --debug --dangerously-skip-permissions `"$HeadlessPrompt`""
$AgentProcess = Start-Process -FilePath "ollama" -ArgumentList $ClaudeArgs -PassThru

# 3. The Watchdog Monitor
$StartTime = Get-Date
$Breached = $false

while (-not $AgentProcess.HasExited) {
    $RunTime = (Get-Date) - $StartTime
    
    # Time Breach Check
    if ($RunTime.TotalMinutes -gt $TimeoutMinutes) {
        Write-Host "[GOVERNOR] Time Breach detected ($TimeoutMinutes min). Terminating agent." -ForegroundColor Red
        Stop-Process -Id $AgentProcess.Id -Force
        $Breached = $true
        
        # Rename and move back to Todo so Orchestrator sees the failure
        $TodoDir = $TaskFile.Replace("IN_PROGRESS", "todo")
        $FailedPath = Join-Path (Split-Path $TodoDir) ("[TIME_BREACH]_" + (Split-Path $TaskFile -Leaf))
        Move-Item -LiteralPath $TaskFile -Destination $FailedPath -Force -ErrorAction SilentlyContinue
        break
    }
    
    Start-Sleep -Seconds 10
}

# 4. Successful Completion Handoff
if (-not $Breached) {
    Write-Host "[GOVERNOR] Agent session ended normally. Moving task to done/." -ForegroundColor Green
    $DonePath = $TaskFile.Replace("IN_PROGRESS", "done")
    Move-Item -LiteralPath $TaskFile -Destination $DonePath -Force -ErrorAction SilentlyContinue
}