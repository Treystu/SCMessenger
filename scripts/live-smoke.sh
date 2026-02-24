#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TS="$(date +%Y%m%d-%H%M%S)"
LOG_DIR="${LOG_DIR:-$ROOT_DIR/logs/live-smoke/$TS}"
DURATION_SEC="${DURATION_SEC:-300}"
ANDROID_INSTALL="${ANDROID_INSTALL:-1}"
IOS_INSTALL="${IOS_INSTALL:-1}"
IOS_TARGET="${IOS_TARGET:-device}" # device | simulator
ANDROID_SERIAL="${ANDROID_SERIAL:-}"
IOS_DEVICE_UDID="${IOS_DEVICE_UDID:-}"
IOS_SIM_UDID="${IOS_SIM_UDID:-}"
IOS_SIM_NAME="${IOS_SIM_NAME:-iPhone 17}"
APPLE_TEAM_ID="${APPLE_TEAM_ID:-}"
ANDROID_APP_ID="${ANDROID_APP_ID:-com.scmessenger.android}"
IOS_BUNDLE_ID="${IOS_BUNDLE_ID:-SovereignCommunications.SCMessenger}"
GCP_RELAY_CHECK="${GCP_RELAY_CHECK:-0}"
GCP_RELAY_IP="${GCP_RELAY_IP:-34.135.34.73}"
GCP_RELAY_PORT="${GCP_RELAY_PORT:-9001}"

mkdir -p "$LOG_DIR"

echo "== SCMessenger Live Smoke =="
echo "Log dir:        $LOG_DIR"
echo "Duration:       ${DURATION_SEC}s"
echo "Android install:$ANDROID_INSTALL"
echo "iOS install:    $IOS_INSTALL"
echo "iOS target:     $IOS_TARGET"
echo "GCP check:      $GCP_RELAY_CHECK (${GCP_RELAY_IP}:${GCP_RELAY_PORT})"
echo

if command -v adb >/dev/null 2>&1; then
  adb devices -l | tee "$LOG_DIR/adb-devices.txt"
else
  echo "error: adb not found" >&2
  exit 1
fi

read_connected_android_serials() {
  ANDROID_DEVICE_SERIALS=()
  while IFS= read -r serial; do
    [ -n "$serial" ] || continue
    ANDROID_DEVICE_SERIALS+=("$serial")
  done < <(adb devices | awk -F'\t' 'NR>1 && $2=="device" {print $1}')
}

read_connected_android_serials
if [ "${#ANDROID_DEVICE_SERIALS[@]}" -eq 0 ]; then
  echo "No active adb device found, attempting wireless reconnect..."
  while IFS= read -r endpoint; do
    [ -n "$endpoint" ] || continue
    adb connect "$endpoint" >/dev/null 2>&1 || true
  done < <(adb mdns services 2>/dev/null | awk '/_adb-tls-connect\._tcp/ {print $NF}')
  read_connected_android_serials
fi

if [ "${#ANDROID_DEVICE_SERIALS[@]}" -eq 0 ]; then
  echo "error: no connected Android device detected (adb devices shows none)." >&2
  exit 1
fi

if [ -z "$ANDROID_SERIAL" ]; then
  for serial in "${ANDROID_DEVICE_SERIALS[@]}"; do
    if [[ "$serial" == *:* ]]; then
      ANDROID_SERIAL="$serial"
      break
    fi
  done
  if [ -z "$ANDROID_SERIAL" ]; then
    ANDROID_SERIAL="${ANDROID_DEVICE_SERIALS[0]}"
  fi
fi

adb_sel() {
  adb -s "$ANDROID_SERIAL" "$@"
}

echo "Using Android serial: $ANDROID_SERIAL"

IOS_RUNTIME_ID=""
IOS_RUNTIME_KIND=""

