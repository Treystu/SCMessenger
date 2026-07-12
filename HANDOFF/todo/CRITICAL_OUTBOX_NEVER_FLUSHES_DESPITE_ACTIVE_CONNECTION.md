# TASK [CRITICAL]: Outbox-queued messages never get delivered even with an active direct connection

Status: TODO. Found 2026-07-12 during the first-ever successful live
Windows-CLI<->Android-emulator connection test this session. This is
arguably the most important finding of the whole session - it's the actual
core promise of a messenger app (message gets delivered) failing silently.

## What was proven working (real progress, do not re-litigate)

- Windows CLI successfully dialed and connected to the Android emulator app
  via `adb forward tcp:19001 tcp:9001` + a manually-configured
  `bootstrap_nodes` entry in `%APPDATA%\scmessenger\config.json` (NOTE:
  `SC_BOOTSTRAP_NODES` env var did NOT work for the CLI's `start` command -
  see separate note below). Log: `Connected to
  12D3KooWSLkR1yNngFGG7mheNM4wbQYRRo4D9599Rwga1gvcVfY7 via
  /ip4/127.0.0.1/tcp/19001 (promiscuous mode)`.
- Android app's own stats confirmed the SAME thing independently:
  `peersDiscovered` went from 0 (stuck the entire rest of this session,
  `Bootstrap all-failed (consecutive=106+)`) to 1, and STAYED at 1 for the
  full ~11 hour connection lifetime observed. This is the first confirmed
  successful transport-layer connection between these two nodes all session.
- Contact was added correctly via the daemon's own `/api/contacts` endpoint
  (`curl -X POST http://127.0.0.1:9876/api/contacts -d
  '{"peer_id":"12D3KooWSLkR1yNngFGG7mheNM4wbQYRRo4D9599Rwga1gvcVfY7",
  "public_key":"f583919e981275d4a9ca669987592d88fb03519630dad7068969f7b1dd651cd4",
  "name":"android-emu-v2"}'`), and `/api/send` returned `{"success":true}`.

## The bug

Despite the message API reporting success, and despite a continuously
active, stable direct connection to the exact recipient for the ENTIRE
observation window (~11 hours), the message never actually left the
sender's outbox:

```
routing_decision message_id=d596a2d9... recipient_hint=f1e95a16 priority=128
  next_hop=RouteDiscovery { hint: [241, 233, 90, 22] } decided_by=StoreAndCarry
  confidence=0.0
outbox_enqueue message_id=d596a2d9-3dfe-49b1-9864-845479c5d649
  recipient_id=f583919e981275d4a9ca669987592d88fb03519630dad7068969f7b1dd651cd4
  queued_at=1783850494996 attempts=0 payload_size=464
```

11 hours later, `grep` for that message_id in the daemon's log still shows
ONLY these two lines - no retry, no delivery attempt, `attempts` never
incremented past 0. Android's own stats confirm zero bytes ever arrived:
`ServiceStats(peersDiscovered=1, messagesRelayed=0, bytesTransferred=0,
uptimeSecs=41577)` at the same time.

**The routing engine decided `StoreAndCarry` with `confidence=0.0` for a
recipient it had an active, healthy, direct libp2p connection to the entire
time.** This strongly suggests the routing/next-hop decision logic
(`core/src/routing/optimized_engine.rs` per the log's module path) is not
correctly correlating "recipient_id" (a canonical/public-key-hex identifier)
with "the peer_id of an already-connected swarm peer" - it should have
recognized the direct connection and delivered immediately, or at minimum
retried against the known-active connection instead of leaving the message
in the outbox indefinitely with zero attempts.

## What to investigate

1. Trace `core/src/routing/optimized_engine.rs`'s routing decision logic:
   why does it not check "is this recipient's underlying peer_id part of an
   active swarm connection" before falling back to `StoreAndCarry`/
   `RouteDiscovery`? Is there a missing mapping from `recipient_id`
   (public-key-hex/canonical ID) to the swarm's connected `PeerId` set?
2. Is there ANY outbox flush/retry mechanism at all? `attempts=0` after 11
   hours suggests either no periodic retry exists, or it exists but is
   itself broken/never triggered. Check `core/src/store/outbox.rs` and
   whatever's supposed to call it periodically.
3. Confirm whether this reproduces with the SAME identifier convention
   throughout - i.e. is part of the problem that `recipient_id` in the
   outbox/routing layer is the public-key-hex while the swarm's connected
   peers are tracked by libp2p `PeerId`, and something in between
   (`extract_public_key_from_peer_id`/reverse lookup) is either missing or
   not being consulted by the routing decision?

## Separate, smaller finding worth folding into cleanup

`SC_BOOTSTRAP_NODES` environment variable (read by
`core/src/transport/bootstrap.rs::resolve_env_bootstrap_nodes`) had NO
effect on the CLI's `start` command - only editing `bootstrap_nodes` in
`~/AppData/Roaming/scmessenger/config.json` directly worked. Worth checking
whether `cmd_start` in `cli/src/main.rs` actually wires `SC_BOOTSTRAP_NODES`
through to whatever consumes `BootstrapManager`, or whether the CLI's own
config-file bootstrap list is a completely separate code path from
`BootstrapManager`/`resolve_env_bootstrap_nodes` (the "promiscuous ledger
dialing" behavior observed in CLI logs looks like a third, distinct
mechanism from both).

## Gate

This is the actual core value proposition of the app (message delivery) -
treat as release-blocking for v1.0.0. Mandatory adversarial review once
fixed (`core/src/routing/`). Verification: repeat this exact live-device
test (Windows CLI <-> Android emulator via adb forward + bootstrap config)
and confirm a sent message is actually received (bytesTransferred > 0 on the
recipient, message visible in the recipient's history) within a reasonable
time, not just accepted by the API.
