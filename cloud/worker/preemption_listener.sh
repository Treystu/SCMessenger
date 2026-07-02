#!/usr/bin/env bash
###############################################################################
# preemption_listener.sh — GCP Spot/Preemptible VM Preemption Daemon
#
# Runs in the background during an Aider sprint. Polls GCP metadata every 5s
# for preemption or maintenance signals. When detected, performs an emergency
# state save:
#
#   1. Kill running cargo build/test processes
#   2. Capture Aider tmux pane output
#   3. Save sprint state to .sprint-state/resume.json
#   4. git add -A && commit && push --force
#   5. Notify orchestrator via POST /api/preempted
#
# Usage:
#   bash preemption_listener.sh &
#
# Environment (read from GCP metadata or inherited):
#   ORCHESTRATOR_IP, SPRINT_ID, GIT_BRANCH
###############################################################################
set -euo pipefail

readonly POLL_INTERVAL=5
readonly LOG_PREFIX="[preemption]"
readonly METADATA_BASE="http://metadata.google.internal/computeMetadata/v1"
readonly METADATA_HEADER="Metadata-Flavor: Google"
readonly WORK_DIR="/tmp/scmessenger"

# ─── Helpers ────────────────────────────────────────────────────────────────

log() {
  echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] ${LOG_PREFIX} $*"
}

# Fetch a metadata value. Returns empty string on failure.
meta() {
  curl -sf -H "$METADATA_HEADER" "${METADATA_BASE}/$1" 2>/dev/null || true
}

# ─── Read identity ──────────────────────────────────────────────────────────

ORCHESTRATOR_IP="${ORCHESTRATOR_IP:-$(meta 'instance/attributes/ORCHESTRATOR_IP')}"
SPRINT_ID="${SPRINT_ID:-$(meta 'instance/attributes/SPRINT_ID')}"
GIT_BRANCH="${GIT_BRANCH:-$(meta 'instance/attributes/GIT_BRANCH')}"
INSTANCE_NAME="$(meta 'instance/name')"
INSTANCE_ZONE="$(meta 'instance/zone' | awk -F/ '{print $NF}')"

log "Preemption listener started"
log "  Instance : ${INSTANCE_NAME}@${INSTANCE_ZONE}"
log "  Sprint   : ${SPRINT_ID:-<unset>}"
log "  Polling every ${POLL_INTERVAL}s"

# ─── Emergency Save ────────────────────────────────────────────────────────

