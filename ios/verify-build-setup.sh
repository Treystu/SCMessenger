#!/bin/bash
# iOS Build Verification Script
# Checks prerequisites and verifies the build setup

echo "=== SCMessenger iOS Build Verification ==="
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

# 2. Check iOS Rust targets
echo "2. Checking iOS Rust targets..."
TARGETS_OK=true
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    if rustup target list | grep -q "^$target (installed)"; then
        echo -e "${GREEN}✓${NC} $target installed"
    else
        echo -e "${RED}✗${NC} $target not installed"
        TARGETS_OK=false
    fi
done

if [ "$TARGETS_OK" = false ]; then
    ALL_OK=false
    echo -e "   ${YELLOW}Install targets:${NC}"
    echo "   rustup target add aarch64-apple-ios"
    echo "   rustup target add aarch64-apple-ios-sim"
fi
echo

# 3. Check Xcode command line tools
echo "3. Checking Xcode command line tools..."
if check_command xcode-select; then
    XCODE_PATH=$(xcode-select -p 2>/dev/null || echo "not configured")
    if [ "$XCODE_PATH" != "not configured" ]; then
        echo -e "${GREEN}✓${NC} Xcode CLI tools configured at: $XCODE_PATH"
    else
        echo -e "${RED}✗${NC} Xcode CLI tools not configured"
        ALL_OK=false
        echo -e "   ${YELLOW}Install: xcode-select --install${NC}"
    fi
else
    echo -e "${RED}✗${NC} xcode-select not found (Xcode not installed?)"
    ALL_OK=false
    echo -e "   ${YELLOW}Install Xcode from the App Store${NC}"
fi
echo

# 4. Check project structure
echo "4. Checking project structure..."
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT" || exit 1

check_file "core/src/api.udl"
check_file "core/src/bin/gen_swift.rs"
check_dir "core/src/mobile"
check_file "core/src/mobile/ios_strategy.rs"
echo

# 5. Check gen_swift binary
echo "5. Testing gen_swift binary..."
cd "$PROJECT_ROOT/core" || exit 1
if cargo run --bin gen_swift --features gen-bindings > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} gen_swift binary runs successfully"
    
    # Check generated outputs
    if check_file "target/generated-sources/uniffi/swift/api.swift"; then
        SWIFT_LINES=$(wc -l < "target/generated-sources/uniffi/swift/api.swift")
        echo "   Generated api.swift: $SWIFT_LINES lines"
    else
        ALL_OK=false
    fi
    
    if check_file "target/generated-sources/uniffi/swift/apiFFI.h"; then
        echo -e "${GREEN}✓${NC} apiFFI.h generated"
    else
        ALL_OK=false
    fi
    
    if check_file "target/generated-sources/uniffi/swift/apiFFI.modulemap"; then
        echo -e "${GREEN}✓${NC} apiFFI.modulemap generated"
    else
        ALL_OK=false
    fi
else
    echo -e "${RED}✗${NC} gen_swift binary failed to run"
    ALL_OK=false
    echo -e "   ${YELLOW}Check cargo output above for errors${NC}"
fi
echo

# 6. Test static library compilation (optional, can be slow)
echo "6. Testing static library compilation..."
echo -e "${YELLOW}Note: This step can take several minutes...${NC}"

cd "$PROJECT_ROOT/mobile" || exit 1
if cargo build --target aarch64-apple-ios --lib 2>&1 | tail -5; then
    if [ -f "target/aarch64-apple-ios/debug/libscmessenger_mobile.a" ]; then
        echo -e "${GREEN}✓${NC} Static library compiled for aarch64-apple-ios"
        LIB_SIZE=$(du -h "target/aarch64-apple-ios/debug/libscmessenger_mobile.a" | cut -f1)
        echo "   Library size: $LIB_SIZE"
    else
        echo -e "${RED}✗${NC} Static library not found"
        ALL_OK=false
    fi
else
    echo -e "${RED}✗${NC} Static library compilation failed"
    ALL_OK=false
fi
echo

# Summary
echo "=== Summary ==="
if [ "$ALL_OK" = true ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo
    echo "You're ready to build the iOS app."
    echo
    echo "Next steps:"
    echo "  1. Run: ./ios/copy-bindings.sh"
    echo "  2. Open Xcode project in ios/"
    echo "  3. Build and run on device or simulator"
    exit 0
else
    echo -e "${RED}✗ Some checks failed.${NC}"
    echo
    echo "Please fix the issues above and run this script again."
    exit 1
fi
