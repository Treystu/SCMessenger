# generate_flow_map.ps1
# Safe In-Memory Version: Avoids Dictionary Collection Limits & Ignores Test Suites

$TodoDir = "HANDOFF\todo"
$SearchExtensions = "*.rs", "*.kt"
$ExcludeDirs = ".git", "target", "build", "node_modules", ".claude", "tmp", "HANDOFF"

Write-Host "🧠 Initiating Deep-Scan for Unwired Functions..." -ForegroundColor Cyan

# Ensure task directory exists
$null = New-Item -ItemType Directory -Force -Path $TodoDir

# 1. Get all valid files (IGNORING TEST DIRECTORIES)
$Files = Get-ChildItem -Path . -Include $SearchExtensions -Recurse | Where-Object {
    $path = $_.FullName
    -not ($ExcludeDirs | Where-Object { $path -match "\\$_\\" }) -and
    $path -notmatch "\\tests?\\" -and
    $path -notmatch "\\androidTest\\" -and
    $path -notmatch "test[s]?\.rs$" -and
    $path -notmatch "Test[s]?\.kt$"
}

Write-Host "   -> Found $($Files.Count) production files. Loading repository into RAM..." -ForegroundColor Gray

# 2. Load all text into ONE massive string for instant searching
$AllText = ""
foreach ($File in $Files) {
    $AllText += (Get-Content -Path $File.FullName -Raw) + "`n"
}

# 3. Extract Definitions (IGNORING TEST FUNCTIONS)
$Functions = @()
$RustRegex = "^\s*(pub\s+)?(async\s+)?fn\s+([a-zA-Z0-9_]+)"
$KotlinRegex = "^\s*(private\s+|public\s+|suspend\s+|override\s+)*fun\s+([a-zA-Z0-9_]+)"

$Lines = $AllText -split "`r?`n"
foreach ($Line in $Lines) {
    $FuncName = $null
    if ($Line -match $RustRegex) { $FuncName = $matches[3] }
    elseif ($Line -match $KotlinRegex) { $FuncName = $matches[2] }

    if (![string]::IsNullOrWhiteSpace($FuncName)) {
        $FuncName = $FuncName.Trim()
        
        # Filter out common false positives and test suites
        if ($FuncName -ne "main" -and $FuncName -ne "onCreate" -and $FuncName -notmatch "^test_" -and $FuncName -notmatch "^assert_") {
            $Functions += $FuncName
        }
    }
}

$UniqueFunctions = $Functions | Select-Object -Unique
Write-Host "   -> Extracted $($UniqueFunctions.Count) unique functions." -ForegroundColor Yellow
Write-Host "   -> Cross-referencing occurrences against codebase... please wait a few seconds..." -ForegroundColor Gray

# 4. Safely Search the Massive Text Block
$DeadEnds = 0
foreach ($FuncName in $UniqueFunctions) {
    # Escape the function name just in case, and use word boundaries
    $Pattern = "\b" + [regex]::Escape($FuncName) + "\b"
    $UsageCount = [regex]::Matches($AllText, $Pattern).Count

    # If it only appears once (its own definition), it is a DEAD END.
    if ($UsageCount -le 1) {
        $DeadEnds++
        
        # Find exactly which file it lives in so we can write the task ticket
        $TargetFile = $Files | Where-Object { (Get-Content $_.FullName -Raw) -match "fun $FuncName|fn $FuncName" } | Select-Object -First 1
        
        # Failsafe if regex matching gets weird on a specific line
        if ($null -eq $TargetFile) { continue }
        
        $RelPath = $TargetFile.FullName.Replace("$PWD\", "")
        
        # Generate the micro-task file (using single quotes so PowerShell doesn't escape variables)
        $TaskFileName = "$TodoDir\task_wire_$FuncName.md"
        $TaskContent = @"
TARGET: $RelPath

SYSTEM DIRECTIVE: COMPREHENSIVE DEAD-END RESOLUTION
The function '$FuncName' is defined in '$RelPath' but is NEVER called anywhere in the repository. Your job is to fully integrate it into the overarching application loop. It may need to be wired into 1, 2, or 3+ different locations.

You MUST follow this strict analytical loop. Do not skip phases.

PHASE 1: CONTEXT GATHERING (Search & Ponder)
1. Open '$RelPath' and read the implementation of '$FuncName'. Understand its parameters, return type, and exact purpose.
2. Use your terminal search tools (grep, cat, ls) to hunt for related concepts, APIs, UI buttons, or parent managers where similar functions are called.
3. Identify EVERY valid location in the codebase where '$FuncName' MUST be invoked to work fully. 

PHASE 2: THE INTEGRATION PLAN
Write a concise list of exactly which files you are going to modify and where the function will be injected. 

PHASE 3: EXECUTION
Wire the function into ALL identified locations. Ensure you add the proper imports to the top of those files.

PHASE 4: TEST & ITERATE
1. Run a localized compiler check (cargo check for Rust, or .\gradlew lint for Kotlin).
2. Read the terminal output. 
3. IF COMPILE FAILS: Enter ITERATION. Read the exact error, fix the syntax or imports, and run the test again. 
4. IF SUCCESSFUL: Verify you successfully wired all targets from Phase 2. If the integration is 100% complete and compiles cleanly, output exactly:
STATUS: SUCCESS_STOP
"@
        Set-Content -Path $TaskFileName -Value $TaskContent
    }
}

Write-Host "✅ Scan Complete! Found $DeadEnds legitimately unwired functions. Tasks added to HANDOFF\todo\." -ForegroundColor Green