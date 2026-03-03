# Community Relay & Bootstrap Node Operator Guide

> **Version:** v0.2.0-alpha  
> **Last updated:** 2026-03-03

## Overview

SCMessenger is designed as a community-operated mesh network. Anyone can run a
relay/bootstrap node to strengthen the network. This guide covers both
cloud-hosted and self-hosted setups.

## Quick Start (Docker)

The fastest way to run a relay node:

```bash
# Clone the repo
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger

# Build and run with Docker Compose
docker compose -f docker-compose.yml up -d
```

This starts a headless relay node on port 9001 (TCP).

## Manual Setup (Binary)

### Prerequisites

- Rust 1.75+ (stable toolchain)
- Linux (x86_64 or aarch64) or macOS (arm64 or x86_64)
- TCP port 9001 accessible from the internet

### Build

```bash
cargo build --release -p scmessenger-cli
```

### Run as Relay

```bash
./target/release/scm relay \
  --listen /ip4/0.0.0.0/tcp/9001 \
  --http-port 8080
```

The `relay` command starts the node in **headless mode**:
- No user identity is created
- Functions purely as a relay and bootstrap node
- Stable PeerId persisted in `storage/relay_network_key.pb`
- HTTP landing page + API on the specified HTTP port

### Run as Full Node

```bash
./target/release/scm start \
  --port 9001 \
  --ws-port 8080
```

The `start` command runs a **full node** with user identity and messaging.

## Install-Mode Choice Parity

SCMessenger now supports the same install-mode intent across GUI and operator flows:

- GUI clients (iOS, Android, Desktop/WASM) show first-run choice:
  - `Generate Identity Now`
  - `Skip for Relay-Only Install`
- Relay-only installs can be promoted later without reinstall from Settings -> Identity.

For CLI/Docker/operator usage, the same choice is command-driven:

- Relay-only mode:
  ```bash
  scm relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 8080
  ```
- Full mode with identity (at any later time, no reinstall):
  ```bash
  scm init --name "<nickname>"
  scm start --port 9001
  ```

This preserves headless relay-first deployments while allowing later identity enablement on the same node.

## Cloud Deployment (GCP Example)

### Instance Setup

```bash
# Create a VM
gcloud compute instances create scm-relay \
  --machine-type=e2-micro \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --tags=scm-relay

# Allow TCP 9001
gcloud compute firewall-rules create allow-scm \
  --allow=tcp:9001,tcp:8080 \
  --target-tags=scm-relay
```

### Systemd Service

Create `/etc/systemd/system/scm-relay.service`:

```ini
[Unit]
Description=SCMessenger Relay Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=scm
WorkingDirectory=/opt/scm
ExecStart=/opt/scm/scm relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 8080
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable scm-relay
sudo systemctl start scm-relay
```

## Self-Hosted / Home Server Setup

### Requirements

- Any Linux device (Raspberry Pi, NAS, old laptop)
- Port 9001 forwarded on your router
- Static IP or dynamic DNS recommended

### Steps

1. Build or download the binary for your architecture
2. Forward TCP port 9001 on your router to the host
3. Run the relay command
4. Add your public IP to client bootstrap config:

```bash
export SC_BOOTSTRAP_NODES="/ip4/YOUR_PUBLIC_IP/tcp/9001"
```

### Low-Resource Configuration

For devices with limited RAM (e.g., Raspberry Pi):

- The relay binary uses ~30-50 MB RAM under normal load
- CPU usage is negligible for relay-only operation
- Disk usage is minimal (relay state only, no message storage)

## Connecting Clients to Your Relay

### Option 1: Environment Variable

```bash
export SC_BOOTSTRAP_NODES="/ip4/YOUR_IP/tcp/9001"
```

### Option 2: Mobile App Configuration

On Android/iOS, the bootstrap nodes can be configured in Settings → Network.
Add your relay's multiaddr: `/ip4/YOUR_IP/tcp/9001`

### Option 3: Remote Bootstrap URL

