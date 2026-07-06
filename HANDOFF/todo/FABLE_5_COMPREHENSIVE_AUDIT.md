# FABLE 5 COMPREHENSIVE ARCHITECTURE & CODEBASE AUDIT

> **Date:** 2026-07-05  
> **Audit Scope:** Android (`android/`), Windows CLI (`cli/`), Rust Core (`core/`)  
> **Purpose:** Definitive work assignment backlog for the Fable 5 networking and stabilization sprint.  
> **Method:** Live device testing (Pixel + Windows CLI), log analysis, and automated 8-subagent codebase sweep.

---

## PART I: Primary Pairing Failures (Live-Device Confirmed)

These 4 issues were discovered during hands-on testing and confirmed via `adb logcat` and `daemon.log` analysis. They collectively explain why the Android ↔ Windows pairing is currently non-functional.

---

### Issue 1 · TCP Listener "Zombie" State
**Severity:** P0 — Primary blocker for all inbound Android connections  
**Components:** `core/src/mobile_bridge.rs`, `core/src/transport/swarm.rs`, `MeshRepository.kt`  
**Evidence:** `adb shell cat /proc/net/tcp` shows no socket in LISTEN state on port 9001, even when foregrounded.

**Root Cause Chain:**
1. `MeshService::start()` (`mobile_bridge.rs`) synchronously sets `*self.state.lock() = ServiceState::Running` before any network activity.
2. `start_swarm()` spawns a detached OS thread (`std::thread::Builder::new().spawn(...)`) and returns `Ok(())` to the FFI caller immediately.
3. Inside the thread, `swarm.listen_on(addr)` returns `Ok(ListenerId)` — but this only queues the bind request. The actual OS socket bind happens asynchronously.
4. If the bind fails, libp2p emits `SwarmEvent::ListenerError` (line 3952 of `swarm.rs`). The event loop logs it with `tracing::error!` and **discards it**. No `event_tx` notification, no state downgrade.
5. `SwarmEvent2` enum (line 1348) has a `ListeningOn` variant but **no** `ListenerFailed` variant — there is no mechanism to propagate failure.

**Fable 5 Deliverables:**
- [ ] Add `ListenerFailed { listener_id, error }` variant to `SwarmEvent2`
- [ ] In native event loop: on `ListenerError`, send `SwarmEvent2::ListenerFailed` via `event_tx`
- [ ] In `mobile_bridge.rs`: `start_swarm_with_config` must use a `oneshot::channel` to await the first `NewListenAddr` event before returning success
- [ ] Kotlin: `MeshRepository.initializeAndStartSwarm` must not mark state `RUNNING` until the awaited startup returns success
- [ ] ~50 LOC Rust + ~20 LOC Kotlin

---

### Issue 2 · SubnetProbe ANR (Thread Starvation)
**Severity:** P0 — App crashes on "Rescan" in Nearby Contacts  
**Component:** `android/.../transport/SubnetProbe.kt`  
**Evidence:** `adb shell dumpsys dropbox --print data_app_anr` captured thread starvation trace originating from `SubnetProbe$runSweep`.

**Root Cause Chain:**
1. `runSweep()` (line 162) creates a blocking `java.util.concurrent.Semaphore(32)`.
2. Thousands of `scope.async(Dispatchers.IO) { sem.acquire(); ... }` coroutines are launched.
3. `sem.acquire()` is a **thread-blocking** call. 32 threads get permits; all remaining threads in `Dispatchers.IO` (pool size ~64) block indefinitely waiting for permits.
4. The 32 permitted threads then call `Socket().connect()` (line 188) — another thread-blocking call.
5. The entire `Dispatchers.IO` pool is frozen. All other background work (outbox flush, diagnostics, identity sync) starves. ANR fires.

**Fable 5 Deliverables:**
- [ ] Replace `java.util.concurrent.Semaphore` with `kotlinx.coroutines.sync.Semaphore` and `sem.withPermit { ... }`
- [ ] Replace `Socket().connect()` in `probeHost()` with `suspendCancellableCoroutine` wrapping `java.nio.channels.AsynchronousSocketChannel` (`CompletionHandler` pattern, verified compatible with `minSdk=26`)
- [ ] Wrap with `withTimeoutOrNull(connectTimeoutMs)` for clean cancellation
- [ ] ~80 LOC Kotlin

---

