#!/usr/bin/env bash
# =============================================================================
# P5: Clean Build & Deploy Script
# =============================================================================
# Ensures a clean build before deploying to physical devices, preventing the
# Hilt NoClassDefFoundError crash caused by stale APKs with missing generated
# code (MeshApplication_GeneratedInjector).
#
# Usage:
#   ./scripts/deploy_to_device.sh android     # Build & deploy to connected Android device
#   ./scripts/deploy_to_device.sh ios          # Build & deploy to connected iOS device
#   ./scripts/deploy_to_device.sh both         # Build & deploy to both
# =============================================================================

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

deploy_android() {
    log_info "=== Android Clean Build & Deploy ==="
    local android_dir="$PROJECT_ROOT/android"

    if [ ! -d "$android_dir" ]; then
        log_error "Android directory not found: $android_dir"
        return 1
    fi

    # Check for connected device
    if ! adb devices | grep -q "device$"; then
        log_error "No Android device connected. Enable USB/Wireless debugging."
        return 1
    fi

    cd "$android_dir"

    # P5: CRITICAL — Always clean before device deploy to prevent stale APK issues
    log_info "Cleaning build artifacts (prevents Hilt NoClassDefFoundError)..."
    ./gradlew clean 2>&1 | tail -n 3

    log_info "Building debug APK..."
    ./gradlew assembleDebug 2>&1 | tail -n 5

    local apk_path
    apk_path=$(find app/build/outputs/apk/debug -name "*.apk" -type f | head -n 1)

    if [ -z "$apk_path" ]; then
        log_error "APK not found after build"
        return 1
    fi

    log_info "Installing APK: $apk_path"
    adb install -r "$apk_path"

    log_info "Launching app..."
    adb shell am start -n com.scmessenger.android/.ui.MainActivity

    log_info "✓ Android deploy complete"
}

deploy_ios() {
    log_info "=== iOS Clean Build & Deploy ==="
    local ios_dir="$PROJECT_ROOT/iOS/SCMessenger"

    if [ ! -d "$ios_dir" ]; then
        log_error "iOS directory not found: $ios_dir"
        return 1
    fi

    cd "$ios_dir"

    # Get connected device ID
    local device_id
    device_id=$(xcrun devicectl list devices 2>/dev/null | grep -E "iPhone|iPad" | head -n 1 | awk '{print $NF}')

    if [ -z "$device_id" ]; then
        log_warn "No iOS device detected via devicectl. Attempting xcodebuild anyway..."
    else
        log_info "Detected iOS device: $device_id"
    fi

    # P5: Clean build to prevent stale binary issues
    log_info "Cleaning build artifacts..."
    xcodebuild clean -scheme SCMessenger -quiet 2>/dev/null || true

    log_info "Building for device..."
    xcodebuild build \
        -scheme SCMessenger \
        -destination "platform=iOS,id=$device_id" \
        -quiet 2>&1 | tail -n 5

    log_info "✓ iOS build complete — install via Xcode or Finder"
}

# Parse arguments
TARGET="${1:-}"

case "$TARGET" in
    android)
        deploy_android
        ;;
    ios)
        deploy_ios
        ;;
    both)
        deploy_android
        deploy_ios
        ;;
    *)
        echo "Usage: $0 {android|ios|both}"
        echo ""
        echo "Performs a CLEAN build before deploying to prevent stale APK crashes."
        echo "This script was created after diagnosing a NoClassDefFoundError crash"
        echo "caused by Hilt-generated code missing from a non-clean build."
        exit 1
        ;;
esac
