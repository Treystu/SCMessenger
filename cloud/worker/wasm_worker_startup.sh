#!/usr/bin/env bash
###############################################################################
# wasm_worker_startup.sh — WASM-Specific Cloud Worker
#
# Builds and tests the SCMessenger WASM target:
#
#   1. Install toolchain: wasm-pack, Node.js 20, Playwright
#   2. wasm-pack build --target web --release
#   3. wasm-pack test --headless (Chrome + Firefox)
#   4. Optional WASI testing with wasmtime and wasmer
#   5. Report results to orchestrator
#
# Expects the same metadata attributes as startup.sh.
###############################################################################
set -euo pipefail

readonly LOG_FILE="/tmp/wasm_worker.log"
exec > >(tee -a "$LOG_FILE") 2>&1

# ─── Helpers ────────────────────────────────────────────────────────────────

log() {
  echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [wasm] $*"
}

die() {
  log "FATAL: $*"
  heartbeat "failed" "$*"
  exit 1
}

metadata() {
  curl -sf -H "Metadata-Flavor: Google" \
    "http://metadata.google.internal/computeMetadata/v1/instance/attributes/${1}" 2>/dev/null || true
}

ORCHESTRATOR_IP="$(metadata 'ORCHESTRATOR_IP')"
SPRINT_ID="$(metadata 'SPRINT_ID')"
GIT_BRANCH="$(metadata 'GIT_BRANCH')"
INSTANCE_NAME="$(curl -sf -H 'Metadata-Flavor: Google' \
  http://metadata.google.internal/computeMetadata/v1/instance/name)"
INSTANCE_ZONE="$(curl -sf -H 'Metadata-Flavor: Google' \
  http://metadata.google.internal/computeMetadata/v1/instance/zone | awk -F/ '{print $NF}')"

heartbeat() {
  local status="$1" message="${2:-}"
  [[ -z "$ORCHESTRATOR_IP" ]] && return 0
  curl -sf -X POST \
    -H "Content-Type: application/json" \
    -d "{
      \"sprint_id\":\"${SPRINT_ID}\",
      \"instance_name\":\"${INSTANCE_NAME}\",
      \"zone\":\"${INSTANCE_ZONE}\",
      \"status\":\"wasm_${status}\",
      \"message\":\"${message}\",
      \"timestamp\":\"$(date -u '+%Y-%m-%dT%H:%M:%SZ')\"
    }" \
    "http://${ORCHESTRATOR_IP}:8080/api/heartbeat" \
    --connect-timeout 5 --max-time 10 \
  || log "WARNING: heartbeat failed"
}

###############################################################################
# Step 1 — Install Toolchain
###############################################################################
log "Step 1 — Installing WASM toolchain"
heartbeat "toolchain_install" "Installing wasm-pack, Node.js, Playwright"

# System packages
sudo apt-get update -qq
sudo apt-get install -y -qq curl git build-essential pkg-config libssl-dev

# Rust
export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  source "$HOME/.cargo/env"
fi

# wasm32 target
rustup target add wasm32-unknown-unknown

# wasm-pack
if ! command -v wasm-pack &>/dev/null; then
  log "  Installing wasm-pack…"
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Node.js 20 (via NodeSource)
if ! command -v node &>/dev/null || [[ "$(node --version | cut -d. -f1 | tr -d 'v')" -lt 20 ]]; then
  log "  Installing Node.js 20…"
  curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
  sudo apt-get install -y -qq nodejs
fi

# Playwright browsers (chromium + firefox)
log "  Installing Playwright browsers…"
npx --yes playwright install --with-deps chromium firefox 2>&1 | tail -3

log "  wasm-pack : $(wasm-pack --version 2>/dev/null || echo 'not found')"
log "  node      : $(node --version 2>/dev/null || echo 'not found')"
log "  npm       : $(npm --version 2>/dev/null || echo 'not found')"

heartbeat "toolchain_done" "WASM toolchain installed"

###############################################################################
# Step 2 — Clone & Build WASM
###############################################################################
log "Step 2 — Building WASM target"
heartbeat "build" "Building wasm-pack --target web"

WORK_DIR="/tmp/scmessenger"
if [[ ! -d "$WORK_DIR" ]]; then
  git clone --depth 50 "https://github.com/nicholasgonzalezsc/SCMessenger_Clean.git" "$WORK_DIR"
fi
cd "$WORK_DIR"

if [[ -n "${GIT_BRANCH:-}" ]]; then
  git checkout "$GIT_BRANCH" 2>/dev/null || git checkout -b "$GIT_BRANCH" origin/main
fi

WASM_BUILD_OUTPUT="/tmp/wasm_build.txt"
WASM_BUILD_STATUS="success"

if [[ -d "wasm" ]]; then
  cd wasm

  log "  wasm-pack build --target web --release"
  if wasm-pack build --target web --release 2>&1 | tee "$WASM_BUILD_OUTPUT"; then
    log "  WASM build succeeded"
  else
    WASM_BUILD_STATUS="failed"
    log "  WASM build FAILED"
  fi
else
  log "  WARNING: wasm/ directory not found — attempting workspace build"
  if cargo build --target wasm32-unknown-unknown --release 2>&1 | tee "$WASM_BUILD_OUTPUT"; then
    log "  WASM workspace build succeeded"
  else
    WASM_BUILD_STATUS="failed"
    log "  WASM workspace build FAILED"
  fi
fi

heartbeat "build_done" "WASM build: ${WASM_BUILD_STATUS}"

