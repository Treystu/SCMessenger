<#
.SYNOPSIS
    The Perfection Engine (Swarm Edition) - Persistent Worker Pool
.DESCRIPTION
    A more robust version of the Cartographer that maintains a persistent pool of 
    Ollama workers to avoid constant process spawning and port binding failures.
#>
[CmdletBinding()]
param (
    [int]$PoolCount = 3,    # Number of persistent Ollama instances
    [int]$ThreadsPerPod = 2, # CPU threads per Ollama instance
    [int]$RamLimit = 75     # RAM limit to throttle launches
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
# PRE-FLIGHT PURGE
# =====================================================================
Write-Host "[INIT] Nuclear Purge of existing Ollama instances..." -ForegroundColor DarkYellow
Get-Job | Remove-Job -Force
try { & taskkill.exe /F /IM ollama.exe /T 2>&1 | Out-Null } catch {}
try { & taskkill.exe /F /IM ollama_llama_server.exe /T 2>&1 | Out-Null } catch {}
Stop-Process -Name "ollama", "ollama_llama_server" -Force -ErrorAction SilentlyContinue

# Rescue orphans
Get-ChildItem -Path $PROC -Filter *.txt | Move-Item -Destination $TODO -Force -ErrorAction SilentlyContinue
Get-ChildItem -Path $ERRS -Filter *.txt | Move-Item -Destination $TODO -Force -ErrorAction SilentlyContinue

# Hardware Sensors
Add-Type -AssemblyName Microsoft.VisualBasic
$global:ComputerInfo = New-Object Microsoft.VisualBasic.Devices.ComputerInfo
try {
    $global:CpuCounter = New-Object System.Diagnostics.PerformanceCounter("Processor", "% Processor Time", "_Total")
    $global:CpuCounter.NextValue() | Out-Null 
} catch { $global:CpuCounter = $null }

function Get-SystemTelemetry {
    $totalRam = $global:ComputerInfo.TotalPhysicalMemory
    $freeRam = $global:ComputerInfo.AvailablePhysicalMemory
    $ram = [math]::Round((($totalRam - $freeRam) / $totalRam) * 100)
    $cpu = 0
    if ($null -ne $global:CpuCounter) { try { $cpu = [math]::Round($global:CpuCounter.NextValue()) } catch {} }
    return [PSCustomObject]@{ CPU = $cpu; RAM = $ram }
}

# --- THE PRE-PARSER ---
function Get-CodeSkeleton {
    param([string]$FilePath)
    $Content = Get-Content -Path $FilePath -Raw
    $Extension = [System.IO.Path]::GetExtension($FilePath)
    $Skeleton = @{ classes = @(); funcs = @() }
    $Lines = $Content -split "`r?`n"
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        $Line = $Lines[$i]; $LineNum = $i + 1
        if ($Extension -match "\.rs|\.kt|\.swift|\.java|\.cpp|\.c|\.h|\.ts|\.js") {
            if ($Line -match "^\s*(pub\s+|private\s+|internal\s+)?(class|struct|interface|enum|object|actor)\s+([a-zA-Z0-9_]+)") {
                $Skeleton.classes += [PSCustomObject]@{ name = $matches[3]; line = $LineNum }
            }
            if ($Line -match "^\s*(pub\s+|private\s+|protected\s+|internal\s+|async\s+|suspend\s+|override\s+)*\b(fn|fun|func|void|int|bool|string|Task)\b\s+([a-zA-Z0-9_]+)\s*\(") {
                $funcName = $matches[3]; $isStub = $false; $lookAhead = ""
                for ($k = 0; $k -le 3; $k++) { if ($i + $k -lt $Lines.Count) { $lookAhead += $Lines[$i + $k] } }
                if ($lookAhead -match "todo!|unimplemented!|TODO\(\)|//\s*stub|\{\s*\}|pass|FIXME") { $isStub = $true }
                $Skeleton.funcs += [PSCustomObject]@{ name = $funcName; line = $LineNum; is_stub = $isStub }
            }
        }
    }
    return $Skeleton
}

# --- THE SWARM MANAGER ---
$global:Workers = New-Object System.Collections.ArrayList

function Start-OllamaWorker {
    param($Port)
    Write-Host "   [SWARM] Booting Worker on Port $Port..." -ForegroundColor Cyan
    $env:OLLAMA_HOST = "127.0.0.1:$Port"
    $proc = Start-Process -FilePath "ollama" -ArgumentList "serve" -WindowStyle Hidden -PassThru
    
    $booted = $false
    for ($k = 0; $k -lt 45; $k++) {
        Start-Sleep -Seconds 1
        try { 
            $tags = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/api/tags" -TimeoutSec 1
            $booted = $true; break 
        } catch { }
    }
    if (-not $booted) { throw "Worker on port $Port failed to bind after 45s." }
    return $proc
}

