#!/usr/bin/env bash
###############################################################################
# startup.sh — SCMessenger Cloud Worker Startup Script
#
# Runs as a GCP startup-script via instance metadata. Orchestrates the full
# worker lifecycle:
#
#   Phase 1 (boot)          → environment & toolchain setup
#   Phase 2 (clone)         → git clone + branch checkout
#   Phase 3 (build)         → cargo build --workspace
#   Phase 4 (test)          → cargo test --workspace
#   Phase 5 (agent)         → Aider AI sprint (if TASK_PROMPT set)
#   Phase 6 (commit)        → commit & push results
#   Phase 7 (self-destruct) → delete this VM
#
# All phases send heartbeats to the orchestrator so it can track progress.
###############################################################################
set -euo pipefail

readonly LOG_FILE="/tmp/worker_startup.log"
exec > >(tee -a "$LOG_FILE") 2>&1

# ─── Helpers ────────────────────────────────────────────────────────────────

log() {
  echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"
}

die() {
  log "FATAL: $*"
  heartbeat "failed" "$*"
  exit 1
}

# Read a GCP instance metadata attribute. Returns empty string on failure.
metadata() {
  local key="$1"
  curl -sf -H "Metadata-Flavor: Google" \
    "http://metadata.google.internal/computeMetadata/v1/instance/attributes/${key}" 2>/dev/null || true
}

# ─── Metadata ───────────────────────────────────────────────────────────────

log "Phase 0 — reading instance metadata"

ORCHESTRATOR_IP="$(metadata 'ORCHESTRATOR_IP')"
SPRINT_ID="$(metadata 'SPRINT_ID')"
TASK_PROMPT="$(metadata 'TASK_PROMPT')"
GIT_BRANCH="$(metadata 'GIT_BRANCH')"
OPENROUTER_API_KEY="$(metadata 'OPENROUTER_API_KEY')"

export OPENROUTER_API_KEY

# Derive instance identity
INSTANCE_NAME="$(curl -sf -H 'Metadata-Flavor: Google' \
  http://metadata.google.internal/computeMetadata/v1/instance/name)"
INSTANCE_ZONE="$(curl -sf -H 'Metadata-Flavor: Google' \
  http://metadata.google.internal/computeMetadata/v1/instance/zone | awk -F/ '{print $NF}')"

log "  ORCHESTRATOR_IP = ${ORCHESTRATOR_IP:-<unset>}"
log "  SPRINT_ID       = ${SPRINT_ID:-<unset>}"
log "  GIT_BRANCH      = ${GIT_BRANCH:-<unset>}"
log "  INSTANCE_NAME   = ${INSTANCE_NAME}"
log "  INSTANCE_ZONE   = ${INSTANCE_ZONE}"
log "  TASK_PROMPT      = ${TASK_PROMPT:+<set, ${#TASK_PROMPT} chars>}"

# ─── Heartbeat ──────────────────────────────────────────────────────────────

# POST status updates to the orchestrator. Non-fatal on failure so a missing
# orchestrator never kills the worker.
heartbeat() {
  local status="${1:-unknown}"
  local message="${2:-}"

  if [[ -z "$ORCHESTRATOR_IP" ]]; then
    log "  [heartbeat] no orchestrator configured — skipping"
    return 0
  fi

  local payload
  payload=$(cat <<EOF
{
  "sprint_id":     "${SPRINT_ID}",
  "instance_name": "${INSTANCE_NAME}",
  "zone":          "${INSTANCE_ZONE}",
  "status":        "${status}",
  "message":       "${message}",
  "timestamp":     "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
}
EOF
  )

  curl -sf -X POST \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "http://${ORCHESTRATOR_IP}:8080/api/heartbeat" \
    --connect-timeout 5 --max-time 10 \
  || log "  [heartbeat] WARNING: failed to reach orchestrator"
}

