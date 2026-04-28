#!/bin/bash
# SCMessenger Test Script
# Unified test execution with result reporting and build status monitoring

set -e

echo "=== SCMessenger Test Runner ==="
echo "Started: $(date -Iseconds)"
echo "================================"

# Exit codes
EXIT_SUCCESS=0
EXIT_WARNINGS=1
EXIT_FAILURES=2
EXIT_INTERRUPTED=130

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNINGS=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local name=$2
    local exit_code=$3

    if [ "$status" == "PASS" ]; then
        echo -e "${GREEN}✓ PASS${NC}: $name"
        ((PASSED_TESTS++))
    elif [ "$status" == "FAIL" ]; then
        echo -e "${RED}✗ FAIL${NC}: $name"
        ((FAILED_TESTS++))
    elif [ "$status" == "WARN" ]; then
        echo -e "${YELLOW}⚠ WARN${NC}: $name"
        ((WARNINGS++))
    fi
}

run_cargo_test() {
    local package=$1
    local test_name=$2
    local features=$3
    local mode=$4  # "normal", "ignored", "no-run"

    local test_cmd="cargo test"
    if [ -n "$package" ]; then
        test_cmd="$test_cmd -p $package"
    fi
    if [ -n "$features" ]; then
        test_cmd="$test_cmd --features $features"
    fi

    if [ "$mode" == "no-run" ]; then
        test_cmd="$test_cmd --no-run"
    elif [ "$mode" == "ignored" ]; then
        test_cmd="$test_cmd -- --ignored"
    else
        test_cmd="$test_cmd --test $test_name"
    fi

    echo "Running: $test_cmd"
    local start_time=$(date +%s)
    if $test_cmd; then
        print_status "PASS" "$package/$test_name" 0
        return 0
    else
        print_status "FAIL" "$package/$test_name" 1
        return 1
    fi
}

run_cargo_test_all() {
    local package=$1
    local features=$2

    echo "Running all tests for $package"
    local cmd="cargo test -p $package"
    if [ -n "$features" ]; then
        cmd="$cmd --features $features"
    fi

    if $cmd 2>&1; then
        return 0
    else
        return 1
    fi
}

# Phase 1: Build verification
echo ""
echo "--- Phase 1: Build Verification ---"
if cargo check --workspace --lib 2>&1; then
    echo "✅ Library check passed"
else
    echo -e "${RED}❌ Library check failed${NC}"
    exit 1
fi

# Phase 2: Unit tests (no Phase 2 APIs required)
echo ""
echo "--- Phase 2: Unit Tests (Core) ---"
if run_cargo_test_all "scmessenger-core" ""; then
    echo "✅ Core unit tests passed"
else
    echo -e "${YELLOW}⚠ Some core unit tests failed (non-critical)${NC}"
    ((WARNINGS++))
fi

# Phase 3: Core tests with Phase 2 APIs
echo ""
echo "--- Phase 3: Core Tests with Phase 2 APIs ---"
if run_cargo_test_all "scmessenger-core" "phase2_apis"; then
    echo "✅ Core tests with Phase 2 APIs passed"
else
    echo -e "${YELLOW}⚠ Some Phase 2 tests failed${NC}"
    ((WARNINGS++))
fi

# Phase 4: Specific integration tests
echo ""
echo "--- Phase 4: Integration Tests ---"
INTEGRATION_TESTS=(
    "integration_contact_block"
    "integration_e2e"
    "integration_ironcore_roundtrip"
    "integration_registration_protocol"
    "test_address_observation"
    "test_mesh_routing"
    "test_persistence_restart"
    "integration_offline_partition_matrix"
    "integration_retry_lifecycle"
    "integration_receipt_convergence"
    "integration_relay_custody"
)

for test in "${INTEGRATION_TESTS[@]}"; do
    if run_cargo_test "scmessenger-core" "$test" "phase2_apis" "normal"; then
        echo "✅ Integration test: $test"
    else
        echo -e "${YELLOW}⚠ Integration test skipped/failed: $test${NC}"
        ((WARNINGS++))
    fi
done

# Phase 5: WASM build verification
echo ""
echo "--- Phase 5: WASM Build Verification ---"
if cargo check -p scmessenger-wasm --features wasm 2>&1; then
    echo "✅ WASM build verification passed"
else
    echo -e "${YELLOW}⚠ WASM build had issues${NC}"
    ((WARNINGS++))
fi

# Phase 6: Android build verification (if on Linux)
echo ""
echo "--- Phase 6: Android Build Verification ---"
if [ -f "android/verify-build-setup.sh" ]; then
    bash android/verify-build-setup.sh
    if [ $? -eq 0 ]; then
        echo "✅ Android build verification passed"
    else
        echo -e "${YELLOW}⚠ Android build verification had issues${NC}"
        ((WARNINGS++))
    fi
else
    echo "⚠ Android verification script not found"
fi

# Summary
echo ""
echo "================================"
echo "=== Test Summary ==="
echo "Total Tests: $PASSED_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Warnings: $WARNINGS"
if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
else
    echo "Failed: $FAILED_TESTS"
fi
echo "================================"

if [ $FAILED_TESTS -gt 0 ]; then
    echo -e "${RED}Test run completed with failures${NC}"
    exit 1
elif [ $WARNINGS -gt 0 ]; then
    echo -e "${YELLOW}Test run completed with warnings${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
fi
