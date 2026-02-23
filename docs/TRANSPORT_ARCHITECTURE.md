> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# SCMessenger Transport Architecture

> Design and implementation narrative. For current verified runtime/test status, use `docs/CURRENT_STATE.md`.

Complete implementation of sovereign mesh networking with zero external dependencies.

## [Needs Revalidation] Overview

The SCMessenger transport layer implements a fully decentralized, self-healing peer-to-peer network where every node contributes to routing, discovery, and reliability. There are no special "bootstrap nodes" or "relay servers"—all nodes are equal participants that help each other based on their capability.

## [Needs Revalidation] Six-Phase Implementation

### [Needs Revalidation] Phase 1: Real Address Observation ✅

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
5. Handles multiple interfaces (home, mobile, VPN) gracefully

**Result**: Nodes discover their actual public IPs without external STUN servers.

**Code**: `core/src/transport/observation.rs` (~280 LoC)

---

### [Needs Revalidation] Phase 2: Multi-Port Adaptive Listening ✅

**Problem**: Restrictive networks (corporate firewalls) may block common P2P ports, limiting connectivity.

**Solution**: Listen on multiple ports simultaneously, prioritizing common ports for firewall traversal.

**Components**:
- `MultiPortConfig`: Configurable port selection strategy
- `BindResult`: Tracks success/failure per port
- `BindAnalysis`: Connectivity assessment
- `ConnectivityStatus`: 5-level quality rating

**Port Strategy**:
```
Priority 1: 443 (HTTPS) - most likely to be allowed
Priority 2: 80 (HTTP) - widely allowed
Priority 3: 8080, 9090 - common alternatives
Priority 4: Random port (0) - OS-assigned fallback
```

**How it works**:
1. Generate list of addresses (IPv4/IPv6 for each port)
2. Attempt `swarm.listen_on()` for each address
3. Collect BindResult (success/failure/reason)
4. Analyze results and categorize connectivity
5. Advertise all successful bindings to DHT
6. Gracefully handle permission denied (ports < 1024 on Unix)

**Result**: Nodes maximize connectivity by trying multiple ports, with automatic fallback.

**Code**: `core/src/transport/multiport.rs` (~370 LoC)

---

### [Needs Revalidation] Phase 3: Relay Capability ✅

**Problem**: Direct peer-to-peer connections may fail due to NAT/firewall restrictions.

**Solution**: Every node can relay messages for others.

**Components**:
- `RelayStats`: Tracks relay performance per peer
- Metrics: messages relayed, bytes transferred, success/failure rates, latency

**Principle**: No dedicated relay servers
- Every desktop/CLI node can relay
- Mobile nodes relay when capable
- Relay contribution based on device capability

