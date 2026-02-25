#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_ID="${APP_ID:-com.scmessenger.android}"
UNINSTALL_FIRST="${UNINSTALL_FIRST:-0}"
ANDROID_SERIAL="${ANDROID_SERIAL:-}"

cd "$ROOT_DIR/android"

echo "== SCMessenger Android clean install =="
echo "App ID:          $APP_ID"
echo "Uninstall first: $UNINSTALL_FIRST"
echo

if ! command -v adb >/dev/null 2>&1; then
  echo "error: adb is required but not found in PATH"
  exit 1
fi

adb_cmd() {
  if [ -n "$ANDROID_SERIAL" ]; then
    adb -s "$ANDROID_SERIAL" "$@"
  else
    adb "$@"
  fi
}

read_connected_serials() {
  DEVICE_SERIALS=()
  while IFS= read -r serial; do
    [ -n "$serial" ] || continue
    DEVICE_SERIALS+=("$serial")
  done < <(adb devices | awk -F'\t' 'NR>1 && $2=="device" {print $1}')
}

read_connected_serials
DEVICE_COUNT="${#DEVICE_SERIALS[@]}"
if [ "$DEVICE_COUNT" -eq 0 ]; then
  echo "No active adb device found, attempting wireless reconnect..."
  while IFS= read -r endpoint; do
    [ -n "$endpoint" ] || continue
    adb connect "$endpoint" >/dev/null 2>&1 || true
  done < <(adb mdns services 2>/dev/null | awk '/_adb-tls-connect\._tcp/ {print $NF}')
  read_connected_serials
  DEVICE_COUNT="${#DEVICE_SERIALS[@]}"
fi

if [ "$DEVICE_COUNT" -eq 0 ]; then
  echo "error: no connected Android device detected (adb devices shows none)."
  exit 1
fi

if [ -z "$ANDROID_SERIAL" ]; then
  for serial in "${DEVICE_SERIALS[@]}"; do
    if [[ "$serial" == *:* ]]; then
      ANDROID_SERIAL="$serial"
      break
    fi
  done
  if [ -z "$ANDROID_SERIAL" ]; then
    ANDROID_SERIAL="${DEVICE_SERIALS[0]}"
  fi
fi

echo "Android serial:   $ANDROID_SERIAL"

if [ "$UNINSTALL_FIRST" = "1" ]; then
  echo "1) Uninstalling existing app (if present)..."
  adb_cmd uninstall "$APP_ID" >/dev/null 2>&1 || true
  echo
fi

echo "2) Stopping Gradle daemons..."
./gradlew --stop

echo
echo "3) Clean build + installDebug..."
./gradlew clean :app:installDebug

echo
echo "4) Granting required runtime permissions..."
for perm in \
  android.permission.ACCESS_FINE_LOCATION \
  android.permission.ACCESS_COARSE_LOCATION \
  android.permission.BLUETOOTH_SCAN \
  android.permission.BLUETOOTH_ADVERTISE \
  android.permission.BLUETOOTH_CONNECT \
  android.permission.NEARBY_WIFI_DEVICES \
  android.permission.POST_NOTIFICATIONS
do
  adb_cmd shell pm grant "$APP_ID" "$perm" >/dev/null 2>&1 || true
done

echo
echo "Install complete."
