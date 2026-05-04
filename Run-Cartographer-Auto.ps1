<#
.SYNOPSIS
    The Perfection Engine - OOB Verification & Cartography Engine

.DESCRIPTION
    Scans the repository for logic files, chunks them, and aggressively parses them into a 
    perfect JSON schema using concurrent Ollama worker nodes. Self-corrects bad JSON 
    output automatically.

.PARAMETER Pool
    The total number of CPU threads across all Ollama instances to utilize concurrently. Default: 11.

.PARAMETER RamLimit
    The maximum allowed RAM usage percentage before the master dispatcher starts throttling tasks. Default: 85.

.EXAMPLE
    .\Run-Cartographer-Auto.ps1
    Runs with defaults: 11 Threads, 85% RAM limit.

.EXAMPLE
    .\Run-Cartographer-Auto.ps1 -Pool 8 -RamLimit 75
    Runs with 8 threads max, throttling if RAM exceeds 75%.
#>
# Run-Cartographer-Auto.ps1 (V51 - The Perfection Engine)
[CmdletBinding()]
param (
    [int]$Pool = 11,       
    [int]$RamLimit = 85    
)

$ErrorActionPreference = "Stop"
try { [Console]::TreatControlCAsInput = $false } catch {}

$QUEUE_DIR = "$PWD\HANDOFF_AUDIT"
$TODO = Join-Path $QUEUE_DIR "todo"
$PROC = Join-Path $QUEUE_DIR "processing"
$DONE = Join-Path $QUEUE_DIR "done"
$ERRS = Join-Path $QUEUE_DIR "errors"
$OUT_DIR = Join-Path $QUEUE_DIR "output"
$MASTER_MAP = Join-Path $QUEUE_DIR "REPO_MAP.jsonl"

foreach ($dir in @($QUEUE_DIR, $TODO, $PROC, $DONE, $ERRS, $OUT_DIR)) {
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
}

# =====================================================================
# THE PRE-FLIGHT PURGE & ORPHAN RESCUE
# =====================================================================
Write-Host "[INIT] Executing Pre-Flight Teardown & Orphan Rescue..." -ForegroundColor DarkYellow
Get-Job | Remove-Job -Force
try { & taskkill.exe /F /IM ollama.exe /T 2>&1 | Out-Null } catch {}
try { & taskkill.exe /F /IM ollama_llama_server.exe /T 2>&1 | Out-Null } catch {}
Stop-Process -Name "ollama", "ollama_llama_server" -Force -ErrorAction SilentlyContinue

$orphans = Get-ChildItem -Path $PROC -Filter *.txt
foreach ($orphan in $orphans) { 
    try { Move-Item -LiteralPath $orphan.FullName -Destination $TODO -Force -ErrorAction Stop } catch {}
}

$failures = Get-ChildItem -Path $ERRS -Filter *.txt
if ($failures.Count -gt 0) {
    Write-Host "[INIT] Rescuing $($failures.Count) failed tasks for retry..." -ForegroundColor Magenta
    foreach ($fail in $failures) { 
        try { Move-Item -LiteralPath $fail.FullName -Destination $TODO -Force -ErrorAction Stop } catch {}
    }
}
Start-Sleep -Seconds 2

# --- INITIALIZE NATIVE .NET HARDWARE SENSORS ---
Write-Host "[INIT] Booting .NET Hardware Sensors..." -ForegroundColor Cyan
Add-Type -AssemblyName Microsoft.VisualBasic
$global:ComputerInfo = New-Object Microsoft.VisualBasic.Devices.ComputerInfo

try {
    $global:CpuCounter = New-Object System.Diagnostics.PerformanceCounter("Processor", "% Processor Time", "_Total")
    $global:CpuCounter.NextValue() | Out-Null 
} catch {
    Write-Host "[WARN] Windows CPU Counters bypassed to prevent hangs." -ForegroundColor Yellow
    $global:CpuCounter = $null
}

function Get-SystemTelemetry {
    $totalRam = $global:ComputerInfo.TotalPhysicalMemory
    $freeRam = $global:ComputerInfo.AvailablePhysicalMemory
    $ram = [math]::Round((($totalRam - $freeRam) / $totalRam) * 100)

    $cpu = 0
    if ($null -ne $global:CpuCounter) {
        try { $cpu = [math]::Round($global:CpuCounter.NextValue()) } catch { $cpu = 0 }
    }
    return [PSCustomObject]@{ CPU = $cpu; RAM = $ram }
}

