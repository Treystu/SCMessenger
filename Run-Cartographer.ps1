# Run-Cartographer.ps1 (V8 - The Sanitizer Edition)
# SCMessenger Autonomous Repo Cartographer

$repoRoot = "C:\Users\kanal\Documents\Github\SCMessenger"
$outputFile = "$repoRoot\HANDOFF\discovery\REPO_MAP.jsonl"
$ollamaEndpoint = "http://localhost:11434/api/generate"
$modelName = "qwen2.5-coder:7b"

# 1. Rules of Engagement
$allowedExtensions = @('.rs', '.kt', '.swift', '.ts', '.tsx', '.js', '.py')
$blockedDirs = @('.git', 'target', 'build', 'node_modules', 'ios/Pods', '.gradle', '.claude')
$chunkSize = 150 # Shrunk to 150 lines for lightning-fast parsing

Write-Host "[INIT] Booting SCMessenger Cartographer V8..." -ForegroundColor Cyan

if (-not (Test-Path (Split-Path $outputFile))) { New-Item -ItemType Directory -Path (Split-Path $outputFile) -Force | Out-Null }

# 2. Incremental Cache
$mappedFiles = @{}
if (Test-Path $outputFile) {
    Get-Content $outputFile | ForEach-Object {
        try {
            $entry = $_ | ConvertFrom-Json
            $mappedFiles[$entry.file_path] = [datetime]$entry.last_analyzed
        } catch {}
    }
}

$allFiles = Get-ChildItem -Path $repoRoot -Recurse -File | Where-Object {
    $ext = $_.Extension; $path = $_.FullName
    $isAllowed = $allowedExtensions -contains $ext
    $isNotBlocked = $true
    foreach ($blocked in $blockedDirs) { if ($path -match "[\\/]$blocked[\\/]") { $isNotBlocked = $false; break } }
    return $isAllowed -and $isNotBlocked
}

Write-Host "Found $($allFiles.Count) code files. Beginning Delta Extraction..." -ForegroundColor DarkGray

# 3. Extraction Loop
foreach ($file in $allFiles) {
    $relativePath = $file.FullName.Replace("$repoRoot\", "").Replace("\", "/")
    
    if ($mappedFiles.ContainsKey($relativePath) -and $file.LastWriteTimeUtc -lt $mappedFiles[$relativePath]) {
        continue 
    }

    Write-Host "Auditing: $relativePath" -ForegroundColor Yellow
    
    # Safely load the file enforcing UTF8 to prevent mojibake reading errors
    $lines = Get-Content -Path $file.FullName -Encoding UTF8 -Raw -ErrorAction SilentlyContinue
    if ([string]::IsNullOrWhiteSpace($lines)) { $lines = @("") } else { $lines = $lines -split "`n" }
    
    $totalChunks = [math]::Ceiling($lines.Count / $chunkSize)
    if ($totalChunks -eq 0) { $totalChunks = 1 } 
    
    $fileState = @{
        file_path = $relativePath
        last_analyzed = ""
        file_type = "Unknown"
        overall_purpose = ""
        imports_and_exports = @()
        structs_or_classes = @()
        functions = @()
    }

    for ($i = 0; $i -lt $totalChunks; $i++) {
        $start = $i * $chunkSize
        $end = [math]::Min(($start + $chunkSize - 1), ($lines.Count - 1))
        
        # THE SANITIZER: Strip out all invisible control characters (ANSI, Null bytes) that break JSON
        $rawChunk = $lines[$start..$end] -join "`n"
        $chunkText = $rawChunk -replace "[\x00-\x08\x0B-\x0C\x0E-\x1F\x7F]", ""
        
        $promptHeader = @"
You are an expert code auditor. Analyze this chunk of code (Lines $($start+1) to $($end+1)) from '$relativePath'.
Extract the requested data and output ONLY a raw JSON object. Do not include markdown blocks.

CRITICAL REQUIREMENT: Properly escape all double quotes (\") and backslashes (\\) inside your JSON string values.

SCHEMA REQUIREMENTS:
{
  "file_type_guess": "Choose ONE: Core Logic, UI Component, API Route, Database Model, Config, FFI Bridge, Interface/Trait, Utility Script, or Other",
  "chunk_purpose": "Briefly describe what this specific block of code does.",
  "imports_exports": ["List key dependencies imported or symbols exported"],
  "structs_classes": ["List names of structs, classes, or interfaces defined here"],
  "functions": [
    {
      "name": "Function Name",
      "line_approx": $($start+1),
      "calls_out_to": ["FunctionA", "ModuleB"],
      "is_stub_or_incomplete": true
    }
  ]
}
If a field does not apply, return an empty array [].

CODE CHUNK:
"@
        
        $promptStr = $promptHeader + "`n" + $chunkText

        $bodyObj = @{
            model = $modelName
            prompt = $promptStr
            stream = $false
            format = "json"
            options = @{
                num_ctx = 16384 
            }
        }
        
        $bodyJson = $bodyObj | ConvertTo-Json -Depth 10

        $retries = 0; $valid = $false
        while (-not $valid -and $retries -lt 3) {
            try {
                $response = Invoke-RestMethod -Uri $ollamaEndpoint -Method Post -Body $bodyJson -ContentType "application/json" -TimeoutSec 180
                $rawJson = $response.response
                
                $cleanJson = $rawJson -replace '(?s)^[^{]*', '' -replace '(?s)[^}]*$', ''
                $delta = $cleanJson | ConvertFrom-Json -ErrorAction Stop
                
                if ($null -ne $delta.file_type_guess -or $null -ne $delta.chunk_purpose) {
                    if ($fileState.file_type -eq "Unknown" -and $delta.file_type_guess) { $fileState.file_type = $delta.file_type_guess }
                    if ($delta.chunk_purpose) { $fileState.overall_purpose += " " + $delta.chunk_purpose }
                    if ($delta.imports_exports) { $fileState.imports_and_exports += $delta.imports_exports }
                    if ($delta.structs_classes) { $fileState.structs_or_classes += $delta.structs_classes }
                    if ($delta.functions) { $fileState.functions += $delta.functions }

                    $valid = $true
                    Write-Host "  -> Chunk $($i+1)/$totalChunks parsed." -ForegroundColor DarkGray
                } else {
                    throw "JSON missing core keys."
                }
            } catch {
                $retries++
                $psError = $_.Exception.Message
                Write-Host "  -> Retry ($retries/3) on chunk $($i+1) Failed." -ForegroundColor Red
                
                if ($psError -match "400") {
                    Write-Host "     [FATAL] HTTP 400 - Invisible bytes broke the payload." -ForegroundColor DarkRed
                } else {
                    Write-Host "     [JSON ERROR] The LLM hallucinated bad syntax." -ForegroundColor DarkRed
                }
            }
        }
        
        if (-not $valid) {
            Write-Host "  -> Chunk $($i+1) skipped after 3 failures." -ForegroundColor Yellow
            $fileState.overall_purpose += " [Warning: Chunk $($i+1) failed to parse]"
        }
    }

    $fileState.overall_purpose = $fileState.overall_purpose.Trim()
    $fileState.last_analyzed = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    
    $finalJsonLine = $fileState | ConvertTo-Json -Depth 10 -Compress
    Add-Content -Path $outputFile -Value $finalJsonLine -Encoding UTF8
    Write-Host "  [+] Mapped to REPO_MAP.jsonl" -ForegroundColor Green
}

Write-Host "[DONE] Cartography Complete! Map saved to $outputFile" -ForegroundColor Cyan