# SCMessenger: Sovereign Mesh Implementation Plan
## "Works everywhere, owned by no one, unstoppable by design"

**Date:** 2026-02-06 (plan), 2026-02-07 (last updated)
**Status:** Phases 1-7 IMPLEMENTED. Phase 8 (WASM upgrade) pending.
**Estimation:** LoC (lines of code) only. No time estimates.

---

## FOUNDATIONAL DECISIONS

### Relay = Messaging (The Incentive Lock)

This is the single most important architectural decision. It solves the tragedy of the commons that killed every previous mesh project.

**The rule:** One toggle. ON = you can send messages AND you relay for others. OFF = you can do neither. There is no "receive only" mode. There is no "don't relay" mode. If you're on the network, you serve the network.

**Why this works:** Every previous mesh (FireChat, Bridgefy, RightMesh) failed because users took from the network without giving back. SCMessenger makes this structurally impossible. The act of messaging IS the act of relaying. They're the same codepath, the same background service, the same toggle.

**UX framing:** "SCMessenger is always working for you and your community. When you're connected, you're helping messages reach people who need them — and they're doing the same for you."

### No Third-Party Relays

No Nostr relays. No external WebSocket servers. No cloud infrastructure. Every node with internet connectivity IS a relay server for the mesh. The network is self-sustaining or it doesn't exist.

When Node A has internet and Node B doesn't, Node A relays for Node B — not through some external service, but directly. When Node A later loses internet, Node C picks up the relay role. The network is a living organism, not a client-server architecture with extra steps.

### Internet Is A Transport

Internet (TCP/QUIC via libp2p) is the fastest, highest-bandwidth transport available. Use it aggressively when it's there. But it sits alongside BLE, WiFi Aware, WiFi Direct, and physical proximity as just another transport in the stack. The protocol doesn't care how bytes get from A to B.

### Mycorrhizal Routing

Modeled on fungal mycorrhizal networks — the underground communication system connecting trees in a forest:

