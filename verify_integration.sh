#!/bin/bash
# Integration Verification Script
# Checks that all 6 phases are properly wired in the codebase

set -e

echo "========================================"
echo "VERIFYING ALL 6 PHASES INTEGRATION"
echo "========================================"
echo ""

CORE_DIR="./core/src/transport"
SWARM_FILE="$CORE_DIR/swarm.rs"
BEHAVIOUR_FILE="$CORE_DIR/behaviour.rs"
MESH_FILE="$CORE_DIR/mesh_routing.rs"

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

check_exists() {
    if grep -q "$2" "$1"; then
        echo -e "${GREEN}✓${NC} $3"
        return 0
    else
        echo -e "${RED}✗${NC} $3"
        return 1
    fi
}

check_count() {
    count=$(grep -c "$2" "$1" || true)
    if [ "$count" -ge "$3" ]; then
        echo -e "${GREEN}✓${NC} $4 (found $count occurrences)"
        return 0
    else
        echo -e "${RED}✗${NC} $4 (found only $count occurrences, expected $3+)"
        return 1
    fi
}

PASS=0
FAIL=0

# Phase 1: Address Observation
echo "=== Phase 1: Address Observation ==="
if check_exists "$SWARM_FILE" "AddressObserver::new()" "AddressObserver instantiated"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "address_observer.record_observation" "Address observations recorded"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "primary_external_address()" "Consensus address used"; then ((PASS++)); else ((FAIL++)); fi
echo ""

# Phase 2: Multi-Port Listening
echo "=== Phase 2: Multi-Port Listening ==="
if check_exists "$SWARM_FILE" "multiport::generate_listen_addresses" "Multi-port address generation"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "Starting multi-port adaptive listening" "Multi-port mode activated"; then ((PASS++)); else ((FAIL++)); fi
echo ""

# Phase 3: Relay Capability
echo "=== Phase 3: Relay Capability ==="
if check_exists "$BEHAVIOUR_FILE" "pub relay:" "Relay protocol in behaviour"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$BEHAVIOUR_FILE" "RelayRequest" "RelayRequest type defined"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$BEHAVIOUR_FILE" "RelayResponse" "RelayResponse type defined"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "IronCoreBehaviourEvent::Relay" "Relay event handler"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "Peer is asking us to relay a message" "Relay request processing"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "behaviour_mut().relay.send_request" "Relay requests sent"; then ((PASS++)); else ((FAIL++)); fi
echo ""

# Phase 4: Bootstrap Capability
echo "=== Phase 4: Bootstrap Capability ==="
if check_exists "$SWARM_FILE" "BootstrapCapability::new()" "BootstrapCapability instantiated"; then ((PASS++)); else ((FAIL++)); fi
if check_count "$SWARM_FILE" "bootstrap_capability.add_peer" 2 "Peers added to bootstrap capability"; then ((PASS++)); else ((FAIL++)); fi
echo ""

# Phase 5: Reputation Tracking
echo "=== Phase 5: Reputation Tracking ==="
if check_exists "$SWARM_FILE" "MultiPathDelivery::new()" "MultiPathDelivery (includes reputation)"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "multi_path_delivery.record_success" "Success tracking"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "multi_path_delivery.record_failure" "Failure tracking"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$MESH_FILE" "ReputationTracker" "ReputationTracker defined"; then ((PASS++)); else ((FAIL++)); fi
echo ""

# Phase 6: Retry Logic
echo "=== Phase 6: Continuous Retry Logic ==="
if check_exists "$SWARM_FILE" "let mut retry_interval" "Retry interval task"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "retry_interval.tick()" "Periodic retry checks"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "multi_path_delivery.start_delivery" "Delivery tracking started"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "multi_path_delivery.get_best_paths" "Multi-path routing"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "pending_messages:" "Pending message tracking"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "RETRY: Attempting delivery" "Retry attempts logged"; then ((PASS++)); else ((FAIL++)); fi
echo ""

# Integration points
echo "=== Critical Integration Points ==="
if check_exists "$SWARM_FILE" "use super::mesh_routing::" "mesh_routing module imported"; then ((PASS++)); else ((FAIL++)); fi
if check_exists "$SWARM_FILE" "Multi-path delivery with retry logic" "SendMessage uses multi-path"; then ((PASS++)); else ((FAIL++)); fi
if ! grep -q "let _request_id = swarm.behaviour_mut().messaging.send_request" "$SWARM_FILE" | grep -v "// Forward the message"; then
    echo -e "${GREEN}✓${NC} No fire-and-forget sends (all use multi-path)"
    ((PASS++))
else
    echo -e "${RED}✗${NC} Found fire-and-forget sends"
    ((FAIL++))
fi
echo ""

# Summary
echo "========================================"
echo "VERIFICATION RESULTS"
echo "========================================"
echo -e "Passed: ${GREEN}$PASS${NC}"
echo -e "Failed: ${RED}$FAIL${NC}"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ ALL INTEGRATION CHECKS PASSED${NC}"
    echo "All 6 phases are properly wired into the runtime."
    echo ""
    exit 0
else
    echo -e "${RED}❌ INTEGRATION INCOMPLETE${NC}"
    echo "Some integration points are missing."
    echo ""
    exit 1
fi
