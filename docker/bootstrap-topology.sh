#!/bin/bash
set -e

# Bootstrap script for farm-sim topology
# Wires identity discovery via existing /api/identity and /api/contacts endpoints

NODES=(alice bob carol david eve relay1 relay2)
API_PORT=9876
MAX_RETRIES=30
RETRY_DELAY=2

# Wait for node to be healthy
wait_node_healthy() {
    local node=$1
    local container="scm-$node"

    for i in $(seq 1 $MAX_RETRIES); do
        if docker exec "$container" curl -s "http://127.0.0.1:$API_PORT/api/identity" >/dev/null 2>&1; then
            echo "[OK] $node is healthy"
            return 0
        fi
        echo "[WAIT] $node starting... ($i/$MAX_RETRIES)"
        sleep "$RETRY_DELAY"
    done

    echo "[ERROR] $node failed to start"
    return 1
}

# Fetch node identity (raw JSON response)
get_node_identity() {
    local node=$1
    local container="scm-$node"

    docker exec "$container" curl -s "http://127.0.0.1:$API_PORT/api/identity"
}

# Add contact to node. handle_add_contact (cli/src/api.rs) expects
# {"peer_id": "<libp2p_peer_id>", "public_key": "<public_key_hex>", "name": "<optional>"}
add_contact() {
    local node=$1
    local peer_id=$2
    local public_key=$3
    local name=$4
    local container="scm-$node"

    docker exec "$container" curl -s -X POST "http://127.0.0.1:$API_PORT/api/contacts" \
        -H "Content-Type: application/json" \
        -d "{\"peer_id\":\"$peer_id\",\"public_key\":\"$public_key\",\"name\":\"$name\"}" >/dev/null 2>&1
}

echo "=== Bootstrap Farm-Sim Topology ==="

# Step 1: Wait for all nodes to be healthy
echo "[PHASE 1] Waiting for all nodes to start..."
for node in "${NODES[@]}"; do
    wait_node_healthy "$node"
done

# Step 2: Fetch all identities. handle_get_identity (cli/src/api.rs) returns
# {"identity_id", "public_key_hex", "device_id", "seniority_timestamp",
#  "initialized", "nickname", "libp2p_peer_id"} -- libp2p_peer_id is the real
# network peer ID used for dialing/routing; public_key_hex is the crypto
# identity key used for encryption. Both are needed to add a usable contact.
echo "[PHASE 2] Fetching node identities..."
declare -A peer_ids
declare -A public_keys

for node in "${NODES[@]}"; do
    echo "[FETCH] $node identity..."
    response=$(get_node_identity "$node")
    peer_id=$(echo "$response" | jq -r '.libp2p_peer_id')
    public_key=$(echo "$response" | jq -r '.public_key_hex')
    if [ -z "$peer_id" ] || [ "$peer_id" = "null" ]; then
        echo "[ERROR] $node returned no libp2p_peer_id: $response"
        exit 1
    fi
    peer_ids["$node"]="$peer_id"
    public_keys["$node"]="$public_key"
    echo "[OK] $node = $peer_id"
done

# Step 3: Cross-provision contacts (every node learns every other node)
echo "[PHASE 3] Provisioning contacts..."
for source in "${NODES[@]}"; do
    for target in "${NODES[@]}"; do
        if [ "$source" != "$target" ]; then
            add_contact "$source" "${peer_ids[$target]}" "${public_keys[$target]}" "$target"
            echo "[PROV] $source -> $target"
        fi
    done
done

# Step 4: Verify provisioning
echo "[PHASE 4] Verifying provisioning..."
for node in "${NODES[@]}"; do
    container="scm-$node"
    contact_count=$(docker exec "$container" curl -s "http://127.0.0.1:$API_PORT/api/contacts" | jq '.contacts | length' 2>/dev/null || echo "0")
    if [ "$contact_count" -eq 6 ]; then
        echo "[OK] $node has 6 contacts"
    else
        echo "[WARN] $node has $contact_count contacts (expected 6)"
    fi
done

echo ""
echo "=== Bootstrap Complete ==="
echo "Farm-sim topology is ready for Phase 2 & 3 testing"
