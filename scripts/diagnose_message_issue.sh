#!/bin/bash
# Diagnose message persistence issue on Android
echo "Analyzing message persistence behavior..."
echo ""

# Check what the ChatViewModel is using
adb shell "run-as com.scmessenger.android ls -la /data/data/com.scmessenger.android/databases/" 2>/dev/null || echo "Can't access app data"

# Pull the sled database to inspect
echo ""
echo "Attempting to pull message history database..."
adb pull /data/data/com.scmessenger.android/files/scmessenger/history.db scripts/history_android.db 2>&1 | grep -v "does not exist"

echo ""
echo "Checking for SEND_MSG entries in current logcat buffer..."
adb logcat -d | grep "SEND_MSG" | tail -20

echo ""
echo "Checking for message persistence logs..."
adb logcat -d | grep -E "historyManager.*add|message_prepared_local_history" | tail -10
