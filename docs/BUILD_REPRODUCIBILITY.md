# Build Reproducibility Guide

This document describes the build reproducibility system for SCMessenger, ensuring consistent and verifiable builds across all platforms.

## Overview

Build reproducibility means that building the same source code with the same toolchain produces byte-identical (or functionally identical) binaries. This is critical for:

- **Security**: Users can verify that published binaries match the source code
- **Debugging**: Developers can reproduce exact build conditions
- **Trust**: Open source projects can prove their binaries are built from public source
- **Compliance**: Some industries require reproducible builds for audit trails

## Reproducibility Status

| Platform | Status | Notes |
|----------|--------|-------|
| Rust Core | ✅ Reproducible | Pinned toolchain, locked dependencies |
| CLI (Linux) | ✅ Reproducible | Docker build environment available |
| CLI (macOS) | ⚠️ Partial | Xcode version affects builds |
| CLI (Windows) | ⚠️ Partial | MSVC version affects builds |
| Android | ✅ Reproducible | Pinned Gradle, NDK, SDK versions |
| iOS | ⚠️ Partial | Xcode version affects builds |
| WASM | ✅ Reproducible | Pinned toolchain, wasm-opt version |

## Pinned Versions

### Rust Toolchain

**File**: `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.75.0"
components = ["rustfmt", "clippy"]
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "aarch64-linux-android",
    "armv7-linux-androideabi",
    "x86_64-linux-android",
    "i686-linux-android",
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
    "x86_64-apple-ios",
    "wasm32-unknown-unknown",
]
```

This file ensures:
- All developers use Rust 1.75.0
- CI/CD uses Rust 1.75.0
- Required components (rustfmt, clippy) are installed
- All target platforms are available

### Rust Dependencies

**File**: `Cargo.lock`

The `Cargo.lock` file pins exact versions of all Rust dependencies. Always commit this file to version control.

**Important**: Always use `cargo build --locked` in CI/CD to ensure exact dependency versions.

### Android Build

**File**: `android/build.gradle`

Pinned versions:
- **Gradle**: 8.13 (via `gradle/wrapper/gradle-wrapper.properties`)
- **Gradle Plugin**: 8.13.2
- **Kotlin**: 1.9.20
- **Compose**: 1.5.4
- **Hilt**: 2.50
- **NDK**: 30.0.14904198 (r30)
- **SDK**: compileSdk 35, targetSdk 35, minSdk 26

### iOS Build

**File**: `iOS/SCMessenger.xcodeproj/project.pbxproj`

Pinned versions:
- **Xcode**: 15.0+ (specified in CI workflows)
- **Swift**: 5.9 (bundled with Xcode)
- **iOS Deployment Target**: 14.0

### WASM Build

Pinned versions:
- **Rust**: 1.75.0 (via rust-toolchain.toml)
- **wasm-pack**: Latest stable (installed via curl script)
- **wasm-opt**: From binaryen package (version varies by platform)

## Docker Build Environment

For maximum reproducibility on Linux, use the Docker build environment.

### Quick Start

```bash
# Build the Docker image
./docker/build.sh build-image

# Build CLI binary
./docker/build.sh build-cli

# Output: target/docker-release/scmessenger-cli
```

### What's Inside

The Docker image (`docker/build.Dockerfile`) includes:
- **Base**: `rust:1.75.0-slim` (Debian-based)
- **System packages**: build-essential, pkg-config, libssl-dev
- **User**: Non-root `builder` user (UID 1000)
- **Dependencies**: Pre-downloaded Cargo dependencies (cached layer)

### Benefits

1. **Consistent environment**: Same base OS, same system libraries
2. **Isolated builds**: No host system dependencies
3. **Cached layers**: Fast rebuilds when only source code changes
4. **CI/CD ready**: Can be used in GitHub Actions

## Verifying Reproducibility

### Local Verification

Build the same commit twice and compare:

```bash
# First build
git checkout v0.2.1
cargo clean
cargo build --release --locked
cp target/release/scmessenger-cli build1

# Second build
cargo clean
cargo build --release --locked
cp target/release/scmessenger-cli build2

# Compare
sha256sum build1 build2
```

**Expected result**: Checksums should match (or differ only in timestamps).

### Docker Verification

```bash
# Build in Docker
./docker/build.sh build-cli
sha256sum target/docker-release/scmessenger-cli

# Build locally
cargo build --release --locked
sha256sum target/release/scmessenger-cli
```

**Expected result**: Checksums should match (or differ only in timestamps).

## Known Non-Determinism

