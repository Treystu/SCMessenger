# Script Implementation Plan

Status: Active  
Last updated: 2026-03-16  
Purpose: Step-by-step implementation instructions for improving the deployment → log capture → analysis → iteration workflow.

---

## Prerequisites

Before executing any step:

1. Read `docs/SCRIPT_SANITY_CHECK_PROMPT.md` for full context
2. Read `docs/TESTING_GUIDE.md` for testing pyramid understanding
3. Verify you are on a feature branch (never edit on `main`)
4. Run `git status --short` to confirm clean working tree

---

## Phase 1: Pre-Flight Validation Improvements

### Step 1.1: Create Unified Pre-Flight Check Script

**File**: `scripts/preflight.sh`  
**Action**: create  
**Rationale**: No single script validates all prerequisites before a debug session. Scattered device detection logic leads to inconsistent error handling.  
**Implementation**:

```bash
#!/usr/bin/env bash
# scripts/preflight.sh — Unified pre-flight validation for SCMessenger debug sessions
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

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
  check_pass "Physical device: $IOS_DEVICE_NAME ($IOS_DEVICE_UDID)"
else
  check_warn "No physical iOS device connected"
fi

IOS_SIM_UDID=""
if xcrun simctl list devices 2>/dev/null | grep -q Booted; then
  IOS_SIM_UDID=$(xcrun simctl list devices | awk -F '[()]' '/Booted/{print $2; exit}')
  IOS_SIM_NAME=$(xcrun simctl list devices | grep "$IOS_SIM_UDID" | sed 's/^[[:space:]]*//' | awk '{print $1, $2}')
  check_pass "Simulator booted: $IOS_SIM_NAME ($IOS_SIM_UDID)"
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
```

**Verification**:

```bash
chmod +x scripts/preflight.sh
./scripts/preflight.sh
# Should show all checks with pass/warn/fail status
# Exit code 1 if any failures, 0 otherwise
```

**LOC Estimate**: ~130 LOC

---

### Step 1.2: Add Pre-Flight Gate to `run5-live-feedback.sh`

**File**: `scripts/run5-live-feedback.sh`  
**Action**: modify  
**Rationale**: The live verification loop should fail fast if prerequisites aren't met, rather than failing mid-session.  
**Implementation**: Add after line 111 (after the `phase()` function definition):

```bash
# Pre-flight gate
phase "Pre-Flight Validation"
if ! "$SCRIPT_DIR/preflight.sh"; then
  echo "Pre-flight checks failed. Aborting verification loop." >&2
  exit 1
fi
```

**Verification**:

```bash
./scripts/run5-live-feedback.sh --step=test --time=1
# Should run preflight.sh before any deployment
```

**LOC Estimate**: ~5 LOC

---

## Phase 2: Log Capture Standardization

### Step 2.1: Create Unified Log Capture Script

**File**: `scripts/capture_logs.sh`  
**Action**: create  
**Rationale**: Multiple log capture scripts (`capture_both_logs.sh`, `comprehensive_log_capture.sh`, `live-smoke.sh`) have overlapping but inconsistent logic. A single canonical script reduces maintenance burden.  
**Implementation**:

