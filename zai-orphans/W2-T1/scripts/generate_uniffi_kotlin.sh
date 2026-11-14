#!/usr/bin/env bash
# =============================================================================
# scripts/generate_uniffi_kotlin.sh — Generate UniFFI Kotlin bindings
#
# Builds the scmessenger-mobile cdylib for the host target, then runs
# the gen_kotlin binary to produce Kotlin bindings.
#
# Output: core/target/generated-sources/uniffi/kotlin/
# =============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo -e "${GREEN}Generating UniFFI Kotlin bindings...${NC}"

# Ensure output directory exists
mkdir -p "$REPO_ROOT/core/target/generated-sources/uniffi/kotlin"

# Build scmessenger-mobile cdylib for host target
echo "Building scmessenger-mobile for host target..."
cargo build -p scmessenger-mobile

# Run the Kotlin binding generator
echo "Running UniFFI Kotlin generator..."
cd "$REPO_ROOT/core"
cargo run --bin gen_kotlin --features gen-bindings
cd "$REPO_ROOT"

# Verify output
BINDING_DIR="$REPO_ROOT/core/target/generated-sources/uniffi/kotlin"
if [ -d "$BINDING_DIR" ] && [ "$(ls -A "$BINDING_DIR")" ]; then
    echo -e "${GREEN} UniFFI Kotlin bindings generated in: ${BINDING_DIR}${NC}"
    find "$BINDING_DIR" -name "*.kt" | head -5
else
    echo -e "${YELLOW} Binding directory is empty: ${BINDING_DIR}${NC}"
fi
