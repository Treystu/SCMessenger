# Cross-OS Build Compatibility Plan

**Generated:** 2026-06-03
**Host:** WSL2 Ubuntu 26.04 LTS (JayGoCOMP) x86_64, 8 cores, 16GB RAM
**Rust:** 1.95.0 (scmessenger toolchain override)

---

## Current State Assessment

###  What Works Right Now

| Target | Method | Status | Notes |
|--------|--------|--------|-------|
| Linux x86_64 native | `cargo build --workspace` |  PASS | 2m 52s check, full build ~5m |
| Android (all 4 archs) | `cargo-ndk` + NDK |  Ready | aarch64/armv7/i686/x86_64 targets installed |
| WASM | `wasm32-unknown-unknown` |  Target installed | Needs `wasm-pack` for full build |
| Windows x86_64 MSVC | `x86_64-pc-windows-msvc` | ️ Target installed | Needs MSVC linker on Windows host |
| macOS/iOS | apple targets installed | ️ Can compile | Needs macOS host for linking/signing |

###  What's Missing / Blocked

| Issue | Impact | Solution |
|-------|--------|----------|
| No `protoc` (protobuf compiler) | Potential build failures for libp2p gossipsub | `apt install protobuf-compiler` |
| No `libsqlite3-dev` | sled store may fail to compile natively | `apt install libsqlite3-dev` |
| No `clang` | Required by some Rust crates (ring, etc.) | `apt install clang` |
| No `wasm-pack` | Can't build WASM package for web | `cargo install wasm-pack` or `npm install -g wasm-pack` |
| No Docker | Can't use containerized cross-compilation | Install Docker Desktop WSL2 backend |
| No `cross` | Can't use cross-rs for easy cross-compile | `cargo install cross` |
| No `zig` | Can't use zig as cross-linker | `apt install zig` or download binary |
| NDK not found in WSL | Android cross-compile needs NDK path | Set `ANDROID_HOME` or `ANDROID_SDK_ROOT` |
| CRLF in shell scripts | `docs_sync_check.sh` and others fail | `git config core.autocrlf input` |

---

## Cross-Compilation Matrix

### From This WSL2 Ubuntu Host

```
                    ┌─────────────────────────────────────────────────────┐
                    │           WSL2 Ubuntu 26.04 x86_64                 │
                    │           Rust 1.95.0 + cargo-ndk                  │
                    └──────────┬──────────┬──────────┬──────────┬────────┘
                               │          │          │          │
              ┌────────────────┘          │          │          └──────────────────┐
              ▼                           ▼          ▼                           ▼
   ┌──────────────────┐    ┌──────────────────┐  ┌──────────────┐   ┌──────────────────┐
   │  Linux x86_64    │    │  Android (4 arch)│  │  WASM        │   │  Windows MSVC    │
   │   Native       │    │   cargo-ndk    │  │  ️ Needs    │   │  ️ Needs MSVC  │
   │  cargo build     │    │  Needs NDK path  │  │  wasm-pack   │   │  linker on Win   │
   └──────────────────┘    └──────────────────┘  └──────────────┘   └──────────────────┘
              │
              ▼
   ┌──────────────────┐
   │  macOS / iOS     │
   │   Needs macOS  │
   │  host for link   │
   └──────────────────┘
```

### Recommended Build Strategy Per OS

#### 1. Linux (Native — This Host)
```bash
export CARGO_INCREMENTAL=0
cargo build --workspace          # Full build
cargo test --workspace           # Full test suite
cargo build -p scmessenger-wasm --target wasm32-unknown-unknown  # WASM
```

#### 2. Android (Cross-Compile from Linux)
```bash
# Set NDK path (find it first)
export ANDROID_HOME=/path/to/android-sdk
export ANDROID_SDK_ROOT=$ANDROID_HOME

# Build all Android targets
cargo ndk -t aarch64-linux-android -t armv7-linux-androideabi \
          -t x86_64-linux-android -t i686-linux-android \
          -o ./jniLibs build --release

# Then build the APK on Windows with Gradle
cd android && ./gradlew assembleDebug
```

#### 3. Windows (Cross-Compile from Linux — Advanced)
```bash
# Option A: Use cross-rs with Docker
cargo install cross
cross build --target x86_64-pc-windows-msvc

# Option B: Build natively on Windows host
# Run from PowerShell/CMD on Windows:
cargo build --workspace
```

