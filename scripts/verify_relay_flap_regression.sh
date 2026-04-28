#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <ios_diagnostics_log> [dial_attempt_threshold]" >&2
  exit 2
fi

IOS_LOG="$1"
DIAL_THRESHOLD="${2:-12}"

if [[ ! -f "$IOS_LOG" ]]; then
  echo "Error: file not found: $IOS_LOG" >&2
  exit 2
fi

count_matches() {
  local pattern="$1"
  local count
  count="$(rg -n "$pattern" "$IOS_LOG" 2>/dev/null | wc -l | tr -d ' ' || true)"
  if [[ -z "$count" ]]; then
    count=0
  fi
  echo "$count"
}

relay_identified_count="$(count_matches 'peer_identified.*agent=.*relay')"
relay_dial_attempt_count="$(count_matches 'relay_state .*event=dial_attempt|relay_dial_debounced')"
relay_dial_started_count="$(count_matches 'relay_state .*event=dial_started')"
relay_dial_failed_count="$(count_matches 'relay_state .*event=dial_failed')"
relay_flapping_state_count="$(count_matches 'relay_state .*state=flapping')"

relay_identified_count=${relay_identified_count:-0}
relay_dial_attempt_count=${relay_dial_attempt_count:-0}
relay_dial_started_count=${relay_dial_started_count:-0}
relay_dial_failed_count=${relay_dial_failed_count:-0}
relay_flapping_state_count=${relay_flapping_state_count:-0}

echo "relay_identified_count=$relay_identified_count"
echo "relay_dial_attempt_count=$relay_dial_attempt_count"
echo "relay_dial_started_count=$relay_dial_started_count"
echo "relay_dial_failed_count=$relay_dial_failed_count"
echo "relay_flapping_state_count=$relay_flapping_state_count"

if (( relay_dial_attempt_count >= DIAL_THRESHOLD )) && (( relay_dial_started_count == 0 )); then
  echo "FAIL: dial-attempt loop detected without any dial_started transitions" >&2
  exit 1
fi

if (( relay_dial_attempt_count >= DIAL_THRESHOLD )) && (( relay_identified_count == 0 )); then
  echo "FAIL: dial-attempt pressure detected without relay peer identification" >&2
  exit 1
fi

echo "PASS: no deterministic relay dial-loop regression detected for this artifact"
