# CI Failures Troubleshooting Guide

Status: Active  
Last updated: 2026-03-07

This guide covers common GitHub Actions CI failures and their solutions.

## Table of Contents

- [General CI Issues](#general-ci-issues)
- [Workflow Failures](#workflow-failures)
- [Build Failures](#build-failures)
- [Test Failures](#test-failures)
- [Timeout Issues](#timeout-issues)
- [Cache Issues](#cache-issues)
- [Platform-Specific CI Failures](#platform-specific-ci-failures)
- [Debugging CI Failures](#debugging-ci-failures)

## General CI Issues

### Issue: "Workflow run failed"

**Symptoms:**
- Red X on commit
- Email notification of failure

**Solution:**
1. Click on the failed workflow
2. Identify which job failed
3. Click on the failed job
4. Expand the failed step
5. Read error message
6. Follow specific guidance below

### Issue: "Workflow not triggered"

**Symptoms:**
- No workflow run appears after push

**Solution:**
```bash
# Check workflow file syntax
cat .github/workflows/ci.yml

# Verify workflow is enabled
# Go to: Actions → Select workflow → Enable workflow

# Check branch protection rules
# Settings → Branches → Branch protection rules

# Force trigger manually
# Actions → Select workflow → Run workflow
```

### Issue: "Required status check not found"

**Symptoms:**
```
Required status check "ci / rust-core" is expected but not present
```

**Solution:**
1. Go to Settings → Branches → Branch protection rules
2. Edit rule for `main` branch
3. Update required status checks
4. Save changes

## Workflow Failures

### Issue: "Path filter job failed"

**Symptoms:**
```
Error: Unable to process file command 'output' successfully
```

**Solution:**
```yaml
# Check .github/workflows/ci.yml path filter configuration
- uses: dorny/paths-filter@v2
  id: filter
  with:
    filters: |
      core:
        - 'core/**'
        - 'Cargo.toml'
        - 'Cargo.lock'
      android:
        - 'android/**'
      # ... etc
```

### Issue: "Checkout failed"

**Symptoms:**
```
Error: fatal: repository not found
```

**Solution:**
```yaml
# Verify checkout action in workflow
- uses: actions/checkout@v4
  with:
    fetch-depth: 0  # Full history for changelog generation
```

### Issue: "Setup Rust failed"

**Symptoms:**
```
Error: Toolchain '1.75.0' not found
```

**Solution:**
```yaml
# Check rust-toolchain.toml exists and is valid
[toolchain]
channel = "1.75.0"
components = ["rustfmt", "clippy"]

# Or update workflow to install toolchain
- uses: dtolnay/rust-toolchain@stable
  with:
    toolchain: 1.75.0
    components: rustfmt, clippy
```

## Build Failures

### Issue: "Rust build failed"

**Symptoms:**
```
error: could not compile `scmessenger-core` due to 5 previous errors
```

**Solution:**
1. Reproduce locally:
   ```bash
   cargo clean
   cargo build --workspace
   ```
2. Fix compilation errors
3. Commit and push fix

### Issue: "Android build failed"

**Symptoms:**
```
Error: Task :app:compileDebugKotlin FAILED
```

**Solution:**
1. Check Android SDK/NDK versions in workflow
2. Verify Gradle version compatibility
3. Reproduce locally:
   ```bash
   cd android
   ./gradlew clean assembleDebug
   ```

### Issue: "iOS build failed"

**Symptoms:**
```
error: Build input file cannot be found: 'libscmessenger_mobile.a'
```

**Solution:**
1. Ensure Rust library is built before iOS build
2. Check workflow order:
   ```yaml
   - name: Build Rust core for iOS
     run: cargo build --release --target aarch64-apple-ios
   
   - name: Build iOS app
     run: xcodebuild build ...
   ```

### Issue: "WASM build failed"

**Symptoms:**
```
error: target 'wasm32-unknown-unknown' not found
```

**Solution:**
```yaml
# Add WASM target in workflow
- name: Add WASM target
  run: rustup target add wasm32-unknown-unknown

- name: Build WASM
  run: |
    cd wasm
    wasm-pack build --target web --release
```

## Test Failures

### Issue: "Unit tests failed"

**Symptoms:**
```
test result: FAILED. 265 passed; 1 failed; 0 ignored
```

**Solution:**
1. Identify failing test from logs
2. Reproduce locally:
   ```bash
   cargo test --workspace -- --nocapture
   ```
3. Fix test or code
4. Verify fix:
   ```bash
   cargo test --workspace
   ```

### Issue: "Integration tests timeout"

**Symptoms:**
```
test integration_test ... timeout after 60s
```

**Solution:**
```yaml
# Increase timeout in workflow
- name: Run integration tests
  run: cargo test --workspace --test '*'
  timeout-minutes: 30  # Increase from default
```

### Issue: "Flaky tests"

**Symptoms:**
- Tests pass locally but fail in CI
- Tests fail intermittently

**Solution:**
1. Identify flaky test
2. Add retry logic or fix race condition
3. Run test multiple times locally:
   ```bash
   for i in {1..10}; do cargo test test_name || break; done
   ```

### Issue: "Property-based tests failed"

**Symptoms:**
```
proptest: Shrunk minimal failing case to: ...
```

**Solution:**
1. Note the minimal failing case from logs
2. Reproduce locally:
   ```bash
   RUST_LOG=debug cargo test test_name -- --nocapture
   ```
3. Fix the bug revealed by the failing case
4. Verify fix with property test

## Timeout Issues

### Issue: "Job exceeded maximum execution time"

**Symptoms:**
```
Error: The job running on runner has exceeded the maximum execution time of 360 minutes
```

**Solution:**
```yaml
# Reduce timeout or optimize build
jobs:
  build:
    timeout-minutes: 30  # Set reasonable timeout
    steps:
      # Use caching to speed up builds
      - uses: Swatinem/rust-cache@v2
```

### Issue: "Step timeout"

**Symptoms:**
```
Error: The operation was canceled.
```

**Solution:**
```yaml
# Add timeout to specific steps
- name: Run tests
  run: cargo test --workspace
  timeout-minutes: 15
```

### Issue: "Network timeout"

**Symptoms:**
```
error: failed to download from `https://crates.io/...`
```

**Solution:**
```yaml
# Add retry logic
- name: Build with retry
  uses: nick-fields/retry@v2
  with:
    timeout_minutes: 10
    max_attempts: 3
    command: cargo build --workspace
```

## Cache Issues

### Issue: "Cache restore failed"

**Symptoms:**
```
Warning: Failed to restore cache
```

**Solution:**
```yaml
# Check cache configuration
- uses: Swatinem/rust-cache@v2
  with:
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    cache-on-failure: true
```

### Issue: "Cache size exceeded"

**Symptoms:**
```
Warning: Cache size of ~10 GB (10000 MB) is over the 10GB limit
```

**Solution:**
```yaml
# Reduce cache size
- uses: Swatinem/rust-cache@v2
  with:
    cache-all-crates: false  # Only cache workspace crates
    cache-targets: "release"  # Only cache release builds
```

### Issue: "Stale cache causing failures"

**Symptoms:**
- Builds fail after dependency updates
- "error: package requires rustc X.Y.Z or newer"

**Solution:**
1. Clear cache manually:
   - Go to Actions → Caches
   - Delete relevant caches
2. Or update cache key:
   ```yaml
   - uses: Swatinem/rust-cache@v2
     with:
       key: v2-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
   ```

## Platform-Specific CI Failures

### Android CI Failures

**Issue: "ANDROID_HOME not set"**

**Solution:**
```yaml
- uses: android-actions/setup-android@v3
- uses: nttld/setup-ndk@v1
  with:
    ndk-version: r26b

- name: Set ANDROID_HOME
  run: echo "ANDROID_HOME=$ANDROID_SDK_ROOT" >> $GITHUB_ENV
```

**Issue: "Gradle daemon disappeared"**

**Solution:**
```yaml
- name: Build Android
  run: |
    cd android
    ./gradlew assembleDebug --no-daemon
```

### iOS CI Failures

**Issue: "Xcode version mismatch"**

**Solution:**
```yaml
- name: Select Xcode version
  run: sudo xcode-select -s /Applications/Xcode_15.0.app
```

**Issue: "Provisioning profile not found"**

**Solution:**
```yaml
# For CI, use debug builds without signing
- name: Build iOS
  run: |
    xcodebuild build \
      -workspace SCMessenger.xcworkspace \
      -scheme SCMessenger \
      -configuration Debug \
      CODE_SIGN_IDENTITY="" \
      CODE_SIGNING_REQUIRED=NO
```

### WASM CI Failures

**Issue: "wasm-pack not found"**

**Solution:**
```yaml
- name: Install wasm-pack
  run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

- name: Build WASM
  run: |
    cd wasm
    wasm-pack build --target web --release
```

## Debugging CI Failures

### Enable Debug Logging

```yaml
# Add to workflow
env:
  RUST_LOG: debug
  RUST_BACKTRACE: 1
```

Or enable GitHub Actions debug logging:
1. Go to Settings → Secrets → Actions
2. Add secret: `ACTIONS_STEP_DEBUG` = `true`
3. Re-run workflow

### Download Artifacts

```yaml
# Add artifact upload to workflow
- name: Upload build artifacts
  if: failure()
  uses: actions/upload-artifact@v4
  with:
    name: build-artifacts
    path: |
      target/
      *.log
```

### Reproduce Locally

```bash
# Use act to run GitHub Actions locally
# Install: https://github.com/nektos/act
act -j job-name

# Or use Docker
docker run --rm -v $(pwd):/workspace -w /workspace rust:1.75.0 \
  bash -c "cargo build --workspace"
```

### Check Workflow Logs

1. Go to Actions tab
2. Click on failed workflow run
3. Click on failed job
4. Expand failed step
5. Read full logs
6. Look for error messages

### Common Log Patterns

```bash
# Compilation error
error[E0XXX]: ...

# Test failure
test result: FAILED. X passed; Y failed

# Timeout
Error: The operation was canceled

# Network error
error: failed to download

# Permission error
Permission denied
```

## Prevention

### Pre-Commit Checks

```bash
# Install pre-commit hooks
./scripts/install_hooks.sh

# Run checks locally before pushing
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

### Local CI Simulation

```bash
# Run same commands as CI
cargo clean
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

### Monitor CI Minutes

- Go to Settings → Billing
- Check Actions minutes usage
- Optimize workflows to stay within limits

## Getting Help

If CI failures persist:

1. **Check workflow logs**: Read full error messages
2. **Reproduce locally**: Run same commands locally
3. **Search issues**: https://github.com/Treystu/SCMessenger/issues
4. **Ask for help**: https://github.com/Treystu/SCMessenger/discussions
5. **Check GitHub Status**: https://www.githubstatus.com/

### Useful Commands

```bash
# View workflow file
cat .github/workflows/ci.yml

# Validate workflow syntax
# Use: https://rhysd.github.io/actionlint/

# Check branch protection
# Settings → Branches → Branch protection rules

# View workflow runs
# Actions → All workflows
```

---

**Related Guides:**
- [Build Issues Guide](BUILD_ISSUES.md)
- [Runtime Issues Guide](RUNTIME_ISSUES.md)
- [Testing Guide](../TESTING_GUIDE.md)
- [Contributing Guide](../../CONTRIBUTING.md)
