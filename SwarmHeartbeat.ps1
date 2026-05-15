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
$Script:ScraperIntervalMinutes = 5
$Script:ScraperStaleMinutes    = 5
$Script:ModelAllowlistStaleMinutes = 5

# Runtime state
$Script:LastScraperRun               = [datetime]::MinValue
$Script:LastOrchLaunch              = [datetime]::MinValue
$Script:OrchCompletedCurrentCycle    = $false
$Script:PrevDoneCount                = 0
$Script:WakeOrchestratorForTriage    = $false
$Script:PulseCount                   = 0
$Script:OrchCycleCount               = 0
$Script:CurrentQuotaTier             = 1
$Script:CurrentPhaseName             = "HEAVY-LIFT"
$Script:OrchModelTier                = "qwen3-coder:480b:cloud"
$Script:WorkerModelAllowList         = @()
$Script:MaxBudgetOverride            = 0
$Script:LastOrchModelDispatched      = ""
$Script:LastOrchReasonDispatched     = ""
$Script:ShutdownRequested            = $false
$Script:CachedClaudeCount            = $null
$Script:CachedClaudeTimestamp        = [datetime]::MinValue
$Script:LastModelAllowlistRefresh    = [datetime]::MinValue
$Script:DynamicModelAllowList        = @()

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
    $now = Get-Date
    # Cache for 3 seconds to avoid redundant costly cross-process calls within a single pulse
    if ($Script:CachedClaudeCount -ne $null -and ($now - $Script:CachedClaudeTimestamp).TotalSeconds -lt 3) {
        return $Script:CachedClaudeCount
    }

    $job = Start-Job -ScriptBlock {
        @(Get-Process -Name "claude" -ErrorAction SilentlyContinue).Count
    }
    $null = Wait-Job -Job $job -Timeout 5
    if ($job.State -eq 'Completed') {
        $result = Receive-Job -Job $job
        Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
        $Script:CachedClaudeCount = $result
        $Script:CachedClaudeTimestamp = $now
        return $result
    } else {
        Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
        Write-HeartbeatLog "WARN" "Get-Process for claude.exe timed out (5s) -- using cached or default value"
        if ($Script:CachedClaudeCount -ne $null) { return $Script:CachedClaudeCount }
        return 0
    }
}

function Get-TrackedAgentCount {
    $workerCount = Get-ActiveWorkerCount
    $claudeCount = Get-ClaudeProcessCount

    # When worker jobs are active, claude.exe IS the swarm agent.
    # Each worker spawns via "ollama launch claude" which creates claude.exe.
    # PID-file tracking is supplementary; claude.exe is the canonical indicator.
    if ($workerCount -gt 0) {
        return $claudeCount
    }

    # No active workers: fall back to PID-file tracking for orphan/stale detection
    $agentsDir = Join-Path $Script:BaseDir ".claude\agents"
    if (-not (Test-Path $agentsDir)) { return 0 }

    $tracked = 0
    $agentDirs = Get-ChildItem -LiteralPath $agentsDir -Directory -ErrorAction SilentlyContinue
    foreach ($dir in $agentDirs) {
        $pidFile = Join-Path $dir.FullName "pid"
        if (Test-Path $pidFile) {
            try {
                $pid = [int](Get-Content -LiteralPath $pidFile -Raw -ErrorAction Stop).Trim()
                $proc = Get-Process -Id $pid -ErrorAction SilentlyContinue
                if ($proc) { $tracked++ }
            } catch {
                # Stale PID file -- ignore
            }
        }
    }
    return $tracked
}

function Get-ActiveSlotCount {
    $workerCount       = Get-ActiveWorkerCount
    $claudeCount       = Get-ClaudeProcessCount
    $trackedCount      = Get-TrackedAgentCount
    $orchRunning       = Get-OrchestratorRunning
    $orchSlots         = if ($orchRunning) { 1 } else { 0 }

    # PID cross-check: claude.exe is master indicator; PID files are supplementary
    if ($claudeCount -gt ($trackedCount + $orchSlots)) {
        Write-HeartbeatLog "WARN" "claude.exe leak detected: OS reports $claudeCount claude.exe but only $trackedCount tracked by swarm (+$orchSlots orch slot). Possible orphaned agent processes."
    } elseif ($claudeCount -lt ($workerCount + $orchSlots)) {
        Write-HeartbeatLog "DEBUG" "$workerCount worker job(s) pending process spawn (claude=$claudeCount tracked=$trackedCount orch=$orchSlots)"
    }

    return @{
        WorkerJobs    = $workerCount
        ClaudeProcs   = $claudeCount
        TrackedAgents = $trackedCount
        OrchSlots     = $orchSlots
        TotalUsed     = [math]::Max($workerCount, $claudeCount) + $orchSlots
        SlotsFree     = $Script:MaxConcurrentSlots - ([math]::Max($workerCount, $claudeCount) + $orchSlots)
    }
}