```bash
#!/usr/bin/env bash
# scripts/capture_logs.sh — Unified log capture for all SCMessenger nodes
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

TIMESTAMP="${TIMESTAMP:-$(date +%Y%m%d_%H%M%S)}"
LOGDIR="${LOGDIR:-$ROOT_DIR/logs/capture/$TIMESTAMP}"
DURATION_SEC="${DURATION_SEC:-60}"
ANDROID_SERIAL="${ANDROID_SERIAL:-}"
IOS_DEVICE_UDID="${IOS_DEVICE_UDID:-}"
IOS_SIM_UDID="${IOS_SIM_UDID:-}"
CAPTURE_ANDROID="${CAPTURE_ANDROID:-1}"
CAPTURE_IOS_DEVICE="${CAPTURE_IOS_DEVICE:-1}"
CAPTURE_IOS_SIM="${CAPTURE_IOS_SIM:-1}"

mkdir -p "$LOGDIR"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  SCMessenger Unified Log Capture                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo "Log directory: $LOGDIR"
echo "Duration:      ${DURATION_SEC}s"
echo ""

# PID tracking for cleanup
declare -a CAPTURE_PIDS=()

cleanup() {
  echo ""
  echo "Stopping log capture..."
  for pid in "${CAPTURE_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  done
  echo "Capture complete. Files in: $LOGDIR"
}
trap cleanup EXIT

# Android capture
if [ "$CAPTURE_ANDROID" = "1" ]; then
  if [ -z "$ANDROID_SERIAL" ]; then
    ANDROID_SERIAL=$(adb devices 2>/dev/null | awk 'NR>1 && $2=="device" {print $1; exit}')
  fi
  if [ -n "$ANDROID_SERIAL" ]; then
    echo "Starting Android log capture (serial: $ANDROID_SERIAL)..."
    adb -s "$ANDROID_SERIAL" logcat -c || true
    timeout "$DURATION_SEC" adb -s "$ANDROID_SERIAL" logcat -v time > "$LOGDIR/android.log" 2>&1 &
    CAPTURE_PIDS+=($!)
  else
    echo "Skipping Android: no device connected"
  fi
fi

# iOS Device capture
if [ "$CAPTURE_IOS_DEVICE" = "1" ]; then
  if [ -z "$IOS_DEVICE_UDID" ]; then
    IOS_DEVICE_UDID=$(xcrun devicectl list devices --hide-default-columns --columns Identifier --columns State --hide-headers 2>/dev/null | awk '$2 ~ /(available|connected)/ {print $1; exit}')
  fi
  if [ -n "$IOS_DEVICE_UDID" ]; then
    echo "Starting iOS device log capture (UDID: $IOS_DEVICE_UDID)..."
    timeout "$DURATION_SEC" xcrun devicectl device log show --device "$IOS_DEVICE_UDID" --predicate 'subsystem == "com.scmessenger"' --style compact > "$LOGDIR/ios-device.log" 2>&1 &
    CAPTURE_PIDS+=($!)
  else
    echo "Skipping iOS device: none connected"
  fi
fi

# iOS Simulator capture
if [ "$CAPTURE_IOS_SIM" = "1" ]; then
  if [ -z "$IOS_SIM_UDID" ]; then
    IOS_SIM_UDID=$(xcrun simctl list devices 2>/dev/null | awk -F '[()]' '/Booted/{print $2; exit}')
  fi
  if [ -n "$IOS_SIM_UDID" ]; then
    echo "Starting iOS simulator log capture (UDID: $IOS_SIM_UDID)..."
    timeout "$DURATION_SEC" xcrun simctl spawn "$IOS_SIM_UDID" log stream --predicate 'subsystem == "com.scmessenger" OR process == "SCMessenger"' --style compact > "$LOGDIR/ios-sim.log" 2>&1 &
    CAPTURE_PIDS+=($!)
  else
    echo "Skipping iOS simulator: none booted"
  fi
fi

echo ""
echo "Capturing for ${DURATION_SEC}s..."
echo "Perform your test actions now."
echo ""

# Wait for all captures
for pid in "${CAPTURE_PIDS[@]}"; do
  wait "$pid" 2>/dev/null || true
done

# Clear trap since we're exiting normally
trap - EXIT

# Quick analysis
echo ""
echo "Capture Summary:"
for logfile in "$LOGDIR"/*.log; do
  [ -f "$logfile" ] || continue
  lines=$(wc -l < "$logfile" | tr -d ' ')
  errors=$(grep -ciE "error|exception|crash|failed" "$logfile" 2>/dev/null || echo 0)
  echo "  $(basename "$logfile"): $lines lines, $errors errors"
done

echo ""
echo "Log files saved to: $LOGDIR"
```

**Verification**:

```bash
chmod +x scripts/capture_logs.sh
DURATION_SEC=10 ./scripts/capture_logs.sh
# Should capture logs from all connected devices for 10 seconds
# Should produce summary with line counts and error counts
```

**LOC Estimate**: ~110 LOC

---

### Step 2.2: Add Structured Log Markers

**File**: `scripts/capture_logs.sh`  
**Action**: modify  
**Rationale**: Inject sync markers into all device logs to enable cross-platform correlation.  
**Implementation**: Add before capture starts:

```bash
# Inject sync marker for cross-platform correlation
SYNC_MARKER="=== CAPTURE_START: $TIMESTAMP ==="
if [ -n "$ANDROID_SERIAL" ]; then
  adb -s "$ANDROID_SERIAL" logcat -b main -d | head -1 > /dev/null  # Force logcat buffer
  echo "$SYNC_MARKER" | adb -s "$ANDROID_SERIAL" logcat -b main -T 1 2>/dev/null || true
fi
```

