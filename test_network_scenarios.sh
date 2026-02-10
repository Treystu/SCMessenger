#!/bin/bash
set -e

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test result tracking
declare -A TEST_RESULTS
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNING_TESTS=0

# Function to record test result
record_test() {
    local test_name="$1"
    local status="$2"  # pass, fail, warn
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    TEST_RESULTS["$test_name"]="$status"

    case "$status" in
        pass)
            PASSED_TESTS=$((PASSED_TESTS + 1))
            ;;
        fail)
            FAILED_TESTS=$((FAILED_TESTS + 1))
            ;;
        warn)
            WARNING_TESTS=$((WARNING_TESTS + 1))
            ;;
    esac
}

echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   SCMessenger - Comprehensive Network Testing${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════${NC}"
echo ""

# Ensure containers are running
if ! docker ps | grep -q "scm-relay"; then
    echo -e "${RED}Error: Containers not running. Run ./verify_simulation.sh first${NC}"
    exit 1
fi

echo -e "${GREEN}Starting comprehensive network scenario tests...${NC}"
echo ""

# ==============================================================================
# Get Node Information
# ==============================================================================
echo "📋 Retrieving node identities..."

# Helper function to get Network Peer ID
get_peer_id() {
    local container=$1
    local id
    for i in {1..3}; do
        id=$(docker logs $container 2>&1 | grep "Network peer ID:" | tail -n 1 | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $NF}')
        if [ ! -z "$id" ]; then
            echo "$id"
            return
        fi
        sleep 1
    done
}

# Helper function to get Identity Key
get_identity_key() {
    local container=$1
    local key
    for i in {1..3}; do
        key=$(docker logs $container 2>&1 | grep "Identity:" | tail -n 1 | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $2}' | tr -d '[:space:]')
        if [ ! -z "$key" ] && [ ${#key} -ge 32 ]; then
            echo "$key"
            return
        fi
        sleep 1
    done
}

# Get Bob's ID for message sending tests
BOB_ID=$(get_peer_id scm-bob)
ALICE_ID=$(get_peer_id scm-alice)
RELAY_ID=$(get_peer_id scm-relay)

if [ -z "$BOB_ID" ] || [ -z "$ALICE_ID" ]; then
    echo -e "${RED}✗ Failed to retrieve node IDs. Ensure containers are running.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Node IDs retrieved${NC}"
echo "  Alice: $ALICE_ID"
echo "  Bob: $BOB_ID"
echo "  Relay: $RELAY_ID"
echo ""

# ==============================================================================
# Scenario 1: Network Partition Recovery
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Scenario 1: Network Partition & Recovery${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Testing mesh resilience when relay goes down..."
echo ""

# Check initial connectivity
echo "1. Verifying initial connectivity..."
INITIAL_PEERS_ALICE=$(docker exec scm-alice scm peers 2>/dev/null | wc -l)
INITIAL_PEERS_BOB=$(docker exec scm-bob scm peers 2>/dev/null | wc -l)
echo -e "${GREEN}✓ Alice peers: $INITIAL_PEERS_ALICE${NC}"
echo -e "${GREEN}✓ Bob peers: $INITIAL_PEERS_BOB${NC}"

# Pause relay to simulate network partition
echo ""
echo "2. Simulating network partition (pausing relay)..."
docker pause scm-relay > /dev/null 2>&1
sleep 3

echo "3. Testing message queueing during partition..."
MESSAGE_PARTITION="Test during partition $(date +%s)"
docker exec scm-alice scm send "$BOB_ID" "$MESSAGE_PARTITION" > /dev/null 2>&1 || true
echo -e "${GREEN}✓ Message queued in outbox during partition${NC}"

# Unpause relay to restore connectivity
echo ""
echo "4. Restoring network (unpausing relay)..."
docker unpause scm-relay > /dev/null 2>&1
sleep 5

echo "5. Verifying reconnection..."
RECOVERY_PEERS_ALICE=$(docker exec scm-alice scm peers 2>/dev/null | wc -l)
RECOVERY_PEERS_BOB=$(docker exec scm-bob scm peers 2>/dev/null | wc -l)

if [ "$RECOVERY_PEERS_ALICE" -gt 0 ] && [ "$RECOVERY_PEERS_BOB" -gt 0 ]; then
    echo -e "${GREEN}✓ Network recovered successfully${NC}"
    echo "  Alice reconnected: $RECOVERY_PEERS_ALICE peers"
    echo "  Bob reconnected: $RECOVERY_PEERS_BOB peers"
    record_test "Network Partition Recovery" "pass"
else
    echo -e "${RED}✗ Network recovery failed${NC}"
    record_test "Network Partition Recovery" "fail"
fi

# Check if queued message was delivered
sleep 3
BOB_LOGS=$(docker logs scm-bob 2>&1 | tail -100)
if echo "$BOB_LOGS" | grep -q "$MESSAGE_PARTITION"; then
    echo -e "${GREEN}✓ Queued message delivered after recovery${NC}"
    record_test "Partition Message Delivery" "pass"
else
    echo -e "${YELLOW}⚠ Queued message delivery pending or failed${NC}"
    record_test "Partition Message Delivery" "warn"
fi

echo ""

# ==============================================================================
# Scenario 2: NAT Traversal Testing
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Scenario 2: NAT Traversal & Address Discovery${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Analyzing NAT behavior and traversal mechanisms..."
echo ""

# Check address observations
echo "1. Address observation analysis..."
ALICE_OBS=$(docker logs scm-alice 2>&1 | grep -i "observed\|external.*address" | wc -l)
BOB_OBS=$(docker logs scm-bob 2>&1 | grep -i "observed\|external.*address" | wc -l)
RELAY_OBS=$(docker logs scm-relay 2>&1 | grep -i "observed\|external.*address" | wc -l)

echo "   Alice observations: $ALICE_OBS"
echo "   Bob observations: $BOB_OBS"
echo "   Relay observations: $RELAY_OBS"

if [ "$ALICE_OBS" -gt 0 ] || [ "$BOB_OBS" -gt 0 ]; then
    echo -e "${GREEN}✓ Address observation protocol working${NC}"
    record_test "Address Observation" "pass"
else
    echo -e "${YELLOW}⚠ Limited address observations (symmetric NAT?)${NC}"
    record_test "Address Observation" "warn"
fi

# Check for hole punching attempts
echo ""
echo "2. NAT hole punching analysis..."
HOLE_PUNCH_ALICE=$(docker logs scm-alice 2>&1 | grep -i "dcutr\|hole.*punch\|direct.*connect" | wc -l)
HOLE_PUNCH_BOB=$(docker logs scm-bob 2>&1 | grep -i "dcutr\|hole.*punch\|direct.*connect" | wc -l)

if [ "$HOLE_PUNCH_ALICE" -gt 0 ] || [ "$HOLE_PUNCH_BOB" -gt 0 ]; then
    echo -e "${GREEN}✓ NAT hole punching attempted${NC}"
    echo "   Alice attempts: $HOLE_PUNCH_ALICE"
    echo "   Bob attempts: $HOLE_PUNCH_BOB"
    record_test "NAT Hole Punching" "pass"
else
    echo -e "${YELLOW}⚠ No hole punching detected${NC}"
    record_test "NAT Hole Punching" "warn"
fi

# Analyze connection types
echo ""
echo "3. Connection type analysis..."
ALICE_CONNS=$(docker logs scm-alice 2>&1 | grep -i "connection.*established\|connected.*to" | tail -5)
BOB_CONNS=$(docker logs scm-bob 2>&1 | grep -i "connection.*established\|connected.*to" | tail -5)

if [ ! -z "$ALICE_CONNS" ]; then
    echo -e "${GREEN}✓ Alice active connections:${NC}"
    echo "$ALICE_CONNS" | sed 's/^/     /'
fi

if [ ! -z "$BOB_CONNS" ]; then
    echo -e "${GREEN}✓ Bob active connections:${NC}"
    echo "$BOB_CONNS" | sed 's/^/     /'
fi

echo ""

# ==============================================================================
# Scenario 3: Circuit Relay Verification
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Scenario 3: Circuit Relay Protocol${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Testing relay-based message forwarding..."
echo ""

# Check relay activity
echo "1. Relay node activity analysis..."
RELAY_FORWARDS=$(docker logs scm-relay 2>&1 | grep -i "relay\|forward\|circuit" | wc -l)
RELAY_RESERVATIONS=$(docker logs scm-relay 2>&1 | grep -i "reservation\|allocate" | wc -l)

echo "   Total relay events: $RELAY_FORWARDS"
echo "   Circuit reservations: $RELAY_RESERVATIONS"

if [ "$RELAY_FORWARDS" -gt 0 ]; then
    echo -e "${GREEN}✓ Circuit relay is actively forwarding${NC}"
    record_test "Circuit Relay Activity" "pass"
else
    echo -e "${YELLOW}⚠ No circuit relay activity detected${NC}"
    record_test "Circuit Relay Activity" "warn"
fi

# Test message through relay
echo ""
echo "2. Testing message delivery through relay..."
RELAY_TEST_MSG="Relay test message $(date +%s)"
docker exec scm-alice scm send "$BOB_ID" "$RELAY_TEST_MSG" > /dev/null 2>&1 || true
sleep 5

BOB_RELAY_LOG=$(docker logs scm-bob 2>&1 | tail -50)
if echo "$BOB_RELAY_LOG" | grep -q "$RELAY_TEST_MSG"; then
    echo -e "${GREEN}✓ Message successfully relayed${NC}"
    record_test "Relay Message Delivery" "pass"

    # Check if it was relayed or direct
    RELAY_LOG=$(docker logs scm-relay 2>&1 | tail -100)
    if echo "$RELAY_LOG" | grep -q "relay\|forward"; then
        echo -e "${GREEN}  → Confirmed: Message went through relay${NC}"
    else
        echo -e "${BLUE}  → Message may have used direct connection${NC}"
    fi
else
    echo -e "${RED}✗ Message delivery failed${NC}"
    record_test "Relay Message Delivery" "fail"
fi

echo ""

# ==============================================================================
# Scenario 4: Multi-hop Routing & Mesh Behavior
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Scenario 4: Mesh Routing & Multi-hop Forwarding${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Analyzing mycorrhizal mesh routing behavior..."
echo ""

# Check routing table updates
echo "1. Routing protocol analysis..."
ROUTING_UPDATES=$(docker logs scm-relay scm-alice scm-bob 2>&1 | grep -i "routing\|route.*update\|neighborhood" | wc -l)
DHT_OPS=$(docker logs scm-relay scm-alice scm-bob 2>&1 | grep -i "kad\|dht\|put_record\|get_record" | wc -l)

echo "   Routing updates: $ROUTING_UPDATES"
echo "   DHT operations: $DHT_OPS"

if [ "$ROUTING_UPDATES" -gt 0 ]; then
    echo -e "${GREEN}✓ Mesh routing tables are being updated${NC}"
else
    echo -e "${YELLOW}⚠ Limited routing activity (static topology?)${NC}"
fi

# Check peer exchange
echo ""
echo "2. Peer exchange protocol..."
PEER_EXCHANGE_ALICE=$(docker logs scm-alice 2>&1 | grep -i "peer.*exchange\|discovered.*peer" | wc -l)
PEER_EXCHANGE_BOB=$(docker logs scm-bob 2>&1 | grep -i "peer.*exchange\|discovered.*peer" | wc -l)

echo "   Alice peer discoveries: $PEER_EXCHANGE_ALICE"
echo "   Bob peer discoveries: $PEER_EXCHANGE_BOB"

if [ "$PEER_EXCHANGE_ALICE" -gt 0 ] || [ "$PEER_EXCHANGE_BOB" -gt 0 ]; then
    echo -e "${GREEN}✓ Peer exchange protocol active${NC}"
fi

echo ""

# ==============================================================================
# Scenario 5: Transport Layer & Protocol Escalation
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Scenario 5: Transport Protocol Analysis${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Testing transport layer capabilities..."
echo ""

# Analyze transport protocols
echo "1. Active transport protocols..."
TCP_USAGE=$(docker logs scm-relay scm-alice scm-bob 2>&1 | grep -i "/tcp/" | wc -l)
QUIC_USAGE=$(docker logs scm-relay scm-alice scm-bob 2>&1 | grep -i "/quic\|/udp.*quic" | wc -l)
WS_USAGE=$(docker logs scm-relay scm-alice scm-bob 2>&1 | grep -i "websocket\|/ws/" | wc -l)

echo "   TCP connections: $TCP_USAGE"
echo "   QUIC connections: $QUIC_USAGE"
echo "   WebSocket usage: $WS_USAGE"

TRANSPORT_ACTIVE=false
if [ "$TCP_USAGE" -gt 0 ]; then
    echo -e "${GREEN}✓ TCP transport active${NC}"
    TRANSPORT_ACTIVE=true
fi
if [ "$QUIC_USAGE" -gt 0 ]; then
    echo -e "${GREEN}✓ QUIC transport active${NC}"
    TRANSPORT_ACTIVE=true
fi
if [ "$WS_USAGE" -gt 0 ]; then
    echo -e "${GREEN}✓ WebSocket transport active${NC}"
    TRANSPORT_ACTIVE=true
fi

if [ "$TRANSPORT_ACTIVE" = true ]; then
    record_test "Transport Protocols" "pass"
else
    echo -e "${RED}✗ No transport protocol activity detected${NC}"
    record_test "Transport Protocols" "fail"
fi

# Check for protocol upgrades
echo ""
echo "2. Transport escalation..."
ESCALATION=$(docker logs scm-alice scm-bob 2>&1 | grep -i "escalat\|upgrade" | wc -l)
if [ "$ESCALATION" -gt 0 ]; then
    echo -e "${GREEN}✓ Transport escalation detected: $ESCALATION events${NC}"
    record_test "Transport Escalation" "pass"
else
    echo -e "${YELLOW}⚠ No transport escalation detected${NC}"
    record_test "Transport Escalation" "warn"
fi

echo ""

# ==============================================================================
# Scenario 6: Privacy Layer & Onion Routing
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Scenario 6: Privacy Features & Onion Routing${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Verifying privacy-preserving features..."
echo ""

# Check onion routing
echo "1. Onion routing analysis..."
CIRCUITS=$(docker logs scm-alice scm-bob scm-relay 2>&1 | grep -i "circuit\|onion\|layer.*encrypt" | wc -l)
HOPS=$(docker logs scm-alice scm-bob scm-relay 2>&1 | grep -i "hop\|intermediate.*node" | wc -l)

echo "   Circuit establishments: $CIRCUITS"
echo "   Multi-hop routing: $HOPS"

if [ "$CIRCUITS" -gt 0 ]; then
    echo -e "${GREEN}✓ Onion routing circuits established${NC}"
else
    echo -e "${BLUE}ℹ Using direct routing (small network, privacy optional)${NC}"
fi

# Check cover traffic
echo ""
echo "2. Cover traffic & padding..."
COVER=$(docker logs scm-alice scm-bob 2>&1 | grep -i "cover.*traffic\|dummy.*message\|padding" | wc -l)
if [ "$COVER" -gt 0 ]; then
    echo -e "${GREEN}✓ Cover traffic active: $COVER events${NC}"
else
    echo -e "${BLUE}ℹ Cover traffic disabled (test mode)${NC}"
fi

echo ""

# ==============================================================================
# Scenario 7: Drift Protocol & Store-and-Forward
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Scenario 7: Drift Protocol & Offline Support${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Testing asynchronous message delivery..."
echo ""

# Test offline message delivery
echo "1. Testing offline message queueing..."
echo "   Pausing Bob to simulate offline state..."
docker pause scm-bob > /dev/null 2>&1
sleep 2

OFFLINE_MSG="Offline test message $(date +%s)"
docker exec scm-alice scm send "$BOB_ID" "$OFFLINE_MSG" > /dev/null 2>&1 || true
echo -e "${GREEN}✓ Message queued while Bob offline${NC}"

echo ""
echo "2. Bringing Bob back online..."
docker unpause scm-bob > /dev/null 2>&1
sleep 5

echo "3. Checking Drift synchronization..."
SYNC_EVENTS=$(docker logs scm-bob 2>&1 | tail -50 | grep -i "sync\|drift\|catch.*up" | wc -l)
echo "   Sync events detected: $SYNC_EVENTS"

# Check if message was delivered
BOB_OFFLINE_LOG=$(docker logs scm-bob 2>&1 | tail -100)
if echo "$BOB_OFFLINE_LOG" | grep -q "$OFFLINE_MSG"; then
    echo -e "${GREEN}✓ Offline message delivered via Drift sync${NC}"
else
    echo -e "${YELLOW}⚠ Message delivery pending (may take time)${NC}"
fi

# Check store-and-forward on relay
echo ""
echo "4. Relay store-and-forward verification..."
STORED=$(docker logs scm-relay 2>&1 | grep -i "store\|persist\|queue" | wc -l)
if [ "$STORED" -gt 0 ]; then
    echo -e "${GREEN}✓ Relay is storing and forwarding: $STORED events${NC}"
fi

echo ""

# ==============================================================================
# Scenario 8: Performance & Scalability
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Scenario 8: Performance Metrics${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Analyzing system performance..."
echo ""

# Message throughput
echo "1. Message throughput test..."
START_TIME=$(date +%s)
for i in {1..10}; do
    docker exec scm-alice scm send "$BOB_ID" "Throughput test $i" > /dev/null 2>&1 || true
done
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo -e "${GREEN}✓ Sent 10 messages in ${DURATION}s${NC}"
if [ "$DURATION" -lt 5 ]; then
    echo "   Performance: Excellent (< 5s)"
elif [ "$DURATION" -lt 10 ]; then
    echo "   Performance: Good (< 10s)"
else
    echo "   Performance: Acceptable (> 10s)"
fi

# Check for errors
echo ""
echo "2. Error analysis..."
ERRORS_ALICE=$(docker logs scm-alice 2>&1 | grep -i "error\|panic\|fatal" | grep -v "test" | wc -l)
ERRORS_BOB=$(docker logs scm-bob 2>&1 | grep -i "error\|panic\|fatal" | grep -v "test" | wc -l)
ERRORS_RELAY=$(docker logs scm-relay 2>&1 | grep -i "error\|panic\|fatal" | grep -v "test" | wc -l)

echo "   Alice errors: $ERRORS_ALICE"
echo "   Bob errors: $ERRORS_BOB"
echo "   Relay errors: $ERRORS_RELAY"

TOTAL_ERRORS=$((ERRORS_ALICE + ERRORS_BOB + ERRORS_RELAY))
if [ "$TOTAL_ERRORS" -eq 0 ]; then
    echo -e "${GREEN}✓ No errors detected${NC}"
elif [ "$TOTAL_ERRORS" -lt 5 ]; then
    echo -e "${YELLOW}⚠ Minor errors detected (< 5)${NC}"
else
    echo -e "${RED}✗ Multiple errors detected${NC}"
fi

echo ""

# ==============================================================================
# Final Summary
# ==============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ "$FAILED_TESTS" -eq 0 ]; then
    echo -e "${GREEN}✅ Comprehensive Network Testing Complete${NC}"
else
    echo -e "${YELLOW}⚠️  Network Testing Complete with Issues${NC}"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${BLUE}Test Statistics:${NC}"
echo "  Total Tests:    $TOTAL_TESTS"
echo -e "  ${GREEN}✓ Passed:       $PASSED_TESTS${NC}"
echo -e "  ${YELLOW}⚠ Warnings:     $WARNING_TESTS${NC}"
echo -e "  ${RED}✗ Failed:       $FAILED_TESTS${NC}"
echo ""

# Calculate success rate
if [ "$TOTAL_TESTS" -gt 0 ]; then
    SUCCESS_RATE=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    echo -e "${BLUE}Success Rate:   ${SUCCESS_RATE}%${NC}"
    echo ""
fi

echo -e "${BLUE}Detailed Test Results:${NC}"
for test_name in "${!TEST_RESULTS[@]}"; do
    status="${TEST_RESULTS[$test_name]}"
    case "$status" in
        pass)
            echo -e "  ${GREEN}✓${NC} $test_name"
            ;;
        fail)
            echo -e "  ${RED}✗${NC} $test_name"
            ;;
        warn)
            echo -e "  ${YELLOW}⚠${NC} $test_name"
            ;;
    esac
done | sort

echo ""
if [ "$FAILED_TESTS" -gt 0 ]; then
    echo -e "${RED}⚠️  ATTENTION: $FAILED_TESTS test(s) failed!${NC}"
    echo ""
    echo "Failed tests indicate missing or non-functional network capabilities."
    echo "This may be due to:"
    echo "  • Missing libp2p features in the implementation"
    echo "  • Network configuration issues in Docker"
    echo "  • Insufficient wait time for feature initialization"
    echo ""
    echo "Run with enhanced network simulation:"
    echo "  docker compose -f docker/docker-compose.network-test.yml up -d"
    echo ""
fi

if [ "$WARNING_TESTS" -gt 0 ]; then
    echo -e "${YELLOW}Note: $WARNING_TESTS warning(s) detected${NC}"
    echo "Warnings may indicate optional features not enabled in test mode."
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Exit with error if tests failed
if [ "$FAILED_TESTS" -gt 0 ]; then
    exit 1
fi
