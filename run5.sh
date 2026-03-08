#!/bin/bash
# run5.sh — 5-Node SCMessenger Mesh Test Harness
#
# Nodes:
#   1. GCP      — headless relay (Docker on scmessenger-bootstrap)
#   2. OSX      — headless relay (local cargo binary)
#   3. Android  — full node (Pixel 6a via adb)
#   4. iOS Dev  — full node (physical device via devicectl)
#   5. iOS Sim  — full node (simulator via simctl)
#
# Usage: ./run5.sh [--time=5] [--update]
#   --time=N   Run for N minutes then auto-exit (default: 5)
#   --update   Rebuild headless nodes; push latest to mobile if available
#
# NOTE: set -e intentionally NOT used. Every command handles its own errors.

# ── Args ───────────────────────────────────────────────────────────────────────
DURATION_MIN=5
UPDATE_APPS=0
for arg in "$@"; do
  case "$arg" in
    -t=*|--time=*)  DURATION_MIN="${arg#*=}" ;;
    -u|--update)    UPDATE_APPS=1 ;;
  esac
done

# ── Log Directory (timestamped — preserves all historical runs) ────────────────
TIMESTAMP=$(date '+%Y%m%d_%H%M%S')
LOGDIR="logs/5mesh/$TIMESTAMP"
mkdir -p "$LOGDIR"
ln -sfn "$TIMESTAMP" "logs/5mesh/latest"

# ── Constants ──────────────────────────────────────────────────────────────────
BUNDLE_ID="SovereignCommunications.SCMessenger"
SYNC_MARKER="=== TEST_START_MARKER: $(date -u +'%Y-%m-%dT%H:%M:%SZ') ==="
GCP_ZONE="us-central1-a"
GCP_HOST="scmessenger-bootstrap"
GCP_IMAGE="us-central1-docker.pkg.dev/scmessenger-bootstrapnode/scmessenger-repo/scmessenger-cli:latest"
OSX_RUST_LOG="info,libp2p_autonat=debug,libp2p_dcutr=debug,libp2p_relay=debug,scmessenger_core::transport::swarm=debug,scmessenger_core::store::relay_custody=debug,scmessenger_core::mesh::delivery=debug"

# Prefer pre-built binary (instant start) over cargo run (30-60s compile)
if [ -f "target/debug/scmessenger-cli" ]; then
  OSX_RELAY_CMD="./target/debug/scmessenger-cli"
else
  OSX_RELAY_CMD="cargo run -p scmessenger-cli --"
fi

# ── PID tracking (always ours to kill — log collectors only) ──────────────────
GCP_LOG_PID=""
OSX_LOG_PID=""
ANDROID_LOGCAT_PID=""
IOS_DEV_LAUNCH_PID=""
IOS_DEV_STREAM_PID=""
IOS_SIM_STREAM_PID=""
TICKER_PID=""
OSX_PID=""

# Did WE start the app/service? (0=pre-existing, leave alone; 1=we started it)
STARTED_OSX=0
STARTED_ANDROID_APP=0
STARTED_IOS_DEV_APP=0
STARTED_IOS_SIM_APP=0

# ── Helper: log to stderr + logdir/harness.log ────────────────────────────────
HARNESS_LOG="$LOGDIR/harness.log"
hlog() {
  local msg="[$(date '+%H:%M:%S')] $*"
  echo "$msg" | tee -a "$HARNESS_LOG" >&2
}

# ── Helper: check if a PID is still alive ─────────────────────────────────────
pid_alive() { kill -0 "$1" 2>/dev/null; }

# ── Device Detection ───────────────────────────────────────────────────────────

# Android: pin to a specific serial to avoid "more than one device" errors
ADB_SERIAL=""
if adb devices 2>/dev/null | grep -q "device$"; then
  # Use a simpler, more robust selection that handles both TCP and USB
  ADB_SERIAL=$(adb devices -l | tail -n +2 | awk '$2=="device"{print $1; exit}')
fi
ANDROID_AVAILABLE=0
[ -n "$ADB_SERIAL" ] && ANDROID_AVAILABLE=1

# iOS Physical Device: use devicectl JSON output for reliable UDID detection
IOS_DEVICE_UDID="${IOS_DEVICE_UDID:-}"
if [ -z "$IOS_DEVICE_UDID" ]; then
  IOS_DEVICE_UDID=$(xcrun devicectl list devices \
    --hide-default-columns --columns Identifier --columns State --hide-headers 2>/dev/null | \
    awk '$2 ~ /(available|connected)/ {print $1; exit}')
fi

# iOS Simulator: find booted sim; boot one if missing
IOS_SIM_UDID="${IOS_SIM_UDID:-}"
if [ -z "$IOS_SIM_UDID" ]; then
  IOS_SIM_UDID=$(xcrun simctl list devices 2>/dev/null | awk -F '[()]' '/Booted/{print $2; exit}')