**Verification**: Check that log files contain the sync marker near the beginning  
**LOC Estimate**: ~5 LOC

---

## Phase 3: Analysis Improvements

### Step 3.1: Enhance `check_logs.py` with Severity Classification

**File**: `scripts/check_logs.py`  
**Action**: modify  
**Rationale**: The current script shows raw counts but doesn't classify issues by severity or affected component.  
**Implementation**: Add after line 134 (before the visibility matrix section):

```python
# === Severity Classification ===
print()
print("  Issue Severity Classification:")
print("  " + "─" * 60)

CRASH_PAT = re.compile(r'(panic|fatal|stack.?overflow|SIGSEGV|SIGABRT)', re.I)
CRITICAL_PAT = re.compile(r'(NoClassDefFound|OutOfMemory|FATAL|core dumped)', re.I)
ERROR_PAT_CLASS = re.compile(r'(error|exception|failed|fail)', re.I)
WARN_PAT_CLASS = re.compile(r'(warning|warn|deprecated)', re.I)

for name, content in contents.items():
    if not content:
        continue
    crashes = len(CRASH_PAT.findall(content))
    critical = len(CRITICAL_PAT.findall(content))
    errors = len(ERROR_PAT_CLASS.findall(content))
    warnings = len(WARN_PAT_CLASS.findall(content))

    icon = '🔴' if crashes > 0 else ('🟠' if critical > 0 else ('🟡' if errors > 5 else '🟢'))
    print(f"  {icon} {name:<10} crashes={crashes} critical={critical} errors={errors} warnings={warnings}")

# Component breakdown
print()
print("  Component Error Breakdown:")
print("  " + "─" * 60)
COMPONENTS = {
    'BLE': re.compile(r'BLE|Gatt|L2CAP|bluetooth', re.I),
    'Transport': re.compile(r'transport|dial|connect|relay|swarm', re.I),
    'Delivery': re.compile(r'deliver|receipt|send_msg|message', re.I),
    'Identity': re.compile(r'identity|peer.?id|12D3KooW', re.I),
    'Storage': re.compile(r'sled|database|store|persist', re.I),
    'UI': re.compile(r'ViewModel|Fragment|Activity|SwiftUI|View', re.I),
}

for name, content in contents.items():
    if not content:
        continue
    print(f"  {name}:")
    for comp_name, comp_pat in COMPONENTS.items():
        comp_errors = len(comp_pat.findall(content))
        if comp_errors > 0:
            print(f"    {comp_name}: {comp_errors} events")
```

**Verification**:

```bash
python3 scripts/check_logs.py scripts/logs_20260310_020758/
# Should show severity classification and component breakdown
```

**LOC Estimate**: ~35 LOC

---

### Step 3.2: Create Automated Triage Script

**File**: `scripts/triage.sh`  
**Action**: create  
**Rationale**: No script automatically classifies issues and suggests next actions. This reduces manual analysis time.  
**Implementation**:

