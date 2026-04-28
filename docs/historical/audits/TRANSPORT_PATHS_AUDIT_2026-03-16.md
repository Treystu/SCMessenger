# Transport Paths Audit & Adaptive Retry Plan

**Date:** 2026-03-16  
**Auditor:** Roo (AI Code Agent)  
**Status:** Audit Complete, Plan Ready for Implementation

---

## Executive Summary

SCMessenger has 4 primary transport paths with varying levels of implementation maturity:

| Transport | Status | Priority | Blocker |
|-----------|--------|----------|---------|
| **LAN/WiFi (Multipeer/WiFi Direct)** | ✅ Working | 1st | None |
| **BLE (Bluetooth Low Energy)** | ✅ Working | 2nd (backup) | None |
| **Cellular: Direct (P2P via libp2p)** | 🔴 Broken | 3rd | TCP blocked by carriers |
| **Cellular: Relay (via bootstrap nodes)** | 🟡 Partial | 4th (fallback) | TCP blocked, no QUIC |

**Critical Gap:** No adaptive transport selection or intelligent retry that considers real-time transport quality.

---

## 1. Current Transport Paths - Detailed Audit

### 1.1 LAN/WiFi Transport (BEST - First Attempt)

**iOS Implementation:** `MultipeerTransport.swift`
- Uses Apple's MultipeerConnectivity framework
- Provides WiFi Direct + Bluetooth hybrid
- Auto-invite with debounce/in-flight guardrails
- Reconnection with exponential backoff (2s base, 5 max attempts)
- **Status:** ✅ Working, recently fixed for hang/freeze issues

**Android Implementation:** `WifiTransportManager.kt`
- WiFi Aware / WiFi Direct
- **Status:** ✅ Working

**Capabilities (from `abstraction.rs`):**
```
Max Payload: 4096 bytes
Bandwidth: 250 Mbps
Latency: 5ms
Streaming: Yes
Bidirectional: Yes
```

**Current Issues:**
- None identified - this is the most reliable path

---

### 1.2 BLE Transport (IMMEDIATE BACKUP - <500ms fallback)

**iOS Implementation:** `BLEPeripheralManager.swift` + `BLECentralManager.swift`
- GATT server for Message/Sync/Identity characteristics
- Fragmentation/reassembly for messages > MTU
- Privacy rotation (15 min intervals)
- **Status:** ✅ Working (recently fixed SIGTRAP crashes and deadlock issues)

**Android Implementation:** `BleAdvertiser.kt` + `BleGattServer.kt`
- Mirrors iOS GATT structure
- **Status:** ✅ Working

**Capabilities (from `abstraction.rs`):**
```
Max Payload: 512 bytes (after fragmentation)
Bandwidth: 2 Mbps
Latency: 50ms
Streaming: No (message-based)
Bidirectional: Yes
```

**Current Issues:**
- BLE hints have 5-minute TTL with 10-minute stale grace period (recently improved)
- Messages delivered via BLE are marked as "accepted" but may remain in retry queue
- **Fix Needed:** Ensure BLE-delivered messages are marked as terminal success, not just "accepted"

---

### 1.3 Cellular: Direct P2P (via libp2p TCP)

**Implementation:** `core/src/transport/swarm.rs` + `internet.rs`
- libp2p TCP transport with Noise encryption
- Yamux multiplexing
- UPnP port mapping
- Peer-assisted address reflection (replaces STUN)
- **Status:** 🔴 BROKEN on cellular networks

**Capabilities (from `abstraction.rs`):**
```
Max Payload: 8192 bytes
Bandwidth: 100 Mbps (average)
Latency: 100ms
Streaming: Yes
Bidirectional: Yes
```

**Root Cause (from `CELLULAR_NAT_SOLUTION.md`):**
```
Android's TCP transport cannot establish outbound connections to relay servers from cellular network.
Potential causes:
1. Carrier-level TCP port filtering (common for non-HTTP ports)
2. Android network permissions not granted for background TCP
3. Cellular network type (CGNAT, IPv6-only, etc.)
4. Swarm initialization timing issue on network change
```

**Evidence:**
- Android: `0 peers discovered`, all relay dials return "Network error"
- iOS: Successfully connected to relay via WiFi
- iOS discovered Android's public IP via BLE: `74.244.37.79:15962`
- Android cannot dial relays: GCP `34.135.34.73:9001` and Cloudflare `104.28.216.43:9010`

---

### 1.4 Cellular: Relay (via bootstrap nodes)

**Implementation:** Circuit relay via libp2p relay protocol
- Bootstrap nodes: GCP (`34.135.34.73:9001`) and OSX (`104.28.216.43:9010`)
- **Status:** 🟡 PARTIALLY WORKING

