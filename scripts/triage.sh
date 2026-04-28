#!/usr/bin/env bash
# scripts/triage.sh — Automated issue triage from log directory
#
# Usage:
#   ./scripts/triage.sh <log_directory>
#
# Analyzes log files and classifies issues by severity and component.
# Provides actionable recommendations for each issue category.
#
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
  exit 1
else
  echo "✅ TRIAGE COMPLETE: No critical issues detected"
  exit 0
fi