### Issue 3 · BLE Subscription Spin-Loop (Windows CLI)
**Severity:** P1 — Causes high CPU, log spam, and wasted Bluetooth I/O  
**Component:** `cli/src/ble_mesh.rs`, function `run_ble_central_ingress` (lines 219–297)  
**Evidence:** `daemon.log` shows `GattCommunicationStatus(1)` warning every ~30 seconds, continuously for 24+ hours.

**Root Cause Chain:**
1. `run_ble_central_ingress` consumes BLE `CentralEvent` stream via `while let Some(evt) = events.next().await`.
2. For each advertisement, it checks if the peripheral ID is in a `HashSet<String>` (`tracked`). If absent, adds it and spawns a connection task.
3. If `subscribe_ingress_for_peripheral` fails (e.g., `GattCommunicationStatus(1)` = Unreachable), the spawned task logs the warning and **removes the peripheral from `tracked`**.
4. The very next BLE advertisement (emitted multiple times per second by the unreachable device) re-enters the loop, sees the ID missing from `tracked`, and spawns a new task. Infinite spin-loop.

**Fable 5 Deliverables:**
- [x] Replace `HashSet<String>` with `HashMap<String, ConnectionState>` where `ConnectionState` tracks `{ active: bool, failures: u32, cooldown_until: Option<Instant> }` — **DONE** (verified `cargo check`)
- [x] On failure: increment `failures`, set `cooldown_until = Instant::now() + Duration::from_secs((1 << failures).min(60))` — **DONE**
- [x] In the event loop: skip peripherals where `Instant::now() < cooldown_until` — **DONE**
- [x] On success: reset `failures` to 0 and clear `cooldown_until` — **DONE**
- [x] ~40 LOC Rust — **DONE**

---

### Issue 4 · Outbound Dial Failures (Discovery → libp2p Gap)
**Severity:** P1 — Android discovers Windows but never establishes a libp2p connection  
**Components:** `SubnetProbe.kt`, `MeshRepository.kt` (line 935–943), `core/src/mobile_bridge.rs` (line 3013–3026)  
**Evidence:** Windows `daemon.log` shows: `Incoming connection error from /ip4/192.168.0.148/tcp/50746 -> /ip4/192.168.0.121/tcp/9001: Failed to negotiate transport protocol(s)` — this is the SubnetProbe raw ping, not a real libp2p handshake.

**Root Cause Chain (3 compounding failures):**
1. **Blocking FFI on IO thread:** `onLanAddressResolved` (line 938) calls `swarmBridge?.dial(multiaddr)` inside `repoScope.launch`. While `repoScope` uses `Dispatchers.IO` (not Main — corrected from earlier subagent report), the `SwarmBridge::dial()` Rust function calls `rt.block_on(handle.dial(addr))` which blocks the IO thread synchronously.
2. **Missing PeerId:** `SubnetProbe` constructs multiaddrs as `/ip4/$host/tcp/$port` without a `/p2p/<PeerId>` component. `rust-libp2p` dials this as a promiscuous `UnknownPeerId` connection, which can be internally aborted by connection limits or Kademlia routing rules before negotiation starts.
3. **TIME_WAIT socket poisoning:** `SubnetProbe` opens a raw `Socket().connect()`, the Windows CLI receives it, attempts Noise negotiation, fails (raw TCP, not libp2p), and the socket enters `TIME_WAIT`. Milliseconds later, Android fires the real libp2p dial to the same IP:port — the OS may reject the rapid reconnection.

**Fable 5 Deliverables:**
- [ ] `MeshRepository.onLanAddressResolved` must use the existing `open suspend fun dial()` wrapped in `withContext(Dispatchers.IO)` instead of calling the bridge directly
- [ ] Implement PeerId extraction: either a lightweight probe handshake in `SubnetProbe`, or rely purely on mDNS (which already provides PeerId)
- [ ] Add a configurable delay (e.g., 500ms) between the SubnetProbe raw ping and the libp2p dial to let `TIME_WAIT` clear
- [ ] ~30 LOC Kotlin + ~0 LOC Rust (if mDNS-only approach)

---

## PART II: Systemic Codebase Audit Findings

These were discovered by 4 parallel audit agents sweeping Android, CLI, and Rust core for the same anti-patterns that caused the Part I failures.

---

### Issue 5 · 14 Synchronous FFI Functions Blocking IO Threads
**Severity:** P1 — Systemic ANR risk across entire app  
**Component:** `core/src/mobile_bridge.rs` (all `SwarmBridge` methods using `rt.block_on()`)

