# === CONFIGURATION ===
$Script:BaseDir              = $PSScriptRoot
$Script:DoneDir              = Join-Path $BaseDir "HANDOFF\done"
$Script:TodoDir              = Join-Path $BaseDir "HANDOFF\todo"
$Script:InProgDir            = Join-Path $BaseDir "HANDOFF\IN_PROGRESS"
$Script:CompleteFlag         = Join-Path $BaseDir "HANDOFF\SWARM_COMPLETE"
$Script:MaxConcurrentSlots    = 3
$Script:PollIntervalSeconds   = 10
$Script:OrchModel             = "deepseek-v4-pro:cloud"
$Script:OrchFallbackModel     = "kimi-k2.6:cloud"
$Script:StaleThresholdMinutes = 60
$Script:OrchCooldownSeconds   = 120

# Runtime state
$Script:LastOrchLaunch           = [datetime]::MinValue
$Script:OrchCompletedCurrentCycle = $false
$Script:PrevDoneCount             = 0

function Write-HeartbeatLog {
    param([string]$Level, [string]$Message)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss.fff"
    $line = "[$timestamp][$Level] $Message"
    switch ($Level) {
        "ERROR" { Write-Host $line -ForegroundColor Red }
        "WARN"  { Write-Host $line -ForegroundColor Yellow }
        "INFO"  { Write-Host $line -ForegroundColor Cyan }
        "DEBUG" { Write-Host $line -ForegroundColor DarkGray }
        default { Write-Host $line }
    }
}

function Get-ActiveWorkerCount {
    $running = Get-Job -State Running -ErrorAction SilentlyContinue
    $workers = $running | Where-Object { $_.Name -like "Worker_*" }
    return @($workers).Count
}

function Get-OrchestratorRunning {
    $running = Get-Job -State Running -ErrorAction SilentlyContinue
    $orch = $running | Where-Object { $_.Name -eq "Orchestrator" }
    return @($orch).Count -gt 0
}

function Get-SwarmFileState {
    $doneFiles    = @(Get-ChildItem -LiteralPath $Script:DoneDir -Filter "*.md" -ErrorAction SilentlyContinue)
    $pendingFiles = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[VALIDATED]_*.md" -ErrorAction SilentlyContinue)
    $inProgFiles  = @(Get-ChildItem -LiteralPath $Script:InProgDir -Filter "*.md" -ErrorAction SilentlyContinue)

    $state = @{
        DoneCount       = $doneFiles.Count
        PendingCount    = $pendingFiles.Count
        InProgressCount = $inProgFiles.Count
        PendingTasks    = $pendingFiles | Sort-Object Name
        HasNewDone      = $doneFiles.Count -gt $Script:PrevDoneCount
    }
    return $state
}

function Get-StaleInProgressTasks {
    $staleTaskFiles = @()
    $cutoff = (Get-Date).AddMinutes(-$Script:StaleThresholdMinutes)
    try {
        $inProgFiles = Get-ChildItem -LiteralPath $Script:InProgDir -Filter "*.md" -ErrorAction SilentlyContinue
        foreach ($f in $inProgFiles) {
            if ($f.LastWriteTime -lt $cutoff) {
                $staleTaskFiles += $f
            }
        }
    } catch {
        Write-HeartbeatLog "ERROR" "Failed to scan IN_PROGRESS: $($_.Exception.Message)"
    }
    return $staleTaskFiles
}

function Test-OllamaReachable {
    try {
        $null = Invoke-RestMethod -Uri "http://localhost:11434/api/tags" -TimeoutSec 5 -ErrorAction Stop
        return $true
    } catch {
        Write-HeartbeatLog "ERROR" "Ollama service not reachable at localhost:11434"
        return $false
    }
}

