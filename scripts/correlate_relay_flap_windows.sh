#!/usr/bin/env bash
set -euo pipefail

IOS_LOG="${1:-ios_diagnostics_latest.log}"
GCP_LOG="${2:-logs/5mesh/gcp.log}"

if [[ ! -f "$IOS_LOG" ]]; then
  echo "ERROR: iOS diagnostics log not found: $IOS_LOG" >&2
  exit 2
fi
if [[ ! -f "$GCP_LOG" ]]; then
  echo "ERROR: GCP relay log not found: $GCP_LOG" >&2
  exit 2
fi

tmp_gcp="$(mktemp)"
tmp_ios_min="$(mktemp)"
tmp_gcp_min="$(mktemp)"
trap 'rm -f "$tmp_gcp" "$tmp_ios_min" "$tmp_gcp_min"' EXIT

# Strip ANSI color sequences for deterministic parsing.
sed -E 's/\x1B\[[0-9;]*[[:alpha:]]//g' "$GCP_LOG" > "$tmp_gcp"

ios_event_count="$(rg -c "peer_identified .*relay|relay_state .*state=flapping|dial_attempt|dial_failure|dial_throttled" "$IOS_LOG" || true)"
gcp_disconnect_count="$(rg -c "Disconnected from|Lost relay peer|scheduling reconnect with backoff" "$tmp_gcp" || true)"
gcp_connect_count="$(rg -c "Connected to .*via" "$tmp_gcp" || true)"

awk '
  /peer_identified .*relay|relay_state .*state=flapping|dial_attempt|dial_failure|dial_throttled/ {
    if (length($1) >= 16) print substr($1, 1, 16)
  }
' "$IOS_LOG" | sort -u > "$tmp_ios_min"

awk '
  /Disconnected from|Lost relay peer|scheduling reconnect with backoff|Connected to .*via/ {
    if (length($1) >= 16) print substr($1, 1, 16)
  }
' "$tmp_gcp" | sort -u > "$tmp_gcp_min"

overlap_minutes="$(comm -12 "$tmp_ios_min" "$tmp_gcp_min" | wc -l | tr -d " ")"

classification="inconclusive"
if [[ "$ios_event_count" -gt 0 && "$gcp_disconnect_count" -gt 0 && "$overlap_minutes" -gt 0 ]]; then
  classification="mixed_client_and_remote_churn_likely"
elif [[ "$ios_event_count" -gt 0 && "$gcp_disconnect_count" -gt 0 && "$overlap_minutes" -eq 0 ]]; then
  classification="unsynchronized_artifacts_no_time_overlap"
elif [[ "$ios_event_count" -gt 0 && "$gcp_disconnect_count" -eq 0 ]]; then
  classification="client_dominant_churn"
elif [[ "$ios_event_count" -eq 0 && "$gcp_disconnect_count" -gt 0 ]]; then
  classification="remote_dominant_churn"
elif [[ "$ios_event_count" -eq 0 && "$gcp_disconnect_count" -eq 0 ]]; then
  classification="no_significant_relay_churn_detected"
fi

echo "relay_flap_correlation:"
echo "  ios_log: $IOS_LOG"
echo "  gcp_log: $GCP_LOG"
echo "  ios_relay_events: $ios_event_count"
echo "  gcp_connect_events: $gcp_connect_count"
echo "  gcp_disconnect_events: $gcp_disconnect_count"
echo "  overlapping_event_minutes: $overlap_minutes"
echo "  classification: $classification"
