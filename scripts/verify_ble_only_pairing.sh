#!/usr/bin/env bash
set -euo pipefail

ANDROID_LOG="${1:-android_logcat_latest.txt}"
IOS_LOG="${2:-ios_diagnostics_latest.log}"
TIMEOUT_THRESHOLD="${TIMEOUT_THRESHOLD:-5}"
ZERO_ADV_THRESHOLD="${ZERO_ADV_THRESHOLD:-3}"

count_matches() {
  local pattern="$1"
  shift
  local out
  out="$(rg -c "$pattern" "$@" 2>/dev/null || true)"
  if [[ -z "$out" ]]; then
    echo 0
    return
  fi
  echo "$out" | awk -F: '{sum += $NF} END {print sum + 0}'
}

if [[ ! -f "$ANDROID_LOG" ]]; then
  echo "ERROR: Android log not found: $ANDROID_LOG" >&2
  exit 2
fi
if [[ ! -f "$IOS_LOG" ]]; then
  echo "ERROR: iOS log not found: $IOS_LOG" >&2
  exit 2
fi

ble_only_markers="$(count_matches "delivery_attempt .*medium=ble-only .*phase=mode .*outcome=enabled|delivery_attempt .*phase=ble_only .*outcome=blocked .*strict_ble_only_mode" "$ANDROID_LOG" "$IOS_LOG")"
core_attempts="$(count_matches "delivery_attempt .*medium=core .*phase=direct .*outcome=attempt" "$ANDROID_LOG" "$IOS_LOG")"
zero_adv_windows="$(count_matches "No BLE Fast/GATT advertisements found|NearbyMediums: No BLE" "$ANDROID_LOG")"
addr_mismatch_events="$(count_matches "Address type mismatch" "$ANDROID_LOG")"
invite_timeout_events="$(count_matches "multipeer_invite_timeout|Invite timeout|declined invitation|multipeer_invite_not_connected" "$IOS_LOG")"

tmp_strict_ids="$(mktemp)"
tmp_core_ids="$(mktemp)"
tmp_conflicts="$(mktemp)"
trap 'rm -f "$tmp_strict_ids" "$tmp_core_ids" "$tmp_conflicts"' EXIT

rg -o "delivery_attempt msg=[^ ]+ medium=ble-only phase=mode outcome=enabled" "$ANDROID_LOG" "$IOS_LOG" 2>/dev/null \
  | sed -E 's/.*delivery_attempt msg=([^ ]+) .*/\1/' \
  | grep -v '^unknown$' \
  | sort -u > "$tmp_strict_ids" || true

rg -o "delivery_attempt msg=[^ ]+ medium=core phase=direct outcome=attempt" "$ANDROID_LOG" "$IOS_LOG" 2>/dev/null \
  | sed -E 's/.*delivery_attempt msg=([^ ]+) .*/\1/' \
  | grep -v '^unknown$' \
  | sort -u > "$tmp_core_ids" || true

comm -12 "$tmp_strict_ids" "$tmp_core_ids" > "$tmp_conflicts" || true
strict_ids_count="$(wc -l < "$tmp_strict_ids" | tr -d " ")"
conflict_ids_count="$(wc -l < "$tmp_conflicts" | tr -d " ")"

echo "ble_only_pairing_verification:"
echo "  android_log: $ANDROID_LOG"
echo "  ios_log: $IOS_LOG"
echo "  strict_ble_only_markers: $ble_only_markers"
echo "  strict_message_ids: $strict_ids_count"
echo "  core_attempt_events: $core_attempts"
echo "  strict_vs_core_conflict_message_ids: $conflict_ids_count"
echo "  zero_advertisement_windows: $zero_adv_windows"
echo "  address_type_mismatch_events: $addr_mismatch_events"
echo "  invite_timeout_or_decline_events: $invite_timeout_events"

if [[ "$conflict_ids_count" -gt 0 ]]; then
  echo "FAIL: core direct attempts observed for message IDs that were sent in strict BLE-only mode" >&2
  echo "  conflicting_message_ids:"
  sed 's/^/    - /' "$tmp_conflicts"
  exit 1
fi
if [[ "$zero_adv_windows" -gt "$ZERO_ADV_THRESHOLD" ]]; then
  echo "FAIL: zero-advertisement windows exceeded threshold ($ZERO_ADV_THRESHOLD)" >&2
  exit 1
fi
if [[ "$invite_timeout_events" -gt "$TIMEOUT_THRESHOLD" ]]; then
  echo "FAIL: invite timeout/decline loops exceeded threshold ($TIMEOUT_THRESHOLD)" >&2
  exit 1
fi
