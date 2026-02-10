# SCMessenger Docker Quick Start

This guide gets you up and running with SCMessenger in Docker in under 5 minutes.

## Prerequisites

- Docker and Docker Compose installed
- Ports 9000 and 9001 available (or configure different ports)

## Single Node Setup

The simplest way to run SCMessenger:

```bash
# 1. Build the image
docker compose build

# 2. Start the node
docker compose up -d

# 3. View logs
docker compose logs -f

# 4. Access the interactive CLI
docker compose exec scmessenger bash -c "scm identity"
```

## GCP or Cloud Deployment

### One-Command Deploy

```bash
# Build and run on a GCP VM or any cloud server
docker build -t scmessenger -f docker/Dockerfile .

# Run with persistent storage
docker run -d \
  --name scmessenger \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  -v ~/scm_config:/root/.config/scmessenger \
  -e LISTEN_PORT=9000 \
  scmessenger

# View your identity and Peer ID
docker logs scmessenger
```

### With Bootstrap Nodes

To connect your node to an existing network, add bootstrap nodes:

```bash
# Your GCP node's multiaddress format:
# /ip4/<PUBLIC_IP>/tcp/9001/p2p/<PEER_ID>

# Example with bootstrap node
docker run -d \
  --name scmessenger \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  -e LISTEN_PORT=9000 \
  -e BOOTSTRAP_NODES="/ip4/136.117.121.95/tcp/9001/p2p/12D3KooWGhWrfkwWRxmskC8bfGGvhd3gHYBQgigRbJeZL9Yd3W2S" \
  scmessenger
```

## Connect Two Nodes (Local + Cloud)

### Step 1: Start your cloud node (GCP)

```bash
# On your GCP VM
docker run -d \
  --name scmessenger \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  scmessenger

# Get the Peer ID from logs
docker logs scmessenger | grep "Network peer ID"
# Example output: ✓ Network peer ID: 12D3KooWGhWrfkwWRxmskC8bfGGvhd3gHYBQgigRbJeZL9Yd3W2S

# Get your public IP
curl ifconfig.me
# Example output: 136.117.121.95
```

**Your cloud node's multiaddress:**
```
/ip4/<YOUR_PUBLIC_IP>/tcp/9001/p2p/<YOUR_PEER_ID>
```

### Step 2: Start your local node (Mac/Linux)

```bash
# On your local machine
docker run -d \
  --name scmessenger-local \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data_local:/root/.local/share/scmessenger \
  -e BOOTSTRAP_NODES="/ip4/136.117.121.95/tcp/9001/p2p/12D3KooWGhWrfkwWRxmskC8bfGGvhd3gHYBQgigRbJeZL9Yd3W2S" \
  scmessenger

# Check logs - you should see "Connected to bootstrap node"
docker logs -f scmessenger-local
```

### Step 3: Verify Connection

```bash
# On local machine - check peer count
docker exec scmessenger-local scm status

# You should see "Peers: 1" or more
```

## Without Docker (Native Binary)

If you prefer to run the binary directly:

```bash
# Build from source
cargo build --release --bin scmessenger-cli

# Start node
./target/release/scmessenger-cli start --port 9000

# Add bootstrap node
./target/release/scmessenger-cli config bootstrap add \
  /ip4/136.117.121.95/tcp/9001/p2p/12D3KooWGhWrfkwWRxmskC8bfGGvhd3gHYBQgigRbJeZL9Yd3W2S

# Restart to connect
./target/release/scmessenger-cli start --port 9000
```

## Port Configuration

SCMessenger uses **two ports** by default:

- **Port 9000**: WebSocket interface (for web UI and API)
- **Port 9001**: P2P network communication (automatically set to `--port + 1`)

When you specify `--port 9000`, the P2P port becomes 9001.

**Both ports must be open in your firewall for internet connectivity.**

### GCP Firewall Example

```bash
gcloud compute firewall-rules create allow-scmessenger \
  --allow tcp:9000,udp:9000,tcp:9001,udp:9001 \
  --description="SCMessenger P2P and WebSocket traffic" \
  --direction=INGRESS
```

## Data Persistence

Your identity and messages are stored in:
- **Linux/Mac**: `~/.local/share/scmessenger/storage/`
- **Docker**: Mounted volume (e.g., `~/scm_data/`)

**Important**: The network keypair (which determines your Peer ID) is now persisted in:
- `network_keypair.dat` - Your Peer ID will remain constant across restarts

## Troubleshooting

### Peer Count Stays at 0

**Check 1**: Verify both nodes show "Listening on" messages:
```bash
docker logs scmessenger | grep "Listening on"
```

**Check 2**: Ensure firewall ports are open:
```bash
# Test from another machine
nc -zv <YOUR_PUBLIC_IP> 9001
```

**Check 3**: Verify bootstrap address format:
```
/ip4/<IP>/tcp/9001/p2p/<PEER_ID>
```
All three components must be correct.

**Check 4**: Check bootstrap connection logs:
```bash
docker logs scmessenger | grep -i bootstrap
```

### Identity Changes on Restart

This should be fixed now! The network keypair is persisted. If you still see changing Peer IDs:
- Ensure your volume mount is working (`-v ~/scm_data:/root/.local/share/scmessenger`)
- Check that `network_keypair.dat` exists in the data directory

### Port Already in Use

```bash
# Find what's using the port
lsof -i :9000
lsof -i :9001

# Kill the process or change ports
docker run -p 8000:9000 -p 8001:9001 -e LISTEN_PORT=9000 scmessenger
```

## Commands Reference

### Docker

```bash
# Start node
docker compose up -d

# Stop node
docker compose down

# View logs
docker compose logs -f

# Restart node
docker compose restart

# Remove everything (including data)
docker compose down -v
```

### CLI (inside container)

```bash
# Show identity
docker exec scmessenger scm identity

# Add contact
docker exec scmessenger scm contact add <peer_id> <public_key> --name "Alice"

# List contacts
docker exec scmessenger scm contact list

# Check status
docker exec scmessenger scm status

# View history
docker exec scmessenger scm history

# Bootstrap nodes management
docker exec scmessenger scm config bootstrap list
docker exec scmessenger scm config bootstrap add <multiaddr>
```

## Next Steps

1. Open the web UI: `http://localhost:9000` (UI implementation in progress)
2. Add contacts via CLI
3. Send messages!

## Support

- Report issues: https://github.com/Treystu/SCMessenger/issues
- See main README.md for architecture details
