# Quick Connect Guide

Get two nodes talking in under 2 minutes!

## Your Nodes Are Already Running!

**GCP Node:** `34.168.102.7`
- Peer ID: `12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W`
- Multiaddress: `/ip4/34.168.102.7/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W`

## Get Your Node Info (Any Machine)

```bash
# Get all connection info for sharing
./scripts/get-node-info.sh

# Or manually from Docker logs
docker logs scmessenger | grep "Network peer ID"
```

## Connect From Another Machine

### Method 1: Docker with Bootstrap (Easiest)

```bash
docker run -d \
  --name scmessenger-local \
  --platform linux/amd64 \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  -e BOOTSTRAP_NODES="/ip4/34.168.102.7/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W" \
  testbotz/scmessenger:latest

# Watch connection happen
docker logs -f scmessenger-local
```

### Method 2: Add Bootstrap After Starting

```bash
# Start node normally
docker run -d --name scmessenger -p 9000:9000 -p 9001:9001 testbotz/scmessenger:latest

# Stop it
docker stop scmessenger
docker rm scmessenger

# Restart with bootstrap
docker run -d \
  --name scmessenger \
  -p 9000:9000 -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  -e BOOTSTRAP_NODES="/ip4/34.168.102.7/tcp/9001/p2p/12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W" \
  testbotz/scmessenger:latest
```

## Verify Connection

```bash
# On connecting node - should see:
docker logs scmessenger-local
# Output:
# ⚙ Connecting to bootstrap nodes...
#   1. Dialing /ip4/34.168.102.7/tcp/9001/p2p/12D3Koo... ...
#   ✓ Connected to bootstrap node 1
# ✓ Peer: 12D3KooWGGdvGNJb3JwkNpmYuapgk7SAZ4DsBmQsU989yhvnTB8W
```

## Access the UI

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

## Troubleshooting

### Port Already in Use

```bash
# Check what's using the port
lsof -i :9000
lsof -i :9001

# Use different ports
docker run -p 8000:9000 -p 8001:9001 -e LISTEN_PORT=9000 ...
```

### No Connection

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

### Container Won't Start

```bash
# Check logs
docker logs scmessenger

# Clean restart
docker stop scmessenger
docker rm scmessenger
docker pull testbotz/scmessenger:latest
# Then run again
```

## Adding More Nodes

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

## Next Steps

- [Docker Quick Start](DOCKER_QUICKSTART.md) - Complete Docker guide
- [Native Install](INSTALL.md) - Build from source
- [README](README.md) - Architecture and philosophy

---

**Your mesh is ready!** 🎉

Every node you add makes the network stronger. No central servers, no phone numbers, no accounts. Just pure peer-to-peer sovereignty.
