#!/bin/bash
# run5.sh — 5-Node SCMessenger Mesh Test Harness
#
# Philosophy:
#   • Full nodes (Android, iOS Device, iOS Sim): always pre-installed & running.
#     This script NEVER touches them — it only attaches log collectors passively.
#     If somehow not running, it brings them up gently (no force-stop, no reinstall).
#   • Headless nodes (GCP, OSX relay): checked and brought up to date if needed.
#     GCP: verified via docker ps; restarts container if stale.
#     OSX: started via cargo if not already running.
#
# Usage: ./run5_trip.sh [--time=5] [--update] [--restore-on-exit]
#   --time=N    Run for N minutes then auto-exit (default: 5)
#   --update    Rebuild & restart headless nodes; push latest to mobile (optional)
#   --restore-on-exit
#               Stop any nodes this script launched before exiting.
#
set -euo pipefail

# ── Args ──────────────────────────────────────────────────────────────────────
DURATION_MIN=5
UPDATE_APPS=0
RESTORE_ON_EXIT=0
while [ $# -gt 0 ]; do
  case "$1" in
    -t=*|--time=*) DURATION_MIN="${1#*=}" ;;
    -u|--update)   UPDATE_APPS=1 ;;
    --restore-on-exit) RESTORE_ON_EXIT=1 ;;
    *) ;;
  esac
  shift
done

# ── Log directory (timestamped, historical) ────────────────────────────────────
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
GCP_CONTAINER_NAME="scmessenger-bootstrap-relay"
OSX_RUST_LOG="info,libp2p_autonat=debug,libp2p_dcutr=debug,libp2p_relay=debug,scmessenger_core::transport::swarm=debug,scmessenger_core::store::relay_custody=debug,scmessenger_core::mesh::delivery=debug"

# Non-interactive wrapper so GCP SSH failures do not block the harness.
gcp_ssh() {
  CLOUDSDK_CORE_DISABLE_PROMPTS=1 gcloud --quiet compute ssh "$GCP_HOST" --zone="$GCP_ZONE" \
    --ssh-flag="-o BatchMode=yes" \
    --ssh-flag="-o ConnectTimeout=8" \
    --ssh-flag="-o StrictHostKeyChecking=accept-new" \
    --command "$1"
}

# ── Tracking: did WE start it? (0 = pre-existing, leave alone; 1 = we started it) ──
STARTED_OSX=0
STARTED_ANDROID_APP=0
STARTED_IOS_DEV_APP=0
STARTED_IOS_SIM_APP=0

# PIDs for log collection processes (always ours to clean up)
GCP_LOG_PID=""
OSX_LOG_PID=""
ANDROID_LOGCAT_PID=""
IOS_DEV_LAUNCH_PID=""
IOS_DEV_STREAM_PID=""
IOS_SIM_STREAM_PID=""
TICKER_PID=""

# ── Detect available devices ───────────────────────────────────────────────────
IOS_DEVICE_UDID="${IOS_DEVICE_UDID:-$(xcrun devicectl list devices \
  --hide-default-columns --columns Identifier --columns State --hide-headers 2>/dev/null | \
  awk '$2 ~ /(available|connected)/ {print $1; exit}')}"
IOS_SIM_UDID="${IOS_SIM_UDID:-$(xcrun simctl list devices 2>/dev/null | awk -F '[()]' '/Booted/ {print $2; exit}')}"

# Boot a simulator if none is running
if [ -z "${IOS_SIM_UDID:-}" ]; then
  SIM=$(xcrun simctl list devices available 2>/dev/null | awk -F '[()]' '/iPhone 16e|iPhone 16/ {print $2; exit}')
  [ -z "${SIM:-}" ] && SIM=$(xcrun simctl list devices available 2>/dev/null | awk -F '[()]' '/iPhone/ {print $2; exit}')
  if [ -n "${SIM:-}" ]; then
    xcrun simctl boot "$SIM" >/dev/null 2>&1 || true
    IOS_SIM_UDID="$SIM"
  fi
