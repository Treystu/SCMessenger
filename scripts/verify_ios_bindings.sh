#!/bin/bash
# verify_ios_bindings.sh
# Verifies that the generated iOS Swift bindings are in sync with the UDL definition.

set -e

GENERATED_SWIFT="core/target/generated-sources/uniffi/swift/SCMessengerCore.swift"
COMMITTED_SWIFT="iOS/SCMessenger/SCMessenger/Generated/api.swift"

echo "Verifying iOS Swift bindings..."

# Build host-native library required by gen_swift
if ! cargo build -p scmessenger-mobile; then
    echo "ERROR: Failed to build scmessenger-mobile"
    exit 1
fi

# Generate Swift bindings (writes to fixed output directory)
if ! cargo run --bin gen_swift --features gen-bindings; then
    echo "ERROR: Failed to generate Swift bindings"
    exit 1
fi

# Compare generated bindings with committed file
if ! diff -u "$COMMITTED_SWIFT" "$GENERATED_SWIFT"; then
    echo "ERROR: Swift bindings are out of sync!"
    echo "Please regenerate with 'cargo run --bin gen_swift --features gen-bindings' and commit the changes."
    exit 1
fi

echo "iOS Binding Verification Passed!"
