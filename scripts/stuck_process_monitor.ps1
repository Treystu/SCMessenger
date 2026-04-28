# SCMessenger Stuck Process Monitor
# Runs on 15-minute loop to detect and report stuck agents

$SCRIPT_DIR = $PSScriptRoot
$MONITOR_INTERVAL_MINUTES = 15

function Monitor-StuckProcesses {
    while ($true) {
        Write-Host "=== SCMessenger Stuck Process Monitor ==="
        Write-Host "Cycle start: $(Get-Date)"
        Write-Host "=========================================="

        # Run the process monitor
        $result = powershell -File "$SCRIPT_DIR/process_monitor.ps1"

        # Check if any processes are stuck
        if ($result -match "STUCK") {
            Write-Host "🚨 ALERT: Stuck processes detected!" -ForegroundColor Red

            # Send notification (could integrate with PushNotification tool)
            Write-Host "Sending alert for stuck processes..."

            # Log to file
            $log_entry = "$(Get-Date) - Stuck processes detected: $result"
            Add-Content "$SCRIPT_DIR/../.claude/stuck_process_log.txt" $log_entry
        } else {
            Write-Host "✅ All processes running normally" -ForegroundColor Green
        }

        Write-Host "Next check in $MONITOR_INTERVAL_MINUTES minutes..."
        Write-Host ""

        # Sleep for the monitoring interval
        Start-Sleep -Seconds ($MONITOR_INTERVAL_MINUTES * 60)
    }
}

# Start monitoring
Monitor-StuckProcesses