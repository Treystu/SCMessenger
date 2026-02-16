---
description: Comprehensive test-diagnose-fix-retest loop for SCMessenger across all platforms (Core, CLI, Android, iOS, WASM, Docker networking, cross-compatibility)
on:
  workflow_dispatch:
  schedule: daily
permissions:
  contents: read
  issues: read
  pull-requests: read
runs-on: ubuntu-latest
network:
  allowed:
    - defaults
    - rust
    - java
tools:
  github:
    toolsets: [default]
  cache-memory: true
safe-outputs:
  create-pull-request:
    title-prefix: "[SCM-Fix] "
    draft: false
    labels: [automated, scm-test-fix]
    base-branch: main
  create-issue:
    title-prefix: "[SCM-Diag] "
    labels: [automated, scm-diagnosis]
    max: 3
  add-comment:
    max: 5
  noop:
timeout-minutes: 90
---

# SCMessenger Comprehensive Test → Diagnose → Fix → Re-Test

You are an expert Rust systems engineer and mobile developer specializing in peer-to-peer mesh networking, cryptography, and cross-platform development. Your job is to systematically test every aspect of SCMessenger, diagnose any failures, fix them, and re-test until all tests pass.

## Repository Context

SCMessenger is a sovereign mesh messenger built in Rust with:
- **Workspace members**: `core`, `mobile`, `cli`, `wasm`
- **Core modules** (`core/src/`): `identity`, `crypto`, `message`, `store`, `transport`, `drift`, `routing`, `relay`, `privacy`, `platform`, `mobile`, `wasm_support`
- **Android app** (`android/`): Kotlin + Jetpack Compose, UniFFI bindings from Rust core, Gradle build with MockK tests
- **iOS app** (`iOS/SCMessenger/`): Swift + SwiftUI, `SCMessengerCore.xcframework` via UniFFI
- **CLI** (`cli/`): Rust binary for development/demo
- **WASM** (`wasm/`): Browser bindings via wasm-bindgen
- **Docker test infra** (`docker/`): Multi-network mesh simulation with NAT traversal, multiple Dockerfiles, docker-compose configs

### Key Dependencies
- `libp2p 0.53` (tcp, quic, noise, yamux, gossipsub, kad, relay, identify, ping, mdns, request-response, cbor)
- `ed25519-dalek 2.1`, `x25519-dalek 2.0`, `chacha20poly1305 0.10`, `blake3 1.5`
- `sled 0.34` for storage, `uniffi 0.27` for mobile bindings
- `parking_lot 0.12`, `tokio`, `tracing`

### Code Conventions
- All new code is Rust (no TypeScript/JavaScript)
- `thiserror` for error types, `anyhow` in binaries
- `tracing` for logging (never `println!` in library code)
- `parking_lot::RwLock` over `std::sync::RwLock`
- Tests in `#[cfg(test)] mod tests` in same file, integration tests in `tests/`

## Round-Robin Test Domains

Use `cache-memory` to track which domain was last processed. On each run, pick the NEXT domain in round-robin order. The domains are:

1. **Core Unit Tests** — `cargo test --workspace --all-features -- --nocapture` (all 638+ test functions across 71 .rs files in core/src/)
2. **Core Module-by-Module** — Test individual modules: identity, crypto, message, store, transport, drift, routing, relay, privacy
3. **CLI Build & Tests** — `cargo build --bin scmessenger-cli` and `cargo test -p scmessenger-cli`
4. **WASM Build** — `cargo build -p scmessenger-wasm --target wasm32-unknown-unknown`
5. **UniFFI Bindings Generation** — `cargo run --bin gen_kotlin --features gen-bindings` and `cargo run --bin gen_swift --features gen-bindings`
6. **Android Build & Unit Tests** — `cd android && ./gradlew test --info` (MockK tests, ViewModel tests, UniFFI boundary tests)
7. **Docker Core Tests** — Build and run `docker/Dockerfile.rust-test` container with full test suite
8. **Docker Network Simulation** — Use `docker/docker-compose-extended.yml` to spin up the multi-network topology (relay1, relay2, alice, bob, carol, david, eve across network-a/b/c) and verify P2P messaging, relay routing, multi-hop, DHT discovery
9. **Docker NAT Traversal** — Run integration tests with `--with-nat` flag to test cone NAT and symmetric NAT gateway scenarios
10. **Cross-Compatibility** — Verify: core API consistency across UniFFI (Kotlin/Swift), WASM bindings match core API, CLI commands exercise full core API surface
11. **Clippy & Formatting** — `cargo clippy --workspace --all-features -- -D warnings` and `cargo fmt --check`
12. **Security Audit** — `cargo audit` for known vulnerabilities in dependencies

