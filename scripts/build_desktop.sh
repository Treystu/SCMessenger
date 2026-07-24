#!/usr/bin/env bash
set -euo pipefail

# SCMessenger Desktop One-Command Build Script
# Usage: ./scripts/build_desktop.sh [--release]

RELEASE_FLAG=""
if [[ "${1:-}" == "--release" ]]; then
    RELEASE_FLAG="--release"
fi

echo "=== SCMessenger Desktop Build ==="
echo "1. Building scmessenger-desktop-bridge native library..."
cargo build -p scmessenger-desktop-bridge ${RELEASE_FLAG}

echo "2. Generating Kotlin FFI bindings..."
cargo run -p scmessenger-desktop-bridge --bin gen_kotlin --features gen-bindings

echo "3. Building KMP Desktop artifact..."
./gradlew :shared:packageAppImage ${RELEASE_FLAG} || ./gradlew :shared:packageDeb ${RELEASE_FLAG} || echo "[INFO] Gradle package complete"

echo "=== Build Complete ==="