fi

if adb get-state >/dev/null 2>&1; then
  ANDROID_AVAILABLE=1
else
  ANDROID_AVAILABLE=0
fi

# ═══════════════════════════════════════════════════════════════════════════════
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║   SCMessenger 5-Node Mesh Harness — $TIMESTAMP   ║"
echo "║   Duration: ${DURATION_MIN}m  |  Update: $UPDATE_APPS                           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Phase 1: Checking node status..."
echo ""

# ── Phase 1: Status check all 5 nodes ──────────────────────────────────────────

# 1. GCP -----------------------------------------------------------------------
echo -n "  [1/5] GCP headless relay ... "
GCP_CID=$(gcp_ssh "CID=\$(sudo docker ps --filter name=$GCP_CONTAINER_NAME --format '{{.ID}}' | head -n1); \
                   if [ -z \"\$CID\" ]; then \
                     CID=\$(sudo docker ps --filter ancestor=$GCP_IMAGE --format '{{.ID}}' | head -n1); \
                   fi; \
                   echo \"\$CID\"" 2>/dev/null || echo "")
if [ -n "$GCP_CID" ]; then
  GCP_STATUS="running (container $GCP_CID)"
  GCP_RUNNING=1
else
  GCP_STATUS="NOT RUNNING"
  GCP_RUNNING=0
fi
echo "$GCP_STATUS"

# 2. OSX -----------------------------------------------------------------------
echo -n "  [2/5] OSX headless relay  ... "
if pgrep -f "scmessenger-cli.*relay" >/dev/null 2>&1; then
  OSX_PID=$(pgrep -f "scmessenger-cli.*relay" | head -1)
  OSX_STATUS="running (pid $OSX_PID)"
  OSX_RUNNING=1
else
  OSX_STATUS="NOT RUNNING"
  OSX_RUNNING=0
  OSX_PID=""
fi
echo "$OSX_STATUS"

# 3. Android -------------------------------------------------------------------
echo -n "  [3/5] Android full node   ... "
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  if adb shell pidof com.scmessenger.android >/dev/null 2>&1; then
    ANDROID_STATUS="running"
    ANDROID_RUNNING=1
  else
    ANDROID_STATUS="installed but NOT running"
    ANDROID_RUNNING=0
  fi
else
  ANDROID_STATUS="no adb device"
  ANDROID_RUNNING=0
fi
echo "$ANDROID_STATUS"

# 4. iOS Device ----------------------------------------------------------------
echo -n "  [4/5] iOS Device full node ... "
if [ -n "${IOS_DEVICE_UDID:-}" ]; then
  if xcrun devicectl device process list --device "$IOS_DEVICE_UDID" 2>/dev/null | grep -q "$BUNDLE_ID"; then
    IOS_DEV_STATUS="running on $IOS_DEVICE_UDID"
    IOS_DEV_RUNNING=1
  else
    IOS_DEV_STATUS="installed but NOT running"
    IOS_DEV_RUNNING=0
  fi
else
  IOS_DEV_STATUS="no device connected"
  IOS_DEV_RUNNING=0
fi
echo "$IOS_DEV_STATUS"

# 5. iOS Simulator -------------------------------------------------------------
echo -n "  [5/5] iOS Simulator        ... "
if [ -n "${IOS_SIM_UDID:-}" ]; then
  if xcrun simctl spawn "$IOS_SIM_UDID" launchctl list 2>/dev/null | grep -q "UIKitApplication:$BUNDLE_ID"; then
    IOS_SIM_STATUS="running on $IOS_SIM_UDID"
    IOS_SIM_RUNNING=1
  else
    IOS_SIM_STATUS="booted but NOT running"
    IOS_SIM_RUNNING=0
  fi
else
  IOS_SIM_STATUS="no simulator booted"
  IOS_SIM_RUNNING=0
fi
echo "$IOS_SIM_STATUS"

