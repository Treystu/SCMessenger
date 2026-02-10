#!/bin/bash
set -e

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "🚀 Building Docker images..."
docker compose -f docker/docker-compose.yml build

echo "---------------------------------------------------"
echo "🌐 Starting Network Simulation (Relay + Alice + Bob)"
echo "---------------------------------------------------"
docker compose -f docker/docker-compose.yml up -d

echo "⏳ Waiting for nodes to initialize and discover each other (15s)..."
sleep 15

# Get Peer IDs
echo "📋 Retrieving Peer IDs..."
ALICE_STATUS=$(docker exec scm-alice scm status)
BOB_STATUS=$(docker exec scm-bob scm status)

# Extract Peer IDs (simulated parsing, in reality we might need a better way to get ID specifically)
# For now, let's use 'scm identity show' which outputs the ID clearly
ALICE_ID=$(docker exec scm-alice scm identity show | grep "ID:" | awk '{print $2}')
BOB_ID=$(docker exec scm-bob scm identity show | grep "ID:" | awk '{print $2}')

echo "👤 Alice ID: $ALICE_ID"
echo "👤 Bob ID:   $BOB_ID"

if [ -z "$ALICE_ID" ] || [ -z "$BOB_ID" ]; then
    echo "${RED}✗ Failed to retrieve Peer IDs${NC}"
    docker compose -f docker/docker-compose.yml logs
    exit 1
fi

echo "---------------------------------------------------"
echo "📨 Test 1: Alice -> Bob (Message Send)"
echo "---------------------------------------------------"

# Add Bob as contact for Alice (optional but good for testing contact logic)
docker exec scm-alice scm contact add "$BOB_ID" "test-key-placeholder" --name Bob

# Send message
MESSAGE="Hello from Alice $(date +%s)"
echo "Sending: '$MESSAGE'"
docker exec scm-alice scm send "$BOB_ID" "$MESSAGE"

echo "⏳ Waiting for message delivery (5s)..."
sleep 5

# Check Bob's history
echo "---------------------------------------------------"
echo "📥 Test 1: Verifying Receipt on Bob"
echo "---------------------------------------------------"
BOB_HISTORY=$(docker exec scm-bob scm history --limit 5)

if echo "$BOB_HISTORY" | grep -q "$MESSAGE"; then
    echo "${GREEN}✓ Message received successfully!${NC}"
else
    echo "${RED}✗ Message not found in Bob's history${NC}"
    echo "Bob's History:"
    echo "$BOB_HISTORY"
    docker-compose -f docker/docker-compose.yml logs
    exit 1
fi

echo "---------------------------------------------------"
echo "✅ Simulation Verified Successfully"
echo "---------------------------------------------------"
echo "To explore manually:"
echo "  docker exec -it scm-alice scm status"
echo "  docker exec -it scm-bob scm status"
echo ""
echo "To tear down:"
echo "  docker-compose -f docker/docker-compose.yml down"
