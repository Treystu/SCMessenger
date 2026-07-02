#!/usr/bin/env bash
# ==============================================================================
# SCMessenger WireGuard Cross-VM Mesh Tunnel Setup
# ==============================================================================
#
# Sets up a WireGuard tunnel for connecting SCMessenger mesh nodes running
# on separate VMs or cloud instances. This enables testing the proximity
# mesh across physical machine boundaries while still applying tc/netem
# network impairment on the WireGuard interface.
#
# Architecture:
#   VM-A (GCP us-central1)  ←── WireGuard tunnel ──→  VM-B (GCP us-east1)
#        wg-mesh: 10.10.0.1                           wg-mesh: 10.10.0.2
#        + tc netem on wg-mesh                         + tc netem on wg-mesh
#
# This script:
#   1. Generates WireGuard keypairs (or uses existing ones)
#   2. Creates /etc/wireguard/wg-mesh.conf
#   3. Brings up the wg-mesh interface
#   4. Applies tc/netem rules on the WireGuard interface for link simulation
#
# Prerequisites:
#   - WireGuard installed: apt install wireguard-tools
#   - Root/sudo access
#   - UDP port 51820 open between VMs (GCP firewall rule)
#   - iproute2 installed (for tc/netem)
#
# Usage:
#   # On VM-A (initiator):
#   sudo bash setup_cross_vm.sh \
#     --role initiator \
#     --local-ip 10.10.0.1 \
#     --peer-ip 10.10.0.2 \
#     --peer-endpoint <VM-B-PUBLIC-IP>:51820 \
#     --peer-pubkey <VM-B-PUBLIC-KEY> \
#     --netem "delay 50ms 20ms loss 5%"
#
#   # On VM-B (responder):
#   sudo bash setup_cross_vm.sh \
#     --role responder \
#     --local-ip 10.10.0.2 \
#     --peer-ip 10.10.0.1 \
#     --peer-endpoint <VM-A-PUBLIC-IP>:51820 \
#     --peer-pubkey <VM-A-PUBLIC-KEY> \
#     --netem "delay 50ms 20ms loss 5%"
#
#   # Generate keys only (for exchanging public keys before setup):
#   sudo bash setup_cross_vm.sh --genkeys
#
#   # Tear down:
#   sudo bash setup_cross_vm.sh --teardown
#
# ==============================================================================
set -euo pipefail

# --- Defaults ---
WG_IFACE="wg-mesh"
WG_PORT="51820"
WG_CONF_DIR="/etc/wireguard"
WG_CONF="${WG_CONF_DIR}/${WG_IFACE}.conf"
KEY_DIR="${WG_CONF_DIR}/keys"
NETEM_PROFILE=""
ROLE=""
LOCAL_IP=""
PEER_IP=""
PEER_ENDPOINT=""
PEER_PUBKEY=""
ACTION="setup"

# --- Parse Arguments ---
while [[ $# -gt 0 ]]; do
    case "$1" in
        --role)        ROLE="$2";          shift 2 ;;
        --local-ip)    LOCAL_IP="$2";      shift 2 ;;
        --peer-ip)     PEER_IP="$2";       shift 2 ;;
        --peer-endpoint) PEER_ENDPOINT="$2"; shift 2 ;;
        --peer-pubkey) PEER_PUBKEY="$2";   shift 2 ;;
        --netem)       NETEM_PROFILE="$2"; shift 2 ;;
        --port)        WG_PORT="$2";       shift 2 ;;
        --genkeys)     ACTION="genkeys";   shift ;;
        --teardown)    ACTION="teardown";  shift ;;
        --help|-h)
            echo "Usage: $0 [--genkeys | --teardown | --role <initiator|responder> ...]"
            echo "Run '$0 --help' at the top of this script for full documentation."
            exit 0
            ;;
        *)
            echo "ERROR: Unknown argument: $1"
            exit 1
            ;;
    esac
done

# --- Helper Functions ---

log() {
    echo "[wg-mesh] $(date '+%H:%M:%S') $*"
}

ensure_root() {
    if [ "$(id -u)" -ne 0 ]; then
        echo "ERROR: This script must be run as root (sudo)."
        exit 1
    fi
}

generate_keypair() {
    mkdir -p "${KEY_DIR}"
    chmod 700 "${KEY_DIR}"

    if [ ! -f "${KEY_DIR}/private.key" ]; then
        wg genkey | tee "${KEY_DIR}/private.key" | wg pubkey > "${KEY_DIR}/public.key"
        chmod 600 "${KEY_DIR}/private.key"
        log "Generated new WireGuard keypair."
    else
        log "Existing keypair found, reusing."
    fi

    PRIVATE_KEY=$(cat "${KEY_DIR}/private.key")
    PUBLIC_KEY=$(cat "${KEY_DIR}/public.key")

    echo ""
    echo "============================================================"
    echo "  Your WireGuard Public Key (share with peer):"
    echo "  ${PUBLIC_KEY}"
    echo "============================================================"
    echo ""
}

