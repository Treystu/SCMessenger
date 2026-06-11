#!/bin/bash
# OllamaQuotaScraper.sh -- macOS port of OllamaQuotaScraper.ps1
# Scrapes ollama.com/settings using native curl (no PowerShell dependency).
# Writes to .claude/quota_state.json in the same format the .ps1 does.
#
# Usage:
#   bash OllamaQuotaScraper.sh              # verbose
#   bash OllamaQuotaScraper.sh --quiet      # quiet (writes JSON only)
#
# Cookie: hardcoded from the upstream .ps1 (Windows-side, last refreshed
# 2026-05 per git log "swarm: update OllamaQuotaScraper for new machine cookie
# and baseDir"). If Cloudflare starts blocking, the cookie needs to be rotated
# from a working browser session -- see HANDOFF/STATE/2026-06-05_QUOTA_LEDGER_REPAIR.md.
#
# Schema written to quota_state.json:
#   { "fiveHour": <float>, "sevenDay": <float>, "resetMinutes": <int|null>,
#     "timestamp": <ISO-8601 with TZ>, "status": "ok" | "error", "error"?: <string> }

set -euo pipefail

QUIET=0
for arg in "$@"; do
    case "$arg" in
        --quiet|-q|-Quiet) QUIET=1 ;;
    esac
done

log() { [ "$QUIET" -eq 1 ] || echo "$@"; }
err() { echo "$@" >&2; }

# Locate repo: the .ps1 lives at the repo root, and so do we.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR" && pwd)"  # ps1 is at repo root
JSON_FILE="$REPO_ROOT/.claude/quota_state.json"
DEBUG_HTML="$REPO_ROOT/tmp/debug_ollama.html"
mkdir -p "$(dirname "$DEBUG_HTML")"

# Cookie from OllamaQuotaScraper.ps1 (last updated 2026-05; rotate if Cloudflare blocks)
COOKIE='aid=bf5f45fb-b5ea-4b39-b61c-abacf9cc81bb; __Secure-session=YWdlLWVuY3J5cHRpb24ub3JnL3YxCi0-IFgyNTUxOSA0US9BSEh5THJwNkVaa1VsZVJWTDR3UU9SRGZETVZqVFh5ZHRWUWl6b2g0CmV0dCs4aEFUT1BYWlNLQ0JqVmpoM05BeVRXQkd2WmY5enlBUXlwUWY5UXMKLS0tIHB6VTBTWnZvbTBybnp6ZitWaGNEQmIwblV3emxMSm9oR3NlbTBJTnloY0kKS1pTxMYSRROiBR0gy6DrFvbsPrd-Rmq5RLLRDLu_Z5HwZmAl2KDqJDLJtTL_8RyyTZoa_MfRR42Yz3GJoUoFhotn2vOyVUlwe9pFG-DoRF8V5cpu8EGZutiAasa1TnjdA1hXiNAK0cuXjVq1xZe6mUatp1wx2Q0uS5BJyyy0a5HGR1OZh-N8fkoH8A=='

log "[INFO] Executing native curl bypass (macOS)..."

HTTP_CODE=$(curl -s -o "$DEBUG_HTML" -w "%{http_code}" "https://ollama.com/settings" \
    -H "authority: ollama.com" \
    -H "accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7" \
    -H "accept-language: en-US,en;q=0.9" \
    -H "cookie: $COOKIE" \
    -H 'sec-ch-ua: "Chromium";v="124", "Google Chrome";v="124", "Not-A.Brand";v="99"' \
    -H "sec-ch-ua-mobile: ?0" \
    -H 'sec-ch-ua-platform: "Windows"' \
    -H "sec-fetch-dest: document" \
    -H "sec-fetch-mode: navigate" \
    -H "sec-fetch-site: none" \
    -H "sec-fetch-user: ?1" \
    -H "upgrade-insecure-requests: 1" \
    -H "user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36" \
) || HTTP_CODE="000"