###############################################################################
# Phase 1 — Boot / Environment Setup
###############################################################################
log "Phase 1 — boot"
heartbeat "phase1_boot" "Setting up environment"

# Ensure Cargo is on PATH (rustup default install location)
export PATH="$HOME/.cargo/bin:$PATH"

# Install Rust if not present
if ! command -v cargo &>/dev/null; then
  log "  Installing Rust toolchain…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi

# Install essential packages (Debian/Ubuntu)
if command -v apt-get &>/dev/null; then
  sudo apt-get update -qq
  sudo apt-get install -y -qq git tmux build-essential pkg-config libssl-dev curl jq
fi

# Install aider if missing
if ! command -v aider &>/dev/null; then
  log "  Installing aider…"
  pip3 install --quiet aider-chat 2>/dev/null \
    || pip install --quiet aider-chat 2>/dev/null \
    || log "  WARNING: aider install failed — agent phase will be skipped"
fi

log "  Rust   : $(rustc --version 2>/dev/null || echo 'not found')"
log "  Cargo  : $(cargo --version 2>/dev/null || echo 'not found')"
log "  Git    : $(git --version 2>/dev/null || echo 'not found')"
log "  Aider  : $(aider --version 2>/dev/null || echo 'not found')"

heartbeat "phase1_done" "Environment ready"

###############################################################################
# Phase 2 — Clone Repository
###############################################################################
log "Phase 2 — clone"
heartbeat "phase2_clone" "Cloning repository"

readonly WORK_DIR="/tmp/scmessenger"
rm -rf "$WORK_DIR"

git clone --depth 50 "https://github.com/nicholasgonzalezsc/SCMessenger_Clean.git" "$WORK_DIR"
cd "$WORK_DIR"

# Checkout the target branch (create from main if it doesn't exist remotely)
if [[ -n "${GIT_BRANCH:-}" ]]; then
  if git ls-remote --exit-code --heads origin "$GIT_BRANCH" &>/dev/null; then
    log "  Checking out existing branch: $GIT_BRANCH"
    git checkout "$GIT_BRANCH"
  else
    log "  Creating new branch from main: $GIT_BRANCH"
    git checkout -b "$GIT_BRANCH" origin/main
  fi
fi

log "  Branch : $(git branch --show-current)"
log "  HEAD   : $(git rev-parse --short HEAD)"

heartbeat "phase2_done" "Repository cloned on $(git branch --show-current)"

###############################################################################
# Phase 3 — Build
###############################################################################
log "Phase 3 — build"
heartbeat "phase3_build" "Building workspace"

BUILD_OUTPUT="/tmp/build_output.txt"

if cargo build --workspace 2>&1 | tee "$BUILD_OUTPUT"; then
  BUILD_STATUS="success"
  log "  Build succeeded"
else
  BUILD_STATUS="failed"
  log "  Build FAILED — see $BUILD_OUTPUT"
fi

heartbeat "phase3_done" "Build ${BUILD_STATUS}"

if [[ "$BUILD_STATUS" == "failed" && -z "${TASK_PROMPT:-}" ]]; then
  die "Build failed and no TASK_PROMPT set to attempt repair"
fi

###############################################################################
# Phase 4 — Test
###############################################################################
log "Phase 4 — test"
heartbeat "phase4_test" "Running tests"

TEST_RESULTS="/tmp/test_results.txt"

if cargo test --workspace 2>&1 | tee "$TEST_RESULTS"; then
  TEST_STATUS="success"
  TESTS_PASSED=$(grep -c "test .* ok" "$TEST_RESULTS" 2>/dev/null || echo "?")
  log "  Tests passed (${TESTS_PASSED} ok)"
else
  TEST_STATUS="failed"
  TESTS_FAILED=$(grep -c "FAILED" "$TEST_RESULTS" 2>/dev/null || echo "?")
  log "  Tests FAILED (${TESTS_FAILED} failures)"
fi

