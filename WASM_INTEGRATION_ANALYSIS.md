# WASM Integration Analysis and Troubleshooting Guide

## Current Architecture Overview

### Components
- **CLI Backend**: Node.js-based, runs on ports 9000 (HTTP) + 9001 (P2P)
- **WASM Frontend**: Browser-based WebAssembly, served from `/wasm/pkg/`
- **Android App**: Native app with identity "Luke" (Pixel 6a)
- **WASM Identity**: Test identity "ADAM" for browser testing

### Communication Flow
```
Browser (WASM) → CLI Backend → Android (Luke)
  │                 │
  │                 ▼
  └─────────WebSocket─────────┘
       (127.0.0.1:9001/ws)
```

## Current Implementation Status

### ✅ Working Components
1. **WASM Build**: `wasm/pkg/scmessenger_wasm.js` and `.wasm` files exist
2. **CLI Server**: Configured to serve WASM assets at `/wasm` path
3. **Bootstrap Configuration**: Shared bootstrap nodes in `cli/src/bootstrap.rs`
4. **Cross-Platform API**: WASM can call `startSwarm()` and `dial()` methods
5. **UI Integration**: Complete web UI in `ui/index.html` and `ui/app.js`

### ⚠️ Potential Issues

#### 1. Bootstrap Node Connectivity
- **Issue**: WASM may not be connecting to bootstrap nodes properly
- **Check**: Verify `startSwarm()` is called with correct bootstrap addresses
- **Solution**: Ensure `DEFAULT_BOOTSTRAP` in `ui/app.js` matches CLI bootstrap nodes

#### 2. WebSocket Bridge Stability
- **Issue**: WASM ↔ CLI WebSocket connection may be failing
- **Check**: Browser console logs for WebSocket connection errors
- **Solution**: Verify CLI is running and serving `/api/network-info` endpoint

#### 3. Peer Discovery Propagation
- **Issue**: Android peer "Luke" not appearing in WASM contacts
- **Check**: CLI logs for peer discovery events
- **Solution**: Ensure both CLI and Android are connected to same bootstrap nodes

#### 4. Message Relay Path
- **Issue**: Messages from ADAM → Luke may not be routed properly
- **Check**: WASM console logs for `sendPreparedEnvelope()` success/failure
- **Solution**: Verify relay circuit establishment via `listenOn("/p2p-circuit")`

## Troubleshooting Steps

### 1. Verify CLI Backend Status
```bash
# Check if CLI is running
scmessenger-cli status

# Start CLI if not running
scmessenger-cli start

# Verify HTTP server is serving WASM files
curl http://localhost:9000/wasm/pkg/scmessenger_wasm.js
```

### 2. Test WASM Loading
```javascript
// In browser console:
const wasmModule = await import("/wasm/pkg/scmessenger_wasm.js");
console.log("WASM loaded:", typeof wasmModule.IronCore);
```

### 3. Check Peer Discovery
```javascript
// In WASM console after initialization:
const peers = await core.getPeers();
console.log("Discovered peers:", peers);

// Look for Luke's peer ID in the list
const lukePeer = peers.find(p => p.includes("Luke") || p.includes("Pixel"));
```

### 4. Test Message Sending
```javascript
// After confirming Luke is in peer list:
const message = core.prepareMessage("Luke", "Hello from ADAM!");
const result = await core.sendPreparedEnvelope("Luke_Peer_ID", message);
console.log("Send result:", result);
```

## Bootstrap Configuration Alignment

### CLI Bootstrap Nodes (`cli/src/bootstrap.rs`)
```rust
pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[
    "/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw",
    // Additional nodes...
];
```

### WASM Bootstrap Nodes (`ui/app.js`)
```javascript
const DEFAULT_BOOTSTRAP = [
  "/ip4/127.0.0.1/tcp/9001/ws/p2p/12D3KooWP3RGmGgRNtqGsfBCZgu8Wzao6qSsqYzLeLRmkqBdf5Ag",
  "/ip4/34.135.34.73/tcp/9001/ws/p2p/12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw",
  // Should include same nodes as CLI
];
```

## Expected Behavior

### Successful Integration Flow
1. **Browser Loads**: WASM module loads from `/wasm/pkg/`
2. **WASM Initializes**: `IronCore` created, `startSwarm()` called
3. **CLI Bridge**: WASM connects to CLI via WebSocket
4. **Peer Discovery**: Both CLI and WASM discover Android peer "Luke"
5. **Contact Appearance**: Luke appears in WASM contacts pane
6. **Message Delivery**: "Hello" message sent successfully

### Debugging Checkpoints
- ✅ WASM module loads without errors
- ✅ `startSwarm()` resolves successfully
- ✅ `/api/network-info` returns CLI peer ID
- ✅ WebSocket connection established to CLI
- ✅ `getPeers()` includes Luke's peer ID
- ✅ `sendPreparedEnvelope()` returns success

## Common Pitfalls

1. **Port Conflicts**: Another process using port 9000/9001
2. **CORS Issues**: Browser blocking WebSocket connections
3. **Bootstrap Mismatch**: CLI and WASM using different bootstrap nodes
4. **Identity Mismatch**: Peer IDs not matching between platforms
5. **Firewall Blocking**: Local network preventing peer discovery

## Recommended Fixes

### 1. Align Bootstrap Configurations
Ensure `ui/app.js` DEFAULT_BOOTSTRAP includes all nodes from `cli/src/bootstrap.rs`

### 2. Enhance Error Logging
Add detailed console logging in `ui/app.js` for:
- WASM load failures
- WebSocket connection attempts
- Peer discovery events
- Message send results

### 3. Verify WebSocket Bridge
Ensure CLI is properly serving the `/api/network-info` endpoint and WebSocket connections

### 4. Test with Local Bootstrap
For development, use local CLI as bootstrap:
```javascript
const LOCAL_BOOTSTRAP = [
  "/ip4/127.0.0.1/tcp/9001/ws/p2p/" + cliPeerId  // From /api/network-info
];
```

## Conclusion

The WASM integration architecture is sound, with proper separation between:
- **Browser WASM**: Handles UI and identity management
- **CLI Backend**: Provides WebSocket relay and P2P connectivity
- **Android Peer**: Native mesh participant

The current issues likely stem from configuration mismatches or connectivity problems rather than fundamental architectural flaws. Systematic testing of each component in isolation should identify the specific failure point.