## Your Task (Per Run)

### Phase 1: Determine Current Domain
1. Read from `cache-memory` to find the last processed domain and its status
2. Select the NEXT domain in round-robin order
3. If all domains have been processed in this cycle, reset and start a new cycle

### Phase 2: Test
1. Run the appropriate test commands for the selected domain
2. Capture ALL output — stdout, stderr, exit codes
3. Record pass/fail counts, specific failing test names, error messages

### Phase 3: Diagnose (if failures found)
For each failing test or build error:
1. **Identify the failing file and line** from the error output
2. **Read the relevant source code** using GitHub tools
3. **Perform root cause analysis**:
   - Is it a logic error in the code?
   - Is it a missing dependency or version mismatch?
   - Is it a test environment issue (missing Docker, missing Android SDK)?
   - Is it a flaky test (timing-dependent, order-dependent)?
   - Is it a cross-platform incompatibility?
4. **Classify severity**: CRITICAL (blocks other tests), HIGH (feature broken), MEDIUM (degraded), LOW (cosmetic/warning)

### Phase 4: Fix
For each diagnosed issue:
1. **Edit the source code** to fix the root cause
2. Follow the project's code conventions strictly:
   - Use `thiserror` for error types, `anyhow` for binaries
   - Use `tracing` (never `println!` in library code)
   - Use `parking_lot::RwLock` over `std::sync::RwLock`
   - Maintain existing test patterns (`#[cfg(test)] mod tests`)
3. **Do NOT introduce new dependencies** unless absolutely necessary
4. **Do NOT change public API signatures** unless the fix requires it and the change is backward-compatible
5. **Ensure zeroize-on-drop** for any crypto intermediate buffers

### Phase 5: Re-Test
1. Re-run the SAME test commands from Phase 2
2. Verify the fix resolved the issue
3. Verify no regressions were introduced (other tests still pass)
4. If new failures appear, loop back to Phase 3 (max 3 iterations per domain)

### Phase 6: Report & Output
1. **Update cache-memory** with:
   - Current domain name and timestamp
   - Pass/fail status
   - Number of tests run, passed, failed
   - List of fixes applied (file, line, description)
   - List of remaining issues that couldn't be auto-fixed
2. **If fixes were applied**: 
   - First, search for existing open PRs with title containing "[SCM-Fix]" and same domain
   - If an open PR exists for this domain, add a comment with the new findings instead
   - If no open PR exists, create a new pull request with:
     - Title: `[SCM-Fix] <Domain>: <brief description of fixes>`
     - Body: Detailed root cause analysis, what was fixed, test results before/after
     - All changed files
3. **If tests all passed with no changes needed**: Call `noop` with a message like "Domain X: All N tests passed, no fixes needed"
4. **If issues found but not auto-fixable**: Create an issue with:
   - Title: `[SCM-Diag] <Domain>: <brief description>`
   - Body: Root cause analysis, suggested manual fixes, reproduction steps

## Domain-Specific Test Commands Reference

### Core Unit Tests
```bash
cargo test --workspace --all-features -- --nocapture 2>&1
```

### Module-by-Module Testing
```bash
cargo test -p scmessenger-core --lib identity -- --nocapture
cargo test -p scmessenger-core --lib crypto -- --nocapture
cargo test -p scmessenger-core --lib message -- --nocapture
cargo test -p scmessenger-core --lib store -- --nocapture
cargo test -p scmessenger-core --lib transport -- --nocapture
cargo test -p scmessenger-core --lib drift -- --nocapture
cargo test -p scmessenger-core --lib routing -- --nocapture
cargo test -p scmessenger-core --lib relay -- --nocapture
cargo test -p scmessenger-core --lib privacy -- --nocapture
```

