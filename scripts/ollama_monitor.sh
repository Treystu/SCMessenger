#!/bin/bash
# ollama_monitor.sh - Continuous Ollama health and model monitoring
# Part of SCMessenger Agent Monitoring System

set -euo pipefail

# Configuration
OLLAMA_HOST="${OLLAMA_HOST:-localhost:11434}"
CHECK_INTERVAL="${CHECK_INTERVAL:-60}" # seconds
MAX_CONSECUTIVE_FAILURES="${MAX_CONSECUTIVE_FAILURES:-3}"
PID_FILE=".claude/ollama_monitor.pid"
LOG_FILE=".claude/ollama_monitor.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    case $level in
        "INFO") echo -e "${GREEN}[INFO]${NC} $timestamp - $message" ;;
        "WARN") echo -e "${YELLOW}[WARN]${NC} $timestamp - $message" ;;
        "ERROR") echo -e "${RED}[ERROR]${NC} $timestamp - $message" ;;
        "DEBUG") echo -e "${BLUE}[DEBUG]${NC} $timestamp - $message" ;;
    esac

    # Also log to file
    echo "[$level] $timestamp - $message" >> "$LOG_FILE"
}

check_ollama_health() {
    if curl -f -s "http://$OLLAMA_HOST/api/version" > /dev/null; then
        return 0
    fi
    return 1
}

restart_ollama() {
    log "INFO" "Attempting to restart Ollama service..."

    # Try systemctl first (if running as service)
    if command -v systemctl > /dev/null 2>&1; then
        if systemctl restart ollama 2>/dev/null; then
            log "INFO" "Ollama service restarted via systemctl"
            return 0
        fi
    fi

    # Fallback: kill and restart ollama process
    pkill -f "ollama serve" 2>/dev/null || true
    sleep 2

    # Start ollama in background
    nohup ollama serve > ".claude/ollama_serve.log" 2>&1 &
    local ollama_pid=$!

    log "INFO" "Started Ollama process with PID: $ollama_pid"

    # Wait for startup
    local wait_attempts=10
    local wait_delay=3

    for attempt in $(seq 1 $wait_attempts); do
        if check_ollama_health; then
            log "INFO" "Ollama started successfully after $((attempt * wait_delay)) seconds"
            return 0
        fi
        sleep $wait_delay
    done

    log "ERROR" "Ollama failed to start after $((wait_attempts * wait_delay)) seconds"
    return 1
}

ensure_models() {
    log "INFO" "Ensuring required models are available..."

    if ./scripts/ensure_models.sh --check-only; then
        log "INFO" "All required models are available"
        return 0
    fi

    log "WARN" "Some models missing, attempting to pull..."

    if ./scripts/ensure_models.sh; then
        log "INFO" "Successfully ensured all models are available"
        return 0
    else
        log "ERROR" "Failed to ensure model availability"
        return 1
    fi
}

monitor_loop() {
    local consecutive_failures=0
    local cycle_count=0

    log "INFO" "Starting Ollama monitoring loop (interval: ${CHECK_INTERVAL}s)"

    while true; do
        ((cycle_count++))

        log "DEBUG" "Monitoring cycle $cycle_count started"

        # Check Ollama health
        if check_ollama_health; then
            log "DEBUG" "Ollama health check passed"
            consecutive_failures=0

            # Check model availability every 5 cycles (5 minutes)
            if [ $((cycle_count % 5)) -eq 0 ]; then
                if ensure_models; then
                    log "INFO" "Model availability verified"
                else
                    log "WARN" "Model availability check failed"
                fi
            fi

        else
            ((consecutive_failures++))
            log "WARN" "Ollama health check failed ($consecutive_failures/$MAX_CONSECUTIVE_FAILURES)"

            if [ $consecutive_failures -ge $MAX_CONSECUTIVE_FAILURES ]; then
                log "ERROR" "Max consecutive failures reached, attempting restart..."

                if restart_ollama; then
                    consecutive_failures=0
                    log "INFO" "Ollama restart successful"
                else
                    log "ERROR" "Ollama restart failed"
                fi
            fi
        fi

        log "DEBUG" "Monitoring cycle $cycle_count completed"
        sleep $CHECK_INTERVAL
    done
}

start_monitor() {
    log "INFO" "Starting Ollama monitor service"

    # Create PID file
    mkdir -p "$(dirname "$PID_FILE")"
    echo $$ > "$PID_FILE"

    # Setup signal handlers
    trap 'cleanup' INT TERM EXIT

    # Start monitoring loop
    monitor_loop
}

stop_monitor() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        log "INFO" "Stopping Ollama monitor (PID: $pid)"

        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid"
            log "INFO" "Monitor stopped successfully"
        else
            log "WARN" "Monitor process not running"
        fi

        rm -f "$PID_FILE"
    else
        log "WARN" "No PID file found - monitor may not be running"
    fi
}

cleanup() {
    log "INFO" "Cleaning up monitor resources"
    rm -f "$PID_FILE"
    exit 0
}

status() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            echo "Ollama monitor is running (PID: $pid)"
            return 0
        else
            echo "Ollama monitor PID file exists but process not running"
            rm -f "$PID_FILE"
            return 1
        fi
    else
        echo "Ollama monitor is not running"
        return 1
    fi
}

# Main execution
case "${1:-}" in
    "start")
        if status > /dev/null 2>&1; then
            log "WARN" "Monitor is already running"
            exit 0
        fi
        start_monitor
        ;;
    "stop")
        stop_monitor
        ;;
    "restart")
        stop_monitor
        sleep 2
        start_monitor
        ;;
    "status")
        status
        ;;
    "-h" | "--help")
        echo "Usage: ollama_monitor.sh {start|stop|restart|status|--help}"
        echo ""
        echo "Continuous Ollama health and model monitoring service"
        echo ""
        echo "Options:"
        echo "  start     Start the monitoring service"
        echo "  stop      Stop the monitoring service"
        echo "  restart   Restart the monitoring service"
        echo "  status    Check if monitor is running"
        echo "  --help    Show this help message"
        ;;
    *)
        echo "Usage: ollama_monitor.sh {start|stop|restart|status|--help}"
        exit 1
        ;;
esac