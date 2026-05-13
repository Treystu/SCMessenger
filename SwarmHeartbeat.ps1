Write-Host "[SWARM HEARTBEAT] Booting Immortal OS Governor..." -ForegroundColor Cyan

$BaseDir = $PSScriptRoot
$DoneDir = Join-Path $BaseDir "HANDOFF\done"
$TodoDir = Join-Path $BaseDir "HANDOFF\todo"
$InProgressDir = Join-Path $BaseDir "HANDOFF\IN_PROGRESS"

# Ensure directories exist
foreach ($dir in @($DoneDir, $TodoDir, $InProgressDir)) {
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
}

while ($true) {
    try {
        Write-Host "--- PULSE $(Get-Date -Format 'HH:mm:ss') ---" -ForegroundColor DarkGray
        
        # 1. DETECT ACTIVE WORKERS (By searching command lines for our script name)
        $ActiveWorkers = @(Get-CimInstance Win32_Process -Filter "CommandLine LIKE '%TaskGovernor.ps1%'" -ErrorAction SilentlyContinue)
        $WorkerCount = $ActiveWorkers.Count

        # 2. DETECT ACTIVE ORCHESTRATORS (By searching command lines for the specific model)
        $ActiveOrchs = @(Get-CimInstance Win32_Process -Filter "CommandLine LIKE '%deepseek-v4-pro:cloud%'" -ErrorAction SilentlyContinue)
        $OrchRunning = $ActiveOrchs.Count
        
        $TotalSlotsUsed = $WorkerCount + $OrchRunning
        
        $DoneTasks = @(Get-ChildItem -LiteralPath $DoneDir -Filter "*.md" -ErrorAction SilentlyContinue)
        $PendingTasks = @(Get-ChildItem -LiteralPath $TodoDir -Filter "*.md" -ErrorAction SilentlyContinue | Where-Object { $_.Name -notmatch "BREACH" })
        
        Write-Host "State -> Done: $($DoneTasks.Count) | Pending: $($PendingTasks.Count) | Workers: $WorkerCount | Orchestrators: $OrchRunning" -ForegroundColor DarkGray

        # LOGIC: Launch Workers (Max 3 slots)
        if ($PendingTasks.Count -gt 0 -and $TotalSlotsUsed -lt 3 -and $OrchRunning -eq 0) {
            $NextTask = $PendingTasks[0]
            $Content = Get-Content -LiteralPath $NextTask.FullName -TotalCount 20 -ErrorAction SilentlyContinue
            
            $Model = ($Content -match "Model:\s*(.+)") -replace "Model:\s*",""
            $Budget = ($Content -match "Budget:\s*(\d+)") -replace "Budget:\s*",""
            
            if ($Model) {
                $Budget = if ($Budget) { $Budget } else { 6000 }
                Write-Host "[HEARTBEAT] Locking $($NextTask.Name) and launching Worker..." -ForegroundColor Cyan
                
                $InProgressPath = Join-Path $InProgressDir $NextTask.Name
                Move-Item -LiteralPath $NextTask.FullName -Destination $InProgressPath -Force
                
                # TOTAL DETACHMENT: Launch via cmd /c start so the exit signal never reaches PowerShell
                $GovCmd = "powershell -ExecutionPolicy Bypass -File `"$BaseDir\TaskGovernor.ps1`" -TaskFile `"$InProgressPath`" -Model `"$Model`" -BudgetLimit $Budget"
                Start-Process "cmd.exe" -ArgumentList "/c start `"$($NextTask.Name)`" $GovCmd" -WindowStyle Hidden
                
                Start-Sleep -Seconds 5
                continue
            }
        }

        # LOGIC: Launch Orchestrator (DeepSeek V4 Pro)
        # We only wake the orchestrator if there is actual work to clean up or a gap to fill
        $NeedsOrchestration = ($PendingTasks.Count -eq 0 -and $InProgressDir.Count -eq 0) -or ($DoneTasks.Count -gt 0)
        if ($NeedsOrchestration -and $OrchRunning -eq 0 -and $WorkerCount -eq 0) {
            Write-Host "[HEARTBEAT] Waking DeepSeek Orchestrator..." -ForegroundColor Yellow
            
            $OrchMandate = "SYSTEM OVERRIDE: You are a headless OS-integrated router. 1. Read HANDOFF/done, update docs, validate HANDOFF/todo (add [VALIDATED]_ prefix). 2. You are FORBIDDEN from using bash scripts. 3. Close immediately when done."
            $OrchArgs = "ollama launch claude --model deepseek-v4-pro:cloud --dangerously-skip-permissions -p `"$OrchMandate`""
            
            # TOTAL DETACHMENT
            Start-Process "cmd.exe" -ArgumentList "/c start `"ORCHESTRATOR`" $OrchArgs" -WindowStyle Hidden
            
            Write-Host "[HEARTBEAT] Orchestrator dispatched. Waiting for cycle..." -ForegroundColor DarkYellow
            Start-Sleep -Seconds 45 
        }

    } catch {
        Write-Host "[CRITICAL ERROR] $($_.Exception.Message)" -ForegroundColor Red
    }

    Start-Sleep -Seconds 10
}