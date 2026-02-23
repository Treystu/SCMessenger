> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# Bootstrap Node Configuration

SCMessenger automatically embeds default bootstrap nodes into all builds (Docker images and native binaries), enabling instant network connectivity without manual configuration.

## [Current] Section Action Outcome (2026-02-23)

- `rewrite`: canonical bootstrap model is env/startup override + dynamic fetch + static fallback.
- `move`: strategic/bootstrap rollout policy is maintained in `docs/UNIFIED_GLOBAL_APP_PLAN.md` and `REMAINING_WORK_TRACKING.md`.
- `keep`: this document remains operator-facing bootstrap usage guidance.
- `delete/replace`: treat default-only interpretations as historical unless reconfirmed in current code paths.

## [Needs Revalidation] How It Works

### [Needs Revalidation] 1. Embedded Defaults

All builds include hardcoded bootstrap nodes in `cli/src/bootstrap.rs`:

```rust
pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[
    "/ip4/34.168.102.7/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W",
    // Additional nodes...
];
```

### [Needs Revalidation] 2. Automatic Merging

When a node starts:
1. **First run**: Creates `config.json` with embedded bootstrap nodes
2. **Subsequent runs**: Merges any new defaults with existing user configuration
3. **Environment variables**: Adds `BOOTSTRAP_NODES` from environment to the merged list
4. **Deduplication**: Ensures no duplicate entries

### [Needs Revalidation] 3. Smart Updates

- Upgrading to a new version automatically adds new bootstrap nodes
- User-added nodes are preserved
- Users can manually remove unwanted nodes: `scm config bootstrap remove <addr>`

## [Needs Revalidation] For End Users

### [Needs Revalidation] Docker

Just run - bootstrap nodes are preconfigured:

```bash
docker run -d \
  --name scmessenger \
  -p 9000:9000 -p 9001:9001 \
  testbotz/scmessenger:latest
```

Watch it connect:

```bash
docker logs -f scmessenger
# ⚙ Connecting to bootstrap nodes...
#   ✓ Connected to bootstrap node 1
```

### [Needs Revalidation] Native Binary

No configuration needed - just start:

```bash
./scmessenger-cli start
# Automatically connects to embedded bootstrap nodes
```

### [Needs Revalidation] Adding Additional Nodes

Environment variable (Docker):

```bash
docker run -d \
  -e BOOTSTRAP_NODES="/ip4/1.2.3.4/tcp/9001/p2p/12D3Koo..." \
  testbotz/scmessenger:latest
```

CLI command (native):

```bash
scmessenger-cli config bootstrap add /ip4/1.2.3.4/tcp/9001/p2p/12D3Koo...
```

### [Needs Revalidation] Viewing Bootstrap Nodes

```bash
# Docker
docker exec scmessenger scm config bootstrap list

# Native
scmessenger-cli config bootstrap list
```

### [Needs Revalidation] Removing Bootstrap Nodes

```bash
# Docker
docker exec scmessenger scm config bootstrap remove /ip4/34.168.102.7/tcp/9001/p2p/...

# Native
scmessenger-cli config bootstrap remove /ip4/34.168.102.7/tcp/9001/p2p/...
```

## [Needs Revalidation] For Developers & Maintainers

### [Needs Revalidation] Adding New Default Bootstrap Nodes

Edit `cli/src/bootstrap.rs`:

```rust
pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[
    "/ip4/34.168.102.7/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W",
    "/ip4/1.2.3.4/tcp/9001/p2p/12D3KooW...",  // New node
    "/ip4/5.6.7.8/tcp/9001/p2p/12D3KooW...",  // Another node
];
```

Rebuild and publish:

```bash
# Native build
cargo build --release

# Docker build
docker build -t testbotz/scmessenger:latest -f docker/Dockerfile .
docker push testbotz/scmessenger:latest
```

Users who upgrade will automatically get the new bootstrap nodes merged into their config.

### [Needs Revalidation] Build-Time Override

Override defaults at build time using environment variable:

```bash
# Native build
export SCMESSENGER_BOOTSTRAP_NODES="/ip4/1.2.3.4/tcp/9001/p2p/12D3Koo...,/ip4/5.6.7.8/tcp/9001/p2p/12D3Koo..."
cargo build --release

# Docker build
docker build \
  --build-arg SCMESSENGER_BOOTSTRAP_NODES="/ip4/1.2.3.4/tcp/9001/p2p/12D3Koo..." \
  -t my-custom-build \
  -f docker/Dockerfile .
```

This is useful for:
- Custom private networks
- Testing with specific bootstrap infrastructure
- Regional deployments with nearby bootstrap nodes

### [Needs Revalidation] Setting Up a Bootstrap Node

Any node can be a bootstrap node. Requirements:

