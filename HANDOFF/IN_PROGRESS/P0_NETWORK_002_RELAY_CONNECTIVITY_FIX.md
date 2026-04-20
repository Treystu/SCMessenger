# P0_NETWORK_002: Relay Connectivity Fix & Fallback Implementation

**Priority:** P0 CRITICAL
**Platform:** Core (Rust) + All Platforms
**Status:** Open
**Source:** Cargo.toml dependency addition + REMAINING_WORK_TRACKING.md ANR-002
**Routing Tags:** [REQUIRES: NETWORK_SYNC] [REQUIRES: FINALIZATION]
**[NATIVE_SUB_AGENT: RESEARCH]** [NATIVE_SUB_AGENT: SECURITY_AUDIT]

## Objective
Fix relay server connectivity failures and implement proper fallback mechanisms. From ANDROID_PIXEL_6A_AUDIT_2026-04-17, all 4 relay servers are failing with generic "Network error" messages.

## Evidence
- **Cargo.toml change**: Added `tokio-tungstenite` WebSocket client dependency
- **REMAINING_WORK_TRACKING.md**: ANR-002 - "Network Bootstrap Complete Failure"
- **Relay servers failing**: `34.135.34.73`, `104.28.216.43` (both UDP/TCP)

## Implementation Plan

### 1. Enhanced Error Diagnostics
**File:** `core/src/transport/bootstrap.rs`
```rust
fn diagnose_connection_error(error: io::Error, relay: &BootstrapNode) -> RelayError {
    match error.kind() {
        io::ErrorKind::ConnectionRefused => RelayError::ConnectionRefused(relay.address.clone()),
        io::ErrorKind::TimedOut => RelayError::Timeout(relay.address.clone()),
        io::ErrorKind::AddrNotAvailable => RelayError::DnsResolutionFailed(relay.hostname.clone()),
        _ => RelayError::Generic(error.to_string())
    }
}
```

### 2. WebSocket Fallback Transport
**File:** `core/src/transport/websocket.rs` (new)
```rust
pub struct WebSocketTransport {
    inner: Option<TungsteniteStream>,
    url: String,
}

impl Transport for WebSocketTransport {
    fn connect(&mut self) -> Result<(), TransportError> {
        // Implement WebSocket connection with TLS
        // Fallback from UDP/TCP when primary fails
    }
}
```

### 3. Multi-Transport Bootstrap
**File:** `core/src/transport/mod.rs`
```rust
pub async fn bootstrap_with_fallback(nodes: Vec<BootstrapNode>) -> Result<(), BootstrapError> {
    for node in nodes {
        // Try UDP first
        match UdpTransport::connect(&node).await {
            Ok(transport) => return Ok(transport),
            Err(_) => {
                // Fallback to TCP
                match TcpTransport::connect(&node).await {
                    Ok(transport) => return Ok(transport),
                    Err(_) => {
                        // Final fallback to WebSocket
                        match WebSocketTransport::connect(&node).await {
                            Ok(transport) => return Ok(transport),
                            Err(e) => log::warn!("All transports failed for {}: {:?}", node.address, e),
                        }
                    }
                }
            }
        }
    }
    Err(BootstrapError::AllTransportsFailed)
}
```

### 4. Alternative Bootstrap Sources
**File:** `core/src/transport/bootstrap.rs`
- Implement DNS-based bootstrap node discovery
- Add hardcoded backup relay addresses
- Create community-sourced relay list mechanism

## Success Criteria
- ✅ Detailed error diagnostics for all connection failures
- ✅ WebSocket fallback transport implemented and tested
- ✅ Multi-transport bootstrap with proper fallback order
- ✅ Alternative bootstrap source discovery
- ✅ Cross-platform compatibility verified

## Priority: CRITICAL
Required for network reliability and production readiness. Current relay failures make the mesh network completely unusable.