# OllamaQuotaScraper.ps1
# Uses native Windows curl.exe to bypass .NET TLS fingerprinting
param(
    [switch]$Quiet
)

$cookie = 'aid=bf5f45fb-b5ea-4b39-b61c-abacf9cc81bb; __Secure-session=YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0-IFgyNTUxOSA0US9BSEh5THJwNkVaa1VsZVJWTDR3UU9SRGZETVZqVFh5ZHRWUWl6b2g0CmV0dCs4aEFUT1BYWlNLQ0JqVmpoM05BeVRXQkd2WmY5enlBUXlwUWY5UXMKLS0tIHB6VTBTWnZvbTBybnp6ZitWaGNEQmIwblV3emxMSm9oR3NlbTBJTnloY0kKS1pTxMYSRROiBR0gy6DrFvbsPrd-Rmq5RLLRDLu_Z5HwZmAl2KDqJDLJtTL_8RyyTZoa_MfRR42Yz3GJoUoFhotn2vOyVUlwe9pFG-DoRF8V5cpu8EGZutiAasa1TnjdA1hXiNAK0cuXjVq1xZe6mUatp1wx2Q0uS5BJyyy0a5HGR1OZh-N8fkoH8A=='

$baseDir     = "E:\SCMessenger-Github-Repo\SCMessenger"
$jsonFile    = "$baseDir\.claude\quota_state.json"
$debugFile   = "$baseDir\tmp\debug_ollama.html"

if (-not $Quiet) {
    Write-Host "[INFO] Executing native curl.exe bypass..." -ForegroundColor Cyan
}

$html = curl.exe -s "https://ollama.com/settings" `
    -H "authority: ollama.com" `
    -H "accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7" `
    -H "accept-language: en-US,en;q=0.9" `
    -H "cookie: $cookie" `
    -H "sec-ch-ua: `"Chromium`";v=`"124`", `"Google Chrome`";v=`"124`", `"Not-A.Brand`";v=`"99`"" `
    -H "sec-ch-ua-mobile: ?0" `
    -H "sec-ch-ua-platform: `"Windows`"" `
    -H "sec-fetch-dest: document" `
    -H "sec-fetch-mode: navigate" `
    -H "sec-fetch-site: none" `
    -H "sec-fetch-user: ?1" `
    -H "upgrade-insecure-requests: 1" `
    -H "user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"

# Dump for debugging
if (-not (Test-Path "$baseDir\tmp")) { New-Item -ItemType Directory -Path "$baseDir\tmp" -Force | Out-Null }
Out-File -FilePath $debugFile -InputObject $html -Encoding utf8

# Detect Cloudflare / sign-in page
$isBlocked = $false
if ($html -match "Sign In" -or $html -match "Log in" -or $html -match "cf-browser-verify" -or $html -match "Just a moment") {
    $isBlocked = $true
}

$sessionMatch = [regex]::Match($html, '(?is)Session usage.*?([\d\.]+)%')
$weeklyMatch  = [regex]::Match($html, '(?is)Weekly usage.*?([\d\.]+)%')

# Parse reset time from HTML (e.g., "Resets in 22 minutes", "Resets in 1 hour")
$resetMinutes = $null
$resetMatch = [regex]::Match($html, '(?is)Resets?\s+in\s+(\d+)\s*(minute|hour|min)s?')
if ($resetMatch.Success) {
    $val = [int]$resetMatch.Groups[1].Value
    $unit = $resetMatch.Groups[2].Value
    if ($unit -eq "hour") {
        $resetMinutes = $val * 60
    } else {
        $resetMinutes = $val
    }
}

$timestamp = Get-Date -Format "o"
$success = $sessionMatch.Success -and $weeklyMatch.Success -and (-not $isBlocked)

if ($success) {
    $sessionUsed = $sessionMatch.Groups[1].Value
    $weeklyUsed  = $weeklyMatch.Groups[1].Value

    # Write structured JSON (single source of truth for all swarm components)
    $jsonData = @{
        fiveHour     = [double]$sessionUsed
        sevenDay     = [double]$weeklyUsed
        resetMinutes = if ($resetMinutes) { $resetMinutes } else { $null }
        timestamp    = $timestamp
        status       = "ok"
    }
    $jsonData | ConvertTo-Json | Out-File -FilePath $jsonFile -Encoding utf8

    if (-not $Quiet) {
        $resetNoteStr = if ($resetMinutes) { "resets in ~$resetMinutes min" } else { "reset time unknown" }
	        Write-Host "[SUCCESS] Session: $sessionUsed% | Weekly: $weeklyUsed% | Reset: $resetNoteStr" -ForegroundColor Green
    }
    exit 0
} else {
    # Write error state to JSON
    $errSession = if ($sessionMatch.Success) { [double]$sessionMatch.Groups[1].Value } else { $null }
    $errWeekly  = if ($weeklyMatch.Success)  { [double]$weeklyMatch.Groups[1].Value }  else { $null }
    $errReason  = if ($isBlocked) { "Cloudflare or sign-in page detected -- cookie may be expired" } else { "Could not parse usage from HTML" }

    $jsonData = @{
        fiveHour     = if ($errSession) { $errSession } else { $null }
        sevenDay     = if ($errWeekly)  { $errWeekly }  else { $null }
        resetMinutes = $null
        timestamp    = $timestamp
        status       = "error"
        error        = $errReason
    }
    $jsonData | ConvertTo-Json | Out-File -FilePath $jsonFile -Encoding utf8

    if (-not $Quiet) {
        Write-Host "[ERROR] Scrape failed: $errReason" -ForegroundColor Red
    }
    exit 1
}