**How mycorrhiza works in nature:**
- Dense local connections (mycelium around each tree's roots)
- Thin long-distance highways ("common mycorrhizal networks" between distant trees)
- Resources flow where they're needed (demand-driven allocation)
- No central coordinator — each node makes local decisions
- Self-healing — damage routes around breaks
- Hub trees ("mother trees") serve as high-connectivity nodes

**How we model this in the mesh:**

```
LAYER 1 — MYCELIUM (Local Cell)
  Range: Physical proximity (BLE/WiFi, ~100m)
  Awareness: FULL — know every peer, every route, every message in transit
  Update frequency: Real-time (every contact)
  Data structure: Complete adjacency list of local peers
  Analogy: The dense fungal mat around a single tree's roots

LAYER 2 — RHIZOMORPHS (Neighborhood)
  Range: 2-3 hops beyond local cell
  Awareness: SUMMARIZED — know gateway nodes, aggregate capacity, avg latency
  Update frequency: On contact with gateway peers (gossip propagation)
  Data structure: Neighborhood summary table (gateway → reachable hints, capacity, freshness)
  Analogy: The thick mycelial cords connecting nearby trees

LAYER 3 — COMMON MYCORRHIZAL NETWORK (Regional/Global)
  Range: Network-wide (via internet-connected nodes)
  Awareness: ROUTES ONLY — known-active paths to distant regions
  Update frequency: On internet connectivity events
  Data structure: Routing table of (destination_hint → next_hop_hint, hop_count, last_seen)
  Analogy: The vast underground network connecting entire forests

DEMAND-DRIVEN ROUTE DISCOVERY:
  When a message needs to reach an unknown destination:
  1. Check local cell (Layer 1) — do I know this recipient?
  2. Check neighborhood (Layer 2) — does a gateway know this recipient?
  3. Broadcast route request to Layer 3 — "who can reach hint 0xABCD?"
  4. Cache the discovered route for future use
  5. If no route found: store-and-forward (carry until a route appears)
  Analogy: A tree sending chemical signals through the mycelium to find nutrients
```

### Smart Auto-Adjust (Background Aggressiveness)

The app profiles the device and adjusts automatically:

```
CHARGING + WIFI    → Maximum relay: aggressive BLE scan, WiFi Aware, full internet relay
CHARGING + NO WIFI → High relay: aggressive BLE scan, mesh-only
BATTERY > 50%      → Standard relay: periodic BLE scan (every 30s), respond to incoming
BATTERY 20-50%     → Reduced relay: BLE scan every 2 minutes, relay only high-priority
BATTERY < 20%      → Minimal relay: passive BLE only (respond but don't scan), no WiFi Aware
MOVING (GPS/accel) → Increase scan frequency (you're encountering new peers)
STATIONARY         → Decrease scan frequency (same peers, less to sync)
```

**User override:** Full granular control over every parameter. Scan interval, relay budget (max messages/hour), transport selection (BLE on/off, WiFi on/off), battery floor. BUT: the relay=messaging coupling cannot be broken. You can throttle HOW MUCH you relay, but you cannot relay zero while messaging.

---

## YOUR QUESTIONS, ANSWERED DIRECTLY

### "Is it using a blockchain ledger? Could that speed up sync?"

**No. Blockchain is the wrong tool here.** Here's why:

Blockchain solves "how do N parties agree on a single truth." That requires consensus (slow, expensive). Your mesh doesn't NEED consensus. Two nodes meeting don't need to agree on one truth — they need to MERGE their truths. That's a CRDT problem, not a consensus problem.

CRDTs (Conflict-free Replicated Data Types) give you:
- Merge any two stores without conflict → mathematically guaranteed
- No consensus round needed → instant, zero overhead
- No mining, no proof-of-work → zero wasted energy
- Works offline → no internet required for merge

For the specific "which messages do you have that I don't?" problem, the research is definitive: **Minisketch** (Bitcoin's PinSketch algorithm) is near-optimal:

```
BLOOM FILTER (current plan):
  8,192 bits = 1 KB
  Tells you: "these items MIGHT be different" (probabilistic, false positives)
  You still need to exchange IDs to confirm
  Total bandwidth for 1,000 diffs: ~1 KB filter + ~32 KB IDs = ~33 KB

MINISKETCH (replacement):
  Sketch size = element_bits × capacity = 256 × 1,000 = 32 KB
  Tells you: EXACTLY which items differ (deterministic, guaranteed)
  ONE round-trip. No follow-up needed.
  Total bandwidth for 1,000 diffs: 32 KB (and that's it)
  Transmission at 1 Mbps BLE: 256 milliseconds
  Battle-tested: Powers Bitcoin Erlay (handling millions of transactions)

NEGENTROPY (for timestamp-ordered data):
  Uses range-based binary search on ordered sets
  Bandwidth for 1,000 diffs: 1-3 KB (!!)
  But requires 3-4 round-trips (problematic for 5-second BLE window)
  Could use for internet-transport sync where latency is low
```

**The plan:** Minisketch for BLE/WiFi sync (single round-trip, fits in 5-second window). Negentropy for internet-transport sync (multiple round-trips ok, minimal bandwidth). Bloom filters retired.

### "BitChat is not the same"

You're right. BitChat is local BLE mesh + Nostr relay bridge. It's a hybrid with third-party dependency. SCMessenger is fundamentally different:

- **No external relays.** Every node IS the relay.
- **Global mesh.** Internet is just another transport, not a bridge to "the real network."
- **Sovereign identity.** Not dependent on Nostr's pubkey model or any ecosystem.
- **Custom relay protocol.** Purpose-built for the Drift mesh, not adapted from another protocol.

No other project is building a global, self-sustaining, phone-native mesh where every node is simultaneously client, server, and relay with no external infrastructure. This is genuinely new.

### "Reverse engineer the problem — what tricks for stability, reliability, privacy?"

Thinking backwards from "8 billion devices running this":

**Stability trick: Heartbeat reputation.**
Every node tracks which peers reliably relay. Peers that are consistently online, relay efficiently, and have low error rates build reputation. The routing algorithm prefers high-reputation paths. This happens automatically — no central authority, no blockchain, just local observations propagated via gossip.

**Reliability trick: Redundant multi-path delivery.**
Critical messages (delivery receipt requested) are sent via N independent paths simultaneously. First delivery wins. Others are dropped as duplicates. The recipient only sees one message. The network treats it as N separate relay jobs. Cost: N× bandwidth. Benefit: exponential increase in delivery probability.

**Privacy trick: Onion-layered relay.**
For maximum privacy mode: wrap the Drift Envelope in N layers of encryption, one for each hop. Each relay peels one layer, discovers the next hop, and forwards. No relay sees the final destination except the last one. No relay sees the origin except the first one. This is Tor's architecture, applied to the mesh.

**Scale trick: Hierarchical summarization.**
At 1 billion nodes, no single device can hold the full topology. Solution: nodes summarize their neighborhoods into compact routing advertisements. "I can reach 50,000 nodes in the San Francisco region with average 3-hop latency." This summary propagates globally. Individual routes are resolved on-demand within regions. Like DNS: you don't need to know every IP address, just how to find the nameserver that does.

**Anti-fragility trick: Deliberate path diversity.**
The routing algorithm intentionally spreads traffic across DIFFERENT paths, not just the "best" one. This keeps alternative routes warm and tested. If the primary path fails, secondaries are already proven. Like a forest with multiple mycorrhizal networks — the death of one pathway doesn't isolate any tree.

---

## IMPLEMENTATION PHASES

All estimates in LoC (lines of code). Ranges indicate minimum viable → full implementation.

---

### PHASE 1: SECURITY HARDENING + PERSISTENCE

*Fix the known gaps before building on top of them.*

**1A. Ed25519 Envelope Signatures**
Add sender authentication to prevent spoofing. Sign the envelope with the sender's Ed25519 key. Include sender_public_key as AAD in the AEAD encryption.

```
Files:
  core/src/crypto/encrypt.rs     — Add sign/verify, AAD binding (~80-120 LoC)
  core/src/crypto/mod.rs         — Export new types (~10 LoC)
  core/src/message/types.rs      — Add SignedEnvelope struct (~40-60 LoC)
  core/src/lib.rs                — Update prepare/receive to use signing (~30-50 LoC)
  core/src/crypto/tests.rs       — Signature + spoofing tests (~80-120 LoC)

Total: 240-360 LoC
```

**1B. Persistent Storage (Outbox + Inbox to Sled)**
Currently in-memory only. Restart = data loss. Persist both stores.

```
Files:
  core/src/store/outbox.rs       — Sled-backed outbox (~150-200 LoC rewrite)
  core/src/store/inbox.rs        — Sled-backed inbox + dedup set (~150-200 LoC rewrite)
  core/src/store/mod.rs          — Storage initialization (~30-50 LoC)
  core/src/store/tests.rs        — Persistence roundtrip tests (~100-150 LoC)

Total: 430-600 LoC
```

**1C. Discovery Mode (Dark Mode)**
Add configurable discovery to control metadata leakage.

```
Files:
  core/src/transport/discovery.rs — NEW: DiscoveryMode enum + encrypted beacons (~200-300 LoC)
  core/src/transport/swarm.rs     — Conditional mDNS/Identify based on mode (~50-80 LoC)
  core/src/transport/behaviour.rs — Optional behaviours (~30-50 LoC)
  core/src/transport/tests.rs     — Discovery mode tests (~80-120 LoC)

Total: 360-550 LoC
```

**Phase 1 Total: 1,030-1,510 LoC**

---

### PHASE 2: DRIFT PROTOCOL CORE

*The custom wire format, sync engine, and CRDT store that make the mesh work.*

**2A. Drift Envelope Format**
Replace bincode-encoded Envelope with compact binary Drift Envelope. Fixed-width fields, binary UUIDs, LZ4 compression.

```
Files:
  core/src/drift/mod.rs          — NEW module root (~20 LoC)
  core/src/drift/envelope.rs     — NEW: DriftEnvelope struct, serialize/deserialize (~250-350 LoC)
  core/src/drift/frame.rs        — NEW: DriftFrame (transport framing with CRC32) (~150-200 LoC)
  core/src/drift/compress.rs     — NEW: LZ4 compress/decompress wrapper (~40-60 LoC)
  core/src/drift/tests.rs        — Roundtrip, fuzz, size validation tests (~150-200 LoC)

New dependency: lz4_flex (pure Rust LZ4)

Total: 610-830 LoC
```

**2B. Minisketch Set Reconciliation**
Replace bloom filters with Minisketch for deterministic, near-optimal sync.

```
Files:
  core/src/drift/sketch.rs       — NEW: Minisketch wrapper (create, merge, decode) (~200-300 LoC)
  core/src/drift/sync.rs         — NEW: SyncProtocol (handshake, sketch exchange, transfer) (~300-450 LoC)
  core/src/drift/tests_sync.rs   — Sync correctness tests (identical sets, disjoint, partial) (~200-300 LoC)

New dependency: minisketch-rs (Rust bindings to libminisketch) or pure-Rust implementation

Total: 700-1,050 LoC
```

**2C. CRDT Message Store**
Replace HashMap-based storage with a proper G-Set CRDT with merge semantics.

```
Files:
  core/src/drift/store.rs        — NEW: MeshStore (CRDT, merge, eviction, priority scoring) (~350-500 LoC)
  core/src/drift/priority.rs     — NEW: Priority score calculation (age, hops, TTL, reputation) (~80-120 LoC)
  core/src/drift/tests_store.rs  — CRDT merge tests, eviction tests, priority tests (~200-300 LoC)

Total: 630-920 LoC
```

**2D. Relay = Messaging Coupling**
The core enforcement: relay and messaging as a single indivisible service.

```
Files:
  core/src/drift/relay.rs        — NEW: RelayService (receive→store→forward loop) (~200-300 LoC)
  core/src/drift/policy.rs       — NEW: RelayPolicy (auto-adjust profiles, battery awareness) (~150-250 LoC)
  core/src/lib.rs                — Update IronCore to enforce relay=messaging coupling (~50-80 LoC)
  core/src/drift/tests_relay.rs  — Relay enforcement tests (~100-150 LoC)

Total: 500-780 LoC
```

**Phase 2 Total: 2,440-3,580 LoC**

---

### PHASE 3: MYCORRHIZAL ROUTING

*The intelligence layer. How messages find their way through the mesh.*

**3A. Layer 1 — Mycelium (Local Cell Topology)**
Full awareness of directly-reachable peers.

```
Files:
  core/src/routing/mod.rs        — NEW module root (~20 LoC)
  core/src/routing/local.rs      — NEW: LocalCell (adjacency list, peer capabilities, real-time updates) (~200-300 LoC)
  core/src/routing/peer_info.rs  — NEW: PeerInfo struct (hint, pubkey, capabilities, battery, connectivity, reputation) (~80-120 LoC)
  core/src/routing/tests_local.rs — Local cell tests (~100-150 LoC)

Total: 400-590 LoC
```

**3B. Layer 2 — Rhizomorphs (Neighborhood Gossip)**
Summarized awareness of 2-3 hop neighbors via gossip propagation.

```
Files:
  core/src/routing/neighborhood.rs — NEW: NeighborhoodTable (gateway summaries, reachability) (~250-400 LoC)
  core/src/routing/gossip.rs       — NEW: GossipProtocol (periodic exchange of neighborhood summaries) (~200-300 LoC)
  core/src/routing/tests_neighborhood.rs — Gossip convergence tests (~150-200 LoC)

Total: 600-900 LoC
```

**3C. Layer 3 — Common Mycorrhizal Network (Global Routes)**
Network-wide route advertisements via internet-connected nodes.

```
Files:
  core/src/routing/global.rs       — NEW: GlobalRouteTable (destination_hint → next_hop, metrics) (~200-300 LoC)
  core/src/routing/advertisement.rs — NEW: RouteAdvertisement (compact route summaries, propagation) (~150-250 LoC)
  core/src/routing/discovery.rs    — NEW: on-demand route discovery ("who can reach hint X?") (~200-300 LoC)
  core/src/routing/tests_global.rs — Route convergence, demand-driven discovery tests (~150-250 LoC)

Total: 700-1,100 LoC
```

**3D. Routing Decision Engine**
The brain. Given a message and the routing tables, decide the optimal next hop(s).

```
Files:
  core/src/routing/engine.rs       — NEW: RoutingEngine (layer cascade, multi-path selection, reputation weighting) (~300-450 LoC)
  core/src/routing/reputation.rs   — NEW: PeerReputation (relay success rate, uptime, consistency) (~150-200 LoC)
  core/src/routing/tests_engine.rs — End-to-end routing decision tests, multi-path tests (~200-300 LoC)

Total: 650-950 LoC
```

**Phase 3 Total: 2,350-3,540 LoC**

---

### PHASE 4: TRANSPORT LAYER (MULTI-TRANSPORT)

*BLE, WiFi Aware, WiFi Direct, Internet — all equal transports.*

**4A. Transport Abstraction**
Unified interface so the routing engine doesn't care HOW bytes move.

```
Files:
  core/src/transport/abstraction.rs — NEW: TransportTrait, TransportEvent, TransportCommand (~100-150 LoC)
  core/src/transport/manager.rs     — NEW: TransportManager (multiplexes all transports, feeds routing engine) (~200-300 LoC)
  core/src/transport/tests_mgr.rs   — Transport multiplexing tests (~100-150 LoC)

Total: 400-600 LoC
```

**4B. BLE Transport (L2CAP + GATT Fallback)**
The core offline transport. Must work in iOS/Android background.

```
Files:
  core/src/transport/ble/mod.rs       — NEW: BLE module root (~20 LoC)
  core/src/transport/ble/beacon.rs    — NEW: Encrypted BLE beacon format + rotation (~150-200 LoC)
  core/src/transport/ble/l2cap.rs     — NEW: L2CAP channel management (stream-oriented) (~200-300 LoC)
  core/src/transport/ble/gatt.rs      — NEW: GATT service definition (0xDF01) + fragmentation (~250-350 LoC)
  core/src/transport/ble/scanner.rs   — NEW: BLE scanner with duty cycle management (~150-200 LoC)
  core/src/transport/ble/tests.rs     — BLE transport tests (~150-200 LoC)

Note: iOS/Android native BLE code lives in mobile/ via UniFFI callbacks.
The Rust side defines the protocol; platform code handles CoreBluetooth/Android BLE APIs.

Total: 920-1,270 LoC (Rust side)
```

**4C. WiFi Aware Transport (Android)**
High-bandwidth local transport. P2P without access point.

```
Files:
  core/src/transport/wifi_aware.rs    — NEW: WiFi Aware abstraction (discovery + data path) (~150-250 LoC)
  mobile/src/android/wifi_aware.rs    — NEW: Android WiFi Aware JNI bridge (~200-300 LoC)

Total: 350-550 LoC
```

**4D. Internet Transport (libp2p Enhancement)**
Upgrade existing libp2p transport for Drift Protocol compatibility.

```
Files:
  core/src/transport/swarm.rs        — Refactor to implement TransportTrait (~100-150 LoC changes)
  core/src/transport/internet.rs     — NEW: Internet relay mode (when online, relay for mesh) (~200-300 LoC)
  core/src/transport/nat.rs          — NEW: NAT traversal helpers (hole-punching, relay circuit) (~150-250 LoC)

Total: 450-700 LoC
```

**4E. Transport Escalation Protocol**
Auto-negotiate the best available transport between two peers.

```
Files:
  core/src/transport/escalation.rs   — NEW: Escalation logic (BLE→WiFi Aware→Internet) (~150-250 LoC)
  core/src/transport/tests_esc.rs    — Escalation decision tests (~80-120 LoC)

Total: 230-370 LoC
```

**Phase 4 Total: 2,350-3,490 LoC**

---

### PHASE 5: MOBILE PLATFORM INTEGRATION

*Making it actually work on phones in background.*

**5A. Android Foreground Service + VPN Mode**
The persistent background service with auto-adjust.

```
Files:
  mobile/src/android/service.rs      — NEW: MeshService (foreground service, lifecycle) (~200-300 LoC Rust)
  mobile/src/android/auto_adjust.rs  — NEW: SmartAutoAdjust (battery, charging, motion detection) (~150-250 LoC)
  mobile/src/android/vpn.rs          — NEW: VPN service mode (optional, for persistent background) (~200-300 LoC)

Platform-side Kotlin (generated + custom):
  MeshService.kt                     — Foreground service + notification (~150-200 LoC Kotlin)
  MeshVpnService.kt                  — VPN mode (~100-150 LoC Kotlin)

Total: 800-1,200 LoC (Rust + Kotlin)
```

**5B. iOS Composite Background Strategy**
All background modes working together.

```
Files:
  mobile/src/ios/background.rs       — NEW: iOS background mode orchestration (~200-300 LoC Rust)
  mobile/src/ios/auto_adjust.rs      — NEW: iOS-specific auto-adjust (accounts for iOS restrictions) (~100-150 LoC)

Platform-side Swift (generated + custom):
  MeshManager.swift                  — CoreBluetooth + location + BGTask composite (~250-350 LoC Swift)
  BLETransport.swift                 — CoreBluetooth central+peripheral management (~200-300 LoC Swift)

Total: 750-1,100 LoC (Rust + Swift)
```

**5C. User Controls UI (Settings)**
The smart auto-adjust + manual override interface.

```
Files:
  core/src/drift/settings.rs         — NEW: MeshSettings (serializable config, validation) (~100-150 LoC)
  mobile/src/settings_bridge.rs      — NEW: UniFFI bridge for settings (~50-80 LoC)

Platform UI:
  SettingsView.swift / SettingsActivity.kt — ~200-300 LoC per platform

Total: 550-860 LoC (Rust + platform)
```

**Phase 5 Total: 2,100-3,160 LoC**

---

### PHASE 6: SELF-RELAY NETWORK PROTOCOL

*The custom relay protocol. Every node with internet is a relay server.*

**6A. Drift Relay Protocol**
When a node has internet, it listens for relay connections from other mesh nodes.

```
Files:
  core/src/relay/mod.rs             — NEW module root (~20 LoC)
  core/src/relay/server.rs          — NEW: RelayServer (accept connections, store-and-forward) (~300-450 LoC)
  core/src/relay/client.rs          — NEW: RelayClient (connect to known relays, push/pull sync) (~250-350 LoC)
  core/src/relay/protocol.rs        — NEW: Relay wire protocol (handshake, auth, sync, disconnect) (~200-300 LoC)
  core/src/relay/peer_exchange.rs   — NEW: Exchange known relay addresses (bootstrap) (~150-200 LoC)
  core/src/relay/tests.rs           — Relay roundtrip, multi-hop, NAT traversal tests (~200-300 LoC)

Total: 1,120-1,620 LoC
```

**6B. Bootstrap Protocol**
How new nodes discover their first peers and join the mesh.

```
Files:
  core/src/relay/bootstrap.rs       — NEW: Bootstrap protocol (QR code, invite link, BLE discovery, hardcoded seed peers) (~200-300 LoC)
  core/src/relay/invite.rs          — NEW: Invite system (friend introduces friend = new mesh connection) (~150-250 LoC)
  core/src/relay/tests_bootstrap.rs — Bootstrap scenario tests (~100-150 LoC)

Total: 450-700 LoC
```

**6C. Find My Wake-Up (Optional/Experimental)**
Use Apple Find My beacon protocol to send "you have mail" wakeups through Apple's global relay network. User preference controlled.

```
Files:
  core/src/relay/findmy.rs          — NEW: Find My beacon encoding/decoding (~200-300 LoC)
  mobile/src/ios/findmy_beacon.rs   — NEW: iOS Find My beacon broadcasting (~100-150 LoC)
  core/src/relay/tests_findmy.rs    — Beacon encode/decode tests (~80-120 LoC)

Total: 380-570 LoC
```

**Phase 6 Total: 1,950-2,890 LoC**

---

### PHASE 7: PRIVACY ENHANCEMENTS

*Onion routing, metadata protection, plausible deniability.*

**7A. Onion-Layered Relay**
For maximum privacy mode: wrap messages in N layers of encryption.

```
Files:
  core/src/privacy/mod.rs           — NEW module root (~20 LoC)
  core/src/privacy/onion.rs         — NEW: OnionEnvelope (layer, peel, construct) (~300-450 LoC)
  core/src/privacy/circuit.rs       — NEW: Circuit construction (select N hops, build onion) (~200-300 LoC)
  core/src/privacy/tests.rs         — Onion routing correctness tests (~150-250 LoC)

Total: 670-1,020 LoC
```

**7B. Traffic Analysis Resistance**
Padding, timing jitter, and cover traffic to prevent traffic analysis.

```
Files:
  core/src/privacy/padding.rs       — NEW: Message padding to fixed sizes (~80-120 LoC)
  core/src/privacy/timing.rs        — NEW: Randomized relay delays (~80-120 LoC)
  core/src/privacy/cover.rs         — NEW: Cover traffic generation (dummy messages) (~100-150 LoC)
  core/src/privacy/tests_ta.rs      — Traffic analysis resistance tests (~100-150 LoC)

Total: 360-540 LoC
```

**Phase 7 Total: 1,030-1,560 LoC**

---

### PHASE 8: WASM CLIENT UPGRADE

*Browser as a full mesh participant (when tab is open).*

**8A. WASM Transport (WebRTC + WebSocket)**
Give the browser a real transport layer.

```
Files:
  wasm/src/transport.rs             — NEW: WebRTC data channel + WebSocket to nearby relay nodes (~250-350 LoC)
  wasm/src/mesh.rs                  — NEW: Full mesh participation (relay, sync, route) while tab open (~200-300 LoC)
  wasm/src/lib.rs                   — Update bindings for mesh operations (~50-80 LoC)

Total: 500-730 LoC
```

**8B. Service Worker (Background Sync)**
Brief background sync via service worker + push notifications.

```
Files:
  wasm/src/worker.rs                — NEW: Service worker registration + push handler (~150-200 LoC)
  wasm/src/storage.rs               — NEW: OPFS-backed message store (~100-150 LoC)

Total: 250-350 LoC
```

**Phase 8 Total: 750-1,080 LoC**

---

## GRAND TOTAL

| Phase | Description | LoC Range | Status |
|-------|-------------|-----------|--------|
| 1 | Security Hardening + Persistence | 1,030-1,510 | **COMPLETE** |
| 2 | Drift Protocol Core | 2,440-3,580 | **COMPLETE** |
| 3 | Mycorrhizal Routing | 2,350-3,540 | **COMPLETE** |
| 4 | Transport Layer (Multi-Transport) | 2,350-3,490 | **COMPLETE** |
| 5 | Mobile Platform Integration | 2,100-3,160 | **COMPLETE** |
| 6 | Self-Relay Network Protocol | 1,950-2,890 | **COMPLETE** |
| 7 | Privacy Enhancements | 1,030-1,560 | **COMPLETE** |
| 8 | WASM Client Upgrade | 750-1,080 | Pending |
| **TOTAL** | | **14,000-20,810 LoC** | |

**Current state:** The codebase is ~53,000 LoC across all workspace members (core: ~29K, cli: ~500, wasm: ~2.4K, plus lib.rs at ~19K). ~2,641 tests across 71 source files in core. All phases through 7 are implemented and unit-tested. The remaining integration gap is wiring IronCore to SwarmHandle via the CLI.

---

## DEPENDENCY GRAPH (BUILD ORDER)

```
Phase 1 (Security) ──→ Phase 2 (Drift Protocol) ──→ Phase 3 (Routing)
                                    │                      │
                                    ▼                      ▼
                            Phase 4 (Transport) ←──── Phase 3 (Routing)
                                    │
                                    ▼
                     ┌──── Phase 5 (Mobile) ────┐
                     │                          │
                     ▼                          ▼
              Phase 6 (Relay)            Phase 8 (WASM)
                     │
                     ▼
              Phase 7 (Privacy)
```

Phases 1→2→3 are strictly sequential (each builds on the last).
Phase 4 can begin in parallel with Phase 3 (transport abstraction doesn't depend on routing internals).
Phases 5, 6, 7, 8 can proceed in parallel once Phases 2-4 are complete.

---

## KEY ARCHITECTURAL DECISIONS SUMMARY

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Sync algorithm | Minisketch (BLE) + Negentropy (internet) | Near-optimal bandwidth, deterministic, battle-tested |
| Message store | CRDT (G-Set) | Conflict-free merge, no consensus needed, works offline |
| Routing model | Mycorrhizal (3-layer hierarchical) | Dense local, sparse global, demand-driven, self-healing |
| Relay model | Every node = relay (no third parties) | Sovereign by design, no external dependencies |
| Incentive model | Relay = Messaging (single toggle) | Structural impossibility of free-riding |
| Background survival | VPN Service (Android) + Composite (iOS) | Maximum persistence within OS constraints |
| Wire format | Drift Envelope (154 bytes overhead) | 40% smaller than current, fixed-width, compressed |
| Identity | Ed25519 keypair = you | No phone numbers, no emails, no accounts |
| Encryption | XChaCha20-Poly1305 + per-message ephemeral X25519 | Forward secrecy, authenticated encryption |
| Sender auth | Ed25519 envelope signature + AAD binding | Prevents spoofing, allows relay verification |
| Privacy mode | Optional onion routing (N-layer encryption) | Tor-grade privacy when needed |
| Auto-adjust | Smart profiling (battery, charging, motion) + full override | Mass-market defaults, power-user control |
