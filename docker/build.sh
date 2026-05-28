#!/usr/bin/env bash
# Helper script to build SCMessenger CLI using Docker for reproducible builds
# Requirement 7.7: Docker containers for Linux builds

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

IMAGE_NAME="scmessenger-builder"
IMAGE_TAG="1.75.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Build Docker image
build_image() {
    log_info "Building Docker image: ${IMAGE_NAME}:${IMAGE_TAG}"
    docker build \
        -f "$SCRIPT_DIR/build.Dockerfile" \
        -t "${IMAGE_NAME}:${IMAGE_TAG}" \
        "$PROJECT_ROOT"
    log_info "Docker image built successfully"
}

# Build CLI binary in Docker
build_cli() {
    log_info "Building CLI binary in Docker container"
    
    # Create output directory
    mkdir -p "$PROJECT_ROOT/target/docker-release"
    
    # Run build in container
    docker run --rm \
        -v "$PROJECT_ROOT/target/docker-release:/workspace/target/release" \
        "${IMAGE_NAME}:${IMAGE_TAG}" \
        cargo build --release --bin scmessenger-cli --locked
    
    if [ -f "$PROJECT_ROOT/target/docker-release/scmessenger-cli" ]; then
        log_info "Build successful: $PROJECT_ROOT/target/docker-release/scmessenger-cli"
        ls -lh "$PROJECT_ROOT/target/docker-release/scmessenger-cli"
    else
        log_error "Build failed: binary not found"
        exit 1
    fi
}

# Run tests in Docker
run_tests() {
    log_info "Running tests in Docker container"
    docker run --rm \
        "${IMAGE_NAME}:${IMAGE_TAG}" \
        cargo test --workspace --locked
}

# Clean Docker artifacts
clean() {
    log_info "Cleaning Docker artifacts"
    docker rmi "${IMAGE_NAME}:${IMAGE_TAG}" 2>/dev/null || true
    rm -rf "$PROJECT_ROOT/target/docker-release"
    log_info "Cleanup complete"
}

# Show usage
usage() {
    cat <<EOF
Usage: $0 [COMMAND]

Commands:
    build-image    Build the Docker image
    build-cli      Build CLI binary in Docker
    test           Run tests in Docker
    clean          Remove Docker image and artifacts
    help           Show this help message

Examples:
    $0 build-image
    $0 build-cli
    $0 test

EOF
}

# Main
main() {
    case "${1:-help}" in
        build-image)
            build_image
            ;;
        build-cli)
            build_image
            build_cli
            ;;
        test)
            build_image
            run_tests
            ;;
        clean)
            clean
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            log_error "Unknown command: $1"
            usage
            exit 1
            ;;
    esac
}

main "$@"
