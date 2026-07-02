#!/usr/bin/env bash
# ==============================================================================
# SCMessenger Network Profile: DTN Store-and-Carry
# ==============================================================================
#
# Simulates Delay-Tolerant Networking (DTN) store-and-carry behavior by
# toggling the network link between "connected" and "disconnected" states
# at random intervals. This models physical device movement in and out of
# proximity range.
#
# Behavior:
#   Connected state:  delay 50ms ± 20ms, loss 5% (BLE-like link)
#   Disconnected state: loss 100% (complete blackout, simulating out-of-range)
#
# The script runs in an infinite loop, cycling between states with random
# intervals to simulate realistic human movement patterns:
#   - Connected window:    5-15 seconds (brief encounter)
#   - Disconnected window: 10-30 seconds (walking between encounters)
#
# This profile is designed for testing the DTN store-and-forward layer:
#   - Messages should be queued during disconnected periods
#   - Messages should be forwarded when connectivity is restored
#   - The application should handle frequent connect/disconnect gracefully
#
# Usage:
#   sudo bash profiles/dtn_carry.sh &             # Run in background
#   kill %1                                        # Stop simulation
#
# Environment variables:
#   IFACE               — Network interface (default: eth0)
#   MIN_CONNECTED_SECS  — Minimum connected window (default: 5)
#   MAX_CONNECTED_SECS  — Maximum connected window (default: 15)
#   MIN_DISCONNECTED_SECS — Minimum disconnected window (default: 10)
#   MAX_DISCONNECTED_SECS — Maximum disconnected window (default: 30)
#
# Requires: iproute2 (tc), NET_ADMIN capability
# ==============================================================================
set -euo pipefail

IFACE="${IFACE:-eth0}"
MIN_CONNECTED_SECS="${MIN_CONNECTED_SECS:-5}"
MAX_CONNECTED_SECS="${MAX_CONNECTED_SECS:-15}"
MIN_DISCONNECTED_SECS="${MIN_DISCONNECTED_SECS:-10}"
MAX_DISCONNECTED_SECS="${MAX_DISCONNECTED_SECS:-30}"

CYCLE=0

# Cleanup on exit: remove any tc rules we applied
cleanup() {
    echo "[dtn_carry] Cleaning up tc rules on ${IFACE}..."
    tc qdisc del dev "${IFACE}" root 2>/dev/null || true
    echo "[dtn_carry] Stopped."
    exit 0
}
trap cleanup EXIT INT TERM

# Generate a random integer in [min, max]
rand_between() {
    local min=$1 max=$2
    echo $(( RANDOM % (max - min + 1) + min ))
}

echo "[dtn_carry] Starting DTN store-and-carry simulation on ${IFACE}"
echo "  Connected:    delay 50ms ± 20ms, loss 5%"
echo "  Disconnected: loss 100%"
echo "  Connected window:    ${MIN_CONNECTED_SECS}-${MAX_CONNECTED_SECS}s"
echo "  Disconnected window: ${MIN_DISCONNECTED_SECS}-${MAX_DISCONNECTED_SECS}s"
echo ""

while true; do
    CYCLE=$((CYCLE + 1))

    # --- Connected phase: simulate proximity encounter ---
    CONNECTED_DURATION=$(rand_between "$MIN_CONNECTED_SECS" "$MAX_CONNECTED_SECS")
    echo "[dtn_carry] Cycle ${CYCLE}: CONNECTED for ${CONNECTED_DURATION}s (delay 50ms loss 5%)"

    tc qdisc del dev "${IFACE}" root 2>/dev/null || true
    tc qdisc add dev "${IFACE}" root netem delay 50ms 20ms loss 5%

    sleep "$CONNECTED_DURATION"

    # --- Disconnected phase: simulate out-of-range movement ---
    DISCONNECTED_DURATION=$(rand_between "$MIN_DISCONNECTED_SECS" "$MAX_DISCONNECTED_SECS")
    echo "[dtn_carry] Cycle ${CYCLE}: DISCONNECTED for ${DISCONNECTED_DURATION}s (loss 100%)"

    tc qdisc del dev "${IFACE}" root 2>/dev/null || true
    tc qdisc add dev "${IFACE}" root netem loss 100%

    sleep "$DISCONNECTED_DURATION"
done
