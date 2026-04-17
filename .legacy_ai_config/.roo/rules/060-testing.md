# Test Requirements

## Mandatory Testing
All code changes MUST include appropriate tests before session end.

## Test Coverage Requirements
- Core crypto operations: 100% coverage
- Transport layer: Integration tests required
- Platform bindings: Smoke tests on each platform
- API surface: Contract tests against api.udl

## Test Execution
- Run `cargo test --workspace` for Rust changes
- Run platform-specific test suites for mobile changes
- Verify interop matrix passes for cross-platform changes

## Test Documentation
- Document test scenarios in relevant test files
- Update TESTING_GUIDE.md for new test patterns
- Record test results in validation logs
