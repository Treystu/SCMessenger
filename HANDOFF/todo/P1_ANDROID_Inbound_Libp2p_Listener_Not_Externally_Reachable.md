# TASK: P1-ANDROID-LISTENER-REACHABILITY -- Android's own libp2p inbound listener does not appear to be externally reachable

## Source

Split out of `HANDOFF/todo/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`
("mode A" finding, 2026-07-05 trace-level capture session) plus a follow-up live
retest (2026-07-05/06 native session) that further narrowed the CLI-side "mode B"
symptom to a false positive (see that ticket's latest progress note and
`HANDOFF/review/` once the crypto-security-auditor pass on the log-level fix
lands). With mode B ruled out as the blocker, **this ticket is now the most
likely actual root cause of "phone and Windows CLI cannot message each
other"** and should be treated as the priority investigation.

## Problem (evidence so far, not yet root-caused)

From the 2026-07-05 trace-level capture (`P1_CLI_Transport_Negotiation_Failure...md`,
"(A)" section):

- Windows CLI dialing OUT to the phone's advertised address
  (`/ip4/192.168.0.148/tcp/9001/...`) fails with a plain TCP-level refusal:
  `os error 10061` (`WSAECONNREFUSED`) -- nothing accepted the TCP handshake
  on the phone's side at all. This never reaches Noise/multistream-select,
  ruling out a protocol/negotiation bug for this direction.
- Corroborating evidence this is Android-side, not network/Windows-side:
  Android's own logcat (filtered by the app's PID) shows the phone's own
  libp2p failing to dial **itself** at its own mDNS-advertised address:
  `MdnsServiceDiscovery` resolved its own peer at
  `192.168.0.148:9001` (mDNS self-loopback -- separately tracked in
  `P1_ANDROID_mDNS_Self_Loopback_Discovery.md`), and immediately after:
  `MeshRepository$ensureTransportManager: Failed to dial discovered LAN peer
  /ip4/192.168.0.148/tcp/9001/p2p/...: Network error`.
- Kotlin does explicitly request `meshService?.startSwarm("/ip4/0.0.0.0/tcp/9001",
  listOf())` (`MeshRepository.kt:3051`), so the requested listen address is
  correct.
- `listen_on()` in libp2p returns `Ok` immediately and reports async bind
  failure later via a listener-error swarm event
  (`core/src/transport/swarm.rs:1904-1909` in the single-port branch, per the
  2026-07-05 investigation) -- **that later event has never actually been
  traced/logged to confirm whether the bind itself is failing, succeeding but
  not externally reachable (e.g. Android-specific socket/firewall
  restriction), or something else entirely.** This is the single most
  leveraged unknown blocking Phase 1 device validation.
- Separately confirmed in the same session: Android's `SubnetProbe` (a bare
  `Socket().connect()`, not libp2p) DOES succeed at reaching the Windows
  CLI's ports 9001/9002 -- so the network path phone-to-Windows and Windows's
  own listeners are fine. This failure is specifically about the **phone's
  own inbound listener side**.
- A follow-up worker analysis (2026-07-06) re-confirmed: across the full
  ~12800-line CLI trace capture, no `ConnectionEstablished` ever appears, and
  no distinct real (non-SubnetProbe) inbound dial from the phone was found in
  the capture window either -- consistent with the phone's real libp2p dial
  simply never reaching the CLI, i.e. failing entirely on the Android side
  before it leaves the device, or being silently dropped.

## What needs to happen

1. Add or surface logging on the Android/Rust-core `listen_on()` path
   (`core/src/transport/swarm.rs:~1904-1909`) for the async listener-bind
   outcome specifically -- confirm whether Android's bind to
   `/ip4/0.0.0.0/tcp/9001` actually succeeds at the OS level and produces a
   `NewListenAddr` swarm event, or fails/silently no-ops.
2. If the bind succeeds but the socket isn't externally reachable: investigate
   Android-specific causes -- OS-level per-app network restrictions, whether
   the app has the right permissions/foreground-service type for a listening
   socket, whether Android's network sandboxing (e.g. per-UID firewall rules,
   Doze/App Standby network restrictions) blocks unsolicited inbound
   connections to non-system apps, or whether this needs a specific
   `NETWORK_SETTINGS`/`INTERNET`/foreground-service network exemption.
3. If the bind fails outright: get the actual OS error (`EADDRINUSE`,
   permission denied, etc.) via the listener-error event and fix accordingly.
4. Live device retest (real Pixel 6a, adb, same-LAN CLI) required to close --
   this is fundamentally a live-network bug, not a code-review-only fix.
5. This touches `core/src/transport/` -- mandatory `crypto-security-auditor`
   review before done, per `.claude/rules/security.md`, regardless of how the
   fix lands (logging-only vs. an actual bind/permission fix).

## Files to Touch

- `core/src/transport/swarm.rs` (listener-bind event handling, ~line 1904-1909)
- Possibly `android/app/src/main/AndroidManifest.xml` (permissions/foreground
  service type) if the root cause turns out to be an Android OS restriction
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  (only if the Kotlin-side listen-request call needs adjustment; unlikely
  based on evidence so far -- the request itself looks correct)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core --lib
```

Manual/device verification (required): fresh CLI daemon with
`RUST_LOG=libp2p_swarm=trace,libp2p_tcp=trace,scmessenger_core=debug`,
real Pixel 6a on the same LAN, confirm a `NewListenAddr` event fires on
Android's `0.0.0.0:9001` bind, then confirm the Windows CLI (or another real
peer) can actually complete a TCP handshake + full negotiation +
`ConnectionEstablished` against that address -- not just a SubnetProbe-style
raw connect.

## Do NOT

- Do not assume this is purely a firewall/Windows-Defender-on-the-Windows-side
  issue -- evidence rules out the network path and the Windows CLI's own
  listeners as the cause; the phone's own listener is the object of
  investigation.
- Do not conflate this with the already-fixed `P1_ANDROID_TransportManager_LAN_Discovery_Never_Starts.md`
  (discovery-starting bug, confirmed fixed and verified live) -- this ticket
  is about the listener's reachability once discovery has already worked.
- Do not close `P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`
  as fully resolved based on this ticket alone -- that ticket's own "mode B"
  question needs its separate log-severity fix (in progress, see that file)
  reviewed and merged first.
