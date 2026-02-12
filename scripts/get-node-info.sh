#!/bin/bash
# Get SCMessenger Node Information
# Usage: ./scripts/get-node-info.sh [container-name]

CONTAINER_NAME="${1:-scmessenger}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  SCMessenger Node Information"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

# Check if container exists
if ! docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "❌ Container '${CONTAINER_NAME}' not found."
    echo "Available containers:"
    docker ps -a --format '  - {{.Names}}'
    exit 1
fi

# Check if container is running
if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "⚠️  Container '${CONTAINER_NAME}' is not running."
    echo "Start it with: docker start ${CONTAINER_NAME}"
    exit 1
fi

echo "📦 Container: ${CONTAINER_NAME}"
echo ""

# Get Peer ID
PEER_ID=$(docker logs ${CONTAINER_NAME} 2>&1 | grep "Peer ID:" | tail -1 | awk '{print $NF}')
if [ -z "$PEER_ID" ]; then
    echo "⚠️  Could not find Peer ID in logs. Is the node fully started?"
    exit 1
fi

echo "🆔 Peer ID:"
echo "   ${PEER_ID}"
echo ""

# Get Identity
IDENTITY=$(docker logs ${CONTAINER_NAME} 2>&1 | grep "^Identity:" | tail -1 | awk '{print $2}')
if [ ! -z "$IDENTITY" ]; then
    echo "🔑 Identity:"
    echo "   ${IDENTITY}"
    echo ""
fi

# Get Public IP (try multiple methods)
PUBLIC_IP=""

# First try: Query the node's API for its discovered external address
if command -v curl &> /dev/null; then
    # Check if we're trying to reach a containerized node
    # If docker is available and scmessenger container is running, use docker exec
    if command -v docker &> /dev/null && docker ps --format '{{.Names}}' 2>/dev/null | grep -q '^scmessenger'; then
        API_RESPONSE=$(docker exec scmessenger curl -s http://127.0.0.1:9876/api/external-address 2>/dev/null)
    else
        API_RESPONSE=$(curl -s http://localhost:9876/api/external-address 2>/dev/null)
    fi
    
    if [ $? -eq 0 ] && [ ! -z "$API_RESPONSE" ]; then
        # Parse JSON response to get first address
        PUBLIC_IP=$(echo "$API_RESPONSE" | grep -o '"[0-9]\+\.[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1 | tr -d '"')
    fi
fi

# Fallback: If API unavailable or returns no addresses, try local detection
# Note: This detects the host machine's IP, not necessarily the reachable external IP
if [ -z "$PUBLIC_IP" ]; then
    # Try to get default route IP (works on Linux/macOS)
    if command -v ip &> /dev/null; then
        # Linux
        PUBLIC_IP=$(ip route get 1.1.1.1 2>/dev/null | grep -oP 'src \K\S+')
    elif command -v route &> /dev/null; then
        # macOS
        DEFAULT_IFACE=$(route -n get default 2>/dev/null | grep 'interface:' | awk '{print $2}')
        if [ ! -z "$DEFAULT_IFACE" ]; then
            PUBLIC_IP=$(ifconfig "$DEFAULT_IFACE" 2>/dev/null | grep 'inet ' | awk '{print $2}')
        fi
    fi
fi

if [ -z "$PUBLIC_IP" ]; then
    echo "🌐 Public IP: (could not detect)"
    echo ""
    echo "   To share this node's connection info, you need its public IP address."
    echo "   Options:"
    echo "   1. If behind NAT, configure port forwarding for port 9001"
    echo "   2. Use a cloud VM with a public IP"
    echo "   3. Use the node's local IP for LAN-only connections"
else
    echo "🌐 Public IP:"
    echo "   ${PUBLIC_IP}"
fi
echo ""

# Construct multiaddress
if [ ! -z "$PUBLIC_IP" ]; then
    MULTIADDR="/ip4/${PUBLIC_IP}/tcp/9001/p2p/${PEER_ID}"
    echo "📡 Connection String (Share this with others):"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "${MULTIADDR}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Generate bootstrap command for others
    echo "📋 Bootstrap Command (Others can use this):"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "docker run -d \\"
    echo "  --name scmessenger-local \\"
    echo "  -p 9000:9000 -p 9001:9001 \\"
    echo "  -v ~/scm_data:/root/.local/share/scmessenger \\"
    echo "  -e BOOTSTRAP_NODES=\"${MULTIADDR}\" \\"
    echo "  testbotz/scmessenger:latest"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi

echo ""
echo "✅ Node is running and ready for connections!"
echo ""