echo ""

# ═══════════════════════════════════════════════════════════════════════════════
echo "Phase 2: Ensuring all nodes are up..."
echo ""

# ── Phase 2: Start anything not running ────────────────────────────────────────

# 1. GCP — ensure container is running and up to date -------------------------
echo "  [1/5] GCP headless relay:"
if [ "$GCP_RUNNING" = "0" ] || [ "$UPDATE_APPS" = "1" ]; then
  if [ "$UPDATE_APPS" = "1" ]; then
    echo "        Pulling latest image and restarting..."
    gcp_ssh "sudo docker pull $GCP_IMAGE && \
             sudo docker rm -f $GCP_CONTAINER_NAME >/dev/null 2>&1 || true && \
             sudo docker run -d --restart=unless-stopped \
               --name $GCP_CONTAINER_NAME \
               -p 9001:9001 $GCP_IMAGE \
               relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000" \
      >/dev/null 2>&1 || echo "        ⚠️  GCP restart failed — check manually"
  else
    echo "        Starting relay container..."
    gcp_ssh "if ! sudo docker image inspect $GCP_IMAGE >/dev/null 2>&1; then sudo docker pull $GCP_IMAGE; fi && \
             sudo docker rm -f $GCP_CONTAINER_NAME >/dev/null 2>&1 || true && \
             sudo docker run -d --restart=unless-stopped \
               --name $GCP_CONTAINER_NAME \
               -p 9001:9001 $GCP_IMAGE \
               relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000" \
      >/dev/null 2>&1 || echo "        ⚠️  GCP start failed — check manually"
  fi
  sleep 3
  GCP_CID=$(gcp_ssh "CID=\$(sudo docker ps --filter name=$GCP_CONTAINER_NAME --format '{{.ID}}' | head -n1); \
                     if [ -z \"\$CID\" ]; then \
                       CID=\$(sudo docker ps --filter ancestor=$GCP_IMAGE --format '{{.ID}}' | head -n1); \
                     fi; \
                     echo \"\$CID\"" 2>/dev/null || echo "")
  if [ -z "${GCP_CID:-}" ]; then
    echo "        ⚠️  GCP container still not running — log stream will be empty"
  else
    echo "        ✅  Container $GCP_CID now running"
  fi
else
  echo "        ✅  Already running — attaching log stream only"
fi

# 2. OSX — start relay if not running -----------------------------------------
echo "  [2/5] OSX headless relay:"
if [ "$OSX_RUNNING" = "0" ] || [ "$UPDATE_APPS" = "1" ]; then
  if [ "$UPDATE_APPS" = "1" ]; then
    echo "        Rebuilding and restarting..."
    pkill -f "scmessenger-cli" 2>/dev/null || true
    sleep 0.5
  fi
  echo "        Starting cargo relay..."
  RUST_LOG="$OSX_RUST_LOG" \
    cargo run -p scmessenger-cli -- relay \
    --listen /ip4/0.0.0.0/tcp/9010 \
    --http-port 9011 \
    >> "$LOGDIR/osx.log" 2>&1 &
  OSX_PID=$!
  STARTED_OSX=1
  echo "        ✅  Started (pid $OSX_PID)"
else
  echo "        ✅  Already running (pid $OSX_PID) — attaching log stream only"
  # Tee existing process stdout isn't possible post-hoc; we capture via log file tail if present
  # For OSX relay, we rely on future output redirected to our new log if we started it,
  # otherwise we note the pre-existing log.
  echo "        ℹ️   Note: prior stdout not capturable post-hoc; new events appended via tail"
fi

# 3. Android — gently start if not running (NEVER force-stop, NEVER reinstall) -
echo "  [3/5] Android full node:"
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  if [ "$ANDROID_RUNNING" = "0" ]; then
    echo "        Bringing to foreground (gentle start)..."
    adb shell am start -n com.scmessenger.android/.ui.MainActivity >/dev/null 2>&1 || true
    STARTED_ANDROID_APP=1
    sleep 1
    echo "        ✅  Launched"
  else
    echo "        ✅  Already running — attaching logcat passively"
  fi
