param(
    [Parameter(Mandatory=$true)]
    [string]$Files,
    [string]$Model = "qwen2.5-coder:3b",
    [int]$Threads = 4,
    [int]$Port = 11470
)

$ErrorActionPreference = "Stop"
try { [Console]::TreatControlCAsInput = $false } catch {}

$WorkspaceRoot = (Get-Item $PSScriptRoot).Parent.Parent.FullName
$HandoffAudit = Join-Path $WorkspaceRoot "HANDOFF_AUDIT"
$OutputDir = Join-Path $HandoffAudit "output"
$DoneDir = Join-Path $HandoffAudit "done"
$ErrorsDir = Join-Path $HandoffAudit "errors"

foreach ($dir in @($HandoffAudit, $OutputDir, $DoneDir, $ErrorsDir)) {
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
}

$TargetFiles = $Files.Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" }
Write-Host "[TARGETED] Re-index starting for $($TargetFiles.Count) files..." -ForegroundColor Cyan

function Get-CodeSkeleton {
    param([string]$FilePath)
    $Content = Get-Content -Path $FilePath -Raw -ErrorAction SilentlyContinue
    if (-not $Content) { return @{ classes = @(); funcs = @() } }
    $Extension = [System.IO.Path]::GetExtension($FilePath)
    
    $Skeleton = @{ classes = @(); funcs = @() }
    $Lines = $Content -split "`r?`n"
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        $Line = $Lines[$i]
        $LineNum = $i + 1
        if ($Extension -match "\.rs|\.kt|\.swift|\.java|\.cpp|\.c|\.h|\.ts|\.js") {
            if ($Line -match "^\s*(pub\s+|private\s+|internal\s+)?(class|struct|interface|enum|object|actor)\s+([a-zA-Z0-9_]+)") {
                $Skeleton.classes += [PSCustomObject]@{ name = $matches[3]; line = $LineNum }
            }
            if ($Line -match "^\s*(pub\s+|private\s+|protected\s+|internal\s+|async\s+|suspend\s+|override\s+)*\b(fn|fun|func|void|int|bool|string|Task)\b\s+([a-zA-Z0-9_]+)\s*\(") {
                $Skeleton.funcs += [PSCustomObject]@{ name = $matches[3]; line = $LineNum }
            }
        }
    }
    return $Skeleton
}

function Run-SingleWorker {
    param($File, $ChunkLines, $ChunkIndex, $TotalChunks, $Model, $NumCtx, $Threads, $Port, $OutDir, $AttemptReason, $Skeleton, $StartLine, $EndLine)
    $outName = "$($File.Name)_chunk$($ChunkIndex).jsonl"
    $outPath = Join-Path $OutDir $outName
    $tmpPath = $outPath + ".tmp"
    
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
            if ($retryError) { $prompt = "CRITICAL JSON ERROR PREVIOUSLY: $retryError`n`nFIX THIS AND RE-OUTPUT PERFECT JSON.`n`n" + $basePrompt }
            $body = @{ model = $Model; prompt = $prompt; stream = $false; options = @{ num_ctx = $NumCtx; temperature = 0.1; num_thread = $Threads } } | ConvertTo-Json -Depth 10 -Compress
            
            try {
                $response = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/api/generate" -Method Post -Body ([System.Text.Encoding]::UTF8.GetBytes($body)) -ContentType "application/json" -TimeoutSec 600
                $json = $response.response.Trim()
                if ($json.StartsWith('```json')) { $json = $json.Substring(7) }
                elseif ($json.StartsWith('```')) { $json = $json.Substring(3) }
                if ($json.EndsWith('```')) { $json = $json.Substring(0, $json.Length - 3) }
                $json = $json.Trim()
                
                $promptTokens = if ($null -ne $response.prompt_eval_count) { $response.prompt_eval_count } else { 0 }
                $evalTokens = if ($null -ne $response.eval_count) { $response.eval_count } else { 0 }
                
                $tokenLogPath = Join-Path (Get-Item $OutDir).Parent.FullName "token_usage.log"
                "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') | FILE: $($File.Name) CHUNK: $ChunkIndex | PROMPT: $promptTokens | EVAL: $evalTokens" | Out-File -FilePath $tokenLogPath -Append -Encoding utf8 -ErrorAction SilentlyContinue

                $startIndex = $json.IndexOf('{')
                $endIndex = $json.LastIndexOf('}')
                if ($startIndex -ge 0 -and $endIndex -ge $startIndex) {
                    $json = $json.Substring($startIndex, $endIndex - $startIndex + 1)
                }

                $parsed = ConvertFrom-Json $json -ErrorAction Stop
                if ($null -eq $parsed.file -or $null -eq $parsed.funcs -or $null -eq $parsed.imports -or $null -eq $parsed.structs_or_classes) { throw "Missing required arrays" }
                if ($parsed.summary -match "Detailed summary" -or $parsed.summary -eq "REPLACE_WITH_SUMMARY") { throw "Placeholder summary detected." }
                if ($parsed.structs_or_classes -contains "List" -or $parsed.imports -contains "List") { throw "Placeholder classes/imports." }
                if ($parsed.funcs.Count -gt 0) {
                    foreach ($f in $parsed.funcs) {
                        if ($null -eq $f.name -or $f.name -eq "REPLACE_WITH_NAME" -or $f.name -eq "function_name") { throw "Placeholder func" }
                        if ($null -eq $f.line -or $f.line -eq 0) { throw "Invalid line number" }
                        if ($f.calls_out_to -contains "funcs") { throw "Placeholder calls" }
                    }
                }
                if ($null -eq $parsed.summary -or $parsed.summary.Length -lt 25) {
                    $parsed | Add-Member -NotePropertyName "_REVIEW_REQUIRED" -NotePropertyValue $true
                    $json = $parsed | ConvertTo-Json -Depth 10 -Compress
                }
                $json | Out-File -FilePath $tmpPath -Encoding utf8
                Rename-Item -Path $tmpPath -NewName $outName -Force
                return "SUCCESS"
            } catch {
                $retryError = $_.Exception.Message
                if ($retry -eq $maxRetries) { throw "JSON VALIDATION FAILED: $retryError" }
                Start-Sleep -Seconds 2
            }
        }
    } catch {
        return "FAILED: $($_.Exception.Message)"
    }
}

