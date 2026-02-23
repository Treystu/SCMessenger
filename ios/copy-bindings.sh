#!/bin/bash
# Copy UniFFI Swift bindings to iOS project
set -e

# Navigate to project root
cd "$(dirname "$0")/.."

echo "=== Copying Swift bindings to iOS project ==="

# Generate bindings first
echo "1. Generating Swift bindings..."
cd core
cargo run --bin gen_swift --features gen-bindings

# Create destination directories if they don't exist.
# Main app target uses SCMessenger/SCMessenger/Generated, while
# SCMessenger/Generated may be referenced by older docs/scripts.
echo "2. Creating destination directories..."
mkdir -p ../iOS/SCMessenger/SCMessenger/Generated
mkdir -p ../iOS/SCMessenger/Generated

# Copy generated files to both locations to keep project/docs in sync.
echo "3. Copying files..."
cp target/generated-sources/uniffi/swift/api.swift ../iOS/SCMessenger/SCMessenger/Generated/api.swift
cp target/generated-sources/uniffi/swift/apiFFI.h ../iOS/SCMessenger/SCMessenger/Generated/apiFFI.h
cp target/generated-sources/uniffi/swift/apiFFI.modulemap ../iOS/SCMessenger/SCMessenger/Generated/apiFFI.modulemap
cp target/generated-sources/uniffi/swift/api.swift ../iOS/SCMessenger/Generated/api.swift
cp target/generated-sources/uniffi/swift/apiFFI.h ../iOS/SCMessenger/Generated/apiFFI.h
cp target/generated-sources/uniffi/swift/apiFFI.modulemap ../iOS/SCMessenger/Generated/apiFFI.modulemap

echo "✓ Swift bindings copied successfully"
echo
echo "Files copied to:"
echo "  - iOS/SCMessenger/SCMessenger/Generated/"
echo "  - iOS/SCMessenger/Generated/"
echo "  - api.swift ($(wc -l < target/generated-sources/uniffi/swift/api.swift) lines)"
echo "  - apiFFI.h"
echo "  - apiFFI.modulemap"
