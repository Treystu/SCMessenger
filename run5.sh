#!/bin/bash
# run5.sh — Start all 5 mesh nodes and collect logs
# Nodes: GCP (headless relay), OSX (headless relay), Android (Pixel 6a / cellular),
#        iOS Device (iPhone 15 Pro Max), iOS Simulator
#
# Nodes: GCP (headless relay), OSX (headless relay), Android (auto-detect),
#        iOS Device (auto-detect), iOS Simulator (auto-detect)
#
# Usage: ./run5.sh [--time=5] [--update]
set -euo pipefail

DURATION_MIN=5
UPDATE_APPS=0
while [ $# -gt 0 ]; do
  case "$1" in
    -t=*|--time=*)
      DURATION_MIN="${1#*=}"
      ;;
    -u|--update)
      UPDATE_APPS=1
      ;;
    *)
      ;;
  esac
  shift
done

LOGDIR="logs/5mesh"
mkdir -p "$LOGDIR"

APPLE_TEAM_ID=$(security find-identity -v -p codesigning 2>/dev/null | grep -oE '[A-Z0-9]{10}' | head -1 || true)
IOS_DEVICE_UDID="${IOS_DEVICE_UDID:-$(xcrun devicectl list devices 2>/dev/null | awk '/ connected / {print $3; exit}')}"
IOS_SIM_UDID="${IOS_SIM_UDID:-$(xcrun simctl list devices | awk -F '[()]' '/Booted/ {print $2; exit}')}"
BUNDLE_ID="SovereignCommunications.SCMessenger"
TIMESTAMP=$(date '+%Y%m%d_%H%M%S')
SYNC_MARKER="=== TEST_START_MARKER: $(date -u +'%Y-%m-%dT%H:%M:%SZ') ==="

if [ -z "${IOS_DEVICE_UDID:-}" ]; then
  echo "⚠️  No connected iOS physical device found (devicectl) — skipping."
fi

if [ -z "${IOS_SIM_UDID:-}" ]; then
  DEFAULT_SIM=$(xcrun simctl list devices available | awk -F '[()]' '/iPhone 16e/ {print $2; exit}')
  if [ -n "${DEFAULT_SIM:-}" ]; then
    xcrun simctl boot "$DEFAULT_SIM" >/dev/null 2>&1 || true
    IOS_SIM_UDID="$DEFAULT_SIM"
  else
    IOS_SIM_UDID=$(xcrun simctl list devices available | awk -F '[()]' '/iPhone/ {print $2; exit}')
    [ -n "${IOS_SIM_UDID:-}" ] && xcrun simctl boot "$IOS_SIM_UDID" >/dev/null 2>&1 || true
  fi
fi

if ! adb get-state >/dev/null 2>&1; then
  echo "⚠️  Android device not attached/authorized for adb — skipping."
  ANDROID_AVAILABLE=0
else
  ANDROID_AVAILABLE=1
fi

echo "========================================"
echo "  SCMessenger Adaptable Mesh Test — $TIMESTAMP"
echo "  Duration: ${DURATION_MIN}m | Auto-Update: $UPDATE_APPS"
echo "  Android: $( [ "$ANDROID_AVAILABLE" = "1" ] && echo "Ready" || echo "none" )"
echo "  iOS Device: ${IOS_DEVICE_UDID:-none}"
echo "  iOS Sim: ${IOS_SIM_UDID:-none}"
echo "  Logs → $LOGDIR/"
echo "========================================"
echo ""

# ── 1. GCP headless relay ─────────────────────────────────────────────────────
echo "1. Streaming GCP relay logs..."
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a \
  --command="CID=\$(sudo docker ps -q | head -n1); if [ -n \"\$CID\" ]; then echo \"$SYNC_MARKER\"; sudo docker logs -f \"\$CID\" 2>&1; else echo 'No running container found on scmessenger-bootstrap'; fi" \
  > "$LOGDIR/gcp.log" 2>&1 &
GCP_PID=$!
echo "   GCP PID=$GCP_PID → $LOGDIR/gcp.log"

# ── 2. OSX headless relay ─────────────────────────────────────────────────────
echo "2. Starting OSX relay node..."
pkill -f "scmessenger-cli" 2>/dev/null || true
echo "$SYNC_MARKER" > "$LOGDIR/osx.log"
sleep 0.5
# RUST_LOG includes autonat + dcutr + relay at debug so we see NAT probes & hole-punches
RUST_LOG=info,libp2p_autonat=debug,libp2p_dcutr=debug,libp2p_relay=debug,scmessenger_core::transport::swarm=debug,scmessenger_core::store::relay_custody=debug,scmessenger_core::mesh::delivery=debug \
  cargo run -p scmessenger-cli -- relay \
  --listen /ip4/0.0.0.0/tcp/9010 \
  --http-port 9011 \
  >> "$LOGDIR/osx.log" 2>&1 &
OSX_PID=$!
echo "   OSX PID=$OSX_PID → $LOGDIR/osx.log"

