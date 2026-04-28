#!/bin/bash
# SCMessenger Headless Build Script for Netlify

set -e

echo "--- Starting Headless Build Process ---"

# 1. Setup Environment
export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found, installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# 2. Build WASM Package
echo "Building scmessenger-wasm..."
cd wasm
wasm-pack build --target web --release
cd ..

# 3. Assemble Dist Directory
echo "Assembling distribution assets..."
mkdir -p dist/wasm

# Copy headless wrapper files
cp headless/index.html dist/
cp headless/main.js dist/

# Copy built WASM and JS glue
# wasm-pack build outputs to wasm/pkg/
cp wasm/pkg/scmessenger_wasm.js dist/wasm/
cp wasm/pkg/scmessenger_wasm_bg.wasm dist/wasm/

# 4. Final Verification
echo "Build complete. Assets in 'dist/':"
ls -R dist/

echo "--- Headless Build Successful ---"
