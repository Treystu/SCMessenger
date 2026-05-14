# === CONFIGURATION ===
$Script:BaseDir              = $PSScriptRoot
$Script:DoneDir              = Join-Path $BaseDir "HANDOFF\done"
$Script:TodoDir              = Join-Path $BaseDir "HANDOFF\todo"
$Script:InProgDir            = Join-Path $BaseDir "HANDOFF\IN_PROGRESS"
$Script:CompleteFlag         = Join-Path $BaseDir "HANDOFF\SWARM_COMPLETE"
$Script:MaxConcurrentSlots    = 3
$Script:PollIntervalSeconds   = 10
$Script:OrchModelTriage       = "gemini-3-flash-preview:cloud"
$Script:OrchModelStandard     = "deepseek-v4-flash:cloud"
$Script:OrchModelHeavy        = "qwen3-coder-next:cloud"
$Script:OrchFallbackModel     = "kimi-k2.6:cloud"
$Script:StaleThresholdMinutes = 60
$Script:OrchCooldownSeconds   = 120
$Script:QuotaCheckInterval    = 30

# Runtime state
$Script:LastOrchLaunch              = [datetime]::MinValue
$Script:OrchCompletedCurrentCycle    = $false
$Script:PrevDoneCount                = 0
$Script:WakeOrchestratorForTriage    = $false
$Script:PulseCount                   = 0
$Script:OrchCycleCount               = 0
$Script:CurrentQuotaTier             = 1
$Script:Tier4Active                  = $false
$Script:MaxBudgetOverride            = 0
$Script:LastOrchModelDispatched      = ""
$Script:LastOrchReasonDispatched     = ""

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

function Get-ClaudeProcessCount {
    $claudeProcs = @(Get-Process -Name "claude" -ErrorAction SilentlyContinue)
    return $claudeProcs.Count
}

function Get-ActiveSlotCount {
    $workerCount  = Get-ActiveWorkerCount
    $claudeCount  = Get-ClaudeProcessCount
    $orchRunning  = Get-OrchestratorRunning
    $orchSlots    = if ($orchRunning) { 1 } else { 0 }
    return @{
        WorkerJobs   = $workerCount
        ClaudeProcs  = $claudeCount
        OrchSlots    = $orchSlots
        TotalUsed    = [math]::Max($workerCount, $claudeCount) + $orchSlots
        SlotsFree    = $Script:MaxConcurrentSlots - ([math]::Max($workerCount, $claudeCount) + $orchSlots)
    }
}

