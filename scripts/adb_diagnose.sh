#!/bin/bash
# ADB Connectivity Diagnostic Script
# Usage: ./adb_diagnose.sh [device_ip]

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "  ADB Connectivity Diagnostic Tool"
echo "=========================================="
echo ""

# 1. Check ADB installation
echo -e "${YELLOW}[1/8]${NC} Checking ADB installation..."
if command -v adb &> /dev/null; then
    ADB_PATH=$(which adb)
    ADB_VERSION=$(adb version | head -1)
    echo -e "  ${GREEN}✓${NC} ADB found: $ADB_PATH"
    echo "  Version: $ADB_VERSION"
else
    echo -e "  ${RED}✗${NC} ADB not found in PATH!"
    echo "  Install with: brew install android-platform-tools"
    exit 1
fi
echo ""

# 2. Check ADB server status
echo -e "${YELLOW}[2/8]${NC} Checking ADB server status..."
adb kill-server 2>/dev/null || true
sleep 1
adb start-server
echo -e "  ${GREEN}✓${NC} ADB server restarted"
echo ""

# 3. List all connected devices
echo -e "${YELLOW}[3/8]${NC} Scanning for connected devices..."
DEVICE_OUTPUT=$(adb devices -l)
echo "$DEVICE_OUTPUT"
echo ""

# Parse device count
DEVICE_COUNT=$(adb devices | grep -v "List" | grep -v "^$" | wc -l | tr -d ' ')
UNAUTHORIZED_COUNT=$(adb devices | grep "unauthorized" | wc -l | tr -d ' ')
OFFLINE_COUNT=$(adb devices | grep "offline" | wc -l | tr -d ' ')

if [ "$DEVICE_COUNT" -eq 0 ]; then
    echo -e "  ${RED}✗${NC} No devices detected!"
    echo ""
    echo "  TROUBLESHOOTING STEPS:"
    echo "  ====================="
    echo ""
    
    # 4. Check USB connections
    echo -e "${YELLOW}[4/8]${NC} Checking USB connections (macOS)..."
    ioreg -p IOUSB -l 2>/dev/null | grep -E "AppleUSB|Android|Google" | head -20 || echo "  No Android USB devices found in IORegistry"
    echo ""
    
    # 5. Check for USB debugging authorization
    echo -e "${YELLOW}[5/8]${NC} USB Debugging Authorization Status:"
    echo "  ${YELLOW}!${NC} If device shows 'unauthorized', check phone for USB debugging permission dialog"
    echo ""
    
    # 6. Network ADB check
    echo -e "${YELLOW}[6/8]${NC} Checking for network ADB devices..."
    echo "  To connect via WiFi, ensure device and Mac are on same network"
    echo "  On device: Settings > Developer Options > Wireless debugging (Android 11+)"
    echo "  Or use: adb tcpip 5555 then adb connect <device_ip>:5555"
    echo ""
    
    # 7. Check common issues
    echo -e "${YELLOW}[7/8]${NC} Common Issues Checklist:"
    echo ""
    echo "  □ USB cable supports data (not charge-only)"
    echo "  □ USB debugging enabled on device (Settings > Developer Options)"
    echo "  □ USB mode set to 'File Transfer' or 'MTP' (not 'Charge only')"
    echo "  □ Accepted USB debugging authorization on device"
    echo "  □ Different USB port/cable tried"
    echo "  □ Developer Options visible (tap Build Number 7 times in Settings > About)"
    echo ""
    
    # 8. Try network connection if IP provided
    if [ -n "$1" ]; then
        echo -e "${YELLOW}[8/8]${NC} Attempting network connection to $1:5555..."
        adb connect "$1":5555
        if adb devices | grep "$1"; then
            echo -e "  ${GREEN}✓${NC} Connected via network!"
        else
            echo -e "  ${RED}✗${NC} Failed to connect. Ensure:"
            echo "    - Device and Mac are on same WiFi network"
            echo "    - Port 5555 is open (use 'adb tcpip 5555' via USB first)"
            echo "    - No firewall blocking the connection"
        fi
    else
        echo -e "${YELLOW}[8/8]${NC} Network connection test skipped (no IP provided)"
        echo "  Usage: $0 <device_ip>"
    fi
    
elif [ "$UNAUTHORIZED_COUNT" -gt 0 ]; then
    echo -e "  ${YELLOW}!${NC} Device detected but UNAUTHORIZED"
    echo ""
    echo "  ACTION REQUIRED:"
    echo "  Check your Android device screen for a USB debugging"
    echo "  authorization dialog and tap 'Allow'."
    echo ""
    echo "  If no dialog appears:"
    echo "  1. Unplug and replug the USB cable"
    echo "  2. Go to Settings > Developer Options"
    echo "  3. Tap 'Revoke USB debugging authorizations'"
    echo "  4. Reconnect and accept the new dialog"
    
elif [ "$OFFLINE_COUNT" -gt 0 ]; then
    echo -e "  ${YELLOW}!${NC} Device detected but OFFLINE"
    echo ""
    echo "  This usually means:"
    echo "  - USB cable issue (try a different cable)"
    echo "  - USB port issue (try a different port)"
    echo "  - Device needs reboot"
    
else
    echo -e "  ${GREEN}✓${NC} $DEVICE_COUNT device(s) connected and authorized!"
    echo ""
    
    # Show device details
    echo -e "${YELLOW}[4/8]${NC} Device Details:"
    adb devices -l | grep -v "List" | grep -v "^$" | while read line; do
        SERIAL=$(echo $line | awk '{print $1}')
        echo "  Device: $SERIAL"
        MODEL=$(adb -s $SERIAL shell getprop ro.product.model 2>/dev/null | tr -d '\r' || echo "Unknown")
        ANDROID=$(adb -s $SERIAL shell getprop ro.build.version.release 2>/dev/null | tr -d '\r' || echo "Unknown")
        API=$(adb -s $SERIAL shell getprop ro.build.version.sdk 2>/dev/null | tr -d '\r' || echo "Unknown")
        echo "  Model: $MODEL"
        echo "  Android: $ANDROID (API $API)"
    done
    echo ""
    
    echo -e "${GREEN}ADB is working correctly!${NC}"
fi

echo ""
echo "=========================================="
echo "  Diagnostic Complete"
echo "=========================================="
