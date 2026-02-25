#!/bin/bash
echo "Clearing OSX DBs..."
rm -rf core/target/debug/contacts.db core/target/debug/history.db
rm -rf ~/.scmessenger 2>/dev/null || true

echo "Clearing Android DBs..."
adb shell pm clear com.scmessenger.android

echo "Clearing iOS Simulator..."
xcrun simctl uninstall F7AAF4C8-8431-4660-93FE-6E54C559C6B9 SovereignCommunications.SCMessenger

echo "Clearing iOS Device..."
xcrun devicectl device process delete --device 4731D564-2F8F-5BC6-B713-D7774AF598F9 SovereignCommunications.SCMessenger 2>/dev/null || true

echo "Done clearing databases."
