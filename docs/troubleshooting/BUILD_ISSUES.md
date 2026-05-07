# Build Issues Troubleshooting Guide

Status: Active  
Last updated: 2026-03-07

This guide covers common build issues and their solutions across all SCMessenger platforms.

## Table of Contents

- [General Build Issues](#general-build-issues)
- [Rust Core Issues](#rust-core-issues)
- [Android Build Issues](#android-build-issues)
- [iOS Build Issues](#ios-build-issues)
- [WASM Build Issues](#wasm-build-issues)
- [CLI Build Issues](#cli-build-issues)
- [Dependency Issues](#dependency-issues)
- [Platform-Specific Issues](#platform-specific-issues)

## General Build Issues

### Issue: "cargo: command not found"

**Symptoms:**
```
bash: cargo: command not found
```

**Solution:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell
source $HOME/.cargo/env

# Verify installation
cargo --version
```

### Issue: "error: toolchain '1.75.0' is not installed"

**Symptoms:**
```
error: toolchain '1.75.0-x86_64-unknown-linux-gnu' is not installed
```

**Solution:**
```bash
# Install specific toolchain
rustup toolchain install 1.75.0

# Or update to latest
rustup update

# Verify
rustc --version
```

### Issue: "error: failed to download"

**Symptoms:**
```
error: failed to download from `https://crates.io/...`
```

**Solution:**
```bash
# Check internet connection
ping crates.io

# Clear cargo cache
rm -rf ~/.cargo/registry
rm -rf ~/.cargo/git

# Retry build
cargo clean
cargo build
```

### Issue: "error: could not compile due to previous error"

**Symptoms:**
```
error: could not compile `scmessenger-core` due to 5 previous errors
```

**Solution:**
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build --workspace

# If still failing, check specific error messages
cargo build --workspace 2>&1 | tee build.log
```

## Rust Core Issues

### Issue: "error: linker `cc` not found"

**Platform**: Linux

**Symptoms:**
```
error: linker `cc` not found
```

**Solution:**
```bash
# Debian/Ubuntu
sudo apt-get update
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc

# Arch Linux
sudo pacman -S base-devel
```

### Issue: "error: failed to run custom build command for `openssl-sys`"

**Platform**: Linux

**Symptoms:**
```
error: failed to run custom build command for `openssl-sys v0.9.XX`
```

**Solution:**
```bash
# Debian/Ubuntu
sudo apt-get install pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install pkg-config openssl-devel

# Arch Linux
sudo pacman -S pkg-config openssl
```

### Issue: "error: linking with `link.exe` failed"

**Platform**: Windows

**Symptoms:**
```
error: linking with `link.exe` failed: exit code: 1120
```

**Solution:**
1. Install Visual Studio 2019 or later with "Desktop development with C++" workload
2. Or install Build Tools for Visual Studio
3. Restart terminal after installation
4. Verify: `cl.exe` should be in PATH

### Issue: "error: could not find `Cargo.toml`"

**Symptoms:**
```
error: could not find `Cargo.toml` in `/path/to/dir` or any parent directory
```

**Solution:**
```bash
# Ensure you're in the correct directory
cd /path/to/SCMessenger

# Verify Cargo.toml exists
ls -la Cargo.toml

# If missing, re-clone repository
git clone https://github.com/Treystu/SCMessenger.git
```

## Android Build Issues

### Issue: "ANDROID_HOME is not set"

**Symptoms:**
```
Error: ANDROID_HOME environment variable is not set
```

**Solution:**
```bash
# Find Android SDK location
# Linux/macOS: ~/Android/Sdk or ~/Library/Android/sdk
# Windows: %LOCALAPPDATA%\Android\Sdk

# Set environment variable
export ANDROID_HOME=$HOME/Android/Sdk  # Linux
export ANDROID_HOME=$HOME/Library/Android/sdk  # macOS
# Windows: set ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk

# Add to shell profile
echo 'export ANDROID_HOME=$HOME/Android/Sdk' >> ~/.bashrc
source ~/.bashrc
```

### Issue: "NDK not found"

**Symptoms:**
```
Error: NDK (Side by side) 26.3.11579264 is not installed
```

**Solution:**
1. Open Android Studio
2. Tools → SDK Manager → SDK Tools
3. Check "NDK (Side by side)"
4. Select version 26.3.11579264
5. Click "Apply" to install

Or via command line:
```bash
sdkmanager --install "ndk;26.3.11579264"
```

### Issue: "Gradle sync failed"

**Symptoms:**
```
Could not resolve all dependencies for configuration ':app:debugCompileClasspath'
```

**Solution:**
```bash
cd android

# Clean build
./gradlew clean

# Invalidate caches (if using Android Studio)
# File → Invalidate Caches / Restart

# Retry sync
./gradlew build --refresh-dependencies
```

### Issue: "Rust library not found"

**Symptoms:**
```
java.lang.UnsatisfiedLinkError: dlopen failed: library "libscmessenger_mobile.so" not found
```

**Solution:**
```bash
# Rebuild Rust core for Android
rustup target add aarch64-linux-android
cargo build --release --target aarch64-linux-android

# Copy libraries to jniLibs
mkdir -p android/app/src/main/jniLibs/arm64-v8a
cp target/aarch64-linux-android/release/libscmessenger_mobile.so \
   android/app/src/main/jniLibs/arm64-v8a/

# Rebuild Android app
cd android
./gradlew clean assembleDebug
```

## iOS Build Issues

### Issue: "xcode-select: error: tool 'xcodebuild' requires Xcode"

**Platform**: macOS

**Symptoms:**
```
xcode-select: error: tool 'xcodebuild' requires Xcode, but active developer directory '/Library/Developer/CommandLineTools' is a command line tools instance
```

**Solution:**
```bash
# Install Xcode from App Store
# Or download from https://developer.apple.com/xcode/

# Set Xcode path
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer

# Verify
xcode-select -p
```

### Issue: "CocoaPods not found"

**Symptoms:**
```
-bash: pod: command not found
```

**Solution:**
```bash
# Install CocoaPods
sudo gem install cocoapods

# Verify installation
pod --version

# If gem install fails, try:
brew install cocoapods
```

### Issue: "pod install failed"

**Symptoms:**
```
[!] Unable to find a specification for `SomeLibrary`
```

**Solution:**
```bash
cd iOS

# Update CocoaPods repo
pod repo update

# Clean and reinstall
rm -rf Pods Podfile.lock
pod install

# If still failing, try:
pod install --repo-update
```

### Issue: "Code signing failed"

**Symptoms:**
```
error: Signing for "SCMessenger" requires a development team
```

**Solution:**
1. Open Xcode
2. Select project in navigator
3. Select target "SCMessenger"
4. Go to "Signing & Capabilities"
5. Select your development team
6. Enable "Automatically manage signing"

### Issue: "Rust library not found"

**Symptoms:**
```
ld: library not found for -lscmessenger_mobile
```

**Solution:**
```bash
# Rebuild Rust core for iOS
rustup target add aarch64-apple-ios
cargo build --release --target aarch64-apple-ios

# Verify library exists
ls -la target/aarch64-apple-ios/release/libscmessenger_mobile.a

# Clean and rebuild iOS app
cd iOS
rm -rf Build DerivedData
xcodebuild clean
xcodebuild build -workspace SCMessenger.xcworkspace -scheme SCMessenger
```

## WASM Build Issues

### Issue: "wasm-pack: command not found"

**Symptoms:**
```
bash: wasm-pack: command not found
```

**Solution:**
```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Verify installation
wasm-pack --version
```

### Issue: "error: target 'wasm32-unknown-unknown' not found"

**Symptoms:**
```
error: target 'wasm32-unknown-unknown' not found
```

**Solution:**
```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Verify target
rustup target list | grep wasm32
```

### Issue: "wasm-opt: command not found"

**Symptoms:**
```
bash: wasm-opt: command not found
```

**Solution:**
```bash
# macOS
brew install binaryen

# Linux (Debian/Ubuntu)
sudo apt-get install binaryen

# Windows (via npm)
npm install -g binaryen

# Verify
wasm-opt --version
```

### Issue: "Module parse failed"

**Symptoms:**
```
Module parse failed: Unexpected character '@'
```

**Solution:**
```javascript
// Ensure correct import syntax for your bundler

// Webpack 5+
import init from './pkg/scmessenger_wasm.js';

// Vite
import init from './pkg/scmessenger_wasm.js?init';

// Or use dynamic import
const wasm = await import('./pkg/scmessenger_wasm.js');
```

## CLI Build Issues

### Issue: "Permission denied" when running binary

**Platform**: Linux/macOS

**Symptoms:**
```
bash: ./target/release/scmessenger-cli: Permission denied
```

**Solution:**
```bash
# Make binary executable
chmod +x target/release/scmessenger-cli

# Or run with cargo
cargo run --release --bin scmessenger-cli
```

### Issue: Binary size is too large

**Symptoms:**
```
target/release/scmessenger-cli is 50MB+
```

**Solution:**
```bash
# Strip debug symbols
strip target/release/scmessenger-cli

# Or build with optimized profile
cargo build --release --bin scmessenger-cli

# Check size
ls -lh target/release/scmessenger-cli

# For even smaller binaries, use:
cargo build --release --bin scmessenger-cli
strip target/release/scmessenger-cli
upx --best target/release/scmessenger-cli  # Optional, requires UPX
```

## Dependency Issues

### Issue: "error: failed to select a version"

**Symptoms:**
```
error: failed to select a version for `some-crate`
```

**Solution:**
```bash
# Update Cargo.lock
cargo update

# Or update specific dependency
cargo update -p some-crate

# If still failing, clean and rebuild
cargo clean
cargo build
```

### Issue: "error: multiple packages link to native library"

**Symptoms:**
```
error: multiple packages link to native library `openssl`
```

**Solution:**
```bash
# Check dependency tree
cargo tree | grep openssl

# Update conflicting dependencies
cargo update

# Or specify exact version in Cargo.toml
[dependencies]
openssl = "=0.10.XX"
```

### Issue: "error: package requires rustc 1.XX.0 or newer"

**Symptoms:**
```
error: package `some-crate v1.0.0` cannot be built because it requires rustc 1.75.0 or newer
```

**Solution:**
```bash
# Update Rust toolchain
rustup update

# Or install specific version
rustup install 1.75.0
rustup default 1.75.0

# Verify
rustc --version
```

## Platform-Specific Issues

### Linux: "error: failed to load source for dependency"

**Solution:**
```bash
# Install git
sudo apt-get install git

# Clear cargo cache
rm -rf ~/.cargo/registry
rm -rf ~/.cargo/git

# Retry
cargo build
```

### macOS: "xcrun: error: invalid active developer path"

**Solution:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Verify
xcode-select -p
```

### Windows: "error: could not find system library 'advapi32'"

**Solution:**
1. Install Visual Studio with "Desktop development with C++"
2. Or install Windows SDK
3. Restart terminal
4. Retry build

## Getting More Help

If your issue isn't listed here:

1. **Check logs**: Look for specific error messages
2. **Search issues**: https://github.com/Treystu/SCMessenger/issues
3. **Ask for help**: https://github.com/Treystu/SCMessenger/discussions
4. **Read docs**: Check platform-specific setup guides in `docs/platform/`

### Useful Commands for Debugging

```bash
# Show detailed error messages
cargo build --verbose

# Show dependency tree
cargo tree

# Check for outdated dependencies
cargo outdated

# Audit dependencies for security issues
cargo audit

# Clean all build artifacts
cargo clean

# Update all dependencies
cargo update
```

---

**Related Guides:**
- [CI Failures Guide](CI_FAILURES.md)
- [Runtime Issues Guide](RUNTIME_ISSUES.md)
- [Android Setup Guide](../platform/ANDROID_SETUP.md)
- [iOS Setup Guide](../platform/IOS_SETUP.md)
- [WASM Setup Guide](../platform/WASM_SETUP.md)
- [CLI Setup Guide](../platform/CLI_SETUP.md)
