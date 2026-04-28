#!/bin/bash
# Capture logs from both Android and iOS simultaneously
echo "Capturing logs from both devices..."
echo "Android device:"
adb devices | grep device

echo ""
echo "iOS devices:"
xcrun simctl list devices | grep Booted || echo "No simulator booted. Checking physical devices..."
idevice_id -l 2>/dev/null || echo "No physical iOS devices (install libimobiledevice if needed)"

echo ""
echo "Starting log capture for 30 seconds..."
echo "Please interact with both apps now."
echo ""

# Android logs
adb logcat -c
adb logcat -v time "*:D" > scripts/android_live.log &
ANDROID_PID=$!

# iOS logs - try simulator first, then device
if xcrun simctl list devices | grep -q Booted; then
    DEVICE_ID=$(xcrun simctl list devices | grep Booted | head -1 | sed 's/.*(\(.*\)).*/\1/')
    xcrun simctl spawn $DEVICE_ID log stream --predicate 'processImagePath contains "SCMessenger"' --level debug > scripts/ios_live.log &
    IOS_PID=$!
else
    echo "Note: iOS logs require booted simulator or physical device"
    IOS_PID=""
fi

sleep 30

kill $ANDROID_PID 2>/dev/null
[ -n "$IOS_PID" ] && kill $IOS_PID 2>/dev/null

echo ""
echo "Logs captured:"
echo "  Android: scripts/android_live.log ($(wc -l < scripts/android_live.log) lines)"
[ -f scripts/ios_live.log ] && echo "  iOS: scripts/ios_live.log ($(wc -l < scripts/ios_live.log) lines)"
