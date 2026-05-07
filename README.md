# SCMessenger

**Status**: Active  
**Last updated**: 2026-03-07  
**Version**: v0.2.1 (alpha)

**SCMessenger** (Sovereign Encrypted Messaging) is a highly resilient, cross-platform decentralized messaging mesh handling secure communications. Built with cutting-edge peer-to-peer technologies, it bypasses traditional central servers, utilizing BLE, mDNS, LAN, and Quic/TCP Relay circuits to ensure instantaneous delivery under any network condition.

## Overview

SCMessenger is architected for total sovereignty and uncompromised privacy. It features:
- **Rust Core**: A highly secure, multi-transport headless mesh daemon using `libp2p`.
- **WASM Thin-Client Web UI**: A stunning browser-based interface running over a strict `localhost` multiplexed JSON-RPC WebSocket Bridge to perfectly sandbox cryptographic isolation.
- **Native Android & iOS Clients**: Full-featured smart transport routers that elegantly fallback through Multipeer, Wi-Fi Direct, BLE, mDNS, and Internet Relay based on real-time sub-500ms connectivity races.
- **Resilient Transport Matrix**: Automatic Transport path determination delivering messages whether on an airplane, in a crowded stadium, or on a cellular network traversing strict NATs using resilient UDP/QUIC forwarding.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Platform-Specific Build Instructions](#platform-specific-build-instructions)
- [Testing](#testing)
- [Architecture & Documentation](#architecture--documentation)
- [Contributing](#contributing)
- [Security](#security)
- [License](#license)

## Prerequisites

### All Platforms

- **Rust** 1.75.0 or later (see `rust-toolchain.toml`)
- **Git** 2.30+
- **Cargo** (comes with Rust)

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Platform-Specific Prerequisites

- **Android**: Android SDK, NDK r26b, Java 17+ - See [Android Setup Guide](docs/platform/ANDROID_SETUP.md)
- **iOS**: macOS, Xcode 15+, CocoaPods - See [iOS Setup Guide](docs/platform/IOS_SETUP.md)
- **WASM**: Node.js 20+, wasm-pack - See [WASM Setup Guide](docs/platform/WASM_SETUP.md)
- **CLI**: No additional requirements - See [CLI Setup Guide](docs/platform/CLI_SETUP.md)

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger
```

### 2. Build and Test Core

```bash
# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run formatting and linting
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

### 3. Run the CLI

```bash
# Build and run the CLI
cargo run --release --bin scmessenger-cli -- start
```

The daemon binds to `localhost:9002` for security.

## Platform-Specific Build Instructions

### CLI (Linux, macOS, Windows)

```bash
# Build release binary
cargo build --release --bin scmessenger-cli

# Binary location
# Linux/macOS: target/release/scmessenger-cli
# Windows: target/release/scmessenger-cli.exe

# Run
./target/release/scmessenger-cli --help
```

**Platform-specific notes:**
- **Linux**: Requires `build-essential`, `pkg-config`, `libssl-dev`
- **macOS**: Xcode Command Line Tools required
- **Windows**: MSVC build tools required

See [CLI Setup Guide](docs/platform/CLI_SETUP.md) for detailed instructions.

### Android

```bash
# Add Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Build Rust core for Android
cargo build --release --target aarch64-linux-android

# Build Android app
cd android
./gradlew assembleDebug

# Install on device
./gradlew installDebug
```

**Requirements:**
- Android SDK (API 26+)
- NDK r26b
- Java 17+

See [Android Setup Guide](docs/platform/ANDROID_SETUP.md) for detailed instructions.

### iOS

```bash
# Add iOS targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios

# Build Rust core for iOS
cargo build --release --target aarch64-apple-ios

# Install CocoaPods dependencies
cd iOS
pod install

# Open in Xcode
open SCMessenger.xcworkspace
```

**Requirements:**
- macOS 13.0+
- Xcode 15.0+
- CocoaPods

See [iOS Setup Guide](docs/platform/IOS_SETUP.md) for detailed instructions.

### WASM

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Build WASM package
cd wasm
wasm-pack build --target web --release

# Optimize (optional)
wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm
```

**Requirements:**
- Node.js 20+
- wasm-pack
- wasm-opt (optional, for optimization)

See [WASM Setup Guide](docs/platform/WASM_SETUP.md) for detailed instructions.

## Testing

### Run All Tests

```bash
# Run all workspace tests
cargo test --workspace

# Run with logging
RUST_LOG=debug cargo test --workspace -- --nocapture

# Run specific platform tests
cargo test -p scmessenger-core
cargo test -p scmessenger-mobile
cargo test -p scmessenger-wasm
```

### Platform-Specific Tests

```bash
# Android
cd android && ./gradlew test

# iOS
cd iOS && xcodebuild test -workspace SCMessenger.xcworkspace -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 15'

# WASM
cd wasm && wasm-pack test --headless --firefox
```

### Integration Tests

```bash
# Run integration test suite
cargo test --workspace --test '*'

# Run specific integration test
cargo test -p scmessenger-core --test integration_offline_partition_matrix
```

See [Testing Guide](docs/TESTING_GUIDE.md) for comprehensive testing documentation.

## Architecture & Documentation

### Core Documentation

- **[Architecture Overview](docs/ARCHITECTURE.md)** - System architecture and design
- **[Current State](docs/CURRENT_STATE.md)** - Implementation status and roadmap
- **[Testing Guide](docs/TESTING_GUIDE.md)** - Testing strategy and practices
- **[Deployment Guide](docs/DEPLOYMENT.md)** - Release and deployment procedures

### Platform Guides

- **[Android Setup](docs/platform/ANDROID_SETUP.md)** - Android development setup
- **[iOS Setup](docs/platform/IOS_SETUP.md)** - iOS development setup
- **[WASM Setup](docs/platform/WASM_SETUP.md)** - WASM development setup
- **[CLI Setup](docs/platform/CLI_SETUP.md)** - CLI development setup

### Troubleshooting

- **[Build Issues](docs/troubleshooting/BUILD_ISSUES.md)** - Common build problems and solutions
- **[CI Failures](docs/troubleshooting/CI_FAILURES.md)** - CI debugging guide
- **[Runtime Issues](docs/troubleshooting/RUNTIME_ISSUES.md)** - Runtime debugging guide

## Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) for details on:

- Development setup
- Code style guidelines
- Commit message format
- Pull request process
- Testing requirements

### Quick Contribution Checklist

- [ ] Fork the repository
- [ ] Create a feature branch (`git checkout -b feat/my-feature`)
- [ ] Make your changes
- [ ] Run tests (`cargo test --workspace`)
- [ ] Run linters (`cargo fmt --all && cargo clippy --workspace`)
- [ ] Commit with conventional commits (`git commit -m "feat: add feature"`)
- [ ] Push and create a pull request

## Security

Security is a top priority for SCMessenger. If you discover a security vulnerability:

- **DO NOT** open a public GitHub issue
- Report privately via GitHub Security Advisories or email
- See [Security Policy](SECURITY.md) for details

## License

The Unlicense

---

**Project Status**: Alpha (v0.2.1)  
**Supported Platforms**: Linux, macOS, Windows, Android, iOS, WASM  
**Minimum Rust Version**: 1.75.0
