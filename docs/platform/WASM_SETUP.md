# WASM Development Setup Guide

Status: Active  
Last updated: 2026-03-07  
Validates: Requirements 5.9

This guide covers setting up your development environment for SCMessenger WASM development.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Setup](#setup)
- [Building](#building)
- [Testing](#testing)
- [Optimization](#optimization)
- [Debugging](#debugging)
- [Common Issues](#common-issues)
- [Deployment](#deployment)
- [Resources](#resources)

## Prerequisites

### Required Software

1. **Rust** 1.75.0 or later
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **wasm-pack** (WASM build tool)
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

3. **Node.js** 20+ and npm
   - Download from: https://nodejs.org/
   - Verify: `node --version && npm --version`

4. **WASM target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Optional Tools

1. **wasm-opt** (for optimization)
   ```bash
   # macOS
   brew install binaryen

   # Linux (Debian/Ubuntu)
   sudo apt-get install binaryen

   # Windows (via npm)
   npm install -g binaryen
   ```

2. **wasm-bindgen-cli** (for advanced usage)
   ```bash
   cargo install wasm-bindgen-cli
   ```

## Setup

### 1. Clone Repository

```bash
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger/wasm
```

### 2. Install Dependencies

```bash
# Install npm dependencies
npm install

# Verify wasm-pack installation
wasm-pack --version
```

### 3. Verify Setup

```bash
# Check Rust toolchain
rustc --version
cargo --version

# Check WASM target
rustup target list | grep wasm32-unknown-unknown

# Check Node.js
node --version
npm --version
```

## Building

### Development Build

```bash
cd wasm

# Build for web target (development)
wasm-pack build --target web --dev

# Output: wasm/pkg/
```

### Release Build

```bash
cd wasm

# Build for web target (release)
wasm-pack build --target web --release

# Output: wasm/pkg/
```

### Build Targets

```bash
# Web (ES modules)
wasm-pack build --target web

# Node.js
wasm-pack build --target nodejs

# Bundler (webpack, rollup)
wasm-pack build --target bundler

# No modules (UMD)
wasm-pack build --target no-modules
```

### Build with Features

```bash
# Build with specific features
wasm-pack build --target web --release -- --features "feature1,feature2"

# Build with all features
wasm-pack build --target web --release -- --all-features
```

## Testing

### Unit Tests (Native)

```bash
cd wasm

# Run tests in native mode
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### WASM Tests (Browser)

```bash
cd wasm

# Run tests in headless browser
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome
wasm-pack test --headless --safari

# Run tests in browser (interactive)
wasm-pack test --firefox
wasm-pack test --chrome
```

### Node.js Tests

```bash
cd wasm

# Build for Node.js
wasm-pack build --target nodejs

# Run Node.js tests
npm test
```

## Optimization

### Size Optimization

```bash
cd wasm

# Build with size optimization
wasm-pack build --target web --release

# Optimize with wasm-opt
wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm

# Check size
ls -lh pkg/scmessenger_wasm_bg.wasm
```

### Optimization Levels

```bash
# -O0: No optimization
wasm-opt -O0 input.wasm -o output.wasm

# -O1: Basic optimization
wasm-opt -O1 input.wasm -o output.wasm

# -O2: More optimization
wasm-opt -O2 input.wasm -o output.wasm

# -O3: Aggressive optimization
wasm-opt -O3 input.wasm -o output.wasm

# -Oz: Optimize for size
wasm-opt -Oz input.wasm -o output.wasm
```

### Bundle Size Analysis

```bash
# Install wasm-pack-size
npm install -g wasm-pack-size

# Analyze bundle size
wasm-pack-size pkg/scmessenger_wasm_bg.wasm

# Detailed analysis
wasm-opt --print-features pkg/scmessenger_wasm_bg.wasm
```

## Debugging

### Browser DevTools

```javascript
// Enable WASM debugging in browser
// Chrome: chrome://flags/#enable-webassembly-debugging
// Firefox: about:config → devtools.debugger.features.wasm

// Load WASM module
import init, { function_name } from './pkg/scmessenger_wasm.js';

await init();
console.log(function_name());
```

### Console Logging

```rust
// In Rust code
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn debug_function() {
    log("Debug message from WASM");
}
```

### Error Handling

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn fallible_function() -> Result<String, JsValue> {
    // Return error to JavaScript
    Err(JsValue::from_str("Error message"))
}
```

### Performance Profiling

```javascript
// Profile WASM performance
console.time('wasm-function');
await wasmFunction();
console.timeEnd('wasm-function');

// Use Performance API
const start = performance.now();
await wasmFunction();
const end = performance.now();
console.log(`Execution time: ${end - start}ms`);
```

## Common Issues

### Issue: "wasm-pack: command not found"

**Solution**:
```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Verify installation
wasm-pack --version
```

### Issue: "error: target 'wasm32-unknown-unknown' not found"

**Solution**:
```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Verify target
rustup target list | grep wasm32
```

### Issue: "RuntimeError: memory access out of bounds"

**Solution**:
```rust
// Increase WASM memory in Cargo.toml
[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-O4", "--enable-bulk-memory"]

// Or in JavaScript
const memory = new WebAssembly.Memory({
    initial: 256,  // 256 pages = 16MB
    maximum: 512   // 512 pages = 32MB
});
```

### Issue: "Module parse failed: Unexpected character"

**Solution**:
```javascript
// Ensure correct import syntax
// ES modules:
import init from './pkg/scmessenger_wasm.js';

// CommonJS:
const init = require('./pkg/scmessenger_wasm.js');
```

### Issue: Large bundle size

**Solution**:
```bash
# Optimize with wasm-opt
wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm

# Enable LTO in Cargo.toml
[profile.release]
lto = true
opt-level = "z"
```

### Issue: "Cannot find module 'wasm-bindgen'"

**Solution**:
```bash
# Install dependencies
npm install

# Or install wasm-bindgen globally
cargo install wasm-bindgen-cli
```

## Deployment

### Static Hosting

```bash
# Build for production
cd wasm
wasm-pack build --target web --release
wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm

# Deploy pkg/ directory to:
# - GitHub Pages
# - Netlify
# - Vercel
# - Cloudflare Pages
```

### NPM Package

```bash
# Build for npm
cd wasm
wasm-pack build --target bundler --release

# Publish to npm
cd pkg
npm publish
```

### CDN Deployment

```bash
# Build for CDN
cd wasm
wasm-pack build --target web --release
wasm-opt -Oz pkg/scmessenger_wasm_bg.wasm -o pkg/scmessenger_wasm_bg.wasm

# Upload to CDN:
# - jsDelivr
# - unpkg
# - Cloudflare CDN
```

### Example HTML

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>SCMessenger WASM</title>
</head>
<body>
    <script type="module">
        import init, { start_messenger } from './pkg/scmessenger_wasm.js';

        async function run() {
            await init();
            start_messenger();
        }

        run();
    </script>
</body>
</html>
```

## Resources

### Documentation

- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [MDN WebAssembly](https://developer.mozilla.org/en-US/docs/WebAssembly)

### Tools

- [wasm-pack](https://github.com/rustwasm/wasm-pack) - Build tool
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - JS bindings
- [wasm-opt](https://github.com/WebAssembly/binaryen) - Optimizer
- [wabt](https://github.com/WebAssembly/wabt) - WebAssembly Binary Toolkit

### Examples

- [Rust WASM Examples](https://github.com/rustwasm/wasm-bindgen/tree/main/examples)
- [wasm-pack Template](https://github.com/rustwasm/wasm-pack-template)

### Community

- [Rust and WebAssembly Working Group](https://github.com/rustwasm)
- [WebAssembly Community](https://webassembly.org/community/)
- [SCMessenger GitHub Issues](https://github.com/Treystu/SCMessenger/issues)

## Getting Help

- Check [Troubleshooting Guide](../troubleshooting/BUILD_ISSUES.md)
- Search [GitHub Issues](https://github.com/Treystu/SCMessenger/issues)
- Ask in [GitHub Discussions](https://github.com/Treystu/SCMessenger/discussions)
- See [CONTRIBUTING.md](../../CONTRIBUTING.md) for contribution guidelines

---

**Next Steps:**
- Read [Architecture Overview](../ARCHITECTURE.md)
- Review [Testing Guide](../TESTING_GUIDE.md)
- Check [Deployment Guide](../DEPLOYMENT.md)
