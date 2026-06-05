# SCMessenger Dynamic-Port Discovery Migration Plan

**Date:** 2026-06-05
**Author:** deepseek-v4-pro (delegated research subagent)
**Overseer:** Lucas Ballek
**Repo:** `/mnt/e/SCMessenger-Github-Repo/SCMessenger`
**Status:** RESEARCH — ready for Phase 0 kick-off

---

## 1. Executive Summary

- **Feasible; the Rust core is already partway there.** `core/src/transport/multiport.rs` binds ephemeral TCP/QUIC (port 0), `core/src/transport/nat.rs` + `core/src/transport/reflection.rs` already define a peer-assisted STUN-equivalent, and `core/src/transport/swarm.rs:1869,1891` already listen on `tcp/0` and `udp/0/quic-v1`. The only *hardcoded* swarm listener is the WebSocket bridge at `swarm.rs:1901` (`tcp/9002/ws`).
- **The static-port problem is concentrated in (a) the WS bridge literal, (b) the CLI's `--listen` default of 9000/9001, and (c) the *client-side* port scanners on Android.** iOS is already dynamic-port-aware (`mDNSServiceDiscovery.swift:71` takes `port: Int32` as a parameter). Kotlin is not.
- **"Sender spoof" cannot be done unprivileged on modern OSes.** The actually-implementable liveness probe (§4-C) is a UDP-echo trick: send a probe to a peer, the peer echoes the *observed* source port back, and the response IS the NAT-mapping oracle. This is STUN with a peer instead of a third-party server.
- **Recommended first step (§5):** port-range allocation in `core/src/transport/multiport.rs::bind_ephemeral()` + `cli/src/config.rs` `port_range` field. **No behavior change** in v0.2.x; the scaffolding unblocks Lucas's WSL↔Android issue without breaking the v0.2.1 phone build.
- **Total cost:** ~6–10 weeks across 4 phases. Phases 0–1 (~3 weeks) deliver the immediate WSL↔Android↔Windows unblock Lucas is blocked on right now.

---

## 2. Current Static-Port Map (verified by `grep -n`)

All citations refer to `main` at `git log -1` = `118dd6ef`.

### 2.1 Rust core — `core/src/transport/`

| What | File:line | Value | Status |
|---|---|---|---|
| `DiscoveryMode::LanOnly` docstring | `core/src/transport/discovery.rs:63-67` | "well-known port (9001)" | **stub** |
| `multiport::COMMON_PORTS` | `core/src/transport/multiport.rs:12-17` | `[443, 80, 8080, 9090]` | hardcoded |
| `multiport::generate_listen_addresses` | `core/src/transport/multiport.rs:90` | `add_port(0)` | already ephemeral |
| Swarm TCP listener | `core/src/transport/swarm.rs:1869` | `/ip4/0.0.0.0/tcp/0` | already ephemeral |
| Swarm QUIC listener | `core/src/transport/swarm.rs:1891` | `/ip4/0.0.0.0/udp/0/quic-v1` | already ephemeral |
| **WebSocket bridge** (the asymmetry) | `core/src/transport/swarm.rs:1901` | `/ip4/0.0.0.0/tcp/9002/ws` | **hardcoded** |
| Multi-port bind call | `core/src/transport/swarm.rs:1842` | `multiport::generate_listen_addresses(&config)` | wired |
| `NatTraversal::start_hole_punch` API | `core/src/transport/nat.rs:388-437` | in-tree, body unimplemented |
| `NatTraversal::detect_nat_type` | `core/src/transport/nat.rs:96-174` | asks peers via reflection |
| HPTC magic documented | `core/src/transport/nat.rs:473-482` | `0x48505443` "HPTC" | not on wire yet |
| `AddressReflectionRequest/Response` | `core/src/transport/reflection.rs:24-87` | protocol defined | ready |
| `AddressReflectionService::handle_request` | `core/src/transport/reflection.rs:129-148` | server side | ready |
| `random_port` (B1_CORE_ENTRY_008) | `core/src/transport/swarm.rs:1884-1892` | exercised per boot | ready |
| BLE GATT UUID | `core/src/transport/ble/gatt.rs:11,34-36` | `0xDF01..0xDF04` | unaffected |
| BLE beacon UUID | `core/src/transport/ble/beacon.rs:16` | `0xDF01` | unaffected |
| BLE L2CAP PSM | `core/src/transport/ble/l2cap.rs:12-13` | `0x0025` | unaffected |

