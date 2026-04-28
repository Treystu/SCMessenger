#!/usr/bin/env bash
# scripts/capture_logs.sh — Unified log capture for all SCMessenger nodes
#
# Usage:
#   ./scripts/capture_logs.sh
#   DURATION_SEC=120 ./scripts/capture_logs.sh
#   CAPTURE_ANDROID=0 ./scripts/capture_logs.sh
#
# Environment variables:
#   TIMESTAMP         — override timestamp (default: current time)
#   LOGDIR            — override output directory (default: logs/capture/<timestamp>)
#   DURATION_SEC      — capture duration in seconds (default: 60)
#   ANDROID_SERIAL    — specific Android device serial (default: auto-detect)
#   IOS_DEVICE_UDID   — specific iOS device UDID (default: auto-detect)
#   IOS_SIM_UDID      — specific iOS simulator UDID (default: auto-detect)
#   CAPTURE_ANDROID   — capture Android logs (default: 1)
#   CAPTURE_IOS_DEVICE — capture iOS device logs (default: 1)
#   CAPTURE_IOS_SIM   — capture iOS simulator logs (default: 1)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

TIMESTAMP="${TIMESTAMP:-$(date +%Y%m%d_%H%M%S)}"
LOGDIR="${LOGDIR:-$ROOT_DIR/logs/capture/$TIMESTAMP}"
DURATION_SEC="${DURATION_SEC:-60}"
ANDROID_SERIAL="${ANDROID_SERIAL:-}"
IOS_DEVICE_UDID="${IOS_DEVICE_UDID:-}"
IOS_SIM_UDID="${IOS_SIM_UDID:-}"
CAPTURE_ANDROID="${CAPTURE_ANDROID:-1}"
CAPTURE_IOS_DEVICE="${CAPTURE_IOS_DEVICE:-1}"
CAPTURE_IOS_SIM="${CAPTURE_IOS_SIM:-1}"
MAX_LOG_SIZE_MB="${MAX_LOG_SIZE_MB:-500}"

mkdir -p "$LOGDIR"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  SCMessenger Unified Log Capture                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo "Log directory: $LOGDIR"
echo "Duration:      ${DURATION_SEC}s"
echo ""

# PID tracking for cleanup
declare -a CAPTURE_PIDS=()

cleanup() {
  echo ""
  echo "Stopping log capture..."
  for pid in "${CAPTURE_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  done
  
  # Inject sync end marker
  SYNC_MARKER="=== CAPTURE_END: $TIMESTAMP ==="
  if [ -n "${ANDROID_SERIAL:-}" ]; then
    echo "$SYNC_MARKER" | adb -s "$ANDROID_SERIAL" logcat -b main -T 1 2>/dev/null || true
  fi
  
  echo "Capture complete. Files in: $LOGDIR"
}
trap cleanup EXIT INT TERM

# Inject sync marker for cross-platform correlation
SYNC_MARKER="=== CAPTURE_START: $TIMESTAMP ==="
echo "Sync marker: $SYNC_MARKER"

# Android capture
if [ "$CAPTURE_ANDROID" = "1" ]; then
  if [ -z "$ANDROID_SERIAL" ]; then
    ANDROID_SERIAL=$(adb devices 2>/dev/null | awk 'NR>1 && $2=="device" {print $1; exit}')
  fi
  if [ -n "$ANDROID_SERIAL" ]; then
    echo "Starting Android log capture (serial: $ANDROID_SERIAL)..."
    adb -s "$ANDROID_SERIAL" logcat -c || true
    timeout "$DURATION_SEC" adb -s "$ANDROID_SERIAL" logcat -v time > "$LOGDIR/android.log" 2>&1 &
    CAPTURE_PIDS+=($!)
    
    # Inject sync marker
    echo "$SYNC_MARKER" | adb -s "$ANDROID_SERIAL" logcat -b main -T 1 2>/dev/null || true
  else
    echo "Skipping Android: no device connected"
  fi
fi

# iOS Device capture
if [ "$CAPTURE_IOS_DEVICE" = "1" ]; then
  if [ -z "$IOS_DEVICE_UDID" ]; then
    IOS_DEVICE_UDID=$(xcrun devicectl list devices --hide-default-columns --columns Identifier --columns State --hide-headers 2>/dev/null | awk '$2 ~ /(available|connected)/ {print $1; exit}')
  fi
  if [ -n "$IOS_DEVICE_UDID" ]; then
    echo "Starting iOS device log capture (UDID: $IOS_DEVICE_UDID)..."
    timeout "$DURATION_SEC" xcrun devicectl device log show --device "$IOS_DEVICE_UDID" --predicate 'subsystem == "com.scmessenger"' --style compact > "$LOGDIR/ios-device.log" 2>&1 &
    CAPTURE_PIDS+=($!)
  else
    echo "Skipping iOS device: none connected"
  fi
fi

# iOS Simulator capture
if [ "$CAPTURE_IOS_SIM" = "1" ]; then
  if [ -z "$IOS_SIM_UDID" ]; then
    IOS_SIM_UDID=$(xcrun simctl list devices 2>/dev/null | awk -F '[()]' '/Booted/{print $2; exit}')
  fi
  if [ -n "$IOS_SIM_UDID" ]; then
    echo "Starting iOS simulator log capture (UDID: $IOS_SIM_UDID)..."
    timeout "$DURATION_SEC" xcrun simctl spawn "$IOS_SIM_UDID" log stream --predicate 'subsystem == "com.scmessenger" OR process == "SCMessenger"' --style compact > "$LOGDIR/ios-sim.log" 2>&1 &
    CAPTURE_PIDS+=($!)
  else
    echo "Skipping iOS simulator: none booted"
  fi
fi

echo ""
echo "Capturing for ${DURATION_SEC}s..."
echo "Perform your test actions now."
echo ""

# Wait for all captures
for pid in "${CAPTURE_PIDS[@]}"; do
  wait "$pid" 2>/dev/null || true
done

# Clear trap since we're exiting normally
trap - EXIT

# Check log directory size
LOG_SIZE_KB=$(du -sk "$LOGDIR" 2>/dev/null | awk '{print $1}')
if [ "$LOG_SIZE_KB" -gt $((MAX_LOG_SIZE_MB * 1024)) ]; then
  echo "WARNING: Log directory exceeds ${MAX_LOG_SIZE_MB}MB (${LOG_SIZE_KB}KB)"
fi

# Quick analysis
echo ""
echo "Capture Summary:"
for logfile in "$LOGDIR"/*.log; do
  [ -f "$logfile" ] || continue
  lines=$(wc -l < "$logfile" | tr -d ' ')
  errors=$(grep -ciE "error|exception|crash|failed" "$logfile" 2>/dev/null || echo 0)
  echo "  $(basename "$logfile"): $lines lines, $errors errors"
done

echo ""
echo "Log files saved to: $LOGDIR"