ANDROID_PID=""
if [ "$ANDROID_AVAILABLE" = "1" ]; then
  echo "3. Launching SCMessenger on Android..."
  # Bring to foreground without force-stop (preserves mesh state)
  adb shell am start -n com.scmessenger.android/.ui.MainActivity > /dev/null 2>&1 || true
  echo "$SYNC_MARKER" > "$LOGDIR/android.log"
  sleep 1
  # Capture all SCMessenger-relevant tags at Verbose + BLE + Rust bridge + Rust core
  adb logcat -v threadtime \
    MeshRepository:V \
    SwarmBridge:V \
    IronCore:V \
    CoreDelegateImpl:V \
    MainViewModel:V \
    DashboardViewModel:V \
    BleScanner:V \
    BleGattClient:V \
    BleGattServer:V \
    BleAdvertiser:V \
    MeshService:V \
    ContactsViewModel:V \
    Rust:V \
    SCMessengerCore:V \
    rust_logger:V \
    "*:S" \
    >> "$LOGDIR/android.log" 2>&1 &
  ANDROID_PID=$!
  echo "   Android PID=$ANDROID_PID → $LOGDIR/android.log"
else
  echo "3. ⚠️  Skipping Android launch."
fi

# ── 4. iOS Device (iPhone 15 Pro Max) ─────────────────────────────────────────

if [ -n "${IOS_DEVICE_UDID:-}" ]; then
  echo "4. Launching SCMessenger on iOS Device..."
  if [ "$UPDATE_APPS" = "1" ]; then
    # Find freshly built app
    IOS_DEVICE_APP=$(find iOS/SCMessenger/build/Build/Products/Debug-iphoneos \
                          iOS/SCMessenger/build/Device/Build/Products/Debug-iphoneos \
                      -name "SCMessenger.app" -not -path "*/dSYM*" 2>/dev/null | head -1 || true)

    if [ -n "$IOS_DEVICE_APP" ]; then
      echo "   Installing $IOS_DEVICE_APP..."
      xcrun devicectl device install app \
        --device "$IOS_DEVICE_UDID" \
        "$IOS_DEVICE_APP" 2>&1 | grep -E "Install|Error|error|Success" || true
    else
      echo "   ⚠️  No built device app found — skipping reinstall."
    fi
  fi

  echo "$SYNC_MARKER" > "$LOGDIR/ios-device.log"
  # 1. Launch the process and get basic console output
  xcrun devicectl device process launch \
    --device "$IOS_DEVICE_UDID" \
    --console \
    --terminate-existing \
    "$BUNDLE_ID" \
    >> "$LOGDIR/ios-device.log" 2>&1 &
  IOS_DEV_PID=$!

  # 2. Concurrently stream system logs (specifically Bluetooth/Multipeer drop reasons)
  xcrun devicectl device info log stream \
    --device "$IOS_DEVICE_UDID" \
    --predicate 'process == "SCMessenger" OR subsystem == "com.apple.bluetooth" OR subsystem == "com.apple.MultipeerConnectivity"' \
    >> "$LOGDIR/ios-device.log" 2>&1 &
  IOS_DEV_STREAM_PID=$!

  echo "   iOS Dev PID=$IOS_DEV_PID, Stream PID=$IOS_DEV_STREAM_PID → $LOGDIR/ios-device.log"
else
  IOS_DEV_PID=""
  IOS_DEV_STREAM_PID=""
  echo "4. ⚠️  Skipping iOS device launch."
fi

# ── 5. iOS Simulator ──────────────────────────────────────────────────────────

if [ -n "${IOS_SIM_UDID:-}" ]; then
  echo "5. Launching SCMessenger on iOS Simulator..."
  if [ "$UPDATE_APPS" = "1" ]; then
    IOS_SIM_APP=$(find iOS/SCMessenger/build_sim/Build/Products/Debug-iphonesimulator \
                       iOS/SCMessenger/build/Build/Products/Debug-iphonesimulator \
                  -name "SCMessenger.app" -not -path "*/dSYM*" 2>/dev/null | head -1 || true)

    if [ -n "$IOS_SIM_APP" ]; then
      echo "   Installing $IOS_SIM_APP..."
      xcrun simctl install "$IOS_SIM_UDID" "$IOS_SIM_APP" 2>&1 || true
    fi
  fi
  xcrun simctl launch "$IOS_SIM_UDID" "$BUNDLE_ID" > /dev/null 2>&1 || true
  echo "$SYNC_MARKER" > "$LOGDIR/ios-sim.log"
  # Stream logs: info+ from SCMessenger process; captures NSLog + os_log
  xcrun simctl spawn "$IOS_SIM_UDID" log stream \
    --level info \
    --style compact \
    --predicate 'process == "SCMessenger"' \
    >> "$LOGDIR/ios-sim.log" 2>&1 &
  IOS_SIM_PID=$!
  echo "   iOS Sim PID=$IOS_SIM_PID → $LOGDIR/ios-sim.log"
