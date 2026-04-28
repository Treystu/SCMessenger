#!/usr/bin/env bash
# Orchestrator Manager v4 — Single source of truth for agent lifecycle

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
AGENTS_DIR="${SCRIPT_DIR}/agents"
HANDOFF_DIR="${REPO_ROOT}/HANDOFF"

mkdir -p "${AGENTS_DIR}"

usage() {
    echo "Usage: bash .claude/orchestrator_manager.sh pool [list|status|launch|stop]"
    echo "  list   - List agent profiles"
    echo "  status - Show active agents + HANDOFF counts"
    echo "  launch - Launch an agent (max 2 concurrent)"
    echo "  stop   - Stop an agent by ID"
}

cmd_pool_list() {
    if [[ ! -f "${SCRIPT_DIR}/agent_pool.json" ]]; then
        echo "error: agent_pool.json not found"
        exit 1
    fi
    echo "Available agents:"
    grep -o '"name": *"[^"]*"' "${SCRIPT_DIR}/agent_pool.json" | sed 's/.*"name": *"\([^"]*\)".*/  - \1/' || true
}

cmd_pool_status() {
    local count
    count=$(powershell.exe -NoProfile -Command "@(Get-Process -Name claude -ErrorAction SilentlyContinue).Count" 2>/dev/null || echo "0")
    echo "Active claude processes: $count (max 3)"
    echo ""
    echo "Agent configs:"
    ls -1 "${AGENTS_DIR}" 2>/dev/null || echo "  (none)"
    echo ""
    echo "HANDOFF state:"
    for d in todo IN_PROGRESS review done; do
        local n
        n=$(ls -1 "${HANDOFF_DIR}/${d}" 2>/dev/null | wc -l)
        echo "  ${d}: ${n} task(s)"
    done
}

cmd_pool_launch() {
    local agent_name="${1:-}"
    local task_file="${2:-}"
    if [[ -z "$agent_name" ]]; then
        echo "error: agent name required"
        usage; exit 1
    fi
    local count
    count=$(powershell.exe -NoProfile -Command "@(Get-Process -Name claude -ErrorAction SilentlyContinue).Count" 2>/dev/null || echo "0")
    if [[ "$count" -ge 3 ]]; then
        echo "error: Max 3 claude processes allowed (found $count)"
        exit 1
    fi
    local pool_file="${SCRIPT_DIR}/agent_pool.json"
    local model
    model=$(grep -A2 "\"name\": *\"${agent_name}\"" "$pool_file" 2>/dev/null | grep '"model"' | head -1 | sed 's/.*"model": *"\([^"]*\)".*/\1/')
    if [[ -z "$model" ]]; then
        echo "error: Agent '$agent_name' not found in agent_pool.json"
        exit 1
    fi
    local agent_dir="${AGENTS_DIR}/${agent_name}_$$"
    mkdir -p "$agent_dir"
    echo "$$" > "${agent_dir}/pid"
    cat > "${agent_dir}/config" <<AGENTCONF
AGENT_NAME=${agent_name}
MODEL=${model}
TASK_FILE=${task_file}
STARTED_AT=$(date -u +%Y-%m-%dT%H:%M:%SZ)
AGENTCONF
    echo "Launching agent: ${agent_name} (model: ${model})"
    [[ -n "$task_file" ]] && echo "  Task: ${task_file}"
    echo "  Config: ${agent_dir}/config"
}

cmd_pool_stop() {
    local agent_id="${1:-}"
    if [[ -z "$agent_id" ]]; then
        echo "error: agent_id required"
        exit 1
    fi
    local pid_file="${AGENTS_DIR}/${agent_id}/pid"
    if [[ ! -f "$pid_file" ]]; then
        echo "warning: No PID file for agent ${agent_id}"
        exit 1
    fi
    local pid
    pid=$(cat "$pid_file")
    echo "Stopping agent ${agent_id} (PID: ${pid})..."
    powershell.exe -NoProfile -Command "Stop-Process -Id ${pid} -Force -ErrorAction SilentlyContinue" 2>/dev/null || true
    rm -rf "${AGENTS_DIR}/${agent_id}"
    echo "Agent ${agent_id} stopped."
}

case "${1:-}" in
    pool)
        shift
        case "${1:-}" in
            list)   shift; cmd_pool_list "$@" ;;
            status) shift; cmd_pool_status "$@" ;;
            launch) shift; cmd_pool_launch "$@" ;;
            stop)   shift; cmd_pool_stop "$@" ;;
            *) echo "Unknown pool subcommand: ${1:-}"; usage; exit 1 ;;
        esac
        ;;
    *) usage; exit 1 ;;
esac
