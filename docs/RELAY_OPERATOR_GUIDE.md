# Community Relay & Bootstrap Node Operator Guide

> **Version:** v0.1.2-alpha  
> **Last updated:** 2026-02-25

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

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Clients can't connect | Check firewall/port forwarding on TCP 9001 |
| Node shows 0 peers | Verify internet connectivity; check bootstrap config |
| High CPU usage | Check `set_relay_budget` to limit relay throughput |
| PeerId changed | Check if `storage/relay_network_key.pb` was deleted |