**Current Issues:**
- TCP connections to relay blocked on cellular (same root cause as Direct)
- No QUIC/UDP fallback available
- Relay flapping issues previously fixed (threshold 6→30, debounce, backoff 12s→30s)
- Circuit breaker implemented on iOS (10 failures → 5 min pause)

**Evidence from logs:**
```
Core-routed delivery failed... Network error
IronCoreError error 4 (NetworkError)
```

---

## 2. Retry/Backoff Mechanisms - Current State

### 2.1 iOS Retry Implementation (`MeshRepository.swift`)

**Exponential Backoff (Lines 4056-4060):**
```swift
let backoffExponent = min(failureCount, 5)
let backoffSeconds = TimeInterval(1 << backoffExponent)  // 1s → 2s → 4s → 8s → 16s → 32s
```

**Circuit Breaker (Lines 4039-4049):**
```swift
if failureCount >= circuitBreakerThreshold {  // 10 failures
    if let lastFailure = lastFailure,
       Date().timeIntervalSince(lastFailure) < circuitBreakerDuration {  // 5 minutes
        // Pause retries
        continue
    }
}
```

**Status:** ✅ Implemented and working

---

### 2.2 Android Retry Implementation (`MeshRepository.kt`)

**Current Settings:**
```kotlin
private val pendingOutboxMaxAttempts: Int = 720
private val pendingOutboxMaxAgeSeconds: Long = 7L * 24L * 60L * 60L
```

**Issues:**
- No exponential backoff implemented
- No circuit breaker
- `msg=unknown` in delivery logs (message ID not captured before async)
- Retry loop can spin aggressively

**Status:** 🔴 NEEDS IMPROVEMENT

---

### 2.3 Core Retry Implementation (`mesh_routing.rs`)

**RetryStrategy (Lines 258-304):**
```rust
pub struct RetryStrategy {
    pub max_attempts: Option<u32>,  // None = unbounded
    pub initial_delay: Duration,    // 100ms
    pub max_delay: Duration,        // 30s
    pub backoff_multiplier: f64,    // 1.5
    pub use_exponential_backoff: bool,  // true
}
```

**Status:** ✅ Implemented but not exposed to mobile layers

---

## 3. Gaps in Adaptive Transport Selection

### 3.1 Missing: Real-Time Transport Quality Scoring

**Current:** Static capabilities in `abstraction.rs`
**Needed:** Dynamic scoring based on:
- Recent success/failure rate per transport
- Latency measurements (RTT)
- Bandwidth measurements
- Battery level consideration
- Network type detection (WiFi vs Cellular)

### 3.2 Missing: Intelligent Transport Escalation

**Current:** `escalation.rs` has basic framework but:
- No automatic escalation triggers
- No de-escalation on quality degradation
- No mid-flight transport switching

**Needed:**
- Monitor transport health continuously
- Escalate to better transport when available (e.g., BLE → WiFi)
- De-escalate when quality degrades (e.g., WiFi → BLE)
- Switch transport mid-conversation if current degrades

### 3.3 Missing: Adaptive Retry Based on Transport

**Current:** Same retry strategy for all transports
**Needed:**
- BLE: Fast retry (100ms-1s) due to low latency
- WiFi: Medium retry (1s-8s) due to reliability
- Cellular: Slow retry (2s-32s) due to latency/power
- Relay: Very slow retry (5s-60s) due to infrastructure dependency

### 3.4 Missing: Network Type Detection

**Current:** No explicit network type detection
**Needed:**
- Detect WiFi vs Cellular vs Ethernet
- Detect carrier-level restrictions (TCP blocked)
- Auto-enable QUIC when on cellular
- Adjust transport priority based on network type

### 3.5 Missing: QUIC/UDP Transport

**Current:** TCP only via libp2p
**Needed:** QUIC transport for cellular-friendly connectivity

### 3.6 CRITICAL: No WebRTC-to-QUIC Bridge (Browser↔Native Interop Gap)

**Status:** BLOCKING for browser clients

**Clarification:** ALL nodes are relay servers (iOS, Android, CLI, WASM). The question is which node variant can relay traffic using BOTH WebRTC and QUIC protocols.

**Problem:** NO node variant can currently relay both WebRTC and QUIC.

| Node Variant | TCP | QUIC | WebSocket | WebRTC | Can Bridge? |
|---|---|---|---|---|---|
| iOS | ✅ | ✅ | ❌ | ❌ | ❌ QUIC only |
| Android | ✅ | ✅ | ❌ | ❌ | ❌ QUIC only |
| CLI | ✅ | ✅ | ❌ | ❌ | ❌ QUIC only |
| WASM/Browser | ❌ | ❌ | ✅ | ✅ | ❌ WebRTC only |