function Get-SwarmFileState {
    $doneFiles    = @(Get-ChildItem -LiteralPath $Script:DoneDir -Filter "*.md" -ErrorAction SilentlyContinue)
    $pendingFiles = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[VALIDATED]_*.md" -ErrorAction SilentlyContinue)
    $inProgFiles  = @(Get-ChildItem -LiteralPath $Script:InProgDir -Filter "*.md" -ErrorAction SilentlyContinue)
    $failedFiles  = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[FAILED]_*.md" -ErrorAction SilentlyContinue)
    $staleFiles   = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[STALE]_*.md" -ErrorAction SilentlyContinue)

    # Compute orphan status without Get-ActiveSlotCount (which has leak-detection logging)
    # Orphan: IN_PROGRESS files exist but no active slots of any kind
    $orphanInProg = ($inProgFiles.Count -gt 0) -and ((Get-ClaudeProcessCount) -eq 0) -and ((Get-ActiveWorkerCount) -eq 0) -and (-not (Get-OrchestratorRunning))

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
            # Use dispatched_at marker as canonical timestamp; Move-Item preserves
            # the original file's LastWriteTime which may be hours old, causing
            # freshly-dispatched tasks to be misdetected as stale.
            $agentName = [System.IO.Path]::GetFileNameWithoutExtension($f.Name)
            $dispatchMarker = Join-Path $Script:BaseDir ".claude\agents\$agentName\dispatched_at"
            $refTime = $f.LastWriteTime
            if (Test-Path $dispatchMarker) {
                try {
                    $refTime = [datetime]::Parse((Get-Content -LiteralPath $dispatchMarker -Raw -ErrorAction Stop).Trim())
                } catch {
                    # Fall back to file time if marker is unreadable
                }
            }
            if ($refTime -lt $cutoff) {
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

function Invoke-RefreshModelAllowlist {
    $now = Get-Date
    $sinceLast = ($now - $Script:LastModelAllowlistRefresh).TotalMinutes
    if ($sinceLast -lt $Script:ModelAllowlistStaleMinutes -and $Script:DynamicModelAllowList.Count -gt 0) {
        Write-HeartbeatLog "DEBUG" "Model allowlist is fresh ($([math]::Round($sinceLast,1))min old, $($Script:DynamicModelAllowList.Count) models) -- skipping fetch"
        return $Script:DynamicModelAllowList
    }

    Write-HeartbeatLog "INFO" "Fetching available models from ollama.com/api/tags..."
    $Script:LastModelAllowlistRefresh = $now

    try {
        $response = Invoke-RestMethod -Uri "https://ollama.com/api/tags" -TimeoutSec 15 -ErrorAction Stop
        $models = @($response.models | ForEach-Object { "$($_.name):cloud" })

        if ($models.Count -eq 0) {
            Write-HeartbeatLog "WARN" "ollama.com/api/tags returned 0 models -- using cached list"
            return $Script:DynamicModelAllowList
        }

        $Script:DynamicModelAllowList = $models
        Write-HeartbeatLog "INFO" "Refreshed model allowlist: $($models.Count) models available from ollama.com"
        return $models
    } catch {
        Write-HeartbeatLog "ERROR" "Failed to fetch model allowlist from ollama.com: $($_.Exception.Message)"
        if ($Script:DynamicModelAllowList.Count -gt 0) {
            Write-HeartbeatLog "WARN" "Using cached allowlist ($($Script:DynamicModelAllowList.Count) models)"
        }
        return $Script:DynamicModelAllowList
    }
}

# === QUOTA GOVERNOR ===

function Read-QuotaState {
    $fiveHour    = 0
    $sevenDay    = 0
    $resetMins   = $null

    # Prefer structured JSON if available (written by OllamaQuotaScraper.ps1)
    $jsonFile = Join-Path $Script:BaseDir ".claude\quota_state.json"
    if (Test-Path $jsonFile) {
        try {
            $json = Get-Content -Raw -LiteralPath $jsonFile -ErrorAction Stop | ConvertFrom-Json
            if ($json.status -eq "ok") {
                $fiveHour  = [double]$json.fiveHour
                $sevenDay  = [double]$json.sevenDay
                $resetMins = $json.resetMinutes
            }
        } catch {
            Write-HeartbeatLog "DEBUG" "quota_state.json parse failed, falling back to markdown"
        }
    }

    # Fallback: parse markdown file (legacy format)
    if ($fiveHour -eq 0 -and $sevenDay -eq 0) {
        $quotaFile = Join-Path $Script:BaseDir ".claude\API_QUOTA_STATE.md"
        if (Test-Path $quotaFile) {
            try {
                $content = Get-Content -LiteralPath $quotaFile -Raw -ErrorAction Stop
                if ($content -match "5-Hour Usage.*?([\d\.]+)%") { $fiveHour = [double]$matches[1] }
                if ($content -match "7-Day Usage.*?([\d\.]+)%")  { $sevenDay = [double]$matches[1] }
                if ($content -match "resets?\s+in\s+~?(\d+)\s*min") { $resetMins = [int]$matches[1] }
            } catch {
                Write-HeartbeatLog "ERROR" "Failed to read quota markdown: $($_.Exception.Message)"
            }
        } else {
            Write-HeartbeatLog "WARN" "No quota state file found; defaulting to Tier 1"
        }
    }

    # 6-tier phased execution: match task weight to quota abundance
    # HARDLOCK (Tier 6) if either 5-hour or 7-day exceeds 99.5%
    if ($fiveHour -gt 99.5 -or $sevenDay -gt 99.5) {
        $tier = 6
    } elseif ($fiveHour -gt 90) {
        $tier = 5
    } elseif ($fiveHour -gt 75) {
        $tier = 4
    } elseif ($fiveHour -gt 50) {
        $tier = 3
    } elseif ($fiveHour -gt 25) {
        $tier = 2
    } else {
        $tier = 1
    }

    return @{ FiveHour = $fiveHour; SevenDay = $sevenDay; Tier = $tier; ResetMinutes = $resetMins }
}

function Invoke-QuotaRefresh {
    # Check if quota data is fresh enough
    $jsonFile = Join-Path $Script:BaseDir ".claude\quota_state.json"
    if (Test-Path $jsonFile) {
        try {
            $json = Get-Content -Raw -LiteralPath $jsonFile -ErrorAction Stop | ConvertFrom-Json
            if ($json.timestamp) {
                $lastUpdate = [datetime]::Parse($json.timestamp)
                $ageMinutes = ((Get-Date) - $lastUpdate).TotalMinutes
                if ($ageMinutes -lt $Script:ScraperStaleMinutes) {
                    Write-HeartbeatLog "DEBUG" "Quota data is fresh ($([math]::Round($ageMinutes,1))min old) -- skipping scrape"
                    return $true
                }
            }
        } catch {}
    }

    # Also check cooldown to avoid scraping too aggressively
    $sinceLast = ((Get-Date) - $Script:LastScraperRun).TotalMinutes
    if ($sinceLast -lt $Script:ScraperIntervalMinutes) {
        Write-HeartbeatLog "DEBUG" "Scraper cooldown active ($([math]::Round($sinceLast,1))min since last run)"
        return $true
    }

    Write-HeartbeatLog "INFO" "Refreshing quota data via OllamaQuotaScraper.ps1..."
    $Script:LastScraperRun = Get-Date

    $scraperPath = Join-Path $Script:BaseDir "OllamaQuotaScraper.ps1"
    if (-not (Test-Path $scraperPath)) {
        Write-HeartbeatLog "ERROR" "OllamaQuotaScraper.ps1 not found at $scraperPath"
        return $false
    }

    try {
        $result = & "$scraperPath" -Quiet 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-HeartbeatLog "WARN" "Scraper exited with code $LASTEXITCODE -- will use cached data"
            return $false
        }
        Write-HeartbeatLog "INFO" "Quota refresh complete"
        return $true
    } catch {
        Write-HeartbeatLog "WARN" "Quota refresh failed: $($_.Exception.Message) -- will use cached data"
        return $false
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
        6 {
            $Script:MaxConcurrentSlots    = 0
            $Script:OrchCooldownSeconds   = 9999
            $Script:MaxBudgetOverride     = 0
            $Script:CurrentPhaseName      = "HARDLOCK"
            $Script:OrchModelTier         = $null
            Write-HeartbeatLog "ERROR" "HARDLOCK ACTIVE: 5hr=$($quota.FiveHour)% 7d=$($quota.SevenDay)%. ZERO dispatch. Polling until reset."
        }
        5 {
            $Script:MaxConcurrentSlots    = 1
            $Script:OrchCooldownSeconds   = 600
            $Script:MaxBudgetOverride     = 300
            $Script:CurrentPhaseName      = "MICRO"
            $Script:OrchModelTier         = "deepseek-v4-pro:cloud"
            Write-HeartbeatLog "WARN" "MICRO PHASE: Single-slot, max budget 300s, P0/defer-only. Orchestrator: deepseek-v4-pro"
        }
        4 {
            $Script:MaxConcurrentSlots    = 2
            $Script:OrchCooldownSeconds   = 300
            $Script:MaxBudgetOverride     = 900
            $Script:CurrentPhaseName      = "LIGHT"
            $Script:OrchModelTier         = "deepseek-v4-pro:cloud"
            Write-HeartbeatLog "WARN" "LIGHT PHASE: 2-slot, docs/tests/lint/bindings only, max budget 900s. Orchestrator: deepseek-v4-pro"
        }
        3 {
            $Script:MaxConcurrentSlots    = 2
            $Script:OrchCooldownSeconds   = 180
            $Script:MaxBudgetOverride     = 1800
            $Script:CurrentPhaseName      = "MIXED"
            $Script:OrchModelTier         = "deepseek-v4-pro:cloud"
            Write-HeartbeatLog "INFO" "MIXED PHASE: 2-slot, smaller features/validation/testing, max budget 1800s. Orchestrator: deepseek-v4-pro"
        }
        2 {
            $Script:MaxConcurrentSlots    = 3
            $Script:OrchCooldownSeconds   = 120
            $Script:MaxBudgetOverride     = 5400
            $Script:CurrentPhaseName      = "EXECUTE"
            $Script:OrchModelTier         = "kimi-k2.6:cloud"
            Write-HeartbeatLog "INFO" "EXECUTE PHASE: 3-slot, major feature implementation, max budget 5400s"
        }
        default {
            $Script:MaxConcurrentSlots    = 3
            $Script:OrchCooldownSeconds   = 120
            $Script:MaxBudgetOverride     = 0
            $Script:CurrentPhaseName      = "HEAVY-LIFT"
            $Script:OrchModelTier         = "qwen3-coder:480b:cloud"
            Write-HeartbeatLog "INFO" "HEAVY-LIFT PHASE: 3-slot, flagship models, no budget cap. Token-heavy work: multi-file wiring, architecture, deep planning."
        }
    }

    # Resolve worker model allowlist from dynamic source (ollama.com/api/tags).
    # Hardlock: empty list. Tier 5 (MICRO): minimal safe list to avoid burning
    # critically-scarce quota on heavy models. All other tiers: full dynamic list.
    if ($quota.Tier -ge 6) {
        $Script:WorkerModelAllowList = @()
    } elseif ($quota.Tier -ge 5) {
        $Script:WorkerModelAllowList = @("gemini-3-flash-preview:cloud")
    } elseif ($Script:DynamicModelAllowList.Count -gt 0) {
        $Script:WorkerModelAllowList = $Script:DynamicModelAllowList
    } else {
        # Bootstrap: dynamic list hasn't loaded yet, allow anything
        $Script:WorkerModelAllowList = @()
    }
}

# === ORCHESTRATOR MODEL SELECTION ===

function Select-OrchestratorModel {
    param([string]$Reason)

    # HARDLOCK: no orchestrator dispatch allowed
    if ($Script:CurrentQuotaTier -ge 6) {
        Write-HeartbeatLog "ERROR" "Orchestrator dispatch blocked: HARDLOCK active"
        return $null
    }

    # Use the tier-assigned orchestrator model
    $model = $Script:OrchModelTier
    if (-not $model) {
        $model = $Script:OrchModelStandard
    }

    # At MICRO phase (Tier 5), force triage model regardless
    if ($Script:CurrentQuotaTier -ge 5) {
        Write-HeartbeatLog "INFO" "Orchestrator model: $model (quota phase: $($Script:CurrentPhaseName))"
        return @{ Model = $model; MandateType = "micro" }
    }

    # At LIGHT/MIXED (Tier 3-4), use tier-assigned model
    if ($Script:CurrentQuotaTier -ge 3) {
        return @{ Model = $model; MandateType = "standard" }
    }

    # EXECUTE/HEAVY-LIFT (Tier 1-2): select based on reason complexity
    if ($Reason -match "malformed|triage|NEEDS_TRIAGE") {
        return @{ Model = $Script:OrchModelTriage; MandateType = "triage" }
    }
    elseif ($Reason -match "backlog|drained|remaining") {
        return @{ Model = $model; MandateType = "heavy" }
    }
    else {
        return @{ Model = $model; MandateType = "standard" }
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

    # HARDLOCK gate: refuse all dispatch at Tier 6
    if ($Script:CurrentQuotaTier -ge 6) {
        Write-HeartbeatLog "WARN" "HARDLOCK: refusing worker dispatch for $($TaskFile.Name)"
        return $false
    }

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
            Write-HeartbeatLog "WARN" "Budget clamped: ${budget}s -> ${Script:MaxBudgetOverride}s (phase: $($Script:CurrentPhaseName))"
            $budget = $Script:MaxBudgetOverride
        }

        # Worker model allowlist enforcement (Tier 3+)
        if ($Script:WorkerModelAllowList.Count -gt 0) {
            $modelBase = $model -replace ':cloud$', ''
            $allowed = $false
            foreach ($allowedModel in $Script:WorkerModelAllowList) {
                $allowedBase = $allowedModel -replace ':cloud$', ''
                if ($model -eq $allowedModel -or $modelBase -eq $allowedBase) {
                    $allowed = $true
                    break
                }
            }
            if (-not $allowed) {
                Write-HeartbeatLog "WARN" "Worker model $model not in allowlist for phase $($Script:CurrentPhaseName). Allowlist: $($Script:WorkerModelAllowList -join ', ')"
                # Downgrade to first allowed model
                $model = $Script:WorkerModelAllowList[0]
                Write-HeartbeatLog "INFO" "Downgraded worker model to $model"
            }
        }

        Write-HeartbeatLog "INFO" "Dispatching Worker for $($TaskFile.Name) -> model=$model budget=${budget}s"

        $inProgressPath = Join-Path $Script:InProgDir $TaskFile.Name
        Move-Item -LiteralPath $TaskFile.FullName -Destination $inProgressPath -Force -ErrorAction Stop

        # Build quota context for TaskGovernor
        $quotaState = Read-QuotaState
        $quotaContext = @{
            FiveHour      = $quotaState.FiveHour
            SevenDay      = $quotaState.SevenDay
            Phase         = $Script:CurrentPhaseName
            Tier          = $Script:CurrentQuotaTier
            Budget        = $budget
            ResetMinutes  = if ($quotaState.ResetMinutes) { $quotaState.ResetMinutes } else { "?" }
        }

        $jobName = "Worker_" + (Get-Date -Format "HHmmss")
        $job = Start-Job -Name $jobName -ArgumentList $inProgressPath, $model, $budget, $Script:BaseDir, $quotaContext -ScriptBlock {
            param($TaskFile, $Model, $BudgetLimit, $BaseDir, $QuotaContext)
            Set-Location $BaseDir
            $qcJson = ($QuotaContext | ConvertTo-Json -Compress)
            & "$BaseDir\TaskGovernor.ps1" -TaskFile $TaskFile -Model $Model -BudgetLimit $BudgetLimit -QuotaContextJson $qcJson
        }

        # Capture PID for cross-check tracking after process spawn
        $agentName = [System.IO.Path]::GetFileNameWithoutExtension($TaskFile.Name)
        $agentDir = Join-Path $Script:BaseDir ".claude\agents\$agentName"
        $null = New-Item -ItemType Directory -Path $agentDir -Force
        # Record dispatch timestamp for staleness checks
        $dispatchMarker = Join-Path $agentDir "dispatched_at"
        Get-Date -Format "o" | Out-File -LiteralPath $dispatchMarker -Encoding utf8 -ErrorAction SilentlyContinue
        # Give the job a moment to spawn ollama, then capture child claude.exe PID
        try {
            Start-Sleep -Milliseconds 500
            $claudeProcs = @(Get-Process -Name "claude" -ErrorAction SilentlyContinue | Sort-Object StartTime -Descending)
            if ($claudeProcs.Count -gt 0) {
                $pidFile = Join-Path $agentDir "pid"
                $claudeProcs[0].Id | Out-File -LiteralPath $pidFile -Encoding utf8 -ErrorAction SilentlyContinue
            }
        } catch {
            # PID tracking is best-effort
        }

        Write-HeartbeatLog "INFO" "Worker dispatched: job=$jobName file=$($TaskFile.Name)"
        return $true
    } catch {
        Write-HeartbeatLog "ERROR" "Worker dispatch failed for $($TaskFile.Name): $($_.Exception.Message)"
        return $false
    }
}

