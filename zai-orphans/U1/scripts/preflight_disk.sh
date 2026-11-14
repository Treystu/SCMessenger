#!/bin/bash
# SCMessenger Disk Preflight Check
# Validates free disk space before running expensive build gates.
# Measured constraint: full five-gate sweep (fmt, clippy, clippy --all-features,
# cargo test --no-run, wasm release) regrows target/ by ~40-47 GB.

set -e

# Repo root detection
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Threshold: 25 GB minimum free space (chosen to safely accommodate 45 GB rebuild)
THRESHOLD_KB=$((25 * 1024 * 1024))

# Get free space on the drive holding the repo
FREE_KB=$(df "$REPO_ROOT" | awk 'NR==2 {print $4}')
FREE_GB=$((FREE_KB / 1024 / 1024))

# Get current target/ size if it exists
TARGET_SIZE_GB=0
if [ -d "$REPO_ROOT/target" ]; then
  TARGET_KB=$(du -sk "$REPO_ROOT/target" 2>/dev/null | awk '{print $1}')
  TARGET_SIZE_GB=$((TARGET_KB / 1024 / 1024))
fi

# Report current state
echo "[INFO] Disk preflight check"
echo "[INFO] Free space: ${FREE_GB} GB on $(df "$REPO_ROOT" | awk 'NR==2 {print $6}')"
echo "[INFO] Current target/: ${TARGET_SIZE_GB} GB"

# Fail if space is too low
if [ "$FREE_KB" -lt "$THRESHOLD_KB" ]; then
  echo "[ERROR] ERROR: Insufficient free disk space"
  echo "[ERROR] Free: ${FREE_GB} GB, Minimum required: 25 GB"
  echo "[ERROR] A full gate sweep regrows target/ to ~45 GB; ensure at least 25 GB free"
  echo "[ERROR] Run 'cargo clean' and/or 'cd android && ./gradlew clean' to free space"
  exit 1
fi

echo "[OK] Disk space sufficient"
exit 0