Host a JSON file at a URL:

```json
["/ip4/YOUR_IP/tcp/9001", "/ip4/BACKUP_IP/tcp/9001"]
```

Configure clients to fetch from this URL on startup.

## Monitoring

### HTTP Health Check

```bash
curl http://YOUR_IP:8080/
```

Returns the relay landing page with node info, peer count, and uptime.

### CLI Status

```bash
# If running interactively
> status
> peers
```

### Logs

```bash
# Systemd logs
journalctl -u scm-relay -f

# Or check the process stdout for structured tracing output
```

## Network Topology Best Practices

1. **Geographic distribution:** Run nodes in different regions for resilience.
2. **Multiple bootstrap nodes:** Configure clients with 2-3 relay addresses.
3. **Stable IPs:** Use static IPs or DNS names for bootstrap nodes.
4. **Key persistence:** The relay automatically persists its network key.
   Do not delete `storage/relay_network_key.pb` — clients cache the PeerId.
5. **Firewall:** Only TCP 9001 (P2P) and 8080 (HTTP, optional) need to be open.

## Security Considerations

- Relay nodes **cannot read message contents** — all messages are end-to-end encrypted.
- Relay nodes can see transport metadata (source/destination PeerIds, message sizes).
- Relay nodes do not store messages — they forward in real-time.
- The relay budget (max messages/hour) is configurable to prevent abuse.

## Legacy Pending Outbox Triage (No-Give-Up Safe)

SCMessenger v0.2.0 intentionally keeps no terminal retry exhaustion for queued outbound messages.
High-attempt legacy entries are expected in unstable network windows and should be triaged, not dropped.

Recommended operator triage flow:

1. Confirm service/runtime health first (relay reachability, peer count, and recent reconnects).
2. Inspect pending outbox age + attempt distribution.
3. Separate old/high-attempt entries from fresh entries in diagnostics exports.
4. Keep retries enabled; do not manually delete pending outbox files unless doing a full reset procedure.

Android inspection commands:

```bash
adb shell run-as com.scmessenger.android cat files/pending_outbox.json
adb logcat -d | rg "delivery_state|Flushing pending outbox|Core-routed delivery failed|Relay-circuit retry failed"
```

iOS simulator inspection commands:

```bash
APP_DATA=$(xcrun simctl get_app_container booted SovereignCommunications.SCMessenger data)
cat "$APP_DATA/Documents/pending_outbox.json"
xcrun simctl spawn booted log show --style compact --last 15m --predicate 'process == "SCMessenger"'
```

Operational interpretation:

- `attempt_count` high and `created_at` old: legacy backlog item; keep for eventual delivery semantics.
- repeated `stored` -> `forwarding` cycles with growing backoff: expected under intermittent path availability.
- no movement in queue and no dial/relay activity: treat as connectivity/runtime issue first (not message corruption).

## Cross-Platform Receipt Convergence Assertion

Use this deterministic runbook when validating Android<->iOS fallback behavior under degraded internet routing.

1. Capture synchronized UTC timestamps and start logs on both devices.
2. Send one message Android -> iOS and one message iOS -> Android while internet route is degraded.
3. For each message ID, require both:
   - recipient ingest marker (`msg_rx_processed`), and
   - sender delivered marker (`delivery_state ... state=delivered`).
4. If either marker is missing for a message ID after retry delay windows, classify as convergence failure and capture the artifact bundle.

Android capture commands:

```bash
adb shell date -u
adb logcat -v threadtime | rg "delivery_attempt|delivery_state|msg_rx_processed|Core-routed delivery failed|Relay-circuit retry failed"
```

iOS simulator capture commands:

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
- conflicting terminal states for the same message ID after retry window.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Clients can't connect | Check firewall/port forwarding on TCP 9001 |
| Node shows 0 peers | Verify internet connectivity; check bootstrap config |
| High CPU usage | Check `set_relay_budget` to limit relay throughput |
| PeerId changed | Check if `storage/relay_network_key.pb` was deleted |
