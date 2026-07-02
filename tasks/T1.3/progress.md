# T1.3 — Android WifiAwarePlatformBridge native implementation

**Status:** implemented (pending physical two-device verification)
**Track:** 1 (Native Hardware & Proximity Transport Layer)
**Dependencies:** T1.2
**Blocks:** none

## Technical Context
- `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt` — full publish/subscribe/data-path implementation, wired through `TransportManager` → `MeshRepository` → `uniffi.api.MeshService.onWifiAwarePeerDiscovered`/`onWifiAwareDataPathConfirmed` (real FFI calls, verified against the generated Kotlin bindings — this file's original "no FFI connection" framing was stale by the time this pass started).

## Update (2026-07-02)
Re-verified the actual state of this task against the code rather than trusting this file or the punch-list's framing (both were stale in different ways — the punch list understated how much was already wired; this file's Verification checklist below still reflects genuinely-open items). Found:

1. **`attach()` lifecycle** — `TransportManager.startAll()` (called from `MeshRepository.startMeshService()`) already called `wifiAware?.start()`, but `stopMeshService()` never called `transportManager?.stopAll()` at all (only `wifiTransportManager?.stopDiscovery()`, a different/legacy WiFi-Direct-only manager). WiFi Aware's `attach()` session leaked for the process lifetime after the mesh service stopped. **Fixed**: added `transportManager?.stopAll()` to `stopMeshService()`, and clear the reference afterward.

2. **`onServiceDiscovered` → core discovery callback** — already correctly wired (`WifiAwareTransport` → `TransportManager.onWifiAwarePeerDiscovered` → `MeshRepository` → `meshService.onWifiAwarePeerDiscovered(...)`, a real UniFFI call). No change needed.

3. **Network-available callback reporting `(ip, port)` back to core** — wired end-to-end but **broken in two ways** that would have made every real data path confirmation silently fail:
   - `core/src/mobile_bridge.rs`'s `handle_data_path_confirmed` built `SocketAddr` via `format!("{ip}:{port}").parse()`, which is invalid syntax for any IPv6 address (needs `[ip]:port` brackets) — every IPv6 confirmation silently failed to parse and the `if let Ok(...)` swallowed it, so `create_data_path`'s awaiting Rust task always timed out after 30s. **Fixed**: parse the IP and port separately via `IpAddr::from_str` + `SocketAddr::new`, which handles both v4 and v6 correctly. Added 3 regression tests in `mobile_bridge.rs` covering IPv4, IPv6, and the malformed-input case.
   - Even with that fixed, the resulting real peer IPv6+port was being dialed directly via `swarm_bridge.dial_async("/ip6/<addr>/tcp/<port>")` — see point 4.

4. **Scope-id issue** — confirmed unsupported exactly as this file predicted: `"/ip6/fe80::1234%wlan0/tcp/8765"` and the percent-encoded `%25` form both fail to parse as a `libp2p::Multiaddr` (verified directly against this workspace's libp2p version). Beyond that, even a scope-less dial would have been unreliable: the WiFi Aware responder's `ServerSocket` accepts exactly one connection and closes, so a second independent dial attempt from Rust's own TCP transport (as opposed to reusing the socket Kotlin already established) had nothing listening to connect to.

   **Fixed** via the local TCP proxy socket this file already flagged as the fallback: `WifiAwareTransport.kt`'s responder/initiator paths now bridge the real cross-device Aware socket to a freshly-bound `127.0.0.1:<ephemeral>` loopback listener (bidirectional byte pump), and report the loopback address to `onDataPathConfirmed` instead of the peer's real address. `swarm_bridge.dial_async` then dials the always-reachable loopback address; Kotlin relays bytes between it and the real Aware socket. No Rust-side changes were needed for this part beyond the `SocketAddr` fix above, since `"127.0.0.1"` already takes the ip4 (bracket-free, scope-free) branch of the existing multiaddr-construction code.

   Note: `sendData`/`onDataReceived` (a separate discrete-packet API surface, still used by BLE) are now no-ops for WiFi Aware once the loopback proxy is active — the raw socket bytes ARE the libp2p connection (Noise handshake, Yamux multiplexing) once dialed through the proxy, not a parallel message channel.

**Known follow-up limitation (not fixed, out of scope for this pass)**: the responder's Aware-facing `ServerSocket` binds a single fixed port (`AWARE_PORT = 8765`) across all peer connections; if this device is simultaneously the responder for two or more concurrent Aware subscribers, the second `ServerSocket(AWARE_PORT)` bind would fail with "address already in use". Fixing this needs the port to be negotiated per-peer (e.g. via the service-info TLV), which is a larger protocol change than this pass's scope-id/dial-path fix.

## Verification
- [ ] Instrumented test on two physical Android devices (documented manual procedure in `docs/device-testing.md` + an `adb`-scripted check): both report `DataPathActive`, then `SwarmBridge.get_peers()` on each shows the other's PeerId — still requires physical hardware, not attempted in this pass
- [x] `handle_data_path_confirmed` SocketAddr construction covered by unit tests (IPv4, IPv6, malformed input) in `core/src/mobile_bridge.rs`
- [ ] CI-side: Robolectric tests for state machine — Android test infra has no Robolectric wiring in this repo yet (same gap noted for T2.4); the Kotlin changes here could not be compiled/run in this environment (no Android SDK/Gradle available), only carefully manually reviewed
