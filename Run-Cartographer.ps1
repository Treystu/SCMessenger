# Run-Worker.ps1 (V26 - Zero-Leak Strict PID Isolation)
param (
    [int]$Port = 11440, # Start at 11440, use 11441 for terminal 2, etc.
    [int]$Threads = 4  
)

$ErrorActionPreference = "Continue"
[Console]::TreatControlCAsInput = $false

# --- ABSOLUTE FOLDER STRUCTURE ---
$BASE_DIR = (Get-Location).Path
$QUEUE_DIR = Join-Path $BASE_DIR "HANDOFF_AUDIT"
$TODO = Join-Path $QUEUE_DIR "todo"
$PROC = Join-Path $QUEUE_DIR "processing"
$DONE = Join-Path $QUEUE_DIR "done"
$ERRS = Join-Path $QUEUE_DIR "errors"
$OUTP = Join-Path $QUEUE_DIR "output"

foreach ($dir in @($QUEUE_DIR, $TODO, $PROC, $DONE, $ERRS, $OUTP)) {
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
}

$workerId = [guid]::NewGuid().ToString().Substring(0, 4)
$apiUrl = "http://127.0.0.1:$Port/api/generate"
$global:ActiveOllamaPID = $null

# --- STRICT PROCESS TREE KILLER ---
function Kill-OllamaTree {
    if ($null -ne $global:ActiveOllamaPID) {
        Write-Host "   [⚙️] Tearing down Ollama process tree (PID: $($global:ActiveOllamaPID))..." -ForegroundColor DarkGray
        # /T kills child processes (ollama_llama_server.exe). /F forces it.
        & taskkill /F /T /PID $global:ActiveOllamaPID 2>&1 | Out-Null
        $global:ActiveOllamaPID = $null
        Start-Sleep -Seconds 1 # Let Windows free the RAM and Port
    }
}

# --- GRACEFUL EXIT HANDLER ---
trap { 
    Write-Host "`n[WORKER $workerId] INTERRUPT DETECTED." -ForegroundColor Red
    Kill-OllamaTree
    Write-Host "[WORKER $workerId] Shut down safely." -ForegroundColor Yellow
    exit 
}

# --- QUEUE INITIALIZATION ---
$existingTasks = (Get-ChildItem -LiteralPath $TODO).Count + (Get-ChildItem -LiteralPath $PROC).Count + (Get-ChildItem -LiteralPath $DONE).Count + (Get-ChildItem -LiteralPath $ERRS).Count
if ($existingTasks -eq 0) {
    Write-Host "[INIT] Generating Task Queue..." -ForegroundColor Cyan
    $allFiles = Get-ChildItem -Path $BASE_DIR -Recurse -Include *.py, *.rs, *.swift, *.kt | 
                Where-Object { $_.FullName -notmatch "HANDOFF" -and $_.FullName -notmatch "\.claude" }
    
    foreach ($file in $allFiles) {
        $safeName = [guid]::NewGuid().ToString() + ".txt"
        $taskPath = Join-Path $TODO $safeName
        $file.FullName | Out-File -FilePath $taskPath -Encoding utf8 -NoNewline
    }
    Write-Host "[INIT] Added $($allFiles.Count) tasks." -ForegroundColor Green
}

Write-Host "[WORKER $workerId] Assigned to Port $Port. Hunting for tasks..." -ForegroundColor Magenta

