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

if [ -z "$APPLE_TEAM_ID" ]; then
  echo "error: APPLE_TEAM_ID is required."
  echo "usage: APPLE_TEAM_ID=<YOUR_TEAM_ID> DEVICE_UDID=<YOUR_DEVICE_UDID> ./iOS/install-device.sh"
  exit 1
fi

if [ -z "$DEVICE_UDID" ]; then
  echo "error: DEVICE_UDID is required."
  echo "hint: run 'xcrun devicectl list devices' and copy your iPhone UDID."
  echo "usage: APPLE_TEAM_ID=<YOUR_TEAM_ID> DEVICE_UDID=<YOUR_DEVICE_UDID> ./iOS/install-device.sh"
  exit 1
fi

echo "== SCMessenger iOS install =="
echo "Team ID:             $APPLE_TEAM_ID"
echo "Device UDID:         $DEVICE_UDID"
echo "Bundle ID:           $BUNDLE_ID"
echo "Configuration:       $CONFIGURATION"
echo "DerivedData:         $DERIVED_DATA_PATH"
echo "Launch after install $LAUNCH_AFTER_INSTALL"
echo

echo "1) Generating/copying UniFFI bindings..."
"$ROOT_DIR/iOS/copy-bindings.sh"

echo
echo "2) Building signed app for connected device..."
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
echo "3) Installing app on device..."
xcrun devicectl device install app --device "$DEVICE_UDID" "$APP_PATH"

if [ "$LAUNCH_AFTER_INSTALL" = "1" ]; then
  echo
  echo "4) Launching app..."
  xcrun devicectl device process launch --device "$DEVICE_UDID" --terminate-existing "$BUNDLE_ID"
fi

echo
echo "Install complete."
