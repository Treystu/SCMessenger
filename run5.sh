#!/bin/bash
# run5.sh — Start all 5 mesh nodes and collect logs
# Nodes: GCP (headless), OSX (headless), Android (Pixel 6a), iOS Device, iOS Sim
set -euo pipefail

mkdir -p logs/5mesh

# --- GCP relay node ---
echo "1. Streaming GCP relay logs..."
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a \
  --command="sudo docker logs -f \$(sudo docker ps -q)" \
  > logs/5mesh/gcp.log 2>&1 &
GCP_PID=$!

# --- OSX headless relay ---
echo "2. Starting OSX relay node..."
pkill -f scmessenger-cli 2>/dev/null || true
sleep 0.5
RUST_LOG=info cargo run -p scmessenger-cli -- relay \
  --listen /ip4/0.0.0.0/tcp/9010 \
  --http-port 9011 \
  > logs/5mesh/osx.log 2>&1 &
OSX_PID=$!

# --- Android (Pixel 6a) ---
echo "3. Bringing SCMessenger to foreground on Android and collecting logs..."
# Use 'am start' — brings app to foreground without killing it (preserves history/state)
# Do NOT use force-stop or logcat -c here
adb shell am start -n com.scmessenger.android/.ui.MainActivity > /dev/null 2>&1 || true
# Capture: SCMessenger app tags + Rust bridge
adb logcat -v time \
  MeshRepository:V \
  SwarmBridge:V \
  IronCore:V \
  CoreDelegateImpl:V \
  MainViewModel:V \
  BleGattClient:V \
  BleGattServer:V \
  "*:S" \
  > logs/5mesh/android.log 2>&1 &
ANDROID_PID=$!

# --- iOS Device (iPhone 15 Pro Max) ---
echo "4. Launching SCMessenger on iOS Device and streaming console logs..."
IOS_DEVICE_UDID="4731D564-2F8F-5BC6-B713-D7774AF598F9"
# devicectl (Xcode 16): launch app with --console captures stdout + NSLog to terminal
# --terminate-existing restarts any already-running instance cleanly
xcrun devicectl device process launch \
  --device "$IOS_DEVICE_UDID" \
  --console \
  --terminate-existing \
  "SovereignCommunications.SCMessenger" \
  > logs/5mesh/ios-device.log 2>&1 &
IOS_DEV_PID=$!

# --- iOS Simulator ---
echo "5. Launching SCMessenger on iOS Simulator and collecting logs..."
SIM_UDID="F7AAF4C8-8431-4660-93FE-6E54C559C6B9"
# Launch app (ignore if already running)
xcrun simctl launch "$SIM_UDID" SovereignCommunications.SCMessenger > /dev/null 2>&1 || true
# Stream logs: all output from the SCMessenger process
xcrun simctl spawn "$SIM_UDID" log stream --level info --style compact \
  --predicate 'process == "SCMessenger"' \
  > logs/5mesh/ios-sim.log 2>&1 &
IOS_SIM_PID=$!

echo ""
echo "========================================"
echo "All 5 nodes started. Logs → logs/5mesh/"
echo "  GCP    PID=$GCP_PID      → logs/5mesh/gcp.log"
echo "  OSX    PID=$OSX_PID      → logs/5mesh/osx.log"
echo "  Android PID=$ANDROID_PID → logs/5mesh/android.log"
echo "  iOS Dev PID=$IOS_DEV_PID → logs/5mesh/ios-device.log"
echo "  iOS Sim PID=$IOS_SIM_PID → logs/5mesh/ios-sim.log"
echo "========================================"
echo ""
echo "Run in another terminal: python3 analyze_mesh.py"
echo "Press Ctrl+C to stop all logging..."

# Clean shutdown on Ctrl+C
trap "echo 'Stopping...'; kill $GCP_PID $OSX_PID $ANDROID_PID $IOS_DEV_PID $IOS_SIM_PID 2>/dev/null; exit 0" INT TERM
wait
