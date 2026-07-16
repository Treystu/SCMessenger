# P1-ANDROID-FABLE-5-DISCOVERY-REPORT -- Comprehensive App Pairing Discovery

## Context
During a live device test and full cross-platform log debug on 2026-07-05, we investigated the root causes preventing the Android app and Windows CLI from pairing. This report documents the complete state of the networking stack and the specific failures preventing transport negotiation. 

This is handed off to Fable 5 for a unified networking logic overhaul.

## 1. Android TCP Listener Fails to Bind (The Primary Blocker)
**Finding:** The Android app does **not** bind to TCP port 9001 (`2329` in hex) even when the app is fully foregrounded with the screen ON. 
**Root Cause (Deep Dive):** 
- In `MeshService::start`, the Android state machine synchronously transitions to `ServiceState::Running` before touching the network.
- `initializeAndStartSwarm()` fires a command across the FFI to Rust, which spawns a detached OS thread (`start_swarm`) and immediately returns `Ok(())`. 
- The Rust swarm calls `listen_on(addr)`, but OS-level socket binding is asynchronous. If the OS rejects the bind (e.g., port in use), libp2p emits `SwarmEvent::ListenerError`.
- **The Fatal Flaw:** The Rust event loop catches `ListenerError`, logs it to `tracing::error!`, and **swallows it**. It does not abort the swarm or notify Android via the bridge channel. The Android node becomes a "zombie"—incapable of inbound connections but superficially appearing "RUNNING".
**Fable 5 Action:** 
- `start_swarm_with_config` must internally use a `oneshot` channel to `await` the first `SwarmEvent::NewListenAddr` before returning success to Android.
- `MeshService::start_swarm` must be an async UniFFI function that Kotlin `await`s.
- `ListenerError` must trigger a `FatalError` payload over the bridge to downgrade Android's state machine.

## 2. SubnetProbe ANR on Manual Rescan
**Finding:** Triggering a "Rescan" from the "Nearby" contacts UI causes an ANR (Application Not Responding) crash.
**Root Cause (Deep Dive):** 
- `SubnetProbe.kt` launches thousands of `scope.async(Dispatchers.IO)` coroutines.
- It uses a blocking Java `java.util.concurrent.Semaphore`, immediately exhausting the 64-thread `Dispatchers.IO` pool as threads wait for permits.
- The permitted threads execute `java.net.Socket.connect()`, which is a blocking I/O call.
**Fable 5 Action (NIO Refactor):** 
- Replace the Java Semaphore with `kotlinx.coroutines.sync.Semaphore` (`sem.withPermit`).
- Replace `Socket().connect()` with a `suspendCancellableCoroutine` wrapping `java.nio.channels.AsynchronousSocketChannel`, using a `CompletionHandler` to resume the coroutine cleanly without blocking underlying IO threads.

## 3. Windows BLE Subscription Loop (GattCommunicationStatus: 1)
**Finding:** The Windows CLI daemon is trapped in an endless failure loop trying to connect to the Android device over BLE, spamming `GattCommunicationStatus(1)` (Unreachable).
**Root Cause (Deep Dive):** 
- In `cli/src/ble_mesh.rs`, the CLI tracks active connections in a `HashSet<String>`. 
- When a subscription attempt fails, the background task immediately removes the peripheral MAC from the set.
- The main event loop instantly receives another BLE advertisement for the unreachable device, sees it missing from the set, and respawns the connection attempt, creating an infinite spin-loop.
**Fable 5 Action:** 
- Replace the `HashSet<String>` with a `HashMap<String, ConnectionState>`.
- Implement an Exponential Backoff circuit breaker. On failure, set a `cooldown_until` timestamp. The main loop must ignore advertisements from that MAC until the cooldown expires (capping at ~60 seconds).

## 4. Android Outbound Dialing (SubnetProbe vs. Real libp2p)
**Finding:** Android successfully discovers Windows via `SubnetProbe`, but never actually establishes a libp2p connection.
**Root Cause (Deep Dive):** 
- **Blocking FFI on IO Dispatcher:** `SubnetProbe` delegates the discovered IP to `MeshRepository` via `onLanAddressResolved`, which invokes `swarmBridge?.dial(multiaddr)` inside a `repoScope.launch` coroutine. While `repoScope` uses `Dispatchers.IO` (not Main), the `SwarmBridge::dial()` Rust function calls `rt.block_on(handle.dial(addr))` which blocks the IO thread synchronously, tying up threads from the bounded pool during the dial attempt.
- **UnknownPeerId Rejection:** The dialed `multiaddr` lacks a `/p2p/` PeerID component. `rust-libp2p` dials this as a "promiscuous" `UnknownPeerId` dial, which can be aborted internally if routing or security limits trigger before negotiation.
- **TIME_WAIT Socket Interference:** `SubnetProbe`'s raw `Socket().connect()` hits the Windows CLI, generating a "negotiation failed" error on the CLI, then instantly closes (sending FIN/RST). Milliseconds later, the actual libp2p dial fires from Android to the exact same IP/port. The Windows OS TCP stack or CLI connection limits likely reject this rapid reconnection while the previous socket state is still tearing down (`TIME_WAIT`).
**Fable 5 Action:** 
- `MeshRepository` must use its suspending `open suspend fun dial()` method wrapped in `withContext(Dispatchers.IO)` rather than calling the bridge directly from the UI thread.
- Extract the `PeerId` over the raw socket during discovery, or rely purely on mDNS (which provides the PeerId), so the swarm can execute a fully qualified dial.
- Implement a delay or handshake in `SubnetProbe` so the raw ping doesn't immediately poison the TCP socket just milliseconds before the real libp2p dial attempts to use the same port.
