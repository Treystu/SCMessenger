#!/usr/bin/env bash
set -euo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$DIR"

echo "Verifying iOS workspace build..."

bash "../iOS/assert-generated-path.sh"

# Deterministic simulator destination without provisioning requirements.
LOG_FILE="$(mktemp -t scmessenger-ios-build.XXXXXX.log)"
trap 'rm -f "$LOG_FILE"' EXIT

if [ -f "SCMessenger/SCMessenger.xcworkspace/contents.xcworkspacedata" ]; then
  BUILD_CMD=(xcodebuild -workspace SCMessenger/SCMessenger.xcworkspace)
else
  BUILD_CMD=(xcodebuild -project SCMessenger/SCMessenger.xcodeproj)
fi

if ! "${BUILD_CMD[@]}" \
  -scheme SCMessenger \
  -destination "generic/platform=iOS Simulator" \
  -configuration Debug \
  build 2>&1 | tee "$LOG_FILE"; then
  echo "iOS build verification failed. See log output above."
  exit 1
fi

# Explicit warning policy: report warnings; do not fail verification on warnings.
WARNING_COUNT=$(grep -c " warning: " "$LOG_FILE" || true)
if [ "$WARNING_COUNT" -gt 0 ]; then
  echo "iOS build verification completed with $WARNING_COUNT warning(s)."
else
  echo "iOS build verification completed with 0 warnings."
fi

echo "iOS build verification passed!"
