# run_tasks.ps1 v2 -- serialized batch dispatcher for HANDOFF/todo packets.
# Windows PowerShell 5.1 compatible (no &&, no ternary).
#
# v1 post-mortem (2026-07-17, commits 71d02d4d/e298e9bf reverted by 23960b35/
# 8da8cc90) -- the rules below are each a lesson paid for in a bad commit:
#   1. ONE task at a time. v1 ran 2 concurrent delegate_task --verify jobs =
#      2 concurrent cargo/gradle builds, violating the Windows single-build
#      rule (rlib lock corruption risk, CLAUDE.md / .claude/rules/build.md).
#   2. Correct per-platform verify commands. v1 ran gradlew from repo root
#      ("Task 'assembleDebug' not found") and xcodebuild on Windows (does not
#      exist). Android verifies MUST run in android\; iOS tasks are
#      BLOCKED-PLATFORM on Windows and are skipped, not failed.
#   3. NEVER auto-move tickets to done/ and NEVER git commit. v1 committed
#      unreviewed crypto/transport diffs and moved 14 unfinished tickets to
#      done/. Workers implement; the ORCHESTRATOR verifies, reviews
#      (adversarial gate for core/src/{crypto,transport,routing,privacy}),
#      moves tickets, and commits. This script only writes a report.
#   4. Honor delegate_task exit codes: 0=verified, 2=verify failed after
#      rounds, 3=vacuous success (model returned no applicable blocks --
#      counts as FAILED, not done).
#   5. compile-only verify is NOT completion. Even exit 0 needs an
#      orchestrator quality pass: v1's "passing" C-06 diff was 212 lines of
#      simulated/mock dead code. Grep applied diffs for
#      simulate|mock|placeholder|"in a real implementation" before accepting.
#   6. Route models through lake_route.py (quota ledger + rotation), do not
#      hardcode one model; record every outcome with --record.
#
# Usage:
#   .\run_tasks.ps1                       # runs the default task list below
#   .\run_tasks.ps1 -Tasks D-02,T-02     # explicit subset

param(
    [string[]]$Tasks = @('D-02', 'D-04', 'D-05', 'C-05', 'C-06', 'T-02', 'T-03'),
    [string]$Tier = 'CODER'
)

$repoRoot = (Get-Location).Path
$report = Join-Path $repoRoot 'tmp\swarm_report.md'
"# Swarm batch report $(Get-Date -Format s)" | Out-File $report -Encoding utf8

foreach ($t in $Tasks) {
    $file = Get-ChildItem 'HANDOFF/todo' | Where-Object { $_.Name -match ('^' + [regex]::Escape($t)) } | Select-Object -First 1
    if (-not $file) {
        "SKIP ${t}: no ticket file" | Tee-Object -FilePath $report -Append
        continue
    }

    # Parse ## Target Files block
    $content = Get-Content $file.FullName
    $in_tf = $false
    $target_files = @()
    foreach ($line in $content) {
        if ($line -match '^## Target Files') { $in_tf = $true; continue }
        if ($in_tf -and $line -match '^- `?([^`\s]+)`?') { $target_files += $matches[1] }
        elseif ($in_tf -and $line -match '^##') { break }
    }
    if ($target_files.Count -eq 0) {
        "SKIP ${t}: no Target Files in packet (re-run scripts/fix_targets.py)" | Tee-Object -FilePath $report -Append
        continue
    }

    # Rule 2: platform-correct verify command; iOS is not buildable on Windows.
    $joined = $target_files -join ' '
    if ($joined -match '\.swift|\.pbxproj') {
        "BLOCKED-PLATFORM ${t}: iOS target files cannot be verified on Windows (needs macOS runner / H-01)" | Tee-Object -FilePath $report -Append
        continue
    }
    $verify = 'cargo check --workspace'
    if ($joined -match '\.kt|\.gradle') {
        # gradlew lives in android\, not repo root.
        $verify = 'cmd /c "cd android && gradlew.bat assembleDebug -x lint --quiet"'
    }

    # Rule 6: route provider/model through the quota-aware router.
    $routeOut = python scripts/lake_route.py --tier $Tier 2>$null
    if (-not $routeOut) {
        "FAIL ${t}: lake_route returned no available lake for tier $Tier" | Tee-Object -FilePath $report -Append
        continue
    }
    $parts = $routeOut.Trim() -split '\s+', 2
    $provider = $parts[0]; $model = $parts[1]

    $files_args = @()
    foreach ($tf in $target_files) { $files_args += $tf }

    Write-Host "[$t] dispatching to $provider/$model (verify: $verify)"
    $log = Join-Path $repoRoot "tmp\$t.log"

    # Rule 1: strictly sequential -- this call runs the build inline via --verify.
    & python scripts/delegate_task.py --task $file.FullName --provider $provider --model $model `
        --mode diff --apply --verify $verify --files @files_args *>&1 | Out-File $log -Encoding utf8
    $code = $LASTEXITCODE

    # Rule 4: exit-code truth table. Rule 3: report only, no moves, no commits.
    switch ($code) {
        0 { $status = 'VERIFIED (needs orchestrator quality pass + review gate before commit)' }
        2 { $status = 'VERIFY-FAILED after fix rounds' }
        3 { $status = 'VACUOUS (no applicable blocks -- treat as failed)' }
        default { $status = "ERROR exit=$code" }
    }
    "$t -> $provider/$model : $status (log: tmp\$t.log)" | Tee-Object -FilePath $report -Append

    $result = 'error'
    if ($code -eq 0) { $result = 'ok' }
    elseif ($code -eq 3) { $result = 'vacuous' }
    & python scripts/lake_route.py --record --lake $provider --model $model --task $t --result $result | Out-Null
}

Write-Host "Batch complete. Report: $report"
Write-Host "NEXT (orchestrator, not this script): review diffs (grep for simulate/mock/placeholder),"
Write-Host "run adversarial review on any crypto/transport/routing/privacy diff, then move tickets + commit."
