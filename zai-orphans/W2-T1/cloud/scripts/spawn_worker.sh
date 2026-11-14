#!/usr/bin/env bash
###############################################################################
# SCMessenger — Spawn Worker VM
#
# Called by the orchestrator to create a Spot/preemptible worker instance
# for a specific sprint.
#
# Usage:
#   ./spawn_worker.sh <sprint_id> <task_prompt> [git_branch] [machine_type] [zone]
#
# Arguments:
#   sprint_id     — Unique sprint identifier (required)
#   task_prompt   — Task description / prompt for the worker (required)
#   git_branch    — Git branch to work on (default: agent/sprint-<sprint_id>)
#   machine_type  — GCE machine type (default: e2-standard-8)
#   zone          — GCE zone (default: us-central1-b)
#
# Environment:
#   OPENROUTER_API_KEY — Passed to the worker (read from /etc/scm-orchestrator.env)
###############################################################################
set -euo pipefail

readonly SCRIPT_NAME="$(basename "$0")"
readonly LOG_TAG="scm-spawn-worker"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { logger -t "${LOG_TAG}" "$*"; echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
die()  { log "ERROR: $*"; exit 1; }

usage() {
  cat <<EOF
Usage: ${SCRIPT_NAME} <sprint_id> <task_prompt> [git_branch] [machine_type] [zone]

Arguments:
  sprint_id      Unique sprint identifier (required)
  task_prompt    Task description for the worker (required)
  git_branch     Git branch (default: agent/sprint-<sprint_id>)
  machine_type   GCE machine type (default: e2-standard-8)
  zone           GCE zone (default: us-central1-b)
EOF
  exit 1
}

# Fetch a value from instance metadata
metadata() {
  curl -sf -H "Metadata-Flavor: Google" \
    "http://metadata.google.internal/computeMetadata/v1/instance/attributes/${1}" 2>/dev/null || true
}

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
[[ $# -lt 2 ]] && usage

SPRINT_ID="$1"
TASK_PROMPT="$2"
GIT_BRANCH="${3:-agent/sprint-${SPRINT_ID}}"
MACHINE_TYPE="${4:-e2-standard-8}"
ZONE="${5:-us-central1-b}"

INSTANCE_NAME="scm-worker-${SPRINT_ID}"

log "Spawning worker: instance=${INSTANCE_NAME} type=${MACHINE_TYPE} zone=${ZONE}"

# ---------------------------------------------------------------------------
# Resolve orchestrator internal IP (from instance metadata)
# ---------------------------------------------------------------------------
ORCHESTRATOR_IP="$(
  curl -sf -H "Metadata-Flavor: Google" \
    "http://metadata.google.internal/computeMetadata/v1/instance/network-interfaces/0/ip" 2>/dev/null
)" || die "Failed to retrieve orchestrator internal IP from metadata"

log "Orchestrator internal IP: ${ORCHESTRATOR_IP}"

# ---------------------------------------------------------------------------
# Resolve secrets (from orchestrator env file or metadata)
# ---------------------------------------------------------------------------
if [[ -f /etc/scm-orchestrator.env ]]; then
  # shellcheck source=/dev/null
  source /etc/scm-orchestrator.env
fi
OPENROUTER_API_KEY="${OPENROUTER_API_KEY:-$(metadata OPENROUTER_API_KEY)}"
[[ -z "${OPENROUTER_API_KEY}" ]] && die "OPENROUTER_API_KEY is not set"

GCP_PROJECT="${GCP_PROJECT:-$(metadata GCP_PROJECT)}"
[[ -z "${GCP_PROJECT}" ]] && die "GCP_PROJECT is not set"

# ---------------------------------------------------------------------------
# Build gcloud create command
# ---------------------------------------------------------------------------
GCLOUD_ARGS=(
  gcloud compute instances create "${INSTANCE_NAME}"
  --project="${GCP_PROJECT}"
  --zone="${ZONE}"
  --machine-type="${MACHINE_TYPE}"
  --image-family=debian-12
  --image-project=debian-cloud
  --boot-disk-size=50GB
  --boot-disk-type=pd-ssd
  --provisioning-model=SPOT
  --instance-termination-action=DELETE
  --no-restart-on-failure
  --maintenance-policy=TERMINATE
  --tags=scm-worker
  --scopes=compute-rw,storage-ro,logging-write
  --metadata="ORCHESTRATOR_IP=${ORCHESTRATOR_IP},SPRINT_ID=${SPRINT_ID},TASK_PROMPT=${TASK_PROMPT},GIT_BRANCH=${GIT_BRANCH},OPENROUTER_API_KEY=${OPENROUTER_API_KEY}"
  --format=json
  --quiet
)

# Enable nested virtualization for n2-* machine types (required for Android emulators)
if [[ "${MACHINE_TYPE}" == n2-* ]]; then
  log "Enabling nested virtualization for ${MACHINE_TYPE}"
  GCLOUD_ARGS+=(--enable-nested-virtualization)
fi

# ---------------------------------------------------------------------------
# Create the instance
# ---------------------------------------------------------------------------
log "Creating instance ${INSTANCE_NAME}…"
if OUTPUT=$("${GCLOUD_ARGS[@]}" 2>&1); then
  log "Instance ${INSTANCE_NAME} created successfully."
  # Extract external IP from JSON output
  EXTERNAL_IP=$(echo "${OUTPUT}" | jq -r '.[0].networkInterfaces[0].accessConfigs[0].natIP // "none"' 2>/dev/null || echo "unknown")
  log "Worker external IP: ${EXTERNAL_IP}"
else
  die "Failed to create instance ${INSTANCE_NAME}: ${OUTPUT}"
fi

log "Worker spawn complete: ${INSTANCE_NAME} in ${ZONE}"
