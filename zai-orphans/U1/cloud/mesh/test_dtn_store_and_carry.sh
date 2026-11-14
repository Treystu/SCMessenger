#!/usr/bin/env bash
# ==============================================================================
# SCMessenger DTN Store-and-Carry Integration Test
# ==============================================================================
#
# Tests the Delay-Tolerant Networking (DTN) store-and-forward capability
# by simulating a message traversing a chain of intermittently-connected
# nodes. This validates that the mesh layer correctly:
#
#   1. Stores messages when no route to destination exists
#   2. Forwards stored messages when a new peer becomes reachable
#   3. Delivers the message end-to-end despite no simultaneous path existing
#
# Scenario:
#   Node A sends a message to Node E. At no point are A and E simultaneously
#   connected. The message must hop through B → C → D to reach E.
#
#   Phase 1: A ↔ B connected (B,C,D,E isolated)
#   Phase 2: B ↔ C connected (A,D,E isolated)
#   Phase 3: C ↔ D connected (A,B,E isolated)
#   Phase 4: D ↔ E connected (A,B,C isolated)
#
# The test verifies that Node E eventually receives the original message
# from Node A, proving the store-and-carry mechanism works.
#
# Prerequisites:
#   - docker-compose.mesh-test.yml network must be running
#   - All 5 nodes must be started and healthy
#   - Requires NET_ADMIN capability for tc manipulation
#
# Usage:
#   # Start the mesh first:
#   docker compose -f docker-compose.mesh-test.yml up -d node1 node2 node3 node4 node5
#   # Then run this test:
#   bash test_dtn_store_and_carry.sh
#
# ==============================================================================
set -euo pipefail

# --- Configuration ---
NODE_A="10.0.1.1"   # node1 — Message sender
NODE_B="10.0.1.2"   # node2 — First relay
NODE_C="10.0.1.3"   # node3 — Second relay
NODE_D="10.0.1.4"   # node4 — Third relay
NODE_E="10.0.1.5"   # node5 — Message recipient
API_PORT="9001"

PHASE_WAIT=8         # Seconds to wait in each phase for message propagation
VERIFY_WAIT=5        # Extra seconds for final delivery verification

TEST_MESSAGE="dtn-store-carry-$(date +%s)"
PASS=true

# --- Helper Functions ---

log() {
    echo "[$(date '+%H:%M:%S')] $*"
}

# Isolate a node by setting 100% packet loss
isolate_node() {
    local container="$1"
    log "  Isolating ${container} (loss 100%)"
    docker exec "${container}" tc qdisc replace dev eth0 root netem loss 100%
}

# Connect a node with BLE-like characteristics
connect_node() {
    local container="$1"
    log "  Connecting ${container} (delay 50ms loss 5%)"
    docker exec "${container}" tc qdisc replace dev eth0 root netem delay 50ms 20ms loss 5%
}

# Check if a node received a specific message
check_message() {
    local ip="$1"
    local expected_body="$2"
    curl -sf "http://${ip}:${API_PORT}/messages" 2>/dev/null \
        | jq -r ".messages[]? | select(.body == \"${expected_body}\") | .body" 2>/dev/null \
        || echo ""
}

# ==============================================================================
# Test Execution
# ==============================================================================

echo "============================================================"
echo "  SCMessenger DTN Store-and-Carry Integration Test"
echo "============================================================"
echo ""
log "Test message: ${TEST_MESSAGE}"
echo ""

# --- Phase 0: Isolate all nodes ---
log "Phase 0: Isolating all nodes..."
for container in scm-node1 scm-node2 scm-node3 scm-node4 scm-node5; do
    isolate_node "${container}"
done
sleep 2
echo ""

# --- Phase 1: A ↔ B connected ---
log "Phase 1: Connecting A ↔ B (rest isolated)"
log "  A sends message to E (no direct path exists)"
connect_node "scm-node1"
connect_node "scm-node2"

# A sends the message destined for E
MSG_RESPONSE=$(curl -sf -X POST "http://${NODE_A}:${API_PORT}/send" \
    -H "Content-Type: application/json" \
    -d "{\"to\": \"node5\", \"body\": \"${TEST_MESSAGE}\"}" 2>/dev/null || echo '{}')
