# SCMessenger Dynamic-Port Discovery Migration Plan

**Date:** 2026-06-05
**Author:** deepseek-v4-pro (delegated research subagent)
**Overseer:** Lucas Ballek
**Repo:** `/mnt/e/SCMessenger-Github-Repo/SCMessenger`
**Status:** RESEARCH — ready for Phase 0 kick-off

---

## 1. Executive Summary

**Yes, this is feasible — and the codebase is already partway there.**

A clean read of the repo shows that SCMessenger's *Rust core* already has the three
primitives needed for a dynamic-port-aware discovery design:

- `core/src/transport/multiport.rs` — already binds `/ip4/0.0.0.0/tcp/0` and `/ip4/0.0.0.0/udp/0/quic-v1` (ephemeral, kernel-assigned). The infrastructure for "let the OS pick" exists and is exercised on every swarm boot.
- `core/src/transport/nat.rs` and `core/src/transport/reflection.rs` — a *peer-assisted* STUN equivalent: `AddressReflectionRequest/Response` lets any node ask a connected mesh peer "what's my IP:port as you see it?" This is the sovereign-mesh replacement for an external STUN server.
- `core/src/transport/discovery.rs::DiscoveryMode::LanOnly` — UDP broadcast fallback on a "well-known port (9001)" — the placeholder we are about to replace.

**What is missing is wiring, not invention.** The platform clients and the CLI
ship hardcoded port literals: `9000`, `9001`, `9002`, `9001`/`9002` in the Android
`SubnetProbe`, `9001` in Android mDNS TXT, and `0.0.0.0:0` for the swarm's TCP listener
but `0.0.0.0:9002/ws` for the WebSocket listener (line 1901 of `swarm.rs`).

**Cost:** ~6–10 weeks of focused work split across 4 phases, with the first two
phases (~3 weeks) delivering the immediate LAN-discovery unblock Lucas is blocked on
right now (the WSL↔Android↔Windows case in `HANDOFF/todo/P1_ANDROID_LAN_DISCOVERY_REPAIR.md`).

**Win:** A mesh where (a) every node has its own discoverable port per launch, (b)
discovery works across WSL Hyper-V virtual NICs and Windows LAN without requiring
the user to pick a static port, (c) port collisions are detected and recovered
automatically, (d) the protocol can be port-hopped for censorship resistance, and
(e) we can detect our own NAT mapping *without* an external STUN server using the
sender-spoof liveness probe sketched in §4-C.

**Scope this document does NOT cover:** BLE port-hopping (BLE channels are physical,
not TCP), WiFi-Direct/Aware SSID changes (out of v0.3 scope), and the WASM thin
client (browser security model constrains it to whatever port the bridge exposes).

---

## 2. Current State (with line numbers)

All line numbers refer to files in the current `main` branch as of `git log -1` = `118dd6ef`.
Findings are taken from `grep -n` against the working tree, not memory.

### 2.1 Rust core — `core/src/transport/`

| What | File:line | Value |
|---|---|---|
| Discovery mode "well-known port" comment | `core/src/transport/discovery.rs:63` | "well-known port (9001)" |
| `DiscoveryMode::LanOnly` placeholder | `core/src/transport/discovery.rs:64-67` | hardcoded in docstring only |
| Multi-port common-ports list | `core/src/transport/multiport.rs:12-17` | `[443, 80, 8080, 9090]` |
| Multi-port random-port append | `core/src/transport/multiport.rs:90` | `add_port(0);` (already ephemeral!) |
| Default swarm TCP listen | `core/src/transport/swarm.rs:1867-1872` | `"/ip4/0.0.0.0/tcp/0"` ← *already random* |
| Default swarm QUIC listen | `core/src/transport/swarm.rs:1891` | `"/ip4/0.0.0.0/udp/0/quic-v1"` ← *already random* |
| **Hardcoded WS listen (the asymmetry)** | `core/src/transport/swarm.rs:1901` | `"/ip4/0.0.0.0/tcp/9002/ws"` |
| Multi-port bind invocation | `core/src/transport/swarm.rs:1842` | `multiport::generate_listen_addresses(&config)` |
| `NatTraversal::start_hole_punch` | `core/src/transport/nat.rs:388-437` | In-tree, *unimplemented body* (see comments on 467-489) |
| `PeerAddressDiscovery::detect_nat_type` | `core/src/transport/nat.rs:96-174` | Calls `request_address_reflection` on swarm |
| `AddressReflectionRequest` / `Response` | `core/src/transport/reflection.rs:24-87` | Wire protocol already defined |
| `AddressReflectionService::handle_request` | `core/src/transport/reflection.rs:129-148` | Server side (returns observed source addr) |
| Hole-punch probe packet format | `core/src/transport/nat.rs:473-482` | documented as `0x48505443 ("HPTC")` magic — *not yet sent on the wire* |

**Key insight:** the swarm already binds ephemeral TCP/QUIC ports but pins the WS
listener to `9002`. Most of the "static port" problem is therefore concentrated in
(a) the WS listener, (b) the CLI's `--listen` default, and (c) the *client* ports
that the Android/iOS probes scan.

### 2.2 CLI — `cli/src/`

| What | File:line | Value |
|---|---|---|
| `Config::default().listen_port` | `cli/src/config.rs:71` | `9000` |
| `Relay` subcommand `--listen` default | `cli/src/main.rs:184` | `"/ip4/0.0.0.0/tcp/9001"` |
| `Relay` subcommand `--http_port` default | `cli/src/main.rs:187` | `9000` |
| Hardcoded WS bridge print | `cli/src/main.rs:2320` | `"WS Bridge:    ws://0.0.0.0:9002 (libp2p-ws)"` |
| Bootstrap peers hardcoded | `cli/src/bootstrap.rs:28, 183` | `/ip4/34.135.34.73/tcp/9001/p2p/12D3KooWET…` |
| Fallback dial port list | `cli/src/main.rs:1445` | `[9001, 4001, 9000, 8000]` |
| Port-random helper | `cli/src/main.rs:1180-1182` | `if config.listen_port == 0 { 9000 }` ← *oddly re-snaps to 9000* |
| Existing related ticket | `HANDOFF/todo/[VALIDATED]_P1_CLI_028_Config_Listen_Port_Stale_vs_Actual_Port_9101.md` | Confirms the same bug class (config says 9000, daemon bound 9101) |

### 2.3 Android — `android/app/src/main/java/com/scmessenger/android/transport/`