function Wait-ForStabilization {
    Write-Host "   [WAIT] Analyzing Telemetry (Waiting for CPU/RAM to digest load)..." -ForegroundColor DarkCyan
    $history = New-Object System.Collections.ArrayList
    
    for ($w = 0; $w -lt 45; $w++) {
        $tel = Get-SystemTelemetry
        $history.Add($tel) | Out-Null
        if ($history.Count -gt 5) { $history.RemoveAt(0) }
        
        if ($history.Count -eq 5) {
            $rMin = ($history | Measure-Object RAM -Minimum).Minimum
            $rMax = ($history | Measure-Object RAM -Maximum).Maximum
            $cMin = ($history | Measure-Object CPU -Minimum).Minimum
            $cMax = ($history | Measure-Object CPU -Maximum).Maximum
            
            $rVar = $rMax - $rMin
            $cVar = $cMax - $cMin
            
            Write-Host "`r       -> CPU: $($tel.CPU)% (Var: $cVar%) | RAM: $($tel.RAM)% (Var: $rVar%)   " -NoNewline -ForegroundColor DarkGray
            
            if ($rVar -le 2 -and ($cVar -le 4 -or $global:CpuCounter -eq $null)) {
                Write-Host "`n   [READY] System Stabilized. Ready for next dispatch." -ForegroundColor Green
                return
            }
        } else {
            Write-Host "`r       -> CPU: $($tel.CPU)% | RAM: $($tel.RAM)% (Baselining...)   " -NoNewline -ForegroundColor DarkGray
        }
        Start-Sleep -Seconds 1
    }
    Write-Host "`n   [TIMEOUT] Stabilization timeout reached. Proceeding cautiously." -ForegroundColor Yellow
}

# --- THE PRE-PARSER ENGINE (Programmatic Skeleton Extraction) ---
function Get-CodeSkeleton {
    param([string]$FilePath)
    $Content = Get-Content -Path $FilePath -Raw
    $Extension = [System.IO.Path]::GetExtension($FilePath)
    
    $Skeleton = @{
        classes = @()
        funcs = @()
    }
    
    $Lines = $Content -split "`r?`n"
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        $Line = $Lines[$i]
        $LineNum = $i + 1
        
        # Simple Regex for common languages
        if ($Extension -match "\.rs|\.kt|\.swift|\.java|\.cpp|\.c|\.h|\.ts|\.js") {
            # Classes / Structs
            if ($Line -match "^\s*(pub\s+|private\s+|internal\s+)?(class|struct|interface|enum|object|actor)\s+([a-zA-Z0-9_]+)") {
                $Skeleton.classes += [PSCustomObject]@{ name = $matches[3]; line = $LineNum }
            }
            # Functions
            if ($Line -match "^\s*(pub\s+|private\s+|protected\s+|internal\s+|async\s+|suspend\s+|override\s+)*\b(fn|fun|func|void|int|bool|string|Task)\b\s+([a-zA-Z0-9_]+)\s*\(") {
                $Skeleton.funcs += [PSCustomObject]@{ name = $matches[3]; line = $LineNum }
            }
        }
    }
    return $Skeleton
}

