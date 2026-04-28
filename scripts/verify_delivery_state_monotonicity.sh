#!/usr/bin/env bash
set -euo pipefail

ANDROID_LOG="${1:-android_mesh_diagnostics_device.log}"
IOS_LOG="${2:-ios_diagnostics_latest.log}"

if [ ! -f "$ANDROID_LOG" ]; then
  echo "ERROR: Android diagnostics log not found: $ANDROID_LOG" >&2
  exit 2
fi
if [ ! -f "$IOS_LOG" ]; then
  echo "ERROR: iOS diagnostics log not found: $IOS_LOG" >&2
  exit 2
fi

tmp_input="$(mktemp)"
trap 'rm -f "$tmp_input"' EXIT

cat "$ANDROID_LOG" "$IOS_LOG" > "$tmp_input"

python3 - "$tmp_input" "$ANDROID_LOG" "$IOS_LOG" <<'PY'
import re
import sys
from collections import defaultdict

combined_path, android_path, ios_path = sys.argv[1:4]

pat = re.compile(r"delivery_state msg=([^ ]+) state=([a-z_]+)\b")

allowed_after_delivered = {"delivered"}
allowed_after_failed = {"failed"}

state_events = defaultdict(list)
regressions = []

with open(combined_path, "r", errors="ignore") as f:
    for idx, line in enumerate(f, start=1):
        m = pat.search(line)
        if not m:
            continue
        msg_id = m.group(1)
        state = m.group(2).lower()
        state_events[msg_id].append((idx, state, line.rstrip()))

        prior = [event[1] for event in state_events[msg_id][:-1]]
        if "delivered" in prior and state not in allowed_after_delivered:
            regressions.append((msg_id, "delivered", state, line.rstrip()))
            continue
        if "failed" in prior and state not in allowed_after_failed:
            regressions.append((msg_id, "failed", state, line.rstrip()))

messages_with_state = len(state_events)
total_state_events = sum(len(v) for v in state_events.values())

print("delivery_state_monotonicity:")
print(f"  android_log: {android_path}")
print(f"  ios_log: {ios_path}")
print(f"  messages_with_delivery_state: {messages_with_state}")
print(f"  total_delivery_state_events: {total_state_events}")
print(f"  regressions: {len(regressions)}")

if regressions:
    print("  samples:")
    for msg_id, terminal, observed, line in regressions[:10]:
        print(f"    - msg={msg_id} terminal={terminal} observed={observed} line={line}")
    sys.exit(1)

print("  status: PASS")
sys.exit(0)
PY
