> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

# Quick Connect Guide

Get two nodes talking in under 2 minutes!

## [Needs Revalidation] 🎉 Zero Configuration Required!

**Bootstrap nodes are now embedded in all downloads.** Just run the app and you're connected to the mesh instantly. No manual setup needed!

## [Needs Revalidation] Your Nodes Are Already Running!

**GCP Node:** `34.168.102.7`
- Peer ID: `12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W`
- Multiaddress: `/ip4/34.168.102.7/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W`

## [Needs Revalidation] Get Your Node Info (Any Machine)

```bash
# Get all connection info for sharing
./scripts/get-node-info.sh

# Or manually from Docker logs
docker logs scmessenger | grep "Peer ID"
```

## [Needs Revalidation] Connect From Another Machine

### [Needs Revalidation] Method 1: Docker with Embedded Bootstraps (Easiest!)

**Bootstrap nodes are already configured!** Just run:

```bash
docker run -d \
  --name scmessenger-local \
  --platform linux/amd64 \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  testbotz/scmessenger:latest

# Watch automatic connection happen
docker logs -f scmessenger-local
```

That's it! The node automatically connects to embedded bootstrap nodes.

### [Needs Revalidation] Method 2: Add Additional Bootstrap Nodes (Optional)

To connect to specific nodes in addition to defaults:

```bash
docker run -d \
  --name scmessenger-local \
  -p 9000:9000 -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  -e BOOTSTRAP_NODES="/ip4/1.2.3.4/tcp/9001/p2p/12D3KooW..." \
  testbotz/scmessenger:latest
```

This adds your custom node to the embedded defaults.

## [Needs Revalidation] Verify Connection

```bash
# On connecting node - should see:
docker logs scmessenger-local
# Output:
# ⚙ Connecting to bootstrap nodes...
#   1. Dialing /ip4/34.168.102.7/tcp/9001/p2p/12D3Koo... ...
#   ✓ Connected to bootstrap node 1
# ✓ Peer: 12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W
```

## [Needs Revalidation] Access the UI

Once connected, open in your browser:

```
http://localhost:9000
```

**Features:**
- ✅ Real-time peer discovery
- ✅ Encrypted messaging
- ✅ Contact management
- ✅ Automatic reconnection
- ✅ Offline message queue

## [Needs Revalidation] Troubleshooting

### [Needs Revalidation] Port Already in Use

```bash
# Check what's using the port
lsof -i :9000
lsof -i :9001

# Use different ports
docker run -p 8000:9000 -p 8001:9001 -e LISTEN_PORT=9000 ...
```

### [Needs Revalidation] No Connection

```bash
# Test connectivity
nc -zv 34.168.102.7 9001

# Check firewall (GCP)
gcloud compute firewall-rules list | grep scm

# Create rule if missing
gcloud compute firewall-rules create allow-scmessenger \
  --allow tcp:9000,tcp:9001,udp:9001 \
  --direction=INGRESS
```

### [Needs Revalidation] Container Won't Start

```bash
# Check logs
docker logs scmessenger

# Clean restart
docker stop scmessenger
docker rm scmessenger
docker pull testbotz/scmessenger:latest
# Then run again
```

## [Needs Revalidation] Adding More Nodes

Once you have 2 nodes connected, add a third:

```bash
# On third machine, use EITHER node as bootstrap:
docker run -d \
  --name scmessenger \
  -p 9000:9000 -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  -e BOOTSTRAP_NODES="/ip4/<NODE1_IP>/tcp/9001/p2p/<NODE1_PEER_ID>" \
  testbotz/scmessenger:latest
```

The third node will discover both nodes through the DHT!

## [Needs Revalidation] Next Steps

- [Docker Quick Start](DOCKER_QUICKSTART.md) - Complete Docker guide
- [Native Install](INSTALL.md) - Build from source
- [README](README.md) - Architecture and philosophy

---

**Your mesh is ready!** 🎉

Every node you add makes the network stronger. No central servers, no phone numbers, no accounts. Just pure peer-to-peer sovereignty.
