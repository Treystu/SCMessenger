# === CONFIGURATION ===
$Script:BaseDir              = $PSScriptRoot
$Script:DoneDir              = Join-Path $BaseDir "HANDOFF\done"
$Script:TodoDir              = Join-Path $BaseDir "HANDOFF\todo"
$Script:InProgDir            = Join-Path $BaseDir "HANDOFF\IN_PROGRESS"
$Script:CompleteFlag         = Join-Path $BaseDir "HANDOFF\SWARM_COMPLETE"
$Script:MaxConcurrentSlots    = 3
$Script:PollIntervalSeconds   = 10
$Script:OrchModelTriage       = "kimi-k2.6:cloud"
$Script:OrchModelStandard     = "kimi-k2.6:cloud"
$Script:OrchModelHeavy        = "kimi-k2.6:cloud"
$Script:OrchFallbackModel     = "deepseek-v4-pro:cloud"
$Script:OrchTertiaryModel     = "glm-5.1:cloud"
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
$Script:OrchLastFailedModel          = ""
$Script:ShutdownRequested            = $false
$Script:CachedClaudeCount            = $null
$Script:CachedClaudeTimestamp        = [datetime]::MinValue
$Script:LastModelAllowlistRefresh    = [datetime]::MinValue
$Script:DynamicModelAllowList        = @()
$Script:BootTime                     = $null
$Script:JavaOrphanTracker            = @()
$Script:LastWorkerDispatchTime      = [datetime]::MinValue

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

function Get-CanonicalTaskName {
    param([string]$FileName)
    $name = $FileName
    while ($name.StartsWith("IN_PROGRESS_")) { $name = $name.Substring("IN_PROGRESS_".Length) }
    return $name
}

function Get-InProgressName {
    param([string]$FileName)
    $canonical = Get-CanonicalTaskName -FileName $FileName
    return "IN_PROGRESS_$canonical"
}

function Acquire-TaskLock {
    param([string]$TaskFilePath, [int]$TimeoutSeconds = 10)
    $lockFile = "$TaskFilePath.lock"
    $startTime = Get-Date
    while ((Get-Date) -lt $startTime.AddSeconds($TimeoutSeconds)) {
        if (-not (Test-Path -LiteralPath $lockFile)) {
            @{ PID = $PID; LockedAt = (Get-Date -Format "o"); Process = "SwarmHeartbeat" } |
                ConvertTo-Json | Out-File -LiteralPath $lockFile -Encoding utf8
            return $true
        }
        # Check if lock holder is dead
        try {
            $lockData = Get-Content -LiteralPath $lockFile -Raw | ConvertFrom-Json
            $holderPid = [int]$lockData.PID
            $holder = Get-Process -Id $holderPid -ErrorAction SilentlyContinue
            if (-not $holder) {
                Remove-Item -LiteralPath $lockFile -Force -ErrorAction SilentlyContinue
                continue
            }
        } catch {
            Remove-Item -LiteralPath $lockFile -Force -ErrorAction SilentlyContinue
            continue
        }
        Start-Sleep -Milliseconds 250
    }
    return $false
}