# === ORCHESTRATOR MANDATE GENERATOR ===

function New-OrchestratorMandate {
    param(
        [int]$Tier,
        [string]$Phase,
        [double]$FiveHour,
        [double]$SevenDay,
        [int]$Slots,
        [int]$MaxBudget,
        [string[]]$WorkerModels
    )

    $quotaLine = "QUOTA: 5hr=$FiveHour% 7d=$SevenDay% | Phase: $Phase | Slots: $Slots"
    if ($MaxBudget -gt 0) {
        $quotaLine += " | Max budget: ${MaxBudget}s"
    } else {
        $quotaLine += " | Budget: unlimited"
    }

    $workerModelLine = if ($WorkerModels.Count -gt 0) { "Allowed worker models: $($WorkerModels -join ', ')" } else { "Any flagship model allowed for workers" }

    switch ($Tier) {
        1 {
            return @"
$quotaLine
$workerModelLine

Quota is abundant. Prioritize token-heavy work that is most efficient with flagship
models -- multi-file wiring, architecture changes, deep planning, complex integrations.
Also create detailed, well-scoped task files that small models can execute in later
phases. Route aggressively: heavy models for heavy work, but don't waste large models
on tasks a small model could handle. Queue ambitious work now while budget is unlimited.

STANDARD ORCHESTRATOR DUTIES:
1. Read HANDOFF/done/ for newly completed tasks. Update REMAINING_WORK_TRACKING.md.
2. Read HANDOFF/IN_PROGRESS/ for stale tasks (>60 min). Reclaim to todo/ with [STALE]_ prefix.
3. Read HANDOFF/todo/ for [FAILED]_ tasks. Downgrade model, reduce budget 40%, re-validate.
4. Read HANDOFF/todo/ for [STALE]_ tasks. Re-validate, restore [VALIDATED]_ prefix.
5. Read HANDOFF/todo/ for [NEEDS_TRIAGE]_ tasks. Add # MODEL: and # BUDGET: headers.
6. Validate unvalidated tasks. Add [VALIDATED]_ prefix or reject.
7. Create new task files from REMAINING_WORK_TRACKING.md gaps with proper headers.
8. Assign models per routing table. Set budgets based on complexity.
9. Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed.

CRITICAL: You MANAGE the queue. Do NOT write application code. Exit after writing status file.
RETRY CONSTRAINT: Failed tasks MUST downgrade model. Never escalate. Failed on smallest = UNRESOLVABLE.
"@
        }
        2 {
            return @"
$quotaLine
$workerModelLine

Standard orchestrator duties. Dispatch implementation tasks per CLAUDE.md routing table.
Prefer queuing any remaining heavy-lift work now -- the window for flagship models is
closing. Start routing docs/tests/lint to smaller models.

STANDARD ORCHESTRATOR DUTIES:
1. Read HANDOFF/done/ for newly completed tasks. Update REMAINING_WORK_TRACKING.md.
2. Read HANDOFF/IN_PROGRESS/ for stale tasks (>60 min). Reclaim to todo/ with [STALE]_ prefix.
3. Read HANDOFF/todo/ for [FAILED]_ tasks. Downgrade model, reduce budget 40%, re-validate.
4. Read HANDOFF/todo/ for [STALE]_ tasks. Re-validate, restore [VALIDATED]_ prefix.
5. Read HANDOFF/todo/ for [NEEDS_TRIAGE]_ tasks. Add # MODEL: and # BUDGET: headers.
6. Validate unvalidated tasks. Add [VALIDATED]_ prefix or reject.
7. Create new task files from REMAINING_WORK_TRACKING.md gaps with proper headers.
8. Assign models per routing table. Set budgets based on complexity.
9. Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed.

CRITICAL: You MANAGE the queue. Do NOT write application code. Exit after writing status file.
RETRY CONSTRAINT: Failed tasks MUST downgrade model. Never escalate. Failed on smallest = UNRESOLVABLE.
"@
        }
        3 {
            return @"
$quotaLine
$workerModelLine

Standard orchestrator duties. Dispatch implementation tasks. Avoid large multi-file
refactors. Route validation/testing/docs to smaller models. Budgets clamped to ${MaxBudget}s.

STANDARD ORCHESTRATOR DUTIES: (same as above -- scan, triage, validate, create, route)
Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed when done.
CRITICAL: You MANAGE the queue. Do NOT write application code. Exit after writing status file.
"@
        }
        4 {
            return @"
$quotaLine
$workerModelLine

Quota is tight. Only create/dispatch tasks executable by gemma4:31b or
gemini-3-flash-preview: docs, tests, lint, bindings, P0 fixes. Defer ALL feature
work and medium+ refactors to next quota window. Break remaining cleanup into
small chunks (<=${MaxBudget}s each).

ORCHESTRATOR DUTIES (abbreviated):
1. Scan done/, IN_PROGRESS/, todo/.
2. Triage [FAILED]_ and [STALE]_ tasks -- downgrade aggressively.
3. Create ONLY small-model tasks (docs/tests/lint/bindings/P0).
4. DEFER everything else.
5. Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed.
CRITICAL: Exit after writing status file. Do NOT write application code.
"@
        }
        5 {
            return @"
$quotaLine
$workerModelLine

Quota is critically low. ONLY create/dispatch micro-tasks for gemini-3-flash-preview:
P0 fixes, lint, single-line changes. Budget capped at ${MaxBudget}s. ALL other work
MUST be deferred to next quota window. PARTIAL COMPLETION IS ACCEPTABLE.

ORCHESTRATOR DUTIES (micro only):
1. Quick scan of done/ and todo/.
2. Create ONLY gemini-3-flash-preview tasks <= ${MaxBudget}s.
3. Defer everything else.
4. Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed.
CRITICAL: Exit after writing status file. Do NOT initiate any work requiring >${MaxBudget}s.
"@
        }
        default {
            return @"
$quotaLine
$workerModelLine

HARDLOCK -- this mandate should never be dispatched. If you see this, the heartbeat
governor has failed. Exit immediately without doing any work.
"@
        }
    }
}