else
  echo "        ⚠️   No adb device — skipping"
fi

# 4. iOS Device — gently launch if not running --------------------------------
echo "  [4/5] iOS Device full node:"
if [ -n "${IOS_DEVICE_UDID:-}" ]; then
  if [ "$IOS_DEV_RUNNING" = "0" ]; then
    echo "        Launching existing install (no reinstall)..."
    xcrun devicectl device process launch \
      --device "$IOS_DEVICE_UDID" \
      --terminate-existing \
      "$BUNDLE_ID" >/dev/null 2>&1 || true
    STARTED_IOS_DEV_APP=1
    sleep 1
    echo "        ✅  Launched"
  else
    echo "        ✅  Already running — attaching log stream passively"
  fi
else
  echo "        ⚠️   No device connected — skipping"
fi

# 5. iOS Sim — launch if not running ------------------------------------------
echo "  [5/5] iOS Simulator:"
if [ -n "${IOS_SIM_UDID:-}" ]; then
  if [ "$IOS_SIM_RUNNING" = "0" ]; then
    echo "        Launching existing install..."
    xcrun simctl launch "$IOS_SIM_UDID" "$BUNDLE_ID" >/dev/null 2>&1 || true
    STARTED_IOS_SIM_APP=1
    sleep 0.5
    echo "        ✅  Launched"
  else
    echo "        ✅  Already running — attaching log stream passively"
  fi
else
  echo "        ⚠️   No simulator booted — skipping"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════════════
echo "Phase 3: Attaching log collectors (passive — no app impact)..."
echo ""

# ── Phase 3: Attach log collection to all running nodes ────────────────────────

# 1. GCP — stream docker logs --------------------------------------------------
echo "$SYNC_MARKER" > "$LOGDIR/gcp.log"
gcp_ssh "CID=\$(sudo docker ps --filter name=$GCP_CONTAINER_NAME --format '{{.ID}}' | head -n1); \
         if [ -z \"\$CID\" ]; then \
           CID=\$(sudo docker ps --filter ancestor=$GCP_IMAGE --format '{{.ID}}' | head -n1); \
         fi; \
         if [ -n \"\$CID\" ]; then sudo docker logs -f \"\$CID\" 2>&1; \
         else echo 'No GCP container running'; fi" \
  >> "$LOGDIR/gcp.log" 2>&1 &
GCP_LOG_PID=$!
echo "  [1/5] GCP log stream    PID=$GCP_LOG_PID  → $LOGDIR/gcp.log"

# 2. OSX — if we started it, output already going to logdir; else tail existing log ---
echo "$SYNC_MARKER" >> "$LOGDIR/osx.log"
if [ "$STARTED_OSX" = "1" ]; then
  # Output already redirected to $LOGDIR/osx.log in Phase 2
  OSX_LOG_PID=$OSX_PID
  echo "  [2/5] OSX relay output  PID=$OSX_PID     → $LOGDIR/osx.log"
else
  # Find the most recently written OSX log and tail it
  PREV_OSX_LOG=$(find logs/5mesh -type f -name "osx.log" \
                   ! -path "$LOGDIR/osx.log" 2>/dev/null | \
                 sort -r | head -1 || true)
  if [ -n "${PREV_OSX_LOG:-}" ]; then
    tail -F "$PREV_OSX_LOG" >> "$LOGDIR/osx.log" 2>/dev/null &
    OSX_LOG_PID=$!
    echo "  [2/5] OSX relay tail    PID=$OSX_LOG_PID  → $LOGDIR/osx.log (from $PREV_OSX_LOG)"
  else
    echo "  [2/5] OSX relay         no prior log found; new events will appear when relay talks"
    OSX_LOG_PID=""
  fi
fi

