#!/usr/bin/env bash
###############################################################################
# SCMessenger — Cleanup Workers
#
# Finds all running scm-worker-* instances and deletes them in parallel.
# Also cleans up orphaned disks (disks with no users/attachments).
#
# Usage:
#   ./cleanup_workers.sh [--dry-run]
#
# Options:
#   --dry-run   Show what would be deleted without actually deleting anything.
###############################################################################
set -euo pipefail

readonly LOG_TAG="scm-cleanup"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { logger -t "${LOG_TAG}" "$*" 2>/dev/null || true; echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
warn() { log "WARNING: $*"; }

DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true && log "DRY RUN mode — no resources will be deleted"

# Resolve project (from metadata or gcloud config)
PROJECT="$(
  curl -sf -H "Metadata-Flavor: Google" \
    "http://metadata.google.internal/computeMetadata/v1/project/project-id" 2>/dev/null \
  || gcloud config get-value project 2>/dev/null
)" || { log "ERROR: Cannot determine GCP project"; exit 1; }

log "Project: ${PROJECT}"

# ---------------------------------------------------------------------------
# 1. Delete running scm-worker-* instances
# ---------------------------------------------------------------------------
log "=== Scanning for running scm-worker-* instances ==="

# Fetch all RUNNING instances whose names start with scm-worker-
WORKERS_JSON="$(
  gcloud compute instances list \
    --project="${PROJECT}" \
    --filter="name~'^scm-worker-' AND status=RUNNING" \
    --format="json(name,zone)" \
    --quiet 2>/dev/null
)" || WORKERS_JSON="[]"

WORKER_COUNT="$(echo "${WORKERS_JSON}" | jq length)"
log "Found ${WORKER_COUNT} running worker instance(s)."

if [[ "${WORKER_COUNT}" -gt 0 ]]; then
  # Build parallel deletion jobs
  PIDS=()
  while IFS= read -r entry; do
    NAME="$(echo "${entry}" | jq -r '.name')"
    # Zone comes back as a full URI — extract just the zone name
    ZONE_FULL="$(echo "${entry}" | jq -r '.zone')"
    ZONE="$(basename "${ZONE_FULL}")"

    if [[ "${DRY_RUN}" == true ]]; then
      log "[DRY RUN] Would delete instance: ${NAME} (zone: ${ZONE})"
    else
      log "Deleting instance: ${NAME} (zone: ${ZONE})…"
      gcloud compute instances delete "${NAME}" \
        --project="${PROJECT}" \
        --zone="${ZONE}" \
        --delete-disks=all \
        --quiet &
      PIDS+=($!)
    fi
  done < <(echo "${WORKERS_JSON}" | jq -c '.[]')

  # Wait for all background deletions to finish
  FAILURES=0
  for pid in "${PIDS[@]:-}"; do
    if ! wait "${pid}" 2>/dev/null; then
      warn "Deletion job PID ${pid} failed"
      ((FAILURES++))
    fi
  done

  if [[ "${FAILURES}" -gt 0 ]]; then
    warn "${FAILURES} instance deletion(s) failed"
  else
    log "All ${WORKER_COUNT} worker instance(s) deleted successfully."
  fi
else
  log "No running worker instances to clean up."
fi

# ---------------------------------------------------------------------------
# 2. Clean up orphaned disks
# ---------------------------------------------------------------------------
log "=== Scanning for orphaned disks ==="

# Find disks that have no users (i.e. not attached to any instance)
# and whose names match the scm-worker pattern
ORPHANED_JSON="$(
  gcloud compute disks list \
    --project="${PROJECT}" \
    --filter="name~'^scm-worker-' AND -users:*" \
    --format="json(name,zone)" \
    --quiet 2>/dev/null
)" || ORPHANED_JSON="[]"

ORPHAN_COUNT="$(echo "${ORPHANED_JSON}" | jq length)"
log "Found ${ORPHAN_COUNT} orphaned disk(s)."

if [[ "${ORPHAN_COUNT}" -gt 0 ]]; then
  PIDS=()
  while IFS= read -r entry; do
    NAME="$(echo "${entry}" | jq -r '.name')"
    ZONE_FULL="$(echo "${entry}" | jq -r '.zone')"
    ZONE="$(basename "${ZONE_FULL}")"

    if [[ "${DRY_RUN}" == true ]]; then
      log "[DRY RUN] Would delete orphaned disk: ${NAME} (zone: ${ZONE})"
    else
      log "Deleting orphaned disk: ${NAME} (zone: ${ZONE})…"
      gcloud compute disks delete "${NAME}" \
        --project="${PROJECT}" \
        --zone="${ZONE}" \
        --quiet &
      PIDS+=($!)
    fi
  done < <(echo "${ORPHANED_JSON}" | jq -c '.[]')

  # Wait for disk deletions
  FAILURES=0
  for pid in "${PIDS[@]:-}"; do
    if ! wait "${pid}" 2>/dev/null; then
      warn "Disk deletion job PID ${pid} failed"
      ((FAILURES++))
    fi
  done

  if [[ "${FAILURES}" -gt 0 ]]; then
    warn "${FAILURES} disk deletion(s) failed"
  else
    log "All ${ORPHAN_COUNT} orphaned disk(s) deleted successfully."
  fi
else
  log "No orphaned disks to clean up."
fi

log "=== Cleanup complete ==="