# ==============================================================================
# Action: Generate Keys Only
# ==============================================================================
if [ "${ACTION}" = "genkeys" ]; then
    ensure_root
    generate_keypair
    log "Keys generated. Share the public key above with your peer."
    log "Private key: ${KEY_DIR}/private.key"
    log "Public key:  ${KEY_DIR}/public.key"
    exit 0
fi

# ==============================================================================
# Action: Teardown
# ==============================================================================
if [ "${ACTION}" = "teardown" ]; then
    ensure_root
    log "Tearing down ${WG_IFACE}..."

    # Remove tc rules first
    tc qdisc del dev "${WG_IFACE}" root 2>/dev/null || true

    # Bring down WireGuard interface
    wg-quick down "${WG_IFACE}" 2>/dev/null || ip link del "${WG_IFACE}" 2>/dev/null || true

    log "Interface ${WG_IFACE} removed."
    log "Config file preserved at: ${WG_CONF}"
    log "Keys preserved at: ${KEY_DIR}/"
    exit 0
fi

# ==============================================================================
# Action: Setup
# ==============================================================================
ensure_root

# Validate required parameters
if [ -z "${ROLE}" ] || [ -z "${LOCAL_IP}" ] || [ -z "${PEER_IP}" ] || [ -z "${PEER_ENDPOINT}" ] || [ -z "${PEER_PUBKEY}" ]; then
    echo "ERROR: Missing required parameters for setup."
    echo ""
    echo "Required: --role, --local-ip, --peer-ip, --peer-endpoint, --peer-pubkey"
    echo "Run '$0 --help' for usage information."
    exit 1
fi

# Step 1: Generate keypair if needed
log "Step 1/4: Generating keypair..."
generate_keypair

# Step 2: Create WireGuard configuration
log "Step 2/4: Creating WireGuard config at ${WG_CONF}..."
mkdir -p "${WG_CONF_DIR}"

cat > "${WG_CONF}" << EOF
# ==============================================================================
# SCMessenger WireGuard Cross-VM Mesh Configuration
# Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')
# Role: ${ROLE}
# ==============================================================================

[Interface]
# This VM's private key
PrivateKey = ${PRIVATE_KEY}
# Tunnel IP address for this node
Address = ${LOCAL_IP}/24
# WireGuard listen port
ListenPort = ${WG_PORT}
# Save config on wg-quick down
SaveConfig = false

# Optional: run tc netem after interface comes up
$(if [ -n "${NETEM_PROFILE}" ]; then
    echo "PostUp = tc qdisc add dev ${WG_IFACE} root netem ${NETEM_PROFILE}"
    echo "PreDown = tc qdisc del dev ${WG_IFACE} root 2>/dev/null || true"
else
    echo "# No netem profile specified (add --netem to apply network impairment)"
fi)

[Peer]
# Remote VM's public key
PublicKey = ${PEER_PUBKEY}
# Remote VM's public IP and WireGuard port
Endpoint = ${PEER_ENDPOINT}
# Route traffic for the mesh subnet through this peer
AllowedIPs = ${PEER_IP}/32
# Send keepalive every 25s to maintain NAT mappings
PersistentKeepalive = 25
EOF

chmod 600 "${WG_CONF}"
log "Config written to ${WG_CONF}"

# Step 3: Bring up WireGuard interface
log "Step 3/4: Bringing up ${WG_IFACE}..."

# Remove existing interface if present
wg-quick down "${WG_IFACE}" 2>/dev/null || ip link del "${WG_IFACE}" 2>/dev/null || true

# Start the interface
wg-quick up "${WG_IFACE}"

log "Interface ${WG_IFACE} is up."

# Step 4: Apply tc/netem (if not already done via PostUp)
if [ -n "${NETEM_PROFILE}" ]; then
    log "Step 4/4: Applying tc netem on ${WG_IFACE}..."
    # PostUp should have already applied it, but verify
    if tc qdisc show dev "${WG_IFACE}" | grep -q netem; then
        log "  netem already active: ${NETEM_PROFILE}"
    else
        tc qdisc add dev "${WG_IFACE}" root netem ${NETEM_PROFILE}
        log "  Applied: ${NETEM_PROFILE}"
    fi
else
    log "Step 4/4: No netem profile specified (skipping)"
fi

# --- Status Report ---
echo ""
echo "============================================================"
echo "  WireGuard Cross-VM Mesh Tunnel — Active"
echo "============================================================"
echo "  Interface:    ${WG_IFACE}"
echo "  Local IP:     ${LOCAL_IP}"
echo "  Peer IP:      ${PEER_IP}"
echo "  Peer Endpoint: ${PEER_ENDPOINT}"
echo "  Listen Port:  ${WG_PORT}"
echo "  Role:         ${ROLE}"
if [ -n "${NETEM_PROFILE}" ]; then
    echo "  Netem:        ${NETEM_PROFILE}"
fi
echo "============================================================"
echo ""
echo "  Verify connectivity:"
echo "    ping ${PEER_IP}"
echo "    wg show ${WG_IFACE}"
echo ""
echo "  Teardown:"
echo "    sudo $0 --teardown"
echo "============================================================"
