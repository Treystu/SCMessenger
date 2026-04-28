#!/bin/bash
# Watch Android logs for message send operations
echo "Clearing logcat and watching for message send..."
echo "Send a message now in the app..."
echo ""
adb logcat -c
adb logcat | grep -E "ChatViewModel|SEND_MSG|historyManager|messageUpdates|optimistic" | head -50