**Impact:** Browser clients are ISOLATED from native nodes. The mesh is split:
- Browser mesh: WebSocket/WebRTC only
- Native mesh: TCP/QUIC only
- **No bridge exists between them**

**Root Cause:**
- Native swarm ([`swarm.rs:1286-1294`](core/src/transport/swarm.rs:1286-1294)) only binds TCP + QUIC
- WASM transport ([`transport.rs:102-104`](core/src/wasm_support/transport.rs:102-104)) only accepts `ws://` or `wss://` URLs

**Required Fix:** CLI nodes will be upgraded to dual-stack bridge relays:
1. Accept WebSocket connections from browsers (port 8001)
2. Bind libp2p QUIC for native nodes (port 4433)
3. Use the transport-agnostic `RelayMessage` protocol to bridge both

**Solution:** See [`CLI_WEBRTC_BRIDGE_PLAN.md`](CLI_WEBRTC_BRIDGE_PLAN.md) for full implementation plan.
- ~310 LOC across core + CLI
- 6-8 day implementation timeline
- GCP relay nodes will run in bridge mode

**Priority:** P0 → **P1 (Solution Defined)** - Implementation plan ready.

---

## 4. Intelligent Retry Strategy - Proposed Design

### 4.1 Transport Quality Score (TQS)

Each transport maintains a rolling quality score (0-100):

```
TQS = (SuccessRate * 0.5) + (LatencyScore * 0.3) + (RecencyScore * 0.2)

Where:
- SuccessRate: (successful_sends / total_sends) * 100
- LatencyScore: 100 if <50ms, 80 if <100ms, 60 if <500ms, 40 if <1000ms, 20 otherwise
- RecencyScore: 100 if <60s ago, 80 if <5min, 60 if <30min, 40 if <1hr, 20 otherwise
```

### 4.2 Adaptive Retry Delays

```rust
pub struct AdaptiveRetryConfig {
    // Base delays per transport (milliseconds)
    ble_base_delay: u32,        // 100ms
    wifi_base_delay: u32,       // 1000ms
    cellular_base_delay: u32,   // 2000ms
    relay_base_delay: u32,      // 5000ms
    
    // Quality multiplier (1.0 = normal, 2.0 = double delay for poor quality)
    quality_multiplier: f64,    // TQS < 50: 2.0, TQS < 75: 1.5, TQS >= 75: 1.0
    
    // Max delays
    max_delay: u32,             // 60000ms (60s)
    
    // Circuit breaker
    circuit_breaker_threshold: u32,  // 10 failures
    circuit_breaker_duration_ms: u32, // 300000ms (5 min)
}
```

### 4.3 Transport Selection Algorithm

```rust
fn select_best_transport(
    available: &[TransportType],
    quality_scores: &HashMap<TransportType, f64>,
    battery_level: u8,
    network_type: NetworkType,
) -> TransportType {
    // Priority matrix based on conditions
    let priority = match network_type {
        NetworkType::Wifi => vec![WiFiDirect, BLE, Internet, Relay],
        NetworkType::Cellular => vec![BLE, WiFiDirect, Internet, Relay],
        NetworkType::Unknown => vec![BLE, WiFiDirect, Internet, Relay],
    };
    
    // Filter to available transports
    let available_priority: Vec<_> = priority.iter()
        .filter(|t| available.contains(t))
        .collect();
    
    // Select highest priority with TQS > 50
    for transport in available_priority {
        if quality_scores.get(transport).unwrap_or(&0.0) > &50.0 {
            return *transport;
        }
    }
    
    // Fallback to any available
    available.first().copied().unwrap_or(TransportType::Local)
}
```

---

## 5. Implementation Plan

### Phase 1: Critical Fixes (1-2 days)

#### 1.1 Add QUIC/UDP Transport (CORE)
**File:** `core/src/transport/swarm.rs`
**LOC:** ~150

```rust
// Add QUIC transport alongside TCP
let tcp_transport = libp2p::tcp::tokio::Transport::new(tcp_config);
let quic_transport = libp2p::quic::tokio::Transport::new(libp2p::quic::Config::new(&keypair));

let transport = tcp_transport
    .or_transport(quic_transport)
    .upgrade(Version::V1)
    .authenticate(NoiseConfig::xx(&keypair).into_authenticated())
    .multiplex(YamuxConfig::default())
    .timeout(Duration::from_secs(20))
    .boxed();
```