#### 4. macOS / iOS (Requires macOS Host)
```bash
# Must run on macOS (real or CI)
cargo build --target x86_64-apple-darwin      # macOS Intel
cargo build --target aarch64-apple-darwin     # macOS Apple Silicon
cargo build --target aarch64-apple-ios        # iOS device
cargo build --target aarch64-apple-ios-sim    # iOS simulator

# Generate XCFramework for iOS
./scripts/build_ios_xcframework.sh
```

---

## Recommended CI/CD Pipeline (Maximum Efficiency)

### GitHub Actions Matrix Strategy

```yaml
strategy:
  matrix:
    include:
      # Linux native — fastest, runs tests
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        test: true
      
      # Android cross-compile
      - os: ubuntu-latest
        target: aarch64-linux-android
        cargo-ndk: true
      
      # WASM
      - os: ubuntu-latest
        target: wasm32-unknown-unknown
        wasm-pack: true
      
      # Windows cross-compile
      - os: windows-latest
        target: x86_64-pc-windows-msvc
      
      # macOS + iOS
      - os: macos-latest
        target: aarch64-apple-darwin
      - os: macos-latest
        target: aarch64-apple-ios
```

### Build Optimization Tips

1. **Use `sccache`** for shared compilation cache across CI runs
2. **Use `cargo-nextest`** for parallel test execution
3. **Cache `~/.cargo/registry`** and `~/.cargo/git` between runs
4. **Cache `target/` directory** for incremental CI builds (only on same-OS runners)
5. **Use `cross`** for consistent cross-compilation via Docker
6. **Parallelize Android targets** — build all 4 archs in parallel jobs

---

## Immediate Action Items (Priority Order)

### P0 — Fix WSL Build Environment
```bash
# Install missing build dependencies
sudo apt update && sudo apt install -y \
  protobuf-compiler \
  libsqlite3-dev \
  clang \
  lz4 \
  libudev-dev \
  libusb-1.0-0-dev

# Install wasm-pack
cargo install wasm-pack

# Fix CRLF in all shell scripts
find scripts/ -name "*.sh" -exec sed -i 's/\r$//' {} \;

# Set up Android NDK path (find it on Windows side)
# Add to ~/.bashrc:
# export ANDROID_HOME=/mnt/c/path/to/android-sdk
```

### P1 — Enable Full Cross-Compile from WSL
```bash
# Install cross-rs for Docker-based cross-compilation
cargo install cross

# Install zig for zig-linker cross-compilation
# (enables cross-compiling to any target from any host)
```

### P2 — CI/CD Hardening
- Add GitHub Actions workflow for multi-OS matrix builds
- Add `cargo-deny` for license/security auditing
- Add `cargo-audit` for vulnerability scanning
- Set up `sccache` for build caching

---

## OS-Specific Notes

### WSL2 (This Environment)
- **Strengths:** Full Linux toolchain, all Rust targets installed, cargo-ndk available
- **Limitations:** No native USB/BLE (needs usbipd-win), mDNS may not work across Hyper-V NIC
- **Best for:** Linux native builds, Android cross-compile, WASM builds
- **Not suitable for:** macOS/iOS builds, Windows MSVC builds (use Windows host)

### Windows Host
- **Strengths:** MSVC linker, Android Studio/Gradle, Visual Studio Build Tools
- **Limitations:** No native Unix toolchain, slower Rust compile times
- **Best for:** Windows builds, Android APK builds, iOS XCFramework (via remote macOS)
- **Not suitable for:** Linux native builds (use WSL2)

### macOS (CI or Local)
- **Strengths:** Only platform that can build/link macOS and iOS targets
- **Limitations:** Expensive hardware, slower CI runners
- **Best for:** macOS builds, iOS builds, XCFramework generation
- **Not suitable for:** Android builds (use Linux)

---

## Verification Commands

```bash
# Quick health check — run this to verify build environment
echo "=== Rust ===" && rustc --version && cargo --version
echo "=== Targets ===" && rustup target list --installed | wc -l
echo "=== NDK ===" && cargo ndk --version 2>&1 | head -1
echo "=== wasm-pack ===" && wasm-pack --version 2>/dev/null || echo "NOT INSTALLED"
echo "=== protoc ===" && protoc --version 2>/dev/null || echo "NOT INSTALLED"
echo "=== clang ===" && clang --version 2>/dev/null | head -1 || echo "NOT INSTALLED"
echo "=== Docker ===" && docker --version 2>/dev/null || echo "NOT INSTALLED"
echo "=== cross ===" && cross --version 2>/dev/null || echo "NOT INSTALLED"
```
