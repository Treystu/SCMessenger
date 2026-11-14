# Node Operator Guide

> **Status:** Current
> **Last updated:** 2026-07-25

## Overview

This guide covers running a SCMessenger node that is well-connected and
publicly reachable -- on a cloud VM, a home server, or a Raspberry Pi.

**This is not a special role.** SCMessenger has no dedicated relays and no
bootstrap node tier. There are only nodes, and **every node is a full relay**:
every build starts a libp2p relay server and relay client unconditionally, on
every platform, and advertises itself as a relay via libp2p `identify`. Your
phone relays for other people right now. The only thing that distinguishes the
node described here is that it has a **stable, reachable address**, which makes
it useful as a cold-start entry point and as a relay hop for peers behind
restrictive NAT.

Nothing here grants privilege. A node with a public address cannot read message
contents, cannot impersonate peers, and is not enrolled in any shipped list. See
`docs/BOOTSTRAP.md` for how peers actually find each other and
`docs/BOOTSTRAP_GOVERNANCE.md` for the trust model.

## Full Node vs Headless Node

Two ways to run, and the difference is smaller than it looks:

```bash
# Full node -- user identity, interactive console, messaging
./target/release/scm start --port 9001

# Headless node -- no interactive console
./target/release/scm relay \
  --listen /ip4/0.0.0.0/tcp/9001 \
  --http-port 9000 \
  --name my-node
```

The `relay` subcommand is **not a distinct role**. It is effectively
`start --headless --no-tty`. Both build the identical libp2p behaviour and both
relay. The functional differences are:

- `headless = true`, which changes only the `identify` agent string from
  `scmessenger/<ver>/full/relay/<peer_id>` to
  `scmessenger/<ver>/headless/relay/<peer_id>`,
- no interactive console, so it is safe under systemd,
- an HTTP status/landing page on `--http-port` (default 9000),
- a different default listen address (`--listen`, default `/ip4/0.0.0.0/tcp/0`),
- the network keypair is persisted at `<storage>/relay_network_key.pb`, which is
  what keeps the PeerId stable across restarts.

Flags are exactly `--listen`, `--http-port`, `--name` for `relay`, and `--port`
for `start`. There is no `--ws-port` flag on either.

A headless node can gain an identity later without reinstalling:

```bash
scm init --name "<nickname>"
scm start --port 9001
```

GUI clients (Android, iOS, Desktop/WASM) offer the same choice at first run --
generate an identity now, or skip and run relay-only -- and can be promoted later
from Settings.

## Quick Start (Docker)

```bash
git clone https://github.com/Sovereign-Communication/SCMessenger.git
cd SCMessenger

docker compose -f docker/docker-compose.yml up -d
```

> [WARNING] Seed peer addresses are passed via the **`SC_BOOTSTRAP_NODES`**
> environment variable. Some compose files under `docker/` still set an
> unprefixed `BOOTSTRAP_NODES`, which nothing in the codebase reads -- it is a
> silent no-op. Verify the variable name before relying on it.

## Manual Setup (Binary)

### Prerequisites

- Rust 1.75+ (stable toolchain)
- Linux (x86_64 or aarch64) or macOS (arm64 or x86_64)
- Inbound TCP on the P2P port reachable from the internet

### Build

```bash
cargo build --release -p scmessenger-cli
```

## Cloud Deployment (GCP Example)

Substitute your own project, zone, and ports. Do not copy addresses from
documentation -- every deployment's address is its own.

```bash
# Create a VM
gcloud compute instances create scm-node \
  --machine-type=e2-micro \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --tags=scm-node

# Allow inbound P2P and HTTP status
gcloud compute firewall-rules create allow-scm \
  --allow=tcp:9001,tcp:9000 \
  --target-tags=scm-node
```

### Systemd Service

Create `/etc/systemd/system/scm-node.service`:

```ini
[Unit]
Description=SCMessenger Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=scm
WorkingDirectory=/opt/scm
ExecStart=/opt/scm/scm relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable scm-node
sudo systemctl start scm-node
```

## Self-Hosted / Home Server

### Requirements

- Any Linux device (Raspberry Pi, NAS, old laptop)
- P2P port forwarded on your router, or a router with working UPnP
- Static IP or dynamic DNS recommended

### Steps

1. Build or download the binary for your architecture.
2. Forward inbound TCP on the P2P port to the host.
3. Start the node.
4. Read the node's address, and hand it to the clients that should use it as
   their cold-start entry point:

```bash
export SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>"
```

`<NODE_IP>` and `<PEER_ID>` come from your own node -- see Monitoring below.

Note that UPnP and peer-observed address advertisement are built in, so a node
behind a cooperative router may become reachable without manual forwarding. A
node that never becomes reachable still participates fully; it just cannot serve
as an entry point for others.

### Low-Resource Configuration

- Headless operation uses roughly 30-50 MB RAM under normal load.
- CPU is negligible for relay-only traffic.
- Disk is minimal in headless mode -- relay state only, no message storage.

## Pointing Clients at Your Node

### Environment variable

```bash
export SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>"
```

`SC_BOOTSTRAP_NODES` is the only spelling the code reads. Comma-separate multiple
addresses.

### CLI config

```bash
scmessenger-cli config set bootstrap_node_add /ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>
scmessenger-cli config get bootstrap_nodes
```

There is no `config bootstrap` subcommand -- `config` takes `set`, `get`, and
`list` only.

### Mobile