Some factors can cause builds to differ:

### Timestamps

Rust binaries include build timestamps in debug info. This is acceptable for reproducibility as long as the code is functionally identical.

**Mitigation**: Strip debug symbols for release builds (already configured in `Cargo.toml`).

### System Libraries

Linking against system libraries (e.g., OpenSSL) can cause differences across systems.

**Mitigation**: Use Docker for Linux builds, which provides consistent system libraries.

### Compiler Versions

Different Rust versions produce different binaries.

**Mitigation**: Use `rust-toolchain.toml` to pin the Rust version.

### Platform-Specific Differences

macOS and Windows builds depend on Xcode and MSVC versions, which are harder to pin.

**Mitigation**: Document required versions in CI workflows and platform setup guides.

## CI/CD Integration

### GitHub Actions

Our CI workflows use the pinned versions:

```yaml
# .github/workflows/ci.yml
- uses: dtolnay/rust-toolchain@stable
  with:
    toolchain: 1.75.0  # Must match rust-toolchain.toml
    targets: x86_64-unknown-linux-gnu

- name: Build with locked dependencies
  run: cargo build --release --locked
```

### Release Builds

Release builds use the same pinned versions:

```yaml
# .github/workflows/release.yml
- name: Build CLI
  run: cargo build --release --bin scmessenger-cli --target ${{ matrix.target }} --locked
```

## Updating Versions

When updating pinned versions, follow this process:

### 1. Update Rust Toolchain

```bash
# Edit rust-toolchain.toml
# Change channel = "1.75.0" to channel = "1.76.0"

# Test locally
rustup show
cargo build --workspace
cargo test --workspace
```

### 2. Update Android Versions

```bash
# Edit android/build.gradle
# Update kotlin_version, gradle_plugin_version, etc.

# Test locally
cd android
./gradlew clean build
```

### 3. Update Docker Image

```bash
# Edit docker/build.Dockerfile
# Change FROM rust:1.75.0-slim to FROM rust:1.76.0-slim

# Rebuild image
./docker/build.sh build-image
./docker/build.sh test
```

### 4. Update CI Workflows

```bash
# Edit .github/workflows/ci.yml and release.yml
# Update toolchain versions to match rust-toolchain.toml

# Test in CI
git commit -m "chore: update Rust to 1.76.0"
git push
# Watch GitHub Actions
```

### 5. Document Changes

```bash
# Update CHANGELOG.md
# Update this document if process changes
```

## Troubleshooting

### Builds differ across machines

**Symptom**: Same source code produces different binaries on different machines.

**Diagnosis**:
```bash
# Check Rust version
rustc --version

# Check dependency versions
cargo tree

# Check for uncommitted Cargo.lock changes
git status Cargo.lock
```

**Solution**: Ensure `rust-toolchain.toml` is present and `Cargo.lock` is committed.

### Docker builds fail

**Symptom**: `./docker/build.sh build-cli` fails.

**Diagnosis**:
```bash
# Check Docker is running
docker ps

# Check disk space
df -h

# Check image exists
docker images | grep scmessenger-builder
```

**Solution**: See `docker/README.md` for troubleshooting steps.

### Android builds differ

**Symptom**: Android APK/AAB files differ across builds.

**Diagnosis**:
```bash
# Check Gradle version
cd android
./gradlew --version

# Check NDK version
echo $ANDROID_NDK_ROOT
```

**Solution**: Ensure `gradle-wrapper.properties` and `build.gradle` have pinned versions.

## Best Practices

1. **Always commit Cargo.lock**: Never add it to .gitignore
2. **Use --locked flag**: In CI/CD and release builds
3. **Pin all versions**: Rust, Gradle, NDK, SDK, dependencies
4. **Document versions**: In this file and platform setup guides
5. **Test reproducibility**: Before major releases
6. **Use Docker for Linux**: For maximum reproducibility
7. **Update regularly**: Keep toolchains up to date, but test thoroughly

## Related Documentation

- [Docker Build Environment](../docker/README.md)
- [Release Pipeline](../.github/workflows/release.yml)
- [Deployment Guide](./DEPLOYMENT.md)
- [Android Setup](./platform/ANDROID_SETUP.md)
- [iOS Setup](./platform/IOS_SETUP.md)

## References

- [Reproducible Builds Project](https://reproducible-builds.org/)
- [Rust Reproducible Builds](https://doc.rust-lang.org/cargo/reference/build-scripts.html#reproducible-builds)
- [Android Reproducible Builds](https://developer.android.com/studio/build/building-cmdline#reproducible_builds)
