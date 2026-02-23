# Android Unit Tests

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.


This directory contains unit tests for the SCMessenger Android app.

## Running Tests

### Locally
```bash
./gradlew test
```

### In Docker
```bash
cd docker
./run-all-tests.sh --android-only
```

## Test Infrastructure

### Mock Infrastructure
Tests use MockK for mocking UniFFI objects. See `MockTestHelper.kt` for common mock setups.

### Previously @Ignored Tests
Tests that were previously @Ignored due to missing mock infrastructure are now enabled.
They run in Docker with full mock support.

## Test Files

- `MeshRepositoryTest.kt` - Tests for relay enforcement and message flow
- `MeshServiceViewModelTest.kt` - ViewModel tests
- `SettingsViewModelTest.kt` - Settings management tests
- `ChatViewModelTest.kt` - Chat functionality tests
- `ContactsViewModelTest.kt` - Contact management tests
- `UniffiIntegrationTest.kt` - UniFFI boundary integration tests
- `MeshForegroundServiceTest.kt` - Service lifecycle tests

## Adding New Tests

1. Create test file in appropriate package
2. Use `MockTestHelper` for common mock setups
3. Follow existing patterns for coroutine testing
4. Run tests locally before committing

## Notes

- Tests use JUnit 4
- Coroutine testing with `kotlinx-coroutines-test`
- MockK for mocking (relaxed mocks available)
- Tests run in Docker for CI/CD consistency
