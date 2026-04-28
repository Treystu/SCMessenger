> **Component Status Notice (2026-03-09)**
> This document has been updated to reflect the full integration of Peer-Assisted Discovery, UPnP, and Consensus-Based Advertisement.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# NAT Traversal & Address Discovery Guide

> Technical guide. For current verified test/build status, use `docs/CURRENT_STATE.md`.

## [Current] Overview

SCMessenger implements a **sovereign mesh address discovery protocol** that replaces traditional STUN servers with peer-assisted address reflection and UPnP integration. This maintains the "no external dependencies" principle while enabling robust NAT traversal for P2P messaging.

## [Current] Architecture

SCMessenger uses a layered approach to address discovery:

1.  **UPnP Port Mapping**: Attempts to map external ports on compatible home routers.
2.  **Peer-Assisted Discovery**: Each mesh node acts as an address reflector for peers.
3.  **Address Consolidation**: Aggregates observed addresses from multiple sources (UPnP, Address Reflection, Identify protocol).
4.  **Consensus Advertisement**: Proactively advertises verified external addresses to the mesh for direct connectivity.

### [Current] Key Principle: Peer-Assisted Discovery

Instead of relying on external STUN servers, any node can act as an address reflector:

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

## [Current] UPnP Integration

SCMessenger automatically attempts to map ports using UPnP (Universal Plug and Play) when available.

- **Protocol**: `libp2p::upnp`
- **Events**:
  - `NewExternalAddr(addr)`: Successfully mapped a port; the address is automatically added to the swarm's external addresses.
  - `GatewayNotFound`: No UPnP device detected on the local network.
  - `NonRoutableGateway`: Device detected but it lacks a WAN-routable IP.
  - `ExpiredExternalAddr(addr)`: The mapping has been removed or expired.

## [Current] Address Consolidation & Consensus

To ensure reliability, SCMessenger does not blindly trust every observed address. It uses an `AddressObserver` to build consensus:

1.  **Observations**: Addresses are collected from `AddressReflection` responses and `Identify` protocol observations.
2.  **Consensus**: When an address is observed multiple times (threshold-based), it is promoted to the primary external address.
3.  **Advertisement**: Once verified, the address is added to the swarm via `swarm.add_external_address(maddr)`. This ensures that DHT-based peer discovery retrieves valid direct paths.

## [Current] libp2p Protocol Details

- **Protocol ID**: `/sc/address-reflection/1.0.0`
- **Transport**: Request-Response over libp2p
- **Serialization**: CBOR
- **Timeout**: 10 seconds

## [Current] Usage

### 1. NAT Status & Mapping Diagnostics

Mobile and CLI apps can retrieve the current NAT status:

```rust
// Get "open", "full-cone", "symmetric", etc.
let status = mesh_service.get_nat_status();

// Listen for PortMapping events
// SwarmEvent2::PortMapping("mapped:1.2.3.4:4001")
```

### 2. Multi-Path Selection

The `MultiPathDelivery` system leverages these discovered addresses to rank connections:

- **Direct Preferred**: Matches verified external addresses or local network IPs.
- **Relayed Fallback**: Used when direct paths fail or timeout.

## [Current] Comparison to STUN/TURN

| Feature               | STUN Servers               | SCMessenger Traversal   |
| --------------------- | -------------------------- | ----------------------- |
| External Dependencies | ❌ Required                | ✅ None (Peer-assisted) |
| Privacy               | ⚠️ Centralized observation | ✅ Distributed          |
| UPnP Support          | ❌ Service-dependent       | ✅ Automatic Mapping    |
| Availability          | ⚠️ Can be blocked          | ✅ Mesh resilient       |
| Sovereignty           | ❌ Depends on others       | ✅ Fully sovereign      |

## [Current] Troubleshooting

### UPnP "GatewayNotFound"

- **Cause**: UPnP is disabled on the router or the network is restricted.
- **Impact**: Node will fall back to Peer-Assisted Discovery (Reflection). Direct P2P may still work via hole punching or if the other peer has an open port.

### Symmetric NAT Detected

- **Impact**: Direct hole punching typically fails.
- **Solution**: SCMessenger automatically uses Relay Circuit Fallback via bootstrap or neighbor nodes.

---

**Last Updated:** March 9, 2026
**Status:** ✅ CURRENT