fi
if [ -z "$IOS_SIM_UDID" ]; then
  SIM=$(xcrun simctl list devices available 2>/dev/null | \
    awk -F '[()]' '/iPhone 16/{print $2; exit}')
  [ -z "$SIM" ] && SIM=$(xcrun simctl list devices available 2>/dev/null | \
    awk -F '[()]' '/iPhone/{print $2; exit}')
  if [ -n "$SIM" ]; then
    xcrun simctl boot "$SIM" >/dev/null 2>&1 && IOS_SIM_UDID="$SIM" || true
  fi
fi

# ── GCP SSH helper: tries IAP tunnel (more reliable in restricted networks) ───
gcp_ssh() {
  local cmd="$1"
  # Try IAP tunnel first (avoids firewall blocks on port 22)
  if gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" \
      --tunnel-through-iap \
      --ssh-flag="-o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=3" \
      --command="$cmd" 2>/dev/null; then
    return 0
  fi
  # Fall back to direct SSH
  hlog "  GCP IAP tunnel failed, trying direct SSH..."
  gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" \
    --ssh-flag="-o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=3" \
    --command="$cmd" 2>/dev/null
}

# ── Check if iOS device has SCMessenger running ──────────────────────────────
# devicectl device info processes outputs a text list — grep for SCMessenger.app path or bundle exe
ios_dev_running() {
  [ -z "$IOS_DEVICE_UDID" ] && return 1
  xcrun devicectl device info processes \
    --device "$IOS_DEVICE_UDID" 2>/dev/null | \
    grep -qiE "SCMessenger\.app/SCMessenger|SCMessenger$"
}

# ── Check if iOS sim app is running ───────────────────────────────────────────
ios_sim_running() {
  [ -z "$IOS_SIM_UDID" ] && return 1
  xcrun simctl listapps "$IOS_SIM_UDID" 2>/dev/null | grep -q "$BUNDLE_ID" || return 1
  # Check if actually running vs just installed
  xcrun simctl spawn "$IOS_SIM_UDID" pgrep -f "SCMessenger" >/dev/null 2>&1
}

# ══════════════════════════════════════════════════════════════════════════════
echo "╔══════════════════════════════════════════════════════════════╗"
printf  "║   SCMessenger Mesh Harness  %-32s ║\n" "$TIMESTAMP"
printf  "║   Duration: %-4sm  |  Update: %-27s ║\n" "$DURATION_MIN" "$UPDATE_APPS"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "═══ Phase 1: Node Status Audit ══════════════════════════════════"
echo ""

# 1. GCP -----------------------------------------------------------------------
printf "  [1/5] GCP headless relay   ... "
GCP_CID=$(gcp_ssh "sudo docker ps --filter status=running -q | head -n1" 2>/dev/null || true)
GCP_RUNNING=0
if [ -n "${GCP_CID:-}" ]; then
  GCP_VERSION=$(gcp_ssh "sudo docker inspect --format='{{.Config.Image}}' $GCP_CID" 2>/dev/null | grep -oE 'cli:[^\"]+' || echo "unknown")
  echo "✅ running  (container $GCP_CID  image: $GCP_VERSION)"
  GCP_RUNNING=1
else
  echo "❌ NOT RUNNING"
fi

# 2. OSX -----------------------------------------------------------------------
printf "  [2/5] OSX headless relay   ... "
OSX_RUNNING=0
if pgrep -f "scmessenger-cli.*relay" >/dev/null 2>&1; then
  OSX_PID=$(pgrep -f "scmessenger-cli.*relay" | head -1)
  echo "✅ running  (pid $OSX_PID  binary: $OSX_RELAY_CMD)"
  OSX_RUNNING=1
else
  echo "❌ NOT RUNNING  (will start: $OSX_RELAY_CMD)"
fi

# 3. Android -------------------------------------------------------------------
printf "  [3/5] Android full node    ... "
ANDROID_RUNNING=0
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  if adb -s "$ADB_SERIAL" shell pidof com.scmessenger.android >/dev/null 2>&1; then
    echo "✅ running  (serial: $ADB_SERIAL)"
    ANDROID_RUNNING=1
  else
    # Improved check: avoid output capture issues that cause integer expression errors
    ANDROID_STATE=$(adb -s "$ADB_SERIAL" shell pm list packages com.scmessenger.android 2>/dev/null | grep -c "com.scmessenger.android" || echo 0)
    # Ensure it's a valid integer
    [[ "$ANDROID_STATE" =~ ^[0-9]+$ ]] || ANDROID_STATE=0
    if [ "$ANDROID_STATE" -gt 0 ]; then
       echo "⚠️  installed, NOT running  (serial: $ADB_SERIAL)"
    else
       echo "❌ not installed"
    fi
  fi
else
  echo "⚠️  no device  (adb: none found)"
fi