Every one of these Rust functions calls `rt.block_on()`, which blocks the calling OS thread until the async operation completes. On Android, these are invoked from Kotlin coroutines on `Dispatchers.IO`, tying up threads from the bounded pool:

| Function | Rust Line | Kotlin Impact |
|---|---|---|
| `send_message_status` | 2960 | Blocks during `flushPendingOutbox()` background retries |
| `dial` | 3024 | Blocks in `connectToPeer()`, `triggerFallbackProtocol()`, `onLanAddressResolved` |
| `get_peers` | 3056 | **Tight 100ms poll loop** in `awaitPeerConnection()` (line 6451) — burns IO threads every 100ms |
| `shutdown` | 3142 | Blocks during Service teardown |
| `send_message` | 2922 | Legacy, still exported |
| `send_to_all_peers` | 2981/2992 | Called in `ensureCoverTrafficLoop()` |
| `get_listeners` | 3070 | Synchronous wrapper |
| `get_external_addresses` | 3084 | Called during route building |
| `get_topics` | 3099 | Synchronous wrapper |
| `subscribe_topic` | 3111 | Synchronous wrapper |
| `unsubscribe_topic` | 3123 | Synchronous wrapper |
| `publish_topic` | 3135 | Synchronous wrapper |
| `set_relay_budget` | 1147 | Synchronous wrapper |
| `update_keepalive` | 507 | Not actively used but exported |

**Fable 5 Deliverables:**
- [ ] Convert all 14 functions to `async fn` in Rust. UniFFI will automatically export these as Kotlin `suspend fun`.
- [ ] Audit all Kotlin call sites to ensure they use the suspend variant and remove any `withContext(Dispatchers.IO)` wrappers (no longer needed once async).
- [ ] ~200 LOC Rust refactor + ~100 LOC Kotlin call-site updates

---

### Issue 6 · Blocking I/O Inside Coroutines (Android-Wide)
**Severity:** P2 — Contributes to thread starvation and latent ANRs  

| Location | Violation | Fix |
|---|---|---|
| `MeshRepository.kt` lines 4048, 4075, 4103, 4146, 4171, 4258 | `runBlocking { preferencesRepository?.identityNickname?.firstOrNull() }` — blocks calling thread for DataStore disk I/O | Convert to `suspend` functions or use cached in-memory `_identityInfo.value` |
| `NetworkTypeDetector.kt` line 64 | `Socket().connect()` with 3s timeout, no `withContext(Dispatchers.IO)` wrapper; called from `DiagnosticsReporter.generateReport()` | Wrap in `withContext(Dispatchers.IO)` |
| `BleGattClient.kt` line 389 | `CountDownLatch(1).await()` blocks thread waiting for GATT write callback | Replace with `suspendCancellableCoroutine` |
| `BleGattServer.kt` line 259 | `Thread.sleep(2)` pacing MTU fragment sends inside coroutine scope | Replace with `kotlinx.coroutines.delay(2)` |
| `WifiAwareTransport.kt` lines 347, 408, 449 | `ServerSocket.accept()` blocks `Dispatchers.IO` thread indefinitely | Migrate to `ServerSocketChannel` with NIO selectors |
| `WifiDirectTransport.kt` line 336 | `ServerSocket(P2P_PORT).accept()` blocks `Dispatchers.IO` thread | Same NIO migration |
| `BleL2capManager.kt` line 80 | `BluetoothServerSocket.accept()` in `while(isListening)` loop | Same NIO migration |

**Note:** `MeshVpnService.kt` line 87 uses `Thread.sleep(100)` but this runs on a raw `Thread`, not a coroutine — no fix needed.

**Fable 5 Deliverables:**
- [ ] Fix all 6 `runBlocking` call sites (~30 LOC)
- [ ] Add `withContext(Dispatchers.IO)` to `NetworkTypeDetector` (~5 LOC)
- [ ] Replace `CountDownLatch` → `suspendCancellableCoroutine` in BLE GATT (~30 LOC)
- [ ] Convert `BleGattServer.sendFragmented` to `suspend fun` and replace `Thread.sleep(2)` → `delay(2)` (~15 LOC — requires updating callers, not a 2-line fix)
- [ ] NIO migration for 3 transport accept loops (~120 LOC total, complex)

---

### Issue 7 · Swallowed Errors & Silent Failures (Rust Core)
**Severity:** P1 (crypto), P2 (gossipsub, relay)  