### 2.2 CLI — `cli/src/`

| What | File:line | Value |
|---|---|---|
| `Config::default().listen_port` | `cli/src/config.rs:71` | `9000` |
| `Relay --listen` default | `cli/src/main.rs:184` | `/ip4/0.0.0.0/tcp/9001` |
| `Relay --http_port` default | `cli/src/main.rs:187` | `9000` |
| Fallback dial ports | `cli/src/main.rs:1445` | `[9001, 4001, 9000, 8000]` |
| Snap `0 → 9000` (1st site) | `cli/src/main.rs:1180-1182` | `if config.listen_port == 0 { 9000 }` |
| Snap `0 → 9000` (2nd site) | `cli/src/main.rs:723-726` | `ws_port = if config.listen_port == 0 { 9000 } else { config.listen_port }` |
| `WS Bridge:` banner literal | `cli/src/main.rs:2320` | `ws://0.0.0.0:9002 (libp2p-ws)` |
| Bootstrap peer primary | `cli/src/bootstrap.rs:28, 183` | `/ip4/34.135.34.73/tcp/9001/p2p/12D3Koo…` |
| Related ticket (config staleness) | `HANDOFF/todo/[VALIDATED]_P1_CLI_028_*.md` | same bug class (config 9000, daemon 9101) |

### 2.3 Android — `android/app/src/main/java/com/scmessenger/android/transport/`

| What | File:line | Value |
|---|---|---|
| `MdnsServiceDiscovery.serviceType` | `MdnsServiceDiscovery.kt:65` | `_p2p._udp` (libp2p default) |
| `MdnsServiceDiscovery.servicePort` | `MdnsServiceDiscovery.kt:67` | `9001` (**hardcoded**) |
| mDNS TXT `dnsaddr` | `MdnsServiceDiscovery.kt:405` | `/ip4/0.0.0.0/tcp/$servicePort/p2p/$localId` |
| `SubnetProbe.targetPorts` | `SubnetProbe.kt:71` | `intArrayOf(9001, 9002)` (**hardcoded**) |
| `SubnetProbe` doc | `SubnetProbe.kt:33-34` | "libp2p swarm listens on TCP 9001, relay/WS on 9002" |
| `WifiDirectTransport.SERVICE_TYPE` | `WifiDirectTransport.kt:453` | `_scmessenger._tcp` (*different* from mDNS) |
| `TransportManager` comment | `TransportManager.kt:55, 125` | "open port 9001 / 9002" |
| Root-cause ticket | `HANDOFF/todo/P1_ANDROID_LAN_DISCOVERY_REPAIR.md:12-13` | different subnets (192.168.0.x vs 172.26.154.x) |

**Note:** Android has *two* mDNS service types: `_p2p._udp` (libp2p, `MdnsServiceDiscovery.kt:65`) and `_scmessenger._tcp` (WiFi-Direct, `WifiDirectTransport.kt:453`). We preserve this distinction in the migration.

### 2.4 iOS — `iOS/SCMessenger/SCMessenger/Transport/`

| What | File:line | Value |
|---|---|---|
| `mDNSServiceDiscovery.serviceType` | `mDNSServiceDiscovery.swift:35` | `_scmessenger._tcp` |
| `startAdvertising(port:)` | `mDNSServiceDiscovery.swift:71` | **already takes `port: Int32`** |
| Reads back actual port from socket | `mDNSServiceDiscovery.swift:163, 174, 179` | already dynamic |

**iOS is already 80% dynamic-port-aware** — only the Swift side needs to source the port from a new config field.

### 2.5 WASM

Browser security model constrains WASM to the bridge port. No changes.

### 2.6 Other static references worth noting

- `adb reverse` target (per `P1_ANDROID_LAN_DISCOVERY_REPAIR.md:12`): `localhost:9002` (WS bridge).
- Windows relay wrapper cmd (per same ticket): ports `9100/9101`.

---

## 3. Goals & Constraints

