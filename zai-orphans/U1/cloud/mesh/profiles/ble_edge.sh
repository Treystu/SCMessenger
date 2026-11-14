#!/usr/bin/env bash
# ==============================================================================
# SCMessenger Network Profile: BLE Edge-of-Range
# ==============================================================================
#
# Simulates a Bluetooth Low Energy connection at the edge of reliable range
# (~10-15 meters, through walls or with interference).
#
# Characteristics:
#   - Latency:    80ms ± 40ms (high jitter from retransmissions)
#   - Loss:       12% (significant at range boundary)
#   - Reorder:    5% (out-of-order delivery from link-layer retries)
#   - Bandwidth:  ~256 Kbit/s (degraded throughput at range edge)
#
# Real-world equivalent:
#   Two phones in adjacent rooms or across a hallway, connection frequently
#   drops to minimum power level, link-layer retransmissions are common.
#   This is the zone where DTN store-and-forward becomes essential.
#
# Usage:
#   sudo bash profiles/ble_edge.sh              # Apply profile
#   sudo tc qdisc del dev eth0 root 2>/dev/null  # Remove profile
#
# Requires: iproute2 (tc), NET_ADMIN capability
# ==============================================================================
set -euo pipefail

IFACE="${IFACE:-eth0}"

echo "[ble_edge] Applying BLE edge-of-range profile on ${IFACE}..."
echo "  delay: 80ms ± 40ms | loss: 12% | reorder: 5% | rate: 256kbit"

# Remove any existing qdisc first (ignore errors if none exists)
tc qdisc del dev "${IFACE}" root 2>/dev/null || true

# Apply the BLE edge-of-range profile
tc qdisc add dev "${IFACE}" root netem delay 80ms 40ms loss 12% reorder 5% rate 256kbit

echo "[ble_edge] Profile applied successfully."