function Invoke-DispatchWorker {
    param([System.IO.FileInfo]$TaskFile)

    try {
        $content = Get-Content -LiteralPath $TaskFile.FullName -TotalCount 30 -ErrorAction Stop
        $modelLine  = ($content | Where-Object { $_ -match "^#\s*(MODEL|Model):" } | Select-Object -First 1)
        $budgetLine = ($content | Where-Object { $_ -match "^#\s*(BUDGET|Budget):" } | Select-Object -First 1)

        $model  = if ($modelLine  -match ":\s*(.+)") { $matches[1].Trim() } else { $null }
        $budget = if ($budgetLine -match ":\s*(\d+)") { [int]$matches[1] } else { 3600 }

        if (-not $model) {
            Write-HeartbeatLog "WARN" "Task $($TaskFile.Name) has no Model header - skipping"
            return $false
        }

        Write-HeartbeatLog "INFO" "Dispatching Worker for $($TaskFile.Name) -> model=$model budget=${budget}s"

        $inProgressPath = Join-Path $Script:InProgDir $TaskFile.Name
        Move-Item -LiteralPath $TaskFile.FullName -Destination $inProgressPath -Force -ErrorAction Stop

        $jobName = "Worker_" + (Get-Date -Format "HHmmss")
        $job = Start-Job -Name $jobName -ArgumentList $inProgressPath, $model, $budget, $Script:BaseDir -ScriptBlock {
            param($TaskFile, $Model, $BudgetLimit, $BaseDir)
            Set-Location $BaseDir
            & "$BaseDir\TaskGovernor.ps1" -TaskFile $TaskFile -Model $Model -BudgetLimit $BudgetLimit
        }
        Write-HeartbeatLog "INFO" "Worker dispatched: job=$jobName file=$($TaskFile.Name)"
        return $true
    } catch {
        Write-HeartbeatLog "ERROR" "Worker dispatch failed for $($TaskFile.Name): $($_.Exception.Message)"
        return $false
    }
}

function Invoke-DispatchOrchestrator {
    param([string]$Reason)

    $now = Get-Date
    if (($now - $Script:LastOrchLaunch).TotalSeconds -lt $Script:OrchCooldownSeconds) {
        Write-HeartbeatLog "DEBUG" "Orchestrator cooldown active ($([math]::Round(($now - $Script:LastOrchLaunch).TotalSeconds))s elapsed). Reason: $Reason"
        return $false
    }

    Write-HeartbeatLog "INFO" "Dispatching Orchestrator (reason: $Reason)"
    $Script:LastOrchLaunch = $now
    $Script:OrchCompletedCurrentCycle = $false

    $mandate = @'
SYSTEM OVERRIDE: Headless Orchestrator Agent.

YOUR JOB:
1. Read HANDOFF/done/ for newly completed tasks since your last scan. Update REMAINING_WORK_TRACKING.md and any affected canonical docs (DOCUMENTATION.md, docs/CURRENT_STATE.md) to reflect completed work.
2. Read HANDOFF/IN_PROGRESS/ for stale tasks (LastWriteTime > 60 min ago). If found, move them back to HANDOFF/todo/ with [STALE]_ prefix.
3. Read HANDOFF/todo/ for unvalidated tasks (missing [VALIDATED]_ prefix). Validate each: check if the target code still needs work. Add [VALIDATED]_ prefix to validated tasks. Reject false positives (already-wired, test-only, golden-strings).
4. Read REMAINING_WORK_TRACKING.md and HANDOFF/backlog/. If remaining work exists not yet in HANDOFF/todo/, create new task files with proper headers:
   # MODEL: <appropriate model from routing table>
   # BUDGET: <seconds based on task complexity>
   # TARGET: <file path>
   Prefix files with [VALIDATED]_ to signal readiness.
5. Assign models per CLAUDE.md routing table:
   - Rust core/identity/crypto/transport/store -> glm-5.1:cloud
   - Crypto/math/security audit -> deepseek-v3.2:cloud
   - Android/Kotlin -> qwen3-coder-next:cloud
   - iOS/Swift -> glm-5.1:cloud
   - Tests/docs/bindings -> gemma4:31b:cloud
   - Quick fix/lint/CI -> gemini-3-flash-preview:cloud
   - Architecture/planning -> deepseek-v4-pro:cloud
   - Code review merge gate -> kimi-k2-thinking:cloud
6. Set budget per task:
   - Micro tasks (lint, format, single-line): 300s
   - Small tasks (single function, test): 900s
   - Medium tasks (multi-file wiring, platform): 1800s
   - Large tasks (module implementation, refactor): 3600s
   - Architecture/review tasks: 5400s
7. Write HANDOFF/ORCHESTRATOR_STATUS.md containing exactly:
   STATUS=completed (or STATUS=ALL_DONE if genuinely nothing remains)
   TASKS_CREATED=N
   TASKS_VALIDATED=N
   STALE_RECLAIMED=N
   COMPLETED_AT=<timestamp>
   NOTES=<any blockers or observations>

CRITICAL: You MANAGE the queue. Do NOT write application code (.rs, .kt, .swift, .ts). Exit immediately after writing your status file.
'@

    $job = Start-Job -Name "Orchestrator" -ArgumentList $mandate, $Script:BaseDir, $Script:OrchModel -ScriptBlock {
        param($Mandate, $BaseDir, $Model)
        Set-Location $BaseDir
        & ollama launch claude --model $Model -- --dangerously-skip-permissions --print -p $Mandate *>&1 | Out-Null
    }
    Write-HeartbeatLog "INFO" "Orchestrator job started"
    return $true
}

