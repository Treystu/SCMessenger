#!/bin/bash
set -e

echo "[INFO] Starting contact provisioning..."

# Array of node names
NODES=("alice" "bob" "carol" "david" "eve" "relay1" "relay2")

# Wait for identity files to be created
echo "[INFO] Waiting for identity files to be created..."
for retry in {1..20}; do
    ready=0
    for node_name in "${NODES[@]}"; do
        if docker exec scm-$node_name test -f /tmp/scm_identity_${node_name}.json 2>/dev/null; then
            ready=$((ready + 1))
        fi
    done

    if [ $ready -eq ${#NODES[@]} ]; then
        echo "[OK] All identity files ready"
        break
    else
        echo "[$retry/20] Waiting... ($ready/${#NODES[@]} identity files ready)"
    fi
done

# Build a map of node names to their identities
declare -A PEER_IDS
declare -A PUBLIC_KEYS

echo "[INFO] Reading identities from files..."
for node_name in "${NODES[@]}"; do
    identity_file="/tmp/scm_identity_${node_name}.json"

    if ! docker exec scm-$node_name test -f "$identity_file" 2>/dev/null; then
        echo "[ERROR] Identity file not found for $node_name: $identity_file"
        continue
    fi

    identity=$(docker exec scm-$node_name cat "$identity_file" 2>/dev/null || echo "{}")

    peer_id=$(echo "$identity" | jq -r '.peer_id // empty' 2>/dev/null)
    pub_key=$(echo "$identity" | jq -r '.public_key // empty' 2>/dev/null)

    if [ -z "$peer_id" ] || [ "$peer_id" = "null" ]; then
        echo "[ERROR] Failed to get peer_id for $node_name"
        echo "  File contents: $identity"
        continue
    fi

    if [ -z "$pub_key" ] || [ "$pub_key" = "null" ]; then
        echo "[ERROR] Failed to get public_key for $node_name"
        echo "  File contents: $identity"
        continue
    fi

    PEER_IDS[$node_name]=$peer_id
    PUBLIC_KEYS[$node_name]=$pub_key

    echo "[OK] $node_name: peer_id=${peer_id:0:20}..., pk=${pub_key:0:16}..."
done

# Wait for APIs to be ready
echo "[INFO] Waiting for APIs to be ready..."
for retry in {1..30}; do
    api_ready=$(docker exec scm-alice sh -c "curl -s -m 2 127.0.0.1:9876/api/peers 2>&1 | grep -c peers" 2>/dev/null || echo "0")
    if [ "$api_ready" = "1" ]; then
        echo "[OK] APIs ready"
        break
    fi
    echo "[$retry/30] Waiting for API..."
done

# Now provision all contacts
echo "[INFO] Provisioning contacts..."
for node_name in alice bob carol david eve; do
    echo "[INFO] Provisioning contacts on $node_name..."

    for peer_name in alice bob carol david eve relay1 relay2; do
        if [ "$peer_name" != "$node_name" ]; then
            peer_id=${PEER_IDS[$peer_name]}
            pub_key=${PUBLIC_KEYS[$peer_name]}

            if [ -z "$peer_id" ] || [ -z "$pub_key" ]; then
                echo "[WARNING] Skipping $peer_name (missing peer_id or public_key)"
                continue
            fi

            # Add contact via HTTP API
            response=$(docker exec scm-$node_name sh -c \
                "curl -s -X POST 127.0.0.1:9876/api/contacts \
                  -H 'Content-Type: application/json' \
                  -d '{\"peer_id\":\"$peer_id\",\"public_key\":\"$pub_key\",\"name\":\"$peer_name\"}' \
                  2>&1" 2>/dev/null || echo "{}")

            success=$(echo "$response" | jq -r '.success // false' 2>/dev/null)

            if [ "$success" = "true" ]; then
                echo "[OK] Added $peer_name to $node_name"
            else
                error=$(echo "$response" | jq -r '.error // "Unknown error"' 2>/dev/null)
                echo "[WARNING] Failed to add $peer_name to $node_name: $error"
            fi
        fi
    done
done

# Verify provisioning
echo "[INFO] Verifying contact provisioning..."
for node_name in alice bob carol david eve; do
    count=$(docker exec scm-$node_name sh -c \
        "curl -s 127.0.0.1:9876/api/contacts 2>&1 | jq '.contacts | length' 2>/dev/null" 2>/dev/null || echo "0")

    echo "[INFO] $node_name has $count contacts (expected 6)"
done

echo "[OK] Contact provisioning complete"
