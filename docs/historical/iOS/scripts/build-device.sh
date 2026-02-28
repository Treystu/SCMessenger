#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT_PATH="$ROOT_DIR/iOS/SCMessenger/SCMessenger.xcodeproj"
SCHEME="SCMessenger"

APPLE_TEAM_ID="${APPLE_TEAM_ID:-}"
BUNDLE_ID="${BUNDLE_ID:-SovereignCommunications.SCMessenger}"
CONFIGURATION="${CONFIGURATION:-Debug}"
CLEAN_BUILD="${CLEAN_BUILD:-1}"
DERIVED_DATA_PATH="${DERIVED_DATA_PATH:-$ROOT_DIR/.build/ios-device-buildonly}"

if [ -z "$APPLE_TEAM_ID" ]; then
  echo "error: APPLE_TEAM_ID is required."
  echo "usage: APPLE_TEAM_ID=<YOUR_TEAM_ID> ./iOS/build-device.sh"
  exit 1
fi

echo "== SCMessenger iOS device build =="
echo "Team ID:        $APPLE_TEAM_ID"
echo "Bundle ID:      $BUNDLE_ID"
echo "Configuration:  $CONFIGURATION"
echo "DerivedData:    $DERIVED_DATA_PATH"
echo "Clean build:    $CLEAN_BUILD"
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
echo "3) Building for physical iOS device..."
xcodebuild \
  -project "$PROJECT_PATH" \
  -scheme "$SCHEME" \
  -configuration "$CONFIGURATION" \
  -destination "generic/platform=iOS" \
  -derivedDataPath "$DERIVED_DATA_PATH" \
  DEVELOPMENT_TEAM="$APPLE_TEAM_ID" \
  PRODUCT_BUNDLE_IDENTIFIER="$BUNDLE_ID" \
  CODE_SIGN_STYLE=Automatic \
  -allowProvisioningUpdates \
  build

echo
echo "Build complete. Open Xcode Organizer if you want to archive/export, or run from Xcode with your device selected."
