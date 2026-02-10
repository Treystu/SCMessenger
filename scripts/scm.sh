#!/bin/bash
# SCMessenger Control Script
# Usage: ./scripts/scm.sh [start|stop|restart|status|logs]

LOG_FILE="scm.log"
PID_FILE=".scm.pid"
PORT=9000

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

function log_info { echo -e "${GREEN}[INFO]${NC} $1"; }
function log_warn { echo -e "${YELLOW}[WARN]${NC} $1"; }
function log_err { echo -e "${RED}[ERROR]${NC} $1"; }

function check_running {
    # Check for running process by name, excluding grep/self
    if pgrep -f "scmessenger-cli" > /dev/null; then
        return 0
    fi
    return 1
}

function stop_all {
    log_info "Stopping SCMessenger..."
    
    # 1. Stop local process from PID file
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if ps -p "$PID" > /dev/null 2>&1; then
            kill "$PID"
            log_info "Killed process $PID from state file."
        fi
        rm "$PID_FILE"
    fi

    # 2. Aggressive cleanup of any lingering instances
    PIDS=$(pgrep -f "scmessenger-cli")
    if [ ! -z "$PIDS" ]; then
        log_warn "Found lingering local processes: $PIDS. Killing..."
        kill -9 $PIDS 2>/dev/null || true
    fi
    
    # 3. Stop Docker Containers (optional, but requested to 'kill it')
    DOCKER_IDS=$(docker ps -q --filter name="scm-")
    if [ ! -z "$DOCKER_IDS" ]; then
        log_warn "Stopping Docker containers (scm-*)..."
        docker stop $DOCKER_IDS > /dev/null
        docker rm $DOCKER_IDS > /dev/null
    fi

    log_info "All stopped."
}

function start {
    if check_running; then
        log_warn "SCMessenger is already running (PID $(pgrep -f scmessenger-cli))."
        return
    fi

    log_info "Starting SCMessenger (Local) on port $PORT..."
    nohup cargo run -p scmessenger-cli -- start --port $PORT > "$LOG_FILE" 2>&1 &
    
    # Wait for process to settle
    sleep 2
    
    if check_running; then
        NEW_PID=$(pgrep -f scmessenger-cli)
        log_info "Started successfully (PID $NEW_PID)."
        log_info "Logs are being written to $LOG_FILE"
        echo "$NEW_PID" > "$PID_FILE"
    else
        log_err "Failed to start. Check $LOG_FILE for details."
        cat "$LOG_FILE"
        rm -f "$PID_FILE"
    fi
}

function status {
    echo "--- SCMessenger Status ---"
    if check_running; then
        echo -e "Local Process: ${GREEN}RUNNING${NC} (PID $(pgrep -f scmessenger-cli))"
    else
        echo -e "Local Process: ${RED}STOPPED${NC}"
    fi
    
    echo "Docker Containers:"
    docker ps --filter name="scm-" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
}

function logs {
    if [ -f "$LOG_FILE" ]; then
        tail -f "$LOG_FILE"
    else
        log_err "No log file found ($LOG_FILE)."
    fi
}

function factory_reset {
    echo -e "${RED}WARNING: This will delete ALL local data (Identity, Contacts, History).${NC}"
    read -p "Are you sure? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        return
    fi
    
    stop_all
    
    DATA_DIR="$HOME/Library/Application Support/scmessenger"
    CONFIG_DIR="$HOME/.config/scmessenger"
    
    # Check for Linux path if Mac path empty
    if [ ! -d "$DATA_DIR" ]; then
        DATA_DIR="$HOME/.local/share/scmessenger"
    fi
    
    if [ -d "$DATA_DIR" ]; then
        log_warn "Deleting data directory: $DATA_DIR"
        rm -rf "$DATA_DIR"
    fi
    
    if [ -d "$CONFIG_DIR" ]; then
        log_warn "Deleting config directory: $CONFIG_DIR"
        rm -rf "$CONFIG_DIR"
    fi
    
    log_info "Factory reset complete. You can now start fresh."
}

case "$1" in
    "start")
        start
        ;;
    "stop")
        stop_all
        ;;
    "restart")
        stop_all
        sleep 1
        start
        ;;
    "status")
        status
        ;;
    "logs")
        logs
        ;;
    "reset")
        factory_reset
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status|logs|reset}"
        exit 1
        ;;
esac
