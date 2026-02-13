# SCMessenger Docker Test Infrastructure Guide

Comprehensive guide for the Docker-based testing infrastructure that enables "real" testing in isolated containers.

## Overview

This infrastructure provides a complete Docker-based testing environment that allows:
- **Android unit tests** with MockK running in isolated containers
- **Rust core tests** with full feature support
- **Integration tests** with real multi-node mesh networking
- **Mock infrastructure** with configurable NAT, latency, and network conditions
- **CI/CD integration** for automated testing

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Docker Test Infrastructure                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │  Android Tests   │  │   Rust Tests     │  │  Integration  │ │
│  │  - Unit tests    │  │  - Core library  │  │  - E2E tests  │ │
│  │  - MockK mocks   │  │  - All features  │  │  - Multi-node │ │
│  │  - UniFFI tests  │  │  - Cross-compile │  │  - Real mesh  │ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │             Mock Infrastructure (Real Containers)           │ │
│  ├────────────────────────────────────────────────────────────┤ │
│  │  Relay Nodes │ Client Nodes │ NAT Gateways │ Network Sim  │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Run All Tests
```bash
cd docker
./run-all-tests.sh
```

### Run Specific Test Suites
```bash
# Rust core tests only
./run-all-tests.sh --rust-only

# Android unit tests only
./run-all-tests.sh --android-only

# Integration tests only
./run-all-tests.sh --integration-only

# All tests with NAT simulation
./run-all-tests.sh --with-nat
```

### Clean and Run
```bash
# Clean up previous runs and start fresh
./run-all-tests.sh --clean
```

## Test Suites

### 1. Rust Core Tests

**What it tests:**
- All Rust core library functionality
- Identity, crypto, messaging, storage
- Transport layer, routing, relay logic
- Privacy features (onion routing, cover traffic)

**How to run:**
```bash
docker compose -f docker-compose.test.yml --profile test run --rm rust-core-test
```

**Location:** `core/src/**/*.rs` with `#[cfg(test)]` modules
**Results:** Console output + test results in `test-results/rust/`

### 2. Android Unit Tests

**What it tests:**
- MeshRepository enforcement logic (relay=messaging)
- ViewModel state management
- UI component behavior
- UniFFI boundary integration

**How to run:**
```bash
docker compose -f docker-compose.test.yml --profile test run --rm android-unit-test
```

**Location:** `android/app/src/test/`
**Results:** JUnit XML in `test-results/android/`

**Note:** Previously @Ignored tests now run with mock infrastructure!

### 3. Integration Tests

**What it tests:**
- Multi-node mesh networking
- Cross-network message delivery
- Relay routing and DHT discovery
- NAT traversal and hole punching
- End-to-end encryption

**How to run:**
```bash
# Start mock infrastructure first
docker compose -f docker-compose.test.yml --profile test up -d mock-relay mock-client-a mock-client-b

# Run integration tests
docker compose -f docker-compose.test.yml --profile test run --rm integration-test

# Cleanup
docker compose -f docker-compose.test.yml down
```

**Location:** `core/tests/integration_*.rs`
**Results:** Console output + test results in `test-results/integration/`

## Mock Infrastructure

### Components

1. **Mock Relay Node**
   - Bridges multiple networks
   - Provides bootstrap and discovery
   - Logs all relay activity
   - Accessible at `172.30.0.10:4001` and `172.31.0.10:4001`

2. **Mock Client Nodes**
   - Client A on network-a (`172.30.0.0/24`)
   - Client B on network-b (`172.31.0.0/24`)
   - Real SCMessenger instances
   - Can send/receive messages

3. **NAT Gateways** (with `--with-nat` flag)
   - Cone NAT for network-a
   - Symmetric NAT for network-b
   - Simulates real-world NAT scenarios

### Manual Testing with Mock Infrastructure

```bash
# Start the infrastructure
docker compose -f docker-compose.test.yml --profile test up -d mock-relay mock-client-a mock-client-b

# Access client A shell
docker exec -it scm-mock-client-a /bin/bash

# In client A container
scm identity show
scm peers list
scm send <peer-id> "Hello from client A"

# Access client B shell (in another terminal)
docker exec -it scm-mock-client-b /bin/bash

# View logs
docker compose -f docker-compose.test.yml logs -f mock-relay

# Cleanup when done
docker compose -f docker-compose.test.yml down
```

## Docker Images

### 1. Android Test Image (`Dockerfile.android-test`)
- Ubuntu 22.04 base
- Android SDK 34, NDK 26.1.10909125
- Java 17, Gradle
- Rust toolchain with Android targets
- cargo-ndk for cross-compilation

### 2. Rust Test Image (`Dockerfile.rust-test`)
- Rust latest official image
- All Rust components (clippy, rustfmt)
- Android targets for cross-compilation
- Code coverage tools (tarpaulin)
- Security audit tools (cargo-audit)

### 3. SCMessenger Runtime Image (`Dockerfile`)
- Production-ready image
- Minimal runtime dependencies
- Used for mock nodes and integration testing

## Test Results

All test results are saved to `docker/test-results/`:

```
test-results/
├── rust/           # Rust test outputs
├── android/        # JUnit XML from Android tests
└── integration/    # Integration test logs
```

## CI/CD Integration

### GitHub Actions

