> **Component Status Notice (2026-03-09)**
> This document has been updated to reflect the full completion of Phase 7 (UPnP) and enhanced advertisement consolidation.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# SCMessenger Transport Architecture

> Design and implementation narrative. For current verified runtime/test status, use `docs/CURRENT_STATE.md`.

Complete implementation of sovereign mesh networking with zero external dependencies.

## [Current] Overview

The SCMessenger transport layer implements a fully decentralized, self-healing peer-to-peer network where every node contributes to routing, discovery, and reliability. There are no special "bootstrap nodes" or "relay servers"—all nodes are equal participants that help each other based on their capability.

## [Current] Seven-Phase Implementation

### [Current] Phase 1: Real Address Observation ✅

**Problem**: Nodes need to know their external IP addresses to advertise connectivity, but cannot rely on external STUN servers.

**Solution**: Peer-observed address discovery with consensus.

**Components**:

- `AddressObserver`: Aggregates observations from multiple peers
- `ConnectionTracker`: Monitors active connections and endpoints
- Consensus algorithm: Most-observed address wins

**How it works**:

1. When peers connect, each observes the other's remote address
2. Address reflection protocol exchanges observations
3. AddressObserver counts observations per address
4. Consensus calculated: most confirmations = primary external address
5. **Auto-Advertisement**: Verified consensus addresses are automatically added to the swarm via `swarm.add_external_address()`.

**Result**: Nodes discover and advertise their actual public IPs without external STUN servers.

**Code**: `core/src/transport/observation.rs`

---

### [Current] Phase 2: Multi-Port Adaptive Listening ✅

**Problem**: Restrictive networks may block common P2P ports.

**Solution**: Listen on multiple ports simultaneously (443, 80, 0).

**Result**: Nodes maximize connectivity by trying multiple ports, with automatic fallback.

**Code**: `core/src/transport/multiport.rs`

---

### [Current] Phase 3: Relay Capability ✅

**Problem**: Direct P2P may fail due to NAT/firewall restrictions.

**Solution**: Every node can relay messages for others via `/p2p-circuit/`.

**Result**: Messages reach peers even behind restrictive NATs via community relays.

**Code**: `core/src/transport/mesh_routing.rs` (RelayStats)

---

### [Current] Phase 4: Mesh-Based Discovery ✅

**Problem**: Traditional P2P requires hardcoded "bootstrap nodes".

**Solution**: Any node can help others bootstrap via Kademlia DHT.

**Result**: Network entry possible via any peer; no single point of failure.

---

### [Current] Phase 5: Reputation Tracking ✅

**Problem**: Unreliable relays could degrade performance.

**Solution**: Track relay performance and prioritize reliable nodes.

**Result**: Network self-optimizes by routing through best-performing relays.

---

### [Current] Phase 6: Continuous Retry Logic ✅

**Problem**: Messages must be delivered reliably even when best paths fail.

**Solution**: Multi-path delivery with continuous retry and exponential backoff.

**Result**: Maximum delivery reliability; messages eventually reach destination.

---

### [Current] Phase 7: Automatic Port Mapping (UPnP) ✅

**Problem**: Manual port forwarding is complex for average users.

**Solution**: Integrated UPnP mapping.

**Components**:

- `libp2p::upnp`: Automated gateway discovery and port mapping.
- **Auto-Advertisement**: Mapped addresses are immediately added to the swarm for global visibility.

**Result**: Zero-configuration P2P connectivity for users with UPnP-compatible routers.

**Code**: `core/src/transport/swarm.rs`, `core/src/behaviour.rs`

---

## [Current] Complete Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    SCMessenger Node                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Address    │  │  Multi-Port  │  │  UPnP Mapping    │  │
│  │  Observer   │  │  Listener    │  │  (Phase 7)       │  │
│  │  (Phase 1)  │  │  (Phase 2)   │  └──────────────────┘  │
│  └─────────────┘  └──────────────┘           │            │
│         │                  │                 │            │
│         └──────────────────┴─────────────────┘            │
│                          │                                  │
│  ┌──────────────────────▼───────────────────────────────┐  │
│  │            libp2p Swarm (TCP, QUIC)                  │  │
│  │  - Kademlia DHT (peer discovery)                     │  │
│  │  - mDNS (local discovery)                            │  │
│  │  - UPnP / DCUtR (hole-punch)                         │  │
│  │  - Address Reflection (Phase 1)                      │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                  │
│  ┌──────────────────────▼───────────────────────────────┐  │
│  │         Mesh Routing System (Phases 3-6)             │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## [Current] Network Characteristics

- ✅ **Self-Sovereign**: Zero external STUN/TURN dependencies.
- ✅ **Auto-Configuring**: UPnP + Address Reflection = zero user setup.
- ✅ **Resilient**: Multi-path retry + Relay fallback.
- ✅ **High Privacy**: Metadata-minimized relaying + E2E encryption.

---

**Last Updated:** March 9, 2026
**Status:** ✅ 100% COMPLETE