# --- THE ISOLATED WORKER POD (ScriptBlock) ---
$SingleShotWorker = {
    param($File, $ChunkLines, $ChunkIndex, $TotalChunks, $Model, $NumCtx, $Threads, $Port, $OutDir, $AttemptReason, $Skeleton, $StartLine, $EndLine)
    $ErrorActionPreference = "Stop"

    $outName = "$($File.Name)_chunk$($ChunkIndex).jsonl"
    $outPath = Join-Path $OutDir $outName
    $tmpPath = $outPath + ".tmp"
    
    if (Test-Path -LiteralPath $outPath) { return "SUCCESS: SKIPPED (Already exists)" }

    try {
        $booted = $false
        for ($k = 0; $k -lt 15; $k++) {
            Start-Sleep -Seconds 1
            try { $null = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/api/tags" -TimeoutSec 1; $booted = $true; break } catch { }
        }
        if (-not $booted) { throw "Server failed to bind to port $Port" }

        $chunkContent = $ChunkLines -join "`n"
        $skeletonJson = $Skeleton | ConvertTo-Json -Depth 5 -Compress

        $basePrompt = @"
You are an expert code cartographer. Analyze this chunk and extract its architecture.
I have pre-parsed the file and found the following symbols. Use this as your source of truth.

FILE SKELETON (Source of Truth):
$skeletonJson

GOAL:
Extract the summary, class details, imports, and cross-function calls for this specific chunk.
Output RAW JSON ONLY. Do not use markdown blocks. Use the exact line numbers provided in the code.

Schema:
{
  "file": "$($File.Name)",
  "chunk": $ChunkIndex,
  "summary": "REPLACE_WITH_SUMMARY",
  "structs_or_classes": [],
  "imports": [],
  "funcs": [
    {
      "name": "REPLACE_WITH_NAME",
      "line": 1,
      "calls_out_to": []
    }
  ]
}

CODE CHUNK (Lines $StartLine - $EndLine):
$chunkContent
"@

        $retryError = $AttemptReason
        $maxRetries = 2
        
        for ($retry = 0; $retry -le $maxRetries; $retry++) {
            $prompt = $basePrompt
            if ($retryError) {
                $prompt = "CRITICAL JSON ERROR PREVIOUSLY: $retryError`n`nFIX THIS AND RE-OUTPUT PERFECT JSON.`n`n" + $basePrompt
            }

            $body = @{
                model = $Model; prompt = $prompt; stream = $false
                options = @{ num_ctx = $NumCtx; temperature = 0.1; num_thread = $Threads }
            } | ConvertTo-Json -Depth 10 -Compress

            try {
                $response = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/api/generate" -Method Post -Body ([System.Text.Encoding]::UTF8.GetBytes($body)) -ContentType "application/json" -TimeoutSec 600
                
                $json = $response.response.Trim()
                if ($json.StartsWith('```json')) { $json = $json.Substring(7) }
                elseif ($json.StartsWith('```')) { $json = $json.Substring(3) }
                if ($json.EndsWith('```')) { $json = $json.Substring(0, $json.Length - 3) }
                $json = $json.Trim()

                # Robust JSON extraction
                $startIndex = $json.IndexOf('{')
                $endIndex = $json.LastIndexOf('}')
                if ($startIndex -ge 0 -and $endIndex -ge $startIndex) {
                    $json = $json.Substring($startIndex, $endIndex - $startIndex + 1)
                }

                # STRICT DATA VERIFICATION (No Placeholders Allowed)
                $parsed = ConvertFrom-Json $json -ErrorAction Stop
                
                if ($null -eq $parsed.file) { throw "Missing 'file' field." }
                if ($null -eq $parsed.funcs) { throw "Missing 'funcs' array." }
                if ($null -eq $parsed.imports) { throw "Missing 'imports' array." }
                if ($null -eq $parsed.structs_or_classes) { throw "Missing 'structs_or_classes' array." }
                
                # Check for prompt template bleed-through
                if ($parsed.summary -match "Detailed summary" -or $parsed.summary -eq "REPLACE_WITH_SUMMARY") { throw "Placeholder summary detected." }
                if ($parsed.structs_or_classes -contains "List") { throw "Placeholder classes detected." }
                if ($parsed.imports -contains "List") { throw "Placeholder imports detected." }

                if ($parsed.funcs.Count -gt 0) {
                    foreach ($f in $parsed.funcs) {
                        if ($null -eq $f.name -or $f.name -eq "REPLACE_WITH_NAME" -or $f.name -eq "function_name") { throw "Function is missing name or has placeholder." }
                        if ($null -eq $f.line -or $f.line -eq 0) { throw "Function '$($f.name)' has invalid/zero line number." }
                        if ($null -eq $f.calls_out_to) { throw "Function '$($f.name)' missing calls_out_to array." }
                        if ($f.calls_out_to -contains "funcs") { throw "Function '$($f.name)' has placeholder calls." }
                    }
                }
                
                # Minimum summary quality check
                if ($null -eq $parsed.summary -or $parsed.summary.Length -lt 25) {
                    $parsed | Add-Member -NotePropertyName "_REVIEW_REQUIRED" -NotePropertyValue $true
                    $json = $parsed | ConvertTo-Json -Depth 10 -Compress
                }

                # ATOMIC WRITE AND RENAME (Guarantees the Master Engine won't read a half-written file)
                $json | Out-File -FilePath $tmpPath -Encoding utf8
                Rename-Item -Path $tmpPath -NewName $outName -Force
                
                return "SUCCESS: Completed normally."

            } catch {
                $retryError = $_.Exception.Message
                if ($retry -eq $maxRetries) {
                    throw "JSON VALIDATION FAILED AFTER $maxRetries RETRIES: $retryError"
                }
                Start-Sleep -Seconds 2
            }
        }
    } catch {
        return "FAILED: $($_.Exception.Message)"
    }
}

# =====================================================================
# THE MASTER ENGINE
# =====================================================================
$global:TrackedOllamaPIDs = New-Object System.Collections.ArrayList

try {
    Write-Host "[INIT] Scanning repository for LOGIC files only..." -ForegroundColor Cyan
    $BASE_DIR = (Get-Location).Path
    
    $allFiles = Get-ChildItem -Path $BASE_DIR -Recurse -Include *.rs, *.kt, *.swift, *.java, *.py, *.js, *.ts, *.cpp, *.c, *.h | 
                Where-Object { 
                    $_.FullName -notmatch "HANDOFF" -and 
                    $_.FullName -notmatch "\.claude" -and 
                    $_.FullName -notmatch "target\\" -and 
                    $_.FullName -notmatch "build\\" -and 
                    $_.FullName -notmatch "node_modules\\" 
                }

    $jobQueue = New-Object System.Collections.Queue
    $priorityQueue = New-Object System.Collections.Queue
    $chunkSize = 350
    $skippedCount = 0

    foreach ($file in $allFiles) {
        $lines = Get-Content -LiteralPath $file.FullName -ErrorAction SilentlyContinue
        if ($null -eq $lines -or $lines.Count -eq 0) { continue }

        $totalChunks = [math]::Ceiling($lines.Count / $chunkSize)
        for ($i = 0; $i -lt $totalChunks; $i++) {
            $chunkIndex = $i + 1
            $ticketName = "$($file.Name)_chunk$($chunkIndex).txt"
            $expectedOutName = "$($file.Name)_chunk$($chunkIndex).jsonl"
            
            # Skip completed files
            if ((Test-Path -LiteralPath (Join-Path $DONE $ticketName)) -or (Test-Path -LiteralPath (Join-Path $OUT_DIR $expectedOutName))) {
                $skippedCount++
                continue 
            }

            $startLine = $i * $chunkSize
            $endLine = [math]::Min(($startLine + $chunkSize - 1), ($lines.Count - 1))
            
            $chunkLines = @()
            for ($j = $startLine; $j -le $endLine; $j++) { $chunkLines += "$($j + 1): $($lines[$j])" }

            $ticketTodo = Join-Path $TODO $ticketName
            if (-not (Test-Path $ticketTodo)) {
                try { "FILE: $($file.FullName)`nCHUNK: $chunkIndex" | Out-File -FilePath $ticketTodo -Encoding utf8 -ErrorAction Stop } catch {}
            }

            $charCount = ($chunkLines -join "").Length
            $estimatedTokens = [math]::Ceiling($charCount / 3) + 500
            
            if ($estimatedTokens -lt 2048) { $numCtx = 2048 }
            elseif ($estimatedTokens -lt 4096) { $numCtx = 4096 }
            elseif ($estimatedTokens -lt 8192) { $numCtx = 8192 }
            else { $numCtx = 16384 }

            if ($file.Extension -match "\.rs|\.kt|\.swift|\.java|\.cpp|\.c|\.h") {
                $model = "qwen2.5-coder:3b"; $baseThreads = 4
            } else {
                $model = "qwen2.5-coder:1.5b"; $baseThreads = 3
            }

            $contextThreads = [math]::Floor($estimatedTokens / 1500) 
            $reqThreads = [math]::Max(1, [math]::Min($Pool, ($baseThreads + $contextThreads)))

            $jobQueue.Enqueue([PSCustomObject]@{
                File = $file; ChunkLines = $chunkLines; ChunkIndex = $chunkIndex; TicketName = $ticketName
                TotalChunks = $totalChunks; Model = $model; NumCtx = $numCtx; 
                ReqThreads = $reqThreads; Tokens = $estimatedTokens; Attempt = 1; AttemptReason = "";
                Skeleton = (Get-CodeSkeleton -FilePath $file.FullName); StartLine = ($startLine + 1); EndLine = ($endLine + 1)
            })
        }
    }
    $totalChunksFound = $skippedCount + $jobQueue.Count
    Write-Host "[INIT] Pre-Flight Complete. Scanned $totalChunksFound total chunks. Ignored $skippedCount already processed." -ForegroundColor DarkYellow
    Write-Host "[INIT] Built Queue: $($jobQueue.Count) tasks ready for execution." -ForegroundColor Green
    Write-Host "[INIT] System Config: Max Threads = $Pool | Max RAM = $RamLimit%" -ForegroundColor Green

    $activePods = New-Object System.Collections.ArrayList
    $availablePorts = New-Object System.Collections.Queue
    11450..11465 | ForEach-Object { $availablePorts.Enqueue($_) } 

    Write-Host "[DEBUG] Entering Main Dispatcher Loop..." -ForegroundColor DarkCyan

    while ($jobQueue.Count -gt 0 -or $priorityQueue.Count -gt 0 -or $activePods.Count -gt 0) {
        
        $toRemove = New-Object System.Collections.ArrayList
        
        # === OUT-OF-BAND (OOB) STATE VERIFICATION ===
        foreach ($pod in $activePods) {
            $jobQuery = Get-Job -Id $pod.Job.Id -ErrorAction SilentlyContinue
            $expectedOutFile = Join-Path $OUT_DIR "$($pod.File.Name)_chunk$($pod.ChunkIndex).jsonl"
            
            $isDone = $false
            $isSuccess = $false
            $reason = ""

            # Check 1: Did the Atomic JSON file magically appear? (Worker finished but hung on teardown)
            if (Test-Path -LiteralPath $expectedOutFile) {
                $isDone = $true
                $isSuccess = $true
                $reason = "OOB Verification (File Exists)"
            }
            # Check 2: Did the job exit naturally?
            elseif ($null -eq $jobQuery -or $jobQuery.State -ne 'Running') {
                $isDone = $true
                if ($null -ne $jobQuery -and $jobQuery.State -eq 'Completed') {
                    $output = Receive-Job -Job $pod.Job -ErrorAction SilentlyContinue
                    if ($output -match "SUCCESS") { 
                        $isSuccess = $true; $reason = "Standard Exit" 
                    } else { 
                        $null = $output -match "FAILED:(.*)"
                        $reason = "Validation Failed: " + $matches[1]
                    }
                } else {
                    $reason = "Ghost Process / Crashed"
                }
            }

            if ($isDone) {
                if ($isSuccess) {
                    Write-Host "   [SUCCESS] $($pod.File.Name) (Chunk $($pod.ChunkIndex)/$($pod.TotalChunks)) [$reason]" -ForegroundColor DarkGreen
                    try { Move-Item -LiteralPath (Join-Path $PROC $pod.TicketName) -Destination $DONE -Force -ErrorAction Stop } catch {}
                    
                    if (Test-Path -LiteralPath $expectedOutFile) {
                        try { Get-Content $expectedOutFile | Out-File -FilePath $MASTER_MAP -Append -Encoding utf8 -ErrorAction Stop } catch {}
                    }
                } else {
                    Write-Host "   [ERROR] FAILED: $reason | File: $($pod.File.Name)" -ForegroundColor Red
                    
                    if ($pod.Attempt -lt 2) {
                        Write-Host "   [ESCALATING] Routing $($pod.File.Name) Chunk $($pod.ChunkIndex) to Heavyweight 3B Model..." -ForegroundColor Magenta
                        try { Move-Item -LiteralPath (Join-Path $PROC $pod.TicketName) -Destination $TODO -Force -ErrorAction Stop } catch {}

                        $escalatedThreads = [math]::Max(1, [math]::Min($Pool, (4 + [math]::Floor($pod.Tokens / 1500))))
                        $priorityQueue.Enqueue([PSCustomObject]@{
                            File = $pod.File; ChunkLines = $pod.ChunkLines; ChunkIndex = $pod.ChunkIndex; TicketName = $pod.TicketName
                            TotalChunks = $pod.TotalChunks; Model = "qwen2.5-coder:3b"; NumCtx = $pod.NumCtx; 
                            ReqThreads = $escalatedThreads; Tokens = $pod.Tokens; Attempt = 2; AttemptReason = $reason
                        })
                    } else {
                        Write-Host "   [FATAL] File failed parsing even after 3B Escalation." -ForegroundColor DarkRed
                        try { Move-Item -LiteralPath (Join-Path $PROC $pod.TicketName) -Destination $ERRS -Force -ErrorAction Stop } catch {}
                    }
                }
                
                # Sniping the job forcefully kills any WMI locks and fires the cleanup block instantly
                Remove-Job -Job $pod.Job -Force -ErrorAction SilentlyContinue
                
                # Terminate tracked Ollama PID
                if ($null -ne $pod.OllamaPID) {
                    # CRITICAL FIX: taskkill /T must run FIRST to trace and kill child runner processes. 
                    # If Stop-Process runs first, the parent dies and child processes become untraceable ghosts.
                    try { & taskkill.exe /F /T /PID $($pod.OllamaPID) 2>&1 | Out-Null } catch {}
                    try { Stop-Process -Id $pod.OllamaPID -Force -ErrorAction SilentlyContinue } catch {}
                    $global:TrackedOllamaPIDs.Remove($pod.OllamaPID) | Out-Null
                }
                
                # Failsafe: Snipe any ghost process that might have stolen or still holds the port
                try {
                    $conns = Get-NetTCPConnection -LocalPort $pod.Port -ErrorAction SilentlyContinue
                    foreach ($conn in $conns) {
                        try { & taskkill.exe /F /T /PID $($conn.OwningProcess) 2>&1 | Out-Null } catch {}
                        try { Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue } catch {}
                    }
                } catch {}

                $availablePorts.Enqueue($pod.Port) 
                $toRemove.Add($pod) | Out-Null
            }
        }
        foreach ($pod in $toRemove) { $activePods.Remove($pod) | Out-Null }

        $sysStatus = Get-SystemTelemetry
        $currentRam = $sysStatus.RAM
        
        $activeThreads = 0
        foreach ($pod in $activePods) { $activeThreads += $pod.Threads }
        $availableThreads = $Pool - $activeThreads

        if ($jobQueue.Count -gt 0 -or $priorityQueue.Count -gt 0) {
            if ($availablePorts.Count -eq 0) {
                Write-Host "`r[WARN] Waiting for a Port to free up... " -NoNewline -ForegroundColor Yellow
            } else {
                $nextTask = if ($priorityQueue.Count -gt 0) { $priorityQueue.Peek() } else { $jobQueue.Peek() }
                
                $ramOkay = ($currentRam -lt $RamLimit) -or ($activePods.Count -eq 0)
                $threadsOkay = ($availableThreads -ge 1) -or ($activePods.Count -eq 0)
                
                if ($ramOkay -and $threadsOkay) {
                    $task = if ($priorityQueue.Count -gt 0) { $priorityQueue.Dequeue() } else { $jobQueue.Dequeue() }
                    $port = $availablePorts.Dequeue()
                    
                    try { Move-Item -LiteralPath (Join-Path $TODO $task.TicketName) -Destination (Join-Path $PROC $task.TicketName) -Force -ErrorAction Stop } catch {}

                    $allocatedThreads = [math]::Min($task.ReqThreads, $availableThreads)
                    if ($allocatedThreads -lt 1) { $allocatedThreads = 1 } 
                    
                    # Start Ollama directly in the master script to track the PID perfectly
                    $env:OLLAMA_HOST = "127.0.0.1:$port"
                    $ollamaProc = Start-Process -FilePath "ollama" -ArgumentList "serve" -WindowStyle Hidden -PassThru
                    $ollamaPID = $ollamaProc.Id
                    $global:TrackedOllamaPIDs.Add($ollamaPID) | Out-Null

                    $newJob = Start-Job -ScriptBlock $SingleShotWorker -ArgumentList $task.File, $task.ChunkLines, $task.ChunkIndex, $task.TotalChunks, $task.Model, $task.NumCtx, $allocatedThreads, $port, $OUT_DIR, $task.AttemptReason, $task.Skeleton, $task.StartLine, $task.EndLine
                    
                    $activePods.Add([PSCustomObject]@{ 
                        Job = $newJob; Port = $port; Threads = $allocatedThreads; TicketName = $task.TicketName;
                        File = $task.File; ChunkLines = $task.ChunkLines; ChunkIndex = $task.ChunkIndex;
                        TotalChunks = $task.TotalChunks; NumCtx = $task.NumCtx; Tokens = $task.Tokens;
                        Attempt = $task.Attempt; OllamaPID = $ollamaPID
                    }) | Out-Null
                    
                    Write-Host "`n-> Launched: $($task.File.Name) (Chunk $($task.ChunkIndex)/$($task.TotalChunks)) [$($task.Model)]"
                    Write-Host "   [INFO] Specs: $allocatedThreads Threads | Port: $port | Attempt: $($task.Attempt)" -ForegroundColor Cyan
                    
                    Wait-ForStabilization
                } else {
                    $totalQ = $jobQueue.Count + $priorityQueue.Count
                    Write-Host "`r[STATUS] Queue: $totalQ | Active Pods: $($activePods.Count) | Threads: $activeThreads/$Pool | RAM: $currentRam% / $RamLimit%   " -NoNewline -ForegroundColor Yellow
                }
            }
        }
        Start-Sleep -Seconds 1 
    }

    Write-Host "`n[COMPLETE] Swarm Audit Complete. Aggregated output saved to HANDOFF_AUDIT\REPO_MAP.jsonl" -ForegroundColor Magenta

    Write-Host "[INDEX] Building REPO_MAP index metadata..." -ForegroundColor Cyan
    try {
        & python .claude/scripts/build_repo_index.py --full-rebuild
        Write-Host "[INDEX] repo_map_index.json generated successfully." -ForegroundColor Green
    } catch {
        Write-Host "[INDEX] WARNING: Failed to build index: $($_.Exception.Message)" -ForegroundColor Yellow
    }

} catch {
    Write-Host "`n[FATAL ERROR] An unexpected exception crashed the script:" -ForegroundColor Red
    Write-Host "$($_.Exception.Message)" -ForegroundColor Red
} finally {
    Write-Host "`n[CLEANUP] Executing Nuclear Teardown..." -ForegroundColor Red
    
    Write-Host "   -> Terminating Tracked Ollama instances..." -ForegroundColor DarkGray
    if ($null -ne $global:TrackedOllamaPIDs) {
        foreach ($pid in $global:TrackedOllamaPIDs) {
            try { & taskkill.exe /F /T /PID $pid 2>&1 | Out-Null } catch {}
            try { Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue } catch {}
        }
    }

    Write-Host "   -> Performing aggressive sweep for orphaned Ollama processes..." -ForegroundColor DarkGray
    try { & taskkill.exe /F /IM ollama.exe /T 2>&1 | Out-Null } catch {}
    try { & taskkill.exe /F /IM ollama_llama_server.exe /T 2>&1 | Out-Null } catch {}

    $lingering = Get-Process -Name "ollama", "ollama_llama_server" -ErrorAction SilentlyContinue
    if ($lingering) {
        foreach ($proc in $lingering) {
            try { & taskkill.exe /F /T /PID $($proc.Id) 2>&1 | Out-Null } catch {}
            try { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue } catch {}
        }
    }

    Write-Host "   -> Validating process termination..." -ForegroundColor DarkGray
    Start-Sleep -Seconds 2
    $stillAlive = Get-Process -Name "ollama", "ollama_llama_server" -ErrorAction SilentlyContinue
    if ($stillAlive) {
        Write-Host "   [FATAL] SOME OLLAMA PROCESSES RESISTED TERMINATION!" -ForegroundColor Red
        $stillAlive | Select-Object Name, Id | Format-Table | Out-String | Write-Host -ForegroundColor Red
    } else {
        Write-Host "   [SUCCESS] Zero Ollama instances remaining in memory." -ForegroundColor Green
    }

    Write-Host "[CLEANUP] Resources Freed. Good to go." -ForegroundColor Green
}