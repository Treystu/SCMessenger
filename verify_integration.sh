#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

echo "========================================"
echo "VERIFYING CURRENT INTEGRATION BASELINE"
echo "========================================"
echo "Legacy grep-based six-phase checks were retired due frequent false negatives."
echo "Running canonical WS12 integration matrix instead."
echo ""

if [[ -z "${ANDROID_HOME:-}" ]]; then
  echo "ANDROID_HOME is not set; defaulting to /Users/christymaxwell/Library/Android/sdk"
  export ANDROID_HOME="/Users/christymaxwell/Library/Android/sdk"
fi

exec ./scripts/verify_ws12_matrix.sh
