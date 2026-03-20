#!/bin/bash

# Test script for bootstrap node health checks
# This script verifies connectivity to all configured bootstrap nodes
# and provides fallback discovery mechanisms

set -e

echo "=== Bootstrap Node Health Check ==="
echo "Testing connectivity to all configured bootstrap nodes..."

# Configuration - replace with actual bootstrap node addresses
BOOTSTRAP_NODES=(
    "bootstrap1.scmessenger.net:4001"
    "bootstrap2.scmessenger.net:4001"
    "bootstrap3.scmessenger.net:4001"
    "bootstrap4.scmessenger.net:4001"
)

# Test each bootstrap node
FAILED_NODES=()
for node in "${BOOTSTRAP_NODES[@]}"; do
    echo "Testing $node..."
    
    # Test TCP connectivity (timeout after 5 seconds)
    if timeout 5 bash -c "</dev/tcp/$node" 2>/dev/null; then
        echo "✓ $node - TCP connection successful"
    else
        echo "✗ $node - TCP connection failed"
        FAILED_NODES+=("$node")
    fi
done

# Summary
TOTAL_NODES=${#BOOTSTRAP_NODES[@]}
WORKING_NODES=$((TOTAL_NODES - ${#FAILED_NODES[@]}))
FAILED_COUNT=${#FAILED_NODES[@]}

echo ""
echo "=== Health Check Summary ==="
echo "Total nodes tested: $TOTAL_NODES"
echo "Working nodes: $WORKING_NODES"
echo "Failed nodes: $FAILED_COUNT"

if [ $FAILED_COUNT -gt 0 ]; then
    echo ""
    echo "Failed nodes:"
    for failed in "${FAILED_NODES[@]}"; do
        echo "  - $failed"
    done
    
    # Fallback discovery - try to find alternative nodes
    echo ""
    echo "=== Attempting Fallback Discovery ==="
    
    # Try DNS-based discovery
    echo "Checking DNS for alternative bootstrap nodes..."
    if host bootstrap.scmessenger.net >/dev/null 2>&1; then
        echo "✓ DNS discovery successful - alternative nodes available"
    else
        echo "✗ DNS discovery failed"
    fi
    
    # Try ledger-based discovery (if available)
    echo "Checking ledger for stable relay nodes..."
    if [ -f "/tmp/scmessenger_ledger.json" ]; then
        echo "✓ Ledger-based discovery available"
        NODE_COUNT=$(jq '.relays | length' /tmp/scmessenger_ledger.json 2>/dev/null || echo "0")
        echo "  Found $NODE_COUNT relay nodes in ledger"
    else
        echo "✗ No ledger file found"
    fi
    
    exit 1
else
    echo ""
    echo "✓ All bootstrap nodes healthy!"
    exit 0
fi