1. **Stable public IP address**
2. **Open firewall ports** (9000, 9001)
3. **Persistent identity** (don't delete data directory)
4. **High availability** (24/7 uptime preferred)

Get your node's multiaddress:

```bash
# If using Docker
docker exec scmessenger scm identity

# If using native binary
scmessenger-cli identity
```

Share the multiaddress with the community or add it to `DEFAULT_BOOTSTRAP_NODES`.

### [Needs Revalidation] Bootstrap Node Strategy

**Geographic Distribution**: Place bootstrap nodes in different regions (North America, Europe, Asia, etc.) to ensure low-latency initial connections for users worldwide.

**Redundancy**: Always have at least 3-5 bootstrap nodes. If one goes down, others provide connectivity.

**Diversity**: Mix cloud providers (GCP, AWS, Azure, DigitalOcean, etc.) to avoid single-provider dependency.

**Community Nodes**: Encourage community members to run stable bootstrap nodes and submit PRs to add them to defaults.

## [Needs Revalidation] Architecture

### [Needs Revalidation] Bootstrap vs Relay

**Bootstrap nodes** help new peers join the network and discover other peers. They are NOT required for ongoing communication.

**All nodes relay**: Every node that can relay does relay. Bootstrap nodes are just well-known entry points.

### [Needs Revalidation] Discovery After Bootstrap

Once connected to a bootstrap node:
1. **DHT Discovery**: Node joins Kademlia DHT and discovers nearby peers
2. **mDNS Discovery**: Finds peers on local network (LAN)
3. **Peer Exchange**: Bootstrap nodes share their peer list
4. **Gossipsub**: Subscribes to mesh network topics

After initial bootstrap, nodes discover each other organically through the DHT. Bootstrap nodes are no longer needed for that session.

### [Needs Revalidation] Security Considerations

- **Bootstrap nodes see connection attempts** but cannot decrypt messages (end-to-end encryption)
- **Bootstrap nodes cannot impersonate peers** (cryptographic identities)
- **Bootstrap nodes can go rogue** (malicious node can refuse connections or provide bad peer info)
  - Mitigation: Multiple bootstrap nodes, automatic failover
- **DDoS risk**: Bootstrap nodes are public and may be targeted
  - Mitigation: Rate limiting, connection limits, use a CDN or DDoS protection

## [Needs Revalidation] Testing

### [Needs Revalidation] Test Bootstrap Configuration

```bash
# Check embedded defaults (in source)
cat cli/src/bootstrap.rs | grep DEFAULT_BOOTSTRAP_NODES -A 10

# Check runtime config (after start)
scmessenger-cli config bootstrap list
```

### [Needs Revalidation] Test Bootstrap Connection

```bash
# Start node with verbose logging
RUST_LOG=debug scmessenger-cli start

# Watch for bootstrap connection logs:
# ⚙ Connecting to bootstrap nodes...
#   1. Dialing /ip4/34.168.102.7/tcp/9001/p2p/12D3Koo... ...
#   ✓ Connected to bootstrap node 1
```

### [Needs Revalidation] Test Bootstrap Merging

```bash
# Start node (creates config with defaults)
scmessenger-cli start

# Stop node (Ctrl+C)

# Add custom bootstrap
scmessenger-cli config bootstrap add /ip4/1.2.3.4/tcp/9001/p2p/12D3Koo...

# Check - should have both default and custom
scmessenger-cli config bootstrap list
```

## [Needs Revalidation] Troubleshooting

### [Needs Revalidation] "No bootstrap nodes configured"

**Cause**: Something went wrong during config initialization.

**Fix**:
```bash
# Regenerate config
rm ~/.config/scmessenger/config.json
scmessenger-cli start
```

### [Needs Revalidation] "Failed to connect to bootstrap nodes"

**Causes**:
1. Bootstrap node is down
2. Firewall blocking connection
3. Invalid multiaddress

**Debug**:
```bash
# Test connectivity
nc -zv 34.168.102.7 9001

# Check firewall
# Linux: sudo ufw status
# macOS: /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Verify multiaddress format
scmessenger-cli config bootstrap list
# Should be: /ip4/<IP>/tcp/9001/p2p/12D3Koo...
```

### [Needs Revalidation] "Peer count stays at 0"

**Possible causes**:
1. All bootstrap nodes are down
2. Your ports (9000, 9001) are not open to internet
3. You're behind strict NAT without relay

**Debug**:
```bash
# Check bootstrap connection
docker logs scmessenger | grep -i bootstrap

# Check listening addresses
docker logs scmessenger | grep "Listening on"

# Test your own port accessibility
# From another machine:
nc -zv <YOUR_PUBLIC_IP> 9001
```

**Fix**:
```bash
# Ensure firewall allows inbound on 9001
# GCP example:
gcloud compute firewall-rules create allow-scmessenger \
  --allow tcp:9000,tcp:9001,udp:9001 \
  --direction=INGRESS
```

## [Needs Revalidation] Contributing Bootstrap Nodes

If you run a stable node with good uptime, consider contributing it as a default bootstrap node:

1. Run your node for at least 1 week with >99% uptime
2. Ensure ports are open and accessible from internet
3. Get your multiaddress: `scmessenger-cli identity`
4. Submit a PR adding it to `cli/src/bootstrap.rs`
5. Include location, provider, and your contact info in PR description

**Example PR**:

```rust
pub const DEFAULT_BOOTSTRAP_NODES: &[&str] = &[
    // North America - GCP (maintained by @user1)
    "/ip4/34.168.102.7/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W",

    // Europe - AWS (maintained by @user2)
    "/ip4/1.2.3.4/tcp/9001/p2p/12D3KooW...",

    // Asia - DigitalOcean (maintained by @user3)
    "/ip4/5.6.7.8/tcp/9001/p2p/12D3KooW...",
];
```

---

**Key Insight**: Bootstrap nodes are just friendly entry points. The mesh is the network. Every node strengthens the whole.
