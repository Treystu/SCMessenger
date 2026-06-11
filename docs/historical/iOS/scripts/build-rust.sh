#!/bin/bash
# Xcode Build Phase Script
# Compiles Rust code for the appropriate iOS target
set -e

# Navigate to mobile crate
cd "${SRCROOT}/../mobile"

# Determine target architecture based on platform
if [ "$PLATFORM_NAME" = "iphonesimulator" ]; then
    # Simulator build
    if [ "$(uname -m)" = "arm64" ]; then
        # Apple Silicon Mac running simulator
        RUST_TARGET="aarch64-apple-ios-sim"
    else
        # Intel Mac running simulator
        RUST_TARGET="x86_64-apple-ios"
    fi
else
    # Device build
    RUST_TARGET="aarch64-apple-ios"
fi

# Determine build profile
if [ "$CONFIGURATION" = "Release" ]; then
    RUST_PROFILE="--release"
    RUST_DIR="release"
else
    RUST_PROFILE=""
    RUST_DIR="debug"
fi

echo "Building Rust for target: $RUST_TARGET ($CONFIGURATION)"

# Build the Rust library
cargo build $RUST_PROFILE --target "$RUST_TARGET"

# Copy library to Xcode's expected location
mkdir -p "${CONFIGURATION_BUILD_DIR}"
cp "target/${RUST_TARGET}/${RUST_DIR}/libscmessenger_mobile.a" \
   "${CONFIGURATION_BUILD_DIR}/libscmessenger_mobile.a"

echo "Rust library copied to: ${CONFIGURATION_BUILD_DIR}/libscmessenger_mobile.a"
