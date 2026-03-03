#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT_PATH="$ROOT_DIR/iOS/SCMessenger/SCMessenger.xcodeproj"
SCHEME="SCMessenger"

SIM_UDID="${SIM_UDID:-}"
SIM_NAME="${SIM_NAME:-iPhone 17}"
BUNDLE_ID="${BUNDLE_ID:-SovereignCommunications.SCMessenger}"
CONFIGURATION="${CONFIGURATION:-Debug}"
LAUNCH_AFTER_INSTALL="${LAUNCH_AFTER_INSTALL:-1}"
DERIVED_DATA_PATH="${DERIVED_DATA_PATH:-$ROOT_DIR/.build/ios-sim}"
APP_PATH="$DERIVED_DATA_PATH/Build/Products/${CONFIGURATION}-iphonesimulator/SCMessenger.app"
CLEAN_BUILD="${CLEAN_BUILD:-1}"
UNINSTALL_FIRST="${UNINSTALL_FIRST:-1}"

find_sim_udid_by_name() {
  local name="$1"
  xcrun simctl list devices available | awk -v sim_name="$name" '
    index($0, sim_name) && $0 ~ /\([0-9A-F-]{36}\)/ {
      match($0, /\([0-9A-F-]{36}\)/)
      print substr($0, RSTART + 1, RLENGTH - 2)
      exit
    }
  '
}

find_first_available_iphone_udid() {
  xcrun simctl list devices available | awk '
    /iPhone/ && $0 ~ /\([0-9A-F-]{36}\)/ {
      match($0, /\([0-9A-F-]{36}\)/)
      print substr($0, RSTART + 1, RLENGTH - 2)
      exit
    }
  '
}

if [ -z "$SIM_UDID" ]; then
  SIM_UDID="$(find_sim_udid_by_name "$SIM_NAME")"
fi

if [ -z "$SIM_UDID" ]; then
  SIM_UDID="$(find_first_available_iphone_udid)"
fi

if [ -z "$SIM_UDID" ]; then
  echo "error: no available iOS simulator found."
  exit 1
fi

echo "== SCMessenger iOS simulator install =="
echo "Simulator UDID:      $SIM_UDID"
echo "Simulator name hint: $SIM_NAME"
echo "Bundle ID:           $BUNDLE_ID"
echo "Configuration:       $CONFIGURATION"
echo "DerivedData:         $DERIVED_DATA_PATH"
echo "Clean build:         $CLEAN_BUILD"
echo "Uninstall first:     $UNINSTALL_FIRST"
echo "Launch after install $LAUNCH_AFTER_INSTALL"
echo

echo "0) Booting simulator..."
xcrun simctl boot "$SIM_UDID" >/dev/null 2>&1 || true
open -a Simulator >/dev/null 2>&1 || true
xcrun simctl bootstatus "$SIM_UDID" -b

echo
echo "1) Generating/copying UniFFI bindings..."
"$ROOT_DIR/iOS/copy-bindings.sh"

echo
echo "1b) Verifying generated path invariants..."
bash "$ROOT_DIR/iOS/assert-generated-path.sh"

echo
echo "2) Preparing clean build workspace..."
if [ "$CLEAN_BUILD" = "1" ]; then
  rm -rf "$DERIVED_DATA_PATH"
  echo "Removed DerivedData: $DERIVED_DATA_PATH"
fi

echo
echo "3) Building app for simulator..."
xcodebuild \
  -project "$PROJECT_PATH" \
  -scheme "$SCHEME" \
  -configuration "$CONFIGURATION" \
  -destination "id=$SIM_UDID" \
  -derivedDataPath "$DERIVED_DATA_PATH" \
  CODE_SIGNING_ALLOWED=NO \
  CODE_SIGNING_REQUIRED=NO \
  build

if [ ! -d "$APP_PATH" ]; then
  echo "error: expected simulator app bundle not found at:"
  echo "  $APP_PATH"
  exit 1
fi

echo
if [ "$UNINSTALL_FIRST" = "1" ]; then
  echo "4) Removing previous simulator install (if present)..."
  xcrun simctl uninstall "$SIM_UDID" "$BUNDLE_ID" || true
  echo
fi

echo "5) Installing app on simulator..."
xcrun simctl install "$SIM_UDID" "$APP_PATH"

if [ "$LAUNCH_AFTER_INSTALL" = "1" ]; then
  echo
  echo "6) Launching app..."
  xcrun simctl launch "$SIM_UDID" "$BUNDLE_ID" || true
fi

echo
echo "Install complete."
