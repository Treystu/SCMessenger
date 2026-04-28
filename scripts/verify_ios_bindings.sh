#!/bin/bash
# verify_ios_bindings.sh
# Verifies that the generated iOS Swift bindings are in sync with the UDL definition.

set -e

UDL_FILE="core/src/api.udl"
SWIFT_FILE="iOS/SCMessenger/SCMessenger/Generated/api.swift"

echo "Comparing $UDL_FILE and $SWIFT_FILE..."

# Extract fields from UDL and convert snake_case to camelCase
UDL_FIELDS=$(perl -ne 'if (/dictionary IdentityInfo \{/ .. /\}/) { print "$1\n" if /([a-z0-9_]+);/ }' "$UDL_FILE" | perl -pe 's/_([a-z])/\U$1/g' | sort)
SWIFT_FIELDS=$(grep -A 25 "public struct IdentityInfo {" "$SWIFT_FILE" | grep "public var" | awk '{print $3}' | sed 's/://' | sort)

if [ "$UDL_FIELDS" != "$SWIFT_FIELDS" ]; then
    echo "❌ ERROR: IdentityInfo fields are out of sync!"
    echo "UDL Fields:"
    echo "$UDL_FIELDS"
    echo "Swift Fields:"
    echo "$SWIFT_FIELDS"
    exit 1
else
    echo "✅ IdentityInfo fields are in sync."
fi

# Check for SwarmBridge::send_message signature
echo "Checking SwarmBridge::send_message signature..."
UDL_SIG=$(grep "void send_message" "$UDL_FILE" | tr -d '[:space:]')
# In Swift it looks like: func sendMessage(peerId: String, data: Data, recipientIdentityId: String?, intendedDeviceId: String?)
SWIFT_SIG=$(grep "func sendMessage" "$SWIFT_FILE" | head -n 1 | tr -d '[:space:]')

# Check if new fields are present in Swift
if [[ "$SWIFT_SIG" == *"recipientIdentityId"* ]] && [[ "$SWIFT_SIG" == *"intendedDeviceId"* ]]; then
    echo "✅ SwarmBridge::send_message has WS13 metadata fields."
else
    echo "❌ ERROR: SwarmBridge::send_message is missing WS13 metadata fields!"
    echo "Swift signature found: $SWIFT_SIG"
    exit 1
fi

echo "🚀 iOS Binding Verification Passed!"
