#!/usr/bin/env bash
# ==============================================================================
# SCMessenger Network Profile: BLE Nearby
# ==============================================================================
#
# Simulates a Bluetooth Low Energy connection with a nearby device (~1-3 meters).
#
# Characteristics:
#   - Latency:    15ms ± 5ms (connection interval dependent, typically 7.5-15ms)
#   - Loss:       1% (minimal at close range)
#   - Bandwidth:  ~1 Mbit/s (BLE 4.2 effective throughput)
#
# Real-world equivalent:
#   Two phones side-by-side exchanging messages over BLE GATT characteristics.
#   Connection is stable with minimal interference.
#
# Usage:
#   sudo bash profiles/ble_nearby.sh          # Apply profile
#   sudo tc qdisc del dev eth0 root 2>/dev/null  # Remove profile
#
# Requires: iproute2 (tc), NET_ADMIN capability
# ==============================================================================
set -euo pipefail

IFACE="${IFACE:-eth0}"

echo "[ble_nearby] Applying BLE nearby profile on ${IFACE}..."
echo "  delay: 15ms ± 5ms | loss: 1% | rate: 1mbit"

# Remove any existing qdisc first (ignore errors if none exists)
tc qdisc del dev "${IFACE}" root 2>/dev/null || true

# Apply the BLE nearby profile
tc qdisc add dev "${IFACE}" root netem delay 15ms 5ms loss 1% rate 1mbit

echo "[ble_nearby] Profile applied successfully."