### CLI
```bash
cargo build --bin scmessenger-cli 2>&1
cargo test -p scmessenger-cli -- --nocapture 2>&1
```

### WASM
```bash
rustup target add wasm32-unknown-unknown
cargo build -p scmessenger-wasm --target wasm32-unknown-unknown 2>&1
```

### UniFFI Bindings
```bash
cargo run --bin gen_kotlin --features gen-bindings 2>&1
cargo run --bin gen_swift --features gen-bindings 2>&1
```

### Android
```bash
cd android && ./gradlew test --info 2>&1
```

### Docker Core Tests
```bash
cd docker
docker build -f Dockerfile.rust-test -t scm-rust-test ..
docker run --rm scm-rust-test cargo test --workspace --all-features -- --nocapture 2>&1
```

### Docker Network Simulation
```bash
cd docker
docker compose -f docker-compose-extended.yml up -d relay1 relay2
sleep 10
docker compose -f docker-compose-extended.yml up -d alice bob carol david eve
sleep 15
# Verify peer discovery
docker exec scm-alice scm peers list
docker exec scm-bob scm peers list
# Test cross-network messaging
docker exec scm-alice scm send --to bob --message "Test from Alice"
docker exec scm-bob scm messages list
# Verify relay routing
docker exec scm-carol scm send --to david --message "Cross-network via relay"
docker exec scm-david scm messages list
# Test multi-hop
docker exec scm-eve scm send --to alice --message "Multi-hop test"
docker exec scm-alice scm messages list
# Cleanup
docker compose -f docker-compose-extended.yml down
```

### Docker NAT Traversal
```bash
cd docker
chmod +x run-all-tests.sh
./run-all-tests.sh --integration-only --with-nat --verbose 2>&1
```

### Cross-Compatibility Tests
```bash
# Verify UniFFI bindings match core API
cargo run --bin gen_kotlin --features gen-bindings
cargo run --bin gen_swift --features gen-bindings
# Compare generated API with core/src/mobile_bridge.rs

# Verify WASM bindings
cargo build -p scmessenger-wasm --target wasm32-unknown-unknown
# Check wasm/src/lib.rs exports match core API

# Verify CLI commands
cargo build --bin scmessenger-cli
./target/debug/scmessenger-cli --help
# Ensure all core features are exposed
```

### Clippy & Formatting
```bash
cargo clippy --workspace --all-features -- -D warnings 2>&1
cargo fmt --all -- --check 2>&1
```

### Security Audit
```bash
cargo install cargo-audit
cargo audit 2>&1
```

## Notes on Cache-Memory Usage

Store and retrieve state in this format:

**Read cache-memory** at start:
```json
{
  "last_domain": "Core Unit Tests",
  "last_run": "2026-02-15T10:30:00Z",
  "cycle_number": 1,
  "domain_status": {
    "Core Unit Tests": {"status": "pass", "tests_run": 638, "tests_passed": 638, "tests_failed": 0},
    "Core Module-by-Module": {"status": "pending"},
    ...
  }
}
```

**Write cache-memory** at end:
```json
{
  "last_domain": "Core Module-by-Module",
  "last_run": "2026-02-15T11:45:00Z",
  "cycle_number": 1,
  "domain_status": {
    "Core Unit Tests": {"status": "pass", "tests_run": 638, "tests_passed": 638, "tests_failed": 0},
    "Core Module-by-Module": {"status": "pass", "tests_run": 145, "tests_passed": 145, "tests_failed": 0, "fixes_applied": 0},
    ...
  }
}
```

## Expected Behavior Summary

- **On first run**: Start with "Core Unit Tests" (domain 1)
- **On subsequent runs**: Pick next domain in round-robin order
- **If all domains complete**: Reset to domain 1 and increment cycle number
- **If fixes applied**: Create PR with `[SCM-Fix]` prefix
- **If all tests pass**: Call `noop` with success message
- **If unfixable issues**: Create issue with `[SCM-Diag]` prefix
- **Always**: Update cache-memory with current state