**Layered model** (per Lucas's directive):

1. **Discovery protocol is port-agnostic.** A peer that hears about another peer (mDNS, BLE, SubnetProbe, DHT) learns the *current* listen port as part of the payload — never assumes `9001`.
2. **Listen port is ephemeral by default, configurable per-launch.** Default range `9000–9100`; actual port written to `config.json` on shutdown (re-used on restart for determinism).
3. **Backwards compatible with v0.2.x.** A v0.2.x peer that probes 9001 must still find a v0.3.x peer; v0.3.x must still fall back to 9001 if the mDNS TXT is missing the port-range attribute.
4. **Kernel picks the port when possible** (port 0 in `bind(2)`); eliminates "in use" errors.
5. **Self-NAT-mapping via liveness probe** (§4-C) — no third-party STUN server.

**Scope-out:** BLE channel hopping (BLE uses physical channels, not TCP/UDP ports), WiFi-Direct SSID changes (separate ticket), WASM (constrained by browser).

---

## 4. Three Concrete Approaches (code sketches)

All sketches target `core/src/transport/`. Stub types marked `// stub:`.

### 4.A Ephemeral port binding + mDNS TXT extension (simplest, immediate win)

**Premise.** Let the kernel pick a port in a configured range, advertise it via mDNS TXT, and read it back from the swarm listeners.

```rust
// core/src/transport/multiport.rs (NEW)
pub struct AllocatedPort {
    pub socket: tokio::net::TcpListener,
    pub port: u16,
}

pub async fn bind_ephemeral(
    range: std::ops::RangeInclusive<u16>,
) -> Result<AllocatedPort, std::io::Error> {
    use socket2::{Domain, Socket, Type};
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};

    // 1) Kernel-assigned first (port 0) — preferred for uniqueness.
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    #[cfg(all(unix, not(target_os = "macos")))]
    socket.set_reuse_port(true)?;
    socket.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0).into())?;
    socket.listen(128)?;
    socket.set_nonblocking(true)?;
    let local: SocketAddr = socket.local_addr()?;
    if range.contains(&local.port()) {
        return Ok(AllocatedPort { socket: socket.into(), port: local.port() });
    }
    // 2) Walk the range as fallback.
    for candidate in range {
        let s = Socket::new(Domain::IPV4, Type::STREAM, None)?;
        s.set_reuse_address(true)?;
        if s.bind(&SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), candidate).into()).is_ok()
            && s.listen(128).is_ok() {
            s.set_nonblocking(true)?;
            return Ok(AllocatedPort { socket: s.into(), port: candidate });
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "no port in range"))
}
```

**Integration at `swarm.rs:1898-1906`** (replace hardcoded WS literal):

```rust
// core/src/transport/swarm.rs (REPLACE the "/ip4/0.0.0.0/tcp/9002/ws" block)
let ws_port = match multiport::bind_ephemeral(9000..=9100).await {
    Ok(p) => {
        let _ = swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}/ws", p.port).parse()?);
        p.port
    }
    Err(e) => {
        tracing::warn!("ephemeral bind failed, falling back to 9002: {}", e);
        let _ = swarm.listen_on("/ip4/0.0.0.0/tcp/9002/ws".parse()?);
        9002
    }
};
state.ws_port.set(ws_port);  // expose to discovery module for mDNS TXT
```

**Kotlin mDNS update** (replaces `MdnsServiceDiscovery.kt:67`):

```kotlin
// android/.../transport/MdnsServiceDiscovery.kt
private val serviceType = "_p2p._udp"   // unchanged
private val servicePort: Int
    get() = scmessengerCore.getState().tcpPort   // 0 → ephemeral, else static
```

**Platform support for `SO_REUSEADDR`/`SO_REUSEPORT`:** Linux ≥ 3.9, Windows 10 1709+ (REUSEADDR), Windows 11 22H2+ (REUSEPORT), macOS ≥ 10.10, Android N+ (kernel 4.4+), iOS 15+.

### 4.B Relay-rendezvous UDP hole-punching (uses existing bootstrap as STUN-like server)

**Premise.** Reuse the existing bootstrap relay (`34.135.34.73:9001` per `cli/src/bootstrap.rs:28`) as a rendezvous coordinator. Both A and B send a UDP probe; the relay only forwards *addresses*, never payloads. Fills in the *unimplemented* body of `nat.rs::send_hole_punch_probes` documented at `nat.rs:467-489`.

```rust
// core/src/transport/nat.rs (REPLACE the simulated body of send_hole_punch_probes)
async fn send_hole_punch_probes(&self, attempt_key: &str) -> Result<(), NatTraversalError> {
    use tokio::net::UdpSocket;
    use rand::RngCore;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mut pkt = Vec::with_capacity(60);
    pkt.extend_from_slice(&0x48505443_u32.to_be_bytes());     // "HPTC"
    let mut nonce = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    pkt.extend_from_slice(&nonce);
    pkt.extend_from_slice(&current_unix_timestamp().to_be_bytes());
    // signature omitted — sign with local_peer_id (see nat.rs::HolePunchAttempt)

    for _ in 0..10 {
        let attempt = self.hole_punch_attempts.read().get(attempt_key).cloned()
            .ok_or(NatTraversalError::HolePunchFailed("attempt vanished".into()))?;
        socket.send_to(&pkt, attempt.remote_external_addr).await?;
        let mut buf = [0u8; 64];
        match tokio::time::timeout(Duration::from_millis(500), socket.recv_from(&mut buf)).await {
            Ok(Ok((n, src))) if n >= 4 && &buf[..4] == b"HPTC" => {
                tracing::info!("Hole-punch success: {} echoed by {}", attempt_key, src);
                return Ok(());
            }
            _ => continue,
        }
    }
    Err(NatTraversalError::HolePunchFailed("exhausted probes".into()))
}
```

**Why not just `libp2p::relay::v2::client` + `AutoNAT`?** AutoNAT only reports the *external* address — does not punch. Circuit Relay is a guaranteed hop but slow. We use **relay for signalling, hole-punch for the data path** (Tailscale/ZeroTier pattern). libp2p primitives are already at `swarm.rs:1826`; the rendezvous protocol extends `reflection.rs`:

```rust
// core/src/transport/reflection.rs (NEW message variant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolePunchRendezvousRequest {
    pub request_id: [u8; 16],
    pub target_peer_id: PeerId,                  // stub: see core::types::PeerId
    pub my_observed_address: SocketAddr,
    pub prefer_udp_port: Option<u16>,            // hint, e.g. midpoint of 9000..=9100
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolePunchRendezvousResponse {
    pub request_id: [u8; 16],
    pub target_observed_address: SocketAddr,     // the *other* peer's external addr
    pub suggested_local_port: u16,               // rendezvous guesses the NAT port
    pub nat_type_hint: NatType,                  // stub: see core::types::NatType
}
```

### 4.C UDP liveness probe with "sender-spoof" trick → self-NAT-mapping oracle

**Premise.** Lucas said *"spoof sender and open sessions that would be routed intentionally back to a node to test a response."* On modern OSes (Linux ≥ 4.4, Windows ≥ 10, macOS ≥ 10.13) you **cannot** spoof the source address of a UDP datagram from an unprivileged socket. But the *response* to a probe IS the NAT-mapping oracle — that's exactly what STUN does with a third-party server. We do it with a peer.

```
A: bind UDP on 0.0.0.0:0 (kernel picks SRC)
A: send(LPRB + nonce) → B:DISCOVERED_PORT
B: read source address (the IP:port A used as seen by B)
B: echo the packet back to that source
A: receives echo → observed_local_port = source port B reported = A's NAT mapping
```

```rust
// core/src/transport/liveness.rs  (NEW)
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use rand::RngCore;

const MAGIC: [u8; 4] = *b"LPRB";

#[derive(Debug, Clone, Copy)]
pub struct ProbeResult {
    pub rtt: Duration,
    pub observed_local_port: u16,    // = A's NAT-mapped port, as B saw it
    pub responder_addr: SocketAddr,
}

pub struct LivenessProbe { socket: UdpSocket }

impl LivenessProbe {
    pub async fn new() -> std::io::Result<Self> {
        Ok(Self { socket: UdpSocket::bind("0.0.0.0:0").await? })
    }

    pub async fn probe(&self, target: SocketAddr, timeout: Duration)
        -> Result<ProbeResult, ProbeError>
    {
        let mut nonce = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let mut pkt = Vec::with_capacity(20);
        pkt.extend_from_slice(&MAGIC);
        pkt.extend_from_slice(&nonce);
        let started = Instant::now();
        self.socket.send_to(&pkt, target).await?;
        let mut buf = [0u8; 64];
        loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() { return Err(ProbeError::Timeout); }
            match tokio::time::timeout(remaining, self.socket.recv_from(&mut buf)).await {
                Ok(Ok((n, responder)))
                    if n >= 20 && buf[..4] == MAGIC && buf[4..20] == nonce =>
                {
                    return Ok(ProbeResult {
                        rtt: started.elapsed(),
                        observed_local_port: responder.port(),
                        responder_addr: responder,
                    });
                }
                Ok(Ok(_)) => continue,
                Ok(Err(e)) => return Err(ProbeError::Io(e)),
                Err(_) => return Err(ProbeError::Timeout),
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("probe timed out")] Timeout,
    #[error("io: {0}")] Io(std::io::Error),
}
```

**Honest caveat on "spoof sender":** true source-IP spoofing needs `CAP_NET_RAW` (Linux) or `IP_HDRINCL` raw sockets (Windows admin) — out of scope for v0.3. The §4-C sketch is the *actually-implementable* equivalent and is what Tailscale/Zerotier/WebRTC ICE use internally. Capture the v0.4 requirement separately.

---

## 5. 4-Phase Migration Plan

### Phase 0: Foundation (1–2 weeks, no behavior change)

**Goal:** expose ports as data, not constants. Pure scaffolding. This phase alone is the **minimum viable unblock** for Lucas's WSL↔Android issue.

1. **`core/src/transport/multiport.rs`** — add `bind_ephemeral()` (§4-A).
2. **`cli/src/config.rs`** — add `port_range: Option<(u16, u16)>` to `NetworkConfig`. Default `Some((9000, 9100))`. **Do not change `listen_port` default of `9000`.**
3. **`cli/src/main.rs`** — accept `--port-range 9000-9100` flag on `start` and `relay`; *ignored* until Phase 1.
4. **`android/.../transport/SubnetProbe.kt:71`** — read `targetPorts` from `SharedPreferences` (default `[9001, 9002]`); settings screen exposes override. Keeps v0.2.1 behavior identical.
5. **Metrics** — add counters: `port_collision_count`, `port_in_use_count`, `ephemeral_bind_attempts`. Expose via `/api/health`.
6. **Docs** — `BOOTSTRAP.md` "Port model" section.

**Gate:** all existing tests pass; no defaults change. CLI *accepts* `--port-range` but ignores it.

### Phase 1: Ephemeral CLI port (1 week)

**Goal:** the CLI's P2P listener and the WS bridge both bind to ephemeral ports, advertise the actual port, and write it to `config.json`.

1. **`core/src/transport/swarm.rs:1898-1906`** — replace hardcoded `9002/ws` with `bind_ephemeral(9000..=9100)`.
2. **`core/src/transport/swarm.rs:1836-1873`** — read actual port from `swarm.listeners()` for the *advertised* mDNS TXT (the bind is already ephemeral).
3. **`cli/src/main.rs:1180-1182`** and `723-726` — remove both `if config.listen_port == 0 { 9000 }` snaps. Let `0` mean "kernel-assigned" end-to-end.
4. **`cli/src/main.rs::cmd_start`** — after binding, find the first non-loopback `/ip4/.../tcp/<port>` in `swarm.listeners()` and write it to `config.listen_port`. Reuse the pattern from `[VALIDATED]_P1_CLI_028_…md`.
5. **mDNS TXT** — extend `dnsaddr` from `MdnsServiceDiscovery.kt:405` literal to actual `/ip4/<local-ip>/tcp/<actual-port>/p2p/<localId>`.
6. **CLI banner** — `cli/src/main.rs:2320` `WS Bridge: ws://0.0.0.0:9002` becomes `WS Bridge: ws://0.0.0.0:{ws_port}`.
7. **Backwards-compat shim** — if `config.listen_port ∈ 9000..=9100`, *also* try to bind `9001`; advertise *both* in mDNS TXT. v0.2.x clients dial 9001; v0.3.x clients dial the ephemeral. Merges with ticket `[VALIDATED]_P1_CLI_028_*.md`.

**Gate:**
- CLI listens on `9000..=9100` (printed to stdout).
- Restart picks the *same* port (deterministic, no flapping).
- Older v0.2.x Android client that only probes `9001/9002` still finds the new CLI.
- The port-staleness warning from `P1_CLI_028` no longer fires.

### Phase 2: Discovery port-range negotiation (2–3 weeks)

**Goal:** every discovery mechanism (mDNS, BLE, SubnetProbe, DHT) carries the *current* listen port as part of its payload.

1. **mDNS TXT extension** — add `port-range=9000-9100` to the daemon's TXT record. `MdnsServiceDiscovery.kt:152-160` already parses `peer-id`/`p2p`/`dnsaddr`; add `port-range` parsing → new `MdnsTxtRecord.portRange: (u16, u16)?` field.
2. **mDNS TXT for Android** — `MdnsServiceDiscovery.kt:405` writes `dnsaddr=/ip4/0.0.0.0/tcp/9001/p2p/<id>`. Replace `0.0.0.0` with local IP and `9001` with actual port from `scmessenger_core::get_state().tcp_port`.
3. **Android `SubnetProbe.kt:71`** — replace `intArrayOf(9001, 9002)` with the resolved `port-range` (default `[9001, 9002]` if no mDNS TXT). **Do not probe every port in 9000–9100** (101 ports × /24 subnets × 254 hosts ≈ 25k TCP attempts/cycle — battery killer). Probe a *sample*: `[range.start, range.end, 9001, 9002, 4001]` deduped.
4. **BLE manufacturer data** — extend `core/src/transport/ble/beacon.rs` to include current TCP port (2 bytes) + CRC8 in the manufacturer-data field. Service UUID stays `0xDF01`. Backward-compatible: scanners ignore unknown fields.
5. **Core RPC** — new `port_announce { listen_addrs: Vec<Multiaddr>, port_range: (u16, u16), nonce: u64 }` on a libp2p request-response protocol. Peers cache it; on a new range (port hop) they update.
6. **iOS** — `mDNSServiceDiscovery.swift:71` already takes `port`. Wire the port source to a new `BridgeConfig.tcpPort`. *iOS is already 80% done.*
7. **Merge** — Phase 2 also fixes `[VALIDATED]_P1_CLI_024_*.md` (mDNS TXT > 1300 bytes) by stripping circuit addresses from the cached p2p-circuit chains (keep the listen address, drop the relayed routes).

**Gate:**
- Two CLIs on the same LAN see each other via mDNS TXT and dial the ephemeral port, not 9001.
- Android SubnetProbe on a foreign subnet (192.168.0.x probing 172.26.x.x) finds the daemon within 30 s.
- A peer that hops ports (kill + restart CLI) is rediscovered within `min(mDNS TTL, 60s)`.
- BLE-discovered peers can be dialed on TCP without a separate discovery step.

### Phase 3: Liveness probe + NAT traversal (3–4 weeks)

**Goal:** when mDNS/BLE/SubnetProbe all fail, the liveness probe from §4-C is the last-mile tiebreaker AND doubles as a self-STUN oracle. Then fill in the UDP hole-punch body from §4-B.

1. **`core/src/transport/liveness.rs`** — implement `LivenessProbe` (§4-C). Wire into `core/src/transport/manager.rs` as a long-lived background task.
2. **`DiscoveryConfig`** — add `liveness_probe_timeout_ms: u64` (default `800`) and `liveness_probe_fanout: usize` (default `3`).
3. **Probe routing** — when `discovery_transport()` returns `Unknown` or `OtherLAN` (per `[VALIDATED]_P1_CLI_030_…md`), fall back to liveness probe. Probe up to `fanout` candidates in parallel.
4. **Self-mapping** — every 60 s probe a known-good peer (bootstrap or any connected peer) to learn the kernel's NAT mapping. Cache in `NatTraversal::external_address`. Compare against `request_address_reflection` answer — mismatch suggests symmetric NAT.
5. **`core/src/transport/nat.rs::send_hole_punch_probes`** — fill in the body (§4-B). Extend `nat.rs:154-170` to classify `Symmetric` when *port* changes between probes (not just address).
6. **`core/src/transport/reflection.rs`** — add `HolePunchRendezvousRequest/Response` (§4-B). Wire through a libp2p request-response behaviour.
7. **Relay fallback** — keep `internet.rs::RelayMode::Client` as the always-on fallback. Hole-punch is the fast path.
8. **iOS / Android background** — iOS suspends UDP in background; the probe is server-side only on iOS (answer, don't send). Mobile uses relay as the always-on path.

**Gate:**
- A node behind CGNAT on phone LTE reports its external port within 5 s of startup.
- Two laptops on the same coffee-shop WiFi with multicast disabled are discoverable in < 15 s via liveness probe alone.
- Two nodes on different NATs (one home WiFi, one CGNAT) establish direct UDP > 70% of the time. (Baseline: Tailscale reports ~80% on first try — **needs measurement** for SCMessenger.)
- When hole-punch fails, the relay fallback engages within 1 s; UI shows a "relayed" indicator.
- End-to-end: Pixel 6a on LTE ↔ Ubuntu on home WiFi — direct connection in 10 s.

---

## 6. Recommended First Step (the one ticket to write today)

**Write: `HANDOFF/todo/P0_PORT_RANGE_FOUNDATION.md`** — implement `multiport::bind_ephemeral()` + `Config::port_range` + `--port-range` flag (Phase 0 tasks 1–3).

**Why this, not the others:**

- **PortRange config (A)** is the *only* choice that has **zero behavior change**. v0.2.1 keeps shipping to Lucas's Pixel; v0.2.1 tests keep passing. We layer ephemeral binding on top in Phase 1.
- **SubnetProbe port-range support (B)** changes Android behavior immediately — would block on a new APK build/test cycle, which is what we are trying to *shorten*, not extend.
- **UDP liveness probe (C)** is high-value but depends on a real bootstrap peer being reachable; without Phase 0's port-range plumbing, the probe has nowhere to send its echo (still using hardcoded 9001).

**Effect:** unblocks Lucas's WSL↔Android issue at the *config* level (he can now set `--port-range 9000-9100` explicitly) before we finish the runtime work. The actual bind change in Phase 1 is then a one-line `swarm.rs:1901` edit, fully covered by the Phase 0 metrics.

### Full ticket content (paste into `HANDOFF/todo/P0_PORT_RANGE_FOUNDATION.md`)

```markdown
# P0 — Port-Range Foundation (Dynamic-Port Discovery Phase 0)

**Owner:** TBD
**Phase:** 0 of 4 (no behavior change)
**Blocked-by:** none
**Blocks:** Phase 1 (Ephemeral CLI Port)
**Related:** `[VALIDATED]_P1_CLI_028_*.md`, `P1_ANDROID_LAN_DISCOVERY_REPAIR.md`

## Goal
Expose `port_range` as a CLI/config field and implement
`multiport::bind_ephemeral()`. No defaults change; v0.2.1 behavior is preserved.

## Tasks
1. **`core/src/transport/multiport.rs`** — add `AllocatedPort` struct and
   `bind_ephemeral(range: RangeInclusive<u16>) -> Result<AllocatedPort, io::Error>`.
   Tries kernel-assigned port 0 first, then walks the range. SO_REUSEADDR
   always set; SO_REUSEPORT on Linux/Android. See
   `HANDOFF/research/2026-06-05_DYNAMIC_PORT_DISCOVERY_RESEARCH.md` §4-A.
2. **`cli/src/config.rs`** — add `port_range: Option<(u16, u16)>` to
   `NetworkConfig` (after `listen_port: u16` at line 71). Default
   `Some((9000, 9100))`. Serialize as `port_range: [u16; 2]` in JSON.
3. **`cli/src/main.rs`** — accept `--port-range 9000-9100` flag on `start`
   and `relay` (next to `--listen` at line 184). Parse with
   `s.parse::<(u16, u16)>()` semantics. **Ignore the value** for now —
   just store it in `Config`. Phase 1 will start reading it.
4. **Unit tests** — `bind_ephemeral` returns a port within the range
   (statistical test: 100 calls, all in range). Concurrent binds succeed
   with `SO_REUSEPORT`. `port_range` round-trips through JSON.
5. **Metrics** — `port_collision_count`, `port_in_use_count`,
   `ephemeral_bind_attempts` exposed via `/api/health` (counter, not
   implemented yet — just the `prometheus::IntCounter` registration).
6. **Docs** — one paragraph in `BOOTSTRAP.md` "Port model" section.

## Acceptance
- [ ] `cargo test -p scmessenger-core transport::multiport::tests` passes
- [ ] `cargo test -p scmessenger-cli config::` passes
- [ ] `cargo run --bin scmessenger-cli start --port-range 9000-9100` starts
      and prints "port range configured: 9000-9100 (Phase 0: ignored)"
- [ ] Default `./scmessenger-cli start` (no flag) behaves identically to
      v0.2.1 (regression check: same ports bound as v0.2.1 tag)

## Out of scope
- No change to `swarm.rs:1901` (WS bridge literal).
- No change to `cli/src/main.rs:1180-1182` snap-to-9000.
- No change to `MdnsServiceDiscovery.kt:67` literal.
- All those are Phase 1.
```

---

## 7. Risks (5 max, with mitigations)

| Risk | Mitigation |
|---|---|
| **Backwards compat with v0.2.x** (certain — we ship v0.2.1 to Lucas's phone) | Phase 1 backwards-compat shim: daemon binds both ephemeral AND 9001 (if free); mDNS TXT carries both; clients pick the newer one. |
| **Corporate firewall blocks high ports** (high likelihood, high impact) | Default range `9000–9100` is a private-range choice; `--port 443` and `--port 80` already supported via `multiport::COMMON_PORTS` (`multiport.rs:12-17`). |
| **Debugging complexity: port is no longer constant in logs** (high likelihood, medium impact) | Phase 0 mandates: every log line that mentions a port also includes the boot timestamp + short node ID, so `grep "boot=2026-06-05T11:30Z nodeId=abc" log.txt` works. |
| **mDNS TXT > 1300 bytes** (already happening, per `P1_CLI_024`) | Phase 2 fixes by stripping cached p2p-circuit chains from TXT; keep the listen address only. |
| **WSL Hyper-V NAT eats multicast** (already broken in v0.2.1) | Phase 2+3 provide SubnetProbe + liveness probe as non-multicast fallbacks. |

---

## 8. References

### 8.1 Existing repo docs (read first)
- `HANDOFF/todo/P1_ANDROID_LAN_DISCOVERY_REPAIR.md` — root-cause ticket (different subnets).
- `HANDOFF/todo/[VALIDATED]_P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses.md` — TXT size limit.
- `HANDOFF/todo/[VALIDATED]_P1_CLI_026_External_Address_Omits_LAN_Interface.md` — AutoNAT LAN bug.
- `HANDOFF/todo/[VALIDATED]_P1_CLI_028_Config_Listen_Port_Stale_vs_Actual_Port_9101.md` — config staleness pattern reused in Phase 1.
- `HANDOFF/todo/[VALIDATED]_P1_CLI_030_Discovery_Peers_Transport_Hardcoded_As_TCP_LAN.md` — `DiscoveryTransport` enum extended in Phase 3.
- `HANDOFF/STATE/2026-06-05_NEARBY_DISCOVERY_PRODUCTION_PUSH.md` — production-ready directive.
- `core/src/transport/discovery.rs` (471 lines) — file we extend most.
- `core/src/transport/multiport.rs` (345 lines) — adds `bind_ephemeral`.
- `core/src/transport/nat.rs` (854 lines) — fills in hole-punch body.
- `core/src/transport/reflection.rs` (360 lines) — adds rendezvous.

### 8.2 Standards
- **RFC 5389** — STUN — https://datatracker.ietf.org/doc/html/rfc5389
- **RFC 5766** — TURN — https://datatracker.ietf.org/doc/html/rfc5766
- **RFC 8445** — ICE — https://datatracker.ietf.org/doc/html/rfc8445
- **RFC 9000** — QUIC — https://datatracker.ietf.org/doc/html/rfc9000
- **RFC 6762/6763** — mDNS / DNS-SD — https://datatracker.ietf.org/doc/html/rfc6762

### 8.3 libp2p
- `tcp::Config` — https://docs.rs/libp2p/latest/libp2p/tcp/struct.Config.html
- `relay::v2::client` — https://github.com/libp2p/rust-libp2p/tree/master/protocols/relay
- `autonat` — https://github.com/libp2p/rust-libp2p/tree/master/protocols/autonat
- `mdns` (advertises `_p2p._udp`) — https://github.com/libp2p/rust-libp2p/tree/master/protocols/mdns

### 8.4 Inspiration
- **Tailscale** — *How NAT traversal works* (simultaneous hole-punch via coordinator, ~80% first-try).
- **ZeroTier** — `ztnc` control-plane, rendezvous-coordinates-peers model.
- **WebRTC ICE** (RFC 8445) — candidate-pair + connectivity-check pattern.
- **Bitcoin Core** — `BindListenPort` accepts port 0, writes actual port to `bitcoin.conf` (same pattern as Phase 1).
- **Cjdns** — IPv6-address-encodes-key; reference for sovereign mesh design.

### 8.5 WSL+Android+Windows stack notes
- **WSL2 Hyper-V NIC** (`172.26.x.x`): mDNS `224.0.0.251` does not cross the WSL↔host boundary. Workaround: `netsh interface portproxy add v4tov4 listenport=… connectport=… connectaddress=172.26.154.211`. Phase 0+1's ephemeral ports still need this proxy for inbound-from-LAN.
- **Windows Defender** blocks inbound TCP on public profile (CLI must add a firewall rule on first run). **Android battery-saver** suspends mDNS/BLE scans. **iOS Local Network** first-launch prompt required.
