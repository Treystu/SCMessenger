#!/bin/bash
# run5.sh — Start all 5 mesh nodes and collect logs
# Nodes: GCP (headless relay), OSX (headless relay), Android (Pixel 6a / cellular),
#        iOS Device (iPhone 15 Pro Max), iOS Simulator
#
# Android is expected to run on CELLULAR to validate WAN/NAT traversal.
# BLE discovery between Android ↔ iOS Device are logged at V level.
#
# Usage: ./run5.sh
set -uo pipefail

LOGDIR="logs/5mesh"
mkdir -p "$LOGDIR"

APPLE_TEAM_ID=$(security find-identity -v -p codesigning 2>/dev/null | grep -oE '[A-Z0-9]{10}' | head -1)
IOS_DEVICE_UDID="00008130-001A48DA18EB8D3A"
IOS_SIM_UDID="F7AAF4C8-8431-4660-93FE-6E54C559C6B9"
BUNDLE_ID="SovereignCommunications.SCMessenger"
TIMESTAMP=$(date '+%Y%m%d_%H%M%S')

echo "========================================"
echo "  SCMessenger 5-Node Mesh Test — $TIMESTAMP"
echo "  Android: CELLULAR (NAT traversal test)"
echo "  Logs → $LOGDIR/"
echo "========================================"
echo ""

# ── 1. GCP headless relay ─────────────────────────────────────────────────────
echo "1. Streaming GCP relay logs..."
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a \
  --command="sudo docker logs -f \$(sudo docker ps -q) 2>&1" \
  > "$LOGDIR/gcp.log" 2>&1 &
GCP_PID=$!
echo "   GCP PID=$GCP_PID → $LOGDIR/gcp.log"

# ── 2. OSX headless relay ─────────────────────────────────────────────────────
echo "2. Starting OSX relay node..."
pkill -f "scmessenger-cli" 2>/dev/null || true
sleep 0.5
# RUST_LOG includes autonat + dcutr + relay at debug so we see NAT probes & hole-punches
RUST_LOG=info,libp2p_autonat=debug,libp2p_dcutr=debug,libp2p_relay=debug,scmessenger_core::transport::swarm=debug \
  cargo run -p scmessenger-cli -- relay \
  --listen /ip4/0.0.0.0/tcp/9010 \
  --http-port 9011 \
  > "$LOGDIR/osx.log" 2>&1 &
OSX_PID=$!
echo "   OSX PID=$OSX_PID → $LOGDIR/osx.log"

# ── 3. Android (Pixel 6a) ─────────────────────────────────────────────────────
echo "3. Launching SCMessenger on Android (ensure it is on CELLULAR, not WiFi)..."
# Bring to foreground without force-stop (preserves mesh state)
adb shell am start -n com.scmessenger.android/.ui.MainActivity > /dev/null 2>&1 || true
sleep 1
# Capture all SCMessenger-relevant tags at Verbose + BLE + Rust bridge
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
  "*:S" \
  > "$LOGDIR/android.log" 2>&1 &
ANDROID_PID=$!
echo "   Android PID=$ANDROID_PID → $LOGDIR/android.log"

# ── 4. iOS Device (iPhone 15 Pro Max) ─────────────────────────────────────────
echo "4. Installing + launching SCMessenger on iOS Device..."
# Find freshly built app
IOS_DEVICE_APP=$(find iOS/SCMessenger/build/Build/Products/Debug-iphoneos \
                      iOS/SCMessenger/build/Device/Build/Products/Debug-iphoneos \
                  -name "SCMessenger.app" -not -path "*/dSYM*" 2>/dev/null | head -1)

if [ -n "$IOS_DEVICE_APP" ]; then
  echo "   Installing $IOS_DEVICE_APP..."
  xcrun devicectl device install app \
    --device "$IOS_DEVICE_UDID" \
    "$IOS_DEVICE_APP" 2>&1 | grep -E "Install|Error|error|Success" || true
else
  echo "   ⚠️  No built device app found — launching existing install (may be stale)"
fi

xcrun devicectl device process launch \
  --device "$IOS_DEVICE_UDID" \
  --console \
  --terminate-existing \
  "$BUNDLE_ID" \
  > "$LOGDIR/ios-device.log" 2>&1 &
IOS_DEV_PID=$!
echo "   iOS Dev PID=$IOS_DEV_PID → $LOGDIR/ios-device.log"

# ── 5. iOS Simulator ──────────────────────────────────────────────────────────
echo "5. Installing + launching SCMessenger on iOS Simulator..."
IOS_SIM_APP=$(find iOS/SCMessenger/build_sim/Build/Products/Debug-iphonesimulator \
                   iOS/SCMessenger/build/Build/Products/Debug-iphonesimulator \
              -name "SCMessenger.app" -not -path "*/dSYM*" 2>/dev/null | head -1)

if [ -n "$IOS_SIM_APP" ]; then
  echo "   Installing $IOS_SIM_APP..."
  xcrun simctl install "$IOS_SIM_UDID" "$IOS_SIM_APP" 2>&1 || true
fi

xcrun simctl launch "$IOS_SIM_UDID" "$BUNDLE_ID" > /dev/null 2>&1 || true
# Stream logs: info+ from SCMessenger process; captures NSLog + os_log
xcrun simctl spawn "$IOS_SIM_UDID" log stream \
  --level info \
  --style compact \
  --predicate 'process == "SCMessenger"' \
  > "$LOGDIR/ios-sim.log" 2>&1 &
IOS_SIM_PID=$!
echo "   iOS Sim PID=$IOS_SIM_PID → $LOGDIR/ios-sim.log"

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "========================================"
echo "All 5 nodes started."
echo "  GCP     PID=$GCP_PID        → $LOGDIR/gcp.log"
echo "  OSX     PID=$OSX_PID        → $LOGDIR/osx.log"
echo "  Android PID=$ANDROID_PID    → $LOGDIR/android.log"
echo "  iOS Dev PID=$IOS_DEV_PID    → $LOGDIR/ios-device.log"
echo "  iOS Sim PID=$IOS_SIM_PID    → $LOGDIR/ios-sim.log"
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
trap "echo ''; echo 'Stopping all nodes...'; kill $GCP_PID $OSX_PID $ANDROID_PID $IOS_DEV_PID $IOS_SIM_PID $TICKER_PID 2>/dev/null; echo 'Done.'; exit 0" INT TERM

wait