```bash
#!/usr/bin/env bash
# scripts/triage.sh — Automated issue triage from log directory
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <log_directory>" >&2
  exit 2
fi

LOGDIR="$1"
if [ ! -d "$LOGDIR" ]; then
  echo "Error: directory not found: $LOGDIR" >&2
  exit 2
fi

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  SCMessenger Automated Triage                               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo "Log directory: $LOGDIR"
echo ""

ISSUES_FOUND=0

# 1. Check for crashes
echo "1. Crash Detection:"
for logfile in "$LOGDIR"/*.log "$LOGDIR"/*.txt; do
  [ -f "$logfile" ] || continue
  crashes=$(grep -ciE "panic|fatal|SIGSEGV|SIGABRT|stack.?overflow" "$logfile" 2>/dev/null || echo 0)
  if [ "$crashes" -gt 0 ]; then
    ((ISSUES_FOUND++))
    echo "  🔴 CRASH in $(basename "$logfile"): $crashes crash markers"
    echo "     Action: Review crash stack traces, check for stale builds"
  fi
done
if [ "$ISSUES_FOUND" -eq 0 ]; then
  echo "  ✓ No crashes detected"
fi
echo ""

# 2. Check for connection failures
echo "2. Connection Health:"
CONN_ISSUES=0
for logfile in "$LOGDIR"/*.log "$LOGDIR"/*.txt; do
  [ -f "$logfile" ] || continue
  conn_fails=$(grep -ciE "connection.?failed|dial.?failed|failed.?to.?connect" "$logfile" 2>/dev/null || echo 0)
  if [ "$conn_fails" -gt 10 ]; then
    ((CONN_ISSUES++))
    echo "  🟠 HIGH FAILURE RATE in $(basename "$logfile"): $conn_fails connection failures"
    echo "     Action: Check network connectivity, verify GCP relay status"
  fi
done
if [ "$CONN_ISSUES" -eq 0 ]; then
  echo "  ✓ Connection failure rates acceptable"
fi
echo ""

# 3. Check for BLE issues
echo "3. BLE Transport:"
BLE_ISSUES=0
for logfile in "$LOGDIR"/*.log "$LOGDIR"/*.txt; do
  [ -f "$logfile" ] || continue
  ble_errors=$(grep -ciE "BLE.?error|BLE.?fail|bluetooth.?unavailable" "$logfile" 2>/dev/null || echo 0)
  zero_adv=$(grep -ciE "No BLE Fast|No BLE.*advertisement" "$logfile" 2>/dev/null || echo 0)
  if [ "$ble_errors" -gt 5 ] || [ "$zero_adv" -gt 3 ]; then
    ((BLE_ISSUES++))
    echo "  🟠 BLE ISSUES in $(basename "$logfile"): $ble_errors errors, $zero_adv zero-adv windows"
    echo "     Action: Check Bluetooth permissions, verify BLE hardware"
  fi
done
if [ "$BLE_ISSUES" -eq 0 ]; then
  echo "  ✓ BLE transport healthy"
fi
echo ""

# 4. Check for delivery issues
echo "4. Message Delivery:"
DELIVERY_ISSUES=0
for logfile in "$LOGDIR"/*.log "$LOGDIR"/*.txt; do
  [ -f "$logfile" ] || continue
  send_count=$(grep -ciE "send_msg|delivery_attempt" "$logfile" 2>/dev/null || echo 0)
  receipt_count=$(grep -ciE "delivery_receipt|receipt.*received" "$logfile" 2>/dev/null || echo 0)
  if [ "$send_count" -gt 0 ] && [ "$receipt_count" -eq 0 ]; then
    ((DELIVERY_ISSUES++))
    echo "  🟠 DELIVERY GAP in $(basename "$logfile"): $send_count sends, $receipt_count receipts"
    echo "     Action: Run verify_receipt_convergence.sh, check transport connectivity"
  fi
done
if [ "$DELIVERY_ISSUES" -eq 0 ]; then
  echo "  ✓ No delivery gaps detected (or no messages sent)"
fi
echo ""

# 5. Check for identity issues
echo "5. Identity Resolution:"
IDENTITY_ISSUES=0
for logfile in "$LOGDIR"/*.log "$LOGDIR"/*.txt; do
  [ -f "$logfile" ] || continue
  id_errors=$(grep -ciE "identity.*error|peer.*id.*mismatch|id.*conflict" "$logfile" 2>/dev/null || echo 0)
  if [ "$id_errors" -gt 0 ]; then
    ((IDENTITY_ISSUES++))
    echo "  🟠 IDENTITY ISSUES in $(basename "$logfile"): $id_errors identity errors"
    echo "     Action: Check identity persistence, verify no stale identities"
  fi
done
if [ "$IDENTITY_ISSUES" -eq 0 ]; then
  echo "  ✓ No identity issues detected"
fi
echo ""

# Summary
echo "══════════════════════════════════════════════════════════════"
TOTAL=$((ISSUES_FOUND + CONN_ISSUES + BLE_ISSUES + DELIVERY_ISSUES + IDENTITY_ISSUES))
if [ "$TOTAL" -gt 0 ]; then
  echo "🔴 TRIAGE COMPLETE: $TOTAL issue categories detected"
  echo ""
  echo "Recommended next steps:"
  [ "$ISSUES_FOUND" -gt 0 ] && echo "  1. Address crashes first (check build freshness)"
  [ "$CONN_ISSUES" -gt 0 ] && echo "  2. Verify network/relay connectivity"
  [ "$BLE_ISSUES" -gt 0 ] && echo "  3. Run BLE-specific verification: ./scripts/verify_ble_only_pairing.sh"
  [ "$DELIVERY_ISSUES" -gt 0 ] && echo "  4. Run delivery verification: ./scripts/verify_receipt_convergence.sh"
  [ "$IDENTITY_ISSUES" -gt 0 ] && echo "  5. Check identity persistence across restarts"
else
  echo "✅ TRIAGE COMPLETE: No critical issues detected"
fi
echo "══════════════════════════════════════════════════════════════"
```

