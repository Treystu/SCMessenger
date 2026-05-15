param (
    [Parameter(Mandatory=$true)]
    [ValidateScript({ Test-Path -LiteralPath $_ -PathType Leaf })]
    [string]$TaskFile,

    [Parameter(Mandatory=$true)]
    [string]$Model,

    [Parameter(Mandatory=$false)]
    [int]$BudgetLimit = 3600,

    [Parameter(Mandatory=$false)]
    [string]$QuotaContextJson = ""
)

$ErrorActionPreference = "Stop"

function Write-GovernorLog {
    param([string]$Level, [string]$Message)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss.fff"
    $line = "[$timestamp][GOVERNOR][$Level] $Message"
    # Write-Host goes directly to host console, bypassing job output buffering
    switch ($Level) {
        "ERROR" { Write-Host $line -ForegroundColor Red }
        "WARN"  { Write-Host $line -ForegroundColor Yellow }
        "INFO"  { Write-Host $line -ForegroundColor Cyan }
        "DEBUG" { Write-Host $line -ForegroundColor DarkGray }
        default { Write-Host $line }
    }
}

function Read-TaskHeaders {
    param([string]$FilePath)
    $headers = @{
        Model    = $Script:Model
        Budget   = $Script:BudgetLimit
        Target   = ""
        Fallback = ""
        Agent    = ""
    }
    try {
        $lines = Get-Content -LiteralPath $FilePath -TotalCount 50 -ErrorAction Stop
        foreach ($line in $lines) {
            if ($line -match "^#\s*(MODEL|Model)\s*:\s*(.+)")   { $headers.Model    = $matches[2].Trim() }
            if ($line -match "^#\s*(BUDGET|Budget)\s*:\s*(\d+)") { $headers.Budget   = [int]$matches[2] }
            if ($line -match "^#\s*TARGET\s*:\s*(.+)")          { $headers.Target   = $matches[1].Trim() }
            if ($line -match "^#\s*FALLBACK\s*:\s*(.+)")        { $headers.Fallback = $matches[1].Trim() }
            if ($line -match "^#\s*AGENT\s*:\s*(.+)")           { $headers.Agent    = $matches[1].Trim() }
        }
    } catch {
        Write-GovernorLog "ERROR" "Failed to read task headers: $($_.Exception.Message)"
    }
    if ($headers.Budget -ne $Script:BudgetLimit -and $Script:BudgetLimit -eq 3600) {
        $Script:BudgetLimit = $headers.Budget
    }
    $Script:Model = $headers.Model
    return $headers
}

function New-ClaudeCommand {
    param([string]$Model)
    $args = @(
        "launch",
        "claude",
        "--model", $Model,
        "--",
        "--dangerously-skip-permissions",
        "--print"
    )
    return @{ Exe = "ollama"; Args = $args }
}

