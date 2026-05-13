param (
    [Parameter(Mandatory=$true)][string]$TaskFile,
    [Parameter(Mandatory=$true)][string]$Model,
    [Parameter(Mandatory=$true)][int]$BudgetLimit,
    [Parameter(Mandatory=$false)][int]$MaxRuntimeMinutes = 45
)

Write-Host "[GOVERNOR] Launching $Model for $TaskFile (Limit: $BudgetLimit tokens | Timeout: $MaxRuntimeMinutes min)" -ForegroundColor Cyan
$StartTime = Get-Date

$AgentProcess = Start-Process -FilePath "claude.cmd" -ArgumentList "--model", $Model, "--debug", "--dangerously-skip-permissions", '"Execute $TaskFile"' -PassThru -NoNewWindow
Start-Sleep -Seconds 3

$LogDir = "$env:USERPROFILE\.claude\debug"
$LatestLog = Get-ChildItem -Path $LogDir -Filter "*.txt" | Sort-Object LastWriteTime -Descending | Select-Object -First 1

if (-not $LatestLog) { Write-Host "[GOVERNOR] No debug log found. Exiting." -ForegroundColor Red; exit 1 }

$TotalTokens = 0
while (-not $AgentProcess.HasExited) {
    Start-Sleep -Seconds 5
    
    if ((Get-Date) -gt $StartTime.AddMinutes($MaxRuntimeMinutes)) {
        Stop-Process -Id $AgentProcess.Id -Force
        Rename-Item -Path $TaskFile -NewName ($TaskFile -replace 'HANDOFF\\todo\\', 'HANDOFF\todo\[TIME_BREACH]_')
        exit 1
    }

    $LogContent = Get-Content $LatestLog.FullName -Raw
    $In = ([regex]::Matches($LogContent, '"input_tokens"\s*:\s*(\d+)') | ForEach-Object { [int]$_.Groups[1].Value } | Measure-Object -Sum).Sum
    $Out = ([regex]::Matches($LogContent, '"output_tokens"\s*:\s*(\d+)') | ForEach-Object { [int]$_.Groups[1].Value } | Measure-Object -Sum).Sum
    $TotalTokens = $In + $Out

    if ($TotalTokens -gt $BudgetLimit) {
        Stop-Process -Id $AgentProcess.Id -Force
        Rename-Item -Path $TaskFile -NewName ($TaskFile -replace 'HANDOFF\\todo\\', 'HANDOFF\todo\[BUDGET_BREACH]_')
        exit 1
    }
}
Write-Host "[GOVERNOR] Agent completed natively. Tokens: $TotalTokens." -ForegroundColor Green