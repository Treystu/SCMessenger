#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT_PATH="$ROOT_DIR/iOS/SCMessenger/SCMessenger.xcodeproj"
SCHEME="SCMessenger"

APPLE_TEAM_ID="${APPLE_TEAM_ID:-}"
DEVICE_UDID="${DEVICE_UDID:-}"
BUNDLE_ID="${BUNDLE_ID:-SovereignCommunications.SCMessenger}"
CONFIGURATION="${CONFIGURATION:-Debug}"
LAUNCH_AFTER_INSTALL="${LAUNCH_AFTER_INSTALL:-1}"
DERIVED_DATA_PATH="${DERIVED_DATA_PATH:-$ROOT_DIR/.build/ios-device}"
APP_PATH="$DERIVED_DATA_PATH/Build/Products/${CONFIGURATION}-iphoneos/SCMessenger.app"
CLEAN_BUILD="${CLEAN_BUILD:-1}"
UNINSTALL_FIRST="${UNINSTALL_FIRST:-1}"

if [ -z "$APPLE_TEAM_ID" ]; then
  echo "error: APPLE_TEAM_ID is required."
  echo "usage: APPLE_TEAM_ID=<YOUR_TEAM_ID> DEVICE_UDID=<YOUR_DEVICE_UDID> ./iOS/install-device.sh"
  exit 1
fi

if [ -z "$DEVICE_UDID" ]; then
  echo "error: DEVICE_UDID is required."
  echo "hint: run 'xcrun xcdevice list' (Xcode ID) or 'xcrun devicectl list devices' (CoreDevice ID)."
  echo "usage: APPLE_TEAM_ID=<YOUR_TEAM_ID> DEVICE_UDID=<YOUR_DEVICE_UDID> ./iOS/install-device.sh"
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required for device ID resolution." >&2
  exit 1
fi

XCDEVICE_JSON="$(mktemp)"
DEVICECTL_JSON="$(mktemp)"
cleanup_temp_files() {
  rm -f "$XCDEVICE_JSON" "$DEVICECTL_JSON"
}
trap cleanup_temp_files EXIT

if ! xcrun xcdevice list >"$XCDEVICE_JSON" 2>/dev/null; then
  echo "error: failed to query Xcode devices via 'xcrun xcdevice list'." >&2
  exit 1
fi

if ! xcrun devicectl list devices --json-output "$DEVICECTL_JSON" >/dev/null 2>&1; then
  echo "error: failed to query CoreDevice list via 'xcrun devicectl list devices'." >&2
  exit 1
fi

XCODE_DEVICE_UDID=""
DEVICECTL_IDENTIFIER=""
DEVICE_NAME=""

# Accept either Xcode destination UDID or CoreDevice identifier as DEVICE_UDID.
XCODE_DEVICE_UDID="$(jq -r --arg id "$DEVICE_UDID" '
  .[] | select(.simulator == false and (.platform | contains("iphoneos")) and .available == true and .identifier == $id) | .identifier
' "$XCDEVICE_JSON" | head -n 1)"

if [ -n "$XCODE_DEVICE_UDID" ]; then
  DEVICE_NAME="$(jq -r --arg id "$XCODE_DEVICE_UDID" '
    .[] | select(.identifier == $id) | .name
  ' "$XCDEVICE_JSON" | head -n 1)"
fi

