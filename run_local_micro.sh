# SCMessenger Local Micro-Worker State Machine (Windows Native)
# Guarantees context clears by forcing a fresh process for every task.

$TodoDir = "HANDOFF\todo"
$DoneDir = "HANDOFF\done"
$BacklogDir = "HANDOFF\backlog"
$LogDir = ".claude\session_logs"
$PromptFile = ".claude\prompts\ministral_override.txt"
$LogFile = "$LogDir\latest_micro_run.log"

# Ensure our state directories exist so the script doesn't crash
$null = New-Item -ItemType Directory -Force -Path $DoneDir
$null = New-Item -ItemType Directory -Force -Path $BacklogDir
$null = New-Item -ItemType Directory -Force -Path $LogDir

# Load the lobotomized system prompt
$SystemPrompt = Get-Content -Path $PromptFile -Raw

Write-Host "Starting Local Micro-Worker Swarm (ministral-3:8b)..." -ForegroundColor Cyan

# Grab all pending tasks
$Tasks = Get-ChildItem -Path $TodoDir -Filter "*.md"
if ($Tasks.Count -eq 0) {
    Write-Host "No tasks found in $TodoDir. Exiting." -ForegroundColor Yellow
    exit
}

foreach ($Task in $Tasks) {
    Write-Host "=================================================="
    Write-Host "GRABBING TASK: $($Task.Name)" -ForegroundColor Green
    Write-Host "=================================================="
    
    $TaskContent = Get-Content -Path $Task.FullName -Raw
    
    # Construct the single-shot payload
    $FullPrompt = "$SystemPrompt`n`nYOUR CURRENT TASK:`n$TaskContent"

    # Launch Claude Code, execute the task, and tee the output to our log file
    & ollama launch claude --model ministral-3:8b -p $FullPrompt | Tee-Object -FilePath $LogFile
    
    # Read the log to evaluate the model's self-reported status
    $LogOutput = Get-Content -Path $LogFile -Raw

    if ($LogOutput -match "STATUS: SUCCESS_STOP") {
        Write-Host "✅ Task Successful! Moving to done..." -ForegroundColor Green
        Move-Item -Path $Task.FullName -Destination $DoneDir -Force
    }
    elseif ($LogOutput -match "STATUS: ESCALATE_TO_ORCHESTRATOR") {
        Write-Host "⚠️ Task Failed 3 times. Escalating for 671B model..." -ForegroundColor Yellow
        Move-Item -Path $Task.FullName -Destination $BacklogDir -Force
    }
    else {
        Write-Host "❌ Unexpected exit or crash. Leaving in todo\ for next retry." -ForegroundColor Red
    }

    Write-Host "Task complete. Destroying process to clear context window..." -ForegroundColor DarkGray
    Start-Sleep -Seconds 2 # RAM flush buffer
}

Write-Host "Queue empty or processed. Shutting down local swarm." -ForegroundColor Cyan