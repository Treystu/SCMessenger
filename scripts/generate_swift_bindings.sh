#!/bin/bash
set -euo pipefail

# Generate Swift bindings using UniFFI 0.31
# This script replaces the programmatic generation with a simpler approach

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORE_DIR="$REPO_ROOT/core"

cd "$CORE_DIR"

# Ensure target directory exists
mkdir -p target/generated-sources/uniffi/swift

echo "🔧 Generating Swift bindings with UniFFI 0.31..."

# Use Rust to generate bindings - simplified approach
cargo run --features gen-bindings --bin gen_swift_simple

echo "✅ Swift bindings generated successfully"