#!/bin/bash
# =============================================================================
# scripts/build_desktop.sh — Desktop build script for SCMessenger KMP
#
# Builds the Rust workspace and then packages the .deb via Gradle.
#
# Usage:
#   ./scripts/build_desktop.sh
# =============================================================================

set -euo pipefail

export CARGO_INCREMENTAL=0

echo "━━━ Building Rust workspace ━━━"
cargo build --workspace

echo "━━━ Packaging .deb ━━━"
cd android && ./gradlew :shared:packageDeb
