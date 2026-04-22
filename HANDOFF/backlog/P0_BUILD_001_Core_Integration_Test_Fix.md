# P0_BUILD_001: Core Integration Test Fix

**Priority:** P0 (Build Blocking)
**Platform:** Core/Rust
**Status:** Open
**Source:** REMAINING_WORK_TRACKING.md - Build Status section

## Problem Description
Core integration tests fail with rlib format errors and rustc crash on `test_address_observation.rs`. Tests reference unimplemented Phase 2+ APIs (ReputationTracker, MultiPathDelivery, etc.).

## Impact
- `cargo test --workspace` fails completely
- Blocks comprehensive testing
- Prevents CI pipeline implementation
- Hides potential regressions

## Root Cause
Integration tests reference advanced features that are implemented but not fully wired:
1. `ReputationTracker` - exists but integration tests expect full functionality
2. `MultiPathDelivery` - implemented but test dependencies not satisfied
3. Phase 2+ API expectations vs. current Phase 1 implementation

## Implementation Required
1. Fix rlib format compatibility issues
2. Resolve rustc crash in `test_address_observation.rs`
3. Mock or stub unimplemented Phase 2+ APIs for testing
4. Ensure all existing functionality has proper test coverage

## Key Files
- `core/tests/integration/` - All integration test files
- `test_address_observation.rs` - Specific crashing test
- `core/src/lib.rs` - Test configuration and setup
- Build system configuration files

## Expected Outcome
- `cargo test --workspace` passes completely
- All existing functionality properly tested
- No crashes or rlib format errors
- Foundation for CI pipeline established