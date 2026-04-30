# ==============================================================================
# SCMESSENGER LOCAL NARC - LIVE TRACKER EDITION (V17)
# Model: qwen2.5-coder:7b 
# Upgrades: Live Batch Tracking, 4K Fast-Cache (Massive Speedup)
# ==============================================================================

$ModelName = "qwen2.5-coder:7b"
$ReportFile = "docs\NARC_MASTER_TRACKER.md"
$ErrorLog = "docs\NARC_ERROR.log"
$StateFile = "docs\NARC_PROCESSED_FILES.txt"

$EnableVerbose = $true  

if (!(Test-Path "docs")) { New-Item -ItemType Directory -Path "docs" | Out-Null }

if (!(Test-Path $ReportFile)) {
    "## SCMessenger Master Debt Tracker" > $ReportFile
    "Generated on: $(Get-Date)`n" >> $ReportFile
    "| File | Line | Issue Type | Snippet | Status |" >> $ReportFile
    "|---|---|---|---|---|" >> $ReportFile
}

Write-Host "Booting up Terminator Narc Agent ($ModelName)..." -ForegroundColor Cyan

# ======================================================================
# LOAD SAVE STATE
# ======================================================================
$ProcessedFiles = @{}
if (Test-Path $StateFile) {
    Get-Content $StateFile | ForEach-Object { $ProcessedFiles[$_] = $true }
    Write-Host "-> [STATE] Loaded: $($ProcessedFiles.Count) files already audited." -ForegroundColor Green
}

$Files = Get-ChildItem -Path . -Include *.rs, *.kt, *.ts, *.js -Recurse | 
    Where-Object { 
        $_.FullName -notmatch "\\(target|build|\.git|tests|node_modules|tmp)\\?" -and 
        $_.Length -gt 0 
    }

$TotalFiles = $Files.Count
Write-Host "-> Found $TotalFiles target files in repository.`n" -ForegroundColor Yellow

$FileCounter = 1
$SkippedCount = 0

