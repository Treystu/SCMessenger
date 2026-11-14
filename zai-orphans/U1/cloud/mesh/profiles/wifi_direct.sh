#!/usr/bin/env bash
# ==============================================================================
# SCMessenger Network Profile: WiFi Direct
# ==============================================================================
#
# Simulates a WiFi Direct (P2P) connection between two nearby devices.
#
# Characteristics:
#   - Latency:    5ms ± 2ms (near-LAN performance)
#   - Loss:       0.5% (very reliable link)
#   - Bandwidth:  ~50 Mbit/s (WiFi Direct effective throughput)
#
# Real-world equivalent:
#   Two phones connected via WiFi Direct group, one acting as Group Owner.
#   Best-case proximity transport with high throughput for file transfers
#   and real-time messaging.
#
# Usage:
#   sudo bash profiles/wifi_direct.sh            # Apply profile
#   sudo tc qdisc del dev eth0 root 2>/dev/null   # Remove profile
#
# Requires: iproute2 (tc), NET_ADMIN capability
# ==============================================================================
set -euo pipefail

IFACE="${IFACE:-eth0}"

echo "[wifi_direct] Applying WiFi Direct profile on ${IFACE}..."
echo "  delay: 5ms ± 2ms | loss: 0.5% | rate: 50mbit"

# Remove any existing qdisc first (ignore errors if none exists)
tc qdisc del dev "${IFACE}" root 2>/dev/null || true

# Apply the WiFi Direct profile
tc qdisc add dev "${IFACE}" root netem delay 5ms 2ms loss 0.5% rate 50mbit

echo "[wifi_direct] Profile applied successfully."