DEVICECTL_IDENTIFIER="$(jq -r --arg id "$DEVICE_UDID" '
  .result.devices[]
  | select((.connectionProperties.tunnelState // "") == "connected" and .identifier == $id)
  | .identifier
' "$DEVICECTL_JSON" | head -n 1)"

if [ -n "$DEVICECTL_IDENTIFIER" ] && [ -z "$DEVICE_NAME" ]; then
  DEVICE_NAME="$(jq -r --arg id "$DEVICECTL_IDENTIFIER" '
    .result.devices[]
    | select(.identifier == $id)
    | (.deviceProperties.name // .name // "")
  ' "$DEVICECTL_JSON" | head -n 1)"
fi

if [ -n "$DEVICE_NAME" ] && [ -z "$XCODE_DEVICE_UDID" ]; then
  XCODE_DEVICE_UDID="$(jq -r --arg name "$DEVICE_NAME" '
    .[]
    | select(.simulator == false and (.platform | contains("iphoneos")) and .available == true and .name == $name)
    | .identifier
  ' "$XCDEVICE_JSON" | head -n 1)"
fi

if [ -n "$DEVICE_NAME" ] && [ -z "$DEVICECTL_IDENTIFIER" ]; then
  DEVICECTL_IDENTIFIER="$(jq -r --arg name "$DEVICE_NAME" '
    .result.devices[]
    | select((.connectionProperties.tunnelState // "") == "connected" and (.deviceProperties.name // .name // "") == $name)
    | .identifier
  ' "$DEVICECTL_JSON" | head -n 1)"
fi

if [ -z "$XCODE_DEVICE_UDID" ] || [ -z "$DEVICECTL_IDENTIFIER" ]; then
  echo "error: failed to resolve both device IDs for '$DEVICE_UDID'." >&2
  echo "Connected iOS devices (Xcode IDs):" >&2
  jq -r '
    .[]
    | select(.simulator == false and (.platform | contains("iphoneos")) and .available == true)
    | "  - \(.name): \(.identifier)"
  ' "$XCDEVICE_JSON" >&2
  echo "Connected iOS devices (CoreDevice IDs):" >&2
  jq -r '
    .result.devices[]
    | select((.connectionProperties.tunnelState // "") == "connected")
    | "  - \((.deviceProperties.name // .name // "Unknown")): \(.identifier)"
  ' "$DEVICECTL_JSON" >&2
  exit 1
fi

DEVICE_UDID="$XCODE_DEVICE_UDID"

echo "== SCMessenger iOS install =="
echo "Team ID:             $APPLE_TEAM_ID"
echo "Device Name:         ${DEVICE_NAME:-Unknown}"
echo "Xcode Device UDID:   $DEVICE_UDID"
echo "CoreDevice ID:       $DEVICECTL_IDENTIFIER"
echo "Bundle ID:           $BUNDLE_ID"
echo "Configuration:       $CONFIGURATION"
echo "DerivedData:         $DERIVED_DATA_PATH"
echo "Clean build:         $CLEAN_BUILD"
echo "Uninstall first:     $UNINSTALL_FIRST"
echo "Launch after install $LAUNCH_AFTER_INSTALL"
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
echo "3) Building signed app for connected device..."
xcodebuild \
  -project "$PROJECT_PATH" \
  -scheme "$SCHEME" \
  -configuration "$CONFIGURATION" \
  -destination "id=$DEVICE_UDID" \
  -derivedDataPath "$DERIVED_DATA_PATH" \
  DEVELOPMENT_TEAM="$APPLE_TEAM_ID" \
  PRODUCT_BUNDLE_IDENTIFIER="$BUNDLE_ID" \
  CODE_SIGN_STYLE=Automatic \
  -allowProvisioningUpdates \
  build

if [ ! -d "$APP_PATH" ]; then
  echo "error: expected app bundle not found at:"
  echo "  $APP_PATH"
  exit 1
fi

echo
if [ "$UNINSTALL_FIRST" = "1" ]; then
  echo "4) Removing previous install (if present)..."
  xcrun devicectl device uninstall app --device "$DEVICECTL_IDENTIFIER" "$BUNDLE_ID" || true
  echo
fi

echo "5) Installing app on device..."
xcrun devicectl device install app --device "$DEVICECTL_IDENTIFIER" "$APP_PATH"

if [ "$LAUNCH_AFTER_INSTALL" = "1" ]; then
  echo
  echo "6) Launching app..."
  if ! xcrun devicectl device process launch --device "$DEVICECTL_IDENTIFIER" --terminate-existing "$BUNDLE_ID"; then
    echo "warning: app install succeeded, but launch failed."
    echo "hint: on iPhone, trust the developer profile and re-launch manually."
  fi
fi

echo
echo "Install complete."
