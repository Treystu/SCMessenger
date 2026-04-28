#!/bin/bash
# check_ollama_models.sh - Verify required Ollama models are available
# Part of SCMessenger Agent Monitoring System

set -euo pipefail

# Configuration
OLLAMA_HOST="${OLLAMA_HOST:-localhost:11434}"
REQUIRED_MODELS=(
    "glm-5.1:cloud"
    "qwen3-coder:480b:cloud"
    "deepseek-v3.2:cloud"
    "mistral-large-3:675b:cloud"
    "devstral-2:123b:cloud"
    "gemma4:31b:cloud"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

check_ollama_health() {
    local max_retries=3
    local retry_delay=2

    for attempt in $(seq 1 $max_retries); do
        if curl -f -s "http://$OLLAMA_HOST/api/version" > /dev/null; then
            return 0
        fi

        if [ $attempt -lt $max_retries ]; then
            log_warning "Ollama health check attempt $attempt failed, retrying in ${retry_delay}s..."
            sleep $retry_delay
            retry_delay=$((retry_delay * 2)) # Exponential backoff
        fi
    done

    log_error "Ollama service unavailable after $max_retries attempts"
    return 1
}

list_available_models() {
    local response
    response=$(curl -s "https://ollama.com/api/tags" 2>/dev/null || true)

    if [ -z "$response" ]; then
        log_error "Failed to list models - empty response from Ollama"
        return 1
    fi

    # Try to parse with jq if available, otherwise use grep
    if command -v jq > /dev/null 2>&1; then
        if ! echo "$response" | jq -e '.models' > /dev/null 2>&1; then
            log_error "Invalid JSON response from Ollama"
            return 1
        fi
        echo "$response" | jq -r '.models[].name + ":cloud"'
    else
        # Fallback: extract model names with grep/sed
        echo "$response" | grep -o '"name"\s*:\s*"[^"]*"' | sed 's/"name"\s*:\s*"\([^"]*\)"/\1:cloud/'
    fi
}

check_required_models() {
    local available_models
    local missing_models=()
    local available_count=0

    if ! available_models=$(list_available_models); then
        return 1
    fi

    log_info "Checking required models availability..."

    for required_model in "${REQUIRED_MODELS[@]}"; do
        if echo "$available_models" | grep -q "^$required_model$"; then
            log_info "✓ Model available: $required_model"
            ((available_count++))
        else
            log_warning "✗ Model missing: $required_model"
            missing_models+=("$required_model")
        fi
    done

    local total_required=${#REQUIRED_MODELS[@]}
    log_info "Model availability: $available_count/$total_required"

    if [ ${#missing_models[@]} -gt 0 ]; then
        log_warning "Missing models: ${missing_models[*]}"
        return 1
    fi

    return 0
}

main() {
    log_info "Starting Ollama model availability check"

    if ! check_ollama_health; then
        log_error "Cannot proceed - Ollama service unavailable"
        exit 1
    fi

    if check_required_models; then
        log_info "All required models are available"
        exit 0
    else
        log_error "Some required models are missing"
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    "-h" | "--help")
        echo "Usage: check_ollama_models.sh [--help]"
        echo "Check if required Ollama models are available and loaded."
        echo ""
        echo "Required models:"
        for model in "${REQUIRED_MODELS[@]}"; do
            echo "  - $model"
        done
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac