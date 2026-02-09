# SCMessenger Testing Guide

## Quick Start

### Run All Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run with output visible
cargo test -- --nocapture

# Run specific package
cargo test -p scmessenger-core
cargo test -p scmessenger-mobile
cargo test -p scmessenger-wasm
```

### Run by Category

```bash
# Unit tests only (fast)
cargo test --lib

# Integration tests only
cargo test --test integration_e2e
cargo test --test integration_nat_reflection

# All integration tests
cargo test --tests
```

## Test Categories

### 1. Unit Tests (Fast, No Network)

Located in `src/**/*.rs` files as `#[cfg(test)] mod tests`.

**Run all unit tests:**
```bash
cargo test --lib
```

**By module:**
```bash
# Crypto tests
cargo test -p scmessenger-core crypto::

# Identity tests
cargo test -p scmessenger-core identity::

# Message tests
cargo test -p scmessenger-core message::

# Storage tests
cargo test -p scmessenger-core store::

# Transport tests
cargo test -p scmessenger-core transport::
```

**Coverage:**
- ✅ Cryptography (encrypt, sign, verify)
- ✅ Identity management (keys, storage)
- ✅ Message encoding/decoding
- ✅ Inbox/Outbox persistence
- ✅ Address reflection protocol (mocked)
- ✅ NAT configuration

**Test count:** 106 passing

### 2. Integration Tests (Network-Based)

Located in `core/tests/*.rs` files.

#### E2E Integration Tests

```bash
# Run all E2E tests
cargo test --test integration_e2e

# With output
cargo test --test integration_e2e -- --nocapture
```

**Coverage:**
- End-to-end message encryption
- Multi-user message flow
- Identity verification
- Storage persistence

**Test count:** 6 tests

#### NAT Traversal Integration Tests

```bash
# Run all NAT tests
cargo test --test integration_nat_reflection

# With output
cargo test --test integration_nat_reflection -- --nocapture
```

**Coverage:**
- ✅ Two-node address reflection
- ✅ Peer address discovery with live swarm
- ✅ Multi-peer NAT type detection
- ✅ Multiple sequential reflections
- ✅ Timeout and error handling

**Test count:** 5 tests

**Sample output:**
```
running 5 tests
✅ Address reflection test passed!
   Node 1 observed Node 2 at: 0.0.0.0:0
test test_two_node_address_reflection ... ok
✅ Peer address discovery test passed!
   Discovered external address: 0.0.0.0:0
test test_peer_address_discovery_with_live_swarm ... ok
✅ NAT traversal test passed!
   Detected NAT type: FullCone
   External address: 0.0.0.0:0
test test_nat_traversal_with_live_swarms ... ok
```

### 3. Mobile Tests

```bash
cargo test -p scmessenger-mobile
```

**Coverage:**
- UniFFI bindings lifecycle
- Mobile identity management
- Mobile messaging API

**Test count:** 3 tests

### 4. WASM Tests

```bash
cargo test -p scmessenger-wasm
```

**Coverage:**
- WASM transport layer
- WebSocket relay
- WebRTC peer connections
- Connection state management

**Test count:** 17/18 tests passing (1 timing test flaky)

## Test Results Summary

```
Total Tests: 137
├─ Core Unit:        106/113  (7 marked as integration-only)
├─ Core Integration:  11/11
├─ Mobile:            3/3
└─ WASM:             17/18    (1 flaky timing test)

Overall: 137/145 (94.5%)
```

## Continuous Integration

### Pre-Commit Checks

```bash
# Full CI check
./scripts/ci-check.sh

# Or manually:
cargo fmt --check
cargo clippy -- -D warnings
cargo test --all-targets
cargo build --release
```

### Quick Validation

```bash
# Fast smoke test (~10 seconds)
cargo test --lib

# Medium test (~30 seconds)
cargo test --lib && cargo test --tests

# Full test suite (~2 minutes)
cargo test --all-targets
```

## Testing Best Practices

### 1. Test Isolation

Each test should be independent:

```rust
#[tokio::test]
async fn test_something() {
    // Create fresh state
    let store = InMemoryStore::new();

    // Run test
    // ...

    // Cleanup happens automatically
}
```