function Invoke-AgentWithBudget {
    param([string]$TaskFilePath)

    $cmd = New-ClaudeCommand -Model $Script:Model

    # Build prompt with optional quota context
    $quotaBlock = ""
    if ($Script:QuotaContextJson -and $Script:QuotaContextJson.Trim() -ne "") {
        try {
            $qc = $Script:QuotaContextJson | ConvertFrom-Json
            $quotaBlock = @"

QUOTA CONTEXT (from SwarmHeartbeat at dispatch time):
  5-Hour Usage: $($qc.FiveHour)% (resets in ~$($qc.ResetMinutes) min)
  7-Day Usage: $($qc.SevenDay)%
  Active Phase: $($qc.Phase) (Tier $($qc.Tier) -- HARDLOCK at 99.5%)
  Your Budget: $($qc.Budget)s

NOTE: If the above data was scraped more than 5 minutes ago, trigger a forced
re-check by running:
  powershell -NoProfile -File OllamaQuotaScraper.ps1 -Quiet
Do NOT make quota-dependent decisions on stale data.

PARTIAL COMPLETION IS ACCEPTABLE. If you cannot finish within budget, write what
you completed and mark remaining work with [REMAINING] comments. Exit cleanly.

"@
        } catch {
            # JSON parse failed - skip quota block silently
        }
    }

    $prompt = "${quotaBlock}SYSTEM OVERRIDE: Read and execute all instructions in $TaskFilePath. Do not ask for help. CRITICAL: Do NOT move, rename, or relocate the task file -- the governor handles file movement. When finished, output TASK COMPLETE and exit."

    $startTime = Get-Date
    $budgetSeconds = $Script:BudgetLimit
    $warnThreshold = [math]::Floor($budgetSeconds * 0.80)
    $warned = $false
    $timedOut = $false

    Write-GovernorLog "INFO" "Starting agent: model=$($Script:Model) budget=${budgetSeconds}s task=$TaskFilePath"

    # Capture PID for agent directory tracking
    $taskFileName = Split-Path $TaskFilePath -Leaf
    $canonicalName = Get-CanonicalTaskName -FileName ([System.IO.Path]::GetFileNameWithoutExtension($taskFileName))
    $handoffRoot = Split-Path (Split-Path $TaskFilePath -Parent) -Parent
    $agentDir = Join-Path (Join-Path $handoffRoot ".claude\agents") $canonicalName

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $cmd.Exe
    $psi.Arguments = $cmd.Args -join " "
    $psi.UseShellExecute = $false
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true

    $process = $null
    $eventJobs = @()
    $outputList = New-Object System.Collections.ArrayList
    $stderrList = New-Object System.Collections.ArrayList
    $syncLock   = New-Object System.Object

    try {
        $process = New-Object System.Diagnostics.Process
        $process.StartInfo = $psi
        $process.Start() | Out-Null

        # Write agent directory and PID file for heartbeat tracking
        if (-not (Test-Path $agentDir)) {
            try { $null = New-Item -ItemType Directory -Path $agentDir -Force } catch {}
        }
        $pidFile = Join-Path $agentDir "pid"
        $process.Id | Out-File -LiteralPath $pidFile -Encoding utf8 -ErrorAction SilentlyContinue
        # Tree-walk to find claude.exe child of the ollama process
        Start-Sleep -Milliseconds 300
        $claudePid = $null
        try {
            $searchPids = @($process.Id)
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
            if ($null -ne $claudePid) {
                $claudePid | Out-File -LiteralPath $pidFile -Encoding utf8 -ErrorAction SilentlyContinue
            }
        } catch {}
        # Write job registry for persistent cross-session tracking
        $registryFile = Join-Path $agentDir "job_registry.json"
        try {
            $registry = @{
                source        = "TaskGovernor"
                task_file     = $taskFileName
                canonical_name = $canonicalName
                model         = $Script:Model
                budget        = $Script:BudgetLimit
                dispatched_at = (Get-Date -Format "o")
                governor_pid  = $PID
                process_pid   = $process.Id
                claude_pid    = $claudePid
            }
            $registry | ConvertTo-Json | Out-File -LiteralPath $registryFile -Encoding utf8 -ErrorAction SilentlyContinue
        } catch {}

        $process.StandardInput.WriteLine($prompt)
        $process.StandardInput.Close()

        # Register async event handlers for non-blocking output reading
        # Event actions run in child runspaces - MessageData passes shared .NET objects
        $evtOut = Register-ObjectEvent -InputObject $process -EventName "OutputDataReceived" `
            -MessageData @{ List = $outputList; Lock = $syncLock } `
            -Action {
                $data = $EventArgs.Data
                if ($null -ne $data) {
                    [System.Threading.Monitor]::Enter($Event.MessageData.Lock)
                    try { $null = $Event.MessageData.List.Add($data) }
                    finally { [System.Threading.Monitor]::Exit($Event.MessageData.Lock) }
                }
            }
        $eventJobs += $evtOut

        $evtErr = Register-ObjectEvent -InputObject $process -EventName "ErrorDataReceived" `
            -MessageData @{ List = $stderrList; Lock = $syncLock } `
            -Action {
                $data = $EventArgs.Data
                if ($null -ne $data) {
                    [System.Threading.Monitor]::Enter($Event.MessageData.Lock)
                    try { $null = $Event.MessageData.List.Add($data) }
                    finally { [System.Threading.Monitor]::Exit($Event.MessageData.Lock) }
                }
            }
        $eventJobs += $evtErr

        # Begin async reads AFTER registering handlers
        $process.BeginOutputReadLine()
        $process.BeginErrorReadLine()

        # Polling loop: drain output, enforce budget, check for exit
        while (-not $process.HasExited) {
            $elapsed = ((Get-Date) - $startTime).TotalSeconds

            # Drain accumulated output lines
            [System.Threading.Monitor]::Enter($syncLock)
            try {
                for ($i = 0; $i -lt $outputList.Count; $i++) {
                    Write-Output $outputList[$i]
                }
                $outputList.Clear()
                for ($i = 0; $i -lt $stderrList.Count; $i++) {
                    Write-Output "[STDERR] $($stderrList[$i])"
                }
                $stderrList.Clear()
            } finally {
                [System.Threading.Monitor]::Exit($syncLock)
            }

            # Budget warning at 80%
            if (-not $warned -and $elapsed -ge $warnThreshold) {
                $warned = $true
                Write-GovernorLog "WARN" "BUDGET WARNING: $([math]::Round($elapsed,1))s / ${budgetSeconds}s elapsed (80% threshold)"
            }

            # Budget breach: kill process tree
            if ($elapsed -ge $budgetSeconds) {
                Write-GovernorLog "ERROR" "BUDGET BREACH: $([math]::Round($elapsed,1))s >= ${budgetSeconds}s -- force-killing agent"
                $timedOut = $true
                try {
                    Stop-Process -Id $process.Id -Force -ErrorAction Stop
                    Start-Sleep -Milliseconds 500
                    Get-CimInstance Win32_Process -Filter "ParentProcessId = $($process.Id)" `
                        -ErrorAction SilentlyContinue |
                        ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
                } catch {
                    Write-GovernorLog "ERROR" "Force kill failed: $($_.Exception.Message)"
                }
                break
            }

            Start-Sleep -Milliseconds 500
        }

        # Flush any output that arrived after process exit or during kill
        Start-Sleep -Milliseconds 300
        [System.Threading.Monitor]::Enter($syncLock)
        try {
            for ($i = 0; $i -lt $outputList.Count; $i++) {
                Write-Output $outputList[$i]
            }
            $outputList.Clear()
            for ($i = 0; $i -lt $stderrList.Count; $i++) {
                Write-Output "[STDERR] $($stderrList[$i])"
            }
            $stderrList.Clear()
        } finally {
            [System.Threading.Monitor]::Exit($syncLock)
        }

        $process.WaitForExit(5000) | Out-Null
        $exitCode = $process.ExitCode
        $elapsedTotal = [math]::Round(((Get-Date) - $startTime).TotalSeconds, 1)

        Write-GovernorLog "INFO" "Agent exit: code=$exitCode elapsed=${elapsedTotal}s timeout=$timedOut"

        return @{
            ExitCode = $exitCode
            Elapsed  = $elapsedTotal
            TimedOut = $timedOut
        }
    } finally {
        # Cancel async reads
        try { $process.CancelOutputRead() } catch {}
        try { $process.CancelErrorRead() } catch {}

        # Unregister event subscribers
        foreach ($j in $eventJobs) {
            Unregister-Event -SourceIdentifier $j.Name -ErrorAction SilentlyContinue
            Remove-Job -Job $j -Force -ErrorAction SilentlyContinue
        }

        # Ensure child process tree is dead
        if ($process -and -not $process.HasExited) {
            try {
                Stop-Process -Id $process.Id -Force -ErrorAction Stop
                Start-Sleep -Milliseconds 500
                Get-CimInstance Win32_Process -Filter "ParentProcessId = $($process.Id)" `
                    -ErrorAction SilentlyContinue |
                    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
            } catch {
                Write-GovernorLog "WARN" "Process cleanup in finally failed: $($_.Exception.Message)"
            }
        }
        if ($process) { $process.Dispose() }
    }
}

function Get-CanonicalTaskName {
    param([string]$FileName)
    $name = $FileName
    while ($name.StartsWith("IN_PROGRESS_")) { $name = $name.Substring("IN_PROGRESS_".Length) }
    return $name
}

function Invoke-Governor {
    $taskFilePath = $Script:TaskFile
    $taskFileName = Split-Path $taskFilePath -Leaf
    $handoffRoot   = Split-Path (Split-Path $taskFilePath -Parent) -Parent
    $todoDir       = Join-Path $handoffRoot "todo"
    $doneDir       = Join-Path $handoffRoot "done"

    # Strip IN_PROGRESS_ prefix for destination naming and agent directory lookup
    $canonicalName = Get-CanonicalTaskName -FileName ([System.IO.Path]::GetFileNameWithoutExtension($taskFileName))

    Write-GovernorLog "INFO" "Governor engaged: task=$taskFileName canonical=$canonicalName model=$($Script:Model) budget=$($Script:BudgetLimit)"

    $headers = Read-TaskHeaders -FilePath $taskFilePath

    if ($headers.Model -ne $Script:Model) {
        Write-GovernorLog "INFO" "Task file overrides model: $($headers.Model)"
        $Script:Model = $headers.Model
    }

    $result = Invoke-AgentWithBudget -TaskFilePath $taskFilePath

    # Determine destination based on result, using canonical name (stripped of IN_PROGRESS_ prefix)
    if ($result.TimedOut) {
        $destName = "[TIME_BREACH]_$canonicalName.md"
        $destPath = Join-Path $todoDir $destName
        $destLabel = "TIMEOUT"
    } elseif ($result.ExitCode -eq 0) {
        $destName = "$canonicalName.md"
        $destPath = Join-Path $doneDir $destName
        $destLabel = "SUCCESS"
    } else {
        $destName = "[FAILED]_$canonicalName.md"
        $destPath = Join-Path $todoDir $destName
        $destLabel = "FAILED (exit=$($result.ExitCode))"
    }

    # Move the task file (agent may have already moved it per CLAUDE.md)
    if (Test-Path -LiteralPath $taskFilePath) {
        try {
            Move-Item -LiteralPath $taskFilePath -Destination $destPath -Force -ErrorAction Stop
            Write-GovernorLog "INFO" "RESULT: $destLabel -> $destPath"
        } catch {
            Write-GovernorLog "ERROR" "HANDOFF MOVE FAILED: $($_.Exception.Message)"
            exit 3
        }
    } else {
        Write-GovernorLog "INFO" "RESULT: $destLabel (agent already moved file; expected target: $destPath)"
    }

    # Write COMPLETION marker for heartbeat/orchestrator detection
    $agentDir = Join-Path (Split-Path $taskFilePath -Parent | Split-Path -Parent | Join-Path -ChildPath ".claude\agents") $canonicalName
    if (-not (Test-Path $agentDir)) {
        try { $null = New-Item -ItemType Directory -Path $agentDir -Force } catch {}
    }
    $completionFile = Join-Path $agentDir "COMPLETION"
    try {
        $completionContent = "STATUS=$destLabel`nTASK_FILE=$destPath`nCHANGED_FILES=unknown`nBUILD_STATUS=unknown`nCOMPLETED_AT=$(Get-Date -Format 'o')`nNEXT_TASK_REQUESTED=false`nGOVERNOR_EXIT_CODE=$($result.ExitCode)`nGOVERNOR_ELAPSED=$($result.Elapsed)`nGOVERNOR_TIMED_OUT=$($result.TimedOut)"
        $completionContent | Out-File -LiteralPath $completionFile -Encoding utf8 -ErrorAction SilentlyContinue
        Write-GovernorLog "INFO" "Wrote COMPLETION marker to $completionFile"
    } catch {
        Write-GovernorLog "WARN" "Failed to write COMPLETION marker: $($_.Exception.Message)"
    }

    exit $result.ExitCode
}

# === MAIN ===
try {
    Invoke-Governor
} catch {
    Write-GovernorLog "FATAL" "Unhandled exception: $($_.Exception.Message)"
    Write-GovernorLog "FATAL" "Stack trace: $($_.ScriptStackTrace)"
    exit 2
}
