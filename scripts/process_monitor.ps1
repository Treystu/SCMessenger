# SCMessenger Process Monitor
# Checks for stuck agents and provides health status

$AGENT_ROOT = "$PSScriptRoot/../.claude/agents"
$MAX_RUNTIME_MINUTES = 60  # Alert if running longer than 1 hour

function Get-ProcessRuntime {
    param($process_id)

    try {
        $process = Get-Process -Id $process_id -ErrorAction Stop
        $runtime = (Get-Date) - $process.StartTime
        return $runtime
    } catch {
        return $null
    }
}

function Check-StuckAgents {
    Write-Host "=== SCMessenger Process Health Check ==="
    Write-Host "Check time: $(Get-Date)"
    Write-Host "========================================"

    $stuck_count = 0

    # Check CLI agents via PID files
    if (Test-Path $AGENT_ROOT) {
        $pid_files = Get-ChildItem "$AGENT_ROOT/*/pid" -ErrorAction SilentlyContinue

        foreach ($pid_file in $pid_files) {
            $process_id = Get-Content $pid_file -ErrorAction SilentlyContinue
            if ($process_id) {
                $runtime = Get-ProcessRuntime $process_id
                if ($runtime) {
                    $minutes = [math]::Round($runtime.TotalMinutes, 1)
                    $agent_id = (Split-Path (Split-Path $pid_file -Parent) -Leaf)

                    Write-Host "Agent $agent_id (PID: $process_id): Running for $minutes minutes"

                    if ($minutes -gt $MAX_RUNTIME_MINUTES) {
                        Write-Host "🚨 STUCK: $agent_id has been running for over $MAX_RUNTIME_MINUTES minutes" -ForegroundColor Red
                        $stuck_count++

                        # Get last activity from config
                        $config_file = "$AGENT_ROOT/$agent_id/config"
                        if (Test-Path $config_file) {
                            $last_task = Get-Content $config_file | Select-String "TASK="
                            Write-Host "   Last task: $($last_task -replace 'TASK=', '')" -ForegroundColor Yellow
                        }
                    }
                } else {
                    Write-Host "Agent $agent_id (PID: $process_id): Process not found (stale PID)" -ForegroundColor Yellow
                }
            }
        }
    }

    # Check native Claude processes
    $claude_processes = Get-Process -Name "claude*" -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $PID }

    foreach ($proc in $claude_processes) {
        $runtime = (Get-Date) - $proc.StartTime
        $minutes = [math]::Round($runtime.TotalMinutes, 1)

        Write-Host "Native Claude (PID: $($proc.Id)): Running for $minutes minutes"

        if ($minutes -gt $MAX_RUNTIME_MINUTES) {
            Write-Host "🚨 STUCK: Native Claude PID $($proc.Id) has been running for over $MAX_RUNTIME_MINUTES minutes" -ForegroundColor Red
            $stuck_count++

            # Try to get command line
            try {
                $cmdline = (Get-WmiObject Win32_Process -Filter "ProcessId = $($proc.Id)").CommandLine
                if ($cmdline -match "--model") {
                    $model = $cmdline -replace ".*--model\s+([^\s]+).*", '$1'
                    Write-Host "   Model: $model" -ForegroundColor Yellow
                }
            } catch {}
        }
    }

    if ($stuck_count -eq 0) {
        Write-Host "✅ All processes running normally" -ForegroundColor Green
    } else {
        Write-Host "🚨 Found $stuck_count potentially stuck processes" -ForegroundColor Red
    }

    return $stuck_count
}

function Get-AgentSessionLog {
    param($agent_id)

    $log_file = "$AGENT_ROOT/$agent_id/session.log"
    if (Test-Path $log_file) {
        Write-Host "=== Session Log for $agent_id ==="
        Get-Content $log_file -Tail 10
        Write-Host "=================================="
    } else {
        Write-Host "No session log found for $agent_id"
    }
}

# Main execution
if ($args[0] -eq "tail") {
    if ($args[1]) {
        Get-AgentSessionLog $args[1]
    } else {
        Write-Host "Usage: .\process_monitor.ps1 tail <agent_id>"
    }
} else {
    Check-StuckAgents
}