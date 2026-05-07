# CLI Development Setup Guide

Status: Active  
Last updated: 2026-03-07  
Validates: Requirements 5.9

This guide covers setting up your development environment for SCMessenger CLI development across Linux, macOS, and Windows.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Platform-Specific Setup](#platform-specific-setup)
- [Building](#building)
- [Running](#running)
- [Testing](#testing)
- [Debugging](#debugging)
- [Common Issues](#common-issues)
- [Advanced Configuration](#advanced-configuration)
- [Resources](#resources)

## Prerequisites

### All Platforms

- **Rust** 1.75.0 or later (see `rust-toolchain.toml`)
- **Cargo** (comes with Rust)
- **Git** 2.30+

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify installation:
```bash
rustc --version
cargo --version
```

### Platform-Specific Requirements

#### Linux

**Required packages:**
```bash
# Debian/Ubuntu
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install gcc pkg-config openssl-devel

# Arch Linux
sudo pacman -S base-devel pkg-config openssl
```

#### macOS

**Required tools:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Verify installation
xcode-select -p
```

**Optional (via Homebrew):**
```bash
brew install openssl pkg-config
```

#### Windows

**Required tools:**
- **Visual Studio 2019 or later** with "Desktop development with C++" workload
- Or **Build Tools for Visual Studio 2019** (lighter weight)

**Download:**
- https://visualstudio.microsoft.com/downloads/

**Alternative (MSYS2/MinGW):**
```bash
# Install MSYS2 from https://www.msys2.org/
# Then in MSYS2 terminal:
pacman -S mingw-w64-x86_64-toolchain mingw-w64-x86_64-openssl
```

## Platform-Specific Setup

### Linux Setup

```bash
# Clone repository
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger

# Build CLI
cargo build --release --bin scmessenger-cli

# Binary location
ls -lh target/release/scmessenger-cli
```

### macOS Setup

```bash
# Clone repository
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger

# Build CLI
cargo build --release --bin scmessenger-cli

# Binary location
ls -lh target/release/scmessenger-cli
```

### Windows Setup

**PowerShell:**
```powershell
# Clone repository
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger

# Build CLI
cargo build --release --bin scmessenger-cli

# Binary location
dir target\release\scmessenger-cli.exe
```

**Git Bash:**
```bash
# Clone repository
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger

# Build CLI
cargo build --release --bin scmessenger-cli

# Binary location
ls -lh target/release/scmessenger-cli.exe
```

## Building

### Debug Build

```bash
# Fast compilation, includes debug symbols
cargo build --bin scmessenger-cli

# Binary location: target/debug/scmessenger-cli
```

### Release Build

```bash
# Optimized compilation, smaller binary
cargo build --release --bin scmessenger-cli

# Binary location: target/release/scmessenger-cli
```

### Build with Specific Features

```bash
# Build with all features
cargo build --release --bin scmessenger-cli --all-features

# Build with specific features
cargo build --release --bin scmessenger-cli --features "feature1,feature2"
```

### Cross-Compilation

#### Linux → Windows

```bash
# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Install cross-compiler
sudo apt-get install mingw-w64

# Build
cargo build --release --bin scmessenger-cli --target x86_64-pc-windows-gnu
```

#### macOS → Linux

```bash
# Add Linux target
rustup target add x86_64-unknown-linux-gnu

# Build (requires Docker or cross-compilation toolchain)
cargo build --release --bin scmessenger-cli --target x86_64-unknown-linux-gnu
```

## Running

### Basic Usage

```bash
# Show help
./target/release/scmessenger-cli --help

# Start daemon
./target/release/scmessenger-cli start

# Start with custom port
./target/release/scmessenger-cli start --port 9003

# Start with verbose logging
RUST_LOG=debug ./target/release/scmessenger-cli start
```

### Common Commands

```bash
# Show version
./target/release/scmessenger-cli --version

# Generate identity
./target/release/scmessenger-cli identity create

# List identities
./target/release/scmessenger-cli identity list

# Send message
./target/release/scmessenger-cli send --to <peer-id> --message "Hello"

# List peers
./target/release/scmessenger-cli peers list
```

### Running from Cargo

```bash
# Run without building first
cargo run --release --bin scmessenger-cli -- start

# Run with arguments
cargo run --release --bin scmessenger-cli -- --help

# Run with logging
RUST_LOG=debug cargo run --release --bin scmessenger-cli -- start
```

## Testing

### Unit Tests

```bash
# Run all CLI tests
cargo test -p scmessenger-cli

# Run specific test
cargo test -p scmessenger-cli test_name

# Run with logging
RUST_LOG=debug cargo test -p scmessenger-cli -- --nocapture
```

### Integration Tests

```bash
# Run integration tests
cargo test -p scmessenger-cli --test '*'

# Run specific integration test
cargo test -p scmessenger-cli --test integration_test_name
```

### Smoke Test

```bash
# Build and run basic smoke test
cargo build --release --bin scmessenger-cli
./target/release/scmessenger-cli --version
./target/release/scmessenger-cli --help
```

## Debugging

### Debug Logging

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/scmessenger-cli start

# Enable trace logging
RUST_LOG=trace ./target/release/scmessenger-cli start

# Enable logging for specific modules
RUST_LOG=scmessenger_cli=debug,scmessenger_core=info ./target/release/scmessenger-cli start
```

### Using Debugger

#### Linux/macOS (lldb)

```bash
# Build with debug symbols
cargo build --bin scmessenger-cli

# Run with lldb
lldb target/debug/scmessenger-cli

# In lldb:
(lldb) run start
(lldb) breakpoint set --name main
(lldb) continue
```

#### Linux (gdb)

```bash
# Build with debug symbols
cargo build --bin scmessenger-cli

# Run with gdb
gdb target/debug/scmessenger-cli

# In gdb:
(gdb) run start
(gdb) break main
(gdb) continue
```

#### Windows (Visual Studio)

1. Build with debug symbols: `cargo build --bin scmessenger-cli`
2. Open Visual Studio
3. Debug → Attach to Process
4. Select `scmessenger-cli.exe`
5. Set breakpoints and debug

### Performance Profiling

#### Linux (perf)

```bash
# Build with debug symbols
cargo build --release --bin scmessenger-cli

# Profile with perf
perf record -g ./target/release/scmessenger-cli start
perf report
```

#### macOS (Instruments)

```bash
# Build with debug symbols
cargo build --release --bin scmessenger-cli

# Profile with Instruments
instruments -t "Time Profiler" ./target/release/scmessenger-cli start
```

## Common Issues

### Issue: "error: linker `cc` not found"

**Platform**: Linux

**Solution**:
```bash
# Debian/Ubuntu
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc

# Arch Linux
sudo pacman -S base-devel
```

### Issue: "error: failed to run custom build command for `openssl-sys`"

**Platform**: Linux

**Solution**:
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

**Solution**:
1. Install Visual Studio with "Desktop development with C++" workload
2. Or install Build Tools for Visual Studio
3. Restart terminal after installation

### Issue: "Permission denied" when running binary

**Platform**: Linux/macOS

**Solution**:
```bash
# Make binary executable
chmod +x target/release/scmessenger-cli

# Or run with cargo
cargo run --release --bin scmessenger-cli
```

### Issue: Binary size is too large

**Solution**:
```bash
# Strip debug symbols
strip target/release/scmessenger-cli

# Or build with optimized profile
cargo build --release --bin scmessenger-cli

# Check size
ls -lh target/release/scmessenger-cli
```

### Issue: Slow compilation

**Solution**:
```bash
# Use faster linker (Linux)
sudo apt-get install lld
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Use faster linker (macOS)
brew install llvm
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Increase parallel jobs
cargo build --release --bin scmessenger-cli -j 8
```

## Advanced Configuration

### Custom Build Profiles

Add to `Cargo.toml`:

```toml
[profile.release-small]
inherits = "release"
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip debug symbols
panic = "abort"     # Smaller panic handler
```

Build with custom profile:
```bash
cargo build --profile release-small --bin scmessenger-cli
```

### Environment Variables

```bash
# Rust compiler flags
export RUSTFLAGS="-C target-cpu=native"

# Cargo build jobs
export CARGO_BUILD_JOBS=8

# Cargo target directory
export CARGO_TARGET_DIR=/tmp/cargo-target

# Rust backtrace
export RUST_BACKTRACE=1  # Enable backtraces
export RUST_BACKTRACE=full  # Full backtraces
```

### Static Linking (Linux)

```bash
# Add musl target
rustup target add x86_64-unknown-linux-musl

# Build statically linked binary
cargo build --release --bin scmessenger-cli --target x86_64-unknown-linux-musl

# Verify static linking
ldd target/x86_64-unknown-linux-musl/release/scmessenger-cli
# Should output: "not a dynamic executable"
```

### Binary Optimization

```bash
# Build with LTO and optimizations
RUSTFLAGS="-C lto=fat -C embed-bitcode=yes" cargo build --release --bin scmessenger-cli

# Strip binary
strip target/release/scmessenger-cli

# Compress with UPX (optional)
upx --best --lzma target/release/scmessenger-cli
```

## Resources

### Documentation

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [SCMessenger Architecture](../ARCHITECTURE.md)
- [SCMessenger Testing Guide](../TESTING_GUIDE.md)

### Tools

- [rust-analyzer](https://rust-analyzer.github.io/) - IDE support
- [cargo-watch](https://github.com/watchexec/cargo-watch) - Auto-rebuild on changes
- [cargo-edit](https://github.com/killercup/cargo-edit) - Manage dependencies
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit) - Security audits

### Community

- [Rust Users Forum](https://users.rust-lang.org/)
- [Rust Discord](https://discord.gg/rust-lang)
- [SCMessenger GitHub Issues](https://github.com/Treystu/SCMessenger/issues)
- [SCMessenger GitHub Discussions](https://github.com/Treystu/SCMessenger/discussions)

## Getting Help

- Check [Troubleshooting Guide](../troubleshooting/BUILD_ISSUES.md)
- Search [GitHub Issues](https://github.com/Treystu/SCMessenger/issues)
- Ask in [GitHub Discussions](https://github.com/Treystu/SCMessenger/discussions)
- See [CONTRIBUTING.md](../../CONTRIBUTING.md) for contribution guidelines

---

**Next Steps:**
- Read [Architecture Overview](../ARCHITECTURE.md)
- Review [Testing Guide](../TESTING_GUIDE.md)
- Check [Deployment Guide](../DEPLOYMENT.md)