**Verification**:

```bash
chmod +x scripts/triage.sh
./scripts/triage.sh scripts/logs_20260310_020758/
# Should produce categorized issue report with recommended actions
```

**LOC Estimate**: ~120 LOC

---

## Phase 4: Workflow Integration

### Step 4.1: Create Unified Debug Session Script

**File**: `scripts/debug-session.sh`  
**Action**: create  
**Rationale**: No single entry point orchestrates the full debug lifecycle. This script ties together preflight, deploy, capture, analysis, and triage.  
**Implementation**:

```bash
#!/usr/bin/env bash
# scripts/debug-session.sh — Unified debug session orchestrator
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

STEP_ID="${STEP_ID:-adhoc}"
DEPLOY="${DEPLOY:-1}"
DURATION_SEC="${DURATION_SEC:-60}"
AUTO_TRIAGE="${AUTO_TRIAGE:-1}"
OUTPUT_ROOT="${OUTPUT_ROOT:-$ROOT_DIR/logs/debug-session}"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SESSION_DIR="$OUTPUT_ROOT/$STEP_ID/$TIMESTAMP"
mkdir -p "$SESSION_DIR"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  SCMessenger Debug Session                                  ║"
echo "║  Step: $STEP_ID                                             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo "Session directory: $SESSION_DIR"
echo ""

# Phase 1: Pre-flight
echo "━━━ Phase 1: Pre-Flight ━━━"
if ! "$SCRIPT_DIR/preflight.sh" | tee "$SESSION_DIR/preflight.log"; then
  echo "Pre-flight failed. Aborting." >&2
  exit 1
fi
echo ""

# Phase 2: Deploy (optional)
if [ "$DEPLOY" = "1" ]; then
  echo "━━━ Phase 2: Deploy ━━━"
  "$SCRIPT_DIR/deploy_to_device.sh" both 2>&1 | tee "$SESSION_DIR/deploy.log" || {
    echo "Deploy failed. Continuing with existing builds..." >&2
  }
  echo ""
fi

# Phase 3: Capture
echo "━━━ Phase 3: Log Capture ━━━"
LOGDIR="$SESSION_DIR/logs" DURATION_SEC="$DURATION_SEC" "$SCRIPT_DIR/capture_logs.sh" 2>&1 | tee "$SESSION_DIR/capture.log"
echo ""

# Phase 4: Analysis
echo "━━━ Phase 4: Analysis ━━━"
if [ -f "$SESSION_DIR/logs/android.log" ]; then
  python3 "$SCRIPT_DIR/check_logs.py" "$SESSION_DIR/logs/" 2>&1 | tee "$SESSION_DIR/analysis.log"
fi
echo ""

# Phase 5: Triage (optional)
if [ "$AUTO_TRIAGE" = "1" ]; then
  echo "━━━ Phase 5: Triage ━━━"
  "$SCRIPT_DIR/triage.sh" "$SESSION_DIR/logs/" 2>&1 | tee "$SESSION_DIR/triage.log"
  echo ""
fi

# Summary
echo "══════════════════════════════════════════════════════════════"
echo "Debug session complete."
echo "All artifacts saved to: $SESSION_DIR"
echo "══════════════════════════════════════════════════════════════"
echo ""
echo "Files:"
ls -la "$SESSION_DIR/"
echo ""
echo "Next steps:"
echo "  - Review analysis.log for peer visibility matrix"
echo "  - Review triage.log for automated issue classification"
echo "  - Run specific verifiers if issues detected:"
echo "    ./scripts/verify_relay_flap_regression.sh $SESSION_DIR/logs/ios-device.log"
echo "    ./scripts/verify_ble_only_pairing.sh $SESSION_DIR/logs/android.log $SESSION_DIR/logs/ios-device.log"
```