| What | File:line | Value |
|---|---|---|
| `MdnsServiceDiscovery.serviceType` | `MdnsServiceDiscovery.kt:65` | `"_p2p._udp"` (libp2p default) |
| `MdnsServiceDiscovery.servicePort` | `MdnsServiceDiscovery.kt:67` | `9001` (hardcoded) |
| mDNS TXT `dnsaddr` literal | `MdnsServiceDiscovery.kt:405` | `"/ip4/0.0.0.0/tcp/$servicePort/p2p/$localId"` |
| `SubnetProbe.targetPorts` | `SubnetProbe.kt:71` | `intArrayOf(9001, 9002)` |
| SubnetProbe doc-comment | `SubnetProbe.kt:33-34` | "The libp2p swarm listens on TCP 9001 and the relay/WS on 9002" |
| `WifiDirectTransport.SERVICE_TYPE` | `WifiDirectTransport.kt:453` | `"_scmessenger._tcp"` (a *different* type than `MdnsServiceDiscovery`!) |
| `TransportManager` comment on probing | `TransportManager.kt:55, 125` | "/24 subnets for open port 9001 (libp2p TCP) / 9002 (WS relay)" |
| Existing related ticket | `HANDOFF/todo/P1_ANDROID_LAN_DISCOVERY_REPAIR.md` | root cause = different subnets + service-type mismatch |

**Important finding:** the Android codebase contains *two* mDNS service-type
identities that are not equal to each other:
- `MdnsServiceDiscovery.kt:65` → `_p2p._udp` (NsdManager, libp2p-style)
- `WifiDirectTransport.kt:453` → `_scmessenger._tcp` (DNS-SD TXT records)

The CLI daemon advertises via *libp2p-mdns* (`_p2p._udp`), so it is only visible
to `MdnsServiceDiscovery`. The `WifiDirectTransport` is a separate DNS-SD
implementation used for WiFi-Direct group-owner service records. We will preserve
this distinction in the migration.

### 2.4 iOS — `iOS/SCMessenger/SCMessenger/Transport/`

