#!/bin/bash

# Bootstrap Node Health Check
# Tests connectivity to all configured bootstrap/relay nodes
# Supports env var override: SCMESSENGER_BOOTSTRAP_NODES (comma-separated host:port)

set -euo pipefail

echo "=== SCMessenger Bootstrap Node Health Check ==="
echo "Started at: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
echo ""

# Resolve bootstrap nodes: env var > hardcoded defaults
if [ -n "${SCMESSENGER_BOOTSTRAP_NODES:-}" ]; then
    IFS=',' read -ra BOOTSTRAP_NODES <<< "$SCMESSENGER_BOOTSTRAP_NODES"
    echo "Using SCMESSENGER_BOOTSTRAP_NODES env override"
else
    # Primary nodes (actual deployed infrastructure)
    BOOTSTRAP_NODES=(
        "34.135.34.73:9001"       # GCP primary (us-central1)
        "104.28.216.43:443"       # Cloudflare relay
    )
    echo "Using default bootstrap node list"
fi

# Test each bootstrap node
FAILED_NODES=()
HEALTHY_NODES=()

for node in "${BOOTSTRAP_NODES[@]}"; do
    HOST="${node%%:*}"
    PORT="${node##*:}"
    echo "Testing $node ..."

    FAILED=0

    # 1. TCP connectivity (timeout 5s)
    if timeout 5 bash -c "echo >/dev/tcp/$HOST/$PORT" 2>/dev/null; then
        echo "  [PASS] TCP connection successful"
    else
        echo "  [FAIL] TCP connection failed"
        FAILED=1
    fi

    # 2. TLS certificate check (port 443 or HTTPS)
    if [ "$PORT" = "443" ] || [ "$PORT" = "9000" ]; then
        CERT_RESULT=$(echo | timeout 5 openssl s_client -connect "$HOST:$PORT" -servername "$HOST" 2>/dev/null | openssl x509 -noout -dates 2>/dev/null || echo "")
        if [ -n "$CERT_RESULT" ]; then
            echo "  [PASS] TLS certificate valid"
            echo "  $CERT_RESULT"
        else
            echo "  [WARN] TLS check failed (may not use TLS)"
        fi
    fi

    # 3. ICMP ping (may be blocked by firewall)
    if ping -c 1 -W 2 "$HOST" >/dev/null 2>&1; then
        echo "  [PASS] ICMP ping successful"
    else
        echo "  [WARN] ICMP ping failed (may be firewall-blocked)"
    fi

    # 4. DNS resolution
    if [ "$HOST" != "${HOST#*[a-zA-Z]}" ]; then
        DNS_RESULT=$(host "$HOST" 2>/dev/null | head -1 || echo "")
        if [ -n "$DNS_RESULT" ]; then
            echo "  [PASS] DNS: $DNS_RESULT"
        else
            echo "  [FAIL] DNS resolution failed"
            FAILED=1
        fi
    fi

    if [ "$FAILED" -eq 0 ]; then
        HEALTHY_NODES+=("$node")
    else
        FAILED_NODES+=("$node")
    fi
    echo ""
done

# Summary
TOTAL=${#BOOTSTRAP_NODES[@]}
HEALTHY=${#HEALTHY_NODES[@]}
FAILED_COUNT=${#FAILED_NODES[@]}

echo "=== Health Check Summary ==="
echo "Total: $TOTAL | Healthy: $HEALTHY | Failed: $FAILED_COUNT"

if [ "$FAILED_COUNT" -gt 0 ]; then
    echo ""
    echo "Failed nodes:"
    for f in "${FAILED_NODES[@]}"; do echo "  - $f"; done

    # Fallback discovery
    echo ""
    echo "=== Fallback Discovery ==="

    # DNS-based SRV record lookup
    echo "Checking DNS SRV records..."
    SRV=$(host -t SRV _scmessenger._tcp.scmessenger.net 2>/dev/null || echo "")
    if [ -n "$SRV" ]; then
        echo "  Found SRV: $SRV"
    else
        echo "  No SRV records found"
    fi

    # Ledger-based discovery
    echo "Checking local relay ledger..."
    LEDGER="/tmp/scmessenger_ledger.json"
    if [ -f "$LEDGER" ]; then
        NODE_COUNT=$(jq '.relays | length' "$LEDGER" 2>/dev/null || echo "0")
        echo "  Found $NODE_COUNT relay nodes in ledger"
    else
        echo "  No ledger file found"
    fi

    # Local network mDNS discovery
    echo "Checking local network for mDNS peers..."
    if command -v avahi-browse >/dev/null 2>&1; then
        MDNS_PEERS=$(timeout 3 avahi-browse -rt _scmessenger._tcp 2>/dev/null | grep "=.*=" | wc -l || echo "0")
        echo "  Found $MDNS_PEERS mDNS peers"
    else
        echo "  avahi-browse not available"
    fi

    # Environment variable suggestion
    echo ""
    echo "To override bootstrap nodes, set:"
    echo "  export SCMESSENGER_BOOTSTRAP_NODES=host1:port1,host2:port2"

    exit 1
else
    echo ""
    echo "All bootstrap nodes healthy!"
    exit 0
fi