# === ORCHESTRATOR DISPATCH ===

function Invoke-DispatchOrchestrator {
    param([string]$Reason)

    # HARDLOCK gate: refuse all dispatch at Tier 6
    if ($Script:CurrentQuotaTier -ge 6) {
        Write-HeartbeatLog "WARN" "HARDLOCK: refusing orchestrator dispatch (reason: $Reason)"
        return $false
    }

    $now = Get-Date
    if (($now - $Script:LastOrchLaunch).TotalSeconds -lt $Script:OrchCooldownSeconds) {
        Write-HeartbeatLog "DEBUG" "Orchestrator cooldown active ($([math]::Round(($now - $Script:LastOrchLaunch).TotalSeconds))s elapsed). Reason: $Reason"
        return $false
    }

    # Select tier-appropriate model
    $orchSelection = Select-OrchestratorModel -Reason $Reason
    if (-not $orchSelection) {
        Write-HeartbeatLog "ERROR" "Orchestrator model selection returned null -- dispatch blocked"
        return $false
    }
    $selectedModel = $orchSelection.Model

    Write-HeartbeatLog "INFO" "Dispatching Orchestrator (reason: $Reason, model: $selectedModel, phase: $($Script:CurrentPhaseName))"
    $Script:LastOrchLaunch = $now
    $Script:OrchCompletedCurrentCycle = $false
    $Script:LastOrchModelDispatched = $selectedModel
    $Script:LastOrchReasonDispatched = $Reason

    # Read live quota data for mandate injection
    $quota = Read-QuotaState
    $mandate = New-OrchestratorMandate -Tier $Script:CurrentQuotaTier -Phase $Script:CurrentPhaseName `
        -FiveHour $quota.FiveHour -SevenDay $quota.SevenDay `
        -Slots $Script:MaxConcurrentSlots -MaxBudget $Script:MaxBudgetOverride `
        -WorkerModels $Script:WorkerModelAllowList

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

    # Write mandate to file and pipe via stdin
    try {
        $mandate | Out-File -LiteralPath $promptFile -Encoding utf8 -ErrorAction Stop
    } catch {
        Write-HeartbeatLog "ERROR" "Failed to write orchestrator prompt file: $($_.Exception.Message)"
        return $false
    }

    # Write PID marker for slot tracking
    $dispatchMarker = Join-Path $orchWorkDir "dispatched_at"
    Get-Date -Format "o" | Out-File -LiteralPath $dispatchMarker -Encoding utf8 -ErrorAction SilentlyContinue

    $job = Start-Job -Name "Orchestrator" -ArgumentList $promptFile, $logFile, $stderrFile, $Script:BaseDir, $selectedModel -ScriptBlock {
        param($PromptFile, $LogFile, $StderrFile, $BaseDir, $Model)
        Set-Location $BaseDir
        Get-Content -Raw -LiteralPath $PromptFile |
            & ollama launch claude --model $Model -- --dangerously-skip-permissions --print `
                >> $LogFile 2>> $StderrFile
    }
    # Record orchestrator job PID for cross-check
    $orchPidFile = Join-Path $orchWorkDir "pid"
    try {
        # Give the job a moment to spawn, then capture child process PID
        Start-Sleep -Milliseconds 500
        $claudeProcs = @(Get-Process -Name "claude" -ErrorAction SilentlyContinue | Sort-Object StartTime -Descending)
        if ($claudeProcs.Count -gt 0) {
            $claudeProcs[0].Id | Out-File -LiteralPath $orchPidFile -Encoding utf8 -ErrorAction SilentlyContinue
        }
    } catch {
        # PID tracking is best-effort
    }

    Write-HeartbeatLog "INFO" "Orchestrator job started (mandate=$(($mandate.Length)) chars, model=$selectedModel, log=$logFile)"
    return $true
}