foreach ($File in $Files) {
    $RelPath = $File.FullName.Replace($PWD.Path + "\", "")
    
    if ($ProcessedFiles.ContainsKey($RelPath)) {
        $SkippedCount++
        $FileCounter++
        continue
    }

    $RawLines = @(Get-Content $File.FullName -Encoding UTF8)
    $TotalLines = $RawLines.Count
    
    if ($TotalLines -eq 0) { 
        $RelPath | Out-File -FilePath $StateFile -Append -Encoding utf8
        $FileCounter++
        continue 
    }

    Write-Host "[$FileCounter/$TotalFiles] [SCAN] $RelPath ($TotalLines lines)" -ForegroundColor White

    # ======================================================================
    # PHASE 1: DETERMINISTIC EXTRACTION
    # ======================================================================
    $SuspectLines = @()
    
    for ($i = 0; $i -lt $TotalLines; $i++) {
        $Line = $RawLines[$i].Trim()
        $LineNum = $i + 1
        
        if ($Line.Length -eq 0) { continue }

        if ($Line -match "^///" -or $Line -match "^//!" -or $Line -match "^\*\*") { continue }

        $IsSuspect = $false
        if ($Line -match "(?i)(todo|fixme|unimplemented|panic)") { $IsSuspect = $true }
        elseif ($Line -match "(?i)\b(mock|fake|stub)\b") { $IsSuspect = $true }
        elseif ($Line -match "\{\s*\}") { $IsSuspect = $true }
        elseif ($Line -match "^(//|/\*|#)") { $IsSuspect = $true }

        if ($IsSuspect) {
            $SuspectLines += "$LineNum | $Line"
        }
    }

    if ($SuspectLines.Count -eq 0) {
        Write-Host " -> [CLEAN] Instant Pass" -ForegroundColor DarkGreen
        $RelPath | Out-File -FilePath $StateFile -Append -Encoding utf8
        $FileCounter++
        continue
    }

    $TotalBatches = [math]::Ceiling($SuspectLines.Count / 150)
    Write-Host " -> Analyzing $($SuspectLines.Count) suspect lines across $TotalBatches batches..." -ForegroundColor DarkGray

    # ======================================================================
    # PHASE 2: LLM SEMANTIC EVALUATION (Micro-Prompt)
    # ======================================================================
    $BatchSize = 150
    $StartIndex = 0
    $FileFailed = $false
    $CurrentBatchNum = 1
    
    while ($StartIndex -lt $SuspectLines.Count) {
        $EndIndex = [math]::Min($StartIndex + $BatchSize, $SuspectLines.Count)
        $CurrentBatch = $SuspectLines[$StartIndex..($EndIndex - 1)] -join "`n"

        # The LIVE TRACKER
        Write-Host "   -> [BATCH $CurrentBatchNum/$TotalBatches] Sending lines $($StartIndex + 1)-$EndIndex to LLM... " -NoNewline -ForegroundColor Cyan

        $SystemPrompt = "SYSTEM DIRECTIVE: STRICT AUDITOR`n"
        $SystemPrompt += "You are an expert code reviewer. I have extracted suspicious lines from a source file.`n"
        $SystemPrompt += "Analyze ONLY these specific lines and identify technical debt.`n`n"
        $SystemPrompt += "CATEGORIES TO FIND:`n"
        $SystemPrompt += "1. TODO_LINES: Lines containing TODO, FIXME, unimplemented!(), or panic!()`n"
        $SystemPrompt += "2. MOCK_LINES: Lines explicitly defining hardcoded Fake, Mock, or Stub functions.`n"
        $SystemPrompt += "3. DISABLED_LINES: Lines where actual programming logic is commented out (e.g., '// let x = 5;').`n"
        $SystemPrompt += "4. EMPTY_LINES: Lines with empty function bodies {}.`n`n"
        $SystemPrompt += "OUTPUT EXACTLY IN THIS FORMAT:`n"
        $SystemPrompt += "TODO_LINES: 1, 2, 3 (Numbers only! Do not use brackets)`n"
        $SystemPrompt += "MOCK_LINES: 0`n"
        $SystemPrompt += "DISABLED_LINES: 14, 15 (Numbers only! Do not use brackets)`n"
        $SystemPrompt += "EMPTY_LINES: 0`n`n"
        $SystemPrompt += "EXTRACTED LINES TO ANALYZE:`n---`n$CurrentBatch`n---"

        $PayloadJSON = @{
            model = $ModelName
            prompt = $SystemPrompt
            stream = $false
            options = @{ 
                temperature = 0.0
                num_ctx = 4096 # MASSIVE SPEEDUP: Halved context to stop memory swapping
            }
        } | ConvertTo-Json -Depth 3

        $PayloadBytes = [System.Text.Encoding]::UTF8.GetBytes($PayloadJSON)

        try {
            $Response = Invoke-RestMethod -Uri "http://localhost:11434/api/generate" -Method Post -Body $PayloadBytes -ContentType "application/json; charset=utf-8" -ErrorAction Stop
            $Output = $Response.response.Trim() -replace "\[|\]", ""
            
            Write-Host "Done." -ForegroundColor Green

            if ($EnableVerbose) {
                Write-Host ($Output -replace "(?m)^", "      | ") -ForegroundColor DarkGray
            }

            function Verify-And-Log ($Type, $RegexLines) {
                $LinesMatch = [regex]::Match($Output, "$RegexLines`:\s*([\d,\s]+)")
                
                if ($LinesMatch.Success) {
                    $LineNumbers = $LinesMatch.Groups[1].Value -split "," | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "0" -and $_ -ne "" }
                    
                    foreach ($LineStr in $LineNumbers) {
                        $LineNum = [int]$LineStr
                        if ($LineNum -gt 0 -and $LineNum -le $TotalLines) {
                            $ActualCode = $RawLines[$LineNum - 1].Trim()
                            
                            $IsValid = $false
                            
                            if ($Type -eq "TODO" -and $ActualCode -match "(?i)(todo|fixme|unimplemented|panic)") { 
                                $IsValid = $true 
                            }
                            elseif ($Type -eq "MOCK" -and $ActualCode -match "(?i)(mock|fake|stub)") { 
                                $IsValid = $true 
                            }
                            elseif ($Type -eq "EMPTY_FUNC" -and $ActualCode -match "\{\s*\}") { 
                                $IsValid = $true 
                            }
                            elseif ($Type -eq "DISABLED_CODE" -and $ActualCode -match "^(//|/\*|#)") { 
                                if ($ActualCode -match "[=;{}\(\)\[\]]|let |fn |mut |if |return |match |=>|->") {
                                    $IsValid = $true
                                }
                            }

                            if ($IsValid) {
                                $CleanCode = $ActualCode -replace "\|", "" -replace "`n", ""
                                "| $RelPath | $LineNum | $Type | \`$CleanCode\` | UNRESOLVED |" | Out-File -FilePath $ReportFile -Append -Encoding utf8

                                if ($script:EnableVerbose) {
                                    Write-Host "      -> [LOGGED] Line $LineNum ($Type) - `"$CleanCode`"" -ForegroundColor Magenta
                                }
                            }
                        }
                    }
                }
            }

            Verify-And-Log "TODO" "TODO_LINES"
            Verify-And-Log "MOCK" "MOCK_LINES"
            Verify-And-Log "DISABLED_CODE" "DISABLED_LINES"
            Verify-And-Log "EMPTY_FUNC" "EMPTY_LINES"

        } catch {
            $ErrMsg = $_.Exception.Message
            Write-Host "FAILED!" -ForegroundColor Red
            Write-Host "      [ERROR] $ErrMsg" -ForegroundColor Red
            "$([datetime]::Now.ToString('s')) | FILE: $RelPath | ERROR: $ErrMsg" | Out-File -FilePath $ErrorLog -Append -Encoding utf8
            $FileFailed = $true
            break 
        }
        
        $StartIndex += $BatchSize
        $CurrentBatchNum++
    }

    if (-not $FileFailed) {
        $RelPath | Out-File -FilePath $StateFile -Append -Encoding utf8
    } else {
        Write-Host "   -> [WARN] FILE FAILED: Will retry on next run." -ForegroundColor DarkRed
    }

    $FileCounter++
}

Write-Host "`n[DONE] Master Audit Complete! (Skipped $SkippedCount previously audited files)" -ForegroundColor Cyan