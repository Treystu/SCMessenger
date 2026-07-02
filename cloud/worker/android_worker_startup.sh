#!/usr/bin/env bash
###############################################################################
# android_worker_startup.sh — Android-Specific Cloud Worker
#
# Designed for GCP VMs with nested virtualisation enabled (--enable-nested-
# virtualization). Performs:
#
#   1. Verify KVM availability
#   2. Install Android SDK, emulator, platform-tools, system images
#   3. Create AVD and launch headless emulator
#   4. Build Rust core for Android targets (arm64-v8a, x86_64)
#   5. Copy .so → jniLibs, generate UniFFI Kotlin bindings
#   6. Gradle build + connected Android tests
#   7. Report results to orchestrator
#
# Expects the same metadata attributes as startup.sh.
###############################################################################
set -euo pipefail

readonly LOG_FILE="/tmp/android_worker.log"
exec > >(tee -a "$LOG_FILE") 2>&1

# ─── Helpers ────────────────────────────────────────────────────────────────

log() {
  echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] [android] $*"
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
      \"status\":\"android_${status}\",
      \"message\":\"${message}\",
      \"timestamp\":\"$(date -u '+%Y-%m-%dT%H:%M:%SZ')\"
    }" \
    "http://${ORCHESTRATOR_IP}:8080/api/heartbeat" \
    --connect-timeout 5 --max-time 10 \
  || log "WARNING: heartbeat failed"
}

###############################################################################
# Step 1 — Verify KVM
###############################################################################
log "Step 1 — Verifying KVM support"
heartbeat "kvm_check" "Verifying nested virtualisation"

if [[ ! -e /dev/kvm ]]; then
  die "/dev/kvm not found — this VM was not launched with nested virtualisation. \
Add --enable-nested-virtualization when creating the instance."
fi

# Ensure current user can access KVM
if [[ ! -r /dev/kvm ]] || [[ ! -w /dev/kvm ]]; then
  sudo chmod 666 /dev/kvm
fi

log "  KVM is available: $(ls -la /dev/kvm)"
heartbeat "kvm_ok" "KVM verified"

###############################################################################
# Step 2 — Install Android SDK
###############################################################################
log "Step 2 — Installing Android SDK"
heartbeat "sdk_install" "Installing Android SDK"

export ANDROID_HOME="/opt/android-sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/emulator:${PATH}"

# Install prerequisites
sudo apt-get update -qq
sudo apt-get install -y -qq openjdk-17-jdk-headless unzip wget curl git

# Download command-line tools if not present
if [[ ! -d "${ANDROID_HOME}/cmdline-tools/latest" ]]; then
  log "  Downloading Android command-line tools…"
  sudo mkdir -p "${ANDROID_HOME}/cmdline-tools"

  CMDLINE_TOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
  wget -q "$CMDLINE_TOOLS_URL" -O /tmp/cmdline-tools.zip
  sudo unzip -q /tmp/cmdline-tools.zip -d "${ANDROID_HOME}/cmdline-tools"
  sudo mv "${ANDROID_HOME}/cmdline-tools/cmdline-tools" "${ANDROID_HOME}/cmdline-tools/latest"
  rm -f /tmp/cmdline-tools.zip
fi

# Accept licenses and install components
yes | sdkmanager --licenses >/dev/null 2>&1 || true

log "  Installing SDK packages…"
sdkmanager --install \
  "platform-tools" \
  "emulator" \
  "platforms;android-34" \
  "build-tools;34.0.0" \
  "system-images;android-34;google_apis;x86_64" \
  2>&1 | tail -5

log "  Android SDK installed at ${ANDROID_HOME}"
heartbeat "sdk_done" "Android SDK installed"

###############################################################################
# Step 3 — Create AVD & Launch Emulator
###############################################################################
log "Step 3 — Creating AVD and launching emulator"
heartbeat "emulator_start" "Creating AVD and starting emulator"

AVD_NAME="test_device"

# Create the AVD (non-interactive)
echo "no" | avdmanager create avd \
  --name "$AVD_NAME" \
  --package "system-images;android-34;google_apis;x86_64" \
  --device "pixel_7" \
  --force \
  2>&1 || die "Failed to create AVD"

log "  AVD '${AVD_NAME}' created"

# Launch emulator headlessly with KVM acceleration
emulator -avd "$AVD_NAME" \
  -no-window \
  -no-audio \
  -no-boot-anim \
  -gpu swiftshader_indirect \
  -memory 4096 \
  -cores 4 \
  -accel on \
  -no-snapshot \
  &
EMULATOR_PID=$!
log "  Emulator PID: ${EMULATOR_PID}"

# Wait for emulator to boot
log "  Waiting for emulator to boot…"
adb wait-for-device

# Poll sys.boot_completed
BOOT_TIMEOUT=300
BOOT_ELAPSED=0
while true; do
  BOOT_COMPLETE=$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r\n' || true)
  if [[ "$BOOT_COMPLETE" == "1" ]]; then
    log "  Emulator booted successfully (${BOOT_ELAPSED}s)"
    break
  fi
  if (( BOOT_ELAPSED >= BOOT_TIMEOUT )); then
    die "Emulator boot timed out after ${BOOT_TIMEOUT}s"
  fi
  sleep 5
  BOOT_ELAPSED=$((BOOT_ELAPSED + 5))
done

heartbeat "emulator_ready" "Emulator booted in ${BOOT_ELAPSED}s"

###############################################################################
# Step 4 — Clone & Build Rust Core for Android
###############################################################################
log "Step 4 — Building Rust core for Android"
heartbeat "rust_build" "Building Rust core for Android targets"