The infrastructure is designed to work with GitHub Actions:

```yaml
name: Docker Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run all tests
        run: |
          cd docker
          ./run-all-tests.sh
      - name: Upload test results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: docker/test-results/
```

## Configuration

### Environment Variables

**Rust Tests:**
- `RUST_LOG=debug` - Enable debug logging
- `RUST_BACKTRACE=1` - Show backtraces on panic

**Android Tests:**
- `ANDROID_HOME=/opt/android-sdk` - SDK location
- `ANDROID_NDK_HOME` - NDK location
- `RUSTFLAGS=-C link-arg=-Wl,-z,max-page-size=16384` - Android 15+ compatibility

**Mock Nodes:**
- `LISTEN_PORT` - Port to listen on
- `BOOTSTRAP_NODES` - Comma-separated multiaddrs
- `NODE_NAME` - Human-readable node identifier
- `RUST_LOG` - Logging level

### Network Configuration

Test networks use isolated subnets:
- `test-network-a`: 172.30.0.0/24
- `test-network-b`: 172.31.0.0/24
- `test-public`: 172.32.0.0/24

## Troubleshooting

### Tests Fail to Start

```bash
# Check Docker status
docker info

# Check for port conflicts
netstat -an | grep LISTEN | grep 4001

# Clean up everything
docker compose -f docker-compose.test.yml down -v
docker system prune -f
```

### Android Build Fails

```bash
# Verify Android SDK installation
docker compose -f docker-compose.test.yml run --rm android-unit-test bash -c "sdkmanager --list"

# Check Rust toolchain
docker compose -f docker-compose.test.yml run --rm android-unit-test bash -c "rustup show"

# Rebuild from scratch
docker compose -f docker-compose.test.yml build --no-cache android-unit-test
```

### Mock Infrastructure Not Healthy

```bash
# Check relay logs
docker compose -f docker-compose.test.yml logs mock-relay

# Check network connectivity
docker compose -f docker-compose.test.yml exec mock-relay ss -tunlp

# Restart with verbose logging
docker compose -f docker-compose.test.yml down
RUST_LOG=trace docker compose -f docker-compose.test.yml up mock-relay
```

### Integration Tests Timeout

```bash
# Increase wait time in run-all-tests.sh (line ~165)
# Or manually start and verify infrastructure first

docker compose -f docker-compose.test.yml --profile test up -d mock-relay mock-client-a mock-client-b
docker compose -f docker-compose.test.yml ps  # Should show all healthy
docker compose -f docker-compose.test.yml logs  # Check for errors
```

## Best Practices

1. **Always clean up** between test runs if you modify infrastructure
   ```bash
   ./run-all-tests.sh --clean
   ```

2. **Use verbose mode** when debugging
   ```bash
   ./run-all-tests.sh --verbose
   ```

3. **Run tests in isolation** during development
   ```bash
   ./run-all-tests.sh --rust-only  # Fast iteration on Rust code
   ```

4. **Check test results** even when tests pass
   ```bash
   ls -R test-results/
   ```

5. **Keep Docker clean** to avoid resource issues
   ```bash
   docker system prune -f
   docker volume prune -f
   ```

## Advanced Usage

### Custom Test Scenarios

Create your own test scenarios by extending `docker-compose.test.yml`:

```yaml
services:
  my-custom-test:
    image: scmessenger:latest
    environment:
      - CUSTOM_VAR=value
    networks:
      - test-network-a
    # Your custom test setup
```

### Performance Testing

Add bandwidth and latency constraints:

```yaml
mock-client-a:
  # ... existing config ...
  cap_add:
    - NET_ADMIN
  command: >
    sh -c "
      tc qdisc add dev eth0 root tbf rate 10mbit burst 32kbit latency 400ms &&
      exec /usr/local/bin/entrypoint.sh
    "
```

### Debugging Containers

Enter running containers for interactive debugging:

```bash
# Android test container
docker compose -f docker-compose.test.yml run --rm android-unit-test bash

# Rust test container
docker compose -f docker-compose.test.yml run --rm rust-core-test bash

# Mock relay
docker exec -it scm-mock-relay /bin/bash
```

## Maintenance

### Updating Dependencies

When updating Rust or Android dependencies:

1. Update `Dockerfile.rust-test` and `Dockerfile.android-test`
2. Rebuild images: `docker compose -f docker-compose.test.yml build --no-cache`
3. Run full test suite: `./run-all-tests.sh --clean`

### Adding New Tests

1. Add tests to appropriate directory (`core/tests/`, `android/app/src/test/`)
2. No changes needed to Docker infrastructure
3. Tests automatically picked up on next run

## Support

For issues or questions:
- Check this guide's troubleshooting section
- Review Docker logs: `docker compose -f docker-compose.test.yml logs`
- Check test results: `ls -R test-results/`
- Open an issue on GitHub with logs and error messages

## Summary

This Docker test infrastructure provides:
- ✅ Isolated, reproducible test environments
- ✅ Real multi-node mesh testing without physical devices
- ✅ Mock infrastructure that behaves like production
- ✅ Android unit tests with proper mocking
- ✅ Comprehensive integration testing
- ✅ Easy CI/CD integration
- ✅ Developer-friendly debugging tools

Run `./run-all-tests.sh` to get started!