Android and iOS find peers on the local network automatically. For internet
reachability they ingest a join bundle through the Join Mesh QR flow. Generate the
bundle from an already-connected node rather than asking users to type a
multiaddr.

Only the first connection needs this. After that, the client learns peers over
`/sc/ledger-exchange/1.0.0` and persists them, so your node stops being
load-bearing for that client.

## Monitoring

### Identity and addresses

```bash
# Native
scmessenger-cli identity

# Docker
docker exec scmessenger scm identity

# Confirm what the node is actually listening on
journalctl -u scm-node | grep "Listening on"
```

### HTTP status page

```bash
curl http://<NODE_IP>:9000/
```

Serves the node status/landing page on the `--http-port` value.

### CLI status

```bash
scmessenger-cli status

# Or, in an interactive `start` session:
# > status
# > peers
```

### Logs

```bash
journalctl -u scm-node -f

# Docker
docker compose logs -f
```

## Topology Best Practices

1. **Stable addresses.** Use a static IP or a DNS name. Every address you hand
   out goes stale when it changes, and it propagates through peers' ledgers
   before it does.
2. **Key persistence.** Do not delete `<storage>/relay_network_key.pb`. If the
   PeerId rotates, every ledger entry pointing at this node is wrong.
3. **Redundancy for your own users.** If people depend on your deployment for
   cold start, run more than one reachable node, in separate regions and
   preferably separate providers. This is redundancy for *your* infrastructure --
   there is no project-wide entry tier to contribute to.
4. **Firewall minimally.** Inbound TCP on the P2P port (9001 by default) and,
   optionally, the HTTP status port (9000).
5. **Persist the data directory.** In Docker, mount a volume; otherwise identity
   and ledger are lost on every recreate.

## Security Considerations

These apply to every node, because every node relays:

- Relaying **cannot** read message contents -- everything is end-to-end
  encrypted.
- Relaying **can** observe transport metadata: source and destination PeerIds,
  message sizes, timing.
- Headless nodes do not store messages -- they forward in real time.
- The relay budget (max messages/hour, `max_relay_budget` in settings, applied
  via `set_relay_budget`) is configurable to bound abuse.
- A publicly reachable node is a DDoS target. Use connection caps, rate limits,
  and provider-level protection.

## Legacy Pending Outbox Triage (No-Give-Up Safe)

SCMessenger intentionally keeps no terminal retry exhaustion for queued outbound
messages. High-attempt legacy entries are expected during unstable network
windows and should be triaged, not dropped.

Recommended triage flow:

1. Confirm service/runtime health first -- reachability, peer count, recent
   reconnects.
2. Inspect pending outbox age and attempt distribution.
3. Separate old/high-attempt entries from fresh entries in diagnostics exports.
4. Keep retries enabled. Do not manually delete pending outbox files except as
   part of a full reset.

Android inspection:

```bash
adb shell run-as com.scmessenger.android cat files/pending_outbox.json
adb logcat -d | rg "delivery_state|Flushing pending outbox|Core-routed delivery failed|Relay-circuit retry failed"
```

iOS simulator inspection:

```bash
APP_DATA=$(xcrun simctl get_app_container booted SovereignCommunications.SCMessenger data)
cat "$APP_DATA/Documents/pending_outbox.json"
xcrun simctl spawn booted log show --style compact --last 15m --predicate 'process == "SCMessenger"'
```

Interpretation:

- `attempt_count` high and `created_at` old: legacy backlog item; keep for
  eventual-delivery semantics.
- Repeated `stored` -> `forwarding` cycles with growing backoff: expected under
  intermittent path availability.
- No queue movement and no dial/relay activity: treat as a connectivity or
  runtime issue first, not message corruption.

## Cross-Platform Receipt Convergence Assertion

Use this runbook when validating Android<->iOS fallback behavior under degraded
internet routing.

1. Capture synchronized UTC timestamps and start logs on both devices.
2. Send one message Android -> iOS and one iOS -> Android while the internet
   route is degraded.
3. For each message ID, require both:
   - recipient ingest marker (`msg_rx_processed`), and
   - sender delivered marker (`delivery_state ... state=delivered`).
4. If either marker is missing after the retry delay windows, classify as a
   convergence failure and capture the artifact bundle.

Android capture:

```bash
adb shell date -u
adb logcat -v threadtime | rg "delivery_attempt|delivery_state|msg_rx_processed|Core-routed delivery failed|Relay-circuit retry failed"
```

iOS simulator capture:

```bash
xcrun simctl spawn booted date -u
xcrun simctl spawn booted log stream --style compact --predicate 'process == "SCMessenger"'
```

Pass criteria per direction (A->iOS, iOS->A):

- same `msg=<id>` appears with `delivery_attempt` timeline entries,
- recipient shows `msg_rx_processed`,
- sender shows `state=delivered` without duplicate terminal oscillation.

Fail criteria:

- repeated retry loops without `msg_rx_processed`,
- recipient ingest observed but sender never reaches `delivered`,
- conflicting terminal states for the same message ID after the retry window.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Clients cannot connect | Check firewall and router port forwarding on the P2P port (TCP 9001 by default) |
| Node shows 0 peers | Verify internet connectivity and that the node is listening; if the ledger is empty and there are no LAN peers, supply one seed address (`docs/BOOTSTRAP.md`) |
| High CPU usage | Lower the relay budget to cap relay throughput |
| PeerId changed | Check whether `<storage>/relay_network_key.pb` was deleted or the data directory was not persisted |
| Address handed out no longer works | The node's IP or PeerId changed; reissue the address and expect stale ledger entries to age out |