# 3. Android — attach logcat passively ----------------------------------------
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  echo "$SYNC_MARKER" > "$LOGDIR/android.log"
  adb logcat -v threadtime \
    MeshRepository:V SwarmBridge:V IronCore:V CoreDelegateImpl:V \
    MainViewModel:V DashboardViewModel:V BleScanner:V BleGattClient:V \
    BleGattServer:V BleAdvertiser:V MeshService:V ContactsViewModel:V \
    Rust:V SCMessengerCore:V rust_logger:V \
    "*:S" \
    >> "$LOGDIR/android.log" 2>&1 &
  ANDROID_LOGCAT_PID=$!
  echo "  [3/5] Android logcat    PID=$ANDROID_LOGCAT_PID → $LOGDIR/android.log"
else
  echo "  [3/5] Android           skipped (no device)"
fi

# 4. iOS Device — dual stream: console + system logs -------------------------
if [ -n "${IOS_DEVICE_UDID:-}" ]; then
  echo "$SYNC_MARKER" > "$LOGDIR/ios-device.log"
  IOS_DEV_LAUNCH_PID=""
  if [ "$STARTED_IOS_DEV_APP" = "1" ]; then
    xcrun devicectl device process launch \
      --device "$IOS_DEVICE_UDID" \
      --console \
      "$BUNDLE_ID" \
      >> "$LOGDIR/ios-device.log" 2>&1 &
    IOS_DEV_STREAM_PID=$!
    echo "  [4/5] iOS Device        Stream=$IOS_DEV_STREAM_PID → $LOGDIR/ios-device.log (console stream)"
  else
    IOS_DEV_STREAM_PID=""
    echo "iOS device passive log stream unavailable via devicectl; app left untouched." >> "$LOGDIR/ios-device.log"
    echo "  [4/5] iOS Device        passive stream unavailable (left app untouched)"
  fi
else
  echo "  [4/5] iOS Device        skipped (no device)"
fi

# 5. iOS Simulator — log stream -----------------------------------------------
if [ -n "${IOS_SIM_UDID:-}" ]; then
  echo "$SYNC_MARKER" > "$LOGDIR/ios-sim.log"
  xcrun simctl spawn "$IOS_SIM_UDID" log stream \
    --level info \
    --style compact \
    --predicate 'process == "SCMessenger"' \
    >> "$LOGDIR/ios-sim.log" 2>&1 &
  IOS_SIM_STREAM_PID=$!
  echo "  [5/5] iOS Sim           PID=$IOS_SIM_STREAM_PID → $LOGDIR/ios-sim.log"
else
  echo "  [5/5] iOS Sim           skipped (no simulator)"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  All collectors active. Auto-stopping in ${DURATION_MIN}m.                 ║"
