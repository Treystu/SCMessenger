#!/usr/bin/env bash
# scripts/preflight.sh — Unified pre-flight validation for SCMessenger debug sessions
#
# Usage:
#   ./scripts/preflight.sh
#
# Validates:
#   - Required commands (adb, xcrun, cargo, gcloud, jq, rg)
#   - Android device connectivity
#   - iOS device/simulator connectivity
#   - Build artifact freshness
#   - GCP relay reachability
#   - Log directory health
#
# Exit codes:
#   0 = all checks passed (warnings allowed)
#   1 = one or more critical failures
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS=0
WARN=0
FAIL=0

check_pass() { ((PASS++)); echo -e "  ${GREEN}✓${NC} $*"; }
check_warn() { ((WARN++)); echo -e "  ${YELLOW}⚠${NC} $*"; }
check_fail() { ((FAIL++)); echo -e "  ${RED}✗${NC} $*"; }

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  SCMessenger Pre-Flight Check                               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# 1. Required commands
echo "1. Required Commands:"
for cmd in adb xcrun cargo gcloud jq rg; do
  if command -v "$cmd" >/dev/null 2>&1; then
    check_pass "$cmd found: $(command -v "$cmd")"
  else
    check_fail "$cmd not found in PATH"
  fi
done
echo ""

# 2. Android device detection
echo "2. Android Devices:"
ANDROID_SERIAL=""
if adb devices 2>/dev/null | awk 'NR>1 && $2=="device" {print $1; exit}' | grep -q .; then
  ANDROID_SERIAL=$(adb devices -l | awk 'NR>1 && $2=="device"{print $1; exit}')
  ANDROID_MODEL=$(adb devices -l | awk -v ser="$ANDROID_SERIAL" '$1==ser {for(i=1;i<=NF;i++) if($i~/model:/) {sub("model:","",$i); print $i; exit}}')
  check_pass "Device connected: $ANDROID_SERIAL ($ANDROID_MODEL)"
else
  # Try wireless reconnect
  while IFS= read -r endpoint; do
    [ -n "$endpoint" ] || continue
    adb connect "$endpoint" >/dev/null 2>&1 || true
  done < <(adb mdns services 2>/dev/null | awk '/_adb-tls-connect\._tcp/ {print $NF}')
  
  if adb devices 2>/dev/null | awk 'NR>1 && $2=="device" {print $1; exit}' | grep -q .; then
    ANDROID_SERIAL=$(adb devices -l | awk 'NR>1 && $2=="device"{print $1; exit}')
    check_pass "Wireless device reconnected: $ANDROID_SERIAL"
  else
    check_fail "No Android device connected (tried USB + wireless)"
  fi
fi
echo ""

# 3. iOS device detection
echo "3. iOS Devices:"
IOS_DEVICE_UDID=""
if xcrun devicectl list devices --hide-default-columns --columns Identifier --columns State --hide-headers 2>/dev/null | awk '$2 ~ /(available|connected)/ {print $1; exit}' | grep -q .; then
  IOS_DEVICE_UDID=$(xcrun devicectl list devices --hide-default-columns --columns Identifier --columns State --hide-headers 2>/dev/null | awk '$2 ~ /(available|connected)/ {print $1; exit}')
  IOS_DEVICE_NAME=$(xcrun devicectl list devices --hide-default-columns --columns Name --columns Identifier --hide-headers 2>/dev/null | awk -v id="$IOS_DEVICE_UDID" '$2==id {print $1; exit}')
  check_pass "Physical device: ${IOS_DEVICE_NAME:-Unknown} ($IOS_DEVICE_UDID)"
else
  check_warn "No physical iOS device connected"
fi

