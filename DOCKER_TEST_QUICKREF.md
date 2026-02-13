# Docker Test Infrastructure - Quick Reference Card

## One-Line Commands

```bash
# Run everything (most common)
cd docker && ./run-all-tests.sh

# Fast iteration
cd docker && ./run-all-tests.sh --rust-only        # 3-5 min

# Test specific area
cd docker && ./run-all-tests.sh --android-only     # 5-10 min
cd docker && ./run-all-tests.sh --integration-only # 5-10 min

# Debug
cd docker && ./run-all-tests.sh --verbose          # Show all logs
cd docker && ./run-all-tests.sh --clean            # Fresh start

# Advanced
cd docker && ./run-all-tests.sh --with-nat         # Include NAT tests
cd docker && ./example-custom-test.sh              # Interactive demo
```

## Infrastructure Components

```
┌─────────────────────────────────────────────────┐
│            Docker Test Infrastructure            │
├─────────────────────────────────────────────────┤
│ Images:                                          │
│  • Dockerfile.android-test (SDK + NDK + Rust)   │
│  • Dockerfile.rust-test (Rust + tools)          │
│  • Dockerfile (production runtime)              │
├─────────────────────────────────────────────────┤
│ Services (docker-compose.test.yml):             │
│  • rust-core-test      → Run Rust tests         │
│  • android-unit-test   → Run Android tests      │
│  • mock-relay          → Bridge 2 networks      │
│  • mock-client-a       → Client on network A    │
│  • mock-client-b       → Client on network B    │
│  • integration-test    → Run E2E tests          │
│  • nat-gateway-a/b     → NAT simulation         │
├─────────────────────────────────────────────────┤
│ Networks:                                        │
│  • test-network-a (172.30.0.0/24)               │
│  • test-network-b (172.31.0.0/24)               │
│  • test-public (172.32.0.0/24)                  │
└─────────────────────────────────────────────────┘
```

## Test Results

```bash
# View results
ls -R docker/test-results/

# Structure
test-results/
├── rust/        # Cargo test output
├── android/     # JUnit XML
└── integration/ # Integration logs
```

## Manual Testing

```bash
# Start mock infrastructure
docker compose -f docker/docker-compose.test.yml --profile test up -d \
  mock-relay mock-client-a mock-client-b

# Access containers
docker exec -it scm-mock-client-a /bin/bash
docker exec -it scm-mock-client-b /bin/bash
docker exec -it scm-mock-relay /bin/bash

# View logs
docker compose -f docker/docker-compose.test.yml logs -f mock-relay

# Stop
docker compose -f docker/docker-compose.test.yml down
```

## CI/CD

```yaml
# .github/workflows/docker-test-suite.yml
Jobs:
  - rust-tests (30 min)
  - android-tests (45 min)
  - integration-tests (30 min)
  - full-suite (60 min, main only)
  - nat-tests (30 min, manual)

Artifacts uploaded with 7-14 day retention
```

## File Locations

```
Key Files:
  docker/run-all-tests.sh              → Main runner
  docker/docker-compose.test.yml       → Test infrastructure
  docker/QUICKSTART.md                 → 5-minute guide
  docker/TESTING_GUIDE.md              → Full documentation
  DOCKER_TEST_SETUP_COMPLETE.md        → Complete overview
  
Test Code:
  core/src/**/*.rs                     → Rust tests
  core/tests/*.rs                      → Integration tests
  android/app/src/test/**/*.kt         → Android tests
  android/.../MockTestHelper.kt        → Mock helpers
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Tests won't start | `docker info` to check Docker is running |
| Port conflicts | `docker compose down` to clean up |
| Out of space | `docker system prune -f` |
| Tests hang | `docker compose logs` to view errors |
| Build fails | `./run-all-tests.sh --clean` for fresh build |

## Help Commands

```bash
./run-all-tests.sh --help            # Show all options
docker compose --help                # Docker Compose help
docker exec -it <container> bash     # Access container shell
```

## Runtimes

| Command | First Run | Cached |
|---------|-----------|--------|
| All tests | 15-30 min | 5-10 min |
| Rust only | 5-10 min | 3-5 min |
| Android only | 10-15 min | 5-10 min |
| Integration only | 10-15 min | 5-10 min |

## Success Indicators

✅ Exit code 0 = all tests passed  
❌ Exit code 1 = some tests failed  
📊 Results in `docker/test-results/`  
🔍 Detailed logs in console output  

## Documentation

📖 Start here: `docker/QUICKSTART.md`  
📚 Full guide: `docker/TESTING_GUIDE.md`  
📋 Summary: `DOCKER_TEST_SETUP_COMPLETE.md`  

---
**Pro tip**: Use `--verbose` flag when debugging failures!