function Restart-Worker {
    param($Worker)
    Write-Host "   [RECOVERY] Restarting crashed worker on Port $($Worker.Port)..." -ForegroundColor Red
    try { & taskkill.exe /F /T /PID $($Worker.PID) 2>&1 | Out-Null } catch {}
    $proc = Start-OllamaWorker -Port $Worker.Port
    $Worker.PID = $proc.Id
    $Worker.Proc = $proc
    $Worker.Busy = $false
    $Worker.ConsecutiveFailures = 0
}

try {
    # 1. Initialize Workers
    Write-Host "[INIT] Initializing $PoolCount Persistent Workers..." -ForegroundColor Green
    for ($i = 0; $i -lt $PoolCount; $i++) {
        $port = 11450 + $i
        $proc = Start-OllamaWorker -Port $port
        $global:Workers.Add([PSCustomObject]@{ 
            Port = $port; PID = $proc.Id; Busy = $false; Proc = $proc; ConsecutiveFailures = 0 
        }) | Out-Null
    }

    # 2. Build Queue
    Write-Host "[INIT] Scanning repository for LOGIC files..." -ForegroundColor Cyan
    $allFiles = Get-ChildItem -Path . -Recurse -Include *.rs, *.kt, *.swift, *.java, *.py, *.js, *.ts, *.cpp, *.c, *.h | 
                Where-Object { $_.FullName -notmatch "HANDOFF|\.claude|target|node_modules|build" }
    
    $jobQueue = New-Object System.Collections.Queue
    $chunkSize = 300

    foreach ($file in $allFiles) {
        $lines = Get-Content -LiteralPath $file.FullName -ErrorAction SilentlyContinue
        if ($null -eq $lines -or $lines.Count -eq 0) { continue }
        $totalChunks = [math]::Ceiling($lines.Count / $chunkSize)
        $fullSkeleton = Get-CodeSkeleton -FilePath $file.FullName

        for ($i = 0; $i -lt $totalChunks; $i++) {
            $chunkIndex = $i + 1
            $expectedOut = Join-Path $OUT_DIR "$($file.Name)_chunk$($chunkIndex).jsonl"
            if (Test-Path $expectedOut) { continue }

            $startLine = $i * $chunkSize
            $endLine = [math]::Min(($startLine + $chunkSize - 1), ($lines.Count - 1))
            $chunkLines = @()
            for ($j = $startLine; $j -le $endLine; $j++) { $chunkLines += "$($j + 1): $($lines[$j])" }

            $scopedSkeleton = @{
                classes = @($fullSkeleton.classes | Where-Object { $_.line -ge ($startLine+1) -and $_.line -le ($endLine+1) })
                funcs = @($fullSkeleton.funcs | Where-Object { $_.line -ge ($startLine+1) -and $_.line -le ($endLine+1) })
            }

            $model = "qwen2.5-coder:1.5b"
            if (($scopedSkeleton.classes.Count + $scopedSkeleton.funcs.Count) -gt 8) { $model = "qwen2.5-coder:3b" }

            $jobQueue.Enqueue([PSCustomObject]@{
                File = $file; ChunkLines = $chunkLines; ChunkIndex = $chunkIndex; TotalChunks = $totalChunks
                Model = $model; Attempt = 1; Skeleton = $scopedSkeleton; StartLine = ($startLine+1); EndLine = ($endLine+1)
            })
        }
    }

    Write-Host "[SWARM] Queue Ready: $($jobQueue.Count) tasks. Pool Size: $PoolCount" -ForegroundColor Green

    # 3. Dispatch Loop
    $activeTasks = New-Object System.Collections.ArrayList
    while ($jobQueue.Count -gt 0 -or $activeTasks.Count -gt 0) {
        
        $toRemove = New-Object System.Collections.ArrayList
        foreach ($task in $activeTasks) {
            if ($task.Job.State -ne 'Running') {
                $rawOutput = Receive-Job $task.Job
                $outputStr = if ($null -ne $rawOutput) { [string]::Join(" ", $rawOutput) } else { "" }
                $worker = $global:Workers | Where-Object { $_.Port -eq $task.Port }
                
                if ($outputStr -match "SUCCESS") {
                    Write-Host "   [SUCCESS] $($task.FileName) Chunk $($task.ChunkIndex)" -ForegroundColor Green
                    $worker.ConsecutiveFailures = 0
                } else {
                    Write-Host "   [ERROR] $($task.FileName) Chunk $($task.ChunkIndex) FAILED: $outputStr" -ForegroundColor Red
                    $worker.ConsecutiveFailures++
                    
                    if ($outputStr -match "500" -or $outputStr -match "unreachable" -or $worker.ConsecutiveFailures -gt 2) {
                        Restart-Worker -Worker $worker
                    }

                    if ($task.Attempt -lt 2) {
                        Write-Host "   [ESCALATING] Retrying with 3B Model..." -ForegroundColor Magenta
                        $task.OriginalTask.Attempt = 2
                        $task.OriginalTask.Model = "qwen2.5-coder:3b"
                        $jobQueue.Enqueue($task.OriginalTask)
                    } else {
                        Write-Host "   [FATAL] Final failure for $($task.FileName) Chunk $($task.ChunkIndex)" -ForegroundColor DarkRed
                    }
                }
                $worker.Busy = $false
                Remove-Job $task.Job -Force
                $toRemove.Add($task) | Out-Null
            }
        }
        foreach ($t in $toRemove) { $activeTasks.Remove($t) | Out-Null }

        # Dispatch
        $tel = Get-SystemTelemetry
        if ($tel.RAM -le $RamLimit -and $jobQueue.Count -gt 0) {
            $idleWorkers = $global:Workers | Where-Object { $_.Busy -eq $false }
            if ($null -ne $idleWorkers -and $idleWorkers.Count -gt 0) {
                $idleWorker = $idleWorkers | Select-Object -First 1
                $task = $jobQueue.Dequeue()
                $idleWorker.Busy = $true
                
                $numCtx = if ($task.Model -match "3b") { 8192 } else { 4096 }

                $prompt = @"
You are a code cartographer. Analyze this chunk and extract architecture RAW JSON ONLY.
FILE: $($task.File.Name) Chunk $($task.ChunkIndex)
SKELETON: $($task.Skeleton | ConvertTo-Json -Depth 5 -Compress)
GOAL: Extract summary, classes, imports, and calls.
Set "is_stub": true for todo!(), unimplemented!(), or empty bodies.
CODE (Lines $($task.StartLine) - $($task.EndLine)):
$($task.ChunkLines -join "`n")
"@
                $job = Start-Job -ScriptBlock {
                    param($Port, $Model, $Prompt, $Threads, $NumCtx, $FileName, $ChunkIndex, $OutDir)
                    try {
                        $body = @{ model = $Model; prompt = $Prompt; stream = $false; options = @{ num_ctx = $NumCtx; num_thread = $Threads; temperature = 0.1 } } | ConvertTo-Json -Depth 10 -Compress
                        $resp = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/api/generate" -Method Post -Body ([System.Text.Encoding]::UTF8.GetBytes($body)) -ContentType "application/json" -TimeoutSec 600
                        $json = $resp.response.Trim()
                        $startIndex = $json.IndexOf('{'); $endIndex = $json.LastIndexOf('}')
                        if ($startIndex -ge 0) { $json = $json.Substring($startIndex, $endIndex - $startIndex + 1) }
                        
                        # Basic JSON validation check
                        $null = ConvertFrom-Json $json -ErrorAction Stop
                        
                        $outPath = Join-Path $OutDir "$FileName`_chunk$ChunkIndex.jsonl"
                        $json | Out-File -FilePath $outPath -Encoding utf8
                        return "SUCCESS"
                    } catch { return "FAILED: $($_.Exception.Message)" }
                } -ArgumentList $idleWorker.Port, $task.Model, $prompt, $ThreadsPerPod, $numCtx, $task.File.Name, $task.ChunkIndex, $OUT_DIR

                $activeTasks.Add([PSCustomObject]@{ 
                    Job = $job; Port = $idleWorker.Port; FileName = $task.File.Name; 
                    ChunkIndex = $task.ChunkIndex; Attempt = $task.Attempt; OriginalTask = $task 
                }) | Out-Null
                Write-Host "-> Launched: $($task.File.Name) [$($task.Model)] on Port $($idleWorker.Port)" -ForegroundColor Gray
            }
        }
        Write-Host "`r[SWARM] Queue: $($jobQueue.Count) | Active: $($activeTasks.Count) | RAM: $($tel.RAM)%   " -NoNewline -ForegroundColor Yellow
        Start-Sleep -Seconds 1
    }

    Write-Host "`n[COMPLETE] Swarm processing finished." -ForegroundColor Magenta

} finally {
    Write-Host "`n[EXIT] Nuking swarm..." -ForegroundColor Red
    if ($null -ne $global:Workers) {
        foreach ($w in $global:Workers) { try { & taskkill.exe /F /T /PID $($w.PID) 2>&1 | Out-Null } catch {} }
    }
    try { & taskkill.exe /F /IM ollama.exe /T 2>&1 | Out-Null } catch {}
}