**How it works**:
1. Node A wants to send to Node B (behind restrictive NAT)
2. Node A finds Node C (mutual connection with both)
3. Node A → Node C → Node B (2-hop relay)
4. Node C records relay statistics
5. Message delivered end-to-end encrypted (Node C can't read)

**Result**: Messages reach peers even behind restrictive NATs via community relays.

**Code**: `core/src/transport/mesh_routing.rs` (RelayStats)

---

### [Needs Revalidation] Phase 4: Mesh-Based Discovery ✅

**Problem**: Traditional P2P networks require hardcoded "bootstrap nodes" for initial network entry.

**Solution**: Any node can help others bootstrap—no special nodes needed.

**Components**:
- `BootstrapCapability`: Tracks known peers
- Every node advertises itself as a bootstrap candidate
- Stable nodes naturally preferred (more uptime = more known peers)

**How it works**:
1. New node connects to ANY reachable peer (from QR code, friend, or previous session)
2. Queries DHT for peer list
3. Adds discovered peers to known_peers
4. Now can help others bootstrap
5. Stable nodes accumulate more peer knowledge → become natural hubs

**Principle**: Democratic network entry
- No hardcoded bootstrap servers
- Community-run nodes automatically assist new joiners
- Network grows organically
- More stable nodes emerge as natural coordinators (not by design, by usage)

**Result**: Network entry possible via any peer; no single point of failure.

**Code**: `core/src/transport/mesh_routing.rs` (BootstrapCapability)

---

### [Needs Revalidation] Phase 5: Reputation Tracking ✅

**Problem**: Malicious or unreliable relays could degrade network performance.

**Solution**: Track relay performance and reputation; prioritize reliable relays.

**Components**:
- `RelayReputation`: Calculates score (0-100) per relay
- `ReputationTracker`: Manages all relay reputations
- Scoring factors:
  - Success rate (70% weight)
  - Latency (20% weight)
  - Recency (10% weight)

**How it works**:
1. Every relay attempt updates statistics
2. Reputation score recalculated after each use
3. Reliable peers (score ≥ 50) marked as usable
4. `best_relays()` returns top-ranked peers
5. Routing automatically prefers high-reputation relays

**Adaptive behavior**:
- Fast, reliable relays get more traffic
- Slow/dropping relays deprioritized
- Reputation can recover (not permanent blacklisting)
- New peers start at neutral score (50.0)

**Result**: Network self-optimizes by routing through best-performing relays.

**Code**: `core/src/transport/mesh_routing.rs` (RelayReputation, ReputationTracker)

---

### [Needs Revalidation] Phase 6: Continuous Retry Logic ✅

**Problem**: Messages must be delivered reliably even when best paths fail.

**Solution**: Multi-path delivery with continuous retry and exponential backoff.

**Components**:
- `RetryStrategy`: Exponential backoff with configurable limits
- `DeliveryAttempt`: Tracks per-message retry state
- `MultiPathDelivery`: Manages multi-path attempts

**Retry Strategy**:
```
Attempt 0: 100ms delay (try direct)
Attempt 1: 150ms delay (try best relay)
Attempt 2: 225ms delay (try next relay)
Attempt 3: 337ms delay (try another relay)
...
Max delay: 30 seconds
Max attempts: 10 (configurable)
```

**How it works**:
1. Start delivery attempt (try direct connection)
2. If fails, get `best_paths()` from ReputationTracker
3. Try path #1 (direct)
4. If fails, try path #2 (via best relay)
5. If fails, try path #3 (via next relay)
6. Update reputation for relays that failed
7. Calculate exponential backoff delay
8. If `should_retry()` → schedule next attempt
9. Repeat until delivered

**Principle**: Never give up
- Messages persist in outbox until delivered
- Continuous adaptation to network conditions
- Multi-path simultaneous attempts (future optimization)
- User can manually delete undeliverable messages

**Result**: Maximum delivery reliability; messages eventually reach destination.

**Code**: `core/src/transport/mesh_routing.rs` (RetryStrategy, MultiPathDelivery)

---

## [Needs Revalidation] Complete Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    SCMessenger Node                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Address    │  │  Multi-Port  │  │  Connection      │  │
│  │  Observer   │  │  Listener    │  │  Tracker         │  │
│  │  (Phase 1)  │  │  (Phase 2)   │  │  (Phase 1)       │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
│         │                  │                   │            │
│         └──────────────────┴───────────────────┘            │
│                          │                                  │
│  ┌──────────────────────▼───────────────────────────────┐  │
│  │            libp2p Swarm (TCP, QUIC)                  │  │
│  │  - Kademlia DHT (peer discovery)                     │  │
│  │  - mDNS (local discovery)                            │  │
│  │  - Request-Response (messaging)                      │  │
│  │  - Address Reflection (Phase 1)                      │  │
│  └──────────────────────────────────────────────────────┘  │
│                          │                                  │
│  ┌──────────────────────▼───────────────────────────────┐  │
│  │         Mesh Routing System (Phases 3-6)             │  │
│  │                                                        │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐ │  │
│  │  │  Relay      │  │ Reputation  │  │  Retry       │ │  │
│  │  │  Stats      │  │ Tracker     │  │  Strategy    │ │  │
│  │  │  (Phase 3)  │  │ (Phase 5)   │  │  (Phase 6)   │ │  │
│  │  └─────────────┘  └─────────────┘  └──────────────┘ │  │
│  │                                                        │  │
│  │  ┌──────────────────┐  ┌─────────────────────────┐   │  │
│  │  │  Bootstrap       │  │  Multi-Path Delivery    │   │  │
│  │  │  Capability      │  │  (Phase 6)              │   │  │
│  │  │  (Phase 4)       │  └─────────────────────────┘   │  │
│  │  └──────────────────┘                                 │  │
│  └────────────────────────────────────────────────────────┘│
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## [Needs Revalidation] Message Delivery Flow

```
1. User sends message to Peer B

2. Lookup Peer B's addresses in DHT

3. Try direct connection
   ├─ Success → Deliver (end)
   └─ Failure → Continue

4. Query ReputationTracker for best relays
   Result: [Relay1 (score: 95), Relay2 (score: 87), Relay3 (score: 72)]

5. Try delivery via Relay1
   ├─ Success → Deliver, update reputation (end)
   └─ Failure → Update reputation, continue

6. Calculate exponential backoff delay (150ms)

7. Try delivery via Relay2
   ├─ Success → Deliver, update reputation (end)
   └─ Failure → Update reputation, continue

8. Try delivery via Relay3
   ├─ Success → Deliver (end)
   └─ Failure → Continue

9. If should_retry() → goto step 4 (with longer delay)

10. Repeat until delivered (never give up)
```

## [Needs Revalidation] Network Characteristics

### [Needs Revalidation] Zero External Dependencies
- ✅ No external STUN servers (Google, Twilio, etc.)
- ✅ No hardcoded bootstrap nodes
- ✅ No centralized relay services
- ✅ No DNS dependencies for core functionality
- ✅ Fully sovereign mesh network

### [Needs Revalidation] Self-Healing & Adaptive
- ✅ Reputation-based routing (bad relays auto-deprioritized)
- ✅ Multi-port listening (firewall traversal)
- ✅ Continuous retry (eventual delivery guaranteed)
- ✅ Peer-observed address discovery (adapts to network changes)

### [Needs Revalidation] Democratic & Distributed
- ✅ Any node can bootstrap others
- ✅ All nodes relay based on capability
- ✅ No special "server" roles
- ✅ Naturally stable nodes emerge as hubs (by usage, not design)

### [Needs Revalidation] Resilient
- ✅ Multi-path delivery
- ✅ Exponential backoff prevents network flooding
- ✅ Reputation recovery (temporary issues tolerated)
- ✅ Messages persist until delivered

## [Needs Revalidation] Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Direct connection latency | ~50-200ms | LAN: <50ms, WAN: 50-200ms |
| 1-hop relay latency | ~100-500ms | Depends on relay location |
| 2-hop relay latency | ~200-1000ms | Multiple relays add latency |
| Address observation time | ~1-3 seconds | Requires 2-3 peer connections |
| Bootstrap time | ~2-5 seconds | Connect + DHT query + peer list |
| Retry max backoff | 30 seconds | Prevents aggressive retry storms |
| Memory per peer | ~1-2 KB | Connection + reputation data |

## [Needs Revalidation] Security Notes

### [Needs Revalidation] Message Confidentiality
- ✅ End-to-end encryption (relays cannot read)
- ✅ Relays only see: peer IDs, message size, timing
- ✅ No metadata leakage to external services

### [Needs Revalidation] Relay Trust Model
- ⚠️ Relays can observe: who talks to whom (metadata)
- ⚠️ Relays can drop/delay messages (but will be deprioritized)
- ✅ Multiple relay attempts mitigate single-relay attacks
- ✅ Reputation system disincentivizes bad behavior

### [Needs Revalidation] Sybil Resistance
- ⚠️ No proof-of-work or staking for peer identity
- ✅ Reputation builds over time (new peers start neutral)
- ✅ Low-reputation peers filtered from routing
- Future: Identity vouching system (web of trust)

## [Needs Revalidation] Testing

All 6 phases have comprehensive test coverage:

- **Phase 1**: `test_address_observation.rs` (~180 LoC)
- **Phase 2**: `test_multiport.rs` (~330 LoC)
- **Phases 3-6**: `test_mesh_routing.rs` (~330 LoC)

**Total test coverage**: ~840 LoC

Run tests:
```bash
cargo test --lib transport
cargo test --test test_address_observation
cargo test --test test_multiport
cargo test --test test_mesh_routing
```

## [Needs Revalidation] Code Structure

```
core/src/transport/
├── behaviour.rs         # libp2p NetworkBehaviour
├── swarm.rs             # Swarm management & event loop
├── reflection.rs        # Address reflection protocol
├── observation.rs       # Phase 1: Address observation
├── multiport.rs         # Phase 2: Multi-port listening
├── mesh_routing.rs      # Phases 3-6: Routing system
├── nat.rs               # NAT type detection
└── internet.rs          # Internet connectivity checks

core/tests/
├── test_address_observation.rs   # Phase 1 tests
├── test_multiport.rs              # Phase 2 tests
└── test_mesh_routing.rs           # Phases 3-6 tests
```

## [Needs Revalidation] Usage Examples

### [Needs Revalidation] Basic Node Startup (Single Port)
```rust
use scmessenger_core::transport;

let (event_tx, event_rx) = mpsc::channel(256);
let swarm = transport::start_swarm(
    keypair,
    Some("/ip4/0.0.0.0/tcp/0".parse().unwrap()),
    event_tx,
).await?;
```

### [Needs Revalidation] Multi-Port Adaptive Node
```rust
use scmessenger_core::transport::{start_swarm_with_config, MultiPortConfig};

let config = MultiPortConfig {
    enable_common_ports: true,    // Try 443, 80, 8080
    enable_random_port: true,     // Plus OS-assigned
    enable_ipv4: true,
    enable_ipv6: true,
    ..Default::default()
};

let swarm = start_swarm_with_config(
    keypair,
    None,
    event_tx,
    Some(config),
).await?;
```

### [Needs Revalidation] Query External Addresses
```rust
let addresses = swarm.get_external_addresses().await?;
println!("My external addresses: {:?}", addresses);
```

### [Needs Revalidation] Bootstrap from Any Peer
```rust
// Connect to ANY known peer
swarm.dial("/ip4/203.0.113.10/tcp/9999/p2p/QmABC...".parse()?).await?;

// Node will automatically:
// 1. Query DHT for more peers
// 2. Add discovered peers to bootstrap capability
// 3. Now can help others bootstrap
```

## [Needs Revalidation] Future Enhancements

### [Needs Revalidation] Short Term
- [ ] Simultaneous multi-path delivery (try 3 relays at once)
- [ ] WebRTC transport for browser support
- [ ] Circuit relay v2 (libp2p standardized)
- [ ] Bandwidth accounting (relay resource limits)

### [Needs Revalidation] Medium Term
- [ ] Onion routing (3-hop privacy)
- [ ] Relay incentivization (reputation → rewards)
- [ ] NAT-PMP/UPnP port forwarding
- [ ] QUIC hole-punching

### [Needs Revalidation] Long Term
- [ ] Distributed reputation consensus (Byzantine fault tolerance)
- [ ] Identity vouching (web of trust)
- [ ] Proof-of-relay (cryptographic relay receipts)
- [ ] Anonymous relay selection (mix networks)

## [Needs Revalidation] Conclusion

The SCMessenger transport layer implements a complete sovereign mesh network with:

✅ **All 6 phases implemented and tested**
✅ **Zero external dependencies**
✅ **Self-healing and adaptive**
✅ **Democratic network participation**
✅ **Production-ready architecture**

**Transport status**: 100% (Core complete)

Next: Web/WASM integration (browser support)
