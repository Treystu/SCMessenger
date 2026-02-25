#!/bin/bash
mkdir -p logs/5mesh

echo "1. Getting GCP logs in background..."
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a --command="sudo docker logs -f \$(sudo docker ps -q)" > logs/5mesh/gcp.log 2>&1 &
GCP_PID=$!

echo "2. Getting OSX Core headless logs in background..."
# This is already running inside an earlier background command, but let's just do it again cleanly to group the logs
pkill scmessenger-cli || true
cargo run -p scmessenger-cli -- relay --listen /ip4/0.0.0.0/tcp/9010 --http-port 9011 > logs/5mesh/osx.log 2>&1 &
OSX_PID=$!

echo "3. Starting Android App and getting logs..."
adb wait-for-device
adb logcat -c
adb shell monkey -p com.scmessenger.android -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1
adb logcat | grep -i scmessenger > logs/5mesh/android.log 2>&1 &
ANDROID_PID=$!

echo "4. Starting iOS Device App and getting logs..."
# Using the pre-identified device UDID
IOS_DEVICE_UDID="4731D564-2F8F-5BC6-B713-D7774AF598F9"
xcrun devicectl device process log stream --device "$IOS_DEVICE_UDID" --predicate 'subsystem == "com.scmessenger" OR process == "SCMessenger"' > logs/5mesh/ios-device.log 2>&1 &
IOS_DEV_PID=$!

echo "5. Starting iOS Simulator App and getting logs..."
SIM_UDID="F7AAF4C8-8431-4660-93FE-6E54C559C6B9"
xcrun simctl launch "$SIM_UDID" SovereignCommunications.SCMessenger >/dev/null 2>&1 || echo "Could not launch app on simulator. Make sure it is installed."
xcrun simctl spawn "$SIM_UDID" log stream --style compact --predicate 'subsystem == "com.scmessenger" OR process == "SCMessenger"' > logs/5mesh/ios-sim.log 2>&1 &
IOS_SIM_PID=$!

echo "All 5 nodes are logging to 'logs/5mesh/'."
echo "PIDs: GCP=$GCP_PID, OSX=$OSX_PID, AND=$ANDROID_PID, IOS_DEV=$IOS_DEV_PID, IOS_SIM=$IOS_SIM_PID"
echo "Press Ctrl+C to stop logging..."
wait 
