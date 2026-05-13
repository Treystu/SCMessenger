Write-Host "[SWARM HEARTBEAT] Booting State-Locked Governor..." -ForegroundColor Cyan

$BaseDir = $PSScriptRoot
$DoneDir = Join-Path $BaseDir "HANDOFF\done"
$TodoDir = Join-Path $BaseDir "HANDOFF\todo"
$InProgressDir = Join-Path $BaseDir "HANDOFF\IN_PROGRESS"

# Ensure all state directories exist
foreach ($dir in @($DoneDir, $TodoDir, $InProgressDir)) {
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
}

$WorkerPIDs = @()

while ($true) {
    try {
        Write-Host "--- PULSE $(Get-Date -Format 'HH:mm:ss') ---" -ForegroundColor DarkGray
        
        # 1. Clean up dead worker PIDs
        $ActiveWorkers = @()
        foreach ($wPid in $WorkerPIDs) {
            if (Get-Process -Id $wPid -ErrorAction SilentlyContinue) {
                $ActiveWorkers += $wPid
            }
        }
        $WorkerPIDs = $ActiveWorkers
        $WorkerCount = $WorkerPIDs.Count

        # 2. Check orchestrators
        $OrchRunning = @(Get-Process -Name "claude" -ErrorAction SilentlyContinue).Count
        $TotalSlotsUsed = $WorkerCount + $OrchRunning
        
        $DoneTasks = @(Get-ChildItem -LiteralPath $DoneDir -Filter "*.md" -ErrorAction SilentlyContinue)
        $PendingTasks = @(Get-ChildItem -LiteralPath $TodoDir -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notmatch "BREACH" })
        
        Write-Host "State -> Done: $($DoneTasks.Count) | Pending: $($PendingTasks.Count) | Workers: $WorkerCount | Orchestrators: $OrchRunning" -ForegroundColor DarkGray

        # LOGIC: Launch Workers
        if ($PendingTasks.Count -gt 0 -and $TotalSlotsUsed -lt 3 -and $OrchRunning -eq 0) {
            $NextTask = $PendingTasks[0]
            $Content = Get-Content -LiteralPath $NextTask.FullName -TotalCount 20 -ErrorAction SilentlyContinue
            
            $Model = ($Content -match "Model:\s*(.+)") -replace "Model:\s*",""
            $Budget = ($Content -match "Budget:\s*(\d+)") -replace "Budget:\s*",""
            
            if ($Model) {
                if (-not $Budget) { $Budget = 6000 }
                Write-Host "[HEARTBEAT] Slot free! Locking $($NextTask.Name) to IN_PROGRESS..." -ForegroundColor Cyan
                
                # STATE LOCK: Move file to IN_PROGRESS
                $InProgressPath = Join-Path $InProgressDir $NextTask.Name
                Move-Item -LiteralPath $NextTask.FullName -Destination $InProgressPath -Force
                
                $GovArgs = "-ExecutionPolicy Bypass -File `".\TaskGovernor.ps1`" -TaskFile `"$InProgressPath`" -Model `"$Model`" -BudgetLimit $Budget"
                
                # LAUNCH & CAPTURE
                $NewWorker = Start-Process -FilePath "powershell.exe" -ArgumentList $GovArgs -WindowStyle Normal -PassThru
                $WorkerPIDs += $NewWorker.Id
                
                Start-Sleep -Seconds 5
                continue
            }
        }

        # LOGIC: Launch Orchestrator
        $NeedsOrchestration = ($PendingTasks.Count -eq 0) -or ($DoneTasks.Count -gt 0)
        if ($NeedsOrchestration -and $OrchRunning -eq 0 -and $WorkerCount -lt 3) {
            Write-Host "[HEARTBEAT] Waking DeepSeek Orchestrator..." -ForegroundColor Yellow
            
            $OrchArgs = "launch claude --model deepseek-v4-pro:cloud -- --debug --dangerously-skip-permissions `"Execute your stateless Hit-and-Run protocol.`""
            
            Start-Process -FilePath "ollama" -ArgumentList $OrchArgs -WindowStyle Normal
            
            Write-Host "[HEARTBEAT] Orchestrator spawned in new window. Waiting 30s..." -ForegroundColor DarkYellow
            Start-Sleep -Seconds 30 
        }

    } catch {
        Write-Host "[CRITICAL ERROR] $($_.Exception.Message)" -ForegroundColor Red
    }

    Start-Sleep -Seconds 10
}