# Start Ollama locally for this targeted run
Write-Host "[TARGETED] Booting Ollama on port $Port..." -ForegroundColor DarkGray
$env:OLLAMA_HOST = "127.0.0.1:$Port"
$ollamaProc = Start-Process -FilePath "ollama" -ArgumentList "serve" -WindowStyle Hidden -PassThru
$ollamaPID = $ollamaProc.Id

try {
    foreach ($FileRel in $TargetFiles) {
        $FileAbs = Join-Path $WorkspaceRoot $FileRel
        if (-not (Test-Path $FileAbs)) { Write-Host "Not found: $FileAbs" -ForegroundColor Yellow; continue }
        
        $FileInfo = Get-Item $FileAbs
        $BaseName = $FileInfo.Name
        
        Get-ChildItem -Path $OutputDir -Filter "${BaseName}_chunk*.jsonl" -ErrorAction SilentlyContinue | Remove-Item -Force
        Get-ChildItem -Path $DoneDir -Filter "${BaseName}_chunk*.txt" -ErrorAction SilentlyContinue | Remove-Item -Force
        Get-ChildItem -Path $ErrorsDir -Filter "${BaseName}_chunk*.txt" -ErrorAction SilentlyContinue | Remove-Item -Force
        
        $Lines = Get-Content -LiteralPath $FileAbs -ErrorAction SilentlyContinue
        if ($null -eq $Lines -or $Lines.Count -eq 0) { continue }
        
        $ChunkSize = 700
        $TotalChunks = [math]::Ceiling($Lines.Count / $ChunkSize)
        $Skeleton = Get-CodeSkeleton -FilePath $FileAbs
        
        for ($i = 0; $i -lt $TotalChunks; $i++) {
            $ChunkIndex = $i + 1
            $StartLine = $i * $ChunkSize
            $EndLine = [math]::Min(($StartLine + $ChunkSize - 1), ($Lines.Count - 1))
            $ChunkLines = @()
            for ($j = $StartLine; $j -le $EndLine; $j++) { $ChunkLines += "$($j + 1): $($Lines[$j])" }
            
            $CharCount = ($ChunkLines -join "").Length
            $EstimatedTokens = [math]::Ceiling($CharCount / 3) + 500
            $NumCtx = if ($EstimatedTokens -lt 2048) { 2048 } elseif ($EstimatedTokens -lt 4096) { 4096 } elseif ($EstimatedTokens -lt 8192) { 8192 } else { 16384 }
            
            $ChunkStartNum = $StartLine + 1
            $ChunkEndNum = $EndLine + 1
            $ScopedSkeleton = @{
                classes = @($Skeleton.classes | Where-Object { $_.line -ge $ChunkStartNum -and $_.line -le $ChunkEndNum })
                funcs = @($Skeleton.funcs | Where-Object { $_.line -ge $ChunkStartNum -and $_.line -le $ChunkEndNum })
            }

            Write-Host "[TARGETED] Processing $BaseName chunk $ChunkIndex/$TotalChunks..."
            $result = Run-SingleWorker -File $FileInfo -ChunkLines $ChunkLines -ChunkIndex $ChunkIndex -TotalChunks $TotalChunks -Model $Model -NumCtx $NumCtx -Threads $Threads -Port $Port -OutDir $OutputDir -AttemptReason "" -Skeleton $ScopedSkeleton -StartLine $ChunkStartNum -EndLine $ChunkEndNum
            
            $TicketName = "${BaseName}_chunk${ChunkIndex}.txt"
            if ($result -eq "SUCCESS") {
                "FILE: $FileAbs`nCHUNK: $ChunkIndex" | Out-File -FilePath (Join-Path $DoneDir $TicketName) -Encoding utf8 -ErrorAction SilentlyContinue
                # Append to REPO_MAP
                $expectedOutFile = Join-Path $OutputDir "${BaseName}_chunk${ChunkIndex}.jsonl"
                if (Test-Path $expectedOutFile) {
                    Get-Content $expectedOutFile | Out-File -FilePath (Join-Path $HandoffAudit "REPO_MAP.jsonl") -Append -Encoding utf8 -ErrorAction SilentlyContinue
                }
            } else {
                Write-Host "[TARGETED] Failed chunk $ChunkIndex of ${BaseName}: $result" -ForegroundColor Red
                "FILE: $FileAbs`nCHUNK: $ChunkIndex`nERROR: $result" | Out-File -FilePath (Join-Path $ErrorsDir $TicketName) -Encoding utf8 -ErrorAction SilentlyContinue
            }
        }
    }
} finally {
    Write-Host "[TARGETED] Tearing down Ollama instance..." -ForegroundColor DarkGray
    try { Stop-Process -Id $ollamaPID -Force -ErrorAction SilentlyContinue } catch {}
    try { & taskkill.exe /F /T /PID $ollamaPID 2>&1 | Out-Null } catch {}
    
    # Update index
    Write-Host "[TARGETED] Updating index..." -ForegroundColor Cyan
    $PythonExec = "python"
    try { & $PythonExec .claude/scripts/build_repo_index.py --incremental --files $Files } catch {}
    
    Write-Host "[TARGETED] Complete." -ForegroundColor Green
}