# 4. iOS Device ----------------------------------------------------------------
printf "  [4/5] iOS Device full node ... "
IOS_DEV_RUNNING=0
if [ -n "$IOS_DEVICE_UDID" ]; then
  if ios_dev_running; then
    echo "✅ running  (udid: ${IOS_DEVICE_UDID:0:8}…)"
    IOS_DEV_RUNNING=1
  else
    echo "⚠️  device found, SCMessenger NOT running  (udid: ${IOS_DEVICE_UDID:0:8}…)"
  fi
else
  echo "⚠️  no device connected"
fi

# 5. iOS Simulator -------------------------------------------------------------
printf "  [5/5] iOS Simulator        ... "
IOS_SIM_RUNNING=0
if [ -n "$IOS_SIM_UDID" ]; then
  if ios_sim_running; then
    echo "✅ running  (udid: ${IOS_SIM_UDID:0:8}…)"
    IOS_SIM_RUNNING=1
  else
    echo "⚠️  booted, SCMessenger NOT running  (udid: ${IOS_SIM_UDID:0:8}…)"
  fi
else
  echo "⚠️  no simulator booted"
fi

echo ""
echo "═══ Phase 2: Ensure All Nodes Running ═══════════════════════════"
echo ""

# 1. GCP -----------------------------------------------------------------------
echo "  [1/5] GCP:"
if [ "$GCP_RUNNING" = "0" ] || [ "$UPDATE_APPS" = "1" ]; then
  if [ "$UPDATE_APPS" = "1" ]; then
    echo "        Pulling latest image..."
    gcp_ssh "sudo docker pull $GCP_IMAGE 2>&1 | tail -3"  || hlog "        ⚠️  docker pull failed"
    gcp_ssh "sudo docker stop \$(sudo docker ps -q) 2>/dev/null; sleep 1; \
             sudo docker run -d --restart=unless-stopped \
               -p 9001:9001 $GCP_IMAGE \
               relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000" \
      || hlog "        ⚠️  GCP restart failed"
    sleep 3
    GCP_CID=$(gcp_ssh "sudo docker ps --filter status=running -q | head -n1" 2>/dev/null || true)
  fi
  if [ -n "${GCP_CID:-}" ]; then
    echo "        ✅ Container $GCP_CID running"
  else
    echo "        ⚠️  GCP still not running — logs will show SSH error"
  fi
else
  echo "        ✅ Already running — passive log attach only"
fi

# 2. OSX -----------------------------------------------------------------------
echo "  [2/5] OSX:"
if [ "$OSX_RUNNING" = "0" ] || [ "$UPDATE_APPS" = "1" ]; then
  if [ "$UPDATE_APPS" = "1" ]; then
    echo "        Rebuilding binary..."
    cargo build -p scmessenger-cli 2>&1 | tail -3 || hlog "        ⚠️  cargo build failed"
    pkill -f "scmessenger-cli.*relay" 2>/dev/null || true; sleep 0.5
  fi
  echo "        Starting relay (nohup, binary: $OSX_RELAY_CMD)..."
  # Use nohup so relay survives if run5.sh's terminal dies
  RUST_LOG="$OSX_RUST_LOG" \
    nohup $OSX_RELAY_CMD relay \
      --listen /ip4/0.0.0.0/tcp/9010 \
      --http-port 9011 \
    >> "$LOGDIR/osx.log" 2>&1 &
  OSX_PID=$!
  STARTED_OSX=1
  # Brief sanity check — if it dies in <2s it's a binary issue
  sleep 2
  if pid_alive "$OSX_PID"; then
    echo "        ✅ Started (pid $OSX_PID)"
  else
    echo "        ❌ Process died immediately — check $LOGDIR/osx.log"
    hlog "OSX relay failed to start"
  fi
else
  echo "        ✅ Already running (pid $OSX_PID)"
fi

# 3. Android -------------------------------------------------------------------
echo "  [3/5] Android:"
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  if [ "$ANDROID_RUNNING" = "0" ]; then
    echo "        Gentle launch (am start, no force-stop)..."
    adb -s "$ADB_SERIAL" shell am start \
      -n com.scmessenger.android/.ui.MainActivity >/dev/null 2>&1 || true
    STARTED_ANDROID_APP=1
    sleep 1
    if adb -s "$ADB_SERIAL" shell pidof com.scmessenger.android >/dev/null 2>&1; then
      echo "        ✅ Launched"
    else
      echo "        ⚠️  Launch command sent but app PID not found yet"
    fi
  else
    echo "        ✅ Already running — passive logcat only"
  fi
else
  echo "        ⚠️  Skipped (no device)"
fi

