#!/usr/bin/env bash
set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"

echo "Verifying iOS workspace build..."

# Use iphonesimulator to verify generic buildness without needing explicit provisioning profiles
xcodebuild \
  -workspace SCMessenger/SCMessenger.xcworkspace \
  -scheme SCMessenger \
  -sdk iphonesimulator \
  build | grep -v "note: " | grep -v "warning: "

echo "iOS build verification passed!"
