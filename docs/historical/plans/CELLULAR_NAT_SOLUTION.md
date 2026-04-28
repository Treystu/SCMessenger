# Cellular NAT Traversal Solution
**Status**: Implementation Required
**Priority**: P0 - Blocking Message Delivery
**Date**: 2026-03-09

## Problem Statement

Android device on cellular network cannot send messages to iOS device despite both apps running.

### Evidence
- **Android**: `0 peers discovered`, all relay dials return "Network error"
- **iOS**: Successfully connected to relay, receiving peer discovery via BLE
- **iOS discovered Android's public IP**: `74.244.37.79:15962` (via BLE identity exchange)
- **Android cannot dial relays**: GCP `34.135.34.73:9001` and Cloudflare `104.28.216.43:9010`

### Root Cause
**Android's TCP transport cannot establish outbound connections to relay servers from cellular network.**

Potential causes:
1. Carrier-level TCP port filtering (common for non-HTTP ports)
2. Android network permissions not granted for background TCP
3. Cellular network type (CGNAT, IPv6-only, etc.)
4. Swarm initialization timing issue on network change

## Immediate Workaround

### Option 1: Use WiFi (Fastest - No Code)
Connect Android to WiFi to bypass cellular restrictions.

### Option 2: Enable BLE Direct Delivery (Partial - Already Implemented)
Messages show BLE is working:
```
iOS: delivery_attempt msg=unknown medium=ble phase=local_fallback outcome=accepted
```

But BLE delivery is marked as "fallback" and messages remain stuck in peer queue.

**Action**: Ensure BLE-delivered messages are marked as successfully sent, not just "accepted".

### Option 3: Force Relay via iOS as Bridge
Since iOS can reach both:
- Android via BLE
- Relay via WiFi/cellular

iOS could act as a bridge to forward Android's messages to the relay.

## Short-Term Fix (This Session)

### 1. Add UDP/QUIC Transport Fallback
Many carriers block TCP but allow UDP. Add QUIC transport as primary:

```rust
// core/src/transport/swarm.rs
let transport = libp2p::dns::tokio::Transport::system(
    libp2p::quic::tokio::Transport::new(libp2p::quic::Config::new(&keypair))
)
.or_transport(libp2p::tcp::tokio::Transport::new(/* ... */))
```

Update relay nodes to listen on UDP as well:
```
/ip4/34.135.34.73/udp/9002/quic-v1
/ip4/104.28.216.43/udp/9011/quic-v1
```

### 2. Aggressive Relay Bootstrap Retry
Current code gives up after first "Network error". Implement exponential backoff with persistent retries:

```kotlin
// MeshRepository.kt
private fun primeRelayBootstrapConnections() {
    DEFAULT_BOOTSTRAP_NODES.forEach { addr ->
        // Try both TCP and UDP versions
        listOf(addr, addr.replace("/tcp/", "/udp/").replace("9001", "9002"))
            .forEach { variant ->
                if (shouldAttemptDial(variant)) {
                    try {
                        bridge.dial(variant)
                    } catch (e: Exception) {
                        // Don't log as error - this is expected on cellular
                        Timber.v("Relay dial attempt $variant: ${e.message}")
                    }
                }
            }
    }
}
```

### 3. Network Connectivity Check
Add explicit connectivity test before assuming "Network error" is fatal:

```kotlin
private fun testRelayConnectivity(): Boolean {
    return try {
        val socket = Socket()
        socket.connect(InetSocketAddress("34.135.34.73", 9001), 5000)
        socket.close()
        true
    } catch (e: Exception) {
        false
    }
}
```

### 4. BLE Message Completion Fix
Ensure BLE-delivered messages are marked as delivered, not stuck in retry:

```kotlin
// When BLE delivery succeeds
if (bleDeliveryResult.outcome == "accepted") {
    markMessageDelivered(msgId) // Don't retry via core
}
```

## Long-Term Solution (Post-Release)

### 1. Implement libp2p Hole-Punching (DCUtR)
Direct Connection Upgrade through Relay - allows direct connection after relay introduces peers.

### 2. AutoNAT Service
Detect NAT type and adjust strategy:
- Full Cone: Direct dial works
- Symmetric: Relay required
- Port-Restricted: Hole-punching may work

### 3. TURN Relay Fallback
When all else fails, use TURN server (similar to WebRTC) for guaranteed delivery.

### 4. Multi-Transport Priority
```
Priority 1: Direct IP (LAN)
Priority 2: QUIC/UDP (cellular-friendly)
Priority 3: TCP (WiFi/enterprise)
Priority 4: Relay circuit
Priority 5: BLE (local only)
```

## Implementation Plan

### Phase 1: Emergency Fix (Today)
1. ✅ Diagnose issue (DONE)
2. Add QUIC transport to core
3. Deploy UDP relay listeners
4. Fix BLE delivery completion
5. Test cellular→WiFi messaging

### Phase 2: Robust Solution (v0.2.1)
1. Implement AutoNAT detection
2. Add hole-punching (DCUtR)
3. Multi-transport fallback logic
4. Carrier compatibility testing

### Phase 3: Production Hardening (v0.3.0)
1. TURN relay infrastructure
2. Geographic relay distribution
3. Bandwidth-adaptive transport selection
4. Offline queue with smart retry

## Testing Checklist

- [ ] Android cellular → iOS WiFi (via relay)
- [ ] Android cellular → iOS cellular (via relay)
- [ ] Android WiFi → iOS cellular (direct + relay fallback)
- [ ] Android offline → online delivery
- [ ] iOS as relay bridge for Android
- [ ] BLE-only delivery (no relays reachable)

## Dependencies

- libp2p-quic (already in Cargo.toml)
- Relay infrastructure with UDP listeners
- Network permission: `INTERNET` (already granted)

## Success Metrics

- Message delivery rate on cellular: >95%
- Time to first relay connection: <5 seconds
- Fallback to BLE when needed: 100%
- Zero stuck messages in queue

---

**Next Steps**: Implement QUIC transport and deploy to test devices.