**Verification**:

```bash
chmod +x scripts/debug-session.sh
STEP_ID=test-run DEPLOY=0 DURATION_SEC=15 ./scripts/debug-session.sh
# Should run full lifecycle: preflight → capture → analysis → triage
# Should produce organized output in logs/debug-session/test-run/<timestamp>/
```

**LOC Estimate**: ~80 LOC

---

## Phase 5: Cleanup and Maintenance

### Step 5.1: Enhance `prune_sim_logs.sh` with Safety Checks

**File**: `scripts/prune_sim_logs.sh`  
**Action**: modify  
**Rationale**: The current pruning script may delete active log files. Add safety checks to preserve recent logs.  
**Implementation**: Add safety check at the beginning:

```bash
# Safety: preserve logs from the last 24 hours
KEEP_HOURS="${KEEP_HOURS:-24}"
echo "Pruning logs older than ${KEEP_HOURS} hours..."

# Never delete the 'latest' symlink or its target
LATEST_LINK="$ROOT_DIR/logs/5mesh/latest"
if [ -L "$LATEST_LINK" ]; then
  LATEST_TARGET=$(readlink "$LATEST_LINK")
  echo "Preserving latest run: $LATEST_TARGET"
fi
```

**Verification**: Run with `--dry-run` flag to see what would be deleted before actual deletion  
**LOC Estimate**: ~15 LOC

---

### Step 5.2: Add `--dry-run` Mode to All Scripts

**File**: Multiple scripts  
**Action**: modify  
**Rationale**: Scripts that modify state (deploy, prune, install) should support `--dry-run` for safe testing.  
**Implementation Pattern** (apply to each script):

```bash
DRY_RUN="${DRY_RUN:-0}"

dry_run() {
  if [ "$DRY_RUN" = "1" ]; then
    echo "[DRY-RUN] Would execute: $*"
    return 0
  fi
  "$@"
}

# Replace direct command calls with:
# dry_run command args...
```

**Verification**: Run each modified script with `DRY_RUN=1` and verify no state changes occur  
**LOC Estimate**: ~10 LOC per script, ~50 LOC total across 5 scripts

---

## Execution Order

Execute steps in this order to respect dependencies:

1. **Step 1.1**: Create `preflight.sh` (no dependencies)
2. **Step 1.2**: Add pre-flight gate to `run5-live-feedback.sh` (depends on 1.1)
3. **Step 2.1**: Create `capture_logs.sh` (no dependencies)
4. **Step 2.2**: Add sync markers (depends on 2.1)
5. **Step 3.1**: Enhance `check_logs.py` (no dependencies)
6. **Step 3.2**: Create `triage.sh` (no dependencies)
7. **Step 4.1**: Create `debug-session.sh` (depends on 1.1, 2.1, 3.2)
8. **Step 5.1**: Enhance `prune_sim_logs.sh` (no dependencies)
9. **Step 5.2**: Add `--dry-run` modes (no dependencies, can be parallelized)

---

## Verification Checklist

After all steps are complete:

- [ ] `./scripts/preflight.sh` passes on a clean machine
- [ ] `./scripts/debug-session.sh --step=verify` runs full lifecycle
- [ ] `python3 scripts/check_logs.py <logdir>` shows severity classification
- [ ] `./scripts/triage.sh <logdir>` produces actionable triage report
- [ ] All modified scripts have `set -euo pipefail`
- [ ] All new scripts are executable (`chmod +x`)
- [ ] `./scripts/docs_sync_check.sh` passes
- [ ] No scripts left with hardcoded paths (use `$ROOT_DIR` or `$SCRIPT_DIR`)

---

## Total LOC Estimate

| Category    | New Files             | Modified Files            | Total LOC    |
| ----------- | --------------------- | ------------------------- | ------------ |
| Pre-flight  | 130 (preflight.sh)    | 5 (run5-live-feedback.sh) | 135          |
| Log capture | 110 (capture_logs.sh) | 5 (sync markers)          | 115          |
| Analysis    | 35 (check_logs.py)    | 120 (triage.sh)           | 155          |
| Integration | 80 (debug-session.sh) | —                         | 80           |
| Cleanup     | —                     | 65 (prune + dry-run)      | 65           |
| **Total**   | **455**               | **95**                    | **~550 LOC** |
