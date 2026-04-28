# run_local_micro.ps1
# SCMessenger Local Micro-Worker State Machine (Windows Native)
# Guarantees context clears by forcing a fresh process for every task.


# 1. Trick Claude Code into thinking it has a valid Anthropic account
$env:ANTHROPIC_API_KEY="sk-ant-dummy-key-for-local-ollama"

# 2. OVERRIDE THE ROGUE VRAM LIMIT (Forces 8k context instead of 131k!)
$env:OLLAMA_CONTEXT_LENGTH="8192"

# 3. FORCE Claude Code to wait up to 5 minutes
$env:ANTHROPIC_TIMEOUT="300000"
$env:ANTHROPIC_MAX_RETRIES="3"

# 4. FORCE Ollama to keep the model locked in RAM permanently
$env:OLLAMA_KEEP_ALIVE="-1"
$TodoDir = "HANDOFF\todo"
$DoneDir = "HANDOFF\done"
$BacklogDir = "HANDOFF\backlog"
$LogDir = ".claude\session_logs"
$PromptFile = "C:\Users\kanal\.claude\prompts\ministral_override.txt" # FIXED ABSOLUTE PATH
$LogFile = "$LogDir\latest_micro_run.log"

# Ensure our state directories exist so the script doesn't crash
$null = New-Item -ItemType Directory -Force -Path $DoneDir
$null = New-Item -ItemType Directory -Force -Path $BacklogDir
$null = New-Item -ItemType Directory -Force -Path $LogDir

# Load the lobotomized system prompt from your global Claude directory
$SystemPrompt = Get-Content -Path $PromptFile -Raw

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

    # Force-create an empty log file to prevent Get-Content object not found errors
    $null = New-Item -ItemType File -Force -Path $LogFile

    # Launch Claude Code, execute the task, and tee the output to our log file
    # Note the double dashes '--' to safely pass the prompt to Claude via the wrapper
    Write-Host "Starting Local Micro-Worker Swarm (qwen-micro)..." -ForegroundColor Cyan
    & ollama launch claude --model qwen-micro -- --debug -- -p $FullPrompt | Tee-Object -FilePath $LogFile
    
    # Read the log to evaluate the model's self-reported status
    $LogOutput = Get-Content -Path $LogFile -Raw

    if ($LogOutput -match "STATUS: SUCCESS_STOP") {
        Write-Host "✅ Task Successful! Moving to done..." -ForegroundColor Green
        Move-Item -Path $Task.FullName -Destination $DoneDir -Force
    }
    elseif ($LogOutput -match "STATUS: ESCALATE_TO_ORCHESTRATOR") {
        Write-Host "⚠️ Task Failed repeatedly. Escalating for 671B model..." -ForegroundColor Yellow
        Move-Item -Path $Task.FullName -Destination $BacklogDir -Force
    }
    else {
        Write-Host "❌ Unexpected exit or crash. Leaving in todo\ for next retry." -ForegroundColor Red
    }

    Write-Host "Task complete. Destroying process to clear context window..." -ForegroundColor DarkGray
    Start-Sleep -Seconds 2 # RAM flush buffer
}

Write-Host "Queue empty or processed. Shutting down local swarm." -ForegroundColor Cyan