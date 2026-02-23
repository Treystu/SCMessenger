> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# NAT Traversal & Address Reflection Guide

> Technical guide. For current verified test/build status, use `docs/CURRENT_STATE.md`.

## [Needs Revalidation] Overview

SCMessenger implements a **sovereign mesh address discovery protocol** that replaces traditional STUN servers with peer-assisted address reflection. This maintains the "no external dependencies" principle while enabling NAT traversal.

## [Needs Revalidation] Architecture

### [Needs Revalidation] Key Principle: Peer-Assisted Discovery

Instead of relying on external STUN servers (like Google's stun.l.google.com), each mesh node can act as an address reflector for other peers:

```
┌─────────────┐              ┌─────────────┐
│   Node A    │──────────────│   Node B    │
│ (Requester) │              │ (Reflector) │
└─────────────┘              └─────────────┘
      │                             │
      │  AddressReflectionRequest   │
      │────────────────────────────>│
      │                             │
      │                             │ Node B observes
      │                             │ A's source address
      │                             │ from connection
      │  AddressReflectionResponse  │
      │<────────────────────────────│
      │  "I see you at X.X.X.X:Y"   │
      │                             │
```

## [Needs Revalidation] libp2p Protocol

### [Needs Revalidation] Protocol Definition

- **Protocol ID**: `/sc/address-reflection/1.0.0`
- **Transport**: Request-Response over libp2p
- **Serialization**: CBOR
- **Timeout**: 10 seconds

### [Needs Revalidation] Message Types

#### [Needs Revalidation] AddressReflectionRequest
```rust
{
    request_id: [u8; 16],  // Unique request identifier
    version: u8,            // Protocol version (currently 1)
}
```

#### [Needs Revalidation] AddressReflectionResponse
```rust
{
    request_id: [u8; 16],      // Matches request ID
    observed_address: String,   // "IP:PORT" observed from connection
    version: u8,                // Protocol version
}
```

## [Needs Revalidation] Usage

### [Needs Revalidation] 1. Basic Address Reflection

Request your observed address from a single peer:

```rust
use scmessenger_core::transport::swarm::{start_swarm, SwarmHandle};
use libp2p::PeerId;

async fn get_my_address(swarm: &SwarmHandle, peer: PeerId) -> Result<String> {
    // Request address reflection
    let observed_addr = swarm.request_address_reflection(peer).await?;
    println!("Peer sees me at: {}", observed_addr);
    Ok(observed_addr)
}
```

### [Needs Revalidation] 2. NAT Type Detection

Detect your NAT type using multiple peer reflectors:

```rust
use scmessenger_core::transport::nat::{PeerAddressDiscovery, NatType};

async fn detect_my_nat(swarm: &SwarmHandle, peers: Vec<String>) -> Result<NatType> {
    let discovery = PeerAddressDiscovery::with_peers(peers, 10);
    let nat_type = discovery.detect_nat_type(swarm).await?;

    match nat_type {
        NatType::Open => println!("✅ No NAT - directly reachable!"),
        NatType::FullCone => println!("✅ Full cone NAT - hole-punching will work"),
        NatType::Symmetric => println!("⚠️  Symmetric NAT - relay required"),
        _ => println!("❓ Unknown NAT configuration"),
    }

    Ok(nat_type)
}
```

### [Needs Revalidation] 3. Complete NAT Traversal

Full NAT probing with external address discovery:

```rust
use scmessenger_core::transport::nat::{NatConfig, NatTraversal};

async fn setup_nat_traversal(swarm: &SwarmHandle, peer_ids: Vec<String>) -> Result<()> {
    // Configure NAT traversal
    let mut config = NatConfig::default();
    config.peer_reflectors = peer_ids;
    config.enable_hole_punch = true;
    config.enable_relay_fallback = true;

    // Create traversal coordinator
    let nat = NatTraversal::new(config)?;

    // Probe NAT type and discover external address
    let nat_type = nat.probe_nat(swarm).await?;
    let external_addr = nat.get_external_address();

    println!("NAT Type: {:?}", nat_type);
    println!("External Address: {:?}", external_addr);

    Ok(())
}
```

## [Needs Revalidation] Testing

### [Needs Revalidation] Unit Tests

Core address reflection logic (mocked, no network):

```bash
cargo test -p scmessenger-core reflection
```

### [Needs Revalidation] Integration Tests

Full protocol tests with live libp2p swarms:

```bash
# Run all NAT integration tests
cargo test --test integration_nat_reflection

# Run specific test
cargo test --test integration_nat_reflection test_two_node_address_reflection

# Run with output
cargo test --test integration_nat_reflection -- --nocapture
```

### [Needs Revalidation] Integration Test Coverage

The test suite verifies:

1. **Two-Node Address Reflection** (`test_two_node_address_reflection`)
   - Sets up 2 libp2p nodes
   - Node 2 requests address reflection from Node 1
   - Verifies response contains valid address

2. **Peer Address Discovery** (`test_peer_address_discovery_with_live_swarm`)
   - Tests PeerAddressDiscovery with live swarm
   - Verifies external address discovery

3. **Multi-Peer NAT Detection** (`test_nat_traversal_with_live_swarms`)
   - Sets up 3 nodes
   - Node 3 queries nodes 1 & 2 for address reflection
   - Detects NAT type from multiple responses

4. **Multiple Requests** (`test_multiple_address_reflections`)
   - Tests handling of multiple sequential requests
   - Verifies service stability under load

5. **Timeout Handling** (`test_address_reflection_timeout`)
   - Tests graceful handling of disconnected peers
   - Verifies timeout behavior

## [Needs Revalidation] Protocol Flow

### [Needs Revalidation] Complete Request-Response Cycle

```
Application Layer
    │
    ├─> SwarmHandle::request_address_reflection(peer_id)
    │
    └─> SwarmCommand::RequestAddressReflection
            │
            ├─> Generate request_id
            ├─> Create AddressReflectionRequest
            ├─> swarm.behaviour_mut().address_reflection.send_request()
            └─> Store reply channel in pending_reflections

            [Network transmission via libp2p]

            ┌─> Peer receives request
            ├─> Extract observed address from connection
            ├─> Create AddressReflectionResponse
            └─> swarm.behaviour_mut().address_reflection.send_response()

            [Network transmission via libp2p]

            ┌─> Original node receives response
            ├─> Match request_id in pending_reflections
            ├─> Send result via reply channel
            └─> Emit SwarmEvent2::AddressReflected

Application Layer receives result
```

## [Needs Revalidation] Performance Characteristics

### [Needs Revalidation] Latency
- **Single reflection**: ~10-50ms (LAN)
- **Single reflection**: ~50-200ms (WAN)
- **NAT detection** (3 peers): ~150-600ms

### [Needs Revalidation] Bandwidth
- **Request**: ~32 bytes
- **Response**: ~50 bytes
- **Total per reflection**: ~82 bytes

### [Needs Revalidation] Scalability
- Each node can handle 1000+ reflection requests/second
- No external infrastructure required
- Scales linearly with mesh size

## [Needs Revalidation] Security Considerations

### [Needs Revalidation] Address Authenticity
- Addresses are observed from actual libp2p connections
- Cannot be spoofed without controlling network path
- Verified at transport layer (Noise encryption)

### [Needs Revalidation] Privacy
- Reflector learns requester's external address
- This is inherent to NAT traversal (STUN has same property)
- Use Tor/VPN for additional privacy if needed

### [Needs Revalidation] Denial of Service
- Rate limiting recommended for production
- Currently no rate limits (trust-based mesh)
- Consider adding in AddressReflectionService

## [Needs Revalidation] Production Deployment

### [Needs Revalidation] Bootstrap Node Configuration

Bootstrap nodes should enable address reflection:

```rust
let reflection_service = AddressReflectionService::new();
reflection_service.enable(); // Enable by default
```

### [Needs Revalidation] Client Configuration

Mobile/browser clients should use bootstrap nodes as reflectors:

```rust
let config = NatConfig {
    peer_reflectors: vec![
        "12D3KooW...bootstrap1".to_string(),
        "12D3KooW...bootstrap2".to_string(),
        "12D3KooW...bootstrap3".to_string(),
    ],
    enable_hole_punch: true,
    enable_relay_fallback: true,
    ..Default::default()
};
```

### [Needs Revalidation] Monitoring

Key metrics to track:

- Reflection requests served
- NAT type distribution
- Hole-punch success rate
- Relay circuit usage

```rust
let service = AddressReflectionService::new();
let stats = service.stats();
println!("Requests served: {}", stats.requests_served);
```

## [Needs Revalidation] Comparison to STUN

| Feature | STUN Servers | Peer-Assisted Reflection |
|---------|--------------|--------------------------|
| External Dependencies | ❌ Required | ✅ None |
| Privacy | ⚠️ Centralized observation | ✅ Distributed |
| Availability | ⚠️ Can be blocked | ✅ Mesh resilient |
| Latency | ✅ ~20-50ms | ✅ ~10-200ms |
| Bandwidth | ✅ Minimal | ✅ Minimal |
| Sovereignty | ❌ Depends on others | ✅ Fully sovereign |

## [Needs Revalidation] Troubleshooting

### [Needs Revalidation] No Reflectors Available

```rust
Error: ProbesFailed("No peer reflectors configured")
```

**Solution**: Ensure you're connected to bootstrap nodes or peers before NAT probing:

```rust
// Connect to bootstrap nodes first
for addr in bootstrap_addrs {
    swarm.dial(addr).await?;
}

// Wait for connections
tokio::time::sleep(Duration::from_secs(2)).await;

// Then probe NAT
nat.probe_nat(&swarm).await?;
```

### [Needs Revalidation] Timeout Errors

```rust
Error: Timeout waiting for peer response
```

**Solution**: Increase timeout or check peer connectivity:

```rust
let discovery = PeerAddressDiscovery::with_peers(
    peers,
    30  // Increase timeout to 30 seconds
);
```

### [Needs Revalidation] Symmetric NAT Detection

When detecting symmetric NAT, hole-punching won't work. Enable relay:

```rust
if nat_type == NatType::Symmetric {
    println!("Using relay fallback for symmetric NAT");
    // Relay circuit will be established automatically
}
```

## [Needs Revalidation] Future Enhancements

1. **Endpoint Observation**: Extract actual remote endpoint from libp2p connection
2. **Rate Limiting**: Add request rate limits to AddressReflectionService
3. **Address Caching**: Cache discovered external address (with TTL)
4. **IPv6 Support**: Full dual-stack support
5. **TURN Integration**: Sovereign relay nodes for symmetric NAT

## [Needs Revalidation] References

- [RFC 5389 - STUN](https://tools.ietf.org/html/rfc5389)
- [RFC 8445 - ICE](https://tools.ietf.org/html/rfc8445)
- [libp2p NAT Traversal](https://github.com/libp2p/specs/tree/master/nat)
- [SCMessenger Architecture](./ARCHITECTURE.md)
