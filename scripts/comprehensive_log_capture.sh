#!/bin/bash
# Comprehensive log capture for both platforms
echo "=========================================="
echo "Comprehensive Log Capture - $(date)"
echo "=========================================="
echo ""

# Check devices
echo "Connected devices:"
echo "Android:"
adb devices | grep -v "List"
echo ""

echo "iOS Simulator:"
xcrun simctl list devices | grep Booted | head -1 || echo "No simulator booted"
echo ""

# Create timestamp for this capture
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOGDIR="scripts/logs_${TIMESTAMP}"
mkdir -p "$LOGDIR"

echo "Capturing logs to: $LOGDIR"
echo ""

# Android - clear and capture 60 seconds
echo "Starting Android log capture (60 seconds)..."
adb logcat -c
timeout 60 adb logcat -v time > "$LOGDIR/android_full.log" 2>&1 &
ANDROID_PID=$!

# iOS - capture if simulator running
if xcrun simctl list devices | grep -q Booted; then
    DEVICE_ID=$(xcrun simctl list devices | grep Booted | head -1 | sed 's/.*(\([A-F0-9-]*\)).*/\1/')
    echo "Starting iOS log capture (60 seconds) - Device: $DEVICE_ID"
    timeout 60 xcrun simctl spawn $DEVICE_ID log stream --predicate 'processImagePath contains "SCMessenger"' --level debug > "$LOGDIR/ios_full.log" 2>&1 &
    IOS_PID=$!
else
    IOS_PID=""
    echo "No iOS simulator booted - skipping iOS logs"
fi

echo ""
echo "Please interact with both apps now:"
echo "  - Send messages"
echo "  - Delete conversations"
echo "  - Switch between screens"
echo "  - Trigger any buggy behavior"
echo ""
echo "Capturing for 60 seconds..."

# Wait for captures
wait $ANDROID_PID 2>/dev/null
[ -n "$IOS_PID" ] && wait $IOS_PID 2>/dev/null

echo ""
echo "Log capture complete!"
echo ""

# Extract key issues
echo "Analyzing Android logs..."
grep -i "error\|exception\|crash\|failed" "$LOGDIR/android_full.log" > "$LOGDIR/android_errors.log" 2>/dev/null
grep "SEND_MSG\|ChatViewModel\|loadMessages" "$LOGDIR/android_full.log" > "$LOGDIR/android_messaging.log" 2>/dev/null
grep "12D3\|f77690\|identity\|peer" "$LOGDIR/android_full.log" | head -100 > "$LOGDIR/android_ids.log" 2>/dev/null

if [ -f "$LOGDIR/ios_full.log" ]; then
    echo "Analyzing iOS logs..."
    grep -i "error\|warning\|fault" "$LOGDIR/ios_full.log" > "$LOGDIR/ios_errors.log" 2>/dev/null
    grep -i "delete\|remove\|conversation" "$LOGDIR/ios_full.log" > "$LOGDIR/ios_deletion.log" 2>/dev/null
fi

echo ""
echo "Summary:"
echo "  Android full: $(wc -l < "$LOGDIR/android_full.log") lines"
echo "  Android errors: $(wc -l < "$LOGDIR/android_errors.log") lines"
echo "  Android messaging: $(wc -l < "$LOGDIR/android_messaging.log") lines"
echo "  Android IDs: $(wc -l < "$LOGDIR/android_ids.log") lines"

if [ -f "$LOGDIR/ios_full.log" ]; then
    echo "  iOS full: $(wc -l < "$LOGDIR/ios_full.log") lines"
    echo "  iOS errors: $(wc -l < "$LOGDIR/ios_errors.log") lines"
    echo "  iOS deletion: $(wc -l < "$LOGDIR/ios_deletion.log") lines"
fi

echo ""
echo "Files saved in: $LOGDIR"
