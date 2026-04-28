#!/usr/bin/env bash
set -euo pipefail

ANDROID_LOG="${1:-android_mesh_diagnostics_device.log}"
IOS_LOG="${2:-ios_diagnostics_latest.log}"
MAX_IDS="${MAX_IDS:-60}"

if [[ ! -f "$ANDROID_LOG" ]]; then
  echo "ERROR: Android diagnostics log not found: $ANDROID_LOG" >&2
  exit 2
fi
if [[ ! -f "$IOS_LOG" ]]; then
  echo "ERROR: iOS diagnostics log not found: $IOS_LOG" >&2
  exit 2
fi

tmp_all="$(mktemp)"
tmp_ids="$(mktemp)"
trap 'rm -f "$tmp_all" "$tmp_ids"' EXIT

cat "$ANDROID_LOG" "$IOS_LOG" > "$tmp_all"

# Prefer newly prepared outbound IDs for deterministic run-scoped validation.
rg -o "delivery_state msg=[^ ]+ state=pending detail=message_prepared_local_history_written" "$tmp_all" \
  | sed -E 's/.*delivery_state msg=([^ ]+) .*/\1/' \
  | grep -v '^unknown$' \
  | tail -n "$MAX_IDS" \
  | sort -u > "$tmp_ids" || true

# Fallback for archives that do not include the pending marker format.
if [[ ! -s "$tmp_ids" ]]; then
  rg -o "delivery_attempt msg=[^ ]+" "$tmp_all" \
    | sed 's/delivery_attempt msg=//' \
    | grep -v '^unknown$' \
    | tail -n "$MAX_IDS" \
    | sort -u > "$tmp_ids" || true
fi

total_ids="$(wc -l < "$tmp_ids" | tr -d " ")"
if [[ "$total_ids" -eq 0 ]]; then
  echo "receipt_convergence: no message ids found in delivery_attempt markers"
  exit 1
fi

missing_rx=0
missing_delivered=0
failed_ids=0

while IFS= read -r msg_id; do
  [[ -z "$msg_id" ]] && continue
  has_rx=0
  has_delivered=0

  if rg -q "msg_rx_processed .*msg=${msg_id}|msg_rx .*msg=${msg_id}" "$tmp_all"; then
    has_rx=1
  fi
  if rg -q "delivery_state msg=${msg_id} state=delivered" "$tmp_all"; then
    has_delivered=1
  fi

  if [[ "$has_rx" -eq 0 || "$has_delivered" -eq 0 ]]; then
    failed_ids=$((failed_ids + 1))
    [[ "$has_rx" -eq 0 ]] && missing_rx=$((missing_rx + 1))
    [[ "$has_delivered" -eq 0 ]] && missing_delivered=$((missing_delivered + 1))
  fi
done < "$tmp_ids"

echo "receipt_convergence:"
echo "  android_log: $ANDROID_LOG"
echo "  ios_log: $IOS_LOG"
echo "  candidate_message_ids: $total_ids"
echo "  failed_message_ids: $failed_ids"
echo "  missing_recipient_ingest_markers: $missing_rx"
echo "  missing_sender_delivered_markers: $missing_delivered"

if [[ "$failed_ids" -gt 0 ]]; then
  exit 1
fi
