#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT_PATH="$ROOT_DIR/iOS/SCMessenger/SCMessenger.xcodeproj"
SCHEME="SCMessenger"

APPLE_TEAM_ID="${APPLE_TEAM_ID:-}"
BUNDLE_ID="${BUNDLE_ID:-SovereignCommunications.SCMessenger}"
CONFIGURATION="${CONFIGURATION:-Debug}"

if [ -z "$APPLE_TEAM_ID" ]; then
  echo "error: APPLE_TEAM_ID is required."
  echo "usage: APPLE_TEAM_ID=<YOUR_TEAM_ID> ./iOS/build-device.sh"
  exit 1
fi

echo "== SCMessenger iOS device build =="
echo "Team ID:        $APPLE_TEAM_ID"
echo "Bundle ID:      $BUNDLE_ID"
echo "Configuration:  $CONFIGURATION"
echo

echo "1) Generating/copying UniFFI bindings..."
"$ROOT_DIR/iOS/copy-bindings.sh"

echo
echo "2) Building for physical iOS device..."
xcodebuild \
  -project "$PROJECT_PATH" \
  -scheme "$SCHEME" \
  -configuration "$CONFIGURATION" \
  -destination "generic/platform=iOS" \
  DEVELOPMENT_TEAM="$APPLE_TEAM_ID" \
  PRODUCT_BUNDLE_IDENTIFIER="$BUNDLE_ID" \
  CODE_SIGN_STYLE=Automatic \
  -allowProvisioningUpdates \
  build

echo
echo "Build complete. Open Xcode Organizer if you want to archive/export, or run from Xcode with your device selected."
