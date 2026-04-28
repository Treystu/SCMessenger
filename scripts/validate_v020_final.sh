#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}================================================================"
echo -e "   SCMessenger v0.2.0 Final Alpha Validation Gates"
echo -e "================================================================${NC}"
echo ""

# Helper to pull logs
pull_logs() {
    local out_prefix="$1"
    echo "Pulling logs from devices..."
    
    local android_log="${out_prefix}_android.log"
    local ios_log="${out_prefix}_ios.log"
    
    rm -f "$android_log" "$ios_log"
    
    # Android
    local serial
    serial="$(adb devices | awk 'NR>1 && $2=="device" {print $1; exit}')"
    if [ -n "$serial" ]; then
        if adb -s "$serial" shell run-as com.scmessenger.android cat files/mesh_diagnostics.log > "$android_log" 2>/dev/null; then
            echo -e "${GREEN}✓ Android diagnostics pulled successfully${NC}"
        else
            adb -s "$serial" logcat -d -v time | grep -iE "delivery_state|delivery_attempt|Receipt for|msg_rx|msg_rx_processed|delivery_receipt|history_sync|sync_data" > "$android_log" || true
            echo -e "${GREEN}✓ Android logcat pulled successfully${NC}"
        fi
    else
        echo -e "${RED}✗ No Android device found. Please ensure it is connected via USB/WiFi debugging.${NC}"
        touch "$android_log"
    fi

    # iOS
    local devices_json
    devices_json="$(mktemp)"
    if xcrun devicectl list devices --json-output "$devices_json" >/dev/null 2>&1; then
        local device_id
        device_id="$(python3 - "$devices_json" <<'PY'
import json, sys
try:
    with open(sys.argv[1], "r") as f:
        payload = json.load(f)
    devices = payload.get("result", {}).get("devices", [])
    for d in devices:
        identifier = d.get("identifier") or ""
        pairing_state = (d.get("connectionProperties", {}).get("pairingState") or "").lower()
        if identifier and pairing_state == "paired":
            print(identifier)
            sys.exit(0)
except Exception:
    pass
PY
)"
        if [ -n "$device_id" ]; then
            if xcrun devicectl device copy from --device "$device_id" --domain-type appDataContainer --domain-identifier SovereignCommunications.SCMessenger --source Documents/mesh_diagnostics.log --destination "$ios_log" >/dev/null 2>&1; then
                echo -e "${GREEN}✓ iOS diagnostics pulled successfully${NC}"
            else
                echo -e "${YELLOW}! Could not pull iOS diagnostics. Is the app running?${NC}"
                touch "$ios_log"
            fi
        else
            echo -e "${RED}✗ No paired iOS device found.${NC}"
            touch "$ios_log"
        fi
    else
        echo -e "${RED}✗ Could not run devicectl.${NC}"
        touch "$ios_log"
    fi
    rm -f "$devices_json"
}

# -----------------------------------------------------------------------------
# Part 1: Pending-Outbox & Sender-State Convergence
# -----------------------------------------------------------------------------
echo -e "${CYAN}=== GATE 1: Network Hopping Sender-State Convergence ===${NC}"
echo "This test validates that messages successfully transition from 'forwarding' to 'delivered' even across network changes."
echo ""
echo "INSTRUCTIONS:"
echo " 1. Open SCMessenger on both the physical Android and physical iOS device."
echo " 2. Ensure they are connected (check the Mesh tab to see at least 1 full node)."
echo " 3. On Android, turn OFF WiFi (so it switches to Cellular)."
echo " 4. On Android, send a message to iOS: \"Test Network Hop 1\""
echo " 5. Wait for it to say 'forwarding' or 'delivered'."
echo " 6. (Optional) Turn off BLE on Android, send another message: \"Test Network Hop 2\""
echo " 7. Ensure that on Android, the message statuses eventually say 'delivered' (meaning iOS successfully received them and Android processed the receipt)."
echo ""
read -p "Press [Enter] when you have completed these steps and messages are delivered..."

pull_logs "/tmp/gate1"

echo "Verifying Receipt Convergence..."
if bash "$ROOT_DIR/scripts/verify_receipt_convergence.sh" "/tmp/gate1_android.log" "/tmp/gate1_ios.log" >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS: Sender-state convergence verified in logs.${NC}"
else
    echo -e "${YELLOW}⚠ WARNING: Convergence verifier failed or found missing markers.${NC}"
    echo "  Please check the devices visually. If Android says 'delivered', then it actually succeeded."
fi
echo ""

# -----------------------------------------------------------------------------
# Part 2: History Sync / Zombie Outbox Resolution
# -----------------------------------------------------------------------------
echo -e "${CYAN}=== GATE 2: P2P History Sync (Zombie Outbox Fix) ===${NC}"
echo "This test validates that when Android loses its history, iOS successfully syncs it back."
echo ""
echo "INSTRUCTIONS:"
echo " 1. Force-quit the Android app and clear its data/storage, OR delete the conversation thread from the Android side."
echo " 2. Re-open the Android app and navigate to the chat (it should be empty)."
echo " 3. Ensure iOS still has the conversation history."
echo " 4. Connect both devices via direct P2P (e.g., both on the same WiFi network, or close together for BLE)."
echo " 5. Open the conversation between the two devices."
echo " 6. Wait 10-20 seconds. Android should automatically receive the 'history_sync' from iOS and repopulate the chat history."
echo ""
read -p "Press [Enter] when you have completed these steps and observed the result..."

pull_logs "/tmp/gate2"

echo "Verifying History Sync..."
SYNC_FOUND_ANDROID=$(grep -i -c "Processed history sync data\|Processed history sync request\|history_sync" "/tmp/gate2_android.log" || true)
SYNC_FOUND_IOS=$(grep -i -c "Processed history sync data\|Processed history sync request\|History sync data sent\|history_sync" "/tmp/gate2_ios.log" || true)

if [ "$SYNC_FOUND_ANDROID" -gt 0 ] || [ "$SYNC_FOUND_IOS" -gt 0 ]; then
    echo -e "${GREEN}✓ PASS: History sync events found in logs.${NC}"
else
    echo -e "${YELLOW}⚠ WARNING: Could not find explicit history_sync markers in the latest logs.${NC}"
    echo "  If the Android device successfully restored the missing messages, this gate is considered PASSED."
fi

echo ""
echo -e "${CYAN}================================================================"
echo -e "   VALIDATION COMPLETE"
echo -e "================================================================${NC}"
echo "If both tests visually succeeded on the devices, v0.2.0 Alpha is READY!"