IOS_SIM_UDID=""
if xcrun simctl list devices 2>/dev/null | grep -q Booted; then
  IOS_SIM_UDID=$(xcrun simctl list devices | awk -F '[()]' '/Booted/{print $2; exit}')
  IOS_SIM_NAME=$(xcrun simctl list devices | grep "$IOS_SIM_UDID" | sed 's/^[[:space:]]*//' | awk '{print $1, $2}')
  check_pass "Simulator booted: ${IOS_SIM_NAME:-Unknown} ($IOS_SIM_UDID)"
else
  check_warn "No iOS simulator booted"
fi
echo ""

# 4. Build artifacts freshness
echo "4. Build Artifacts:"
if [ -f "$ROOT_DIR/target/debug/scmessenger-cli" ]; then
  CLI_AGE=$(( $(date +%s) - $(stat -f %m "$ROOT_DIR/target/debug/scmessenger-cli" 2>/dev/null || stat -c %Y "$ROOT_DIR/target/debug/scmessenger-cli" 2>/dev/null || echo 0) ))
  if [ "$CLI_AGE" -lt 3600 ]; then
    check_pass "CLI binary: ${CLI_AGE}s old"
  else
    check_warn "CLI binary: ${CLI_AGE}s old (consider rebuilding)"
  fi
else
  check_warn "No pre-built CLI binary (will use cargo run)"
fi

if [ -d "$ROOT_DIR/iOS/SCMessengerCore.xcframework" ]; then
  XCF_AGE=$(( $(date +%s) - $(find "$ROOT_DIR/iOS/SCMessengerCore.xcframework" -type f -exec stat -f %m {} \; 2>/dev/null | sort -rn | head -1 || echo 0) ))
  if [ "$XCF_AGE" -lt 3600 ]; then
    check_pass "XCFramework: ${XCF_AGE}s old"
  else
    check_warn "XCFramework: ${XCF_AGE}s old (consider rebuilding)"
  fi
else
  check_warn "No XCFramework found"
fi
echo ""

# 5. GCP relay connectivity
echo "5. GCP Relay:"
GCP_IP="${GCP_RELAY_IP:-34.135.34.73}"
GCP_PORT="${GCP_RELAY_PORT:-9001}"
if nc -z -w 3 "$GCP_IP" "$GCP_PORT" 2>/dev/null; then
  check_pass "GCP relay reachable: $GCP_IP:$GCP_PORT"
else
  check_warn "GCP relay not reachable: $GCP_IP:$GCP_PORT"
fi
echo ""

# 6. Log directory
echo "6. Log Directory:"
LOG_ROOT="$ROOT_DIR/logs"
if [ -d "$LOG_ROOT" ]; then
  LOG_SIZE=$(du -sh "$LOG_ROOT" 2>/dev/null | awk '{print $1}')
  LOG_COUNT=$(find "$LOG_ROOT" -name "*.log" -o -name "*.txt" 2>/dev/null | wc -l | tr -d ' ')
  check_pass "Log directory: $LOG_SIZE, $LOG_COUNT files"
  
  # Warn if logs are consuming too much space
  LOG_SIZE_KB=$(du -sk "$LOG_ROOT" 2>/dev/null | awk '{print $1}')
  if [ "$LOG_SIZE_KB" -gt 1048576 ]; then
    check_warn "Log directory > 1GB — consider running prune_sim_logs.sh"
  fi
else
  check_warn "No log directory found"
fi
echo ""

# Summary
echo "══════════════════════════════════════════════════════════════"
echo "Summary: $PASS passed, $WARN warnings, $FAIL failures"
echo "══════════════════════════════════════════════════════════════"

if [ "$FAIL" -gt 0 ]; then
  echo ""
  echo -e "${RED}PRE-FLIGHT FAILED${NC} — resolve failures before proceeding"
  exit 1
fi

if [ "$WARN" -gt 0 ]; then
  echo ""
  echo -e "${YELLOW}PRE-FLIGHT PASSED WITH WARNINGS${NC} — review warnings"
  exit 0
fi

echo ""
echo -e "${GREEN}PRE-FLIGHT PASSED${NC} — all checks green"
exit 0