function Test-SwarmComplete {
    param([hashtable]$FileState)

    if ($FileState.PendingCount -gt 0)   { return $false }
    if ($FileState.InProgressCount -gt 0) { return $false }
    if ((Get-ActiveWorkerCount) -gt 0)    { return $false }
    if (Get-OrchestratorRunning)           { return $false }

    $orchStatusFile = Join-Path $Script:BaseDir "HANDOFF\ORCHESTRATOR_STATUS.md"
    if (Test-Path $orchStatusFile) {
        $statusContent = Get-Content $orchStatusFile -Raw -ErrorAction SilentlyContinue
        if ($statusContent -match "STATUS=ALL_DONE") {
            return $true
        }
    }
    return $false
}

function Invoke-HeartbeatPulse {
    Write-HeartbeatLog "DEBUG" "=== PULSE $(Get-Date -Format 'HH:mm:ss') ==="

    # Phase 1: Cleanup completed jobs
    $completedJobs = Get-Job -State Completed -ErrorAction SilentlyContinue
    foreach ($j in $completedJobs) {
        if ($j.Name -eq "Orchestrator") {
            $Script:OrchCompletedCurrentCycle = $true
            Write-HeartbeatLog "INFO" "Orchestrator job completed"
            $orchStatusFile = Join-Path $Script:BaseDir "HANDOFF\ORCHESTRATOR_STATUS.md"
            if (Test-Path $orchStatusFile) {
                $statusContent = Get-Content $orchStatusFile -Raw -ErrorAction SilentlyContinue
                if ($statusContent -match "STATUS=ALL_DONE") {
                    Write-HeartbeatLog "INFO" "Orchestrator reports ALL_DONE - swarm may be complete"
                }
            }
        }
        Receive-Job -Job $j 2>$null | Out-Null
        Remove-Job -Job $j -Force -ErrorAction SilentlyContinue
    }

    # Phase 2: Scan file system
    $fileState      = Get-SwarmFileState
    $workerCount    = Get-ActiveWorkerCount
    $orchRunning    = Get-OrchestratorRunning
    $staleTasks     = Get-StaleInProgressTasks
    $totalSlotsUsed = $workerCount + (if ($orchRunning) { 1 } else { 0 })
    $slotsFree      = $Script:MaxConcurrentSlots - $totalSlotsUsed

    Write-HeartbeatLog "INFO" ("State -> Done: {0} | Pending: {1} | InProg: {2} | Stale: {3} | Workers: {4}/{5} free | Orch: {6}" -f
        $fileState.DoneCount, $fileState.PendingCount, $fileState.InProgressCount,
        $staleTasks.Count, $workerCount, $slotsFree, (if ($orchRunning) { "Y" } else { "N" }))

    $Script:PrevDoneCount = $fileState.DoneCount

    # Phase 3: Exit check
    if (Test-SwarmComplete -FileState $fileState) {
        Write-HeartbeatLog "INFO" "SWARM COMPLETE: All tasks finished, orchestrator confirms ALL_DONE."
        try {
            "SWARM_COMPLETED_AT=$(Get-Date -Format 'o')" | Out-File -LiteralPath $Script:CompleteFlag -Encoding utf8
            Write-HeartbeatLog "INFO" "Completion flag written to $($Script:CompleteFlag)"
        } catch {
            Write-HeartbeatLog "ERROR" "Failed to write completion flag: $($_.Exception.Message)"
        }
        exit 0
    }

    # Phase 4: Launch worker (at most 1 per pulse)
    if ($fileState.PendingCount -gt 0 -and $slotsFree -gt 0) {
        $slotsForWorkers = $Script:MaxConcurrentSlots - (if ($orchRunning) { 1 } else { 0 })
        if ($workerCount -lt $slotsForWorkers) {
            $task = $fileState.PendingTasks | Select-Object -First 1
            if ($task) {
                Invoke-DispatchWorker -TaskFile $task
            }
        }
    }

    # Phase 5: Launch orchestrator (only when all workers idle)
    if ($workerCount -eq 0 -and -not $orchRunning) {
        $needsOrch = $false
        $orchReason = ""

        if ($fileState.HasNewDone) {
            $needsOrch = $true
            $orchReason = "new completions detected in done/"
        } elseif ($staleTasks.Count -gt 0) {
            $needsOrch = $true
            $orchReason = "$($staleTasks.Count) stale task(s) in IN_PROGRESS need reclamation"
        } elseif ($fileState.PendingCount -eq 0 -and $fileState.InProgressCount -eq 0) {
            if (-not $Script:OrchCompletedCurrentCycle) {
                $needsOrch = $true
                $orchReason = "queue drained, checking for remaining work in backlog"
            }
        }

        if ($needsOrch) {
            Invoke-DispatchOrchestrator -Reason $orchReason
            Start-Sleep -Seconds 5
        }
    }
}