# 4. iOS Device ----------------------------------------------------------------
echo "  [4/5] iOS Device:"
if [ -n "$IOS_DEVICE_UDID" ]; then
  if [ "$IOS_DEV_RUNNING" = "0" ]; then
    echo "        Launching existing install (no reinstall)..."
    xcrun devicectl device process launch \
      --device "$IOS_DEVICE_UDID" \
      --terminate-existing \
      --no-activate \
      "$BUNDLE_ID" >/dev/null 2>&1 || true
    STARTED_IOS_DEV_APP=1
    sleep 1
    if ios_dev_running; then
      echo "        ✅ Launched"
    else
      echo "        ⚠️  Launch sent — app may take a moment to appear"
    fi
  else
    echo "        ✅ Already running — passive log stream only"
  fi
else
  echo "        ⚠️  Skipped (no device)"
fi

# 5. iOS Sim -------------------------------------------------------------------
echo "  [5/5] iOS Sim:"
if [ -n "$IOS_SIM_UDID" ]; then
  if [ "$IOS_SIM_RUNNING" = "0" ]; then
    echo "        Launching existing install..."
    xcrun simctl launch "$IOS_SIM_UDID" "$BUNDLE_ID" >/dev/null 2>&1 || true
    STARTED_IOS_SIM_APP=1
    sleep 0.5
    echo "        ✅ Launched"
  else
    echo "        ✅ Already running — passive log stream only"
  fi
else
  echo "        ⚠️  Skipped"
fi

echo ""
echo "═══ Phase 3: Attach Passive Log Collectors ══════════════════════"
echo ""