function Invoke-CleanupOrphanJava {
    # Kill OpenJDK/Java processes that have no parent claude.exe process
    # These are abandoned ollama CLI JVMs from completed/failed agent runs
    $killed = 0
    try {
        $javaProcs = @(Get-Process -Name "java" -ErrorAction SilentlyContinue)
        if ($javaProcs.Count -eq 0) { return 0 }

        $claudePids = @{}
        $claudeProcs = @(Get-Process -Name "claude" -ErrorAction SilentlyContinue)
        foreach ($cp in $claudeProcs) { $claudePids[$cp.Id] = $true }

        foreach ($jp in $javaProcs) {
            # Check if this java process has a claude.exe parent
            $hasClaudeParent = $false
            try {
                $parentId = (Get-CimInstance Win32_Process -Filter "ProcessId = $($jp.Id)" -ErrorAction SilentlyContinue).ParentProcessId
                if ($parentId -and $claudePids.ContainsKey($parentId)) {
                    $hasClaudeParent = $true
                }
            } catch {}

            if (-not $hasClaudeParent) {
                try {
                    Stop-Process -Id $jp.Id -Force -ErrorAction Stop
                    Write-HeartbeatLog "INFO" "Killed orphan java PID=$($jp.Id) (WS=$([math]::Round($jp.WorkingSet64/1MB,1))MB)"
                    $killed++
                } catch {
                    Write-HeartbeatLog "WARN" "Could not kill orphan java PID=$($jp.Id): $($_.Exception.Message)"
                }
            }
        }
    } catch {
        Write-HeartbeatLog "ERROR" "Orphan java cleanup error: $($_.Exception.Message)"
    }

    if ($killed -gt 0) {
        Write-HeartbeatLog "INFO" "Java cleanup: $killed orphan process(es) terminated"
    }
    return $killed
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

    # Refresh quota from ollama.com (with cooldown), then re-evaluate every pulse
    Invoke-QuotaRefresh
    Invoke-RefreshModelAllowlist
    Invoke-QuotaGovernor

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
        $jobOutput = Receive-Job -Job $j 2>$null
        if ($jobOutput) {
            Write-HeartbeatLog "DEBUG" "--- Output from $($j.Name) ---"
            foreach ($line in $jobOutput) {
                if ($line -is [string] -and $line.Trim() -ne "") {
                    Write-HeartbeatLog "DEBUG" "  | $($line.Trim())"
                }
            }
            Write-HeartbeatLog "DEBUG" "--- End $($j.Name) ---"
        }
        Remove-Job -Job $j -Force -ErrorAction SilentlyContinue
    }

    # Phase 1b: Clean up abandoned Java processes from completed jobs
    Invoke-CleanupOrphanJava

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
    $trackedCount   = $slotState.TrackedAgents

    Write-HeartbeatLog "INFO" ("State -> Done: {0} | Pending: {1} | InProg: {2} | Failed/Stale: {3} | StaleInProg: {4} | Slots: jobs={5} claude={6} tracked={7} free={8} orphan={9} phase={10} | Orch: {11}" -f
        $fileState.DoneCount, $fileState.PendingCount, $fileState.InProgressCount,
        $fileState.FailedOrStaleCount, $staleTasks.Count, $workerCount, $claudeCount, $trackedCount, $slotsFree, $orphanLabel, $Script:CurrentPhaseName, $orchLabel)

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
            # At MICRO phase (Tier 5), only dispatch P0/emergency tasks
            if ($Script:CurrentQuotaTier -ge 5) {
                $task = $fileState.PendingTasks | Where-Object { $_.Name -match "p0|P0|BLOCKED_BY_QUOTA|EMERGENCY" } | Select-Object -First 1
                if (-not $task) {
                    Write-HeartbeatLog "DEBUG" "$($Script:CurrentPhaseName): no P0 tasks available to dispatch"
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
    # HARD GATE: orchestrator consumes 1 slot. Never dispatch if all slots are full --
    # that would exceed MaxConcurrentSlots (claude.exe count = workers + orchestrator).
    if (-not $orchRunning) {
        $needsOrch = $false
        $orchReason = ""

        # Slot-aware dispatch paths (workers may be running -- check capacity first)
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
            # All workers idle -- slots are guaranteed available
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
            # Brief yield so the PS job scheduler can pick up the orchestrator job
            Start-Sleep -Milliseconds 500
        }
    }
}

