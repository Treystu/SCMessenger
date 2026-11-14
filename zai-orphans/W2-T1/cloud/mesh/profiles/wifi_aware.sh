#!/usr/bin/env bash
# ==============================================================================
# SCMessenger Network Profile: WiFi Aware (NAN)
# ==============================================================================
#
# Simulates a WiFi Aware (Neighbor Awareness Networking) connection.
#
# Characteristics:
#   - Latency:    25ms ± 10ms (discovery windows add latency)
#   - Loss:       3% (moderate, can vary with discovery cycle)
#   - Duplicate:  1% (NAN publish/subscribe can cause duplicates)
#   - Bandwidth:  ~20 Mbit/s (WiFi Aware data path throughput)
#
# Real-world equivalent:
#   Two Android devices using WiFi Aware NAN data path. Service discovery
#   adds initial latency, but once a data path is established, throughput
#   is reasonable. Duplicate detection in the protocol layer is important.
#
# Usage:
#   sudo bash profiles/wifi_aware.sh             # Apply profile
#   sudo tc qdisc del dev eth0 root 2>/dev/null   # Remove profile
#
# Requires: iproute2 (tc), NET_ADMIN capability
# ==============================================================================
set -euo pipefail

IFACE="${IFACE:-eth0}"

echo "[wifi_aware] Applying WiFi Aware (NAN) profile on ${IFACE}..."
echo "  delay: 25ms ± 10ms | loss: 3% | duplicate: 1% | rate: 20mbit"

# Remove any existing qdisc first (ignore errors if none exists)
tc qdisc del dev "${IFACE}" root 2>/dev/null || true

# Apply the WiFi Aware profile
tc qdisc add dev "${IFACE}" root netem delay 25ms 10ms loss 3% duplicate 1% rate 20mbit

echo "[wifi_aware] Profile applied successfully."
