# Documentation sync check (Windows / PowerShell twin of docs_sync_check.sh).
# Run from repo root:  powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs_sync_check.ps1
# Optional: $env:DOC_SYNC_REQUIRE_DOC_UPDATES = "1" and $env:DOC_SYNC_BASE_REF = "<ref>"

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $RootDir

$script:Failed = 0

function Fail([string]$Message) {
    Write-Error "docs-sync-check: $Message" -ErrorAction Continue
    $script:Failed = 1
}

function Test-HeaderFields([string]$RelativePath) {
    $full = Join-Path $RootDir $RelativePath
    if (-not (Test-Path -LiteralPath $full)) {
        Fail "missing required file: $RelativePath"
        return
    }
    $head = (Get-Content -LiteralPath $full -TotalCount 12 -ErrorAction Stop) -join "`n"
    if ($head -notmatch '(?m)^Status:') {
        Fail "$RelativePath is missing a Status header near top-of-file"
    }
    if ($head -notmatch '(?m)^Last updated:') {
        Fail "$RelativePath is missing a Last updated header near top-of-file"
    }
}

function Test-NoMachineLocalPaths([string]$RelativePath) {
    $full = Join-Path $RootDir $RelativePath
    $content = Get-Content -LiteralPath $full -Raw -ErrorAction Stop
    if ($content -match '/Users/' -or $content -match '/home/[^/\s]+/' -or
        $content -match '[A-Za-z]:\\Users\\') {
        Fail "machine-local path found in $RelativePath"
    }
}

function Resolve-MarkdownLinkTarget {
    param(
        [string]$SourceFile,
        [string]$Target
    )
    $cleaned = $Target -replace '#.*$', '' -replace '\?.*$', ''
    $cleaned = $cleaned -replace ':(\d+(-\d+)?)$', ''
    if ([string]::IsNullOrWhiteSpace($cleaned)) { return $null }
    if ($cleaned -match '^(https?|mailto):') { return $null }
    if ($cleaned.StartsWith('#')) { return $null }

    if ($cleaned.StartsWith('/')) {
        return (Join-Path $RootDir $cleaned.TrimStart('/').Replace('/', [IO.Path]::DirectorySeparatorChar))
    }
    $dir = Split-Path -Parent (Resolve-Path -LiteralPath $SourceFile).Path
    return [IO.Path]::GetFullPath((Join-Path $dir $cleaned))
}

function Test-LinksInFile([string]$RelativePath) {
    $full = Join-Path $RootDir $RelativePath
    $content = Get-Content -LiteralPath $full -Raw -ErrorAction Stop
    $rx = [regex]'\[[^\]]+\]\(([^)]+)\)'
    foreach ($m in $rx.Matches($content)) {
        $target = $m.Groups[1].Value.Trim()
        $resolved = Resolve-MarkdownLinkTarget -SourceFile $full -Target $target
        if ($null -eq $resolved) { continue }
        if (-not (Test-Path -LiteralPath $resolved)) {
            Fail "broken markdown link in $RelativePath -> $target"
        }
    }
}

$HeaderFiles = @(
    "DOCUMENTATION.md",
    "docs/DOCUMENT_STATUS_INDEX.md",
    "docs/REPO_CONTEXT.md",
    "docs/CURRENT_STATE.md",
    "REMAINING_WORK_TRACKING.md",
    "docs/TESTING_GUIDE.md",
    "docs/MILESTONE_PLAN_V0.2.0_ALPHA.md",
    "docs/V0.2.0_RESIDUAL_RISK_REGISTER.md",
    "docs/EDGE_CASE_READINESS_MATRIX.md",
    "docs/ARCHITECTURE.md",
    "SECURITY.md",
    "SUPPORT.md",
    ".github/copilot-instructions.md",
    "docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md"
)

$LinkCheckFiles = @(
    "README.md",
    "DOCUMENTATION.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "SUPPORT.md",
    "docs/DOCUMENT_STATUS_INDEX.md",
    "docs/REPO_CONTEXT.md",
    "docs/CURRENT_STATE.md",
    "docs/TESTING_GUIDE.md",
    "docs/ARCHITECTURE.md",
    "docs/V0.2.0_RESIDUAL_RISK_REGISTER.md",
    ".github/copilot-instructions.md",
    "docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md"
)

foreach ($f in $HeaderFiles) {
    Test-HeaderFields $f
    Test-NoMachineLocalPaths $f
}

foreach ($f in $LinkCheckFiles) {
    Test-LinksInFile $f
}

if ($env:DOC_SYNC_REQUIRE_DOC_UPDATES -eq "1") {
    $baseRef = $env:DOC_SYNC_BASE_REF
    if ([string]::IsNullOrWhiteSpace($baseRef)) {
        Fail "DOC_SYNC_REQUIRE_DOC_UPDATES=1 requires DOC_SYNC_BASE_REF"
    }
    else {
        $codePaths = @("core", "android", "iOS", "wasm", "mobile", "cli", "ui", "scripts")
        $docPaths = @(
            "README.md", "DOCUMENTATION.md", "CONTRIBUTING.md", "SECURITY.md", "SUPPORT.md",
            "REMAINING_WORK_TRACKING.md", "docs", ".github/CODEOWNERS", ".github/ISSUE_TEMPLATE",
            ".github/pull_request_template.md", ".github/dependabot.yml", ".github/copilot-instructions.md"
        )
        $codeDiff = git diff --name-only "${baseRef}...HEAD" -- @codePaths 2>$null
        $docDiff = git diff --name-only "${baseRef}...HEAD" -- @docPaths 2>$null
        if ($LASTEXITCODE -ne 0) {
            Fail "git diff failed (is DOC_SYNC_BASE_REF valid?)"
        }
        elseif (($codeDiff | Where-Object { $_ }) -and -not ($docDiff | Where-Object { $_ })) {
            Fail "code changed since $baseRef but no docs changed in canonical docs surface"
        }
    }
}

if ($script:Failed -ne 0) {
    Write-Error "docs-sync-check: FAIL" -ErrorAction Continue
    exit 1
}

Write-Output "docs-sync-check: PASS"
exit 0