# ── GCP: stream docker logs with SSH keepalive ────────────────────────────────
{
  printf "\n%s\n" "$SYNC_MARKER"
  # Try direct SSH; on failure, try IAP; on both failures, emit clear error
  if ! gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" \
      --ssh-flag="-o ConnectTimeout=10 -o ServerAliveInterval=15 -o ServerAliveCountMax=3" \
      --command="CID=\$(sudo docker ps --filter status=running -q | head -n1); \
                 [ -n \"\$CID\" ] && sudo docker logs --tail 200 -f \"\$CID\" 2>&1 \
                 || echo 'ERROR: No running GCP container'" 2>&1; then
    if ! gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" \
        --tunnel-through-iap \
        --ssh-flag="-o ConnectTimeout=15 -o ServerAliveInterval=15 -o ServerAliveCountMax=3" \
        --command="CID=\$(sudo docker ps --filter status=running -q | head -n1); \
                   [ -n \"\$CID\" ] && sudo docker logs --tail 200 -f \"\$CID\" 2>&1 \
                   || echo 'ERROR: No running GCP container'" 2>&1; then
      echo "ERROR: GCP SSH unreachable via direct and IAP tunnel"
    fi
  fi
} >> "$LOGDIR/gcp.log" 2>&1 &
GCP_LOG_PID=$!
echo "  [1/5] GCP log stream      PID=$GCP_LOG_PID  → $(basename $LOGDIR)/gcp.log"

# ── OSX: output already going to logdir if we started it; otherwise tail ──────
printf "\n%s\n" "$SYNC_MARKER" >> "$LOGDIR/osx.log"
if [ "$STARTED_OSX" = "0" ] && [ -n "$OSX_PID" ]; then
  # Pre-existing relay: try to pick up live output via a wrapper that tees
  # stdout of the existing process (best effort on macOS — procfs not available)
  # We can at least tail any log file the relay was previously writing to
  PREV_LOG=$(find logs/5mesh -name "osx.log" \
               -not -path "*/$TIMESTAMP/*" \
               -not -path "*/latest/*" 2>/dev/null | \
             sort -r | head -1 || true)
  if [ -n "$PREV_LOG" ]; then
    tail -F "$PREV_LOG" >> "$LOGDIR/osx.log" 2>/dev/null &
    OSX_LOG_PID=$!
    echo "  [2/5] OSX relay tail      PID=$OSX_LOG_PID  → $(basename $LOGDIR)/osx.log  (from prev run)"
  else
    echo "  [2/5] OSX relay           pre-existing, no prior log — new output not capturable post-hoc"
    OSX_LOG_PID=""
  fi
else
  OSX_LOG_PID="${OSX_PID:-}"
  echo "  [2/5] OSX relay output    PID=$OSX_PID    → $(basename $LOGDIR)/osx.log"
fi

# ── Android: pinned to serial, buffer starts NOW (-T 1) ──────────────────────
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  printf "\n%s\n" "$SYNC_MARKER" > "$LOGDIR/android.log"
  adb -s "$ADB_SERIAL" logcat -v threadtime -T 1 --pid="$(adb -s "$ADB_SERIAL" shell pidof com.scmessenger.android | tr -d '\r\n')" \
    >> "$LOGDIR/android.log" 2>&1 &
  ANDROID_LOGCAT_PID=$!
  echo "  [3/5] Android logcat      PID=$ANDROID_LOGCAT_PID → $(basename $LOGDIR)/android.log  (serial: $ADB_SERIAL)"
else
  echo "  [3/5] Android             skipped"
fi

# ── iOS Device: console stream (captures stdout from SCMessenger) ─────────────
# NOTE: devicectl has NO passive log stream subcommand.
# `process launch --console` both launches AND streams — this is the only way
# to capture stdio from a physical device without Xcode.
# We use --no-activate to avoid interrupting foreground state.
if [ -n "$IOS_DEVICE_UDID" ]; then
  printf "\n%s\n" "$SYNC_MARKER" > "$LOGDIR/ios-device.log"

  # Console stdio stream (Rust core output)
  xcrun devicectl device process launch \
    --device "$IOS_DEVICE_UDID" \
    --console \
    --no-activate \
    "$BUNDLE_ID" \
    >> "$LOGDIR/ios-device.log" 2>&1 &
  IOS_DEV_LAUNCH_PID=$!

  # System log stream: BLE + MPC subsystems (these appear in the host log)
  # Use `log stream` with predicate targeted at the device process name
  # System log stream: BLE + MPC subsystems
  # We try to exclude the Simulator specifically by sender path
  log stream \
    --style compact \
    --level info \
    --predicate 'process == "SCMessenger" OR subsystem == "com.apple.bluetooth" OR subsystem == "com.apple.MultipeerConnectivity"' \
    >> "$LOGDIR/ios-device.log" 2>&1 &
  IOS_DEV_STREAM_PID=$!

  echo "  [4/5] iOS Device          Launch=$IOS_DEV_LAUNCH_PID  Stream=$IOS_DEV_STREAM_PID → $(basename $LOGDIR)/ios-device.log"
else
  echo "  [4/5] iOS Device          skipped (no device)"
fi

# ── iOS Simulator: log stream (reliable, subsystem-filtered) ─────────────────
if [ -n "$IOS_SIM_UDID" ]; then
  printf "\n%s\n" "$SYNC_MARKER" > "$LOGDIR/ios-sim.log"
  xcrun simctl spawn "$IOS_SIM_UDID" log stream \
    --level info \
    --style compact \
    --predicate 'process == "SCMessenger"' \
    >> "$LOGDIR/ios-sim.log" 2>&1 &
  IOS_SIM_STREAM_PID=$!
  echo "  [5/5] iOS Sim             PID=$IOS_SIM_STREAM_PID → $(basename $LOGDIR)/ios-sim.log"
else
  echo "  [5/5] iOS Sim             skipped"
fi

# ── Sanity check all collectors are alive after 5s ───────────────────────────
sleep 5
echo ""
echo "  Collector health check (5s after start):"
_chk() {
  local name=$1 pid=$2
  if [ -z "$pid" ]; then
    printf "    %-20s skipped\n" "$name"
  elif pid_alive "$pid"; then
    local sz
    sz=$(wc -c < "$LOGDIR/${3:-/dev/null}" 2>/dev/null | tr -d ' ') || sz=0
    printf "    %-20s ✅  alive  (pid %-6s %s bytes)\n" "$name" "$pid" "$sz"
  else
    printf "    %-20s ❌  died!\n" "$name"
    hlog "WARN: $name log collector (pid $pid) died early"
  fi
}
_chk "GCP log stream"    "$GCP_LOG_PID"         "gcp.log"
_chk "OSX relay"         "$OSX_LOG_PID"         "osx.log"
_chk "Android logcat"    "$ANDROID_LOGCAT_PID"  "android.log"
_chk "iOS Dev launch"    "$IOS_DEV_LAUNCH_PID"  "ios-device.log"
_chk "iOS Dev stream"    "$IOS_DEV_STREAM_PID"  "ios-device.log"
_chk "iOS Sim stream"    "$IOS_SIM_STREAM_PID"  "ios-sim.log"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
printf "║  All nodes up. Auto-stopping in %-28s ║\n" "${DURATION_MIN}m."
printf "║  Logs → %-52s ║\n" "$LOGDIR/"
echo "║  Ctrl+C to stop early.                                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── Live status ticker ─────────────────────────────────────────────────────────
status_ticker() {
  local iter=0
  sleep 10
  while true; do
    iter=$((iter+1))
    echo ""
    echo "── $(date '+%H:%M:%S') Status (tick $iter) ──────────────────────────────"

    # GCP
    GCP_LINES=$(wc -l < "$LOGDIR/gcp.log" 2>/dev/null | tr -d ' \n')
    GCP_CIRCUITS=$(grep -c "circuit\|Circuit" "$LOGDIR/gcp.log" 2>/dev/null || echo 0)
    printf "  GCP:     %-5s lines  %-5s circuit events" "$GCP_LINES" "$GCP_CIRCUITS"
    pid_alive "${GCP_LOG_PID:-0}" 2>/dev/null && echo " ✅" || echo " ❌ stream dead"

    # OSX
    OSX_RELAYS=$(grep -c "Relay circuit reservation" "$LOGDIR/osx.log" 2>/dev/null || echo 0)
    OSX_PEERS=$(grep -oE "12D3KooW[A-Za-z0-9]+" "$LOGDIR/osx.log" 2>/dev/null | sort -u | wc -l | tr -d ' \n')
    printf "  OSX:     %-5s peers  %-5s relay reservations" "$OSX_PEERS" "$OSX_RELAYS"
    pid_alive "${OSX_LOG_PID:-0}" 2>/dev/null && echo " ✅" || echo " (external)"

    # Android
    if [ -f "$LOGDIR/android.log" ]; then
      ANDROID_EVENTS=$(grep -c "BLE.*scan\|Peer.*discov\|isFull" "$LOGDIR/android.log" 2>/dev/null || echo 0)
      ANDROID_LINES=$(wc -l < "$LOGDIR/android.log" 2>/dev/null | tr -d ' \n')
      ANDROID_NAT=$(grep "NAT status" "$LOGDIR/android.log" 2>/dev/null | tail -1 | grep -oE 'Public|Private|Unknown' || echo "?")
      printf "  Android: %-5s lines  %-5s events  NAT=%s" "$ANDROID_LINES" "$ANDROID_EVENTS" "$ANDROID_NAT"
      pid_alive "${ANDROID_LOGCAT_PID:-0}" 2>/dev/null && echo " ✅" || echo " ❌ logcat dead"
    else
      echo "  Android: no log"
    fi

    # iOS Dev
    if [ -f "$LOGDIR/ios-device.log" ]; then
      IOS_DEV_LINES=$(wc -l < "$LOGDIR/ios-device.log" 2>/dev/null | tr -d ' \n')
      IOS_DEV_PEERS=$(grep -c "Peer.*identif\|BLE.*identity" "$LOGDIR/ios-device.log" 2>/dev/null || echo 0)
      printf "  iOS Dev: %-5s lines  %-5s peer events" "$IOS_DEV_LINES" "$IOS_DEV_PEERS"
      pid_alive "${IOS_DEV_STREAM_PID:-0}" 2>/dev/null && echo " ✅" || echo " ❌ stream dead"
    else
      echo "  iOS Dev: no log"
    fi

    # iOS Sim
    if [ -f "$LOGDIR/ios-sim.log" ]; then
      IOS_SIM_LINES=$(wc -l < "$LOGDIR/ios-sim.log" 2>/dev/null | tr -d ' \n')
      IOS_SIM_PEERS=$(grep -c "Peer.*identif\|BLE.*identity" "$LOGDIR/ios-sim.log" 2>/dev/null || echo 0)
      printf "  iOS Sim: %-5s lines  %-5s peer events" "$IOS_SIM_LINES" "$IOS_SIM_PEERS"
      pid_alive "${IOS_SIM_STREAM_PID:-0}" 2>/dev/null && echo " ✅" || echo " ❌ stream dead"
    else
      echo "  iOS Sim: no log"
    fi

    # Notable recent events
    RECENT=$(grep -hE "✅ Relay|🔭 NAT|🕳️ DCUtR|Peer.*identif|isFull=true" \
      "$LOGDIR"/*.log 2>/dev/null | tail -4)
    if [ -n "$RECENT" ]; then
      echo "  Recent:"
      echo "$RECENT" | sed 's/^/    /'
    fi
    echo "────────────────────────────────────────────────"
    sleep 15
  done
}
status_ticker &
TICKER_PID=$!

# ── Shutdown: release only what we own ────────────────────────────────────────
shutdown() {
  echo ""
  echo "Shutting down..."

  kill "$GCP_LOG_PID"     2>/dev/null || true
  [ -n "${OSX_LOG_PID:-}" ]          && kill "$OSX_LOG_PID"        2>/dev/null || true
  [ -n "${ANDROID_LOGCAT_PID:-}" ]   && kill "$ANDROID_LOGCAT_PID" 2>/dev/null || true
  [ -n "${IOS_DEV_LAUNCH_PID:-}" ]   && kill "$IOS_DEV_LAUNCH_PID" 2>/dev/null || true
  [ -n "${IOS_DEV_STREAM_PID:-}" ]   && kill "$IOS_DEV_STREAM_PID" 2>/dev/null || true
  [ -n "${IOS_SIM_STREAM_PID:-}" ]   && kill "$IOS_SIM_STREAM_PID" 2>/dev/null || true
  [ -n "${TICKER_PID:-}" ]           && kill "$TICKER_PID"         2>/dev/null || true

  if [ "$STARTED_OSX" = "1" ] && [ -n "${OSX_PID:-}" ]; then
    echo "  Stopping OSX relay (we started it)..."
    kill "$OSX_PID" 2>/dev/null || true
  else
    echo "  OSX relay: pre-existing — left running ✅"
  fi

  if [ "$STARTED_ANDROID_APP" = "1" ] && [ "$ANDROID_AVAILABLE" = "1" ]; then
    echo "  Stopping Android app (we started it)..."
    adb -s "$ADB_SERIAL" shell am force-stop com.scmessenger.android >/dev/null 2>&1 || true
  else
    echo "  Android: pre-existing — left running ✅"
  fi

  if [ "$STARTED_IOS_DEV_APP" = "1" ] && [ -n "${IOS_DEVICE_UDID:-}" ]; then
    echo "  Stopping iOS Device app (we started it)..."
    xcrun devicectl device process terminate \
      --device "$IOS_DEVICE_UDID" \
      --bundle-id "$BUNDLE_ID" 2>/dev/null || true
  else
    [ -n "${IOS_DEVICE_UDID:-}" ] && echo "  iOS Device: pre-existing — left running ✅"
  fi

  if [ "$STARTED_IOS_SIM_APP" = "1" ] && [ -n "${IOS_SIM_UDID:-}" ]; then
    echo "  Stopping iOS Sim app (we started it)..."
    xcrun simctl terminate "$IOS_SIM_UDID" "$BUNDLE_ID" 2>/dev/null || true
  else
    [ -n "${IOS_SIM_UDID:-}" ] && echo "  iOS Sim: pre-existing — left running ✅"
  fi

  echo ""

  # ── Post-run analysis ──────────────────────────────────────────────────────
  echo "╔════════════════════════════════════════════════════════╗"
  echo "║   Post-Run Mesh Analysis                               ║"
  echo "╚════════════════════════════════════════════════════════╝"
  echo ""

  local LOGDIR_SNAP="$LOGDIR"
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$LOGDIR_SNAP" <<'PY'
import os, re, sys

LOGDIR = sys.argv[1]
logs = {
    'gcp':     os.path.join(LOGDIR, 'gcp.log'),
    'osx':     os.path.join(LOGDIR, 'osx.log'),
    'android': os.path.join(LOGDIR, 'android.log'),
    'ios_dev': os.path.join(LOGDIR, 'ios-device.log'),
    'ios_sim': os.path.join(LOGDIR, 'ios-sim.log'),
}
NODE_TYPES = {'gcp':'Headless','osx':'Headless','android':'Full','ios_dev':'Full','ios_sim':'Full'}
PAT = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")
OWN_ID_PATTERNS = [
    re.compile(r'===\s*OWN_IDENTITY:\s*(12D3KooW[a-zA-Z0-9]{44,})\s*==='),
    re.compile(r'local_peer_id\s*=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'SwarmBridge with peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Initialized core for peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    # relay agent string pattern: relay/<peerid> — only valid for headless nodes
    re.compile(r'agent: scmessenger/[^/]+/headless/relay/(12D3KooW[a-zA-Z0-9]{44,})'),
    # Android logcat: identity info from MeshRepository/IronCore
    re.compile(r'Mesh service started.*?libp2pPeerId=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'"libp2p_peer_id"\s*:\s*"(12D3KooW[a-zA-Z0-9]{44,})"'),
    # Android: own identity emission
    re.compile(r'Emitted IdentityDiscovered.*?peerId=(12D3KooW[a-zA-Z0-9]{44,})'),
    # Android logcat with tag prefix: "D/Rust  ( 1234): Starting Swarm with PeerID: ..."
    re.compile(r'Rust\s*:\s*Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'SCMessengerCore\s*:\s*.*?peer.?id[:\s]+(12D3KooW[a-zA-Z0-9]{44,})', re.I),
]
CONNECT_PAT = re.compile(r'(connected|PeerConnected|peer.*connect)', re.I)
ERROR_PAT   = re.compile(r'(Failed to negotiate|connection error|ERR)', re.I)
RELAY_PAT   = re.compile(r'(Relay circuit reservation|Relaying message)', re.I)
SENT_PAT    = re.compile(r'(✓ Direct delivery ACK|outcome=success|sent message)', re.I)
RECV_PAT    = re.compile(r'(✓ Received message|msg_rx_processed|receive_message)', re.I)
NAT_PAT     = re.compile(r'AutoNAT.*?(Public|Private|Unknown)', re.I)

def read(path):
    try:
        with open(path, 'r', errors='ignore') as f: return f.read()
    except: return ""

def strip_ansi(s): return re.sub(r'\x1b\[[^m]*m', '', s)

contents = {n: strip_ansi(read(p)) for n, p in logs.items()}

# For each node, try to find its OWN peer ID (not a peer it's talking to)
# Strategy: the OWN ID should NOT appear as a relay agent string in ANOTHER node
file_to_id = {}
all_candidates = {}
for name, content in contents.items():
    if not content: continue
    for pat in OWN_ID_PATTERNS:
        m = pat.search(content)
        if m and len(m.group(1)) >= 52:
            cand = m.group(1)
            # Validate: a real "own" ID generally appears in lines about local config
            # (not in "Peer identified:" lines which describe remote peers)
            all_candidates[name] = cand
            file_to_id[name] = cand
            break

# Cross-check: if a candidate ID appears as a relay AGENT in another log, it's a
# remote peer being described, not the local node's own ID. De-conflict.
relay_agent_ids = set()
for name, content in contents.items():
    for m in re.finditer(r'agent: scmessenger/[^/]+/[^/]+/relay/(12D3KooW[a-zA-Z0-9]{44,})', content):
        relay_agent_ids.add(m.group(1))
    for m in re.finditer(r'agent: scmessenger/[^/]+/[^/]+/identity/(12D3KooW[a-zA-Z0-9]{44,})', content):
        relay_agent_ids.add(m.group(1))

# Un-assign any full node that incorrectly grabbed a headless node's ID
for name in list(file_to_id.keys()):
    if NODE_TYPES.get(name) == 'Full' and file_to_id[name] in relay_agent_ids:
        del file_to_id[name]

# Fallback for nodes (especially Android/iOS) whose startup logs might be truncated.
# The most frequently appearing peer ID that isn't a known relay node or already taken is likely their own.
taken_ids = set(file_to_id.values()) | relay_agent_ids
for name, content in contents.items():
    if name not in file_to_id:
        all_ids = PAT.findall(content)
        freq = {}
        for pid in all_ids:
            if pid not in taken_ids:
                freq[pid] = freq.get(pid, 0) + 1
        if freq:
            best_id = sorted(freq.items(), key=lambda x: x[1], reverse=True)[0][0]
            file_to_id[name] = best_id
            taken_ids.add(best_id)

matrix = {name: set(PAT.findall(c)) for name, c in contents.items()}

# Header
print(f"  {'Node':<10} {'Own ID':<26} {'Sent':>5} {'Recv':>5} {'Relay':>6} {'Conns':>6} {'NAT':<9} {'Errors':>7}")
print("  " + "─" * 88)
for name in logs:
    c   = contents[name]
    pid = file_to_id.get(name, 'unknown')
    pid_d = (pid[:22] + '..') if len(pid) > 22 else pid
    lines  = c.count('\n')
    sent   = len(SENT_PAT.findall(c))
    recv   = len(RECV_PAT.findall(c))
    relays = len(RELAY_PAT.findall(c))
    nat_m  = NAT_PAT.findall(c)
    nat    = nat_m[-1].lower() if nat_m else '?'
    conns  = len(CONNECT_PAT.findall(c))
    errs   = len(ERROR_PAT.findall(c))
    has_content = lines > 2
    icon = '✅' if (pid != 'unknown' or has_content) else '❌'
    print(f"  {icon} {name:<8} {pid_d:<26} {sent:>5} {recv:>5} {relays:>6} {conns:>6} {nat:<9} {errs:>7}")

print()
print("  Visibility Matrix (did node X see node Y's peer ID?):")
print(f"  {'Node':<10} {'Peers Seen':<12} Missing")
print("  " + "─" * 62)
all_ok = True
for name in logs:
    seen = matrix[name]
    missing = []
    for other in logs:
        if other == name: continue
        oid = file_to_id.get(other)
        if not oid or oid not in seen:
            missing.append(other)
            all_ok = False
    seen_count = len(logs) - 1 - len(missing)
    icon = '✅' if not missing else ('⚠️ ' if len(missing) <= 2 else '❌')
    print(f"  {icon} {name:<8} {seen_count}/{len(logs)-1:<10} {', '.join(missing) or 'none'}")

print()
if all_ok:
    print("  🎉 FULL MESH — All nodes visible to all peers!")
else:
    gaps = sum(1 for n in logs for o in logs if o != n and
               (not file_to_id.get(o) or file_to_id.get(o) not in matrix[n]))
    total = len(logs) * (len(logs)-1)
    pct = int(100 * (total - gaps) / total)
    print(f"  ⚠️  Partial mesh — {gaps}/{total} gap(s)  ({pct}% connected)")
    print(f"     Tip: run longer (--time=10) for peer IDs to propagate fully")

print()
# Log file health summary
print("  Log file health:")
for name, path in logs.items():
    if os.path.exists(path):
        sz = os.path.getsize(path)
        lines = contents[name].count('\n')
        icon = '✅' if lines > 5 else ('⚠️' if lines > 0 else '❌')
        print(f"    {icon} {name:<10} {lines:>6} lines  {sz:>8} bytes  {path}")
    else:
        print(f"    ❌ {name:<10} (no log file)")
PY
  else
    echo "  python3 not found — log file sizes:"
    for log in "$LOGDIR"/*.log; do
      [ -f "$log" ] && printf "    %-20s %s lines\n" "$(basename "$log")" "$(wc -l < "$log" | tr -d ' ')"
    done
  fi

  echo ""
  echo "Logs: $LOGDIR/"
  echo "Done."
  exit 0
}
trap "shutdown" INT TERM

# ── Auto-exit after duration ───────────────────────────────────────────────────
sleep $((DURATION_MIN * 60))
echo ""
echo "⏰ Time limit (${DURATION_MIN}m) reached."
kill -TERM $$