function Invoke-HeartbeatBoot {
    Write-HeartbeatLog "INFO" "============================================"
    Write-HeartbeatLog "INFO" "SCMessenger Swarm Heartbeat v4.0 BOOTING"
    Write-HeartbeatLog "INFO" "Base: $($Script:BaseDir)"
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

    # Phase 3: Refresh quota data and model allowlist, then initialize governor before banner
    Invoke-QuotaRefresh
    Invoke-RefreshModelAllowlist
    Invoke-QuotaGovernor
    Write-HeartbeatLog "INFO" "Max Concurrent Slots: $($Script:MaxConcurrentSlots) | Phase: $($Script:CurrentPhaseName) | Orch Model: $($Script:OrchModelTier) | Allowed Models: $($Script:WorkerModelAllowList.Count)"

    # Phase 4: Remove stale completion flag from prior run
    if (Test-Path $Script:CompleteFlag) {
        Remove-Item $Script:CompleteFlag -Force -ErrorAction SilentlyContinue
        Write-HeartbeatLog "INFO" "Removed previous SWARM_COMPLETE flag"
    }

    # Phase 5: Seed initial file counts
    $initial = Get-SwarmFileState
    $Script:PrevDoneCount = $initial.DoneCount
    Write-HeartbeatLog "INFO" "Initial state -- Done: $($initial.DoneCount) | Pending: $($initial.PendingCount) | InProg: $($initial.InProgressCount) | Failed/Stale: $($initial.FailedOrStaleCount) | Orphan: $($initial.OrphanInProgress)"

    # Phase 6: Clean up orphaned PS jobs from prior session
    Get-Job | Remove-Job -Force -ErrorAction SilentlyContinue
    Write-HeartbeatLog "INFO" "Cleaned up orphaned PowerShell jobs"

    # Phase 7: Cleanup stale worktrees from prior sessions
    Invoke-CleanupStaleWorktrees

    # Phase 8: Config drift detection (informational only -- heartbeat is authority)
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

    # Phase 9: Kill abandoned processes from prior session (IN_PROGRESS tasks but no agents)
    if ($initial.OrphanInProgress) {
        Write-HeartbeatLog "WARN" "Boot-time orphan detected: $($initial.InProgressCount) IN_PROGRESS task(s) with no active agents"
        $null = Invoke-CleanupOrphanProcesses -InProgressCount $initial.InProgressCount
    }

    Write-HeartbeatLog "INFO" "Boot complete. Entering main dispatch loop."
}

