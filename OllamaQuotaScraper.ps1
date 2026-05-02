# UltimateQuotaScraper.ps1
# Uses native Windows curl.exe to bypass .NET TLS fingerprinting

$cookie = 'aid=1eca9b49-d336-4265-b530-adf59eda5cb2; __Secure-session=YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0-IFgyNTUxOSB1Y1ZtV29wL0FKMXVYcEdBZ0lRWHFYdkIxM2dWV0o5OVZaTy9tUmpDSWd3Ck1IZHQzaDYzejJiNXNSOWJsYk5IbkVBeFBHL2dXRzR3cjRWbFpwZzdJeW8KLS0tIDNJaStyM0ZiamdNb0h6Z2tRa2VKakhUY0I0bndFbG1WKzFXNlhhbndERVEKvpGZBU-2FNYweIXfJQASlTF89_MpyXhuNrvdxW1WzY3iYg4fqObliNVBdKDp_PHdRYPGfPP4Vbux665sIS6RVD4Q1ehvqvktPzd2jT-NVFGEorfEFwXP85Wr4a-ddvdEqv1gVQc8wuDoFzoHhm2cSJ0JtuMjU21SF2Npdq3ZZWj1iEJEZbu6MRBtkQ==; __stripe_mid=6ca25dae-2b4c-4d5d-bbea-7793516929ca6bf1ee'

$outputFile = "C:\Users\kanal\Documents\Github\SCMessenger\.claude\API_QUOTA_STATE.md"
$debugFile = "C:\Users\kanal\Documents\Github\SCMessenger\debug_ollama.html"

Write-Host "Executing native curl.exe bypass..." -ForegroundColor Cyan

# We call the literal curl.exe binary, NOT the PowerShell alias
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
Out-File -FilePath $debugFile -InputObject $html -Encoding utf8

$sessionMatch = [regex]::Match($html, '(?is)Session usage.*?([\d\.]+)%')
$weeklyMatch = [regex]::Match($html, '(?is)Weekly usage.*?([\d\.]+)%')

if ($sessionMatch.Success -and $weeklyMatch.Success) {
    $sessionUsed = $sessionMatch.Groups[1].Value
    $weeklyUsed = $weeklyMatch.Groups[1].Value

    $stateText = @"
# 📊 OLLAMA CLOUD TELEMETRY DATA
*Last Updated: $(Get-Date)*

* **5-Hour Usage:** $sessionUsed%
* **7-Day Usage:** $weeklyUsed%

**SYSTEM STATUS:** NORMAL
"@
    Out-File -FilePath $outputFile -InputObject $stateText -Encoding utf8
    Write-Host "[SUCCESS] Stealth scrape completed! Session: $sessionUsed% | Weekly: $weeklyUsed%" -ForegroundColor Green
} else {
    Write-Host "[FAILED] Cloudflare blocked curl.exe." -ForegroundColor Red
    if ($html -match "Sign In" -or $html -match "Log in") {
        Write-Host "Reason: Cookie rejected or Cloudflare Challenge Page triggered." -ForegroundColor Yellow
    }
}