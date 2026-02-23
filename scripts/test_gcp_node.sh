#!/bin/bash
# Test connection to GCP node
# Usage: ./scripts/test_gcp_node.sh [GCP_IP]

GCP_IP="${1:-34.135.34.73}"
GCP_PORT="${2:-9001}"

echo "Testing connection to GCP node at $GCP_IP:$GCP_PORT..."

# 1. Basic connectivity check
if ! nc -z -w 5 "$GCP_IP" "$GCP_PORT"; then
    echo "❌ Port $GCP_PORT on $GCP_IP is not reachable."
    exit 1
fi
echo "✅ TCP Port $GCP_PORT is open."

# 2. Check if local ports 9000 and 9001 are free
if nc -z localhost 9000 2>/dev/null || lsof -i :9000 -sTCP:LISTEN >/dev/null; then
    echo "❌ Local port 9000 is already in use. Please stop any running SCM nodes or other services on port 9000."
    exit 1
fi
if nc -z localhost 9001 2>/dev/null || lsof -i :9001 -sTCP:LISTEN >/dev/null; then
    echo "❌ Local port 9001 is already in use. Please stop any running SCM nodes or other services on port 9001."
    exit 1
fi


# 2. Run local SCM node to peer
echo "Starting local temporary node to peer with $GCP_IP..."
echo "Press Ctrl+C to stop once you see 'Peer Discovered' or 'Dial initiated'..."
echo ""

docker run -it --rm --network host \
    -e BOOTSTRAP_NODES="/ip4/$GCP_IP/tcp/$GCP_PORT" \
    testbotz/scmessenger:latest \
    scm start
