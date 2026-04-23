# P0_WINDOWS_001_CLI_Integration_Test_Stabilization

**Priority:** P0
**Type:** BUILD
**Platform:** Windows CLI (Rust)
**Estimated LoC Impact:** 200–400 LoC

## Objective
Stabilize the Windows CLI integration tests and add BLE daemon edge-case handling. The CLI binary compiles cleanly but has zero test coverage.

## Background
`cargo test -p scmessenger-cli` runs 0 tests. `cargo test --workspace` fails in `core/tests/test_mesh_routing.rs` with 27 errors (unresolved types). The CLI needs:
1. Unit tests for command parsing and identity operations
2. Integration tests for message relay and peer discovery
3. BLE daemon edge-case handling (device not found, permission denied)

## Requirements
1. **CLI Unit Tests** (~100 LoC)
   - Test command argument parsing
   - Test identity creation flow
   - Test contact add/remove/search
   
2. **CLI Integration Tests** (~150 LoC)
   - Test two CLI instances can exchange messages via local relay
   - Test identity backup/restore roundtrip
   - Test block/unblock cascade

3. **BLE Daemon Edge Cases** (~150 LoC)
   - Handle Windows BLE adapter not present
   - Handle permission denied on Windows
   - Graceful fallback when BLE is unavailable

## Verification Checklist
- [ ] `cargo test -p scmessenger-cli` runs and passes ≥10 tests
- [ ] `cargo test --workspace` still passes library tests
- [ ] `cargo clippy -p scmessenger-cli` passes
- [ ] Windows build produces runnable `.exe`

## Files to Modify
- `cli/src/main.rs` — add testable modules
- `cli/src/commands.rs` — add unit tests
- `cli/tests/integration.rs` — new file
- `core/src/transport/ble.rs` or `cli/src/ble.rs` — edge case handling

## Rollback
`git restore` if tests break native targets.

[NATIVE_SUB_AGENT: RESEARCH] — Trace existing CLI test infrastructure before writing tests.
