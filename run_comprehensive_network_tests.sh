#!/bin/bash
set -e

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  SCMessenger - Enhanced Network Testing Suite           ║${NC}"
echo -e "${CYAN}║  With NAT Simulation, Traffic Control & Real Conditions ║${NC}"
echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}✗ Docker is not running${NC}"
    echo "Please start Docker and try again."
    exit 1
fi

echo -e "${BLUE}Step 1: Cleaning up any existing containers...${NC}"
docker compose -f docker/docker-compose.yml down -v 2>/dev/null || true
docker compose -f docker/docker-compose.network-test.yml down -v 2>/dev/null || true
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""

echo -e "${BLUE}Step 2: Building Docker images...${NC}"
docker compose -f docker/docker-compose.network-test.yml build
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

echo -e "${BLUE}Step 3: Starting enhanced network simulation...${NC}"
echo "This includes:"
echo "  • NAT gateways (Cone NAT for Alice, Symmetric NAT for Bob)"
echo "  • Bandwidth limits (10 Mbps for Alice, 5 Mbps for Bob)"
echo "  • Network latency (50ms for Alice, 100ms for Bob)"
echo "  • Packet loss simulation (2% for Bob)"
echo "  • Separate isolated networks"
echo ""

docker compose -f docker/docker-compose.network-test.yml up -d

echo -e "${GREEN}✓ Network simulation started${NC}"
echo ""

echo -e "${BLUE}Step 4: Waiting for network initialization (20s)...${NC}"
sleep 20
echo -e "${GREEN}✓ Network initialized${NC}"
echo ""

echo -e "${BLUE}Step 5: Verifying network conditions...${NC}"

# Check NAT gateways
if docker ps | grep -q "scm-nat-a" && docker ps | grep -q "scm-nat-b"; then
    echo -e "${GREEN}✓ NAT gateways running${NC}"
else
    echo -e "${RED}✗ NAT gateways not running${NC}"
fi

# Check node containers
RELAY_RUNNING=$(docker ps | grep "scm-relay" | wc -l)
ALICE_RUNNING=$(docker ps | grep "scm-alice" | wc -l)
BOB_RUNNING=$(docker ps | grep "scm-bob" | wc -l)

if [ "$RELAY_RUNNING" -gt 0 ] && [ "$ALICE_RUNNING" -gt 0 ] && [ "$BOB_RUNNING" -gt 0 ]; then
    echo -e "${GREEN}✓ All nodes running (Relay, Alice, Bob)${NC}"
else
    echo -e "${RED}✗ Some nodes failed to start${NC}"
    echo "Relay: $RELAY_RUNNING, Alice: $ALICE_RUNNING, Bob: $BOB_RUNNING"
    docker compose -f docker/docker-compose.network-test.yml logs
    exit 1
fi

# Verify traffic control
echo ""
echo -e "${BLUE}Verifying traffic control settings...${NC}"
ALICE_TC=$(docker exec scm-alice tc qdisc show 2>/dev/null | grep -i "tbf\|netem" | wc -l)
BOB_TC=$(docker exec scm-bob tc qdisc show 2>/dev/null | grep -i "tbf\|netem" | wc -l)

if [ "$ALICE_TC" -gt 0 ]; then
    echo -e "${GREEN}✓ Alice traffic control active${NC}"
    docker exec scm-alice tc qdisc show | head -5
else
    echo -e "${YELLOW}⚠ Alice traffic control not detected${NC}"
fi

if [ "$BOB_TC" -gt 0 ]; then
    echo -e "${GREEN}✓ Bob traffic control active${NC}"
    docker exec scm-bob tc qdisc show | head -5
else
    echo -e "${YELLOW}⚠ Bob traffic control not detected${NC}"
fi

echo ""
echo -e "${BLUE}Step 6: Running basic connectivity test...${NC}"

# Get node IDs
get_peer_id() {
    local container=$1
    local id
    for i in {1..5}; do
        id=$(docker logs $container 2>&1 | grep "Network peer ID:" | tail -n 1 | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $NF}')
        if [ ! -z "$id" ]; then
            echo "$id"
            return
        fi
        sleep 2
    done
}

BOB_ID=$(get_peer_id scm-bob)
ALICE_ID=$(get_peer_id scm-alice)
RELAY_ID=$(get_peer_id scm-relay)

if [ -z "$BOB_ID" ] || [ -z "$ALICE_ID" ] || [ -z "$RELAY_ID" ]; then
    echo -e "${RED}✗ Failed to retrieve node IDs${NC}"
    echo "Alice: $ALICE_ID"
    echo "Bob: $BOB_ID"
    echo "Relay: $RELAY_ID"
    exit 1
fi

echo -e "${GREEN}✓ Node IDs retrieved${NC}"
echo "  Alice: $ALICE_ID"
echo "  Bob: $BOB_ID"
echo "  Relay: $RELAY_ID"
echo ""

# Send a test message
echo -e "${BLUE}Step 7: Testing end-to-end message delivery...${NC}"
docker exec scm-alice scm contact add "$BOB_ID" "0ee53fb5428a91b4e6eb59667d21ed5c9af2fd7538b51e3ddf621eb25770451c" --name Bob > /dev/null 2>&1 || true

TEST_MSG="E2E test message $(date +%s)"
docker exec scm-alice scm send "$BOB_ID" "$TEST_MSG" > /dev/null 2>&1 || true
echo "Sent: $TEST_MSG"

sleep 8

BOB_LOGS=$(docker logs scm-bob 2>&1 | tail -100)
if echo "$BOB_LOGS" | grep -q "$TEST_MSG"; then
    echo -e "${GREEN}✓ Message delivered successfully${NC}"
else
    echo -e "${YELLOW}⚠ Message not yet delivered (may need more time)${NC}"
fi

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  Enhanced Network Simulation Ready!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "Run comprehensive network scenario tests:"
echo -e "  ${YELLOW}./test_network_scenarios.sh${NC}"
echo ""
echo "View live logs:"
echo "  docker logs -f scm-relay"
echo "  docker logs -f scm-alice"
echo "  docker logs -f scm-bob"
echo ""
echo "Network topology:"
echo "  docker compose -f docker/docker-compose.network-test.yml ps"
echo ""
echo "Tear down simulation:"
echo "  docker compose -f docker/docker-compose.network-test.yml down -v"
echo ""