if [ "$IOS_TARGET" = "device" ]; then
  if ! xcrun devicectl list devices >/tmp/sc-devices.txt 2>&1; then
    cat /tmp/sc-devices.txt
    echo "error: unable to list iOS devices" >&2
    exit 1
  fi
  cat /tmp/sc-devices.txt | tee "$LOG_DIR/ios-devices.txt"

  if [ -z "$IOS_DEVICE_UDID" ]; then
    IOS_DEVICE_UDID="$(awk '/connected/ {print $4; exit}' /tmp/sc-devices.txt)"
  fi

  if [ -z "$IOS_DEVICE_UDID" ]; then
    echo "error: no connected iOS device detected" >&2
    exit 1
  fi

  IOS_RUNTIME_ID="$IOS_DEVICE_UDID"
  IOS_RUNTIME_KIND="device"
  echo "Using iOS device: $IOS_RUNTIME_ID"
elif [ "$IOS_TARGET" = "simulator" ]; then
  if ! xcrun simctl list devices available >/tmp/sc-sim-devices.txt 2>&1; then
    cat /tmp/sc-sim-devices.txt
    echo "error: unable to list iOS simulators" >&2
    exit 1
  fi
  cat /tmp/sc-sim-devices.txt | tee "$LOG_DIR/ios-simulators.txt"

  if [ -z "$IOS_SIM_UDID" ]; then
    IOS_SIM_UDID="$(xcrun simctl list devices available | awk -v name="$IOS_SIM_NAME" '
      index($0, name) && $0 ~ /\([0-9A-F-]{36}\)/ {
        match($0, /\([0-9A-F-]{36}\)/)
        print substr($0, RSTART + 1, RLENGTH - 2)
        exit
      }
    ')"
  fi

  if [ -z "$IOS_SIM_UDID" ]; then
    IOS_SIM_UDID="$(xcrun simctl list devices available | awk '
      /iPhone/ && $0 ~ /\([0-9A-F-]{36}\)/ {
        match($0, /\([0-9A-F-]{36}\)/)
        print substr($0, RSTART + 1, RLENGTH - 2)
        exit
      }
    ')"
  fi

  if [ -z "$IOS_SIM_UDID" ]; then
    echo "error: no available iPhone simulator detected" >&2
    exit 1
  fi

  xcrun simctl boot "$IOS_SIM_UDID" >/dev/null 2>&1 || true
  open -a Simulator >/dev/null 2>&1 || true
  xcrun simctl bootstatus "$IOS_SIM_UDID" -b

  IOS_RUNTIME_ID="$IOS_SIM_UDID"
  IOS_RUNTIME_KIND="simulator"
  echo "Using iOS simulator: $IOS_RUNTIME_ID ($IOS_SIM_NAME)"
else
  echo "error: IOS_TARGET must be 'device' or 'simulator'" >&2
  exit 1
fi

if [ "$GCP_RELAY_CHECK" = "1" ]; then
  echo
  echo "[0/6] Checking GCP relay reachability..."
  if nc -z -w 5 "$GCP_RELAY_IP" "$GCP_RELAY_PORT"; then
    echo "GCP relay reachable: $GCP_RELAY_IP:$GCP_RELAY_PORT" | tee "$LOG_DIR/gcp-relay-check.txt"
  else
    echo "warning: GCP relay not reachable: $GCP_RELAY_IP:$GCP_RELAY_PORT" | tee "$LOG_DIR/gcp-relay-check.txt"
  fi
fi

if [ "$ANDROID_INSTALL" = "1" ]; then
  echo
  echo "[1/6] Clean-install Android..."
  ANDROID_SERIAL="$ANDROID_SERIAL" "$ROOT_DIR/android/install-clean.sh" | tee "$LOG_DIR/android-install.log"
fi