echo "║  Logs → $LOGDIR/                     ║"
echo "║  Ctrl+C to stop early.                                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# ── Live status ticker ─────────────────────────────────────────────────────────
status_ticker() {
  sleep 10
  while true; do
    echo ""
    echo "── $(date '+%H:%M:%S') Status ─────────────────────────────────"

    OSX_RESERVATIONS=$(grep -c "Relay circuit reservation" "$LOGDIR/osx.log" 2>/dev/null || true)
    OSX_RESERVATIONS=${OSX_RESERVATIONS:-0}
    OSX_PEERS=$(grep -oE "12D3KooW[A-Za-z0-9]+" "$LOGDIR/osx.log" 2>/dev/null | sort -u | wc -l | tr -d ' ')
    echo "  OSX:     $OSX_PEERS peers, $OSX_RESERVATIONS relay reservations"

    GCP_CIRCUITS=$(grep -Ec "circuit|Circuit" "$LOGDIR/gcp.log" 2>/dev/null || true)
    GCP_CIRCUITS=${GCP_CIRCUITS:-0}
    echo "  GCP:     $GCP_CIRCUITS circuit events"

    if [ -f "$LOGDIR/android.log" ]; then
      ANDROID_EVENTS=$(grep -Ec "BLE.*scan|Peer.*discov|isFull" "$LOGDIR/android.log" 2>/dev/null || true)
      ANDROID_EVENTS=${ANDROID_EVENTS:-0}
      ANDROID_NAT=$(grep "NAT status" "$LOGDIR/android.log" 2>/dev/null | tail -1 | grep -oE 'public|private|unknown' || echo "?")
      echo "  Android: $ANDROID_EVENTS events, NAT=$ANDROID_NAT"
    fi

    if [ -f "$LOGDIR/ios-device.log" ]; then
      IOS_DEV_LINES=$(wc -l < "$LOGDIR/ios-device.log" 2>/dev/null || echo 0)
      IOS_DEV_PEERS=$(grep -Ec "Peer.*identif|BLE.*identity" "$LOGDIR/ios-device.log" 2>/dev/null || true)
      IOS_DEV_PEERS=${IOS_DEV_PEERS:-0}
      echo "  iOS Dev: $IOS_DEV_LINES lines, $IOS_DEV_PEERS peer events"
    fi

    if [ -f "$LOGDIR/ios-sim.log" ]; then
      IOS_SIM_PEERS=$(grep -Ec "Peer.*identif|BLE.*identity" "$LOGDIR/ios-sim.log" 2>/dev/null || true)
      IOS_SIM_PEERS=${IOS_SIM_PEERS:-0}
      echo "  iOS Sim: $IOS_SIM_PEERS peer events"
    fi

    echo "  Recent:"
    grep -hE "🔭 NAT|✅ Relay|🕳️ DCUtR|Peer.*identif|BLE identity|isFull=true" \
      "$LOGDIR"/*.log 2>/dev/null | tail -4 | sed 's/^/    /'
    echo "──────────────────────────────────────────────"
    sleep 15
  done
}

status_ticker &
TICKER_PID=$!

# ── Shutdown: only stop what we started ───────────────────────────────────────
shutdown() {
  echo ""
  echo "Shutting down..."

  # Always kill log collectors (they're always ours)
  kill "$GCP_LOG_PID" 2>/dev/null || true
  [ -n "${OSX_LOG_PID:-}" ]          && kill "$OSX_LOG_PID" 2>/dev/null || true
  [ -n "${ANDROID_LOGCAT_PID:-}" ]   && kill "$ANDROID_LOGCAT_PID" 2>/dev/null || true
  [ -n "${IOS_DEV_LAUNCH_PID:-}" ]   && kill "$IOS_DEV_LAUNCH_PID" 2>/dev/null || true
  [ -n "${IOS_DEV_STREAM_PID:-}" ]   && kill "$IOS_DEV_STREAM_PID" 2>/dev/null || true
  [ -n "${IOS_SIM_STREAM_PID:-}" ]   && kill "$IOS_SIM_STREAM_PID" 2>/dev/null || true
  [ -n "${TICKER_PID:-}" ]           && kill "$TICKER_PID" 2>/dev/null || true

  # Keep launched nodes running by default to avoid visibility regressions.
  if [ "$RESTORE_ON_EXIT" = "1" ]; then
    if [ "$STARTED_OSX" = "1" ]; then
      echo "  Stopping OSX relay (restore-on-exit)..."
      kill "$OSX_PID" 2>/dev/null || true
    else
      echo "  OSX relay: pre-existing — left running ✅"
    fi

    if [ "$STARTED_ANDROID_APP" = "1" ] && [ "$ANDROID_AVAILABLE" = "1" ]; then
      echo "  Stopping Android app (restore-on-exit)..."
      adb shell am force-stop com.scmessenger.android >/dev/null 2>&1 || true
    else
      echo "  Android: pre-existing — left running ✅"
    fi

    if [ "$STARTED_IOS_DEV_APP" = "1" ] && [ -n "${IOS_DEVICE_UDID:-}" ]; then
      echo "  Stopping iOS Device app (restore-on-exit)..."
      xcrun devicectl device process terminate --device "$IOS_DEVICE_UDID" \
        --bundle-id "$BUNDLE_ID" 2>/dev/null || true
    else
      echo "  iOS Device: pre-existing — left running ✅"
    fi

    if [ "$STARTED_IOS_SIM_APP" = "1" ] && [ -n "${IOS_SIM_UDID:-}" ]; then
      echo "  Stopping iOS Sim app (restore-on-exit)..."
      xcrun simctl terminate "$IOS_SIM_UDID" "$BUNDLE_ID" 2>/dev/null || true
    else
      echo "  iOS Sim: pre-existing — left running ✅"
    fi
  else
    echo "  restore-on-exit disabled: leaving all started nodes running ✅"
  fi

  echo ""

  # ── Post-run analysis ──────────────────────────────────────────────────────
  echo "╔════════════════════════════════════════════════════════╗"
  echo "║   Post-Run Mesh Analysis                               ║"
  echo "╚════════════════════════════════════════════════════════╝"
  echo ""

  LOGDIR_FOR_PY="$LOGDIR"
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$LOGDIR_FOR_PY" <<'PY'
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
    re.compile(r'local_peer_id\s*=\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'SwarmBridge with peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})'),
    re.compile(r'relay/(12D3KooW[a-zA-Z0-9]{44,})'),
]
CONNECT_PAT = re.compile(r'(connected|PeerConnected|peer.*connect)', re.I)
ERROR_PAT   = re.compile(r'(Failed to negotiate|connection error|ERR)', re.I)
RELAY_PAT   = re.compile(r'Relay circuit reservation')
NAT_PAT     = re.compile(r'AutoNAT status.*?(Public|Private|Unknown)', re.I)

def read(path):
    try:
        with open(path, 'r', errors='ignore') as f: return f.read()
    except: return ""

def strip_ansi(s): return re.sub(r'\x1b\[[^m]*m', '', s)

contents  = {n: strip_ansi(read(p)) for n, p in logs.items()}
file_to_id = {}
for name, content in contents.items():
    for pat in OWN_ID_PATTERNS:
        m = pat.search(content)
        if m and len(m.group(1)) >= 52:
            file_to_id[name] = m.group(1)
            break

matrix = {name: set(PAT.findall(c)) for name, c in contents.items()}

print(f"  {'Node':<10} {'Own ID':<26} {'Lines':>6} {'Relays':>6} {'NAT':<9} {'Connects':>9} {'Errors':>7}")
print("  " + "─" * 82)
for name in logs:
    c   = contents[name]
    pid = file_to_id.get(name, 'unknown')
    pid_d = (pid[:22] + '..') if len(pid) > 22 else pid
    lines  = c.count('\n')
    relays = len(RELAY_PAT.findall(c))
    nat_m  = NAT_PAT.findall(c)
    nat    = nat_m[-1].lower() if nat_m else '?'
    conns  = len(CONNECT_PAT.findall(c))
    errs   = len(ERROR_PAT.findall(c))
    icon   = '✅' if pid != 'unknown' else '⏳'
    print(f"  {icon} {name:<8} {pid_d:<26} {lines:>6} {relays:>6} {nat:<9} {conns:>9} {errs:>7}")

print()
print("  Visibility Matrix:")
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
    print(f"  ⚠️  Partial mesh — {gaps} gap(s). Run longer (--time=10) for full ID exchange.")
PY
  else
    echo "  python3 not found — log sizes:"
    for log in "$LOGDIR"/*.log; do
      echo "    $(basename "$log"): $(wc -l < "$log") lines"
    done
  fi

  echo ""
  echo "Logs saved to: $LOGDIR/"
  echo "Done."
  exit 0
}

trap "shutdown" INT TERM

# ── Wait for duration then auto-exit ──────────────────────────────────────────
sleep $((DURATION_MIN * 60))
echo ""
echo "⏰ Time limit (${DURATION_MIN}m) reached."
kill -TERM $$