heartbeat "phase4_done" "Tests ${TEST_STATUS}"

###############################################################################
# Phase 5 — Aider Agent Sprint
###############################################################################
if [[ -n "${TASK_PROMPT:-}" ]]; then
  log "Phase 5 — agent sprint"
  heartbeat "phase5_agent" "Starting Aider sprint"

  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

  # Start the preemption listener in the background
  PREEMPTION_SCRIPT="${SCRIPT_DIR}/preemption_listener.sh"
  if [[ -f "$PREEMPTION_SCRIPT" ]]; then
    log "  Starting preemption listener"
    bash "$PREEMPTION_SCRIPT" &
    PREEMPTION_PID=$!
    log "  Preemption listener PID: $PREEMPTION_PID"
  else
    log "  WARNING: preemption_listener.sh not found — skipping"
    PREEMPTION_PID=""
  fi

  # Launch Aider inside a tmux session so it has a proper TTY
  readonly AIDER_LOG="/tmp/aider_output.log"
  readonly TMUX_SESSION="aider-sprint"

  tmux new-session -d -s "$TMUX_SESSION" \
    "aider \
      --message '${TASK_PROMPT}' \
      --yes-always \
      --model openrouter/deepseek/deepseek-r1:free \
      --auto-test \
      --test-cmd 'cargo test --workspace 2>&1 | tail -50' \
      2>&1 | tee '${AIDER_LOG}'; \
     tmux wait-for -S aider-done"

  log "  Aider tmux session '${TMUX_SESSION}' launched"
  heartbeat "phase5_running" "Aider is working"

  # Block until the aider tmux session finishes
  tmux wait-for aider-done 2>/dev/null || true

  log "  Aider session completed"

  # Clean up preemption listener
  if [[ -n "${PREEMPTION_PID:-}" ]]; then
    kill "$PREEMPTION_PID" 2>/dev/null || true
  fi

  heartbeat "phase5_done" "Aider sprint completed"
else
  log "Phase 5 — skipped (no TASK_PROMPT)"
fi

###############################################################################
# Phase 6 — Commit & Push
###############################################################################
log "Phase 6 — commit & push"
heartbeat "phase6_commit" "Committing results"

cd "$WORK_DIR"

# Check for changes
if git diff --quiet && git diff --cached --quiet; then
  log "  No changes to commit"
  heartbeat "phase6_done" "No changes"
else
  git add -A

  COMMIT_MSG="sprint(${SPRINT_ID:-manual}): worker results

Build:  ${BUILD_STATUS:-unknown}
Tests:  ${TEST_STATUS:-unknown}
Task:   ${TASK_PROMPT:+completed}${TASK_PROMPT:-n/a}
Worker: ${INSTANCE_NAME}@${INSTANCE_ZONE}
Time:   $(date -u '+%Y-%m-%dT%H:%M:%SZ')"

  git commit -m "$COMMIT_MSG"

  if [[ -n "${GIT_BRANCH:-}" ]]; then
    git push origin "$GIT_BRANCH" || log "  WARNING: push failed"
  else
    log "  WARNING: no GIT_BRANCH set — skipping push"
  fi

  log "  Committed: $(git rev-parse --short HEAD)"
  heartbeat "phase6_done" "Pushed $(git rev-parse --short HEAD)"
fi

###############################################################################
# Phase 7 — Self-Destruct
###############################################################################
log "Phase 7 — self-destruct"
heartbeat "phase7_shutdown" "Self-destructing"

# Give the orchestrator a moment to receive the final heartbeat
sleep 5

log "  Deleting instance ${INSTANCE_NAME} in ${INSTANCE_ZONE}…"
gcloud compute instances delete "$INSTANCE_NAME" \
  --zone="$INSTANCE_ZONE" \
  --quiet \
  2>/dev/null || log "  WARNING: self-delete failed (may need manual cleanup)"

log "Startup script complete."