###############################################################################
# Step 3 — Browser Tests (Chrome + Firefox)
###############################################################################
log "Step 3 — Running headless browser tests"
heartbeat "browser_tests" "Running wasm-pack tests in Chrome + Firefox"

CHROME_RESULTS="/tmp/wasm_test_chrome.txt"
FIREFOX_RESULTS="/tmp/wasm_test_firefox.txt"
CHROME_STATUS="skipped"
FIREFOX_STATUS="skipped"

cd "$WORK_DIR"
if [[ -d "wasm" ]]; then
  cd wasm

  # Chrome
  log "  Testing with Chrome…"
  if wasm-pack test --headless --chrome 2>&1 | tee "$CHROME_RESULTS"; then
    CHROME_STATUS="success"
    log "  Chrome tests PASSED"
  else
    CHROME_STATUS="failed"
    log "  Chrome tests FAILED"
  fi

  # Firefox
  log "  Testing with Firefox…"
  if wasm-pack test --headless --firefox 2>&1 | tee "$FIREFOX_RESULTS"; then
    FIREFOX_STATUS="success"
    log "  Firefox tests PASSED"
  else
    FIREFOX_STATUS="failed"
    log "  Firefox tests FAILED"
  fi
else
  log "  wasm/ directory not found — skipping browser tests"
fi

heartbeat "browser_tests_done" "Chrome:${CHROME_STATUS} Firefox:${FIREFOX_STATUS}"

###############################################################################
# Step 4 — WASI Runtime Tests (Optional)
###############################################################################
log "Step 4 — WASI runtime tests"
heartbeat "wasi_tests" "Testing WASI targets"

WASMTIME_STATUS="skipped"
WASMER_STATUS="skipped"

cd "$WORK_DIR"

# Check if there is a WASI target in the workspace
HAS_WASI_TARGET=false
if rustup target list --installed | grep -q "wasm32-wasip1\|wasm32-wasi"; then
  HAS_WASI_TARGET=true
elif rustup target add wasm32-wasip1 2>/dev/null; then
  HAS_WASI_TARGET=true
fi

if [[ "$HAS_WASI_TARGET" == "true" ]]; then
  # Install wasmtime
  if ! command -v wasmtime &>/dev/null; then
    log "  Installing wasmtime…"
    curl https://wasmtime.dev/install.sh -sSf | bash 2>/dev/null || true
    export PATH="$HOME/.wasmtime/bin:$PATH"
  fi

  # Install wasmer
  if ! command -v wasmer &>/dev/null; then
    log "  Installing wasmer…"
    curl https://get.wasmer.io -sSfL | sh 2>/dev/null || true
    export PATH="$HOME/.wasmer/bin:$PATH"
  fi

  # Build WASI target
  WASI_OUTPUT="/tmp/wasi_build.txt"
  if cargo build --target wasm32-wasip1 2>&1 | tee "$WASI_OUTPUT"; then
    log "  WASI build succeeded"

    # Find a .wasm file to test
    WASM_FILE=$(find target/wasm32-wasip1 -name '*.wasm' -type f 2>/dev/null | head -1 || true)

    if [[ -n "$WASM_FILE" ]]; then
      # wasmtime
      if command -v wasmtime &>/dev/null; then
        log "  Testing with wasmtime: ${WASM_FILE}"
        if wasmtime run "$WASM_FILE" 2>&1; then
          WASMTIME_STATUS="success"
        else
          WASMTIME_STATUS="failed"
        fi
      fi

      # wasmer
      if command -v wasmer &>/dev/null; then
        log "  Testing with wasmer: ${WASM_FILE}"
        if wasmer run "$WASM_FILE" 2>&1; then
          WASMER_STATUS="success"
        else
          WASMER_STATUS="failed"
        fi
      fi
    else
      log "  No .wasm output found — skipping runtime tests"
    fi
  else
    log "  WASI build failed — skipping runtime tests"
  fi
else
  log "  No WASI target available — skipping"
fi

heartbeat "wasi_done" "wasmtime:${WASMTIME_STATUS} wasmer:${WASMER_STATUS}"

###############################################################################
# Step 5 — Report Results
###############################################################################
log "Step 5 — Reporting results"

if [[ -n "${ORCHESTRATOR_IP:-}" ]]; then
  curl -sf -X POST \
    -H "Content-Type: application/json" \
    -d "{
      \"sprint_id\":     \"${SPRINT_ID}\",
      \"worker_type\":   \"wasm\",
      \"instance_name\": \"${INSTANCE_NAME}\",
      \"zone\":          \"${INSTANCE_ZONE}\",
      \"wasm_build\":    \"${WASM_BUILD_STATUS}\",
      \"chrome_tests\":  \"${CHROME_STATUS}\",
      \"firefox_tests\": \"${FIREFOX_STATUS}\",
      \"wasmtime\":      \"${WASMTIME_STATUS}\",
      \"wasmer\":        \"${WASMER_STATUS}\",
      \"timestamp\":     \"$(date -u '+%Y-%m-%dT%H:%M:%SZ')\"
    }" \
    "http://${ORCHESTRATOR_IP}:8080/api/results" \
    --connect-timeout 5 --max-time 10 \
  || log "WARNING: failed to report results"
fi

heartbeat "complete" "WASM worker done — build:${WASM_BUILD_STATUS} chrome:${CHROME_STATUS} firefox:${FIREFOX_STATUS}"
log "WASM worker complete."