if [ "$HTTP_CODE" != "200" ]; then
    err "[ERROR] HTTP $HTTP_CODE from ollama.com/settings"
    # Write error state
    TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%S.%6N%z)
    cat > "$JSON_FILE" <<EOF
{
    "fiveHour": null,
    "sevenDay": null,
    "resetMinutes": null,
    "timestamp": "$TIMESTAMP",
    "status": "error",
    "error": "HTTP $HTTP_CODE from ollama.com/settings"
}
EOF
    exit 1
fi

# Detect Cloudflare / sign-in block
IS_BLOCKED=0
if grep -qE "Sign In|Log in|cf-browser-verify|Just a moment" "$DEBUG_HTML" 2>/dev/null; then
    IS_BLOCKED=1
fi

# Parse usage values. .NET regex (?is)Session usage.*?([\d.]+)% translated to Perl-compatible.
# Use -P (PCRE) on macOS BSD grep is unavailable; use python instead (always present via venv or system).
parse_field() {
    # $1 = pattern (Perl regex), $2 = input file
    python3 -c "
import re, sys
html = open(sys.argv[1]).read()
m = re.search(sys.argv[2], html, re.IGNORECASE | re.DOTALL)
print(m.group(1) if m else '')
" "$2" "$1" 2>/dev/null
}

SESSION_USED=$(parse_field 'Session usage.*?([\d.]+)%' "$DEBUG_HTML")
WEEKLY_USED=$(parse_field 'Weekly usage.*?([\d.]+)%'   "$DEBUG_HTML")
RESET_RAW=$(parse_field 'Resets?\s+in\s+(\d+)\s*(minute|hour|min)s?' "$DEBUG_HTML")

# Compute reset minutes
RESET_MINUTES=""
if [ -n "$RESET_RAW" ]; then
    # RESET_RAW contains two capture groups concatenated; re-parse
    RESET_PARTS=$(python3 -c "
import re
m = re.search(r'Resets?\s+in\s+(\d+)\s*(minute|hour|min)s?', open('$DEBUG_HTML').read(), re.IGNORECASE | re.DOTALL)
if m:
    val, unit = int(m.group(1)), m.group(2).lower()
    print(val * 60 if unit == 'hour' else val)
else:
    print('')
")
    if [ -n "$RESET_PARTS" ]; then
        RESET_MINUTES="$RESET_PARTS"
    fi
fi

# Generate ISO-8601 timestamp with microseconds and TZ (Python for portability)
TIMESTAMP=$(python3 -c "import datetime; print(datetime.datetime.now(datetime.timezone.utc).astimezone().isoformat())")

# Build success/error JSON
if [ -n "$SESSION_USED" ] && [ -n "$WEEKLY_USED" ] && [ "$IS_BLOCKED" -eq 0 ]; then
    if [ -n "$RESET_MINUTES" ]; then
        cat > "$JSON_FILE" <<EOF
{
    "fiveHour": $SESSION_USED,
    "sevenDay": $WEEKLY_USED,
    "resetMinutes": $RESET_MINUTES,
    "timestamp": "$TIMESTAMP",
    "status": "ok"
}
EOF
    else
        cat > "$JSON_FILE" <<EOF
{
    "fiveHour": $SESSION_USED,
    "sevenDay": $WEEKLY_USED,
    "resetMinutes": null,
    "timestamp": "$TIMESTAMP",
    "status": "ok"
}
EOF
    fi
    # Refresh the cache timestamp file
    date +%s > "$REPO_ROOT/.claude/quota_cache_timestamp"
    log "[SUCCESS] Session: ${SESSION_USED}% | Weekly: ${WEEKLY_USED}% | Reset: ~${RESET_MINUTES:-?} min"
    exit 0
else
    REASON="Cloudflare or sign-in page detected -- cookie may be expired"
    [ "$IS_BLOCKED" -eq 0 ] && REASON="Could not parse usage from HTML"
    cat > "$JSON_FILE" <<EOF
{
    "fiveHour": $([ -n "$SESSION_USED" ] && echo "$SESSION_USED" || echo "null"),
    "sevenDay": $([ -n "$WEEKLY_USED" ] && echo "$WEEKLY_USED" || echo "null"),
    "resetMinutes": null,
    "timestamp": "$TIMESTAMP",
    "status": "error",
    "error": "$REASON"
}
EOF
    err "[ERROR] Scrape failed: $REASON"
    exit 1
fi
