#!/bin/bash
# Efficiently search for TODOs and incomplete implementations in SCMessenger
# This script avoids large binary/build directories to prevent hanging.

SEARCH_DIR=${1:-"."}

echo "Starting SCMessenger Repository Audit..."
echo "Target: $SEARCH_DIR"
echo "----------------------------------------"

# Define patterns
PATTERNS="TODO|FIXME|HACK|unimplemented\!|panic\!|placeholder|incomplete|missing|TBD"

# Use find to limit search to source files and common readable formats
# Exclude giant directories explicitly
find "$SEARCH_DIR" \
    -path "*/target" -prune -o \
    -path "*/.git" -prune -o \
    -path "*/.gradle" -prune -o \
    -path "*/build" -prune -o \
    -path "*/node_modules" -prune -o \
    -path "*/DerivedData" -prune -o \
    -path "*/.agents" -prune -o \
    -path "*/_agents" -prune -o \
    -type f \( -name "*.rs" -o -name "*.kt" -o -name "*.swift" -o -name "*.ts" -o -name "*.js" -o -name "*.h" -o -name "*.c" -o -name "*.md" -o -name "Cargo.toml" \) \
    -print0 | xargs -0 grep -nE "$PATTERNS" | head -n 200

echo "----------------------------------------"
echo "Audit complete."
