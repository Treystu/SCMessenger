#!/bin/bash
# Android Build Verification Script
# Checks prerequisites and verifies the build setup

echo "=== SCMessenger Android Build Verification ==="
echo

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check functions
check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 found"
        return 0
    else
        echo -e "${RED}✗${NC} $1 not found"
        return 1
    fi
}

check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} $1 exists"
        return 0
    else
        echo -e "${RED}✗${NC} $1 missing"
        return 1
    fi
}

check_dir() {
    if [ -d "$1" ]; then
        echo -e "${GREEN}✓${NC} $1 exists"
        return 0
    else
        echo -e "${RED}✗${NC} $1 missing"
        return 1
    fi
}

# Track status
ALL_OK=true

# 1. Check Rust toolchain
echo "1. Checking Rust toolchain..."
if check_command rustc; then
    RUST_VERSION=$(rustc --version)
    echo "   Version: $RUST_VERSION"
else
    ALL_OK=false
    echo -e "   ${YELLOW}Install from: https://rustup.rs${NC}"
fi

if check_command cargo; then
    CARGO_VERSION=$(cargo --version)
    echo "   Version: $CARGO_VERSION"
else
    ALL_OK=false
fi
echo

# 2. Check cargo-ndk
echo "2. Checking cargo-ndk..."
if check_command cargo-ndk; then
    CARGO_NDK_VERSION=$(cargo-ndk --version 2>&1 || echo "version check failed")
    echo "   Version: $CARGO_NDK_VERSION"
else
    ALL_OK=false
    echo -e "   ${YELLOW}Install: cargo install cargo-ndk${NC}"
fi
echo

# 3. Check Android Rust targets
echo "3. Checking Android Rust targets..."
TARGETS_OK=true
for target in aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android; do
    if rustup target list | grep -q "$target (installed)"; then
        echo -e "   ${GREEN}✓${NC} $target installed"
    else
        echo -e "   ${RED}✗${NC} $target not installed"
        TARGETS_OK=false
    fi
done
if [ "$TARGETS_OK" = false ]; then
    ALL_OK=false
    echo -e "   ${YELLOW}Install: rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android${NC}"
fi
echo

# 4. Check Java/JDK
echo "4. Checking Java..."
if check_command java; then
    JAVA_VERSION=$(java -version 2>&1 | head -n 1)
    echo "   Version: $JAVA_VERSION"
else
    ALL_OK=false
    echo -e "   ${YELLOW}Java 17+ required${NC}"
fi
echo

# 5. Check ANDROID_HOME
echo "5. Checking ANDROID_HOME..."
if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME" ]; then
    echo -e "${GREEN}✓${NC} ANDROID_HOME set: $ANDROID_HOME"
    
    # Check NDK
    if [ -d "$ANDROID_HOME/ndk/26.1.10909125" ]; then
        echo -e "   ${GREEN}✓${NC} NDK 26.1.10909125 installed"
    else
        echo -e "   ${YELLOW}⚠${NC} NDK 26.1.10909125 not found (Android Studio will download)"
    fi
else
    echo -e "${RED}✗${NC} ANDROID_HOME not set or invalid"
    echo "   Set ANDROID_HOME to your Android SDK location"
    ALL_OK=false
fi
echo

# 6. Check project structure
echo "6. Checking project structure..."
cd "$(dirname "$0")/.."
check_file "core/src/api.udl" || ALL_OK=false
check_file "core/src/bin/gen_kotlin.rs" || ALL_OK=false
check_file "android/app/build.gradle" || ALL_OK=false
check_file "android/gradlew" || ALL_OK=false
echo

# 7. Test bindings generation
echo "7. Testing UniFFI bindings generation..."
cd core
OUTPUT=$(cargo run --bin gen_kotlin --features gen-bindings 2>&1)
CARGO_STATUS=$?
if [ $CARGO_STATUS -eq 0 ]; then
    if [ -f "target/generated-sources/uniffi/kotlin/uniffi/api/api.kt" ]; then
        FILE_SIZE=$(stat -f%z "target/generated-sources/uniffi/kotlin/uniffi/api/api.kt" 2>/dev/null || stat -c%s "target/generated-sources/uniffi/kotlin/uniffi/api/api.kt" 2>/dev/null)
        echo -e "${GREEN}✓${NC} Bindings generated successfully ($FILE_SIZE bytes)"
    else
        echo -e "${RED}✗${NC} Bindings generation failed - output file not found"
        echo "   Cargo output:"
        echo "$OUTPUT" | sed 's/^/   /'
        ALL_OK=false
    fi
else
    echo -e "${RED}✗${NC} Bindings generation failed"
    echo "   Cargo output:"
    echo "$OUTPUT" | sed 's/^/   /'
    ALL_OK=false
fi
cd ..
echo

# Final summary
echo "=== Summary ==="
if [ "$ALL_OK" = true ]; then
    echo -e "${GREEN}✓ All checks passed! Android build should work.${NC}"
    echo
    echo "To build:"
    echo "  cd android"
    echo "  ./gradlew assembleDebug"
    exit 0
else
    echo -e "${RED}✗ Some checks failed. Fix the issues above before building.${NC}"
    exit 1
fi