function Invoke-HeartbeatBoot {
    Write-HeartbeatLog "INFO" "============================================"
    Write-HeartbeatLog "INFO" "SCMessenger Swarm Heartbeat v2.0 BOOTING"
    Write-HeartbeatLog "INFO" "Base: $($Script:BaseDir)"
    Write-HeartbeatLog "INFO" "Max Concurrent Slots: $($Script:MaxConcurrentSlots)"
    Write-HeartbeatLog "INFO" "Orchestrator Model: $($Script:OrchModel)"
    Write-HeartbeatLog "INFO" "Poll Interval: $($Script:PollIntervalSeconds)s"
    Write-HeartbeatLog "INFO" "============================================"

    # Phase 1: Directory structure
    foreach ($dir in @($Script:DoneDir, $Script:TodoDir, $Script:InProgDir)) {
        if (-not (Test-Path $dir)) {
            try {
                New-Item -ItemType Directory -Path $dir -Force | Out-Null
                Write-HeartbeatLog "INFO" "Created directory: $dir"
            } catch {
                Write-HeartbeatLog "ERROR" "FATAL: Cannot create $dir - $($_.Exception.Message)"
                exit 1
            }
        }
    }

    # Phase 2: Ollama health check
    if (-not (Test-OllamaReachable)) {
        Write-HeartbeatLog "ERROR" "FATAL: Ollama is not reachable. Ensure 'ollama serve' is running."
        exit 1
    }
    Write-HeartbeatLog "INFO" "Ollama service reachable at localhost:11434"

    # Phase 3: Remove stale completion flag from prior run
    if (Test-Path $Script:CompleteFlag) {
        Remove-Item $Script:CompleteFlag -Force -ErrorAction SilentlyContinue
        Write-HeartbeatLog "INFO" "Removed previous SWARM_COMPLETE flag"
    }

    # Phase 4: Seed initial file counts
    $initial = Get-SwarmFileState
    $Script:PrevDoneCount = $initial.DoneCount
    Write-HeartbeatLog "INFO" "Initial state -- Done: $($initial.DoneCount) | Pending: $($initial.PendingCount) | InProg: $($initial.InProgressCount)"

    # Phase 5: Clean up orphaned PS jobs from prior session
    Get-Job | Remove-Job -Force -ErrorAction SilentlyContinue
    Write-HeartbeatLog "INFO" "Cleaned up orphaned PowerShell jobs"

    Write-HeartbeatLog "INFO" "Boot complete. Entering main dispatch loop."
}

# === MAIN ===
Invoke-HeartbeatBoot
while ($true) {
    try {
        Invoke-HeartbeatPulse
    } catch {
        Write-HeartbeatLog "ERROR" "UNHANDLED EXCEPTION in pulse: $($_.Exception.Message)"
        Write-HeartbeatLog "ERROR" "Stack trace: $($_.ScriptStackTrace)"
    }
    Start-Sleep -Seconds $Script:PollIntervalSeconds
}
