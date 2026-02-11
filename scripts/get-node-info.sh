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
PEER_ID=$(docker logs ${CONTAINER_NAME} 2>&1 | grep "Network peer ID" | tail -1 | awk '{print $NF}')
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
if command -v curl &> /dev/null; then
    PUBLIC_IP=$(curl -s ifconfig.me 2>/dev/null || curl -s icanhazip.com 2>/dev/null)
fi

if [ -z "$PUBLIC_IP" ]; then
    echo "🌐 Public IP: (could not detect - check manually)"
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
