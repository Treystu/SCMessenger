Write-Host "[SWARM HEARTBEAT] Initializing OpenRouter OS Governor..." -ForegroundColor Cyan

$DoneDir = "HANDOFF\done"
$TodoDir = "HANDOFF\todo"

$OrchBudgetLimit = 4000
$OrchTimeLimitMinutes = 3

while ($true) {
    $DoneTasks = @(Get-ChildItem -Path $DoneDir -Filter "*.md")
    $BreachedTasks = @(Get-ChildItem -Path $TodoDir -Filter "*BREACH_*.md")
    $PendingTasks = @(Get-ChildItem -Path $TodoDir -Filter "*.md" | Where-Object { $_.Name -notmatch "BREACH" })
    
    $ActiveWorkers = @(Get-WmiObject Win32_Process -Filter "CommandLine LIKE '%TaskGovernor.ps1%'" -ErrorAction SilentlyContinue).Count
    $OrchRunning = @(Get-Process -Name "node" | Where-Object { $_.CommandLine -match "claude" } -ErrorAction SilentlyContinue).Count
    $TotalSlotsUsed = $ActiveWorkers + $OrchRunning

    if ($DoneTasks.Count -gt 0) { Write-Host "[HEARTBEAT] Tasks completed. Waiting for Orchestrator..." -ForegroundColor Green }

    # Launch Sub-Agents (Max 3 slots total)
    if ($PendingTasks.Count -gt 0 -and $TotalSlotsUsed -lt 3 -and $OrchRunning -eq 0) {
        $NextTask = $PendingTasks[0]
        
        $Content = Get-Content $NextTask.FullName -TotalCount 15
        $Model = ($Content -match "Model:\s*(.+)") -replace "Model:\s*",""
        $Budget = ($Content -match "Budget:\s*(\d+)") -replace "Budget:\s*",""
        
        # Fallbacks
        if (-not $Model) { $Model = "google/gemma-4-26b-a4b-it:free" }
        if (-not $Budget) { $Budget = 6000 }

        Write-Host "[HEARTBEAT] Slot free! Launching $Model for $($NextTask.Name)..." -ForegroundColor Cyan
        Start-Process -FilePath "powershell.exe" -ArgumentList "-File .\TaskGovernor.ps1 -TaskFile `"$($NextTask.FullName)`" -Model `"$Model`" -BudgetLimit $Budget" -WindowStyle Hidden
        
        Start-Sleep -Seconds 2
        continue
    }

    # Wake Master Orchestrator (Owl Alpha)
    $NeedsOrchestration = ($PendingTasks.Count -eq 0) -or ($BreachedTasks.Count -gt 0) -or ($DoneTasks.Count -gt 0)
    if ($NeedsOrchestration -and $OrchRunning -eq 0 -and $ActiveWorkers -lt 3) {
        Write-Host "[HEARTBEAT] Waking Owl Alpha Orchestrator..." -ForegroundColor Yellow
        
        $OrchArgs = "--model openrouter/owl-alpha:free --debug --dangerously-skip-permissions `"Execute your stateless Hit-and-Run protocol.`""
        $OrchProcess = Start-Process -FilePath "claude.cmd" -ArgumentList $OrchArgs -PassThru -NoNewWindow
        
        Start-Sleep -Seconds 3
        $LogDir = "$env:USERPROFILE\.claude\debug"
        $LatestLog = Get-ChildItem -Path $LogDir -Filter "*.txt" | Sort-Object LastWriteTime -Descending | Select-Object -First 1

        $StartTime = Get-Date
        while (-not $OrchProcess.HasExited) {
            Start-Sleep -Seconds 3
            if ((Get-Date) -gt $StartTime.AddMinutes($OrchTimeLimitMinutes)) {
                Stop-Process -Id $OrchProcess.Id -Force
                break
            }
            if ($LatestLog) {
                $LogContent = Get-Content $LatestLog.FullName -Raw
                $In = ([regex]::Matches($LogContent, '"input_tokens"\s*:\s*(\d+)') | ForEach-Object { [int]$_.Groups[1].Value } | Measure-Object -Sum).Sum
                $Out = ([regex]::Matches($LogContent, '"output_tokens"\s*:\s*(\d+)') | ForEach-Object { [int]$_.Groups[1].Value } | Measure-Object -Sum).Sum
                if (($In + $Out) -gt $OrchBudgetLimit) {
                    Stop-Process -Id $OrchProcess.Id -Force
                    break
                }
            }
        }
    }
    Start-Sleep -Seconds 5
}