| Location | Error Swallowed | Impact |
|---|---|---|
| `swarm.rs` line 3952 (native) / 5280 (WASM) | `SwarmEvent::ListenerError` — logged, not propagated | Node silently loses inbound connectivity (see Issue 1) |
| `swarm.rs` line 3960 (native) | `SwarmEvent::ListenerClosed` — logged, not propagated | Listener dies silently under OS memory pressure |
| `swarm.rs` lines 4165–4196 (native) / 4548–4570 (WASM) | `SubscribeTopic`, `UnsubscribeTopic`, `PublishTopic` commands lack `reply` channels | Gossipsub `publish()` can fail with `InsufficientPeers`; caller never learns. Silent message drops. |
| **`crypto/session_manager.rs` lines 299, 305** | **`hex::decode_to_slice(&cs.chain_key_hex, &mut ck).ok()`** | **CRITICAL: If DB contains corrupt hex, decoding silently fails. `ck` remains `[0u8; 32]`. A permanently broken "Zombie" cryptographic session is created with a zeroed chain key. All messages to/from this contact silently fail to encrypt/decrypt.** |
| `swarm.rs` line 3011 | `let _ = swarm.behaviour_mut().relay.send_response(...)` | Relay rejection response may fail to send; swallowed |

**Fable 5 Deliverables:**
- [x] **[CRITICAL]** Fix `session_manager.rs` lines 299/305: replace `.ok()` with `.map_err(|e| anyhow!(...))` and propagate the error to fail session initialization loudly (~10 LOC) — **DONE** (verified `cargo check`)
- [ ] Add `reply: mpsc::Sender<Result<(), String>>` to `SubscribeTopic`, `UnsubscribeTopic`, `PublishTopic` SwarmCommands (~40 LOC)
- [ ] Add `ListenerFailed` variant to `SwarmEvent2` and propagate in event loop (~20 LOC, overlaps with Issue 1)

---

### Issue 8 · Missing Circuit Breaker (Windows CLI BLE)
**Severity:** P1 — Already documented as Issue 3  

The BLE Central Ingress loop in `cli/src/ble_mesh.rs` is the **only** retry-loop in the codebase lacking exponential backoff. Other subsystems (WASM self-healing, Kademlia bootstrap) correctly use `BootstrapBackoffEntry`. This issue is fully covered by Issue 3's deliverables above.

---

## Summary: Sprint Planning View

| # | Issue | Severity | Est. LOC | Complexity |
|---|---|---|---|---|
| 1 | TCP Listener Zombie State | P0 | ~70 | Medium — async FFI + SwarmEvent2 variant |
| 2 | SubnetProbe ANR | P0 | ~80 | Medium — NIO refactor, well-scoped |
| 3 | BLE Backoff Circuit Breaker | ~~P1~~ | ~~40~~ | ~~Low~~ — **DONE** |
| 4 | Outbound Dial Failures | P1 | ~30 | Low-Medium — fix callback + mDNS reliance |
| 5 | 14 Sync FFI → Async Migration | P1 | ~300 | **High** — touches every FFI function and all Kotlin call sites |
| 6 | Blocking I/O in Coroutines | P2 | ~200 | Medium — 7 distinct locations, NIO for 3 transports |
| 7 | Swallowed Errors (Core) | ~~P1~~/P2 | ~60 | Medium — reply channels (crypto fix **DONE**) |
| 8 | (Covered by Issue 3) | — | — | — |
| **Total** | | | **~780** | |

---

## Appendix: Files Referenced

| File | Issues |
|---|---|
| `core/src/mobile_bridge.rs` | 1, 4, 5 |
| `core/src/transport/swarm.rs` | 1, 5, 7 |
| `core/src/crypto/session_manager.rs` | 7 |
| `android/.../data/MeshRepository.kt` | 1, 4, 5, 6 |
| `android/.../transport/SubnetProbe.kt` | 2, 4 |
| `android/.../transport/ble/BleGattClient.kt` | 6 |
| `android/.../transport/ble/BleGattServer.kt` | 6 |
| `android/.../transport/ble/BleL2capManager.kt` | 6 |
| `android/.../transport/WifiAwareTransport.kt` | 6 |
| `android/.../transport/WifiDirectTransport.kt` | 6 |
| `android/.../transport/NetworkTypeDetector.kt` | 6 |
| `cli/src/ble_mesh.rs` | 3, 8 |
