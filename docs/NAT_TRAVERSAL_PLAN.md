> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

# NAT Traversal & Internet Roaming — Implementation Plan

## [Needs Revalidation] Overview

SCMessenger currently works over LAN (mDNS-discovered peers, direct TCP connections).
This plan enables **cross-network messaging** via:

1. **Bootstrap Nodes** — Well-known relay nodes that help peers discover each other
2. **NAT Detection** — Determine what kind of NAT the device is behind
3. **Hole Punching** — Establish direct connections through compatible NATs
4. **Relay Circuits** — Fallback relayed connections when hole punching fails
5. **Internet Roaming** — Seamless connectivity when switching between WiFi/cellular

## [Needs Revalidation] Architecture

```
┌──────────────┐       ┌──────────────┐
│  Mobile A    │       │  Mobile B    │
│  (behind NAT)│       │  (behind NAT)│
└──────┬───────┘       └──────┬───────┘
       │                      │
       │ 1. Connect to        │ 1. Connect to
       │    bootstrap         │    bootstrap
       ▼                      ▼
┌──────────────────────────────────────┐
│         Bootstrap / Relay Node       │
│  (VPS with public IP, always-on)     │
│  - Peer discovery (Kademlia DHT)     │
│  - Address reflection (like STUN)    │
│  - Relay forwarding (like TURN)      │
│  - Store-and-forward for offline     │
└──────────────────────────────────────┘
       │                      │
       │ 2. Exchange observed │
       │    external addrs    │
       │                      │
       │ 3. Attempt hole-punch│
       │◄─────────────────────│  (UDP simultaneous open)
       │─────────────────────►│
       │                      │
       │ 4. If hole-punch     │
       │    fails, use relay  │
       │    circuit via       │
       │    bootstrap node    │
```

---

## [Needs Revalidation] Phase 1: Bootstrap Node Configuration (Rust Core)

### [Needs Revalidation] What

Add configurable bootstrap node addresses that the swarm auto-dials on startup.

### [Needs Revalidation] Where

- `core/src/transport/swarm.rs` — Accept bootstrap addrs in `start_swarm_with_config()`
- `core/src/mobile_bridge.rs` — Expose `set_bootstrap_nodes()` via UniFFI
- `core/src/api.udl` — Add `set_bootstrap_nodes(sequence<string> addrs)` to `MeshService`

### [Needs Revalidation] Changes

1. Add `bootstrap_addrs: Vec<String>` to `SwarmCommand::Connect` or pass as parameter
2. After swarm starts listening, auto-dial each bootstrap addr
3. Add bootstrap addr to Kademlia for DHT participation
4. Mobile apps configure bootstrap addrs from settings/hardcoded defaults

### [Needs Revalidation] Default Bootstrap Nodes

```
/dns4/bootstrap.scmessenger.net/tcp/4001
/ip4/<VPS-IP>/tcp/4001
```

---

## [Needs Revalidation] Phase 2: NAT Detection Integration

### [Needs Revalidation] What

Wire the existing `NatTraversal::probe_nat()` into the swarm lifecycle.

### [Needs Revalidation] Where

- `core/src/transport/nat.rs` — Already implemented
- `core/src/transport/swarm.rs` — Trigger NAT probe after connecting to bootstrap
- `core/src/mobile_bridge.rs` — Expose NAT status

### [Needs Revalidation] Changes

1. After first bootstrap peer connects, request address reflection
2. Store NAT type and external address in `NatTraversal`
3. Periodically re-probe (every 5 min) to detect roaming
4. Expose `get_nat_status() -> String` via UniFFI

---

## [Needs Revalidation] Phase 3: Hole Punching (DCUtR Integration)

### [Needs Revalidation] What

Use libp2p's built-in DCUtR (Direct Connection Upgrade through Relay) protocol.

### [Needs Revalidation] Where

- `core/src/transport/behaviour.rs` — Add `dcutr::Behaviour`
- `core/src/transport/swarm.rs` — Handle DCUtR events

### [Needs Revalidation] Changes

1. Add `libp2p::dcutr` behaviour to `IronCoreBehaviour`
2. Add `libp2p::relay::client::Behaviour` for relay-assisted connections
3. When connecting to a peer behind NAT:
   a. First attempt direct connection
   b. If fails, connect via relay circuit
   c. DCUtR automatically upgrades to direct connection when possible

