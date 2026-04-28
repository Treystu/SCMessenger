#!/usr/bin/env bash
# scripts/debug-session.sh — Unified debug session orchestrator
#
# Usage:
#   ./scripts/debug-session.sh
#   STEP_ID=my-fix DEPLOY=0 DURATION_SEC=120 ./scripts/debug-session.sh
#
# Orchestrates the full debug lifecycle:
#   1. Pre-flight validation
#   2. Deploy (optional)
#   3. Log capture
#   4. Analysis
#   5. Triage (optional)
#
# Environment variables:
#   STEP_ID       — label for this debug session (default: adhoc)
#   DEPLOY        — deploy before capture (default: 1)
#   DURATION_SEC  — capture duration in seconds (default: 60)
#   AUTO_TRIAGE   — run triage after analysis (default: 1)
#   OUTPUT_ROOT   — output directory root (default: logs/debug-session)
#
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
printf "║  Step: %-54s ║\n" "$STEP_ID"
echo "╚══════════════════════════════════════════════════════════════╝"
echo "Session directory: $SESSION_DIR"
echo ""

# Phase 1: Pre-flight
echo "━━━ Phase 1: Pre-Flight ━━━"
if ! "$SCRIPT_DIR/preflight.sh" 2>&1 | tee "$SESSION_DIR/preflight.log"; then
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
  python3 "$SCRIPT_DIR/check_logs.py" "$SESSION_DIR/logs/" 2>&1 | tee "$SESSION_DIR/analysis.log" || true
else
  echo "No Android log found, skipping check_logs.py analysis"
fi
echo ""

# Phase 5: Triage (optional)
if [ "$AUTO_TRIAGE" = "1" ]; then
  echo "━━━ Phase 5: Triage ━━━"
  "$SCRIPT_DIR/triage.sh" "$SESSION_DIR/logs/" 2>&1 | tee "$SESSION_DIR/triage.log" || true
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
