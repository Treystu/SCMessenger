#!/bin/bash
# Run all Docker-based tests for SCMessenger
# This is the main entry point for the comprehensive test infrastructure

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
RUN_RUST=true
RUN_ANDROID=true
RUN_INTEGRATION=true
RUN_NAT=false
CLEAN=false
DETACH=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --rust-only)
            RUN_RUST=true
            RUN_ANDROID=false
            RUN_INTEGRATION=false
            shift
            ;;
        --android-only)
            RUN_RUST=false
            RUN_ANDROID=true
            RUN_INTEGRATION=false
            shift
            ;;
        --integration-only)
            RUN_RUST=false
            RUN_ANDROID=false
            RUN_INTEGRATION=true
            shift
            ;;
        --with-nat)
            RUN_NAT=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --detach|-d)
            DETACH=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "SCMessenger Docker Test Runner"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --rust-only           Run only Rust core tests"
            echo "  --android-only        Run only Android unit tests"
            echo "  --integration-only    Run only integration tests"
            echo "  --with-nat            Include NAT simulation tests"
            echo "  --clean               Clean up before running tests"
            echo "  --detach, -d          Run in detached mode"
            echo "  --verbose, -v         Enable verbose output"
            echo "  --help, -h            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                    # Run all tests"
            echo "  $0 --rust-only        # Run only Rust tests"
            echo "  $0 --with-nat         # Run all tests including NAT simulation"
            echo "  $0 --clean            # Clean and run all tests"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to log with color
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

# Check Docker
if ! docker info > /dev/null 2>&1; then
    log_error "Docker is not running"
    exit 1
fi

# Clean up if requested
if [ "$CLEAN" = true ]; then
    log_section "Cleaning Up Previous Test Runs"
    docker compose -f docker-compose.test.yml down -v 2>/dev/null || true
    log_info "Cleanup complete"
fi

# Build images
log_section "Building Test Infrastructure"
docker compose -f docker-compose.test.yml build

# Prepare test results directory
mkdir -p test-results/{rust,android,integration}
chmod -R 777 test-results

# Track test results
RUST_EXIT=0
ANDROID_EXIT=0
INTEGRATION_EXIT=0

# Run Rust tests
if [ "$RUN_RUST" = true ]; then
    log_section "Running Rust Core Tests"
    docker compose -f docker-compose.test.yml --profile test run --rm rust-core-test || RUST_EXIT=$?
    
    if [ $RUST_EXIT -eq 0 ]; then
        log_info "Rust core tests PASSED"
    else
        log_error "Rust core tests FAILED (exit code: $RUST_EXIT)"
    fi
fi

# Run Android tests
if [ "$RUN_ANDROID" = true ]; then
    log_section "Running Android Unit Tests"
    docker compose -f docker-compose.test.yml --profile test run --rm android-unit-test || ANDROID_EXIT=$?
    
    if [ $ANDROID_EXIT -eq 0 ]; then
        log_info "Android unit tests PASSED"
    else
        log_error "Android unit tests FAILED (exit code: $ANDROID_EXIT)"
    fi
fi

# Run Integration tests (with mock infrastructure)
if [ "$RUN_INTEGRATION" = true ]; then
    log_section "Starting Mock Infrastructure"
    
    # Determine profile based on NAT option
    PROFILE_ARG="--profile test"
    if [ "$RUN_NAT" = true ]; then
        PROFILE_ARG="--profile test --profile nat-test"
        log_info "NAT simulation enabled"
    fi
    
    # Start mock infrastructure
    docker compose -f docker-compose.test.yml $PROFILE_ARG up -d mock-relay mock-client-a mock-client-b
    
    # Wait for mock relay to be ready
    log_info "Waiting for mock infrastructure to initialize..."
    sleep 10
    
    # Check if relay is healthy
    for i in {1..30}; do
        if docker compose -f docker-compose.test.yml ps mock-relay | grep -q "healthy"; then
            log_info "Mock relay is healthy"
            break
        fi
        if [ $i -eq 30 ]; then
            log_error "Mock relay failed to become healthy"
            docker compose -f docker-compose.test.yml logs mock-relay
            INTEGRATION_EXIT=1
        fi
        sleep 2
    done
    
    if [ $INTEGRATION_EXIT -eq 0 ]; then
        log_section "Running Integration Tests"
        docker compose -f docker-compose.test.yml $PROFILE_ARG run --rm integration-test || INTEGRATION_EXIT=$?
        
        if [ $INTEGRATION_EXIT -eq 0 ]; then
            log_info "Integration tests PASSED"
        else
            log_error "Integration tests FAILED (exit code: $INTEGRATION_EXIT)"
        fi
    fi
    
    # Show logs if verbose or if tests failed
    if [ "$VERBOSE" = true ] || [ $INTEGRATION_EXIT -ne 0 ]; then
        log_section "Mock Infrastructure Logs"
        docker compose -f docker-compose.test.yml logs --tail=50 mock-relay mock-client-a mock-client-b
    fi
    
    # Cleanup mock infrastructure
    log_info "Stopping mock infrastructure..."
    docker compose -f docker-compose.test.yml $PROFILE_ARG down
fi

# Summary
log_section "Test Summary"
TOTAL_FAILURES=0

if [ "$RUN_RUST" = true ]; then
    if [ $RUST_EXIT -eq 0 ]; then
        echo -e "  Rust Core Tests:     ${GREEN}✓ PASSED${NC}"
    else
        echo -e "  Rust Core Tests:     ${RED}✗ FAILED${NC}"
        TOTAL_FAILURES=$((TOTAL_FAILURES + 1))
    fi
fi

if [ "$RUN_ANDROID" = true ]; then
    if [ $ANDROID_EXIT -eq 0 ]; then
        echo -e "  Android Unit Tests:  ${GREEN}✓ PASSED${NC}"
    else
        echo -e "  Android Unit Tests:  ${RED}✗ FAILED${NC}"
        TOTAL_FAILURES=$((TOTAL_FAILURES + 1))
    fi
fi

if [ "$RUN_INTEGRATION" = true ]; then
    if [ $INTEGRATION_EXIT -eq 0 ]; then
        echo -e "  Integration Tests:   ${GREEN}✓ PASSED${NC}"
    else
        echo -e "  Integration Tests:   ${RED}✗ FAILED${NC}"
        TOTAL_FAILURES=$((TOTAL_FAILURES + 1))
    fi
fi

echo ""
log_info "Test results saved to: test-results/"

# Exit with failure if any tests failed
if [ $TOTAL_FAILURES -gt 0 ]; then
    log_error "$TOTAL_FAILURES test suite(s) failed"
    exit 1
else
    log_info "All test suites passed!"
    exit 0
fi
