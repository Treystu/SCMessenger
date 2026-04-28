#!/bin/bash
set -euo pipefail
# Test connection to GCP node
# Usage: ./scripts/test_gcp_node.sh [GCP_IP]
#
# NOTE: GCP VM uses ephemeral IP. Direct netcat may be blocked.
#       This script uses gcloud SSH to check container status instead.

GCP_IP="${1:-34.135.34.73}"
GCP_PORT="${2:-9001}"
GCP_ZONE="${GCP_ZONE:-us-central1-a}"
GCP_HOST="${GCP_HOST:-scmessenger-bootstrap}"

echo "Testing GCP node connectivity..."
echo "  Host: $GCP_HOST"
echo "  Zone: $GCP_ZONE"
echo "  IP: $GCP_IP:$GCP_PORT"
echo ""

# 1. Check VM status
echo "Checking VM status..."
VM_STATUS=$(gcloud compute instances describe "$GCP_HOST" --zone="$GCP_ZONE" --format="value(status)" 2>/dev/null || echo "UNKNOWN")
if [ "$VM_STATUS" != "RUNNING" ]; then
    echo "❌ VM is $VM_STATUS (expected RUNNING)"
    echo "   Start with: gcloud compute instances start $GCP_HOST --zone=$GCP_ZONE"
    exit 1
fi
echo "✅ VM is RUNNING"

# 2. Check container status via SSH
echo ""
echo "Checking container status..."
CONTAINER_STATUS=$(gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" --tunnel-through-iap --command="
    sudo docker ps --filter ancestor=us-central1-docker.pkg.dev/scmessenger-bootstrapnode/scmessenger-repo/scmessenger-cli:latest --format '{{.Status}}' | head -1
" 2>/dev/null | grep -v "^WARNING:" | grep -v "^bash:" || echo "")

if [ -z "$CONTAINER_STATUS" ]; then
    echo "❌ No SCMessenger container running"
    echo "   Start with: ./scripts/deploy_gcp_node.sh"
    exit 1
fi
echo "✅ Container status: $CONTAINER_STATUS"

# 3. Check port bindings
echo ""
echo "Checking port bindings..."
PORTS=$(gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" --tunnel-through-iap --command="
    sudo docker ps --filter ancestor=us-central1-docker.pkg.dev/scmessenger-bootstrapnode/scmessenger-repo/scmessenger-cli:latest --format '{{.Ports}}' | head -1
" 2>/dev/null | grep -v "^WARNING:" | grep -v "^bash:" || echo "")

if echo "$PORTS" | grep -q "9001"; then
    echo "✅ Port 9001 is exposed"
else
    echo "❌ Port 9001 is NOT exposed"
    echo "   Ports: $PORTS"
    exit 1
fi

# 4. Check container logs for startup
echo ""
echo "Checking container logs (last 5 lines)..."
gcloud compute ssh "$GCP_HOST" --zone="$GCP_ZONE" --tunnel-through-iap --command="
    sudo docker logs --tail 5 \$(sudo docker ps --filter ancestor=us-central1-docker.pkg.dev/scmessenger-bootstrapnode/scmessenger-repo/scmessenger-cli:latest -q | head -1) 2>&1
" 2>/dev/null | grep -v "^WARNING:" | grep -v "^bash:" || echo "  (no logs available)"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  GCP Node Status: ✅ HEALTHY"
echo "  Container is running with ports exposed"
echo "═══════════════════════════════════════════════════════════"
