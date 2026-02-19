# NAT Traversal & Internet Roaming — Implementation Plan

## Overview

SCMessenger currently works over LAN (mDNS-discovered peers, direct TCP connections).
This plan enables **cross-network messaging** via:

1. **Bootstrap Nodes** — Well-known relay nodes that help peers discover each other
2. **NAT Detection** — Determine what kind of NAT the device is behind
3. **Hole Punching** — Establish direct connections through compatible NATs
4. **Relay Circuits** — Fallback relayed connections when hole punching fails
5. **Internet Roaming** — Seamless connectivity when switching between WiFi/cellular

## Architecture

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

## Phase 1: Bootstrap Node Configuration (Rust Core)

### What

Add configurable bootstrap node addresses that the swarm auto-dials on startup.

### Where

- `core/src/transport/swarm.rs` — Accept bootstrap addrs in `start_swarm_with_config()`
- `core/src/mobile_bridge.rs` — Expose `set_bootstrap_nodes()` via UniFFI
- `core/src/api.udl` — Add `set_bootstrap_nodes(sequence<string> addrs)` to `MeshService`

### Changes

1. Add `bootstrap_addrs: Vec<String>` to `SwarmCommand::Connect` or pass as parameter
2. After swarm starts listening, auto-dial each bootstrap addr
3. Add bootstrap addr to Kademlia for DHT participation
4. Mobile apps configure bootstrap addrs from settings/hardcoded defaults

### Default Bootstrap Nodes

```
/dns4/bootstrap.scmessenger.net/tcp/4001
/ip4/<VPS-IP>/tcp/4001
```

---

## Phase 2: NAT Detection Integration

### What

Wire the existing `NatTraversal::probe_nat()` into the swarm lifecycle.

### Where

- `core/src/transport/nat.rs` — Already implemented
- `core/src/transport/swarm.rs` — Trigger NAT probe after connecting to bootstrap
- `core/src/mobile_bridge.rs` — Expose NAT status

### Changes

1. After first bootstrap peer connects, request address reflection
2. Store NAT type and external address in `NatTraversal`
3. Periodically re-probe (every 5 min) to detect roaming
4. Expose `get_nat_status() -> String` via UniFFI

---

## Phase 3: Hole Punching (DCUtR Integration)

### What

Use libp2p's built-in DCUtR (Direct Connection Upgrade through Relay) protocol.

### Where

- `core/src/transport/behaviour.rs` — Add `dcutr::Behaviour`
- `core/src/transport/swarm.rs` — Handle DCUtR events

### Changes

1. Add `libp2p::dcutr` behaviour to `IronCoreBehaviour`
2. Add `libp2p::relay::client::Behaviour` for relay-assisted connections
3. When connecting to a peer behind NAT:
   a. First attempt direct connection
   b. If fails, connect via relay circuit
   c. DCUtR automatically upgrades to direct connection when possible

### Dependencies

```toml
# Cargo.toml additions
libp2p = { features = ["relay", "dcutr"] }
```

---

## Phase 4: Relay Circuit Fallback

### What

When hole punching fails (symmetric NAT), route traffic through relay nodes.

### Where

- `core/src/relay/server.rs` — Already has `RelayServer` with store-and-forward
- `core/src/transport/internet.rs` — Already has `InternetRelay` with relay mode
- `core/src/transport/swarm.rs` — Wire relay into connection strategy

### Changes

1. On swarm startup with bootstrap, also configure as relay client
2. When direct dial fails, attempt relay connection:
   `/ip4/<relay>/tcp/4001/p2p/<relay-peer-id>/p2p-circuit/p2p/<target-peer-id>`
3. Relay server tracks bandwidth and applies limits
4. Store-and-forward: when target is offline, relay stores messages (up to 100 per peer)
5. When target comes online and connects to relay, deliver stored messages

---

## Phase 5: Internet Roaming

### What

Seamless reconnection when network changes (WiFi → cellular, roaming between APs).

### Where

- `core/src/mobile_bridge.rs` — `on_network_changed()` handler
- iOS: `MeshRepository.swift` — Network reachability monitoring
- Android: `MeshRepository.kt` — ConnectivityManager monitoring

### Changes

1. When network change detected:
   a. Re-dial all bootstrap nodes
   b. Re-probe NAT type (may change on new network)
   c. Re-announce on Kademlia DHT with new addresses
2. Keep-alive mechanism: periodic ping to bootstrap (every 30s)
3. Exponential backoff on connection failures
4. On iOS: Use `NWPathMonitor` for network state changes
5. On Android: Use `ConnectivityManager.NetworkCallback`

---

## Phase 6: Mobile Integration

### iOS Changes

- `MeshRepository.swift`:
  - Add bootstrap node configuration
  - Monitor network with `NWPathMonitor`
  - Call `meshService.reconnect()` on network change
- `SettingsView.swift`:
  - Add "Internet Relay" toggle
  - Show NAT type and external address
  - Allow custom bootstrap node configuration

### Android Changes

- `MeshRepository.kt`:
  - Add bootstrap node configuration
  - Monitor network with `ConnectivityManager`
  - Call `meshService.reconnect()` on network change
- `SettingsScreen.kt`:
  - Add "Internet Relay" toggle
  - Show NAT type and external address
  - Allow custom bootstrap node configuration

---

## Phase 7: Testing & Verification

### Unit Tests

- NAT detection with mocked peer responses
- Relay circuit establishment and teardown
- Store-and-forward queue management
- Bootstrap node connection retry logic

### Integration Tests (Docker)

- Two containers with simulated NAT (iptables)
- Message delivery through NAT
- Relay fallback when direct connection blocked
- Network failover simulation

### Manual Verification

- iOS device → Bootstrap VPS → Android device
- WiFi → Cellular roaming during active conversation
- Offline message delivery (send while recipient offline)

---

## Implementation Order

1. **Phase 1**: Bootstrap node configuration (~2 hours)
2. **Phase 2**: NAT detection integration (~1 hour)
3. **Phase 4**: Relay circuit fallback (~3 hours)
4. **Phase 3**: Hole punching with DCUtR (~2 hours)
5. **Phase 5**: Internet roaming (~2 hours)
6. **Phase 6**: Mobile UI integration (~2 hours)
7. **Phase 7**: Testing (~3 hours)

**Total estimated effort: ~15 hours**

---

## Quick Start: Deploy a Bootstrap Node

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