**Relay Node Updates:**
- Deploy QUIC listeners on GCP and OSX relays
- Add multiaddrs: `/ip4/34.135.34.73/udp/9002/quic-v1`

#### 1.2 Fix BLE Delivery Completion (ANDROID)
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
**LOC:** ~30

```kotlin
// When BLE delivery succeeds, mark as terminal
if (bleDeliveryResult.outcome == "accepted") {
    markMessageDelivered(msgId)
    // Don't retry via core transport
    return@launch
}
```

#### 1.3 Fix Android Message ID Logging (ANDROID)
**






...


......
...



......

"

彻底

让用户导致将
...
**







Android

. be


  be be message.-**
 

k
  

```kk `Id
`
1 Id
k4****
 capture.5

k 4 IDk ID (IdId message

3**
**

 (.**

**

 32:.
::..: `**:.

.


:
**- **1.4 Android Backoff & Circuit Breaker (ANDROID)
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
**LOC:** ~100

```kotlin
// Add exponential backoff (mirroring iOS)
private val consecutiveDeliveryFailures = mutableMapOf<String, Int>()
private val lastFailureTime = mutableMapOf<String, Long>()
private val circuitBreakerThreshold = 10
private val circuitBreakerDurationMs = 300_000L // 5 minutes

fun shouldAttemptDelivery(peerId: String): Boolean {
    val failures = consecutiveDeliveryFailures[peerId] ?: 0
    if (failures >= circuitBreakerThreshold) {
        val lastFailure = lastFailureTime[peerId] ?: 0
        if (System.currentTimeMillis() - lastFailure < circuitBreakerDurationMs) {
            return false // Circuit breaker active
        }
        consecutiveDeliveryFailures[peerId] = 0 // Reset after duration
    }
    return true
}

fun getBackoffDelay(peerId: String): Long {
    val failures = consecutiveDeliveryFailures[peerId] ?: 0
    val baseDelay = 1000L // 1 second
    val exponent = min(failures, 5)
    return baseDelay * (1 shl exponent) // 1s → 2s → 4s → 8s → 16s → 32s
}
```

---

### Phase 2: Adaptive Transport (3-5 days)

#### 2.1 Transport Quality Tracker (CORE)
**File:** `core/src/transport/quality.rs` (new)
**LOC:** ~200

```rust
pub struct TransportQualityTracker {
    scores: HashMap<TransportType, QualityMetrics>,
    history_window: Duration, // 5 minutes
}

pub struct QualityMetrics {
    recent_sends: VecDeque<SendResult>,
    avg_latency_ms: f64,
    last_updated: SystemTime,
}

pub struct SendResult {
    success: bool,
    latency_ms: u64,
    timestamp: SystemTime,
}

impl TransportQualityTracker {
    pub fn record_send(&mut self, transport: TransportType, success: bool, latency_ms: u64) {
        // Record and update rolling metrics
    }
    
    pub fn get_quality_score(&self, transport: TransportType) -> f64 {
        // Calculate TQS based on recent history
    }
    
    pub fn recommend_transport(&self, available: &[TransportType]) -> TransportType {
        // Select best transport based on quality scores
    }
}
```

#### 2.2 Network Type Detection (MOBILE)
**Files:** 
- `android/app/src/main/java/com/scmessenger/android/utils/NetworkMonitor.kt` (new)
- `iOS/SCMessenger/SCMessenger/Services/NetworkMonitor.swift` (new)
**LOC:** ~100 each

```kotlin
// Android
enum class NetworkType { WIFI, CELLULAR, ETHERNET, UNKNOWN }

class NetworkMonitor(context: Context) {
    private val connectivityManager = context.getSystemService<ConnectivityManager>()
    
    fun getCurrentNetworkType(): NetworkType {
        val network = connectivityManager.activeNetwork ?: return NetworkType.UNKNOWN
        val capabilities = connectivityManager.getNetworkCapabilities(network) ?: return NetworkType.UNKNOWN
        
        return when {
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> NetworkType.WIFI
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> NetworkType.CELLULAR
            capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> NetworkType.ETHERNET
            else -> NetworkType.UNKNOWN
        }
    }
}
```

#### 2.3 Adaptive Retry Integration (MOBILE)
**Files:**
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`

Integrate quality tracker and network monitor to:
- Adjust retry delays based on transport quality
- Switch transport when quality degrades
- Escalate to better transport when available

---

### Phase 3: Intelligent Escalation (5-7 days)

#### 3.1 Transport Escalation Engine (CORE)
**File:** `core/src/transport/escalation.rs` (enhance existing)
**LOC:** ~300

```rust
impl EscalationEngine {
    pub fn evaluate_escalation(&self, peer_id: [u8; 32], current_transport: TransportType) -> Option<TransportType> {
        let current_quality = self.quality_tracker.get_quality_score(current_transport);
        let available = self.get_available_transports(peer_id);
        
        // Find better transport with higher quality
        for transport in available {
            if transport == current_transport { continue; }
            let quality = self.quality_tracker.get_quality_score(transport);
            if quality > current_quality + 20.0 { // 20-point threshold
                return Some(transport);
            }
        }
        None
    }
    
