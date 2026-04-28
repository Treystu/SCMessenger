# WASM/CLI Transport Parity Implementation Plan

## Current State Analysis

### Existing Transport Implementation

**WASM (Browser):**
- ✅ WebSocket relay transport (fully implemented)
- ✅ WebRTC peer-to-peer transport (fully implemented)
- ❌ BLE transport (not available in browsers)
- ❌ WiFi Direct transport (not available in browsers)
- ❌ WiFi Aware transport (not available in browsers)
- ❌ LAN/mDNS discovery (limited in browsers)

**CLI (Node.js):**
- ✅ WebSocket relay transport
- ✅ TCP/IP direct transport
- ✅ mDNS LAN discovery
- ❌ BLE transport (not available in Node.js)
- ❌ WiFi Direct transport (not available in Node.js)
- ❌ WiFi Aware transport (not available in Node.js)

**Android (Native):**
- ✅ BLE transport
- ✅ WiFi Direct transport
- ✅ WiFi Aware transport
- ✅ Internet (WebSocket) transport
- ✅ LAN/mDNS discovery

### Key Insight

**Browser limitations are fundamental:** BLE, WiFi Direct, and WiFi Aware are not available in web browsers due to security and API restrictions. However, we can achieve functional parity through the CLI bridge.

## Architecture for Full Parity

### Multi-Layer Transport Strategy

```
Browser (WASM) ↔ CLI Bridge ↔ Android (Luke)
  │               │               │
  │               │               ▼
  ▼               ▼           (BLE, WiFi Direct, WiFi Aware)
WebSocket        TCP/IP        LAN (same network)
  │               │
  ▼               ▼
WebRTC ───────── mDNS 
  │               │
  ▼               ▼
Internet      Local LAN
```

### Transport Path Prioritization

1. **Direct Browser-to-CLI WebSocket** (fastest path for local communication)
2. **Browser WebRTC → CLI WebRTC** (peer-to-peer when possible)
3. **Browser WebSocket → CLI → Android BLE** (for Android proximity)
4. **Browser WebSocket → CLI → Android WiFi Direct** (high bandwidth)
5. **Browser WebSocket → CLI → Android WiFi Aware** (low power)
6. **Browser WebSocket → CLI → Internet Relay** (fallback)

## Implementation Plan

### Phase 1: CLI Transport Bridge Enhancement

**Objective:** Make CLI act as a universal transport bridge for WASM

#### 1.1. Enhance CLI Transport Capabilities

**File: `cli/src/main.rs`**
- Add explicit transport capability advertising
- Implement transport escalation logic
- Add local LAN discovery enhancement

```rust
// Add to CLI capabilities
pub fn advertise_transport_capabilities() -> Vec<TransportType> {
    vec![
        TransportType::Internet,  // WebSocket relay
        TransportType::Local,     // Local TCP/IP
        // Note: BLE/WiFi Direct not available in Node.js
    ]
}
```

#### 1.2. Implement Transport Escalation in CLI

**File: `cli/src/transport_bridge.rs` (new)**
```rust
pub struct TransportBridge {
    wasm_peer_id: PeerId,
    android_peers: HashMap<PeerId, Vec<TransportType>>,
    transport_manager: TransportManager,
}

impl TransportBridge {
    pub fn find_best_path(&self, target: &PeerId) -> Option<TransportRoute> {
        // 1. Check if target is directly reachable via WebSocket
        // 2. Check if CLI can reach target via any native transport
        // 3. Fall back to relay path
    }
}
```

### Phase 2: WASM Transport Awareness

**Objective:** Make WASM aware of all available transport paths through CLI

#### 2.1. Extend WASM Transport API

**File: `wasm/src/lib.rs`**
```rust
#[wasm_bindgen]
impl IronCore {
    /// Get available transport paths to a peer
    pub fn get_available_transports(&self, peer_id: String) -> Result<JsValue, JsValue> {
        // Query CLI for transport capabilities
    }
    
    /// Request transport escalation for a peer
    pub fn request_transport_escalation(&self, peer_id: String) -> Result<JsValue, JsValue> {
        // Ask CLI to find better transport path
    }
}
```

#### 2.2. Enhance WASM Transport Manager

**File: `wasm/src/transport.rs`**
```rust
pub struct WasmTransportManager {
    web_socket_relay: WebSocketRelay,
    web_rtc_transport: WebRtcTransport,
    cli_bridge: CliTransportBridge,  // New
    path_selector: TransportPathSelector, // New
}

impl WasmTransportManager {
    pub fn find_optimal_path(&self, peer_id: &PeerId) -> TransportPath {
        // 1. Check direct WebSocket to peer
        // 2. Check WebRTC to peer
        // 3. Check CLI bridge paths
        // 4. Return best available
    }
}
```

### Phase 3: Path Selection Algorithm

**Objective:** Implement intelligent path selection with fallback

#### 3.1. Transport Scoring System