### [Needs Revalidation] Dependencies

```toml
# Cargo.toml additions
libp2p = { features = ["relay", "dcutr"] }
```

---

## [Needs Revalidation] Phase 4: Relay Circuit Fallback

### [Needs Revalidation] What

When hole punching fails (symmetric NAT), route traffic through relay nodes.

### [Needs Revalidation] Where

- `core/src/relay/server.rs` — Already has `RelayServer` with store-and-forward
- `core/src/transport/internet.rs` — Already has `InternetRelay` with relay mode
- `core/src/transport/swarm.rs` — Wire relay into connection strategy

### [Needs Revalidation] Changes

1. On swarm startup with bootstrap, also configure as relay client
2. When direct dial fails, attempt relay connection:
   `/ip4/<relay>/tcp/4001/p2p/<relay-peer-id>/p2p-circuit/p2p/<target-peer-id>`
3. Relay server tracks bandwidth and applies limits
4. Store-and-forward: when target is offline, relay stores messages (up to 100 per peer)
5. When target comes online and connects to relay, deliver stored messages

---

## [Needs Revalidation] Phase 5: Internet Roaming

### [Needs Revalidation] What

Seamless reconnection when network changes (WiFi → cellular, roaming between APs).

### [Needs Revalidation] Where

- `core/src/mobile_bridge.rs` — `on_network_changed()` handler
- iOS: `MeshRepository.swift` — Network reachability monitoring
- Android: `MeshRepository.kt` — ConnectivityManager monitoring

### [Needs Revalidation] Changes

1. When network change detected:
   a. Re-dial all bootstrap nodes
   b. Re-probe NAT type (may change on new network)
   c. Re-announce on Kademlia DHT with new addresses
2. Keep-alive mechanism: periodic ping to bootstrap (every 30s)
3. Exponential backoff on connection failures
4. On iOS: Use `NWPathMonitor` for network state changes
5. On Android: Use `ConnectivityManager.NetworkCallback`

---

## [Needs Revalidation] Phase 6: Mobile Integration

### [Needs Revalidation] iOS Changes

- `MeshRepository.swift`:
  - Add bootstrap node configuration
  - Monitor network with `NWPathMonitor`
  - Call `meshService.reconnect()` on network change
- `SettingsView.swift`:
  - Add "Internet Relay" toggle
  - Show NAT type and external address
  - Allow custom bootstrap node configuration

### [Needs Revalidation] Android Changes

- `MeshRepository.kt`:
  - Add bootstrap node configuration
  - Monitor network with `ConnectivityManager`
  - Call `meshService.reconnect()` on network change
- `SettingsScreen.kt`:
  - Add "Internet Relay" toggle
  - Show NAT type and external address
  - Allow custom bootstrap node configuration

---

## [Needs Revalidation] Phase 7: Testing & Verification

### [Needs Revalidation] Unit Tests

- NAT detection with mocked peer responses
- Relay circuit establishment and teardown
- Store-and-forward queue management
- Bootstrap node connection retry logic

### [Needs Revalidation] Integration Tests (Docker)

- Two containers with simulated NAT (iptables)
- Message delivery through NAT
- Relay fallback when direct connection blocked
- Network failover simulation

### [Needs Revalidation] Manual Verification

- iOS device → Bootstrap VPS → Android device
- WiFi → Cellular roaming during active conversation
- Offline message delivery (send while recipient offline)

---

## [Needs Revalidation] Implementation Order

1. **Phase 1**: Bootstrap node configuration (~2 hours)
2. **Phase 2**: NAT detection integration (~1 hour)
3. **Phase 4**: Relay circuit fallback (~3 hours)
4. **Phase 3**: Hole punching with DCUtR (~2 hours)
5. **Phase 5**: Internet roaming (~2 hours)
6. **Phase 6**: Mobile UI integration (~2 hours)
7. **Phase 7**: Testing (~3 hours)

**Total estimated effort: ~15 hours**

---

## [Needs Revalidation] Quick Start: Deploy a Bootstrap Node

```bash
# On a VPS with public IP
cargo build --release -p scmessenger-core --bin scm
./scm start --bootstrap-mode --listen /ip4/0.0.0.0/tcp/4001
```

The bootstrap node acts as:

- DHT seed (Kademlia)
- Address reflector (like STUN)
- Relay node (like TURN)
- Store-and-forward mailbox