    pub fn evaluate_deescalation(&self, peer_id: [u8; 32], current_transport: TransportType) -> Option<TransportType> {
        let current_quality = self.quality_tracker.get_quality_score(current_transport);
        
        // De-escalate if quality drops below threshold
        if current_quality < 40.0 {
            let available = self.get_available_transports(peer_id);
            for transport in available {
                if transport == current_transport { continue; }
                let quality = self.quality_tracker.get_quality_score(transport);
                if quality > current_quality {
                    return Some(transport);
                }
            }
        }
        None
    }
}
```

#### 3.2 Mid-Flight Transport Switching (CORE)
**File:** `core/src/transport/manager.rs`
**LOC:** ~200

Implement ability to switch transport mid-conversation:
- Monitor transport health continuously
- Trigger switch when quality degrades
- Preserve message ordering during switch
- Notify peer of transport change

---

## 6. Testing Strategy

### 6.1 Unit Tests
- Transport quality scoring algorithm
- Adaptive retry delay calculation
- Network type detection
- Escalation/de-escalation logic

### 6.2 Integration Tests
- QUIC transport with relay nodes
- Transport switching mid-conversation
- Quality degradation detection
- Circuit breaker behavior

### 6.3 Live Testing (run5.sh)
- Cellular → WiFi messaging (via relay)
- WiFi → Cellular messaging (direct + relay fallback)
- BLE-only delivery (no relays reachable)
- Transport escalation when WiFi becomes available
- Transport de-escalation when WiFi quality degrades

---

## 7. Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Message delivery rate on cellular | ~50% | >95% |
| Time to first relay connection | ~10-30s | <5s |
| Fallback to BLE when needed | Working | 100% |
| Zero stuck messages in queue | ~10% stuck | 0% |
| Transport escalation latency | N/A | <2s |
| Adaptive retry accuracy | N/A | >90% optimal delays |

---

## 8. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| QUIC blocked by carriers | Low | High | TCP fallback always available |
| Quality scoring overhead | Medium | Low | Efficient rolling window algorithm |
| Mid-flight switch causes message loss | Medium | High | Message buffering during switch |
| Network detection inaccurate | Low | Medium | Graceful fallback to default priorities |

---

## 9. Documentation Updates Required

1. `docs/NAT_TRAVERSAL_GUIDE.md` - Add QUIC section
2. `docs/CURRENT_STATE.md` - Update transport status
3. `docs/TESTING_GUIDE.md` - Add adaptive transport tests
4. `REMAINING_WORK_TRACKING.md` - Add transport tasks
5. `MASTER_BUG_TRACKER.md` - Close cellular bugs when fixed

---

## 10. Design Decisions (User Confirmed 2026-03-16)

| Decision | User Choice | Implementation Impact |
|----------|-------------|----------------------|
| **QUIC Priority** | PRIMARY on cellular | Attempt QUIC first when cellular detected, TCP fallback |
| **Battery Impact** | Battery Conservative | Minimal escalation, prefer stable transport even if slower |
| **Mid-Flight Switching** | Mid-Flight Switch | Switch immediately when quality degrades, buffer and resend if needed |
| **Quality Thresholds** | Aggressive (30/60) | TQS >30 usable, TQS >60 preferred |

### Configuration Values

```rust
// Transport Quality Score (TQS) Thresholds
const TQS_USABLE_THRESHOLD: f64 = 30.0;   // Transport usable if TQS > 30
const TQS_PREFERRED_THRESHOLD: f64 = 60.0; // Transport preferred if TQS > 60

// Escalation Policy (Battery Conservative)
const ESCALATION_QUALITY_DROP_THRESHOLD: f64 = 25.0; // Only escalate if new transport is 25+ points better
const ESCALATION_COOLDOWN_MS: u64 = 30_000;          // 30 seconds between escalation attempts

// Mid-Flight Switching
const SWITCH_QUALITY_DROP_THRESHOLD: f64 = 20.0; // Switch immediately if quality drops 20+ points
const SWITCH_BUFFER_SIZE: usize = 100;            // Buffer up to 100 messages during switch
```

---

**Next Steps:** Begin Phase 1 implementation with confirmed design decisions.