function Release-TaskLock {
    param([string]$TaskFilePath)
    $lockFile = "$TaskFilePath.lock"
    Remove-Item -LiteralPath $lockFile -Force -ErrorAction SilentlyContinue
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
        $procs = @(Get-CimInstance Win32_Process -Filter "Name = 'claude.exe'" -ErrorAction SilentlyContinue)
        return $procs.Count
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
        Write-HeartbeatLog "WARN" "Get-CimInstance for claude.exe timed out (5s) -- using cached or default value"
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
        $agentPid = $null

        # Prefer claude_pid from job registry (most accurate)
        $regFile = Join-Path $dir.FullName "job_registry.json"
        if (Test-Path $regFile) {
            try {
                $reg = Get-Content -LiteralPath $regFile -Raw | ConvertFrom-Json
                if ($reg.claude_pid -and $reg.claude_pid -ne 0) {
                    $agentPid = [int]$reg.claude_pid
                }
            } catch {}
        }

        # Fall back to legacy pid file
        if (-not $agentPid) {
            $pidFile = Join-Path $dir.FullName "pid"
            if (Test-Path $pidFile) {
                try {
                    $agentPid = [int](Get-Content -LiteralPath $pidFile -Raw -ErrorAction Stop).Trim()
                } catch {
                    # Stale PID file -- ignore
                }
            }
        }

        if ($agentPid) {
            $proc = Get-Process -Id $agentPid -ErrorAction SilentlyContinue
            if ($proc) { $tracked++ }
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

    # claude.exe processes occupy slots. The host/terminal session is also a
    # claude.exe and counts toward the slot total -- it is never killed.
    # TotalUsed = max(worker jobs, claude procs) + orchestrator slot.
    # If more claude.exe exist than tracked agents + orchestrator, the extras
    # are pre-existing sessions (like the host) that still occupy slots.
    $totalUsed = [math]::Max($workerCount, $claudeCount) + $orchSlots
    $slotsFree = $Script:MaxConcurrentSlots - $totalUsed

    if ($claudeCount -gt ($trackedCount + $orchSlots)) {
        Write-HeartbeatLog "DEBUG" "Slot accounting: $claudeCount claude.exe processes ($trackedCount tracked + $orchSlots orch). Pre-existing sessions occupy $($claudeCount - $trackedCount - $orchSlots) extra slot(s). TotalUsed=$totalUsed SlotsFree=$slotsFree"
    } elseif ($claudeCount -lt ($workerCount + $orchSlots)) {
        Write-HeartbeatLog "DEBUG" "$workerCount worker job(s) pending process spawn (claude=$claudeCount tracked=$trackedCount orch=$orchSlots)"
    }

    return @{
        WorkerJobs    = $workerCount
        ClaudeProcs   = $claudeCount
        TrackedAgents = $trackedCount
        OrchSlots     = $orchSlots
        TotalUsed     = $totalUsed
        SlotsFree     = $slotsFree
    }
}

function Wait-ClaudePidFromJob {
    param(
        [System.Management.Automation.Job]$Job,
        [string]$AgentDir,
        [int]$MaxRetries = 10,
        [int]$InitialDelayMs = 500
    )
    $claudePid = $null
    $dispatchTime = Get-Date
    try {
        $jobProcessId = $Job.ChildJobs[0].JobProcessId
        $retryDelay = $InitialDelayMs
        for ($i = 0; $i -lt $MaxRetries; $i++) {
            Start-Sleep -Milliseconds $retryDelay
            # Walk process tree from job process to find claude.exe descendant
            $searchPids = @($jobProcessId)
            $depth = 0
            while ($depth -lt 5 -and $searchPids.Count -gt 0) {
                $nextPids = @()
                foreach ($spid in $searchPids) {
                    $children = @(Get-CimInstance Win32_Process -Filter "ParentProcessId = $spid" -ErrorAction SilentlyContinue)
                    foreach ($child in $children) {
                        if ($child.Name -eq "claude.exe") {
                            $claudePid = $child.ProcessId
                            break
                        }
                        $nextPids += $child.ProcessId
                    }
                    if ($null -ne $claudePid) { break }
                }
                if ($null -ne $claudePid) { break }
                $searchPids = $nextPids
                $depth++
            }
            if ($null -ne $claudePid) { break }
            # Exponential backoff: 500ms -> 750ms -> 1125ms -> ..., capped at 3000ms
            $retryDelay = [math]::Min([math]::Floor($retryDelay * 1.5), 3000)
        }
        # Fallback: newest untracked claude.exe that started AFTER this dispatch
        if ($null -eq $claudePid) {
            $trackedPids = @()
            $agentDirs = Get-ChildItem -LiteralPath (Join-Path $Script:BaseDir ".claude\agents") -Directory -ErrorAction SilentlyContinue
            foreach ($dir in $agentDirs) {
                $pf = Join-Path $dir.FullName "pid"
                if (Test-Path $pf) {
                    try { $trackedPids += [int](Get-Content -LiteralPath $pf -Raw).Trim() } catch {}
                }
                $regFile = Join-Path $dir.FullName "job_registry.json"
                if (Test-Path $regFile) {
                    try {
                        $reg = Get-Content -LiteralPath $regFile -Raw | ConvertFrom-Json
                        if ($reg.claude_pid -and $reg.claude_pid -ne 0) {
                            $trackedPids += [int]$reg.claude_pid
                        }
                    } catch {}
                }
            }
            $untracked = @(Get-CimInstance Win32_Process -Filter "Name = 'claude.exe'" -ErrorAction SilentlyContinue | Where-Object { $trackedPids -notcontains $_.ProcessId } | Sort-Object CreationDate -Descending)
            foreach ($candidate in $untracked) {
                if ($candidate.CreationDate -gt $dispatchTime.AddSeconds(-5)) {
                    $claudePid = $candidate.ProcessId
                    break
                }
            }
        }
        if ($null -ne $claudePid) {
            $pidFile = Join-Path $AgentDir "pid"
            $claudePid | Out-File -LiteralPath $pidFile -Encoding utf8 -ErrorAction SilentlyContinue
            $attemptStr = if ($i -lt $MaxRetries) { "$($i+1)/$MaxRetries" } else { "fallback" }
            Write-HeartbeatLog "INFO" "Captured claude.exe PID: $claudePid ($attemptStr)"
        } else {
            Write-HeartbeatLog "WARN" "Failed to capture claude.exe PID after $MaxRetries attempts (no new claude.exe found)"
        }
    } catch {
        Write-HeartbeatLog "DEBUG" "PID capture via Wait-ClaudePidFromJob failed: $($_.Exception.Message)"
    }
    return $claudePid
}

function Get-SwarmFileState {
    $doneFiles       = @(Get-ChildItem -LiteralPath $Script:DoneDir -Filter "*.md" -ErrorAction SilentlyContinue)
    $pendingFiles    = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[VALIDATED]_*.md" -ErrorAction SilentlyContinue)
    $inProgFiles     = @(Get-ChildItem -LiteralPath $Script:InProgDir -Filter "*.md" -ErrorAction SilentlyContinue)
    $failedFiles     = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[FAILED]_*.md" -ErrorAction SilentlyContinue)
    $staleFiles      = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[STALE]_*.md" -ErrorAction SilentlyContinue)
    $timeBreachFiles = @(Get-ChildItem -LiteralPath $Script:TodoDir -Filter "[TIME_BREACH]_*.md" -ErrorAction SilentlyContinue)

    # Compute orphan status without Get-ActiveSlotCount (which has leak-detection logging)
    # Orphan: IN_PROGRESS files exist but no active slots of any kind
    # Grace period: skip orphan detection for the first 5 minutes after boot
    # to avoid reclaiming tasks whose agents are still starting up after restart
    $gracePeriodActive = ($null -ne $Script:BootTime) -and (((Get-Date) - $Script:BootTime).TotalMinutes -lt 5)
    $orphanInProg = (-not $gracePeriodActive) -and ($inProgFiles.Count -gt 0) -and ((Get-ClaudeProcessCount) -eq 0) -and ((Get-ActiveWorkerCount) -eq 0) -and (-not (Get-OrchestratorRunning))

    $state = @{
        DoneCount          = $doneFiles.Count
        PendingCount       = $pendingFiles.Count
        InProgressCount    = $inProgFiles.Count
        FailedOrStaleCount = $failedFiles.Count + $staleFiles.Count + $timeBreachFiles.Count
        OrphanInProgress   = $orphanInProg
        PendingTasks       = ($pendingFiles + $timeBreachFiles) | Sort-Object Name
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
            $agentName = Get-CanonicalTaskName -FileName ([System.IO.Path]::GetFileNameWithoutExtension($f.Name))
            $dispatchMarker = Join-Path $Script:BaseDir ".claude\agents\$agentName\dispatched_at"
            $refTime = $f.LastWriteTime
            if (Test-Path $dispatchMarker) {
                try {
                    $dispatchTime = [datetime]::Parse((Get-Content -LiteralPath $dispatchMarker -Raw -ErrorAction Stop).Trim())
                    # Skip tasks dispatched recently (within 30 minutes) -- they may still be starting
                    $minutesSinceDispatch = ((Get-Date) - $dispatchTime).TotalMinutes
                    if ($minutesSinceDispatch -lt 30) {
                        Write-HeartbeatLog "DEBUG" "Skipping stale check for $agentName -- dispatched $([math]::Round($minutesSinceDispatch,1))min ago (grace period)"
                        continue
                    }
                    $refTime = $dispatchTime
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

    # Lazy-refresh-on-read: if quota_state.json exists but the timestamp is
    # older than ScraperStaleMinutes, trigger a forced re-scrape before reading.
    # This ensures any consumer of quota data (even outside the main pulse loop)
    # automatically gets fresh state. Within the pulse loop, Invoke-QuotaRefresh
    # already handles the primary refresh cycle with its own cooldown, so the
    # lazy-refresh here is a safety net.
    $jsonFile = Join-Path $Script:BaseDir ".claude\quota_state.json"
    if (Test-Path $jsonFile) {
        try {
            $checkJson = Get-Content -Raw -LiteralPath $jsonFile -ErrorAction Stop | ConvertFrom-Json
            if ($checkJson.timestamp) {
                $lastUpdate = [datetime]::Parse($checkJson.timestamp)
                $ageMinutes = ((Get-Date) - $lastUpdate).TotalMinutes
                if ($ageMinutes -ge $Script:ScraperStaleMinutes) {
                    Write-HeartbeatLog "DEBUG" "Read-QuotaState: data is stale ($([math]::Round($ageMinutes,1))min >= $($Script:ScraperStaleMinutes)min) -- triggering lazy refresh"
                    Invoke-QuotaRefresh
                }
            }
        } catch {}
    } else {
        Write-HeartbeatLog "DEBUG" "Read-QuotaState: no quota_state.json exists -- triggering initial fetch"
        Invoke-QuotaRefresh
    }

    # Read structured JSON (single source of truth, written by OllamaQuotaScraper.ps1)
    # Re-read in case lazy-refresh above just updated the file
    if (Test-Path $jsonFile) {
        try {
            $json = Get-Content -Raw -LiteralPath $jsonFile -ErrorAction Stop | ConvertFrom-Json
            if ($json.status -eq "ok") {
                $fiveHour  = [double]$json.fiveHour
                $sevenDay  = [double]$json.sevenDay
                $resetMins = $json.resetMinutes
            } else {
                Write-HeartbeatLog "WARN" "quota_state.json status=$($json.status): $($json.error)"
            }
        } catch {
            Write-HeartbeatLog "ERROR" "quota_state.json parse failed: $($_.Exception.Message)"
        }
    } else {
        # Conservative default: assume quota is tight when data is unavailable.
        # Defaulting to Tier 1 (HEAVY-LIFT) risks burning through remaining quota.
        Write-HeartbeatLog "WARN" "No quota_state.json found; defaulting to Tier 5 (MICRO) for safety"
        $fiveHour = 95.0
        $sevenDay = 95.0
        $tier = 5
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

    # 7-day budget override: bump tier up if 7-day is tight, even when
    # 5-hour looks healthy. This prevents burning through the weekly window
    # during brief 5-hour dips.
    if ($sevenDay -gt 99.5) {
        $tier = 6  # HARDLOCK regardless of 5-hour
    } elseif ($sevenDay -gt 95 -and $tier -lt 5) {
        $tier = 5  # MICRO: 7-day critical, conserve aggressively
    } elseif ($sevenDay -gt 90 -and $tier -lt 4) {
        $tier = 4  # LIGHT: 7-day very high, limit to docs/tests
    } elseif ($sevenDay -gt 75 -and $tier -lt 3) {
        $tier = 3  # MIXED: 7-day elevated, avoid heavy tasks
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
            $Script:OrchModelTier         = "kimi-k2.6:cloud"
            Write-HeartbeatLog "WARN" "MICRO PHASE: Single-slot, max budget 300s, P0/defer-only. Orchestrator: kimi-k2.6"
        }
        4 {
            $Script:MaxConcurrentSlots    = 2
            $Script:OrchCooldownSeconds   = 300
            $Script:MaxBudgetOverride     = 900
            $Script:CurrentPhaseName      = "LIGHT"
            $Script:OrchModelTier         = "kimi-k2.6:cloud"
            Write-HeartbeatLog "WARN" "LIGHT PHASE: 2-slot, docs/tests/lint/bindings only, max budget 900s. Orchestrator: kimi-k2.6"
        }
        3 {
            $Script:MaxConcurrentSlots    = 2
            $Script:OrchCooldownSeconds   = 180
            $Script:MaxBudgetOverride     = 1800
            $Script:CurrentPhaseName      = "MIXED"
            $Script:OrchModelTier         = "kimi-k2.6:cloud"
            Write-HeartbeatLog "INFO" "MIXED PHASE: 2-slot, smaller features/validation/testing, max budget 1800s. Orchestrator: kimi-k2.6"
        }
        2 {
            $Script:MaxConcurrentSlots    = 3
            $Script:OrchCooldownSeconds   = 120
            $Script:MaxBudgetOverride     = 5400
            $Script:CurrentPhaseName      = "EXECUTE"
            $Script:OrchModelTier         = "kimi-k2.6:cloud"
            Write-HeartbeatLog "INFO" "EXECUTE PHASE: 3-slot, major feature implementation, max budget 5400s. Orchestrator: kimi-k2.6"
        }
        default {
            $Script:MaxConcurrentSlots    = 3
            $Script:OrchCooldownSeconds   = 120
            $Script:MaxBudgetOverride     = 0
            $Script:CurrentPhaseName      = "HEAVY-LIFT"
            $Script:OrchModelTier         = "kimi-k2.6:cloud"
            Write-HeartbeatLog "INFO" "HEAVY-LIFT PHASE: 3-slot, flagship models, no budget cap. Token-heavy work: multi-file wiring, architecture, deep planning. Orchestrator: kimi-k2.6"
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
        # Bootstrap: no model allowlist loaded yet. Demote to Tier 3 until
        # we have model availability data, to avoid burning quota on heavy models.
        if ($quota.Tier -le 2) {
            Write-HeartbeatLog "WARN" "No model allowlist available; demoting from Tier $($quota.Tier) to Tier 3 (MIXED) for safety"
            $Script:CurrentQuotaTier = 3
            $Script:MaxConcurrentSlots = 2
            $Script:OrchCooldownSeconds = 180
            $Script:MaxBudgetOverride = 1800
            $Script:CurrentPhaseName = "MIXED"
            $Script:OrchModelTier = "deepseek-v4-pro:cloud"
        }
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

    # Fallback chain: primary -> fallback -> tertiary
    $modelChain = @($model)
    if ($Script:OrchFallbackModel) { $modelChain += $Script:OrchFallbackModel }
    if ($Script:OrchTertiaryModel) { $modelChain += $Script:OrchTertiaryModel }
    $resolved = $modelChain | Where-Object { $_ -ne $Script:OrchLastFailedModel } | Select-Object -First 1
    if ($resolved -and $resolved -ne $model) {
        Write-HeartbeatLog "WARN" "Orchestrator model $model failed last cycle -- switching to $resolved"
        $model = $resolved
    }
    $Script:OrchLastFailedModel = ""  # Clear after one fallback attempt

    # At MICRO phase (Tier 5), force triage model regardless
    if ($Script:CurrentQuotaTier -ge 5) {
        Write-HeartbeatLog "INFO" "Orchestrator model: $model (quota phase: $($Script:CurrentPhaseName))"
        return @{ Model = $model; MandateType = "micro" }
    }

    # At LIGHT/MIXED (Tier 3-4), use tier-assigned model
    if ($Script:CurrentQuotaTier -ge 3) {
        return @{ Model = $model; MandateType = "standard" }
    }

    # EXECUTE/HEAVY-LIFT (Tier 1-2): always use the tier-assigned model.
    # The orchestrator needs strong reasoning even for triage/retriage --
    # it must decompose tasks accurately. Small models produce poor task breakdowns.
    # Mandate type drives prompt content, not model selection.
    if ($Reason -match "\bmalformed\b|\bNEEDS_TRIAGE\b|\bretriage\b") {
        return @{ Model = $model; MandateType = "triage" }
    }
    elseif ($Reason -match "\bbacklog\b|\bdrained\b|\bremaining\b") {
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
        $Script:LastWorkerDispatchTime = Get-Date

        # Canonical naming: use IN_PROGRESS_ prefix for files in the IN_PROGRESS directory
        $inProgressName = Get-InProgressName -FileName $TaskFile.Name
        $canonicalName = Get-CanonicalTaskName -FileName ([System.IO.Path]::GetFileNameWithoutExtension($TaskFile.Name))

        # Create agent directory and write dispatched_at BEFORE moving file
        # This eliminates the window where the file is in IN_PROGRESS/ but has no timestamp
        $agentDir = Join-Path $Script:BaseDir ".claude\agents\$canonicalName"
        $null = New-Item -ItemType Directory -Path $agentDir -Force
        $dispatchMarker = Join-Path $agentDir "dispatched_at"
        Get-Date -Format "o" | Out-File -LiteralPath $dispatchMarker -Encoding utf8 -ErrorAction SilentlyContinue

        $inProgressPath = Join-Path $Script:InProgDir $inProgressName
        $lockAcquired = Acquire-TaskLock -TaskFilePath $TaskFile.FullName -TimeoutSeconds 10
        if (-not $lockAcquired) {
            Write-HeartbeatLog "WARN" "Could not acquire lock for $($TaskFile.Name) -- skipping dispatch"
            return $false
        }
        try {
            Move-Item -LiteralPath $TaskFile.FullName -Destination $inProgressPath -Force -ErrorAction Stop
        } finally {
            Release-TaskLock -TaskFilePath $TaskFile.FullName
        }

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

        # Capture PID via retry-based tree-walk from PS job process
        $claudePid = Wait-ClaudePidFromJob -Job $job -AgentDir $agentDir

        # Write job registry for persistent cross-session tracking
        $registryFile = Join-Path $agentDir "job_registry.json"
        try {
            $registry = @{
                source        = "SwarmHeartbeat"
                job_name      = $jobName
                task_file     = $inProgressName
                canonical_name = $canonicalName
                model         = $model
                budget        = $budget
                dispatched_at = (Get-Date -Format "o")
                ps_job_id     = $job.Id.ToString()
                heartbeat_pid = $PID
                claude_pid    = $claudePid
            }
            $registry | ConvertTo-Json | Out-File -LiteralPath $registryFile -Encoding utf8 -ErrorAction Stop
        } catch {
            Write-HeartbeatLog "DEBUG" "Failed to write job registry: $($_.Exception.Message)"
        }

        Write-HeartbeatLog "INFO" "Worker dispatched: job=$jobName file=$inProgressName pid=$claudePid"
        return $true
    } catch {
        Write-HeartbeatLog "ERROR" "Worker dispatch failed for $($TaskFile.Name): $($_.Exception.Message)"
        return $false
    }
}

# === MICRO TASK BATCHING ===

function Invoke-CreateMicroBatch {
    param([System.IO.FileInfo[]]$PendingTasks)
    $microTasks = @($PendingTasks | Where-Object { $_.Name -match "MICRO_" -and $_.Name -notmatch "BATCH_MICRO_" })
    if ($microTasks.Count -lt 2) { return $null }

    $batchName = "BATCH_MICRO_$(Get-Date -Format 'HHmmss').md"
    $batchPath = Join-Path $Script:TodoDir $batchName

    $header = @"
# MODEL: gemini-3-flash-preview:cloud
# BUDGET: $([math]::Min(180, $Script:MaxBudgetOverride))
# BATCHED_TASKS: $($microTasks.Count)
# STRIPPED_CONTEXT: true

# BATCH_INSTRUCTIONS:
# You are processing $($microTasks.Count) MICRO tasks sequentially.
# For each TASK section below:
#   1. Apply the code change described
#   2. Move the original task file from todo/ to done/ (file name shown in each section)
#   3. Proceed to the next TASK section
# Do NOT run `./gradlew :app:assembleDebug` for individual MICRO changes.
# Only run the build ONCE after all tasks are done, if you have time.
# If you run out of budget, stop cleanly -- remaining tasks stay in todo/.
# STRIPPED CONTEXT: You do NOT need to read CLAUDE.md in full.
# Relevant rules only: android.md (minSdk 26, compileSdk 35, Hilt, Compose).

"@

    $body = ""
    foreach ($mt in $microTasks) {
        $body += "---`n## TASK: $($mt.Name)`n"
        try {
            $taskContent = Get-Content -LiteralPath $mt.FullName -Raw -ErrorAction SilentlyContinue
            $body += $taskContent + "`n`n"
        } catch {
            $body += "[ERROR reading task content]`n`n"
        }
    }

    try {
        ($header + $body) | Out-File -LiteralPath $batchPath -Encoding utf8 -ErrorAction Stop
        Write-HeartbeatLog "INFO" "Created MICRO batch: $batchName ($($microTasks.Count) tasks)"
        return Get-Item -LiteralPath $batchPath
    } catch {
        Write-HeartbeatLog "ERROR" "Failed to create MICRO batch: $($_.Exception.Message)"
        return $null
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

DYNAMIC SCOPING & BUDGETING RULES (apply to ALL tiers):
- Budget = (estimated tokens / 10) + (build verification time in seconds) + 30s buffer.
- A task that edits 1 file with 8 LOC change should NEVER have a 120s budget.
- Token budget field MUST be realistic: count prompt characters / 4 for estimate.
- MICRO tasks (null-guard, safe-return, deprecation wrap) get 60s budget MAX.
- If a task requires a full Gradle build, add 120s to budget OR delegate build to orchestrator.

BATCHING RULE (MICRO tasks):
- If 2+ [VALIDATED]_MICRO_* tasks exist in todo/, create BATCH_MICRO_*.md containing ALL of them.
- The batch worker processes them sequentially, moving each to done/ as it completes.
- This amortizes model load + context injection cost across multiple tasks.

BUILD VERIFICATION PROTOCOL:
- Workers executing pure null-guard / safe-return / deprecation-wrap changes SKIP the Gradle build.
- The orchestrator runs FULL build verification (`./gradlew :app:assembleDebug`) after workers finish.
- If the orchestrator build fails, it creates a remediation task for the specific failure.
- This prevents every micro worker from burning 60-120s on Gradle while the change is trivial.

PROMPT STRIPPING (MICRO tasks):
- MICRO tasks do NOT need full CLAUDE.md architecture context.
- Strip to: android.md rules only + the exact task instructions + file context.
- Add header `# STRIPPED_CONTEXT: true` so TaskGovernor knows to omit quota block.

TIME_BREACH PROTOCOL:
- [TIME_BREACH]_ tasks were force-killed by TaskGovernor for exceeding budget.
- Rename [TIME_BREACH]_ prefix to [VALIDATED]_.
- Increase # BUDGET: by +60s minimum (e.g., 120s -> 180s, 180s -> 240s).
- Verify # MODEL: header exists and is appropriate.
- Redispatch as normal. Do NOT downgrade model -- the issue was time, not capability.

STANDARD ORCHESTRATOR DUTIES:
1. Read HANDOFF/done/ for newly completed tasks. Update REMAINING_WORK_TRACKING.md.
2. Read HANDOFF/IN_PROGRESS/ for stale tasks (>60 min). Reclaim to todo/ with [STALE]_ prefix.
3. Read HANDOFF/todo/ for [FAILED]_ tasks. Downgrade model, reduce budget 40%, re-validate.
4. Read HANDOFF/todo/ for [STALE]_ tasks. Re-validate, restore [VALIDATED]_ prefix.
5. Read HANDOFF/todo/ for [TIME_BREACH]_ tasks. Apply TIME_BREACH PROTOCOL above.
6. Read HANDOFF/todo/ for [NEEDS_TRIAGE]_ tasks. Add # MODEL: and # BUDGET: headers.
7. Validate unvalidated tasks. Add [VALIDATED]_ prefix or reject.
8. Create new task files from REMAINING_WORK_TRACKING.md gaps with proper headers.
9. Assign models per routing table. Set budgets based on complexity.
10. Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed.

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

DYNAMIC SCOPING & BUDGETING RULES:
- Budget = (estimated tokens / 10) + (build verification time) + 30s buffer.
- MICRO tasks (null-guard, safe-return, deprecation wrap) get 60s budget MAX.
- If 2+ MICRO tasks exist, batch them into BATCH_MICRO_*.md and dispatch ONE worker.
- Workers skip Gradle for pure null-guard/safe-return changes; orchestrator runs full build after.
- MICRO tasks: strip CLAUDE.md context to android.md rules only + task instructions.

TIME_BREACH PROTOCOL:
- [TIME_BREACH]_ tasks were force-killed by TaskGovernor for exceeding budget.
- Rename [TIME_BREACH]_ prefix to [VALIDATED]_.
- Increase # BUDGET: by +60s minimum.
- Redispatch as normal. Do NOT downgrade model -- the issue was time, not capability.

STANDARD ORCHESTRATOR DUTIES:
1. Read HANDOFF/done/ for newly completed tasks. Update REMAINING_WORK_TRACKING.md.
2. Read HANDOFF/IN_PROGRESS/ for stale tasks (>60 min). Reclaim to todo/ with [STALE]_ prefix.
3. Read HANDOFF/todo/ for [FAILED]_ tasks. Downgrade model, reduce budget 40%, re-validate.
4. Read HANDOFF/todo/ for [STALE]_ tasks. Re-validate, restore [VALIDATED]_ prefix.
5. Read HANDOFF/todo/ for [TIME_BREACH]_ tasks. Apply TIME_BREACH PROTOCOL above.
6. Read HANDOFF/todo/ for [NEEDS_TRIAGE]_ tasks. Add # MODEL: and # BUDGET: headers.
7. Validate unvalidated tasks. Add [VALIDATED]_ prefix or reject.
8. Create new task files from REMAINING_WORK_TRACKING.md gaps with proper headers.
9. Assign models per routing table. Set budgets based on complexity.
10. Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed.

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

DYNAMIC SCOPING & BUDGETING RULES:
- Budget = (estimated tokens / 10) + (build verification time) + 30s buffer.
- MICRO tasks get 60s budget MAX. If 2+ MICRO tasks exist, batch them into BATCH_MICRO_*.md.
- Workers skip Gradle for pure null-guard/safe-return changes; orchestrator runs full build after.
- MICRO tasks: strip CLAUDE.md context to android.md rules only + task instructions.

TIME_BREACH PROTOCOL:
- [TIME_BREACH]_ tasks were force-killed by TaskGovernor for exceeding budget.
- Rename [TIME_BREACH]_ prefix to [VALIDATED]_.
- Increase # BUDGET: by +60s minimum.
- Redispatch as normal. Do NOT downgrade model -- the issue was time, not capability.

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

DYNAMIC SCOPING & BUDGETING RULES:
- Budget = (estimated tokens / 10) + (build verification time) + 30s buffer.
- MICRO tasks get 60s budget MAX. If 2+ MICRO tasks exist, batch them into BATCH_MICRO_*.md.
- Workers skip Gradle for pure null-guard/safe-return changes; orchestrator runs full build after.
- MICRO tasks: strip CLAUDE.md context to android.md rules only + task instructions.

TIME_BREACH PROTOCOL:
- [TIME_BREACH]_ tasks were force-killed by TaskGovernor for exceeding budget.
- Rename [TIME_BREACH]_ prefix to [VALIDATED]_.
- Increase # BUDGET: by +60s minimum (respecting max budget cap).
- Redispatch as normal. Do NOT downgrade model.

ORCHESTRATOR DUTIES (abbreviated):
1. Scan done/, IN_PROGRESS/, todo/.
2. Triage [FAILED]_ and [STALE]_ tasks -- downgrade aggressively.
3. Handle [TIME_BREACH]_ tasks per TIME_BREACH PROTOCOL above.
4. Create ONLY small-model tasks (docs/tests/lint/bindings/P0).
5. DEFER everything else.
6. Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed.
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

DYNAMIC SCOPING & BUDGETING RULES:
- Budget = (estimated tokens / 10) + (build verification time) + 30s buffer.
- MICRO tasks get 60s budget MAX. If 2+ MICRO tasks exist, batch them into BATCH_MICRO_*.md.
- Workers skip Gradle for pure null-guard/safe-return changes; orchestrator runs full build after.
- MICRO tasks: strip CLAUDE.md context to android.md rules only + task instructions.

TIME_BREACH PROTOCOL:
- [TIME_BREACH]_ tasks were force-killed by TaskGovernor for exceeding budget.
- Rename [TIME_BREACH]_ prefix to [VALIDATED]_.
- Increase # BUDGET: by +60s (up to max budget cap).
- Redispatch as normal. Do NOT downgrade model.

ORCHESTRATOR DUTIES (micro only):
1. Quick scan of done/ and todo/.
2. Handle [TIME_BREACH]_ tasks per TIME_BREACH PROTOCOL above.
3. Create ONLY gemini-3-flash-preview tasks <= ${MaxBudget}s.
4. If 2+ MICRO tasks exist, batch them. ONE worker processes all sequentially.
5. Defer everything else.
6. Write HANDOFF/ORCHESTRATOR_STATUS.md with STATUS=completed.
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
    # Capture orchestrator PID via retry-based tree-walk from PS job process
    $orchClaudePid = Wait-ClaudePidFromJob -Job $job -AgentDir $orchWorkDir
    $orchPidFile = Join-Path $orchWorkDir "pid"

    # Write job registry for orchestrator
    $orchRegistryFile = Join-Path $orchWorkDir "job_registry.json"
    try {
        $orchRegistry = @{
            source        = "SwarmHeartbeat"
            job_name      = "Orchestrator"
            task_file     = "ORCHESTRATOR_MANDATE"
            canonical_name = "orchestrator"
            model         = $selectedModel
            budget        = 0
            dispatched_at = (Get-Date -Format "o")
            ps_job_id     = $job.Id.ToString()
            heartbeat_pid = $PID
            claude_pid    = $orchClaudePid
            reason        = $Reason
        }
        $orchRegistry | ConvertTo-Json | Out-File -LiteralPath $orchRegistryFile -Encoding utf8 -ErrorAction Stop
    } catch {
        Write-HeartbeatLog "DEBUG" "Failed to write orchestrator job registry: $($_.Exception.Message)"
    }

    Write-HeartbeatLog "INFO" "Orchestrator job started (mandate=$(($mandate.Length)) chars, model=$selectedModel, pid=$orchClaudePid, log=$logFile)"
    return $true
}

function Invoke-CleanupOrphanJava {
    param([int]$MinAgeSeconds = 60, [int]$ConfirmationSeconds = 120)

    # Skip cleanup if a worker was dispatched recently -- java processes may still be starting up
    $sinceLastDispatch = ((Get-Date) - $Script:LastWorkerDispatchTime).TotalSeconds
    if ($sinceLastDispatch -lt $MinAgeSeconds) {
        Write-HeartbeatLog "DEBUG" "Skipping orphan java cleanup -- worker dispatched ${sinceLastDispatch}s ago (grace=${MinAgeSeconds}s)"
        return 0
    }

    $killed = 0
    try {
        $javaProcs = @(Get-Process -Name "java" -ErrorAction SilentlyContinue)
        if ($javaProcs.Count -eq 0) { return 0 }

        $claudePids = @{}
        $claudeProcs = @(Get-CimInstance Win32_Process -Filter "Name = 'claude.exe'" -ErrorAction SilentlyContinue)
        foreach ($cp in $claudeProcs) { $claudePids[$cp.ProcessId] = $true }

        foreach ($jp in $javaProcs) {
            # Skip young processes -- they may still be starting up
            $procAge = ((Get-Date) - $jp.StartTime).TotalSeconds
            if ($procAge -lt $MinAgeSeconds) {
                Write-HeartbeatLog "DEBUG" "Skipping young java PID=$($jp.Id) (age=${procAge}s < ${MinAgeSeconds}s)"
                continue
            }

            # Check if this java process has a claude.exe parent
            $hasClaudeParent = $false
            try {
                $parentId = (Get-CimInstance Win32_Process -Filter "ProcessId = $($jp.Id)" -ErrorAction SilentlyContinue).ParentProcessId
                if ($parentId -and $claudePids.ContainsKey($parentId)) {
                    $hasClaudeParent = $true
                }
            } catch {}

            if (-not $hasClaudeParent) {
                $now = Get-Date
                $trackerEntry = $Script:JavaOrphanTracker | Where-Object { $_.Pid -eq $jp.Id } | Select-Object -First 1
                if (-not $trackerEntry) {
                    $Script:JavaOrphanTracker += [PSCustomObject]@{ Pid = $jp.Id; FirstSeen = $now }
                    Write-HeartbeatLog "DEBUG" "Orphan java PID=$($jp.Id) first seen, waiting ${ConfirmationSeconds}s before killing"
                    continue
                }

                $orphanAge = ($now - $trackerEntry.FirstSeen).TotalSeconds
                if ($orphanAge -lt $ConfirmationSeconds) {
                    Write-HeartbeatLog "DEBUG" "Orphan java PID=$($jp.Id) age=${orphanAge}s < ${ConfirmationSeconds}s, waiting"
                    continue
                }

                try {
                    Stop-Process -Id $jp.Id -Force -ErrorAction Stop
                    Write-HeartbeatLog "INFO" "Killed confirmed orphan java PID=$($jp.Id) (WS=$([math]::Round($jp.WorkingSet64/1MB,1))MB, orphanAge=${orphanAge}s)"
                    $killed++
                } catch {
                    Write-HeartbeatLog "WARN" "Could not kill orphan java PID=$($jp.Id): $($_.Exception.Message)"
                }
            } else {
                # Has parent -- remove from tracker if present
                $Script:JavaOrphanTracker = @($Script:JavaOrphanTracker | Where-Object { $_.Pid -ne $jp.Id })
            }
        }
    } catch {
        Write-HeartbeatLog "ERROR" "Orphan java cleanup error: $($_.Exception.Message)"
    }

    # Prune tracker of dead PIDs
    $livePids = @($javaProcs | ForEach-Object { $_.Id })
    $Script:JavaOrphanTracker = @($Script:JavaOrphanTracker | Where-Object { $livePids -contains $_.Pid })

    if ($killed -gt 0) {
        Write-HeartbeatLog "INFO" "Java cleanup: $killed confirmed orphan process(es) terminated"
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

# === ORPHAN HANDLING ===
# NEVER kill claude.exe -- it is the master process. Pre-existing claude sessions
# occupy slots. Orphan cleanup reclaims task files only, never terminates processes.

function Invoke-ReclaimOrphanTasks {
    param([int]$InProgressCount)

    Write-HeartbeatLog "WARN" "ORPHAN DETECTED: $InProgressCount IN_PROGRESS task(s) but 0 active agent slots. Reclaiming task files (never killing claude.exe)."

    # Count pre-existing claude.exe processes that will occupy slots
    $preExistingClaude = @(Get-CimInstance Win32_Process -Filter "Name = 'claude.exe'" -ErrorAction SilentlyContinue).Count
    if ($preExistingClaude -gt 0) {
        Write-HeartbeatLog "INFO" "Pre-existing claude.exe processes: $preExistingClaude (occupying $preExistingClaude slot(s) -- accounted for in slot calculations)"
    }

    # Reclaim orphaned IN_PROGRESS tasks back to todo/ with [STALE]_ prefix
    $reclaimed = 0
    try {
        $inProgFiles = Get-ChildItem -LiteralPath $Script:InProgDir -Filter "*.md" -ErrorAction SilentlyContinue
        foreach ($f in $inProgFiles) {
            $canonicalName = Get-CanonicalTaskName -FileName ([System.IO.Path]::GetFileNameWithoutExtension($f.Name))
            $destName = "[STALE]_$canonicalName.md"
            $destPath = Join-Path $Script:TodoDir $destName

            # Check if a [STALE] version already exists to avoid overwrite
            if (Test-Path -LiteralPath $destPath) {
                Write-HeartbeatLog "DEBUG" "Skipping reclaim of $($f.Name) -- [STALE] version already exists in todo/"
                continue
            }

            try {
                Move-Item -LiteralPath $f.FullName -Destination $destPath -Force -ErrorAction Stop
                Write-HeartbeatLog "INFO" "Reclaimed orphan task $($f.Name) -> $destName"
                $reclaimed++

                # Clean up agent directory markers for this task
                $agentDir = Join-Path $Script:BaseDir ".claude\agents\$canonicalName"
                if (Test-Path $agentDir) {
                    # Remove dispatched_at and pid markers but keep job_registry.json for diagnostics
                    $dispatchMarker = Join-Path $agentDir "dispatched_at"
                    $pidFile = Join-Path $agentDir "pid"
                    if (Test-Path $dispatchMarker) { Remove-Item -LiteralPath $dispatchMarker -Force -ErrorAction SilentlyContinue }
                    if (Test-Path $pidFile) { Remove-Item -LiteralPath $pidFile -Force -ErrorAction SilentlyContinue }
                }
            } catch {
                Write-HeartbeatLog "ERROR" "Failed to reclaim orphan task $($f.Name): $($_.Exception.Message)"
            }
        }
    } catch {
        Write-HeartbeatLog "ERROR" "Orphan task reclaim error: $($_.Exception.Message)"
    }

    # Clean up orphaned Java processes via centralized tracker (requires confirmation period)
    $javaKilled = Invoke-CleanupOrphanJava -MinAgeSeconds 60 -ConfirmationSeconds 120

    Write-HeartbeatLog "INFO" "Orphan handling complete: $reclaimed task(s) reclaimed, $javaKilled java process(es) cleaned, $preExistingClaude claude.exe slot(s) accounted for"
    return @{ Reclaimed = $reclaimed; JavaKilled = $javaKilled; PreExistingClaude = $preExistingClaude }
}

# === HEARTBEAT PULSE ===

function Invoke-HeartbeatPulse {
    Write-HeartbeatLog "DEBUG" "=== PULSE $(Get-Date -Format 'HH:mm:ss') ==="
    $Script:PulseCount++

    # Refresh quota from ollama.com (with cooldown), then re-evaluate every pulse
    $null = Invoke-QuotaRefresh
    $null = Invoke-RefreshModelAllowlist
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
            $orchLogFile = Join-Path $Script:BaseDir ".claude\agents\orchestrator\agent.log"
            $statusExists = Test-Path $orchStatusFile
            $logEmpty = if (Test-Path $orchLogFile) { (Get-Item $orchLogFile).Length -lt 200 } else { $true }

            if ($statusExists) {
                $statusContent = Get-Content $orchStatusFile -Raw -ErrorAction SilentlyContinue
                Write-HeartbeatLog "INFO" "Orchestrator status written:"
                foreach ($line in ($statusContent -split "`n" | Where-Object { $_ -match '\S' })) {
                    Write-HeartbeatLog "INFO" "  $($line.Trim())"
                }
                if ($statusContent -match "STATUS=ALL_DONE") {
                    Write-HeartbeatLog "INFO" "Orchestrator reports ALL_DONE - swarm may be complete"
                }
            } elseif ($logEmpty) {
                # Silent failure: no status file AND empty agent.log = model failed to load/process
                Write-HeartbeatLog "ERROR" "Orchestrator SILENT FAILURE: no status file AND agent.log empty (<200b). Model $($Script:LastOrchModelDispatched) likely crashed. Will retry with fallback."
                $Script:OrchLastFailedModel = $Script:LastOrchModelDispatched
                # Reset completion flag and cooldown so retry happens immediately
                $Script:OrchCompletedCurrentCycle = $false
                $Script:LastOrchLaunch = [datetime]::MinValue
                # Don't increment cycle count for a failed run
                $Script:OrchCycleCount--
            } else {
                Write-HeartbeatLog "WARN" "Orchestrator completed but did NOT write ORCHESTRATOR_STATUS.md (agent.log has content -- partial success possible)"
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
    # Throttle to every 6 pulses (~60s) and require confirmation to avoid killing startup processes
    if ($Script:PulseCount % 6 -eq 0) {
        $null = Invoke-CleanupOrphanJava -MinAgeSeconds 60 -ConfirmationSeconds 120
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
    $trackedCount   = $slotState.TrackedAgents

    Write-HeartbeatLog "INFO" ("State -> Done: {0} | Pending: {1} | InProg: {2} | Failed/Stale: {3} | StaleInProg: {4} | Slots: jobs={5} claude={6} tracked={7} free={8} orphan={9} phase={10} | Orch: {11}" -f
        $fileState.DoneCount, $fileState.PendingCount, $fileState.InProgressCount,
        $fileState.FailedOrStaleCount, $staleTasks.Count, $workerCount, $claudeCount, $trackedCount, $slotsFree, $orphanLabel, $Script:CurrentPhaseName, $orchLabel)

    $Script:PrevDoneCount = $fileState.DoneCount

    # Phase 3: Orphan cleanup (IN_PROGRESS tasks but no active slots)
    if ($fileState.OrphanInProgress) {
        $null = Invoke-ReclaimOrphanTasks -InProgressCount $fileState.InProgressCount
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
            $task = $null

            # At MICRO phase (Tier 5), only dispatch P0/emergency tasks
            if ($Script:CurrentQuotaTier -ge 5) {
                $task = $fileState.PendingTasks | Where-Object { $_.Name -match "p0|P0|BLOCKED_BY_QUOTA|EMERGENCY" } | Select-Object -First 1
                if (-not $task) {
                    Write-HeartbeatLog "DEBUG" "$($Script:CurrentPhaseName): no P0 tasks available to dispatch"
                }
            }
            # MICRO batching: if 2+ MICRO tasks exist (excluding batch files) and no batch already pending
            elseif (($fileState.PendingTasks | Where-Object { $_.Name -match "MICRO_" -and $_.Name -notmatch "BATCH_MICRO_" }).Count -ge 2 -and
                    -not ($fileState.PendingTasks | Where-Object { $_.Name -match "BATCH_MICRO_" })) {
                $batchFile = Invoke-CreateMicroBatch -PendingTasks $fileState.PendingTasks
                if ($batchFile) {
                    $task = $batchFile
                    Write-HeartbeatLog "INFO" "Dispatching MICRO batch worker"
                }
            }
            # If a BATCH_MICRO file is pending, dispatch it instead of individual MICRO tasks
            elseif ($fileState.PendingTasks | Where-Object { $_.Name -match "BATCH_MICRO_" } | Select-Object -First 1) {
                $task = $fileState.PendingTasks | Where-Object { $_.Name -match "BATCH_MICRO_" } | Select-Object -First 1
                Write-HeartbeatLog "INFO" "Dispatching existing MICRO batch: $($task.Name)"
            }
            else {
                $task = $fileState.PendingTasks | Select-Object -First 1
            }

            if ($task) {
                $null = Invoke-DispatchWorker -TaskFile $task
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
            $null = Invoke-DispatchOrchestrator -Reason $orchReason
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
    $null = Invoke-QuotaRefresh
    $null = Invoke-RefreshModelAllowlist
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
    # Preserve Worker_* jobs whose claude.exe processes are still alive --
    # killing the PS job disconnects the heartbeat from the running agent,
    # causing orphan detection to reclaim tasks that are still being worked on.
    $allJobs = @(Get-Job -ErrorAction SilentlyContinue)
    $preservedCount = 0
    $removedCount = 0
    foreach ($j in $allJobs) {
        if ($j.Name -like "Worker_*" -and $j.State -eq "Running") {
            $agentName = $j.Name -replace "^Worker_", ""
            $regFile = Join-Path $Script:BaseDir ".claude\agents\$agentName\job_registry.json"
            $isAlive = $false
            if (Test-Path $regFile) {
                try {
                    $reg = Get-Content -LiteralPath $regFile -Raw | ConvertFrom-Json
                    if ($reg.claude_pid -and $reg.claude_pid -ne 0) {
                        $proc = Get-Process -Id ([int]$reg.claude_pid) -ErrorAction SilentlyContinue
                        if ($proc) { $isAlive = $true }
                    }
                } catch {}
            }
            if ($isAlive) {
                Write-HeartbeatLog "INFO" "Preserving live Worker job: $($j.Name)"
                $preservedCount++
                continue
            }
        }
        Remove-Job -Job $j -Force -ErrorAction SilentlyContinue
        $removedCount++
    }
    Write-HeartbeatLog "INFO" "Job cleanup: removed=$removedCount preserved=$preservedCount"

    # Phase 7: Cleanup stale worktrees from prior sessions
    Invoke-CleanupStaleWorktrees

    # Phase 8: Load max_concurrent from agent_pool.json as authoritative source
    $poolConfig = Join-Path $Script:BaseDir ".claude\agent_pool.json"
    if (Test-Path $poolConfig) {
        try {
            $poolJson = Get-Content -Raw -LiteralPath $poolConfig | ConvertFrom-Json
            if ($poolJson.max_concurrent -and $poolJson.max_concurrent -gt 0) {
                if ($poolJson.max_concurrent -ne $Script:MaxConcurrentSlots) {
                    Write-HeartbeatLog "INFO" "Overriding MaxConcurrentSlots from agent_pool.json: $($poolJson.max_concurrent) (was $($Script:MaxConcurrentSlots))"
                    $Script:MaxConcurrentSlots = [int]$poolJson.max_concurrent
                }
            }
        } catch {
            Write-HeartbeatLog "WARN" "Could not parse agent_pool.json for max_concurrent; using default $($Script:MaxConcurrentSlots)"
        }
    }

    # Phase 9: Reclaim orphaned tasks from prior session (IN_PROGRESS tasks but no agents)
    if ($initial.OrphanInProgress) {
        Write-HeartbeatLog "WARN" "Boot-time orphan detected: $($initial.InProgressCount) IN_PROGRESS task(s) with no active agents"
        $null = Invoke-ReclaimOrphanTasks -InProgressCount $initial.InProgressCount
    }

    # Phase 10: Normalize IN_PROGRESS filenames
    Invoke-NormalizeInProgressFiles

    # Phase 11: Restore job awareness from prior session
    Restore-JobAwareness

    Write-HeartbeatLog "INFO" "Boot complete. Entering main dispatch loop."
    $Script:BootTime = Get-Date
}

# === BOOT: NORMALIZE IN_PROGRESS FILENAMES ===
# Ensure all files in IN_PROGRESS/ have the canonical IN_PROGRESS_ prefix
function Invoke-NormalizeInProgressFiles {
    $inProgFiles = Get-ChildItem -LiteralPath $Script:InProgDir -Filter "*.md" -ErrorAction SilentlyContinue
    $normalized = 0
    foreach ($f in $inProgFiles) {
        $expectedName = Get-InProgressName -FileName $f.Name
        if ($f.Name -ne $expectedName) {
            $newPath = Join-Path $Script:InProgDir $expectedName
            try {
                Rename-Item -LiteralPath $f.FullName -NewName $expectedName -ErrorAction Stop
                Write-HeartbeatLog "INFO" "Normalized IN_PROGRESS filename: $($f.Name) -> $expectedName"
                $normalized++
            } catch {
                Write-HeartbeatLog "WARN" "Failed to normalize IN_PROGRESS filename $($f.Name): $($_.Exception.Message)"
            }
        }
    }
    if ($normalized -gt 0) {
        Write-HeartbeatLog "INFO" "Normalized $normalized IN_PROGRESS filename(s)"
    }
}

# === BOOT: RESTORE JOB AWARENESS ===
# Read job_registry.json files from agent directories to rebuild context
# after a PS session restart
function Restore-JobAwareness {
    $agentsDir = Join-Path $Script:BaseDir ".claude\agents"
    if (-not (Test-Path $agentsDir)) { return }

    $agentDirs = Get-ChildItem -LiteralPath $agentsDir -Directory -ErrorAction SilentlyContinue
    $staleAgents = 0
    $liveAgents = 0

    foreach ($dir in $agentDirs) {
        $regFile = Join-Path $dir.FullName "job_registry.json"
        if (Test-Path $regFile) {
            try {
                $reg = Get-Content -LiteralPath $regFile -Raw | ConvertFrom-Json
                $isAlive = $false

                # Check claude_pid first (most accurate)
                if ($reg.claude_pid -and $reg.claude_pid -ne 0) {
                    $proc = Get-Process -Id ([int]$reg.claude_pid) -ErrorAction SilentlyContinue
                    if ($proc) {
                        $isAlive = $true
                        $liveAgents++
                        Write-HeartbeatLog "DEBUG" "Live agent $($dir.Name): claude_pid=$($reg.claude_pid) model=$($reg.model)"
                    }
                }

                # Fall back to legacy pid file
                if (-not $isAlive) {
                    $pidFile = Join-Path $dir.FullName "pid"
                    if (Test-Path $pidFile) {
                        try {
                            $legacyPid = [int](Get-Content -LiteralPath $pidFile -Raw).Trim()
                            $proc = Get-Process -Id $legacyPid -ErrorAction SilentlyContinue
                            if ($proc) {
                                $isAlive = $true
                                $liveAgents++
                                Write-HeartbeatLog "DEBUG" "Live agent $($dir.Name): legacy pid=$legacyPid"
                            }
                        } catch {}
                    }
                }

                if (-not $isAlive) {
                    $staleAgents++
                    Write-HeartbeatLog "DEBUG" "Stale agent $($dir.Name) -- process no longer running"
                }
            } catch {
                Write-HeartbeatLog "WARN" "Could not parse job registry for $($dir.Name)"
            }
        }
    }

    Write-HeartbeatLog "INFO" "Job awareness restored: $liveAgents live, $staleAgents stale"
    # Invalidate process count cache to force fresh counts
    $Script:CachedClaudeCount = $null
    $Script:CachedClaudeTimestamp = [datetime]::MinValue
}

# === REGISTER CTRL+C HANDLER ===
# Only available when PowerShell has an interactive console; silent no-op otherwise
try {
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
} catch [System.NotSupportedException] {
    Write-HeartbeatLog "DEBUG" "No interactive console available -- Ctrl+C handler not registered (running non-interactive)"
} catch {
    Write-HeartbeatLog "DEBUG" "Ctrl+C handler registration failed: $($_.Exception.Message)"
}

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

    # Count running agents before touching anything
    $runningClaude = @(Get-CimInstance Win32_Process -Filter "Name = 'claude.exe'" -ErrorAction SilentlyContinue).Count
    $runningWorkers = @(Get-Job -State Running -ErrorAction SilentlyContinue | Where-Object { $_.Name -like "Worker_*" }).Count

    # Clean up non-worker PS jobs only (orchestrator, stale)
    Get-Job -ErrorAction SilentlyContinue | Where-Object { $_.Name -notlike "Worker_*" } | ForEach-Object {
        try { Stop-Job -Job $_ -ErrorAction SilentlyContinue } catch {}
        try { Remove-Job -Job $_ -Force -ErrorAction SilentlyContinue } catch {}
    }

    Write-Host "Heartbeat loop stopped. No further tasks will be dispatched." -ForegroundColor Cyan
    if ($runningWorkers -gt 0) {
        Write-Host "[ACTIVE] $runningWorkers worker PS job(s) and $runningClaude claude.exe process(es) still running." -ForegroundColor Green
        Write-Host "[ACTIVE] Agents will complete independently. Re-run SwarmHeartbeat.ps1 later to resume orchestration." -ForegroundColor Green
    } elseif ($runningClaude -gt 0) {
        Write-Host "[ACTIVE] $runningClaude claude.exe process(es) still running (detached). They will complete independently." -ForegroundColor Green
    } else {
        Write-Host "No active agents. Clean shutdown." -ForegroundColor Green
    }
}
