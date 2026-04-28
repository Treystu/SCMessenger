#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <android-log> <ios-log>"
    exit 1
fi

ANDROID_LOG="$1"
IOS_LOG="$2"

if [ ! -f "$ANDROID_LOG" ]; then
    echo "Android log file $ANDROID_LOG not found locally. Attempting to pull via adb..."
    if ! adb shell run-as com.scmessenger.android cat files/mesh_diagnostics.log > "$ANDROID_LOG"; then
        echo "FAIL: Could not pull Android log via adb"
        exit 1
    fi
fi

if [ ! -f "$IOS_LOG" ]; then
    echo "FAIL: iOS log file $IOS_LOG not found locally."
    exit 1
fi

if bash ./scripts/verify_receipt_convergence.sh "$ANDROID_LOG" "$IOS_LOG"; then
    echo "PASS: E2E receipt convergence confirmed."
    exit 0
else
    echo "FAIL: E2E receipt convergence checks failed."
    exit 1
fi
