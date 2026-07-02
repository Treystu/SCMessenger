#!/usr/bin/env bash
###############################################################################
# SCMessenger — Create Golden Worker Disk Snapshot
#
# One-time script that:
#   1. Creates a temporary e2-standard-4 VM
#   2. Installs the full build toolchain (Rust, Android SDK, Docker, etc.)
#   3. Snapshots the disk as "scm-worker-golden-<timestamp>"
#   4. Deletes the temporary VM
#
# Usage:
#   ./create_golden_snapshot.sh [project_id] [zone]
#
# This takes ~30-45 minutes to complete.
###############################################################################
set -euo pipefail

readonly SCRIPT_NAME="$(basename "$0")"
readonly LOG_TAG="scm-golden-snapshot"
readonly TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
readonly TEMP_VM_NAME="scm-golden-builder-${TIMESTAMP}"
readonly SNAPSHOT_NAME="scm-worker-golden-${TIMESTAMP}"

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
PROJECT="${1:-$(gcloud config get-value project 2>/dev/null)}"
ZONE="${2:-us-central1-a}"
MACHINE_TYPE="e2-standard-4"
DISK_SIZE="80"  # GB — generous for all toolchains

[[ -z "${PROJECT}" ]] && { echo "ERROR: No project specified and none in gcloud config"; exit 1; }

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"; }
die()  { log "ERROR: $*"; exit 1; }

cleanup_on_exit() {
  local exit_code=$?
  if [[ ${exit_code} -ne 0 ]]; then
    log "Script failed (exit ${exit_code}). The temporary VM '${TEMP_VM_NAME}' may still exist."
    log "To clean up manually: gcloud compute instances delete ${TEMP_VM_NAME} --zone=${ZONE} --project=${PROJECT}"
  fi
}
trap cleanup_on_exit EXIT

# ---------------------------------------------------------------------------
# Startup script that runs INSIDE the temporary VM
# ---------------------------------------------------------------------------
read -r -d '' STARTUP_SCRIPT << 'INNER_EOF' || true
#!/usr/bin/env bash
###############################################################################
# Golden Image Builder — runs inside the temporary VM
###############################################################################
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive

log() { echo "[GOLDEN $(date '+%H:%M:%S')] $*"; }

log "========== Starting golden image build =========="

# -------------------------------------------------------------------
# 1. System packages
# -------------------------------------------------------------------
log "Installing base system packages…"
apt-get update -qq
apt-get install -y -qq \
  build-essential pkg-config libssl-dev libsqlite3-dev \
  git curl wget unzip jq tmux htop \
  apt-transport-https ca-certificates gnupg lsb-release \
  python3 python3-pip python3-venv \
  cmake ninja-build clang lld \
  libfontconfig1-dev libfreetype6-dev \
  qemu-kvm libvirt-daemon-system

# -------------------------------------------------------------------
# 2. Rust 1.95.0 + all 13 cross-compilation targets
# -------------------------------------------------------------------
log "Installing Rust 1.95.0…"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
  sh -s -- -y --default-toolchain 1.95.0

source /root/.cargo/env

log "Adding Rust cross-compilation targets…"
TARGETS=(
  aarch64-linux-android
  armv7-linux-androideabi
  i686-linux-android
  x86_64-linux-android
  aarch64-apple-ios
  aarch64-apple-ios-sim
  x86_64-apple-ios
  aarch64-apple-darwin
  x86_64-apple-darwin
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
  wasm32-unknown-unknown
  x86_64-pc-windows-msvc
)

for target in "${TARGETS[@]}"; do
  log "  Adding target: ${target}"
  rustup target add "${target}" || log "  WARNING: target ${target} may not be available as a tier-1/2 target"
done

# Install cross for cross-compilation (handles x86_64-pc-windows-msvc etc.)
log "Installing cross…"
cargo install cross --git https://github.com/cross-rs/cross || log "WARNING: cross install failed, continuing…"

# -------------------------------------------------------------------
# 3. Cargo tools
# -------------------------------------------------------------------
log "Installing cargo tools…"
cargo install cargo-ndk
cargo install wasm-pack
cargo install cargo-tarpaulin
cargo install cargo-audit
cargo install cargo-deny
cargo install sccache

# Configure sccache as the default Rust compiler wrapper
mkdir -p /root/.cargo
cat >> /root/.cargo/config.toml <<'CARGOCONF'

[build]
rustc-wrapper = "sccache"
CARGOCONF

log "Cargo tools installed."

# -------------------------------------------------------------------
# 4. Docker
# -------------------------------------------------------------------
log "Installing Docker…"
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc

echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] \
  https://download.docker.com/linux/debian $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
  > /etc/apt/sources.list.d/docker.list

apt-get update -qq
apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

systemctl enable docker
systemctl start docker

log "Docker installed: $(docker --version)"

# -------------------------------------------------------------------
# 5. Node.js 20 + Playwright browsers
# -------------------------------------------------------------------
log "Installing Node.js 20…"
curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
apt-get install -y -qq nodejs

log "Node.js installed: $(node --version)"
log "npm installed: $(npm --version)"