**File: `wasm/src/path_selector.rs` (new)**
```rust
pub struct TransportPathSelector {
    // Path scoring based on:
    // - Latency (lower is better)
    // - Bandwidth (higher is better)
    // - Power efficiency (for mobile devices)
    // - Reliability (historical success rate)
}

impl TransportPathSelector {
    pub fn score_path(&self, path: &TransportPath) -> u32 {
        match path.transport_type {
            TransportType::Local => 1000,      // Best for local
            TransportType::WiFiDirect => 900,  // High bandwidth
            TransportType::WiFiAware => 800,   // Good balance
            TransportType::BLE => 700,         // Low power
            TransportType::Internet => 600,    // Reliable fallback
        }
    }
}
```

#### 3.2. Dynamic Path Monitoring

**File: `wasm/src/transport_monitor.rs` (new)**
```rust
pub struct TransportMonitor {
    path_stats: HashMap<TransportPath, PathStatistics>,
}

impl TransportMonitor {
    pub fn update_stats(&mut self, path: &TransportPath, success: bool, latency: u32) {
        // Track success rates and performance
    }
    
    pub fn should_escalate(&self, current_path: &TransportPath) -> bool {
        // Decide if we should try a better path
    }
}
```

### Phase 4: CLI-Android Transport Integration

**Objective:** Ensure CLI can leverage Android's native transports

#### 4.1. Enhance CLI Peer Discovery

**File: `cli/src/discovery.rs` (new)**
```rust
pub async fn discover_android_peers() -> Vec<DiscoveredPeer> {
    // Use mDNS to find Android devices on local network
    // Query their transport capabilities
    // Return list with available transport types
}
```

#### 4.2. Implement Transport Handshake

**File: `cli/src/transport_handshake.rs` (new)**
```rust
pub async fn negotiate_best_transport(
    peer_id: &PeerId,
    available_transports: Vec<TransportType>
) -> Result<TransportType, HandshakeError> {
    // 1. Query peer for capabilities
    // 2. Match against CLI capabilities
    // 3. Select best mutual transport
    // 4. Establish connection
}
```

### Phase 5: Reliability Enhancements

**Objective:** Ensure robust communication across all paths

#### 5.1. Multi-Path Messaging

**File: `core/src/transport/manager.rs`**
```rust
pub fn send_with_redundancy(
    &self,
    peer_id: &PeerId,
    message: Vec<u8>,
    primary_path: TransportType,
    fallback_paths: Vec<TransportType>
) -> Result<(), SendError> {
    // Send via primary path
    // If fails, try fallback paths
    // Track success for future path selection
}
```

#### 5.2. Path Health Monitoring

**File: `core/src/transport/health.rs`**
```rust
pub struct PathHealthMonitor {
    success_rates: HashMap<TransportType, f32>,
    latency_history: HashMap<TransportType, Vec<u32>>,
}

impl PathHealthMonitor {
    pub fn update_metrics(&mut self, transport: TransportType, success: bool, latency: u32) {
        // Update statistics
    }
    
    pub fn get_reliability_score(&self, transport: &TransportType) -> f32 {
        // Calculate reliability based on history
    }
}
```

## Testing Strategy

### Test 1: Local Communication Verification
```bash
# Start CLI with full transport advertising
scmessenger-cli start --advertise-transports

# Verify CLI discovers Android peer
scmessenger-cli peers --show-transports
```

### Test 2: WASM Transport Awareness
```javascript
// In browser console
const transports = await core.getAvailableTransports("Luke_Peer_ID");
console.log("Available transports to Luke:", transports);
// Should show: ["Internet", "Local", "WiFiDirect", "BLE"]
```

### Test 3: Path Selection Verification
```javascript
// Force path selection test
const bestPath = await core.selectBestPath("Luke_Peer_ID");
console.log("Selected path:", bestPath);
// Should select WiFiDirect if available on same LAN
```

### Test 4: Message Delivery Reliability
```javascript
// Send test message with redundancy
const result = await core.sendWithRedundancy(
    "Luke_Peer_ID", 
    "Test message",
    ["WiFiDirect", "Internet"]  // Try WiFiDirect first, fall back to Internet
);
```

## Expected Outcomes

### Success Criteria
1. ✅ WASM can discover Android peer "Luke" through CLI bridge
2. ✅ All available transport paths are visible in WASM UI
3. ✅ Messages automatically use best available path
4. ✅ Automatic fallback when primary path fails
5. ✅ Transport capabilities match Android implementation
6. ✅ Reliable communication in all network conditions

### Parity Checklist
- [ ] BLE transport awareness (via CLI bridge)
- [ ] WiFi Direct transport awareness (via CLI bridge)
- [ ] WiFi Aware transport awareness (via CLI bridge)
- [ ] LAN/mDNS discovery (via CLI bridge)
- [ ] WebSocket relay (direct)
- [ ] WebRTC peer-to-peer (direct)
- [ ] Intelligent path selection
- [ ] Automatic transport escalation
- [ ] Multi-path redundancy
- [ ] Path health monitoring

## Implementation Timeline

This plan achieves **functional parity** despite browser limitations by using the CLI as a transport bridge. The result will be a system where:

1. **WASM sees all transports** available through the CLI
2. **Messages use optimal paths** automatically
3. **Reliability matches native** through redundancy
4. **User experience is identical** to Android

The key innovation is treating the CLI not just as a relay, but as a **transport capability extender** for the browser-based WASM client.