# === REGISTER CTRL+C HANDLER ===
$null = [Console]::CancelKeyPress.Add({
    Write-Host "`n[SHUTDOWN] Ctrl+C received. Signaling graceful shutdown (press again to force)..." -ForegroundColor Yellow
    if ($Script:ShutdownRequested) {
        Write-Host "[SHUTDOWN] Second Ctrl+C -- forcing immediate exit." -ForegroundColor Red
        $_.Cancel = $false
        return
    }
    $Script:ShutdownRequested = $true
    $_.Cancel = $true
})

# === MAIN ===
try {
    Invoke-HeartbeatBoot
    while (-not $Script:ShutdownRequested) {
        try {
            Invoke-HeartbeatPulse
        } catch {
            Write-HeartbeatLog "ERROR" "UNHANDLED EXCEPTION in pulse: $($_.Exception.Message)"
            Write-HeartbeatLog "ERROR" "Stack trace: $($_.ScriptStackTrace)"
        }
        # Sleep in small increments so Ctrl+C is responsive
        if ($Script:ShutdownRequested) { break }
        $sleepEnd = (Get-Date).AddSeconds($Script:PollIntervalSeconds)
        while (-not $Script:ShutdownRequested -and (Get-Date) -lt $sleepEnd) {
            Start-Sleep -Milliseconds 250
        }
    }
} finally {
    Write-Host "`n=== SWARM SHUTDOWN ===" -ForegroundColor Yellow
    Write-Host "Stopping all PS jobs..." -ForegroundColor Cyan
    Get-Job | ForEach-Object {
        try { Stop-Job -Job $_ -ErrorAction SilentlyContinue } catch {}
        try { Remove-Job -Job $_ -Force -ErrorAction SilentlyContinue } catch {}
    }
    Write-Host "Killing remaining claude.exe processes..." -ForegroundColor Cyan
    $claudeProcs = @(Get-Process -Name "claude" -ErrorAction SilentlyContinue)
    foreach ($p in $claudeProcs) {
        try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
    }
    Write-Host "Swarm shutdown complete." -ForegroundColor Green
}
