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

# Create destination directory if it doesn't exist
echo "2. Creating destination directory..."
mkdir -p ../ios/SCMessenger/Generated

# Copy generated files
echo "3. Copying files..."
cp target/generated-sources/uniffi/swift/api.swift ../ios/SCMessenger/Generated/api.swift
cp target/generated-sources/uniffi/swift/apiFFI.h ../ios/SCMessenger/Generated/apiFFI.h
cp target/generated-sources/uniffi/swift/apiFFI.modulemap ../ios/SCMessenger/Generated/apiFFI.modulemap

echo "✓ Swift bindings copied successfully"
echo
echo "Files copied to ios/SCMessenger/Generated/:"
echo "  - api.swift ($(wc -l < target/generated-sources/uniffi/swift/api.swift) lines)"
echo "  - apiFFI.h"
echo "  - apiFFI.modulemap"
