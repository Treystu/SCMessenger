#!/usr/bin/env bash
# =============================================================================
# scripts/build_appimage.sh — Build SCMessenger Desktop AppImage
#
# Produces: build/distributions/SCMessenger-Desktop-<version>-x86_64.AppImage
#
# Prerequisites:
#   - appimagetool in PATH (or at /usr/local/bin/appimagetool)
#   - libfuse2 installed
#   - shared/build/jpackager/scmessenger-desktop/ directory exists (from Gradle jpackage task)
#
# Environment variables:
#   APP_VERSION    — Version string (default: from Cargo.toml)
#   APP_NAME       — Application name (default: SCMessenger-Desktop)
# =============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

APP_VERSION="${APP_VERSION:-$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')}"
APP_NAME="${APP_NAME:-SCMessenger-Desktop}"
ARCH="x86_64"

echo -e "${GREEN}Building AppImage for ${APP_NAME} v${APP_VERSION} (${ARCH})${NC}"

# ---- Verify prerequisites ----
if ! command -v appimagetool &>/dev/null; then
    echo -e "${YELLOW}appimagetool not found, downloading...${NC}"
    wget -q "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage" \
        -O /tmp/appimagetool
    chmod +x /tmp/appimagetool
    sudo mv /tmp/appimagetool /usr/local/bin/appimagetool 2>/dev/null || \
        cp /tmp/appimagetool ~/.local/bin/appimagetool
fi

# ---- Locate jpackage output ----
JPACKAGE_DIR="$REPO_ROOT/shared/build/jpackage/scmessenger-desktop"
if [ ! -d "$JPACKAGE_DIR" ]; then
    echo -e "${YELLOW}jpackage output not found at ${JPACKAGE_DIR}, building first...${NC}"
    cd "$REPO_ROOT/shared"
    ./gradlew jpackage --no-daemon
    cd "$REPO_ROOT"
fi

if [ ! -d "$JPACKAGE_DIR" ]; then
    echo -e "${RED}ERROR: jpackage output directory not found: ${JPACKAGE_DIR}${NC}"
    exit 1
fi

# ---- Prepare AppDir ----
APPDIR="$REPO_ROOT/build/AppImage"
rm -rf "$APPDIR"
mkdir -p "$APPDIR"

# Copy jpackage output into AppDir structure
echo "Preparing AppDir..."
cp -r "$JPACKAGE_DIR"/* "$APPDIR/"

# Bundle Rust .so libraries
RUST_SO_DIR="$REPO_ROOT/target/release"
if [ -d "$RUST_SO_DIR" ]; then
    mkdir -p "$APPDIR/lib"
    cp "$RUST_SO_DIR"/*.so "$APPDIR/lib/" 2>/dev/null || true
fi

# Write AppRun script
cat > "$APPDIR/AppRun" << 'APPRUN'
#!/bin/bash
SELF_DIR="$(dirname "$(readlink -f "$0")")"
export LD_LIBRARY_PATH="${SELF_DIR}/lib:${LD_LIBRARY_PATH:-}"
exec "${SELF_DIR}/bin/scmessenger-desktop" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"

# Write .desktop file
cat > "$APPDIR/scmessenger-desktop.desktop" << DESKTOP
[Desktop Entry]
Type=Application
Name=SCMessenger
Comment=Secure P2P Messaging
Exec=scmessenger-desktop
Icon=scmessenger-desktop
Categories=Network;Chat;InstantMessaging;
Terminal=false
StartupWMClass=scmessenger-desktop
DESKTOP

# Copy icon if available
ICON_SRC=""
for candidate in \
    "$REPO_ROOT/shared/src/linuxX64Main/resources/icon.png" \
    "$REPO_ROOT/shared/src/commonMain/resources/icon.png" \
    "$REPO_ROOT/dist/icon.png"; do
    if [ -f "$candidate" ]; then
        ICON_SRC="$candidate"
        break
    fi
done

if [ -n "$ICON_SRC" ]; then
    cp "$ICON_SRC" "$APPDIR/scmessenger-desktop.png"
fi

# ---- Build AppImage ----
OUTPUT_FILE="$REPO_ROOT/build/distributions/${APP_NAME}-${APP_VERSION}-${ARCH}.AppImage"
mkdir -p "$(dirname "$OUTPUT_FILE")"

echo "Running appimagetool..."
export ARCH="$ARCH"
export VERSION="$APP_VERSION"

appimagetool \
    --comp xz \
    "$APPDIR" \
    "$OUTPUT_FILE"

echo -e "${GREEN} AppImage built: ${OUTPUT_FILE}${NC}"
ls -lh "$OUTPUT_FILE"