MSG_ID=$(echo "${MSG_RESPONSE}" | jq -r '.msg_id // empty' 2>/dev/null || echo "")

if [ -z "${MSG_ID}" ]; then
    log "  WARNING: Could not extract msg_id from send response"
    log "  Response was: ${MSG_RESPONSE}"
fi

log "  Waiting ${PHASE_WAIT}s for A→B propagation..."
sleep "${PHASE_WAIT}"

# Verify B received and stored the message
B_HAS_MSG=$(check_message "${NODE_B}" "${TEST_MESSAGE}")
if [ -n "${B_HAS_MSG}" ]; then
    log "  ✓ Node B has stored the message"
else
    log "  ⚠ Node B may not have the message yet (DTN will retry)"
fi
echo ""

# --- Phase 2: B ↔ C connected (A disconnected) ---
log "Phase 2: Disconnecting A, connecting B ↔ C"
isolate_node "scm-node1"
connect_node "scm-node2"
connect_node "scm-node3"

log "  Waiting ${PHASE_WAIT}s for B→C propagation..."
sleep "${PHASE_WAIT}"

C_HAS_MSG=$(check_message "${NODE_C}" "${TEST_MESSAGE}")
if [ -n "${C_HAS_MSG}" ]; then
    log "  ✓ Node C has the message (relayed from B)"
else
    log "  ⚠ Node C may not have the message yet"
fi

# Disconnect B for next phase
isolate_node "scm-node2"
echo ""

# --- Phase 3: C ↔ D connected (A, B disconnected) ---
log "Phase 3: Disconnecting B, connecting C ↔ D"
connect_node "scm-node3"
connect_node "scm-node4"

log "  Waiting ${PHASE_WAIT}s for C→D propagation..."
sleep "${PHASE_WAIT}"

D_HAS_MSG=$(check_message "${NODE_D}" "${TEST_MESSAGE}")
if [ -n "${D_HAS_MSG}" ]; then
    log "  ✓ Node D has the message (relayed from C)"
else
    log "  ⚠ Node D may not have the message yet"
fi

# Disconnect C for next phase
isolate_node "scm-node3"
echo ""

# --- Phase 4: D ↔ E connected (A, B, C disconnected) ---
log "Phase 4: Disconnecting C, connecting D ↔ E"
connect_node "scm-node4"
connect_node "scm-node5"

log "  Waiting ${PHASE_WAIT}s for D→E propagation..."
sleep "${PHASE_WAIT}"

# Extra wait for final delivery
log "  Final verification wait (${VERIFY_WAIT}s)..."
sleep "${VERIFY_WAIT}"
echo ""

# ==============================================================================
# Verification
# ==============================================================================

log "Verifying message delivery at Node E..."
E_HAS_MSG=$(check_message "${NODE_E}" "${TEST_MESSAGE}")

echo ""
echo "============================================================"
if [ "${E_HAS_MSG}" = "${TEST_MESSAGE}" ]; then
    echo "  RESULT: ✅ PASS"
    echo ""
    echo "  Node E received the message from Node A!"
    echo "  Message: '${TEST_MESSAGE}'"
    echo "  The DTN store-and-carry mechanism is working correctly."
    echo "  Path: A → B → C → D → E (4 hops, no simultaneous end-to-end path)"
else
    echo "  RESULT: ❌ FAIL"
    echo ""
    echo "  Node E did NOT receive the message from Node A."
    echo "  Expected: '${TEST_MESSAGE}'"
    echo "  Got:      '${E_HAS_MSG:-<nothing>}'"
    echo ""
    echo "  Possible causes:"
    echo "    - DTN store-and-forward not implemented or not enabled"
    echo "    - Message TTL expired before reaching E"
    echo "    - Phase timing too short for relay (increase PHASE_WAIT)"
    echo "    - Node health check failures (verify nodes are running)"
    PASS=false
fi
echo "============================================================"
echo ""

# --- Cleanup: restore all nodes to normal ---
log "Restoring all nodes to normal network profile..."
for container in scm-node1 scm-node2 scm-node3 scm-node4 scm-node5; do
    connect_node "${container}" 2>/dev/null || true
done

if [ "${PASS}" = "true" ]; then
    exit 0
else
    exit 1
fi
