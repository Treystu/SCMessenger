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
PEER_ID=$(docker logs "${CONTAINER_NAME}" 2>&1 \
    | sed -E $'s/\x1B\\[[0-9;]*[a-zA-Z]//g' \
    | grep "Network peer ID:" \
    | tail -1 \
    | awk '{print $NF}')
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

# Get Public IP (try multiple methods with timeouts)
PUBLIC_IP=""
if command -v curl &> /dev/null; then
    PUBLIC_IP=$(curl -s --connect-timeout 5 --max-time 10 ifconfig.me/ip 2>/dev/null || curl -s --connect-timeout 5 --max-time 10 icanhazip.com 2>/dev/null)
fi

if [ -z "$PUBLIC_IP" ]; then
    echo "🌐 Public IP: (could not detect - check manually)"
else
    echo "🌐 Public IP:"
    echo "   ${PUBLIC_IP}"
fi
echo ""

# Get Listen Port from config (defaults to 9000)
CONFIG_FILE="/root/.config/scmessenger/config.json"
LISTEN_PORT=$(docker exec "${CONTAINER_NAME}" sh -c "if [ -f ${CONFIG_FILE} ]; then cat ${CONFIG_FILE} | jq -r '.listen_port // 9000'; else echo 9000; fi" 2>/dev/null || echo 9000)
P2P_PORT=$((LISTEN_PORT + 1))

# Construct multiaddress
if [ ! -z "$PUBLIC_IP" ]; then
    MULTIADDR="/ip4/${PUBLIC_IP}/tcp/${P2P_PORT}/p2p/${PEER_ID}"
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
    echo "  -p ${LISTEN_PORT}:${LISTEN_PORT} -p ${P2P_PORT}:${P2P_PORT} \\"
    echo "  -v ~/scm_data:/root/.local/share/scmessenger \\"
    echo "  -e BOOTSTRAP_NODES=\"${MULTIADDR}\" \\"
    echo "  testbotz/scmessenger:latest"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi

echo ""
echo "✅ Node is running and ready for connections!"
echo ""
