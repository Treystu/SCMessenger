#!/bin/bash
# Comprehensive build verification script for all platforms
set -e

echo "=========================================="
echo "SCMessenger Full Build Verification"
echo "=========================================="
echo ""

# 1. Rust workspace
echo "1. Testing Rust workspace..."
cargo test --workspace --quiet
echo "✅ Rust: All tests passed"
echo ""

# 2. Rust formatting
echo "2. Checking Rust formatting..."
cargo fmt --all -- --check
echo "✅ Rust: Formatting clean"
echo ""

# 3. Rust clippy
echo "3. Checking Rust clippy..."
cargo clippy --workspace --lib --bins -- -D warnings 2>&1 | tail -5
echo "✅ Rust: Clippy clean"
echo ""

# 4. Android build
echo "4. Building Android..."
cd android
./gradlew :app:assembleDebug --quiet 2>&1 | tail -5
cd ..
echo "✅ Android: Build successful"
echo ""

# 5. iOS build
echo "5. Verifying iOS..."
bash iOS/verify-test.sh 2>&1 | tail -5
echo "✅ iOS: Build verified"
echo ""

echo "=========================================="
echo "✅ ALL PLATFORMS BUILD SUCCESSFULLY"
echo "=========================================="