# --- CORE EXECUTION LOOP ---
while ($true) {
    $taskFile = Get-ChildItem -LiteralPath $TODO -Filter *.txt | Sort-Object { Get-Random } | Select-Object -First 1
    
    if ($null -eq $taskFile) {
        Write-Host "[WORKER $workerId] No tasks remaining. Swarm Complete!" -ForegroundColor Green
        break
    }

    $processingPath = Join-Path $PROC $taskFile.Name

    # ATOMIC CLAIM
    try { Move-Item -LiteralPath $taskFile.FullName -Destination $processingPath -ErrorAction Stop } 
    catch { continue }

    $targetFilePath = (Get-Content -LiteralPath $processingPath -Raw).Trim()
    
    if (-not (Test-Path -LiteralPath $targetFilePath)) {
        Move-Item -LiteralPath $processingPath -Destination $ERRS -Force
        continue
    }

    $targetFile = Get-Item -LiteralPath $targetFilePath
    Write-Host "`n-> [$workerId | Port $Port] Auditing: $($targetFile.Name)" -ForegroundColor White

    $allLines = Get-Content -LiteralPath $targetFile.FullName -ErrorAction SilentlyContinue
    if ($null -eq $allLines -or $allLines.Count -eq 0) {
        Move-Item -LiteralPath $processingPath -Destination $DONE -Force
        continue
    }

    # ========================================================
    # ISOLATED TASK EXECUTION BLOCK (Guarantees Cleanup)
    # ========================================================
    try {
        # 1. BOOT ISOLATED OLLAMA SERVER FOR THIS SPECIFIC FILE
        $env:OLLAMA_HOST = "127.0.0.1:$Port"
        $proc = Start-Process -FilePath "ollama" -ArgumentList "serve" -WindowStyle Hidden -PassThru
        $global:ActiveOllamaPID = $proc.Id
        
        Write-Host "   [+] Booted isolated Ollama Server (PID: $($proc.Id)). Waiting for API..." -ForegroundColor DarkGray
        
        $booted = $false
        for ($k = 0; $k -lt 15; $k++) {
            Start-Sleep -Seconds 1
            try {
                $null = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/api/tags" -Method Get -TimeoutSec 1 -ErrorAction Stop
                $booted = $true
                break
            } catch { }
        }
        
        if (-not $booted) { throw "Ollama failed to boot on port $Port." }

        # 2. PROCESS CHUNKS
        $chunkSize = 350
        $totalChunks = [math]::Ceiling($allLines.Count / $chunkSize)
        $outName = "$($targetFile.Name)_$($taskFile.Name.Replace('.txt','')).jsonl"
        $outputJsonlFile = Join-Path $OUTP $outName
        $fileCompletelySuccessful = $true

        for ($i = 0; $i -lt $totalChunks; $i++) {
            $startLine = $i * $chunkSize
            $endLine = [math]::Min(($startLine + $chunkSize - 1), ($allLines.Count - 1))
            
            $sb = New-Object System.Text.StringBuilder
            for ($j = $startLine; $j -le $endLine; $j++) {
                $null = $sb.Append($j + 1).Append(": ").AppendLine($allLines[$j])
            }
            $chunkContent = $sb.ToString()

            # --- FULL FIDELITY ENTERPRISE PROMPT ---
            $prompt = @"
You are an expert code cartographer. Analyze this chunk and extract its architecture.
Output RAW JSON ONLY. Do not use markdown blocks. Use the exact line numbers provided.

Schema:
{
  "file": "$($targetFile.Name)",
  "chunk": $($i+1),
  "summary": "Detailed summary of this specific chunk",
  "structs_or_classes": ["List", "of", "classes", "or", "structs"],
  "imports": ["List", "of", "imports"],
  "funcs": [
    {
      "name": "function_name",
      "line": 0,
      "calls_out_to": ["funcs", "it", "calls"],
      "is_stub_or_incomplete": false
    }
  ]
}

CODE CHUNK:
$chunkContent
"@
            
            $body = @{
                model = "qwen2.5-coder:1.5b" 
                prompt = $prompt
                stream = $false
                options = @{ num_ctx = 16384; temperature = 0; num_thread = $Threads }
            } | ConvertTo-Json -Depth 10 -Compress

            $chunkSuccess = $false
            $attempts = 0
            
            while (-not $chunkSuccess -and $attempts -lt 3) {
                $attempts++
                try {
                    $response = Invoke-RestMethod -Uri $apiUrl -Method Post -Body ([System.Text.Encoding]::UTF8.GetBytes($body)) -ContentType "application/json" -TimeoutSec 600 -ErrorAction Stop
                    
                    if ($null -eq $response -or [string]::IsNullOrWhiteSpace($response.response)) { throw "Empty response." }

                    $jsonOutput = $response.response.Trim()
                    
                    if ($jsonOutput.StartsWith('```json')) { $jsonOutput = $jsonOutput.Substring(7) }
                    elseif ($jsonOutput.StartsWith('```')) { $jsonOutput = $jsonOutput.Substring(3) }
                    if ($jsonOutput.EndsWith('```')) { $jsonOutput = $jsonOutput.Substring(0, $jsonOutput.Length - 3) }
                    $jsonOutput = $jsonOutput.Trim()

                    if ([string]::IsNullOrWhiteSpace($jsonOutput)) { throw "Stripped output empty." }

                    # STRICT VALIDATION
                    $null = ConvertFrom-Json $jsonOutput -ErrorAction Stop

                    $jsonOutput | Out-File -FilePath $outputJsonlFile -Append -Encoding utf8
                    Write-Host "   [+] Extracted high-fidelity metadata for chunk $($i+1)/$totalChunks." -ForegroundColor DarkGreen
                    $chunkSuccess = $true

                } catch {
                    Write-Host "   [!] Chunk $($i+1) fail (Attempt $attempts): $($_.Exception.Message)" -ForegroundColor DarkGray
                }
            }

            if (-not $chunkSuccess) {
                Write-Host "   [X] FATAL: Failed to parse Chunk $($i+1). Aborting file." -ForegroundColor Red
                $fileCompletelySuccessful = $false
                break
            }
        }

        # 3. ROUTE COMPLETED FILE
        if ($fileCompletelySuccessful) {
            Move-Item -LiteralPath $processingPath -Destination $DONE -Force
            Write-Host "   [SUCCESS] JSON saved to: $outputJsonlFile" -ForegroundColor Cyan
        } else {
            Move-Item -LiteralPath $processingPath -Destination $ERRS -Force
            if (Test-Path -LiteralPath $outputJsonlFile) { Remove-Item -LiteralPath $outputJsonlFile -Force }
        }

    } catch {
        Write-Host "   [X] Unexpected error: $($_.Exception.Message)" -ForegroundColor Red
        Move-Item -LiteralPath $processingPath -Destination $ERRS -Force -ErrorAction SilentlyContinue
    } finally {
        # ========================================================
        # THE ZERO-LEAK GUARANTEE
        # This ALWAYS fires, even if the file succeeds, fails, or crashes
        # ========================================================
        Kill-OllamaTree
    }
}