$tasks = @('A-04', 'A-05', 'E-02', 'E-04', 'D-01', 'D-02', 'D-03', 'D-04', 'D-05', 'C-05', 'C-06', 'T-02', 'T-03', 'T-04')
$maxConcurrent = 2
$runningJobs = [System.Collections.ArrayList]::new()
$repoRoot = (Get-Location).Path

foreach ($t in $tasks) {
    $file = Get-ChildItem 'HANDOFF/todo' | Where-Object { $_.Name -match ('^' + $t) } | Select-Object -First 1
    if (-not $file) { continue }

    Write-Host "Queueing $($t)..."
    
    # Parse ## Target Files block from task markdown
    $content = Get-Content $file.FullName
    $in_target_files = $false
    $target_files = @()
    foreach ($line in $content) {
        if ($line -match '^## Target Files') {
            $in_target_files = $true
            continue
        }
        if ($in_target_files -and $line -match '^- `?([^`]+)`?') {
            $target_files += $matches[1]
        } elseif ($in_target_files -and $line -match '^##') {
            break
        }
    }
    
    $files_args = ''
    if ($target_files.Count -gt 0) {
        $files_args = '--files ' + ($target_files -join ' ')
    }

    $verify_cmd = "cargo check --workspace"
    if ($files_args -match "\.kt") {
        $verify_cmd = ".\gradlew.bat assembleDebug"
    }
    if ($files_args -match "\.swift") {
        $verify_cmd = "cd iOS && xcodebuild -project SCMessenger.xcodeproj -scheme SCMessenger -sdk iphonesimulator build"
    }

    $cmd = "python scripts/delegate_task.py --task `"$($file.FullName)`" --provider qwen --model qwen-plus --apply --verify `"$verify_cmd`" $files_args"
    
    # Wait if we hit max concurrency
    while ($runningJobs.Count -ge $maxConcurrent) {
        $finished = $runningJobs | Where-Object { $_.State -ne 'Running' }
        foreach ($job in $finished) {
            Receive-Job -Job $job -Keep
            if ($job.State -eq 'Completed' -and $job.ChildJobs[0].JobStateInfo.Reason.Message -notmatch 'failed') {
                $taskName = $job.Name
                $taskFile = Get-ChildItem 'HANDOFF/todo' | Where-Object { $_.Name -match ('^' + $taskName) } | Select-Object -First 1
                if ($taskFile) {
                    Move-Item $taskFile.FullName 'HANDOFF/done/' -Force
                    git add -A
                    git commit -m "swarm: completed $taskName"
                }
            } else {
                Write-Host "Task $($job.Name) failed!"
            }
            $runningJobs.Remove($job)
        }
        Start-Sleep -Seconds 2
    }

    $logFile = "$repoRoot/tmp/$t.log"
    $job = Start-Job -Name $t -ScriptBlock {
        param($cmdStr, $dir, $log)
        Set-Location $dir
        "Executing: $cmdStr" | Out-File $log -Append
        Invoke-Expression "$cmdStr 2>&1" | Out-File $log -Append
        if ($LASTEXITCODE -ne 0) { throw "failed" }
    } -ArgumentList $cmd, $repoRoot, $logFile
    
    [void]$runningJobs.Add($job)
}


# Wait for remaining jobs
while ($runningJobs.Count -gt 0) {
    $finished = $runningJobs | Where-Object { $_.State -ne 'Running' }
    foreach ($job in $finished) {
        Receive-Job -Job $job -Keep
        if ($job.State -eq 'Completed' -and $job.ChildJobs[0].JobStateInfo.Reason.Message -notmatch 'failed') {
            $taskName = $job.Name
            $taskFile = Get-ChildItem 'HANDOFF/todo' | Where-Object { $_.Name -match ('^' + $taskName) } | Select-Object -First 1
            if ($taskFile) {
                Move-Item $taskFile.FullName 'HANDOFF/done/' -Force
                git add -A
                git commit -m "swarm: completed $taskName"
            }
        } else {
            Write-Host "Task $($job.Name) failed!"
        }
        $runningJobs.Remove($job)
    }
    Start-Sleep -Seconds 2
}
