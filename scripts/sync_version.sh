#!/usr/bin/env bash
# Version synchronization script
# Reads version from Cargo.toml and updates all platform manifests
# Validates: Requirements 8.1, 8.2, 8.3, 8.4

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "🔄 Synchronizing version across all platforms..."

# Extract version from workspace Cargo.toml
VERSION=$(grep -m 1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

if [ -z "$VERSION" ]; then
    echo -e "${RED}❌ Failed to extract version from Cargo.toml${NC}"
    exit 1
fi

echo -e "${GREEN}📦 Version: $VERSION${NC}"

# Parse semantic version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$VERSION"

# Calculate Android versionCode (MAJOR * 10000 + MINOR * 100 + PATCH)
VERSION_CODE=$((MAJOR * 10000 + MINOR * 100 + PATCH))

echo "  - Major: $MAJOR"
echo "  - Minor: $MINOR"
echo "  - Patch: $PATCH"
echo "  - Android versionCode: $VERSION_CODE"

# Update Android build.gradle
if [ -f "android/app/build.gradle" ]; then
    echo "📱 Updating Android version..."
    
    # Update versionName
    if grep -q 'versionName' android/app/build.gradle; then
        sed -i.bak "s/versionName \"[^\"]*\"/versionName \"$VERSION\"/" android/app/build.gradle
        echo -e "${GREEN}  ✓ Updated versionName to $VERSION${NC}"
    else
        echo -e "${YELLOW}  ⚠ versionName not found in build.gradle${NC}"
    fi
    
    # Update versionCode
    if grep -q 'versionCode' android/app/build.gradle; then
        sed -i.bak "s/versionCode [0-9]*/versionCode $VERSION_CODE/" android/app/build.gradle
        echo -e "${GREEN}  ✓ Updated versionCode to $VERSION_CODE${NC}"
    else
        echo -e "${YELLOW}  ⚠ versionCode not found in build.gradle${NC}"
    fi
    
    # Remove backup file
    rm -f android/app/build.gradle.bak
else
    echo -e "${YELLOW}  ⚠ android/app/build.gradle not found${NC}"
fi

# Update iOS Info.plist
if [ -f "iOS/SCMessenger/Info.plist" ]; then
    echo "🍎 Updating iOS version..."
    
    # Update CFBundleShortVersionString (user-visible version)
    if /usr/libexec/PlistBuddy -c "Print :CFBundleShortVersionString" iOS/SCMessenger/Info.plist &>/dev/null; then
        /usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" iOS/SCMessenger/Info.plist
        echo -e "${GREEN}  ✓ Updated CFBundleShortVersionString to $VERSION${NC}"
    else
        echo -e "${YELLOW}  ⚠ CFBundleShortVersionString not found in Info.plist${NC}"
    fi
    
    # Update CFBundleVersion (build number - use versionCode for consistency)
    if /usr/libexec/PlistBuddy -c "Print :CFBundleVersion" iOS/SCMessenger/Info.plist &>/dev/null; then
        /usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION_CODE" iOS/SCMessenger/Info.plist
        echo -e "${GREEN}  ✓ Updated CFBundleVersion to $VERSION_CODE${NC}"
    else
        echo -e "${YELLOW}  ⚠ CFBundleVersion not found in Info.plist${NC}"
    fi
else
    echo -e "${YELLOW}  ⚠ iOS/SCMessenger/Info.plist not found (macOS required for PlistBuddy)${NC}"
fi

# Update WASM package.json
if [ -f "wasm/package.json" ]; then
    echo "🌐 Updating WASM version..."
    
    if grep -q '"version"' wasm/package.json; then
        sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" wasm/package.json
        echo -e "${GREEN}  ✓ Updated package.json version to $VERSION${NC}"
        rm -f wasm/package.json.bak
    else
        echo -e "${YELLOW}  ⚠ version field not found in package.json${NC}"
    fi
else
    echo -e "${YELLOW}  ⚠ wasm/package.json not found${NC}"
fi

echo ""
echo -e "${GREEN}✅ Version synchronization complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git add -A && git commit -m \"chore: bump version to $VERSION\""
echo "  3. Tag: git tag v$VERSION"
echo "  4. Push: git push origin main --tags"