| What | File:line | Value |
|---|---|---|
| `mDNSServiceDiscovery.serviceType` | `mDNSServiceDiscovery.swift:35` | `"_scmessenger._tcp"` (does NOT match Android's `_p2p._udp`) |
| `startAdvertising(port:)` signature | `mDNSServiceDiscovery.swift:71` | accepts a `port: Int32` ← already a parameter! |
| `port` extracted from socket | `mDNSServiceDiscovery.swift:163, 174, 179` | from `sockaddr_in` / `sockaddr_in6` ← reads back the actual port |

**iOS is already structured for dynamic ports** — `startAdvertising(port:)` takes
the port as a parameter and reads the actual bound port back from the socket. The
Kotlin equivalent (`MdnsServiceDiscovery.servicePort` = 9001) does not.

### 2.5 WASM — `wasm/`

The WASM thin client connects to whatever port the bridge exposes. It does not
make its own bind/dial decisions. No changes required for this migration.

### 2.6 BLE well-known UUIDs (kept for reference — not affected)

| UUID | File:line | Purpose |
|---|---|---|
| `GATT_SERVICE_UUID = 0x0000_DF01_…` | `core/src/transport/ble/gatt.rs:11` | GATT service (BLE) |
| `BLE_BEACON_SERVICE_UUID = 0xDF01` | `core/src/transport/ble/beacon.rs:16` | Legacy beacon (BLE) |
| `GattCharacteristic::Write` | `core/src/transport/ble/gatt.rs:34` | `0xDF02` |
| `GattCharacteristic::Notify` | `core/src/transport/ble/gatt.rs:35` | `0xDF03` |
| `GattCharacteristic::Status` | `core/src/transport/ble/gatt.rs:36` | `0xDF04` |
| `L2capPsm::SCMessenger` | `core/src/transport/ble/l2cap.rs:12-13` | `0x0025` |

These are *BLE GATT identifiers*, not TCP/UDP ports. They do not need to change
in this migration. We do, however, recommend encoding the current TCP/UDP port
inside the BLE GATT advertisement's `manufacturer data` field (see Phase 2) so
phones that find a peer over BLE can immediately dial it on TCP.

### 2.7 Other "static" references worth knowing about

| What | File:line | Value |
|---|---|---|
| Android `adb reverse` target | (per `P1_ANDROID_LAN_DISCOVERY_REPAIR.md:12`) | `localhost:9002` (WS bridge) |
| Windows relay wrapper cmd | (per `P1_ANDROID_LAN_DISCOVERY_REPAIR.md:13`) | ports `9100/9101` |
| CLI banner `WS Bridge:` literal | `cli/src/main.rs:2320` | `9002` |
| Config fallback (random → snap) | `cli/src/main.rs:723-726` | snaps `0` back to `9000` |

---

## 3. Goals & Constraints

### 3.1 What "dynamic-port-aware" means for SCMessenger

Per Lucas's directive, we interpret this as a **layered** model:

1. **Discovery protocol is port-agnostic.** A peer that hears about another peer
   (mDNS, BLE, SubnetProbe, DHT) learns the *current* listen port as part of the
   discovery payload, never assumes `9001`.
2. **Listen port is ephemeral by default, configurable per-launch.** Every CLI
   instance picks a port from a configured range (default `9000–9100`) and writes
   the actual port into its config file so other tools (Android) can read it.
3. **Backwards compatible.** A v0.2.x peer that only knows about port 9001 must
   still be discoverable from a v0.3.x peer that has moved to ephemeral ports
   *and* the v0.3.x peer must still be able to fall back to 9001 when the mDNS
   payload is missing.
4. **The kernel/OS still picks the port when possible** (port 0 in `bind(2)`),
   eliminating "in use" errors and giving every CLI run a unique fingerprint.
5. **Self-NAT-mapping via sender-spoof liveness probe** (see §4-C) replaces the
   need for a third-party STUN server.

### 3.2 Concrete user-stated requirements (paraphrased)

| User said | We interpret as |
|---|---|
| "move out of static ports, at least for the discovery protocol" | §4-A: bind port 0, advertise via mDNS TXT |
| "spoof sender and open sessions that would be routed intentionally back to a node to test a response" | §4-C: a liveness probe that uses *the kernel's own response* as a NAT-mapping oracle |
| "Custom UDP or TCP packet manipulation" | §4-C: a `LivenessProbe` with a 32-byte magic, 16-byte nonce, and `tokio::net::UdpSocket` |
| "migrate full app to non-static port assignment" | Phases 0–3 below |
| "Perhaps just core?" | Phase 1 is core-only and unblocks Lucas's current WSL/Android issue without touching the platforms |

### 3.3 Trade-offs

| Trade-off | Static port (today) | Dynamic port (proposed) | Mitigations |
|---|---|---|---|
| **LAN discovery across subnets** | breaks on WSL↔Android (current bug) | works on any subnet that mDNS/TXT can reach | None — this is the win |
| **Firewall rules** | easy: "open 9001" | harder: "open 9000–9100" or use mDNS TXT | Phase 1 keeps the 9000–9100 range as a documented IANA-ish private range |
| **Debuggability** | `netstat` shows `*:9001` | port changes per launch | Persist actual port in `config.json` (see ticket `P1_CLI_028` for the pattern) |
| **Censorship resistance** | trivially blocked by port | port-hoppable (Phase 4) | Range-based randomization makes blanket port-blocks impractical |
| **Battery on mobile** | one TCP socket, one BLE advertise | multiple ephemeral sockets + UDP probes | Bound concurrent listeners to `4` (see Phase 2) |
| **Determinism for tests** | predictable | flaky | `--port-range 9000-9000 --port 9000` flag forces single-port mode for CI |
| **Wasted file descriptors** | ~5 | ~15 if multi-port | Linux default `fs.file-max` is 1.4M — non-issue on desktop, mobile gets 32K limit; we cap at 8 listeners |

---

## 4. Three Concrete Approaches (with code sketches)

All sketches target the file `core/src/transport/discovery.rs` (or its
sibling `core/src/transport/multiport.rs`). They are written to compile in
concept, but the surrounding types (`SwarmHandle`, `TransportType`,
`MdnsTxtRecord`) are abbreviated for clarity. Files marked `// FOR DISCUSSION`
are pseudo-code that needs a real prototype.

### 4.A Port-range allocation with kernel-assisted binding

**Premise.** Let the kernel pick a port in a configured range, advertise that
port via mDNS TXT, and read it back from the swarm listeners.

**Platform support matrix** (cross-referenced with `core/src/transport/multiport.rs`):

| Platform | SO_REUSEADDR | SO_REUSEPORT | Behavior |
|---|---|---|---|
| Linux (≥ 3.9) | yes | yes | Two sockets can bind the same port; kernel load-balances |
| Windows 10+ | yes (since Win 10 1709) | partial (Win 11 22H2) | Use `SO_REUSEADDR` only; avoid `SO_EXCLUSIVEADDRUSE` |
| macOS (12+) | yes | yes | `SO_REUSEPORT` available since 10.10 |
| Android (N+) | yes | yes (kernel 4.4+) | Same as Linux |
| iOS (15+) | yes | yes | Same as macOS; sandbox limits which ports you can `bind` to |

**Code sketch** — new function in `core/src/transport/multiport.rs`:

```rust
// core/src/transport/multiport.rs (NEW)
pub struct AllocatedPort {
    pub socket: tokio::net::TcpListener,
    pub port: u16,
    pub interface: std::net::IpAddr,
}

pub async fn bind_ephemeral(
    range: std::ops::RangeInclusive<u16>,
) -> Result<AllocatedPort, std::io::Error> {
    use socket2::{Domain, Socket, Type};
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};

    // 1) Try kernel-assigned first (port 0) — preferred for uniqueness.
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    #[cfg(all(unix, not(target_os = "macos")))]
    socket.set_reuse_port(true)?;            // Linux+Android
    socket.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0).into())?;
    socket.listen(128)?;
    socket.set_nonblocking(true)?;

    let local: SocketAddr = socket.local_addr()?;
    let port = local.port();
    if range.contains(&port) {
        let listener: tokio::net::TcpListener = socket.into();
        return Ok(AllocatedPort { socket: listener, port, interface: local.ip() });
    }

    // 2) Fall back: walk the configured range.
    for candidate in range.clone() {
        let s = Socket::new(Domain::IPV4, Type::STREAM, None)?;
        s.set_reuse_address(true)?;
        if s.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), candidate).into()).is_ok()
            && s.listen(128).is_ok()
        {
            s.set_nonblocking(true)?;
            return Ok(AllocatedPort {
                socket: s.into(),
                port: candidate,
                interface: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            });
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "no port available in range"))
}
```

**Integration with `swarm.rs`** (where the WS listener is currently hardcoded):

```rust
// core/src/transport/swarm.rs:1898-1906 — REPLACE hardcoded "/ip4/0.0.0.0/tcp/9002/ws"
let ws_port = match multiport::bind_ephemeral(9000..=9100).await {
    Ok(p) => {
        // Hand the listener to libp2p via TcpListener::from_std(...)
        let std_listener = p.socket.into_std()?;
        std_listener.set_nonblocking(false)?;
        let _ = swarm.listen_on(
            format!("/ip4/0.0.0.0/tcp/{}/ws", p.port).parse()?
        );
        tracing::info!("✓ WS bridge on ephemeral port {}", p.port);
        p.port
    }
    Err(e) => {
        tracing::warn!("WS bridge failed to bind ephemeral, using 9002: {}", e);
        swarm.listen_on("/ip4/0.0.0.0/tcp/9002/ws".parse()?)?;
        9002
    }
};
state.ws_port.set(ws_port);  // expose to discovery module for mDNS TXT
```

**mDNS TXT update** (`MdnsServiceDiscovery.kt:65-67` becomes):

```kotlin
// android/.../transport/MdnsServiceDiscovery.kt
private val serviceType = "_p2p._udp"   // unchanged
private val servicePort: Int
    get() = scmessengerCore.getState().wsPort   // 0 → ephemeral, 9000..9100 → static
```

### 4.B UDP hole-punching with a rendezvous server

**Premise.** Reuse the existing bootstrap relay (`34.135.34.73:9001` per
`cli/src/bootstrap.rs:28`) as a rendezvous coordinator. When A wants to reach B,
both send a UDP probe to a predicted port range; the relay only forwards the
*addresses*, never the payload, so we get full STUN-equivalent behavior
without a separate STUN service.

**Existing primitives in the codebase:**

- `core/src/transport/nat.rs::NatTraversal` — already orchestrates `start_hole_punch` / `send_hole_punch_probes` (lines 388-492). The methods exist and compile but the body is a stub (the comment block at `nat.rs:467-489` documents what the implementation should do; the actual UDP send is missing).
- `core/src/transport/reflection.rs` — request/response schema for "what's my IP:port?" — already wired through `request_address_reflection` on `SwarmHandle` (`nat.rs:128`).
- `core/src/transport/manager.rs` (TBD) — would own the rendezvous socket.

**Code sketch** — fills in the missing body of `send_hole_punch_probes`:

```rust
// core/src/transport/nat.rs:440-492 — REPLACE
async fn send_hole_punch_probes(&self, attempt_key: &str) -> Result<(), NatTraversalError> {
    use tokio::net::UdpSocket;
    use rand::RngCore;

    let mut attempts = self.hole_punch_attempts.write();
    let attempt = attempts.get_mut(attempt_key)
        .ok_or(NatTraversalError::HolePunchFailed("attempt vanished".into()))?;
    attempt.status = HolePunchStatus::Attempting;

    // 1) Bind a UDP socket on an ephemeral port; read back the assigned port.
    let socket = UdpSocket::bind("0.0.0.0:0").await
        .map_err(|e| NatTraversalError::HolePunchFailed(e.to_string()))?;
    let local_observed: SocketAddr = socket.local_addr()
        .map_err(|e| NatTraversalError::HolePunchFailed(e.to_string()))?;

    // 2) Build the HPTC probe packet (matches the format documented at nat.rs:473-482).
    let mut pkt = Vec::with_capacity(60);
    pkt.extend_from_slice(&0x48505443_u32.to_be_bytes());     // magic
    let mut nonce = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    pkt.extend_from_slice(&nonce);
    pkt.extend_from_slice(&current_unix_timestamp().to_be_bytes());
    // (signature omitted for sketch — fill in nat.rs::HolePunchAttempt.local_peer_id sign)

    // 3) Fire-and-await up to N probes. The remote peer is doing the same.
    for _ in 0..10 {
        socket.send_to(&pkt, attempt.remote_external_addr).await
            .map_err(|e| NatTraversalError::HolePunchFailed(e.to_string()))?;

        let mut buf = [0u8; 64];
        match tokio::time::timeout(
            Duration::from_millis(500),
            socket.recv_from(&mut buf)
        ).await {
            Ok(Ok((n, src))) if n >= 4 && &buf[..4] == b"HPTC" => {
                attempt.status = HolePunchStatus::Success;
                tracing::info!("Hole-punch success: {} echoed by {}", attempt_key, src);
                return Ok(());
            }
            _ => continue,
        }
    }
    attempt.status = HolePunchStatus::Failed;
    Err(NatTraversalError::HolePunchFailed("exhausted probes".into()))
}
```

**Rendezvous protocol over libp2p** — extend `reflection.rs`:

```rust
// core/src/transport/reflection.rs — NEW message variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolePunchRendezvousRequest {
    pub request_id: [u8; 16],
    pub target_peer_id: PeerId,
    pub my_observed_address: SocketAddr,   // from peer reflection
    pub prefer_udp_port: Option<u16>,      // e.g. 9000..=9100 — hint to the rendezvous
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolePunchRendezvousResponse {
    pub request_id: [u8; 16],
    pub target_observed_address: SocketAddr,    // the *other* peer's address
    pub suggested_local_port: u16,             // rendezvous guesses the NAT port
    pub nat_type_hint: NatType,
}
```

**Why not just use libp2p's `Circuit Relay v2` + `AutoNAT`?**
- AutoNAT (`/api/listeners` shows it works in the running build, see ticket `P1_CLI_026`) only reports the *external* address — it does not help with symmetric NATs.
- Circuit Relay is great for guaranteed connectivity but adds a hop. The hole-punch is the *fast path*; relay is the fallback (already implied in `internet.rs::RelayMode::Client`).
- We use both: relay for signalling, hole-punch for the data path. This is exactly what Tailscale and ZeroTier do.

**Reference:** [Tailscale's design doc on STUN and NAT traversal](https://tailscale.com/blog/how-nat-traversal-works/) and [ZeroTier's `ztnc` control-plane](https://github.com/zerotier/ZeroTierOne/blob/main/node/Salsa20.hpp). We borrow the "rendezvous predicts both peers' NAT ports, both punch simultaneously" pattern.

### 4.C Custom UDP beacon with sender-spoof for liveness tests

**Premise.** Lucas described it as: *"spoof sender and open sessions that would
be routed intentionally back to a node to test a response."* This is the
**liveness probe** pattern and is the most novel part of this design.

**Why it matters:** the *response* to a probe tells you (a) the peer is alive,
(b) what port the kernel NAT mapped you to, and (c) what the round-trip time
is. If you can do this with a peer that is itself on the same NAT as you, you
have built a *self-STUN* that does not require an external server.

**How it works:**

1. A binds a UDP socket on `0.0.0.0:SRC` (kernel-assigned).
2. A sends a packet to B's discovered address:port with a magic + nonce.
3. B receives the packet, reads the *source* address (the IP:port A used),
   and **immediately echoes** the packet back to A:SRC — *regardless of what
   the source IP says* (in practice, on most NATs, the kernel routes the
   response to whichever local socket last sent to B, which is A:SRC).
4. A waits on its socket. If it gets a response within `T` ms:
   - The peer is alive.
   - The packet's actual destination port (as observed by B and echoed back) is
     the *NAT-mapped* port that A is currently using. This is your **STUN-like
     answer** without an external STUN server.

**Code sketch** — new file `core/src/transport/liveness.rs`:

```rust
// core/src/transport/liveness.rs  (NEW)
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use rand::RngCore;

const MAGIC: [u8; 4] = *b"LPRB";   // "Liveness PRoBe"

#[derive(Debug, Clone, Copy)]
pub struct ProbeResult {
    pub rtt: Duration,
    pub observed_local_port: u16,   // what the kernel NAT mapped A to
    pub responder_addr: SocketAddr,
}

pub struct LivenessProbe {
    socket: UdpSocket,
    local_addr: SocketAddr,
}

impl LivenessProbe {
    /// Create a probe socket on an ephemeral port. Call once at startup.
    pub async fn new() -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let local_addr = socket.local_addr()?;
        Ok(Self { socket, local_addr })
    }

    /// Send a liveness probe to `target` and wait for the echo.
    /// Returns `Ok(ProbeResult)` if the peer echoes within `timeout`.
    pub async fn probe(
        &self,
        target: SocketAddr,
        timeout: Duration,
    ) -> Result<ProbeResult, ProbeError> {
        // 1) Build the probe packet.
        let mut nonce = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let mut pkt = Vec::with_capacity(24);
        pkt.extend_from_slice(&MAGIC);
        pkt.extend_from_slice(&nonce);

        // 2) Send and time the round-trip.
        let started = std::time::Instant::now();
        self.socket.send_to(&pkt, target).await?;

        // 3) Read up to N packets until we see our nonce echoed back.
        let mut buf = [0u8; 64];
        loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(ProbeError::Timeout);
            }
            match tokio::time::timeout(remaining, self.socket.recv_from(&mut buf)).await {
                Ok(Ok((n, responder))) if n >= 20 && buf[..4] == MAGIC && buf[4..20] == nonce => {
                    // The probe packet's source port as seen by the responder
                    // is `responder` (that's the address B observed A coming from).
                    // That IS A's NAT-mapped port.
                    return Ok(ProbeResult {
                        rtt: started.elapsed(),
                        observed_local_port: responder.port(),
                        responder_addr: responder,
                    });
                }
                Ok(Ok(_)) => continue,    // some other packet, keep listening
                Ok(Err(e)) => return Err(ProbeError::Io(e)),
                Err(_) => return Err(ProbeError::Timeout),
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("probe timed out")]
    Timeout,
    #[error("io: {0}")]
    Io(std::io::Error),
}
```

**Integration with `discovery.rs`** — register a tiebreaker:

```rust
// core/src/transport/discovery.rs — add to DiscoveryConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub mode: DiscoveryMode,
    pub advertise_protocols: bool,
    pub accept_unknown_peers: bool,
    // NEW:
    pub liveness_probe_timeout_ms: u64,     // default 800
    pub liveness_probe_fanout: usize,        // default 3 (probe top 3 candidates)
}
```

**Honest caveat about sender-spoofing:** the user said *"spoof sender"*. On
modern OSes (Linux ≥ 4.4, Windows ≥ 10, macOS ≥ 10.13) you generally *cannot*
spoof the source address of a UDP datagram sent from an unprivileged socket
unless the socket is bound to a raw interface. **We should NOT rely on
source-address spoofing.** What the probe *actually* does is:

1. Force the kernel to create a NAT mapping by sending outbound.
2. Learn the mapping by observing the *source* address that B sees.

This is exactly the STUN binding-request/binding-response pattern, with the
peer acting as a STUN server. We document this in the sketch above (the
"observed_local_port" is the NAT port that the responder *thinks* A is using,
which is what STUN returns).

If Lucas really wants source-spoofing (e.g. to test a firewall rule that
forbids certain source IPs), that needs `CAP_NET_RAW` and is **out of scope**
for v0.3 — but we capture the requirement here and flag it for v0.4.

---

## 5. Recommended Migration Path (phased)

### Phase 0: Foundation (1–2 weeks, **no behavior change**)

**Goal:** expose ports as data, not constants. Add the migration scaffolding
without breaking anything. This phase is **the minimum viable unblock** for
Lucas's WSL↔Android issue.

**Tasks:**

1. **`core/src/transport/multiport.rs`** — add `bind_ephemeral()` (sketch in §4-A).
2. **`cli/src/config.rs`** — add `port_range: Option<(u16, u16)>` to `NetworkConfig`.
   Default `Some((9000, 9100))`. **Do not change `listen_port` default of `9000` yet.**
3. **`cli/src/main.rs`** — accept `--port-range 9000-9100` flag on `start` and `relay`.
4. **`android/.../transport/SubnetProbe.kt:71`** — read `targetPorts` from a
   `SharedPreferences` key (default `[9001, 9002]`, configurable from the in-app
   settings screen — but kept `[9001, 9002]` for v0.2.1 to avoid breaking the
   existing P1 ticket that already merged in commit `87d1ef61`).
5. **Metrics** — add counters: `port_collision_count`, `port_in_use_count`,
   `ephemeral_bind_attempts`. Expose via `/api/health`.
6. **Documentation** — update `BOOTSTRAP.md` and `CLI_WEBRTC_BRIDGE_PLAN.md`
   with a "Port model" section.

**Acceptance gate:** all existing tests pass; no defaults change. The CLI
*accepts* `--port-range` but ignores it. Code review-only.

**Subagent ticket:**
`HANDOFF/todo/[VALIDATED]_P0_PORT_RANGE_FOUNDATION.md` (we will create this in
the same commit as the research document).

---

### Phase 1: Ephemeral CLI port (1 week)

**Goal:** the CLI's P2P listener and the WS bridge both bind to ephemeral
ports, advertise the actual port, and write it back to `config.json`.

**Tasks:**

1. **`core/src/transport/swarm.rs:1898-1906`** — replace hardcoded `9002/ws`
   with `bind_ephemeral(9000..=9100)` (sketch in §4-A).
2. **`core/src/transport/swarm.rs:1836-1873`** — same treatment for the TCP
   listener (currently uses `tcp/0`, but the *advertised* port in mDNS TXT
   should match what we observe from `swarm.listeners()`).
3. **`cli/src/main.rs:1180-1182`** — remove the `if config.listen_port == 0
   { 9000 }` snap. Let `0` mean "kernel-assigned" end-to-end.
4. **`cli/src/main.rs::cmd_start`** — after binding, read `swarm.listeners()`,
   find the first non-loopback `/ip4/.../tcp/<port>`, and write it to
   `config.listen_port`. This is the same pattern as ticket
   `[VALIDATED]_P1_CLI_028_Config_Listen_Port_Stale_vs_Actual_Port_9101.md` —
   we will merge with that ticket.
5. **mDNS TXT** — extend the `dnsaddr` attribute (currently
   `MdnsServiceDiscovery.kt:405` `"/ip4/0.0.0.0/tcp/$servicePort/p2p/$localId"`)
   to be the actual `/ip4/<local-ip>/tcp/<actual-port>/p2p/<localId>`. This is
   a *behaviour change* for the daemon, so the rolling deploy needs to be staged:
   first deploy the new TXT format, then the new bind logic, with a 1-week
   overlap.
6. **CLI banner** — `cli/src/main.rs:2320` "WS Bridge: ws://0.0.0.0:9002"
   becomes a function call: `WS Bridge: ws://0.0.0.0:{ws_port}`.

**Acceptance gate:**
- `cargo run --bin scmessenger-cli start` on a clean machine listens on a
  port in `9000–9100` (printed on stdout).
- The port is written to `config.json` on disk.
- Restart picks the *same* port (reads it back from config) — predictable
  behaviour, no flapping.
- An older v0.2.x Android client that only probes `9001/9002` *still* finds
  the new CLI (because we *also* bind 9001 if available — see below).
- Backwards-compat shim: if `config.listen_port` is `9000..=9100`, *also* try
  to bind port `9001` as a fallback. If that succeeds, advertise *both* in mDNS
  TXT. Older clients dial 9001; newer clients dial the ephemeral.

**Why backwards compat matters:** v0.2.1 is shipping to Lucas's Android
device. We cannot break v0.2.x peers. The shim is a 30-line addition to
`multiport::generate_listen_addresses` (add `9001` to the candidates if
`enable_legacy_9001 = true`).

**Subagent ticket:** `HANDOFF/todo/[VALIDATED]_P1_CLI_DYNAMIC_PORT_LISTEN.md`.

---

### Phase 2: Discovery protocol extension (2–3 weeks)

**Goal:** every discovery mechanism (mDNS, BLE, SubnetProbe, DHT) carries the
*current* listen port as part of its payload, not as a hardcoded constant.

**Tasks:**

1. **mDNS TXT extension** — add a `port-range` attribute to the daemon's mDNS
   TXT record: `port-range=9000-9100`. Android's `MdnsServiceDiscovery.kt:152-160`
   parses the existing `peer-id`/`p2p`/`dnsaddr` attributes; add parsing for
   `port-range` and store it in a new `MdnsTxtRecord.portRange: (u16, u16)?` field.
2. **mDNS TXT for Android** — `MdnsServiceDiscovery.kt:405` currently writes
   `dnsaddr=/ip4/0.0.0.0/tcp/9001/p2p/<id>`. The `0.0.0.0` is a placeholder
   and the `9001` is a literal. Replace with the local IP and the actual TCP
   port (read from `scmessenger_core::get_state().tcp_port`).
3. **Android `SubnetProbe.kt:71`** — replace `intArrayOf(9001, 9002)` with the
   resolved `port-range` (default `[9001, 9002]` if no mDNS TXT seen). **Honor
   the configured range; do not probe every port in 9000–9100** (that's 101
   ports × /24 subnets × 254 hosts = ~25k TCP attempts per cycle — battery
   killer). Probe a *sample*: `[range.start, range.end, 9001, 9002, 4001]`
   deduped.
4. **BLE manufacturer data** — extend the BLE beacon in
   `core/src/transport/ble/beacon.rs` to include the current TCP port (2
   bytes) plus a CRC8. The BLE service UUID stays `0xDF01`. The change is
   backward-compatible: scanners that don't know about the new field just
   ignore the extra bytes.
5. **Core RPC** — new `port_announce { listen_addrs: Vec<Multiaddr>,
   port_range: (u16, u16), nonce: u64 }` message on the libp2p
   request-response protocol. Other peers cache it; if they see the same
   peer announce a new range (e.g. port hop), they update their cache.
6. **iOS** — `mDNSServiceDiscovery.swift:71` `startAdvertising(port:)` already
   takes a port. Wire the port source to a new `BridgeConfig.tcpPort` and
   pass it from the Swift side. *iOS is already 80% done for this phase.*

**Acceptance gate:**
- Two CLIs on the same LAN see each other via mDNS TXT and dial the ephemeral
  port, not 9001.
- Android SubnetProbe on a foreign subnet (e.g. 192.168.0.x probing 172.26.x.x)
  finds the daemon within 30 s.
- A BLE-discovered peer can be dialed on TCP without a separate discovery
  step.
- A peer that hops ports (kills + restarts CLI) is rediscovered within
  `min(mDNS TTL, 60s)` — no manual `peer remove` + `peer add`.

**Subagent ticket:** `HANDOFF/todo/[VALIDATED]_P1_DISCOVERY_PORT_RANGE_NEGOTIATION.md`.

---

### Phase 3: UDP liveness probe (2–3 weeks)

**Goal:** when mDNS, BLE, and SubnetProbe all fail, the liveness probe from §4-C
is the last-mile tiebreaker — and it doubles as a *self-STUN* oracle.

**Tasks:**

1. **`core/src/transport/liveness.rs`** — implement the `LivenessProbe` struct
   from §4-C. Wire it into `core/src/transport/manager.rs` as a long-lived
   background task.
2. **`DiscoveryConfig::liveness_probe_*`** — add config (sketched in §4-C).
3. **Probe routing** — when `discovery_transport()` returns `Unknown` or
   `OtherLAN` (per the new enum in `[VALIDATED]_P1_CLI_030_…md`), fall back to
   liveness probe. Probe *up to* `fanout` candidates in parallel.
4. **Self-mapping** — periodically (every 60 s) probe a known-good peer
   (bootstrap node or any connected peer) to learn the kernel's NAT mapping.
   Cache the answer in `NatTraversal::external_address` — this is what
   `request_address_reflection` returns when the swarm's observed address
   differs from the liveness-derived one (suggests symmetric NAT).
5. **Integration with Phase 2's `port_announce` RPC** — the liveness result
   gets included in subsequent `port_announce` messages, so other peers can
   learn *our* NAT port without asking us.
6. **iOS** — iOS does not allow raw UDP sockets in the same way (sandbox
   restriction). The liveness probe is server-side only on iOS — the iOS
   client answers probes, it does not send them. This is fine: the
   liveness probe is a *core*-level primitive.

**Acceptance gate:**
- A node behind a Carrier-Grade NAT (CGNAT) on a phone LTE connection can
  report its external port within 5 s of startup.
- A node with no mDNS/BLE/SubnetProbe reachability (e.g. two laptops on
  the same coffee-shop WiFi with multicast disabled) is discoverable in
  < 15 s via liveness probe alone.
- Symmetric NAT detection: a node whose observed port changes between
  probes is correctly classified `Symmetric` in `nat.rs` (currently the
  logic at `nat.rs:154-170` uses address equality; we extend it to
  port equality).

**Subagent ticket:** `HANDOFF/todo/[VALIDATED]_P2_LIVENESS_PROBE.md`.

---

### Phase 4: NAT traversal (optional, 4+ weeks, defer to v0.3)

**Goal:** full UDP hole-punching across NATs using the rendezvous protocol
sketched in §4-B.

**Tasks:**

1. **Fill in `nat.rs::send_hole_punch_probes`** (sketch in §4-B). This is the
   single biggest code change in the migration — ~300 LoC of UDP socket
   management, retry logic, and timing.
2. **Rendezvous protocol** — extend `reflection.rs` with
   `HolePunchRendezvousRequest/Response` (sketch in §4-B). Wire it through a
   libp2p request-response behaviour.
3. **Relay-fallback** — keep `internet.rs::RelayMode::Client` working as
   the always-fallback (it already is). Hole-punch is the fast path.
4. **Re-test across the WSL stack** — Lucas's specific blocker is
   WSL↔Android↔Windows. After Phase 4, this should work even across
   Hyper-V's NAT layer (which is full-cone, so hole-punch should succeed).
5. **iOS/Android background restrictions** — iOS suspends UDP sockets in
   the background. Document the constraint: liveness probe is
   foreground-only on mobile. Relay fallback is the only always-on option.

**Acceptance gate:**
- Two nodes, one on home WiFi behind a typical ISP NAT, one on LTE behind
  CGNAT, establish a direct UDP connection in < 10 s, > 70% of the time
  (current industry baseline; Tailscale reports ~80% — **unknown — needs
  measurement** for SCMessenger).
- When the hole-punch fails, the relay fallback engages within 1 s and the
  user sees a "relayed" indicator in the UI.

**Subagent ticket:** `HANDOFF/todo/[VALIDATED]_P3_NAT_TRAVERSAL.md`.

---

## 6. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **Corporate firewall blocks all high ports** | High | High | Phase 1 keeps the 9000–9100 range as a private-RFC-1918-style default; we also support `--port 443` and `--port 80` for restricted networks (already in `multiport::COMMON_PORTS`) |
| **WSL Hyper-V NAT eats multicast** | Already broken today | Medium | Phase 2+3 provide SubnetProbe + liveness probe as non-multicast fallbacks |
| **Debugging complexity: port is no longer constant in logs** | High | Medium | Phase 0 mandates: every log line that mentions a port also includes the boot timestamp + a short node ID, so `grep "boot=2026-06-05T11:30Z nodeId=abc" log.txt` works |
| **Backwards compat with v0.2.x peers** | Certain (we ship v0.2.1 to Lucas's phone) | High | Phase 1 backwards-compat shim: daemon binds both ephemeral port AND 9001 (if free); mDNS TXT carries both; clients pick the newer one |
| **TIME_WAIT exhaustion from ephemeral port churn** | Low (Linux TIME_WAIT is 60s, ephemeral pool is 28K) | Low | Phase 0 metrics expose `time_wait_count`; alert at 5K |
| **Per-port FD overhead on mobile** | Medium (Android RLIMIT_NOFILE defaults to 32K) | Medium | Phase 1 caps concurrent listeners at 8; mobile uses 2 (TCP + UDP/QUIC) |
| **mDNS TXT record size limit (1300 bytes per ticket P1_CLI_024)** | Already happening | High | Phase 2 *also* fixes that ticket by stripping circuit addresses from the mDNS TXT (not the listen address, but the cached p2p-circuit chains) |
| **Symmetric NAT defeats hole-punch** | ~15% of home NATs | Medium | Phase 4 has a relay-fallback path; success rate target 70%, relay picks up the rest |
| **Liveness probe DoS** (peer A floods peer B with probes) | Low | Medium | Rate-limit: 1 probe per peer per 5 s, max 3 in flight; nonce must match within 1 s |
| **Source-spoofing request can't be implemented unprivileged** | Certain | Low (it was a *description*, not a requirement) | The §4-C sketch is the actually-implementable liveness probe; full IP-spoofing is a v0.4+ feature that needs `CAP_NET_RAW` |
| **WSL↔Android across Hyper-V is unrecoverable in v0.2.1** | Already broken | High | Phase 0 alone is the unblock — SubnetProbe and ephemeral ports. Documented in `P1_ANDROID_LAN_DISCOVERY_REPAIR.md` |

---

## 7. Acceptance Criteria

A phase is "done" when **all** of the following are true:

### Phase 0

- [ ] `core/src/transport/multiport.rs::bind_ephemeral()` exists and is unit-tested
- [ ] `cli/src/main.rs --port-range 9000-9100` is accepted (ignored) on `start` and `relay`
- [ ] `cargo test -p scmessenger-core transport::` passes
- [ ] `cargo test -p scmessenger-cli` passes
- [ ] `HANDOFF/research/2026-06-05_DYNAMIC_PORT_DISCOVERY_RESEARCH.md` is committed
- [ ] `/api/health` exposes `port_collision_count`, `port_in_use_count`, `ephemeral_bind_attempts`

### Phase 1

- [ ] CLI's TCP and WS listeners bind to ports in `9000–9100` (default)
- [ ] `config.listen_port` is updated to the actual bound port on exit
- [ ] Restart picks the same port (predictable, no flapping)
- [ ] v0.2.x Android client on the same LAN still finds the v0.3 CLI (legacy 9001 fallback)
- [ ] mDNS TXT for the daemon includes the actual port (verified via `avahi-browse -rt _p2p._udp`)
- [ ] The port-staleness warning from `[VALIDATED]_P1_CLI_028_…md` no longer fires

### Phase 2

- [ ] mDNS TXT includes a `port-range=NNNN-NNNN` attribute
- [ ] Android `MdnsServiceDiscovery` parses `port-range` and passes it to `SubnetProbe`
- [ ] `SubnetProbe` probes `[range.start, range.end, 9001, 9002, 4001]` deduped
- [ ] BLE GATT manufacturer data includes the current TCP port (2 bytes) + CRC8
- [ ] Two Android phones on different subnets find each other within 60 s of app open
- [ ] CLI hopping ports (kill + restart) is rediscovered by Android within 60 s

### Phase 3

- [ ] `LivenessProbe` struct exists in `core/src/transport/liveness.rs` and is unit-tested
- [ ] A node behind CGNAT can report its external port within 5 s
- [ ] When mDNS + BLE + SubnetProbe all fail, the liveness probe rediscovers the peer
- [ ] Symmetric NAT is correctly classified when the observed port changes between probes
- [ ] Probe is rate-limited to 1/5s/peer; nonce TTL is 1 s
- [ ] The `DiscoveryTransport` enum from `[VALIDATED]_P1_CLI_030_…md` is extended with a `Liveness` variant

### Phase 4

- [ ] `nat.rs::send_hole_punch_probes` is implemented and unit-tested with a mock responder
- [ ] Two nodes across CGNAT establish direct UDP > 70% of the time (needs measurement)
- [ ] When hole-punch fails, relay fallback engages within 1 s
- [ ] iOS background restrictions are documented; mobile uses relay as the always-on path
- [ ] End-to-end test: Pixel 6a on LTE ↔ Ubuntu on home WiFi — direct connection in 10 s

---

## 8. References

### 8.1 Existing repo documentation (read first)

- `HANDOFF/todo/P1_ANDROID_LAN_DISCOVERY_REPAIR.md` — the root-cause ticket
  that motivates this migration. (Different subnets, not the ports per se.)
- `HANDOFF/todo/[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses.md` — closely related; mDNS TXT size limit.
- `HANDOFF/todo/[VALIDATED]_P1_CLI_026_External_Address_Omits_LAN_Interface.md` — AutoNAT reports only the WSL interface; Phase 2 fixes this.
- `HANDOFF/todo/[VALIDATED]_P1_CLI_028_Config_Listen_Port_Stale_vs_Actual_Port_9101.md` — config-staleness pattern we re-use in Phase 1.
- `HANDOFF/todo/[VALIDATED]_P1_CLI_030_Discovery_Peers_Transport_Hardcoded_As_TCP_LAN.md` — `DiscoveryTransport` enum we extend in Phase 3.
- `HANDOFF/STATE/2026-06-05_NEARBY_DISCOVERY_PRODUCTION_PUSH.md` — Lucas's directive context (production-ready Android).
- `core/src/transport/discovery.rs` (471 lines) — the file we extend most.
- `core/src/transport/multiport.rs` (345 lines) — already does multi-port bind; we add ephemeral.
- `core/src/transport/nat.rs` (854 lines) — has the hole-punch API; we fill in the body.
- `core/src/transport/reflection.rs` (360 lines) — STUN-equivalent protocol; we add rendezvous.
- `BOOTSTRAP.md` (10.4 KB) — bootstrap node documentation; we will update the port-default section.

### 8.2 Standards & RFCs

- **RFC 5389** — STUN (Session Traversal Utilities for NAT) — https://datatracker.ietf.org/doc/html/rfc5389
- **RFC 5769** — STUN Test Vectors — https://datatracker.ietf.org/doc/html/rfc5769
- **RFC 7350** — DTLS over UDP / TURN / STUN — https://datatracker.ietf.org/doc/html/rfc7350
- **RFC 8445** — ICE (Interactive Connectivity Establishment) — https://datatracker.ietf.org/doc/html/rfc8445
- **RFC 9000** — QUIC — https://datatracker.ietf.org/doc/html/rfc9000 (we use libp2p's quic-v1)
- **RFC 6762** — Multicast DNS — https://datatracker.ietf.org/doc/html/rfc6762
- **RFC 6763** — DNS-Based Service Discovery — https://datatracker.ietf.org/doc/html/rfc6763

### 8.3 libp2p references

- libp2p `tcp::Config` — https://docs.rs/libp2p/latest/libp2p/tcp/struct.Config.html
- libp2p `quic::Config` — https://docs.rs/libp2p/latest/libp2p/quic/struct.Config.html
- libp2p `relay::v2::client` — https://github.com/libp2p/rust-libp2p/tree/master/protocols/relay
- libp2p `autonat` — https://github.com/libp2p/rust-libp2p/tree/master/protocols/autonat
- libp2p `mdns` — https://github.com/libp2p/rust-libp2p/tree/master/protocols/mdns (advertises `_p2p._udp` by default)
- libp2p `identify` — https://github.com/libp2p/rust-libp2p/tree/master/protocols/identify

### 8.4 Inspiration

- **Tailscale** — *"How NAT traversal works"* — https://tailscale.com/blog/how-nat-traversal-works/
  Two-peer simultaneous hole-punch via a coordination server. Their success rate is ~80% on first try.
- **ZeroTier** — control-plane design — https://github.com/zerotier/ZeroTierOne
  Uses a `ztnc` controller + `Salsa20`-encrypted peer-to-peer. We borrow the rendezvous-coordinates-peers model.
- **WebRTC ICE** — RFC 8445. The candidate-pair + connectivity-check pattern is the spiritual ancestor of our liveness probe.
- **Cjdns** — https://github.com/cjdelisle/cjdns
  Mesh networking with a "node's IPv6 address encodes its cryptographic identity" approach; we don't adopt this but it's a useful reference for *sovereign* mesh design.
- **Bitcoin Core's port-randomization** — Bitcoin's `-port` can be 0 for OS-assigned; the actual port is written to a `bitcoin.conf`-equivalent. We use the same pattern.
  https://github.com/bitcoin/bitcoin/blob/master/src/net.cpp (`BindListenPort` function, lines 1400+).

### 8.5 WSL+Android+Windows+Ubuntu stack notes (specific to Lucas's blocker)

- **WSL2 Hyper-V virtual NIC** — `172.26.x.x` is WSL2's NAT'd interface. The host's main LAN (`192.168.0.x`) is on a *different broadcast domain* — packets from Android to `172.26.154.211` traverse the host's routing table and arrive at the WSL VM. mDNS multicast (`224.0.0.251`) **does not cross** the WSL↔host boundary by default.
  - Workaround: `netsh interface portproxy add v4tov4 listenport=9001 listenaddress=0.0.0.0 connectport=9001 connectaddress=172.26.154.211` (host-side). Phase 0+1's ephemeral ports still need this proxy for inbound-from-LAN.
- **Windows Defender Firewall** — blocks inbound TCP by default on public profiles. CLI must add a firewall rule on first run, or document the manual step.
- **Android battery-saver mode** — suspends mDNS and BLE scans. Document this in the user-facing FAQ.
- **iOS Local Network permission** — first-launch prompt is required. Wire it into the BLE/mDNS startup.

---

## Appendix A: Glossary

| Term | Meaning in this document |
|---|---|
| **Ephemeral port** | A port chosen by the OS (bind to port 0); always in the IANA ephemeral range (typically 32768–60999 on Linux) |
| **Kernel-assisted binding** | `bind(2)` with port 0; the kernel returns the assigned port via `getsockname()` |
| **Rendezvous** | A third party (relay) that two peers use to coordinate the start of a hole-punch |
| **STUN** | Session Traversal Utilities for NAT (RFC 5389) — what we replace with `reflection.rs` + `liveness.rs` |
| **Liveness probe** | A small UDP packet sent to a peer; the *response* confirms the peer is alive and reveals the sender's NAT-mapped port |
| **CGNAT** | Carrier-Grade NAT — an ISP-level NAT; makes hole-punching harder |
| **mDNS TXT** | The `TXT` resource record in a multicast DNS response; carries arbitrary key=value pairs (we use `peer-id`, `dnsaddr`, and (new) `port-range`) |
| **Symmetric NAT** | A NAT that assigns a different external port per destination — hole-punching requires the rendezvous to predict the port |

## Appendix B: Out of scope

- BLE channel hopping (BLE uses 40 physical channels, not ports)
- WiFi-Direct/Aware SSID randomization (handled in `wifi_aware.rs` and `WifiDirectTransport.kt`, separate work)
- WASM thin-client port selection (constrained by browser security model)
- Browser WebRTC ICE (we use libp2p's WebSocket transport, not WebRTC)
- Adversarial port-scan detection (a future DDoS-resistance ticket)
- Multi-tenant port isolation (k8s-style; not a current need)

## Appendix C: Why not just use libp2p's `Circuit Relay v2` for everything?

The short answer: it works, but it's slow (every packet traverses the relay)
and it gives up the local-LAN fast path that Lucas's Android↔Ubuntu test
demands. libp2p's `AutoNAT` *also* only tells you your external address — it
does not perform the hole-punch. We use both libp2p primitives (they're
already in the codebase at `swarm.rs:1826`) AND the custom liveness probe +
hole-punch sketched here. This is a belt-and-suspenders design, not a
replacement.