else
  IOS_SIM_PID=""
  echo "5. ⚠️  Skipping iOS simulator launch."
fi

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "========================================"
echo "Available nodes started. Auto-stopping in ${DURATION_MIN}m."
echo "  GCP     PID=$GCP_PID        → $LOGDIR/gcp.log"
echo "  OSX     PID=$OSX_PID        → $LOGDIR/osx.log"
[ -n "$ANDROID_PID" ] && echo "  Android PID=$ANDROID_PID    → $LOGDIR/android.log"
[ -n "$IOS_DEV_PID" ] && echo "  iOS Dev PID=$IOS_DEV_PID    → $LOGDIR/ios-device.log"
[ -n "$IOS_SIM_PID" ] && echo "  iOS Sim PID=$IOS_SIM_PID    → $LOGDIR/ios-sim.log"
echo "========================================"
echo ""
echo "Monitor in another terminal:"
echo "  python3 analyze_mesh.py"
echo ""
echo "Quick live checks:"
echo "  grep -E '🔭|✅|🕳️|NAT|relay|BLE discov|peer.*identif' $LOGDIR/osx.log | tail -20"
echo "  adb logcat -v time MeshRepository:V BleScanner:V '*:S' | grep -E 'NAT|BLE|peer|isFull'"
echo ""
echo "Press Ctrl+C to stop all nodes..."
echo ""

# ── Live status ticker: print key events to terminal every 15s ────────────────
status_ticker() {
  sleep 10
  while true; do
    echo ""
    echo "── $(date '+%H:%M:%S') Status ─────────────────────────────────"

    # OSX: relay reservations & peer count
    OSX_RESERVATIONS=$(grep -c "Relay circuit reservation" "$LOGDIR/osx.log" 2>/dev/null || echo 0)
    OSX_PEERS=$(grep -oE "12D3KooW[A-Za-z0-9]+" "$LOGDIR/osx.log" 2>/dev/null | sort -u | wc -l | tr -d ' ')
    OSX_NAT=$(grep "🔭 NAT status" "$LOGDIR/osx.log" 2>/dev/null | tail -1 | grep -oE 'public|private|unknown' || echo "?")
    echo "  OSX:     $OSX_PEERS unique peers seen, $OSX_RESERVATIONS relay reservations, NAT=$OSX_NAT"

    # GCP: connections
    GCP_CIRCUITS=$(grep -c "circuit\|Circuit" "$LOGDIR/gcp.log" 2>/dev/null || echo 0)
    echo "  GCP:     $GCP_CIRCUITS circuit events"

    # Android: BLE + peer discovery
    ANDROID_BLEPEER=$(grep -c "BLE.*scan\|peripheral.*discover\|Peer.*discov\|isFull" "$LOGDIR/android.log" 2>/dev/null || echo 0)
    ANDROID_NAT=$(grep "NAT status" "$LOGDIR/android.log" 2>/dev/null | tail -1 | grep -oE 'public|private|unknown' || echo "?")
    echo "  Android: $ANDROID_BLEPEER discovery/BLE events, NAT=$ANDROID_NAT"

    # iOS device
    IOS_DEV_LINES=$(wc -l < "$LOGDIR/ios-device.log" 2>/dev/null || echo 0)
    IOS_DEV_PEERS=$(grep -c "peer.*discov\|Peer.*discov\|BLE.*identity\|identity.*read" "$LOGDIR/ios-device.log" 2>/dev/null || echo 0)
    echo "  iOS Dev: $IOS_DEV_LINES log lines, $IOS_DEV_PEERS peer/identity events"

    # iOS sim
    IOS_SIM_PEERS=$(grep -c "peer.*discov\|Peer.*discov\|BLE.*identity\|identity.*read" "$LOGDIR/ios-sim.log" 2>/dev/null || echo 0)
    echo "  iOS Sim: $IOS_SIM_PEERS peer/identity events"

    # Recent notable events across all logs
    echo "  Recent:"
    grep -hE "🔭 NAT|✅ Relay|🕳️ DCUtR|🔌 Inbound|🔌 Outbound|Peer.*identif|BLE identity|isFull=true" \
      "$LOGDIR"/*.log 2>/dev/null | tail -5 | sed 's/^/    /'

    echo "──────────────────────────────────────────────"
    sleep 15
  done
}

status_ticker &
TICKER_PID=$!

# Clean shutdown on Ctrl+C
trap "echo ''; echo 'Stopping all nodes...'; kill $GCP_PID $OSX_PID $ANDROID_PID $IOS_DEV_PID $IOS_DEV_STREAM_PID $IOS_SIM_PID $TICKER_PID 2>/dev/null; echo 'Done.'; exit 0" INT TERM

# Wait for requested duration, then exit
sleep $((DURATION_MIN * 60))
echo ""
echo "⏰ Time limit (${DURATION_MIN}m) reached."
kill -TERM $$
