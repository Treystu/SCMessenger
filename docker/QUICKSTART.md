# Docker Test Infrastructure Quick Start

Get up and running with comprehensive Docker-based testing in under 5 minutes.

## Prerequisites

- Docker and Docker Compose installed
- At least 4GB RAM available for Docker
- 10GB free disk space

## Run All Tests (Fastest Start)

```bash
cd docker
./run-all-tests.sh
```

This will:
1. Build all test Docker images
2. Run Rust core library tests
3. Run Android unit tests (including previously @Ignored tests)
4. Start mock infrastructure and run integration tests
5. Generate test results in `test-results/`

Expected runtime: 15-30 minutes (first run with image builds), 5-10 minutes (subsequent runs)

## Run Individual Test Suites

### Rust Core Tests Only (Fast)
```bash
cd docker
./run-all-tests.sh --rust-only
```
Runtime: ~3-5 minutes

### Android Unit Tests Only
```bash
cd docker
./run-all-tests.sh --android-only
```
Runtime: ~5-10 minutes (includes UniFFI bindings generation)

### Integration Tests Only
```bash
cd docker
./run-all-tests.sh --integration-only
```
Runtime: ~5-10 minutes (includes mock infrastructure startup)

## Run Tests with NAT Simulation

```bash
cd docker
./run-all-tests.sh --integration-only --with-nat
```

This adds cone NAT and symmetric NAT gateways to test NAT traversal.

## Clean and Re-run

```bash
cd docker
./run-all-tests.sh --clean
```

Use this if:
- Tests fail unexpectedly
- You've modified Dockerfiles
- You want to clear all caches

## Verbose Mode

```bash
cd docker
./run-all-tests.sh --verbose
```

Shows detailed logs from all containers.

## Manual Testing with Mock Infrastructure

### Start Mock Environment
```bash
cd docker
docker compose -f docker-compose.test.yml --profile test up -d mock-relay mock-client-a mock-client-b
```

### Access Mock Nodes
```bash
# Client A
docker exec -it scm-mock-client-a /bin/bash

# Client B
docker exec -it scm-mock-client-b /bin/bash

# Relay
docker exec -it scm-mock-relay /bin/bash
```

### Run Commands Inside Containers
```bash
scm identity show
scm peers list
scm send <peer-id> "Hello!"
scm history
```

### View Logs
```bash
docker compose -f docker-compose.test.yml logs -f mock-relay
```

### Stop Mock Environment
```bash
docker compose -f docker-compose.test.yml down
```

## Test Results

Results are saved to `docker/test-results/`:

```
test-results/
├── rust/           # Rust cargo test output
├── android/        # JUnit XML from Gradle
└── integration/    # Integration test logs
```

## Common Issues

### Docker Not Running
```bash
# Start Docker daemon
sudo systemctl start docker  # Linux
# or open Docker Desktop on Mac/Windows
```

### Port Conflicts
The test infrastructure uses isolated networks and ephemeral ports, so conflicts are rare.
If you see port conflicts, check:
```bash
docker ps  # See what's already running
docker compose -f docker-compose.test.yml down  # Clean up
```

### Out of Disk Space
```bash
# Clean up Docker resources
docker system prune -f
docker volume prune -f

# Check space
docker system df
```

### Tests Hang
```bash
# Check container status
docker compose -f docker-compose.test.yml ps

# View logs
docker compose -f docker-compose.test.yml logs

# Force cleanup
docker compose -f docker-compose.test.yml down -v
```

## CI/CD Integration

Tests automatically run in GitHub Actions on push/PR to main or develop branches.

View results in:
- GitHub Actions tab
- PR checks
- Workflow artifacts (test-results.zip)

## Next Steps

- 📖 Read [TESTING_GUIDE.md](TESTING_GUIDE.md) for comprehensive documentation
- 🔧 Modify tests in `android/app/src/test/` or `core/tests/`
- 🚀 Deploy with `./manage.sh deploy` after tests pass
- 📊 Review test results in `test-results/`

## Summary

| Command | What It Does | Runtime |
|---------|--------------|---------|
| `./run-all-tests.sh` | Run everything | 15-30 min (first), 5-10 min (cached) |
| `./run-all-tests.sh --rust-only` | Rust tests only | 3-5 min |
| `./run-all-tests.sh --android-only` | Android tests only | 5-10 min |
| `./run-all-tests.sh --integration-only` | Integration tests | 5-10 min |
| `./run-all-tests.sh --clean` | Clean and run all | 20-35 min |
| `./run-all-tests.sh --verbose` | Show detailed logs | Same + logs |
| `./run-all-tests.sh --with-nat` | Include NAT tests | Same + 2-3 min |

**Most common use case during development:**
```bash
cd docker
./run-all-tests.sh --rust-only  # Fast iteration on Rust code
```

**Before committing:**
```bash
cd docker
./run-all-tests.sh  # Full test suite
```

**Debugging failures:**
```bash
cd docker
./run-all-tests.sh --verbose  # See detailed output
```

Happy testing! 🚀