if [ "$IOS_INSTALL" = "1" ]; then
  echo
  echo "[2/6] Clean-install iOS ($IOS_RUNTIME_KIND)..."
  if [ "$IOS_RUNTIME_KIND" = "device" ]; then
    if [ -z "$APPLE_TEAM_ID" ]; then
      echo "error: APPLE_TEAM_ID is required when IOS_TARGET=device and IOS_INSTALL=1" >&2
      exit 1
    fi
    APPLE_TEAM_ID="$APPLE_TEAM_ID" DEVICE_UDID="$IOS_RUNTIME_ID" CLEAN_BUILD="${IOS_CLEAN_BUILD:-1}" \
      "$ROOT_DIR/iOS/install-device.sh" | tee "$LOG_DIR/ios-install.log"
  else
    SIM_UDID="$IOS_RUNTIME_ID" SIM_NAME="$IOS_SIM_NAME" BUNDLE_ID="$IOS_BUNDLE_ID" CLEAN_BUILD="${IOS_CLEAN_BUILD:-1}" \
      "$ROOT_DIR/iOS/install-sim.sh" | tee "$LOG_DIR/ios-install.log"
  fi
fi

echo
printf "[3/6] Launching Android app...\n"
adb_sel shell monkey -p "$ANDROID_APP_ID" -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1 || true

if [ "$IOS_RUNTIME_KIND" = "simulator" ]; then
  echo "[4/6] Launching iOS app on simulator..."
  xcrun simctl launch "$IOS_RUNTIME_ID" "$IOS_BUNDLE_ID" >/dev/null 2>&1 || true
fi

echo "[5/6] Starting log capture..."
adb_sel logcat -c || true
adb_sel logcat -v time >"$LOG_DIR/android-logcat.txt" &
LOGCAT_PID=$!

IOS_LOG_PID=""
if [ "$IOS_RUNTIME_KIND" = "simulator" ]; then
  xcrun simctl spawn "$IOS_RUNTIME_ID" log stream --style compact --predicate 'subsystem == "com.scmessenger" OR process == "SCMessenger"' >"$LOG_DIR/ios-sim-log.txt" 2>&1 &
  IOS_LOG_PID=$!
fi

trap 'kill "$LOGCAT_PID" >/dev/null 2>&1 || true; if [ -n "$IOS_LOG_PID" ]; then kill "$IOS_LOG_PID" >/dev/null 2>&1 || true; fi' EXIT

echo
cat <<EOF
[6/6] Live interaction window started.
For the next ${DURATION_SEC}s, perform these on devices:
- Open Contacts on both devices.
- Attempt Android -> iOS and iOS -> Android message send.
- Verify LAN + relay discovery and message flow.
- Change iOS power settings once and keep app active.

Capturing:
- Android runtime logs: $LOG_DIR/android-logcat.txt
- iOS install/build logs: $LOG_DIR/ios-install.log (if install ran)
EOF

if [ "$IOS_RUNTIME_KIND" = "simulator" ]; then
  cat <<EOF
- iOS simulator logs: $LOG_DIR/ios-sim-log.txt

Note: iOS Simulator does not provide real CoreBluetooth hardware.
Nearby/BLE discovery tests against Android are expected to be unavailable in simulator mode.
EOF
fi

sleep "$DURATION_SEC"
kill "$LOGCAT_PID" >/dev/null 2>&1 || true
wait "$LOGCAT_PID" 2>/dev/null || true
if [ -n "$IOS_LOG_PID" ]; then
  kill "$IOS_LOG_PID" >/dev/null 2>&1 || true
  wait "$IOS_LOG_PID" 2>/dev/null || true
fi
trap - EXIT

echo
echo "Collecting quick diagnostics..."
adb_sel shell dumpsys package "$ANDROID_APP_ID" >"$LOG_DIR/android-package.txt" 2>/dev/null || true
if [ "$IOS_RUNTIME_KIND" = "device" ]; then
  xcrun devicectl device info processes --device "$IOS_RUNTIME_ID" >"$LOG_DIR/ios-processes.txt" 2>/dev/null || true
else
  xcrun simctl listapps "$IOS_RUNTIME_ID" >"$LOG_DIR/ios-sim-apps.txt" 2>/dev/null || true
  xcrun simctl spawn "$IOS_RUNTIME_ID" log show --last 10m --style compact --predicate 'subsystem == "com.scmessenger" OR process == "SCMessenger"' >"$LOG_DIR/ios-sim-log-last-10m.txt" 2>/dev/null || true
fi

echo "Done. Logs captured in $LOG_DIR"