log "Installing Playwright browsers…"
npx --yes playwright install --with-deps chromium firefox webkit

log "Playwright browsers installed."

# -------------------------------------------------------------------
# 6. JDK 17 (Eclipse Temurin)
# -------------------------------------------------------------------
log "Installing JDK 17 (Temurin)…"
curl -fsSL https://packages.adoptium.net/artifactory/api/gpg/key/public \
  | gpg --dearmor -o /etc/apt/keyrings/adoptium.gpg

echo "deb [signed-by=/etc/apt/keyrings/adoptium.gpg] \
  https://packages.adoptium.net/artifactory/deb $(. /etc/os-release && echo "$VERSION_CODENAME") main" \
  > /etc/apt/sources.list.d/adoptium.list

apt-get update -qq
apt-get install -y -qq temurin-17-jdk

export JAVA_HOME="/usr/lib/jvm/temurin-17-jdk-$(dpkg --print-architecture)"
echo "JAVA_HOME=${JAVA_HOME}" >> /etc/environment
log "JDK 17 installed: $(java -version 2>&1 | head -1)"

# -------------------------------------------------------------------
# 7. Android SDK 35 + NDK r27 + system images
# -------------------------------------------------------------------
log "Installing Android SDK…"
export ANDROID_HOME="/opt/android-sdk"
export ANDROID_SDK_ROOT="${ANDROID_HOME}"
mkdir -p "${ANDROID_HOME}/cmdline-tools"

# Download command-line tools
CMDLINE_TOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
cd /tmp
wget -q "${CMDLINE_TOOLS_URL}" -O cmdline-tools.zip
unzip -q cmdline-tools.zip -d "${ANDROID_HOME}/cmdline-tools"
mv "${ANDROID_HOME}/cmdline-tools/cmdline-tools" "${ANDROID_HOME}/cmdline-tools/latest"
rm cmdline-tools.zip

export PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${PATH}"

# Accept licenses
yes | sdkmanager --licenses >/dev/null 2>&1 || true

# Install SDK components
log "Installing Android SDK components (this takes a while)…"
sdkmanager --install \
  "platform-tools" \
  "platforms;android-35" \
  "build-tools;35.0.0" \
  "ndk;27.0.12077973" \
  "system-images;android-35;google_apis;x86_64" \
  "emulator" \
  2>&1 | tail -5

# Persist ANDROID_HOME
cat >> /etc/environment <<ENVEOF
ANDROID_HOME=${ANDROID_HOME}
ANDROID_SDK_ROOT=${ANDROID_HOME}
ANDROID_NDK_HOME=${ANDROID_HOME}/ndk/27.0.12077973
ENVEOF

# Add to PATH for all users
cat > /etc/profile.d/android-sdk.sh <<'PATHEOF'
export ANDROID_HOME="/opt/android-sdk"
export ANDROID_SDK_ROOT="${ANDROID_HOME}"
export ANDROID_NDK_HOME="${ANDROID_HOME}/ndk/27.0.12077973"
export PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/emulator:${PATH}"
PATHEOF
chmod +x /etc/profile.d/android-sdk.sh

log "Android SDK installed."

# -------------------------------------------------------------------
# 8. Gradle wrapper support
# -------------------------------------------------------------------
log "Setting up Gradle wrapper support…"
mkdir -p /opt/gradle-wrapper
cd /opt/gradle-wrapper
cat > build.gradle <<'GRADLE'
// Placeholder to initialise Gradle wrapper
task wrapper(type: Wrapper) {
    gradleVersion = '8.7'
}
GRADLE

# Pre-download Gradle via wrapper
GRADLE_DIST_URL="https://services.gradle.org/distributions/gradle-8.7-bin.zip"
mkdir -p /opt/gradle
wget -q "${GRADLE_DIST_URL}" -O /tmp/gradle.zip
unzip -q /tmp/gradle.zip -d /opt/gradle
rm /tmp/gradle.zip
ln -sf /opt/gradle/gradle-8.7/bin/gradle /usr/local/bin/gradle
log "Gradle installed: $(gradle --version 2>&1 | head -3 | tail -1)"

# -------------------------------------------------------------------
# 9. Clone SCMessenger + pre-populate cargo cache
# -------------------------------------------------------------------
log "Cloning SCMessenger_Clean and pre-populating cargo cache…"
mkdir -p /opt/scm-workspace
cd /opt/scm-workspace

# Clone — this will fail if repo is private and no credentials are set,
# which is fine for the golden image. The worker startup script will
# re-clone with proper auth.
if git clone --depth=1 https://github.com/user/SCMessenger_Clean.git 2>/dev/null; then
  cd SCMessenger_Clean
  # Fetch all dependencies into the cargo registry cache
  cargo fetch --locked 2>/dev/null || cargo fetch 2>/dev/null || log "WARNING: cargo fetch failed (expected if Cargo.lock missing)"
  cd /opt/scm-workspace
else
  log "WARNING: Could not clone SCMessenger_Clean (may be private). Skipping cargo cache pre-population."
fi

