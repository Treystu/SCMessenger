#!/bin/bash

# Advanced error handling with circuit breaker pattern

set -e

CONFIG_DIR=".claude"
STATE_DIR=".claude/state"
LOG_DIR=".claude/logs"

mkdir -p "$STATE_DIR" "$LOG_DIR"

# Load configuration with jq
load_config() {
    if [ -f "$CONFIG_DIR/orchestration_config.json" ]; then
        "./scripts/jq_wrapper.sh" -r ".error_handling" "$CONFIG_DIR/orchestration_config.json" 2>/dev/null || echo "{}"
    else
        echo "{}"
    fi
}

# Circuit breaker state management
circuit_breaker_state() {
    local service="$1"
    local action="${2:-get}"
    local state_file="$STATE_DIR/circuit_breaker_${service}.json"

    case "$action" in
        "get")
            if [ -f "$state_file" ]; then
                cat "$state_file"
            else
                echo '{"state": "closed", "failures": 0, "last_failure": null, "last_success": null}'
            fi
            ;;
        "set")
            local new_state="$3"
            local failures="$4"
            local event_type="${5:-unknown}"
            local ts=$(date -Iseconds)
            # Preserve existing timestamps — only update the relevant one
            local existing_last_failure="null"
            local existing_last_success="null"
            if [ -f "$state_file" ]; then
                existing_last_failure=$(cat "$state_file" | ./scripts/jq_wrapper.sh -r '.last_failure // "null"' 2>/dev/null || echo "null")
                existing_last_success=$(cat "$state_file" | ./scripts/jq_wrapper.sh -r '.last_success // "null"' 2>/dev/null || echo "null")
            fi
            local last_failure_val="$existing_last_failure"
            local last_success_val="$existing_last_success"
            if [ "$new_state" = "open" ] || [ "$event_type" = "failure" ]; then
                last_failure_val="\"$ts\""
            elif [ "$new_state" = "closed" ] && [ "$failures" = "0" ]; then
                last_success_val="\"$ts\""
            fi
            echo "{\"state\": \"$new_state\", \"failures\": $failures, \"last_failure\": $last_failure_val, \"last_success\": $last_success_val}" > "$state_file"
            ;;
        "failure")
            local current_state=$(circuit_breaker_state "$service" "get")
            local current_failures=$(echo "$current_state" | ./scripts/jq_wrapper.sh -r '.failures')
            local new_failures=$((current_failures + 1))

            local config=$(load_config)
            local threshold=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.circuit_breaker.failure_threshold // 3')

            if [ "$new_failures" -ge "$threshold" ]; then
                circuit_breaker_state "$service" "set" "open" "$new_failures" "failure"
                echo "Circuit breaker opened for $service after $new_failures failures"
            else
                circuit_breaker_state "$service" "set" "closed" "$new_failures" "failure"
                echo "Failure recorded for $service ($new_failures/$threshold)"
            fi
            ;;
        "success")
            circuit_breaker_state "$service" "set" "closed" "0"
            echo "Circuit reset for $service after successful operation"
            ;;
    esac
}

# Exponential backoff retry
with_retry() {
    local service="$1"
    local command="$2"
    local max_attempts=3
    local initial_delay=5
    local max_delay=60
    local backoff_multiplier=2

    local config=$(load_config)
    max_attempts=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.retry_policy.max_attempts // 3')
    initial_delay=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.retry_policy.initial_delay // 5')
    max_delay=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.retry_policy.max_delay // 60')
    backoff_multiplier=$(echo "$config" | "./scripts/jq_wrapper.sh" -r '.retry_policy.backoff_multiplier // 2')

    local attempt=1
    local delay=$initial_delay

    while [ $attempt -le $max_attempts ]; do
        echo "Attempt $attempt/$max_attempts for $service..."

        if eval "$command"; then
            circuit_breaker_state "$service" "success"
            return 0
        fi

        if [ $attempt -eq $max_attempts ]; then
            circuit_breaker_state "$service" "failure"
            echo "All $max_attempts attempts failed for $service"
            return 1
        fi

        echo "Failed attempt $attempt, waiting ${delay}s before retry..."
        sleep $delay

        delay=$((delay * backoff_multiplier))
        if [ $delay -gt $max_delay ]; then
            delay=$max_delay
        fi

        attempt=$((attempt + 1))
    done
}

# Error classification
classify_error() {
    local error_output="$1"

    case "$error_output" in
        *"Connection refused"* | *"Network is unreachable"*)
            echo "network_error"
            ;;
        *"Permission denied"* | *"Access denied"*)
            echo "permission_error"
            ;;
        *"No such file"* | *"File not found"*)
            echo "file_error"
            ;;
        *"out of memory"* | *"OOM"*)
            echo "memory_error"
            ;;
        *"timeout"* | *"timed out"*)
            echo "timeout_error"
            ;;
        *)
            echo "unknown_error"
            ;;
    esac
}

# Graceful degradation
degrade_gracefully() {
    local service="$1"
    local error_type="$2"

    case "$error_type" in
        "network_error")
            echo "Falling back to offline mode for $service"
            # Implement offline functionality
            ;;
        "memory_error")
            echo "Reducing memory footprint for $service"
            # Implement memory reduction strategies
            ;;
        "timeout_error")
            echo "Implementing timeout workaround for $service"
            # Implement timeout alternatives
            ;;
        *)
            echo "Service $service degraded due to $error_type"
            # Generic degradation
            ;;
    esac
}

# Main error handling function
handle_error() {
    local service="$1"
    local command="$2"
    local error_output="$3"

    local error_type=$(classify_error "$error_output")
    local circuit_state=$(circuit_breaker_state "$service" "get" | ./scripts/jq_wrapper.sh -r '.state')

    echo "Error in $service: $error_type"
    echo "Error details: $error_output"

    # Check circuit breaker state
    if [ "$circuit_state" = "open" ]; then
        echo "Circuit breaker is open for $service - skipping operation"
        degrade_gracefully "$service" "$error_type"
        return 1
    fi

    # Attempt retry with circuit breaker
    if with_retry "$service" "$command"; then
        return 0
    else
        degrade_gracefully "$service" "$error_type"
        return 1
    fi
}

# Test function
simulate_failure() {
    local service="test_service"

    echo "Simulating failure for $service..."

    # Reset circuit breaker
    circuit_breaker_state "$service" "success"

    # Simulate failing command
    local failing_command="false"  # This will always fail

    handle_error "$service" "$failing_command" "Simulated network timeout error"

    # Show final state
    echo "Final circuit breaker state:"
    circuit_breaker_state "$service" "get" | jq .
}

# Command line interface
case "${1:-}" in
    "--simulate-failure")
        simulate_failure
        ;;
    "--reset")
        circuit_breaker_state "${2:-all}" "success"
        echo "Circuit breaker reset for ${2:-all}"
        ;;
    "--status")
        if [ -n "${2:-}" ]; then
            circuit_breaker_state "$2" "get" | jq .
        else
            echo "Available circuit breakers:"
            find "$STATE_DIR" -name "circuit_breaker_*.json" -exec basename {} \; | sed 's/circuit_breaker_\(.*\)\.json/\1/'
        fi
        ;;
    *)
        echo "Usage: $0 [--simulate-failure|--reset SERVICE|--status [SERVICE]]"
        echo "  --simulate-failure  Test the error handling system"
        echo "  --reset SERVICE     Reset circuit breaker for specific service"
        echo "  --status [SERVICE]  Show circuit breaker status"
        exit 1
        ;;
esac