# Ensure Rust + cargo-ndk are installed
export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  source "$HOME/.cargo/env"
fi

# Add Android targets
rustup target add aarch64-linux-android x86_64-linux-android

# Install cargo-ndk
if ! command -v cargo-ndk &>/dev/null; then
  cargo install cargo-ndk
fi

# Clone if not already cloned
WORK_DIR="/tmp/scmessenger"
if [[ ! -d "$WORK_DIR" ]]; then
  git clone --depth 50 "https://github.com/nicholasgonzalezsc/SCMessenger_Clean.git" "$WORK_DIR"
fi
cd "$WORK_DIR"

if [[ -n "${GIT_BRANCH:-}" ]]; then
  git checkout "$GIT_BRANCH" 2>/dev/null || git checkout -b "$GIT_BRANCH" origin/main
fi

# Build for both targets
log "  Building for arm64-v8a and x86_64…"
ANDROID_BUILD_OUTPUT="/tmp/android_build.txt"

if cargo ndk \
  -t arm64-v8a \
  -t x86_64 \
  --platform 34 \
  -o android/app/src/main/jniLibs \
  build --release 2>&1 | tee "$ANDROID_BUILD_OUTPUT"; then
  RUST_BUILD="success"
  log "  Rust Android build succeeded"
else
  RUST_BUILD="failed"
  log "  Rust Android build FAILED"
fi

heartbeat "rust_build_done" "Rust Android build: ${RUST_BUILD}"

###############################################################################
# Step 5 — Copy .so Files & Generate UniFFI Bindings
###############################################################################
log "Step 5 — Copying .so files and generating UniFFI bindings"
heartbeat "uniffi" "Generating UniFFI bindings"

# Verify .so files were placed
for arch_dir in arm64-v8a x86_64; do
  so_dir="android/app/src/main/jniLibs/${arch_dir}"
  if [[ -d "$so_dir" ]]; then
    log "  ${arch_dir}: $(find "$so_dir" -name '*.so' | wc -l) .so files"
  else
    log "  WARNING: ${so_dir} not found"
  fi
done

# Generate UniFFI Kotlin bindings (if uniffi-bindgen is available)
if command -v uniffi-bindgen-kotlin &>/dev/null || cargo install uniffi_bindgen 2>/dev/null; then
  UNIFFI_UDL=$(find core/src -name '*.udl' 2>/dev/null | head -1 || true)
  if [[ -n "$UNIFFI_UDL" ]]; then
    log "  Generating bindings from ${UNIFFI_UDL}"
    uniffi-bindgen generate "$UNIFFI_UDL" \
      --language kotlin \
      --out-dir android/app/src/main/java/com/scmessenger/bindings \
      2>&1 || log "  WARNING: UniFFI binding generation failed"
  else
    log "  No .udl files found — skipping UniFFI generation"
  fi
else
  log "  uniffi-bindgen not available — skipping binding generation"
fi

heartbeat "uniffi_done" "UniFFI bindings generated"

###############################################################################
# Step 6 — Gradle Build & Connected Tests
###############################################################################
log "Step 6 — Gradle build and connected tests"
heartbeat "gradle" "Running Gradle build + tests"

cd "$WORK_DIR/android"

GRADLE_OUTPUT="/tmp/gradle_output.txt"

# Ensure gradlew is executable
chmod +x ./gradlew 2>/dev/null || true

# Build debug APKs
log "  assembleDebug…"
if ./gradlew assembleDebug 2>&1 | tee -a "$GRADLE_OUTPUT"; then
  log "  assembleDebug succeeded"
else
  log "  assembleDebug FAILED"
fi

# Build test APK
log "  assembleAndroidTest…"
if ./gradlew assembleAndroidTest 2>&1 | tee -a "$GRADLE_OUTPUT"; then
  log "  assembleAndroidTest succeeded"
else
  log "  assembleAndroidTest FAILED"
fi

# Run connected (instrumented) tests
log "  connectedAndroidTest…"
ANDROID_TEST_RESULTS="/tmp/android_test_results.txt"

if ./gradlew connectedAndroidTest 2>&1 | tee "$ANDROID_TEST_RESULTS"; then
  ANDROID_TEST_STATUS="success"
  log "  Connected tests PASSED"
else
  ANDROID_TEST_STATUS="failed"
  log "  Connected tests FAILED"
fi

heartbeat "gradle_done" "Android tests: ${ANDROID_TEST_STATUS}"

###############################################################################
# Step 7 — Report Results
###############################################################################
log "Step 7 — Reporting results to orchestrator"

if [[ -n "${ORCHESTRATOR_IP:-}" ]]; then
  curl -sf -X POST \
    -H "Content-Type: application/json" \
    -d "{
      \"sprint_id\":     \"${SPRINT_ID}\",
      \"worker_type\":   \"android\",
      \"instance_name\": \"${INSTANCE_NAME}\",
      \"zone\":          \"${INSTANCE_ZONE}\",
      \"rust_build\":    \"${RUST_BUILD}\",
      \"android_tests\": \"${ANDROID_TEST_STATUS}\",
      \"timestamp\":     \"$(date -u '+%Y-%m-%dT%H:%M:%SZ')\"
    }" \
    "http://${ORCHESTRATOR_IP}:8080/api/results" \
    --connect-timeout 5 --max-time 10 \
  || log "WARNING: failed to report results"
fi

# Kill emulator
kill "$EMULATOR_PID" 2>/dev/null || true
adb emu kill 2>/dev/null || true

heartbeat "complete" "Android worker finished — rust:${RUST_BUILD} tests:${ANDROID_TEST_STATUS}"
log "Android worker complete."