function Get-SwarmFileState {
    $doneFiles    = @(Get-ChildItem -LiteralPath $Script:DoneDir -Filter "*.md" -ErrorAction SilentlyContinue)
    $pendingFiles = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[VALIDATED]_*.md" -ErrorAction SilentlyContinue)
    $inProgFiles  = @(Get-ChildItem -LiteralPath $Script:InProgDir -Filter "*.md" -ErrorAction SilentlyContinue)
    $failedFiles  = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[FAILED]_*.md" -ErrorAction SilentlyContinue)
    $staleFiles   = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[STALE]_*.md" -ErrorAction SilentlyContinue)

    $activeSlots  = Get-ActiveSlotCount
    $orphanInProg = ($inProgFiles.Count -gt 0) -and ($activeSlots.TotalUsed -eq 0)

    $state = @{
        DoneCount          = $doneFiles.Count
        PendingCount       = $pendingFiles.Count
        InProgressCount    = $inProgFiles.Count
        FailedOrStaleCount = $failedFiles.Count + $staleFiles.Count
        OrphanInProgress   = $orphanInProg
        PendingTasks       = $pendingFiles | Sort-Object Name
        HasNewDone         = $doneFiles.Count -gt $Script:PrevDoneCount
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

# === QUOTA GOVERNOR ===

function Read-QuotaState {
    $quotaFile = Join-Path $Script:BaseDir ".claude\API_QUOTA_STATE.md"
    if (-not (Test-Path $quotaFile)) {
        Write-HeartbeatLog "WARN" "API_QUOTA_STATE.md not found; defaulting to Tier 1"
        return @{ FiveHour = 0; SevenDay = 0; Tier = 1 }
    }

    try {
        $content = Get-Content -LiteralPath $quotaFile -Raw -ErrorAction Stop
        $fiveHour = 0; $sevenDay = 0
        if ($content -match "5-Hour Usage.*?([\d\.]+)%") { $fiveHour = [double]$matches[1] }
        if ($content -match "7-Day Usage.*?([\d\.]+)%")  { $sevenDay = [double]$matches[1] }

        # Determine tier based on 5-hour usage per orchestrate.md thresholds
        $tier = 1
        if ($fiveHour -gt 92)      { $tier = 4 }
        elseif ($fiveHour -gt 75)  { $tier = 3 }
        elseif ($fiveHour -gt 50)  { $tier = 2 }

        return @{ FiveHour = $fiveHour; SevenDay = $sevenDay; Tier = $tier }
    } catch {
        Write-HeartbeatLog "ERROR" "Failed to read quota state: $($_.Exception.Message)"
        return @{ FiveHour = 0; SevenDay = 0; Tier = 1 }
    }
}

function Invoke-QuotaGovernor {
    $quota = Read-QuotaState
    $oldTier = $Script:CurrentQuotaTier
    $Script:CurrentQuotaTier = $quota.Tier

    if ($oldTier -ne $quota.Tier) {
        Write-HeartbeatLog "WARN" ("QUOTA TIER CHANGE: {0} -> {1} (5hr={2}% 7d={3}%)" -f $oldTier, $quota.Tier, $quota.FiveHour, $quota.SevenDay)
    }

    switch ($quota.Tier) {
        4 {
            $Script:MaxConcurrentSlots = 1
            $Script:OrchCooldownSeconds = 600
            $Script:MaxBudgetOverride = 600
            $Script:Tier4Active = $true
            Write-HeartbeatLog "WARN" "Tier 4 ACTIVE: Single-slot, triage-only, max budget 600s, P0 only"
        }
        3 {
            $Script:MaxConcurrentSlots = 2
            $Script:OrchCooldownSeconds = 300
            $Script:MaxBudgetOverride = 1800
            $Script:Tier4Active = $false
            Write-HeartbeatLog "WARN" "Tier 3 ACTIVE: Mid-tier models only, max budget 1800s"
        }
        2 {
            $Script:MaxConcurrentSlots = 2
            $Script:OrchCooldownSeconds = 180
            $Script:MaxBudgetOverride = 3600
            $Script:Tier4Active = $false
            Write-HeartbeatLog "INFO" "Tier 2 ACTIVE: Mixed tier, batch micro-tasks"
        }
        default {
            $Script:MaxConcurrentSlots = 3
            $Script:OrchCooldownSeconds = 120
            $Script:MaxBudgetOverride = 0
            $Script:Tier4Active = $false
            Write-HeartbeatLog "INFO" "Tier 1 ACTIVE: Full speed, flagship models"
        }
    }
}

# === ORCHESTRATOR MODEL SELECTION ===

function Select-OrchestratorModel {
    param([string]$Reason)

    # At quota Tier 4, force triage model regardless of reason
    if ($Script:CurrentQuotaTier -ge 4) {
        Write-HeartbeatLog "INFO" "Orchestrator model forced to triage (quota Tier $($Script:CurrentQuotaTier))"
        return @{ Model = $Script:OrchModelTriage; MandateType = "triage" }
    }

    # At quota Tier 3, use standard model for everything
    if ($Script:CurrentQuotaTier -ge 3) {
        return @{ Model = $Script:OrchModelStandard; MandateType = "standard" }
    }

    # Tier 1-2: select based on reason complexity
    if ($Reason -match "malformed|triage|NEEDS_TRIAGE") {
        return @{ Model = $Script:OrchModelTriage; MandateType = "triage" }
    }
    elseif ($Reason -match "backlog|drained|remaining") {
        return @{ Model = $Script:OrchModelHeavy; MandateType = "heavy" }
    }
    else {
        return @{ Model = $Script:OrchModelStandard; MandateType = "standard" }
    }
}

# === WORKTREE CLEANUP ===

function Invoke-CleanupStaleWorktrees {
    $worktreeDir = Join-Path $Script:BaseDir ".claude\worktrees"
    if (-not (Test-Path $worktreeDir)) { return }

    $worktrees = Get-ChildItem -LiteralPath $worktreeDir -Directory -ErrorAction SilentlyContinue
    $cleaned = 0

    foreach ($wt in $worktrees) {
        $wtPath = $wt.FullName
        $gitWorktreeOutput = & git worktree list --porcelain 2>$null | Out-String

        if ($gitWorktreeOutput -match [regex]::Escape($wtPath)) {
            $section = ($gitWorktreeOutput -split "`n`n") | Where-Object { $_ -match [regex]::Escape($wtPath) }
            if ($section -match "locked") {
                $shouldRemove = $false
                if ($section -match "pid\s+(\d+)") {
                    $lockPid = [int]$matches[1]
                    $proc = Get-Process -Id $lockPid -ErrorAction SilentlyContinue
                    if (-not $proc) {
                        Write-HeartbeatLog "WARN" "Removing stale locked worktree: $($wt.Name) (PID $lockPid is dead)"
                        $shouldRemove = $true
                    }
                } elseif ($wt.LastWriteTime -lt (Get-Date).AddHours(-24)) {
                    Write-HeartbeatLog "WARN" "Removing stale locked worktree: $($wt.Name) (older than 24h, no PID)"
                    $shouldRemove = $true
                }

                if ($shouldRemove) {
                    & git worktree remove --force $wtPath 2>$null
                    if ($LASTEXITCODE -ne 0) {
                        & git worktree prune 2>$null
                        Remove-Item -LiteralPath $wtPath -Recurse -Force -ErrorAction SilentlyContinue
                    }
                    $cleaned++
                }
            }
        } else {
            Write-HeartbeatLog "WARN" "Removing orphaned worktree directory: $($wt.Name)"
            Remove-Item -LiteralPath $wtPath -Recurse -Force -ErrorAction SilentlyContinue
            $cleaned++
        }
    }

    if ($cleaned -gt 0) {
        Write-HeartbeatLog "INFO" "Cleaned $cleaned stale worktree(s)"
    }
}

# === EFFICIENCY LEDGER ===

function Write-EfficiencyLedger {
    $ledgerFile = Join-Path $Script:BaseDir "API_EFFICIENCY_LEDGER.md"
    $date = Get-Date -Format "yyyy-MM-dd"
    $cycleNum = $Script:OrchCycleCount
    $model = $Script:LastOrchModelDispatched
    $reason = $Script:LastOrchReasonDispatched

    $tokenEstimate = "?"
    $logFile = Join-Path $Script:BaseDir ".claude\agents\orchestrator\agent.log"
    if (Test-Path $logFile) {
        try {
            $logContent = Get-Content -LiteralPath $logFile -Raw -ErrorAction SilentlyContinue
            if ($logContent -match "(\d+)\s*tokens?\s*(used|consumed|burned)") {
                $tokenEstimate = $matches[1]
            } elseif ($logContent -match "budget.*?(\d+)") {
                $tokenEstimate = "~" + $matches[1]
            }
        } catch {}
    }

    $entry = "[$date] - Wake Cycle $(($cycleNum).ToString('000')) ($model) - Tier $($Script:CurrentQuotaTier) - Reason: $reason - Tokens: $tokenEstimate`n"

    try {
        Add-Content -LiteralPath $ledgerFile -Value $entry -Encoding utf8 -ErrorAction Stop
    } catch {
        Write-HeartbeatLog "WARN" "Failed to write efficiency ledger: $($_.Exception.Message)"
    }
}

# === WORKER DISPATCH ===

function Invoke-DispatchWorker {
    param([System.IO.FileInfo]$TaskFile)

    try {
        $content = Get-Content -LiteralPath $TaskFile.FullName -TotalCount 30 -ErrorAction Stop
        $modelLine  = ($content | Where-Object { $_ -match "^#\s*(MODEL|Model):" } | Select-Object -First 1)
        $budgetLine = ($content | Where-Object { $_ -match "^#\s*(BUDGET|Budget):" } | Select-Object -First 1)

        $model  = if ($modelLine  -match ":\s*(.+)") { $matches[1].Trim() } else { $null }
        $budget = if ($budgetLine -match ":\s*(\d+)") { [int]$matches[1] } else { 3600 }

        if (-not $model) {
            Write-HeartbeatLog "WARN" "Task $($TaskFile.Name) has no Model header - triaging"
            $triagedName = "[NEEDS_TRIAGE]_$($TaskFile.Name)"
            $triagedPath = Join-Path $Script:TodoDir $triagedName
            try {
                Rename-Item -LiteralPath $TaskFile.FullName -NewName $triagedName -ErrorAction Stop
                Write-HeartbeatLog "INFO" "Renamed malformed task to $triagedName"
                $Script:WakeOrchestratorForTriage = $true
            } catch {
                Write-HeartbeatLog "ERROR" "Failed to rename malformed task $($TaskFile.Name): $($_.Exception.Message)"
            }
            return $false
        }

        # Quota-aware budget clamping
        if ($Script:MaxBudgetOverride -gt 0 -and $budget -gt $Script:MaxBudgetOverride) {
            Write-HeartbeatLog "WARN" "Budget clamped: ${budget}s -> ${Script:MaxBudgetOverride}s (quota tier $($Script:CurrentQuotaTier))"
            $budget = $Script:MaxBudgetOverride
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

# === ORCHESTRATOR DISPATCH ===

function Invoke-DispatchOrchestrator {
    param([string]$Reason)

    $now = Get-Date
    if (($now - $Script:LastOrchLaunch).TotalSeconds -lt $Script:OrchCooldownSeconds) {
        Write-HeartbeatLog "DEBUG" "Orchestrator cooldown active ($([math]::Round(($now - $Script:LastOrchLaunch).TotalSeconds))s elapsed). Reason: $Reason"
        return $false
    }

    # Select tier-appropriate model
    $orchSelection = Select-OrchestratorModel -Reason $Reason
    $selectedModel = $orchSelection.Model

    Write-HeartbeatLog "INFO" "Dispatching Orchestrator (reason: $Reason, model: $selectedModel)"
    $Script:LastOrchLaunch = $now
    $Script:OrchCompletedCurrentCycle = $false
    $Script:LastOrchModelDispatched = $selectedModel
    $Script:LastOrchReasonDispatched = $Reason

    $mandate = @'
SYSTEM OVERRIDE: Headless Orchestrator Agent.

YOUR JOB:
1. Read HANDOFF/done/ for newly completed tasks since your last scan. Update REMAINING_WORK_TRACKING.md and any affected canonical docs (DOCUMENTATION.md, docs/CURRENT_STATE.md) to reflect completed work.
2. Read HANDOFF/IN_PROGRESS/ for stale tasks (LastWriteTime > 60 min ago). If found, move them back to HANDOFF/todo/ with [STALE]_ prefix.
3. Read HANDOFF/todo/ for [FAILED]_ prefixed tasks. For each: read the file, assess if the task is still relevant. If relevant: DOWNGRADE the model — ALWAYS use a SMALLER model than the one that failed. Replace # MODEL: with the next model DOWN in the hierarchy: deepseek-v4-pro -> deepseek-v3.2 -> qwen3-coder-next -> gemini-3-flash-preview. If the task already failed on the smallest model, move it to HANDOFF/done/ with [UNRESOLVABLE]_ prefix. Reduce budget by 40%. Strip [FAILED]_ prefix, ensure [VALIDATED]_ prefix. If no longer relevant (already done, superseded): move to HANDOFF/done/ with [SUPERSEDED]_ prefix and a note explaining why.
4. Read HANDOFF/todo/ for [STALE]_ prefixed tasks. For each: re-validate that the target code still needs work, update # MODEL: and # BUDGET: if needed, strip the [STALE]_ prefix, and ensure [VALIDATED]_ prefix is present.
5. Read HANDOFF/todo/ for [NEEDS_TRIAGE]_ prefixed tasks. For each, read the file, add the missing # MODEL: and # BUDGET: headers, remove the [NEEDS_TRIAGE]_ prefix, and ensure the [VALIDATED]_ prefix is present.
6. Read HANDOFF/todo/ for unvalidated tasks (missing [VALIDATED]_ prefix). Validate each: check if the target code still needs work. Add [VALIDATED]_ prefix to validated tasks. Reject false positives (already-wired, test-only, golden-strings).
7. Read REMAINING_WORK_TRACKING.md and HANDOFF/backlog/. If remaining work exists not yet in HANDOFF/todo/, create new task files with proper headers:
   # MODEL: <appropriate model from routing table>
   # BUDGET: <seconds based on task complexity>
   # TARGET: <file path>
   Prefix files with [VALIDATED]_ to signal readiness.
8. Assign models per CLAUDE.md routing table:
   - Rust core/identity/crypto/transport/store -> glm-5.1:cloud
   - Crypto/math/security audit -> deepseek-v3.2:cloud
   - Android/Kotlin -> qwen3-coder-next:cloud
   - iOS/Swift -> glm-5.1:cloud
   - Tests/docs/bindings -> gemma4:31b:cloud
   - Quick fix/lint/CI -> gemini-3-flash-preview:cloud
   - Architecture/planning -> deepseek-v4-pro:cloud
   - Code review merge gate -> kimi-k2-thinking:cloud
9. Set budget per task:
   - Micro tasks (lint, format, single-line): 300s
   - Small tasks (single function, test): 900s
   - Medium tasks (multi-file wiring, platform): 1800s
   - Large tasks (module implementation, refactor): 3600s
   - Architecture/review tasks: 5400s
10. Write HANDOFF/ORCHESTRATOR_STATUS.md containing exactly:
   STATUS=completed (or STATUS=ALL_DONE if genuinely nothing remains)
   TASKS_CREATED=N
   TASKS_VALIDATED=N
   STALE_RECLAIMED=N
   FAILED_RETRIAGED=N
   STALE_RETRIAGED=N
   COMPLETED_AT=<timestamp>
   NOTES=<any blockers or observations>

CRITICAL: You MANAGE the queue. Do NOT write application code (.rs, .kt, .swift, .ts). Exit immediately after writing your status file.

RETRY CONSTRAINT: Failed tasks MUST use a model with FEWER parameters than the one that failed. Never escalate. Hierarchy: deepseek-v4-pro:cloud (1.6T) -> deepseek-v3.2:cloud (688B) -> qwen3-coder-next:cloud (81B) -> gemini-3-flash-preview:cloud (lightweight). Failed on smallest = UNRESOLVABLE.

QUOTA CONTEXT: The swarm heartbeat will provide current quota tier information. Use your judgment to determine how aggressively to conserve — when in doubt, prefer smaller models and tighter budgets. You have full autonomy to decide slot allocation and task prioritization within the constraints of the tier.
'@

    $orchWorkDir = Join-Path $Script:BaseDir ".claude\agents\orchestrator"
    $null = New-Item -ItemType Directory -Path $orchWorkDir -Force
    $promptFile = Join-Path $orchWorkDir "prompt.txt"
    $logFile    = Join-Path $orchWorkDir "agent.log"
    $stderrFile = Join-Path $orchWorkDir "stderr.log"
    $statusFile = Join-Path $Script:BaseDir "HANDOFF\ORCHESTRATOR_STATUS.md"

    # Remove stale status from prior runs so we can detect fresh output
    if (Test-Path $statusFile) {
        Remove-Item $statusFile -Force -ErrorAction SilentlyContinue
    }

    # Write mandate to file and pipe via stdin — avoids PowerShell argument-length /
    # character-escaping corruption that breaks -p (matches launch_agent.sh pattern).
    try {
        $mandate | Out-File -LiteralPath $promptFile -Encoding utf8 -ErrorAction Stop
    } catch {
        Write-HeartbeatLog "ERROR" "Failed to write orchestrator prompt file: $($_.Exception.Message)"
        return $false
    }

    $job = Start-Job -Name "Orchestrator" -ArgumentList $promptFile, $logFile, $stderrFile, $Script:BaseDir, $selectedModel -ScriptBlock {
        param($PromptFile, $LogFile, $StderrFile, $BaseDir, $Model)
        Set-Location $BaseDir
        Get-Content -Raw -LiteralPath $PromptFile |
            & ollama launch claude --model $Model -- --dangerously-skip-permissions --print `
                >> $LogFile 2>> $StderrFile
    }
    Write-HeartbeatLog "INFO" "Orchestrator job started (prompt=$(($mandate.Length)) chars, model=$selectedModel, log=$logFile)"
    return $true
}

# === SWARM COMPLETE CHECK ===

function Test-SwarmComplete {
    param([hashtable]$FileState)

    if ($FileState.PendingCount -gt 0)       { return $false }
    if ($FileState.FailedOrStaleCount -gt 0) { return $false }
    if ($FileState.InProgressCount -gt 0)    { return $false }
    if ((Get-ActiveWorkerCount) -gt 0)       { return $false }
    if ((Get-ClaudeProcessCount) -gt 0)      { return $false }
    if (Get-OrchestratorRunning)             { return $false }

    $orchStatusFile = Join-Path $Script:BaseDir "HANDOFF\ORCHESTRATOR_STATUS.md"
    if (Test-Path $orchStatusFile) {
        $statusContent = Get-Content $orchStatusFile -Raw -ErrorAction SilentlyContinue
        if ($statusContent -match "STATUS=ALL_DONE") {
            return $true
        }
    }
    return $false
}

# === ORPHAN CLEANUP ===

function Invoke-CleanupOrphanProcesses {
    param([int]$InProgressCount)

    Write-HeartbeatLog "WARN" "ORPHAN DETECTED: $InProgressCount IN_PROGRESS task(s) but 0 active agent slots. Cleaning up abandoned processes."

    $killed = 0
    try {
        $claudeProcs = @(Get-Process -Name "claude" -ErrorAction SilentlyContinue)
        if ($claudeProcs.Count -gt 0) {
            foreach ($proc in $claudeProcs) {
                try {
                    Stop-Process -Id $proc.Id -Force -ErrorAction Stop
                    Write-HeartbeatLog "INFO" "Killed orphan claude.exe PID=$($proc.Id)"
                    $killed++
                } catch {
                    Write-HeartbeatLog "WARN" "Could not kill claude.exe PID=$($proc.Id): $($_.Exception.Message)"
                }
            }
        }

        $javaProcs = @(Get-Process -Name "java" -ErrorAction SilentlyContinue)
        if ($javaProcs.Count -gt 0) {
            foreach ($proc in $javaProcs) {
                try {
                    Stop-Process -Id $proc.Id -Force -ErrorAction Stop
                    Write-HeartbeatLog "INFO" "Killed orphan java PID=$($proc.Id) (WS=$([math]::Round($proc.WorkingSet64/1MB,1))MB)"
                    $killed++
                } catch {
                    Write-HeartbeatLog "WARN" "Could not kill java PID=$($proc.Id): $($_.Exception.Message)"
                }
            }
        }
    } catch {
        Write-HeartbeatLog "ERROR" "Orphan process cleanup error: $($_.Exception.Message)"
    }

    Write-HeartbeatLog "INFO" "Orphan cleanup complete: $killed process(es) terminated"
    return $killed
}

# === HEARTBEAT PULSE ===

function Invoke-HeartbeatPulse {
    Write-HeartbeatLog "DEBUG" "=== PULSE $(Get-Date -Format 'HH:mm:ss') ==="
    $Script:PulseCount++

    # Re-evaluate quota every N pulses to avoid file thrash
    if ($Script:PulseCount % $Script:QuotaCheckInterval -eq 0) {
        Invoke-QuotaGovernor
    }

    # Phase 1: Cleanup completed jobs
    $completedJobs = Get-Job -State Completed -ErrorAction SilentlyContinue
    foreach ($j in $completedJobs) {
        if ($j.Name -eq "Orchestrator") {
            $Script:OrchCompletedCurrentCycle = $true
            $Script:OrchCycleCount++
            Write-HeartbeatLog "INFO" "Orchestrator job completed (cycle $($Script:OrchCycleCount))"

            # Dump stderr tail for diagnostics
            $stderrFile = Join-Path $Script:BaseDir ".claude\agents\orchestrator\stderr.log"
            if ((Test-Path $stderrFile) -and ((Get-Item $stderrFile).Length -gt 0)) {
                $stderrTail = Get-Content -LiteralPath $stderrFile -Tail 5 -ErrorAction SilentlyContinue
                if ($stderrTail) {
                    Write-HeartbeatLog "DEBUG" "Orch stderr (last 5 lines):"
                    foreach ($line in $stderrTail) { Write-HeartbeatLog "DEBUG" "  | $line" }
                }
            }

            # Log to efficiency ledger
            Write-EfficiencyLedger

            $orchStatusFile = Join-Path $Script:BaseDir "HANDOFF\ORCHESTRATOR_STATUS.md"
            if (Test-Path $orchStatusFile) {
                $statusContent = Get-Content $orchStatusFile -Raw -ErrorAction SilentlyContinue
                Write-HeartbeatLog "INFO" "Orchestrator status written:"
                foreach ($line in ($statusContent -split "`n" | Where-Object { $_ -match '\S' })) {
                    Write-HeartbeatLog "INFO" "  $($line.Trim())"
                }
                if ($statusContent -match "STATUS=ALL_DONE") {
                    Write-HeartbeatLog "INFO" "Orchestrator reports ALL_DONE - swarm may be complete"
                }
            } else {
                Write-HeartbeatLog "WARN" "Orchestrator completed but did NOT write ORCHESTRATOR_STATUS.md"
            }
        }
        Receive-Job -Job $j 2>$null | Out-Null
        Remove-Job -Job $j -Force -ErrorAction SilentlyContinue
    }

    # Phase 2: Scan file system
    $fileState      = Get-SwarmFileState
    $slotState      = Get-ActiveSlotCount
    $workerCount    = $slotState.WorkerJobs
    $claudeCount    = $slotState.ClaudeProcs
    $orchRunning    = Get-OrchestratorRunning
    $staleTasks     = Get-StaleInProgressTasks
    $slotsFree      = $slotState.SlotsFree
    $orchLabel      = if ($orchRunning) { "Y" } else { "N" }
    $orphanLabel    = if ($fileState.OrphanInProgress) { "ORPHAN" } else { "ok" }
    $tierLabel      = "T$($Script:CurrentQuotaTier)"

    Write-HeartbeatLog "INFO" ("State -> Done: {0} | Pending: {1} | InProg: {2} | Failed/Stale: {3} | StaleInProg: {4} | Slots: jobs={5} claude={6} free={7} orphan={8} tier={9} | Orch: {10}" -f
        $fileState.DoneCount, $fileState.PendingCount, $fileState.InProgressCount,
        $fileState.FailedOrStaleCount, $staleTasks.Count, $workerCount, $claudeCount, $slotsFree, $orphanLabel, $tierLabel, $orchLabel)

    $Script:PrevDoneCount = $fileState.DoneCount

    # Phase 3: Orphan cleanup (IN_PROGRESS tasks but no active slots)
    if ($fileState.OrphanInProgress) {
        $null = Invoke-CleanupOrphanProcesses -InProgressCount $fileState.InProgressCount
    }

    # Phase 4: Exit check
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

    # Phase 5: Launch worker (at most 1 per pulse)
    if ($fileState.PendingCount -gt 0 -and $slotsFree -gt 0) {
        $orchSlots = if ($orchRunning) { 1 } else { 0 }
        $slotsForWorkers = $Script:MaxConcurrentSlots - $orchSlots
        if (([math]::Max($workerCount, $claudeCount)) -lt $slotsForWorkers) {
            # At Tier 4, only dispatch P0 tasks
            if ($Script:Tier4Active) {
                $task = $fileState.PendingTasks | Where-Object { $_.Name -match "p0|P0|BLOCKED_BY_QUOTA" } | Select-Object -First 1
                if (-not $task) {
                    Write-HeartbeatLog "DEBUG" "Tier 4: no P0 tasks available to dispatch"
                }
            } else {
                $task = $fileState.PendingTasks | Select-Object -First 1
            }
            if ($task) {
                Invoke-DispatchWorker -TaskFile $task
            }
        }
    }

    # Phase 6: Launch orchestrator (when all workers idle, failed/stale need triage, or periodic)
    # HARD GATE: orchestrator consumes 1 slot. Never dispatch if all slots are full —
    # that would exceed MaxConcurrentSlots (claude.exe count = workers + orchestrator).
    if (-not $orchRunning) {
        $needsOrch = $false
        $orchReason = ""

        # Slot-aware dispatch paths (workers may be running — check capacity first)
        if ($Script:WakeOrchestratorForTriage) {
            if ($slotsFree -gt 0) {
                $needsOrch = $true
                $orchReason = "malformed task(s) need triage in todo/"
                $Script:WakeOrchestratorForTriage = $false
            }
            # else: no free slot; keep flag set, retry next pulse
        } elseif ($fileState.FailedOrStaleCount -gt 0) {
            if ($slotsFree -gt 0) {
                $needsOrch = $true
                $orchReason = "$($fileState.FailedOrStaleCount) failed/stale task(s) in todo/ need retriage"
            } else {
                Write-HeartbeatLog "DEBUG" "Deferring orchestrator: $($fileState.FailedOrStaleCount) failed/stale tasks need triage but no free slots (workers=$workerCount claude=$claudeCount)"
            }
        } elseif (([math]::Max($workerCount, $claudeCount)) -eq 0) {
            # All workers idle — slots are guaranteed available
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
        }

        if ($needsOrch) {
            Invoke-DispatchOrchestrator -Reason $orchReason
            Start-Sleep -Seconds 5
        }
    }
}

function Invoke-HeartbeatBoot {
    Write-HeartbeatLog "INFO" "============================================"
    Write-HeartbeatLog "INFO" "SCMessenger Swarm Heartbeat v3.0 BOOTING"
    Write-HeartbeatLog "INFO" "Base: $($Script:BaseDir)"
    Write-HeartbeatLog "INFO" "Max Concurrent Slots: $($Script:MaxConcurrentSlots)"
    Write-HeartbeatLog "INFO" "Orch Models: triage=$($Script:OrchModelTriage) standard=$($Script:OrchModelStandard) heavy=$($Script:OrchModelHeavy)"
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
    Write-HeartbeatLog "INFO" "Initial state -- Done: $($initial.DoneCount) | Pending: $($initial.PendingCount) | InProg: $($initial.InProgressCount) | Failed/Stale: $($initial.FailedOrStaleCount) | Orphan: $($initial.OrphanInProgress)"

    # Phase 5: Clean up orphaned PS jobs from prior session
    Get-Job | Remove-Job -Force -ErrorAction SilentlyContinue
    Write-HeartbeatLog "INFO" "Cleaned up orphaned PowerShell jobs"

    # Phase 6: Cleanup stale worktrees from prior sessions
    Invoke-CleanupStaleWorktrees

    # Phase 7: Config drift detection (informational only — heartbeat is authority)
    $poolConfig = Join-Path $Script:BaseDir ".claude\agent_pool.json"
    if (Test-Path $poolConfig) {
        try {
            $poolJson = Get-Content -Raw -LiteralPath $poolConfig | ConvertFrom-Json
            if ($poolJson.max_concurrent -ne $Script:MaxConcurrentSlots) {
                Write-HeartbeatLog "WARN" "CONFIG DRIFT: agent_pool.json max_concurrent=$($poolJson.max_concurrent) vs SwarmHeartbeat MaxConcurrentSlots=$($Script:MaxConcurrentSlots)"
                Write-HeartbeatLog "INFO" "SwarmHeartbeat.ps1 is the operational authority. Using MaxConcurrentSlots=$($Script:MaxConcurrentSlots)"
            }
        } catch {
            Write-HeartbeatLog "WARN" "Could not parse agent_pool.json for drift check"
        }
    }

    # Phase 8: Kill abandoned processes from prior session (IN_PROGRESS tasks but no agents)
    if ($initial.OrphanInProgress) {
        Write-HeartbeatLog "WARN" "Boot-time orphan detected: $($initial.InProgressCount) IN_PROGRESS task(s) with no active agents"
        $null = Invoke-CleanupOrphanProcesses -InProgressCount $initial.InProgressCount
    }

    # Phase 9: Initialize quota governor
    Invoke-QuotaGovernor

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
