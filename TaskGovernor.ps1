param (
    [Parameter(Mandatory=$true)]
    [ValidateScript({ Test-Path -LiteralPath $_ -PathType Leaf })]
    [string]$TaskFile,

    [Parameter(Mandatory=$true)]
    [string]$Model,

    [Parameter(Mandatory=$false)]
    [int]$BudgetLimit = 3600
)

$ErrorActionPreference = "Stop"

function Write-GovernorLog {
    param([string]$Level, [string]$Message)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss.fff"
    "[$timestamp][GOVERNOR][$Level] $Message"
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
    $prompt = "SYSTEM OVERRIDE: Read and execute all instructions in $TaskFilePath. Do not ask for help. CRITICAL: Do NOT move, rename, or relocate the task file -- the governor handles file movement. When finished, output TASK COMPLETE and exit."

    $startTime = Get-Date
    $budgetSeconds = $Script:BudgetLimit
    $warnThreshold = [math]::Floor($budgetSeconds * 0.80)
    $warned = $false
    $timedOut = $false

    Write-GovernorLog "INFO" "Starting agent: model=$($Script:Model) budget=${budgetSeconds}s task=$TaskFilePath"

    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $cmd.Exe
    $psi.Arguments = $cmd.Args -join " "
    $psi.UseShellExecute = $false
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.CreateNoWindow = $true

    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $psi
    $process.Start() | Out-Null

    $process.StandardInput.WriteLine($prompt)
    $process.StandardInput.Close()

    while (-not $process.HasExited) {
        $elapsed = ((Get-Date) - $startTime).TotalSeconds

        if (-not $process.StandardOutput.EndOfStream) {
            $line = $process.StandardOutput.ReadLine()
            if ($line) { Write-Output $line }
        }

        if (-not $warned -and $elapsed -ge $warnThreshold) {
            $warned = $true
            Write-GovernorLog "WARN" "BUDGET WARNING: ${elapsed}s / ${budgetSeconds}s elapsed (80% threshold)"
        }

        if ($elapsed -ge $budgetSeconds) {
            Write-GovernorLog "ERROR" "BUDGET BREACH: ${elapsed}s >= ${budgetSeconds}s -- force-killing agent"
            $timedOut = $true
            try {
                Stop-Process -Id $process.Id -Force -ErrorAction Stop
                Start-Sleep -Milliseconds 500
                Get-CimInstance Win32_Process -Filter "ParentProcessId = $($process.Id)" -ErrorAction SilentlyContinue |
                    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
            } catch {
                Write-GovernorLog "ERROR" "Force kill failed: $($_.Exception.Message)"
            }
            break
        }

        Start-Sleep -Milliseconds 500
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
}

function Invoke-Governor {
    $taskFilePath = $Script:TaskFile
    $taskFileName = Split-Path $taskFilePath -Leaf
    $handoffRoot   = Split-Path (Split-Path $taskFilePath -Parent) -Parent
    $todoDir       = Join-Path $handoffRoot "todo"
    $doneDir       = Join-Path $handoffRoot "done"

    Write-GovernorLog "INFO" "Governor engaged: task=$taskFileName model=$($Script:Model) budget=$($Script:BudgetLimit)"

    $headers = Read-TaskHeaders -FilePath $taskFilePath

    if ($headers.Model -ne $Script:Model) {
        Write-GovernorLog "INFO" "Task file overrides model: $($headers.Model)"
        $Script:Model = $headers.Model
    }

    $result = Invoke-AgentWithBudget -TaskFilePath $taskFilePath

    # Determine destination based on result
    if ($result.TimedOut) {
        $destName = "[TIME_BREACH]_$taskFileName"
        $destPath = Join-Path $todoDir $destName
        $destLabel = "TIMEOUT"
    } elseif ($result.ExitCode -eq 0) {
        $destPath = Join-Path $doneDir $taskFileName
        $destLabel = "SUCCESS"
    } else {
        $destName = "[FAILED]_$taskFileName"
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
