#!/bin/bash
# Test connection to GCP node
# Usage: ./scripts/test_gcp_node.sh [GCP_IP]

GCP_IP="${1:-34.168.102.7}"
GCP_PORT="${2:-9001}"

echo "Testing connection to GCP node at $GCP_IP:$GCP_PORT..."

# 1. Basic connectivity check
if ! nc -z -w 5 "$GCP_IP" "$GCP_PORT"; then
    echo "❌ Port $GCP_PORT on $GCP_IP is not reachable."
    exit 1
fi
echo "✅ TCP Port $GCP_PORT is open."

# 2. Run local SCM node to peer
echo "Starting local temporary node to peer with $GCP_IP..."
echo "Press Ctrl+C to stop once you see 'Peer Discovered' or 'Dial initiated'..."
echo ""

docker run -it --rm --network host \
    -e BOOTSTRAP_NODES="/ip4/$GCP_IP/tcp/$GCP_PORT" \
    testbotz/scmessenger:latest \
    scm start
