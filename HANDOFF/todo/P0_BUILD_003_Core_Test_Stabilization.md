# P0_BUILD_003: Core Integration Test Stabilization

## Status: 🔴 P0 BLOCKER - Build System Unstable
**Source:** REMAINING_WORK_TRACKING.md (Build Status), MASTER_BUG_TRACKER.md (Various test failures)

## Problem Statement
Core integration tests reference unimplemented Phase 2+ APIs causing:
- Rust compiler crashes on test_address_observation.rs
- rlib format errors 
- Test failures due to missing ReputationTracker, MultiPathDelivery APIs
- Blocks all downstream development

## Implementation Targets

### 1. Test Infrastructure Repair (~300 LoC)
**Files:** `core/tests/integration_*.rs`, `build.rs`
- Fix rlib compilation and linking issues
- Resolve compiler crashes in test modules
- Stabilize test harness configuration

### 2. API Stub Implementation (~400 LoC)
**Files:** `core/src/routing/reputation.rs`, `core/src/routing/multipath.rs`
- Minimal working stubs for ReputationTracker
- Basic MultiPathDelivery interface
- Placeholder implementations for unimplemented Phase 2 APIs

### 3. Test Mocking System (~300 LoC)
**Files:** `core/tests/mocks/`, `core/src/lib.rs` (test cfg)
- Mock implementations for testing
- Feature flags to disable incomplete functionality
- Test-specific configuration overrides

### 4. CI Pipeline Foundation (~200 LoC)
**Files:** `.github/workflows/ci.yml`, `scripts/test.sh`
- Basic CI test execution
- Test result reporting
- Build status monitoring

## Total Estimate: ~1,200 LoC

## Success Criteria
1. ✅ `cargo test --workspace` completes without crashes
2. ✅ All existing 734 lib tests pass (currently 0 failures)
3. ✅ Integration tests compile and run
4. ✅ No more rlib format errors
5. ✅ CI pipeline executes tests successfully

## Priority: IMMEDIATE
Blocking all other development until build system is stable.