emergency_save() {
  local trigger_reason="${1:-unknown}"

  log "🚨 PREEMPTION DETECTED — reason: ${trigger_reason}"
  log "   Beginning emergency state save…"

  # ── 1. Kill build/test processes ──────────────────────────────────────
  log "   [1/5] Killing cargo processes"
  pkill -f "cargo build" 2>/dev/null || true
  pkill -f "cargo test"  2>/dev/null || true
  pkill -f "rustc"       2>/dev/null || true
  sleep 1

  # ── 2. Capture Aider tmux output ──────────────────────────────────────
  log "   [2/5] Capturing tmux output"
  local aider_capture="/tmp/aider_preempted_output.txt"
  if tmux has-session -t aider-sprint 2>/dev/null; then
    tmux capture-pane -t aider-sprint -p -S -500 > "$aider_capture" 2>/dev/null || true
    tmux kill-session -t aider-sprint 2>/dev/null || true
    log "     Captured $(wc -l < "$aider_capture") lines from aider pane"
  else
    echo "No aider session was running at preemption time." > "$aider_capture"
    log "     No aider tmux session found"
  fi

  # ── 3. Save sprint state ──────────────────────────────────────────────
  log "   [3/5] Writing resume state"
  cd "$WORK_DIR" 2>/dev/null || {
    log "     WARNING: work dir $WORK_DIR not found"
    return 1
  }

  mkdir -p .sprint-state

  # Count dirty files
  local dirty_count
  dirty_count=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')

  # Detect current phase from startup log
  local current_phase="unknown"
  if [[ -f /tmp/worker_startup.log ]]; then
    current_phase=$(grep -oP 'Phase \d+ — \K\w+' /tmp/worker_startup.log | tail -1 || echo "unknown")
  fi

  cat > .sprint-state/resume.json <<RESUME_EOF
{
  "sprint_id":      "${SPRINT_ID:-unknown}",
  "preempted_at":   "$(date -u '+%Y-%m-%dT%H:%M:%SZ')",
  "instance_name":  "${INSTANCE_NAME}",
  "zone":           "${INSTANCE_ZONE}",
  "trigger":        "${trigger_reason}",
  "phase":          "${current_phase}",
  "git_branch":     "${GIT_BRANCH:-unknown}",
  "git_sha":        "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
  "dirty_files":    ${dirty_count},
  "aider_capture":  ".sprint-state/aider_last_output.txt"
}
RESUME_EOF

  # Copy aider capture into the state dir
  cp "$aider_capture" .sprint-state/aider_last_output.txt 2>/dev/null || true

  # Copy logs
  cp /tmp/worker_startup.log .sprint-state/worker.log 2>/dev/null || true
  cp /tmp/test_results.txt   .sprint-state/test_results.txt 2>/dev/null || true
  cp /tmp/build_output.txt   .sprint-state/build_output.txt 2>/dev/null || true

  log "     State saved to .sprint-state/resume.json"
  log "     Dirty files: ${dirty_count}"

  # ── 4. Git commit & force-push ────────────────────────────────────────
  log "   [4/5] Emergency commit & push"
  git add -A
  git commit -m "PREEMPTED(${SPRINT_ID:-unknown}): emergency state save

Trigger:     ${trigger_reason}
Phase:       ${current_phase}
Dirty files: ${dirty_count}
Time:        $(date -u '+%Y-%m-%dT%H:%M:%SZ')
Instance:    ${INSTANCE_NAME}@${INSTANCE_ZONE}" \
    2>/dev/null || log "     WARNING: commit failed (maybe nothing to commit)"

  local git_sha
  git_sha="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"

  if [[ -n "${GIT_BRANCH:-}" ]]; then
    git push --force origin "$GIT_BRANCH" 2>/dev/null \
      || log "     WARNING: force-push failed"
    log "     Pushed ${git_sha} to ${GIT_BRANCH}"
  else
    log "     WARNING: no GIT_BRANCH — skipping push"
  fi

  # ── 5. Notify orchestrator ────────────────────────────────────────────
  log "   [5/5] Notifying orchestrator"
  if [[ -n "${ORCHESTRATOR_IP:-}" ]]; then
    local payload
    payload=$(cat <<NOTIFY_EOF
{
  "sprint_id":     "${SPRINT_ID:-unknown}",
  "git_sha":       "${git_sha}",
  "git_branch":    "${GIT_BRANCH:-unknown}",
  "instance_name": "${INSTANCE_NAME}",
  "zone":          "${INSTANCE_ZONE}",
  "trigger":       "${trigger_reason}",
  "phase":         "${current_phase}",
  "dirty_files":   ${dirty_count},
  "preempted_at":  "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
}
NOTIFY_EOF
    )

    curl -sf -X POST \
      -H "Content-Type: application/json" \
      -d "$payload" \
      "http://${ORCHESTRATOR_IP}:8080/api/preempted" \
      --connect-timeout 3 --max-time 5 \
    || log "     WARNING: failed to notify orchestrator"

    log "     Orchestrator notified"
  else
    log "     No orchestrator configured — skipping notification"
  fi

  log "🏁 Emergency save complete — VM will be reclaimed shortly"
}

# ─── Main Polling Loop ──────────────────────────────────────────────────────

while true; do
  # Check the preempted flag
  preempted="$(meta 'instance/preempted')"
  if [[ "$preempted" == "TRUE" ]]; then
    emergency_save "preempted_flag"
    exit 0
  fi

  # Check the maintenance-event (returns NONE when nothing is scheduled)
  maintenance="$(meta 'instance/maintenance-event')"
  if [[ -n "$maintenance" && "$maintenance" != "NONE" ]]; then
    emergency_save "maintenance_event:${maintenance}"
    exit 0
  fi

  sleep "$POLL_INTERVAL"
done