### 2. Async Testing

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = some_async_fn().await;
    assert!(result.is_ok());
}
```

### 3. Error Testing

Test both success and failure paths:

```rust
#[test]
fn test_validation() {
    // Test success
    assert!(validate("good").is_ok());

    // Test failure
    assert!(validate("bad").is_err());
}
```

### 4. Network Tests

Add timeouts to prevent hangs:

```rust
#[tokio::test]
async fn test_network() {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        network_operation()
    ).await;

    assert!(result.is_ok());
}
```

## Debugging Failed Tests

### Enable Logging

```bash
# Set log level
RUST_LOG=debug cargo test -- --nocapture

# Specific module
RUST_LOG=scmessenger_core::transport=trace cargo test

# Pretty output
RUST_LOG=info cargo test -- --nocapture 2>&1 | less
```

### Run Single Test

```bash
# By name
cargo test test_two_node_address_reflection -- --nocapture

# With pattern
cargo test reflection -- --nocapture

# Show all output
cargo test -- --nocapture --test-threads=1
```

### Check Test Binary

```bash
# List all tests
cargo test -- --list

# Show ignored tests
cargo test -- --ignored --list
```

## Performance Testing

### Benchmark Tests

```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench nat_traversal
```

### Memory Testing

```bash
# With Valgrind (Linux)
cargo build --tests
valgrind --leak-check=full ./target/debug/deps/scmessenger_core-*

# With sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo test
```

### Load Testing

Create load tests in `benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn address_reflection_benchmark(c: &mut Criterion) {
    c.bench_function("address_reflection", |b| {
        b.iter(|| {
            // Benchmark code
            black_box(reflect_address())
        })
    });
}

criterion_group!(benches, address_reflection_benchmark);
criterion_main!(benches);
```

## Writing New Tests

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Integration Test Template

```rust
// In core/tests/integration_myfeature.rs
use scmessenger_core::*;

#[tokio::test]
async fn test_feature_integration() {
    // Setup
    let system = setup_test_environment().await;

    // Exercise
    let result = system.perform_operation().await;

    // Verify
    assert!(result.is_ok());

    // Cleanup (automatic via Drop)
}
```

## Ignored Tests

Some tests are marked `#[ignore]` and require special setup:

```bash
# Run ignored tests
cargo test -- --ignored

# Run all tests including ignored
cargo test -- --include-ignored
```

**Ignored test categories:**
- Tests requiring SwarmHandle (moved to integration tests)
- Tests requiring network access
- Tests requiring manual setup
- Long-running stress tests

## Platform-Specific Testing

### WASM Testing

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Run WASM tests
wasm-pack test --node wasm/

# In browser
wasm-pack test --chrome --headless wasm/
```

### Mobile Testing

```bash
# Generate UniFFI bindings
cargo run --bin uniffi-bindgen generate mobile/src/mobile.udl

# Run mobile tests
cargo test -p scmessenger-mobile
```

## Coverage Reports

### Install tarpaulin

```bash
cargo install cargo-tarpaulin
```

### Generate coverage

```bash
# HTML report
cargo tarpaulin --out Html --output-dir coverage/

# See results
open coverage/index.html

# CI format
cargo tarpaulin --out Xml
```

### Coverage Goals

- **Core crypto**: 100% (critical)
- **Core transport**: >90%
- **Integration**: >80%
- **Overall**: >85%

## Test Data

Test fixtures are in `core/tests/fixtures/`:

```
fixtures/
├── test_message.json
├── test_keys.pem
└── test_envelope.bin
```

Load fixtures:

```rust
let data = include_bytes!("fixtures/test_message.json");
let message: Message = serde_json::from_slice(data)?;
```

## Mocking

For unit tests that need external dependencies:

```rust
use mockall::*;

#[automock]
trait Transport {
    fn send(&self, data: &[u8]) -> Result<()>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockTransport::new();
    mock.expect_send()
        .returning(|_| Ok(()));

    assert!(mock.send(b"test").is_ok());
}
```

## Questions & Issues

If tests fail:

1. Check the test output: `cargo test -- --nocapture`
2. Enable logging: `RUST_LOG=debug cargo test`
3. Run single test: `cargo test test_name -- --nocapture`
4. Check for timing issues (increase timeouts)
5. Verify environment (network access, ports available)

For integration test failures:
- Ensure no firewall blocking localhost connections
- Check available ports (tests use random ports)
- Increase timeout values if on slow system
- Run tests sequentially: `cargo test -- --test-threads=1`
