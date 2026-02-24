#!/usr/bin/env bash
# Copy UniFFI Swift bindings to iOS project (canonical path only).
set -euo pipefail

# Navigate to project root
cd "$(dirname "$0")/.."

echo "=== Copying Swift bindings to iOS project ==="

# Generate bindings first
echo "1. Generating Swift bindings..."
cd core
cargo run --bin gen_swift --features gen-bindings

canonical_dir="../iOS/SCMessenger/SCMessenger/Generated"
legacy_dir="../iOS/SCMessenger/Generated"

echo "2. Creating canonical destination directory..."
mkdir -p "$canonical_dir"

echo "3. Removing legacy generated outputs..."
legacy_removed="not-present"
if [ -d "$legacy_dir" ]; then
  rm -f "$legacy_dir/api.swift" "$legacy_dir/apiFFI.h" "$legacy_dir/apiFFI.modulemap"
  rmdir "$legacy_dir" 2>/dev/null || true
  legacy_removed="cleaned"
fi

echo "4. Copying files..."
cp target/generated-sources/uniffi/swift/api.swift "$canonical_dir/api.swift"
cp target/generated-sources/uniffi/swift/apiFFI.h "$canonical_dir/apiFFI.h"
cp target/generated-sources/uniffi/swift/apiFFI.modulemap "$canonical_dir/apiFFI.modulemap"

echo "✓ Swift bindings copied successfully"
echo
echo "Files copied to:"
echo "  - iOS/SCMessenger/SCMessenger/Generated/"
echo "  - legacy generated path removed: $legacy_removed"
echo "  - api.swift ($(wc -l < target/generated-sources/uniffi/swift/api.swift) lines)"
echo "  - apiFFI.h"
echo "  - apiFFI.modulemap"

echo
bash "../iOS/assert-generated-path.sh"