# -------------------------------------------------------------------
# 10. Clean up temporary files to reduce snapshot size
# -------------------------------------------------------------------
log "Cleaning up temporary files…"
apt-get clean
rm -rf /var/lib/apt/lists/*
rm -rf /tmp/*
rm -rf /var/tmp/*

log "========== Golden image build complete =========="

# Signal completion via metadata
curl -sf -X PUT \
  -H "Metadata-Flavor: Google" \
  -d "COMPLETE" \
  "http://metadata.google.internal/computeMetadata/v1/instance/attributes/build-status" \
  2>/dev/null || true

# Write a completion marker
echo "BUILD_COMPLETE $(date -u '+%Y-%m-%dT%H:%M:%SZ')" > /root/.golden-build-complete
INNER_EOF

# ---------------------------------------------------------------------------
# Step 1: Create the temporary VM
# ---------------------------------------------------------------------------
log "Creating temporary VM: ${TEMP_VM_NAME} (${MACHINE_TYPE}, ${DISK_SIZE}GB, ${ZONE})"
gcloud compute instances create "${TEMP_VM_NAME}" \
  --project="${PROJECT}" \
  --zone="${ZONE}" \
  --machine-type="${MACHINE_TYPE}" \
  --image-family=debian-12 \
  --image-project=debian-cloud \
  --boot-disk-size="${DISK_SIZE}GB" \
  --boot-disk-type=pd-ssd \
  --metadata="build-status=PENDING" \
  --metadata-from-file="startup-script=<(echo "${STARTUP_SCRIPT}")" \
  --scopes=storage-ro \
  --quiet

log "Temporary VM created. Waiting for build to complete…"
log "This typically takes 30-45 minutes."

# ---------------------------------------------------------------------------
# Step 2: Wait for the build to finish
# ---------------------------------------------------------------------------
MAX_WAIT_MINUTES=60
POLL_INTERVAL=60
ELAPSED=0

while [[ ${ELAPSED} -lt $((MAX_WAIT_MINUTES * 60)) ]]; do
  # Check if the startup script has finished by looking at serial port output
  if gcloud compute instances get-serial-port-output "${TEMP_VM_NAME}" \
       --project="${PROJECT}" --zone="${ZONE}" --quiet 2>/dev/null \
     | grep -q "Golden image build complete"; then
    log "Build completed successfully!"
    break
  fi

  ELAPSED=$((ELAPSED + POLL_INTERVAL))
  REMAINING=$(( (MAX_WAIT_MINUTES * 60 - ELAPSED) / 60 ))
  log "Still building… (${REMAINING} min remaining before timeout)"
  sleep "${POLL_INTERVAL}"
done

if [[ ${ELAPSED} -ge $((MAX_WAIT_MINUTES * 60)) ]]; then
  die "Build timed out after ${MAX_WAIT_MINUTES} minutes. VM ${TEMP_VM_NAME} left running for debugging."
fi

# ---------------------------------------------------------------------------
# Step 3: Stop the VM (required before snapshotting)
# ---------------------------------------------------------------------------
log "Stopping VM ${TEMP_VM_NAME}…"
gcloud compute instances stop "${TEMP_VM_NAME}" \
  --project="${PROJECT}" \
  --zone="${ZONE}" \
  --quiet

log "VM stopped. Creating snapshot…"

# ---------------------------------------------------------------------------
# Step 4: Create the snapshot
# ---------------------------------------------------------------------------
log "Creating snapshot: ${SNAPSHOT_NAME}"
gcloud compute disks snapshot "${TEMP_VM_NAME}" \
  --project="${PROJECT}" \
  --zone="${ZONE}" \
  --snapshot-names="${SNAPSHOT_NAME}" \
  --description="SCMessenger golden worker image — Rust 1.95.0, Android SDK 35, NDK r27, Docker, Node.js 20, JDK 17" \
  --quiet

log "Snapshot ${SNAPSHOT_NAME} created successfully."

# ---------------------------------------------------------------------------
# Step 5: Delete the temporary VM
# ---------------------------------------------------------------------------
log "Deleting temporary VM ${TEMP_VM_NAME}…"
gcloud compute instances delete "${TEMP_VM_NAME}" \
  --project="${PROJECT}" \
  --zone="${ZONE}" \
  --delete-disks=all \
  --quiet

log "Temporary VM deleted."

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
log "============================================================"
log "Golden snapshot created successfully!"
log ""
log "  Snapshot name : ${SNAPSHOT_NAME}"
log "  Project       : ${PROJECT}"
log "  Contents      :"
log "    - Rust 1.95.0 (13 targets + cross)"
log "    - cargo-ndk, wasm-pack, cargo-tarpaulin, cargo-audit, cargo-deny, sccache"
log "    - Docker CE"
log "    - Node.js 20 + Playwright (Chromium, Firefox, WebKit)"
log "    - Android SDK 35, NDK r27, system images"
log "    - JDK 17 (Temurin), Gradle 8.7"
log ""
log "  To use in Terraform, set the worker template boot disk to:"
log "    source_snapshot = \"${SNAPSHOT_NAME}\""
log "============================================================"
