# P0_NETWORK_001: Bootstrap Relay Fallback Implementation

**Priority:** P0 CRITICAL  
**Platform:** Android (Cross-platform impact)
**Status:** Open
**Routing Tags:** [REQUIRES: NETWORK_SYNC] [REQUIRES: FINALIZATION]

## Objective
Implement comprehensive fallback strategy for bootstrap relay connectivity failures. All 4 relay servers are failing with "Network error", preventing ANY mesh network connectivity and making cross-device messaging impossible.

## Root Cause
From ANDROID_PIXEL_6A_AUDIT_2026-04-17:
- All 4 bootstrap relay nodes failing: GCP (34.135.34.73) and Cloudflare (104.28.216.43)
- Both QUIC/UDP and TCP endpoints failing with "Network error"
- Likely cellular network blocking non-standard ports (9001, 9010)
- No peer connectivity established (0 peers in mesh stats)
- Complete network isolation

## Implementation Plan

### 1. WebSocket Fallback Transport
**File:** `core/src/transport/websocket.rs` (NEW)
```rust
pub struct WebSocketTransport {
    // WebSocket client implementation for fallback connectivity
    // Supports standard ports 80/443 for cellular network compatibility
}

impl Transport for WebSocketTransport {
    fn dial(&mut self, multiaddr: Multiaddr) -> Result<Connection, TransportError> {
        // Convert libp2p multiaddr to WebSocket URL
        // Implement fallback to WS/WSS when QUIC/TCP fail
    }
}
```

### 2. Protocol Fallback Chain
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
private fun bootstrapWithFallbackStrategy() {
    val bootstrapNodes = getBootstrapNodes()
    
    // Priority: QUIC → TCP → WebSocket → mDNS
    for (node in bootstrapNodes) {
        try {
            when {
                node.isQuic() -> attemptQuicConnection(node)
                node.isTcp() -> attemptTcpConnection(node)  
                node.isWebSocket() -> attemptWebSocketConnection(node)
            }
        } catch (e: NetworkException) {
            Timber.w("Bootstrap failed for ${node.address}: ${e.message}")
            continue
        }
    }
    
    // Final fallback: mDNS local discovery
    if (getConnectedPeers().isEmpty()) {
        startMdnsDiscovery()
    }
}
```

### 3. Cellular Network Detection
**File:** `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt` (NEW)
```kotlin
class NetworkDetector @Inject constructor(
    private val connectivityManager: ConnectivityManager
) {
    fun isCellularNetwork(): Boolean {
        return connectivityManager.activeNetwork?.let { network ->
            connectivityManager.getNetworkCapabilities(network)?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)
        } ?: false
    }
    
    fun getBlockedPorts(): Set<Int> {
        // Detect commonly blocked ports on cellular networks
        return setOf(9001, 9010, 4001, 5001) // Non-standard libp2p ports
    }
}
```

### 4. Circuit Breaker Pattern
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
private val relayCircuitBreakers = mutableMapOf<String, CircuitBreaker>()

private fun attemptRelayConnectionWithCircuitBreaker(relayNode: BootstrapNode) {
    val breaker = relayCircuitBreakers.getOrPut(relayNode.id) { 
        CircuitBreaker(
            failureThreshold = 3,
            resetTimeout = Duration.ofMinutes(5),
            halfOpenTimeout = Duration.ofSeconds(30)
        )
    }
    
    if (breaker.allowRequest()) {
        try {
            connectToRelay(relayNode)
            breaker.onSuccess()
        } catch (e: Exception) {
            breaker.onFailure()
            throw e
        }
    } else {
        Timber.d("Circuit breaker open for relay ${relayNode.id}, skipping attempt")
    }
}
```

### 5. Detailed Error Diagnostics
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
```kotlin
private fun logBootstrapFailureDetails(node: BootstrapNode, exception: Exception) {
    val errorDetails = when (exception) {
        is UnknownHostException -> "DNS resolution failed"
        is ConnectException -> "Connection refused"
        is SocketTimeoutException -> "Connection timeout"
        is SSLException -> "TLS handshake failed"
        else -> "Network error: ${exception.message}"
    }
    
    Timber.w("Bootstrap failed for ${node.address} - $errorDetails")
    
    // Track failure metrics for adaptive routing
    trackRelayFailure(node.id, errorDetails)
}
```

## Files to Modify/Create
1. `core/src/transport/websocket.rs` (NEW) - WebSocket transport implementation
2. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` - Fallback strategy
3. `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt` (NEW) - Network detection
4. `core/src/transport/circuit_breaker.rs` - Circuit breaker pattern
5. `android/app/src/main/java/com/scmessenger/android/utils/CircuitBreaker.kt` - Android circuit breaker

## Alternative Bootstrap Sources
1. **Environment override**: `SC_BOOTSTRAP_NODES` environment variable
2. **Remote config**: HTTP endpoint for dynamic bootstrap node updates  
3. **mDNS fallback**: Local network discovery when internet relays fail
4. **Peer exchange**: Get bootstrap nodes from connected peers

## Test Plan
1. **Cellular Simulation**: Block QUIC/TCP ports 9001/9010
2. **Fallback Verification**: Test WebSocket fallback activates
3. **Circuit Breaker**: Verify failure threshold and recovery
4. **Network Detection**: Test cellular vs WiFi protocol selection
5. **End-to-end**: Verify message delivery through fallback paths

## Success Criteria
- ✅ Bootstrap connectivity established via fallback protocols
- ✅ Circuit breaker prevents hammering failed relays
- ✅ Detailed error diagnostics logged
- ✅ Cellular network detection working
- ✅ Message delivery through fallback paths

## Priority: URGENT
This issue prevents ALL network connectivity. Without relay access, devices cannot communicate across networks.

**Estimated LOC:** ~300-400 LOC across 5 files
**Time Estimate:** 3-4 hours implementation + 2 hours testing