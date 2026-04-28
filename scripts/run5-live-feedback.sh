#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

RUN5_SCRIPT="${RUN5_SCRIPT:-$ROOT_DIR/run5.sh}"
DEPLOY_SCRIPT="$ROOT_DIR/scripts/deploy_to_device.sh"

STEP_ID="adhoc"
TIME_MIN=5
MAX_ATTEMPTS=3
UPDATE_EACH=1
DEPLOY_MOBILE=1
TAIL_LINES=60
MIN_LOG_LINES=20
DIAG_TAIL_LINES=6000
IOS_DIAG_PULL_ATTEMPTS=3
IOS_DIAG_STABILITY_MAX_DELTA_BYTES=256
REQUIRE_RECEIPT_GATE=0
ANDROID_DIAG_SOURCE=""
IOS_DIAG_SOURCE=""
OUTPUT_ROOT="${OUTPUT_ROOT:-$ROOT_DIR/logs/live-verify}"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Live 5-node verification loop with strict phase gating.

Options:
  --step=<id>                 Label for the fix step being validated (default: adhoc)
  --time=<minutes>            Minutes to run each run5 capture window (default: 5)
  --attempts=<n>              Max attempts before final fail (default: 3)
  --tail-lines=<n>            Tail lines per node log in snapshot (default: 60)
  --min-log-lines=<n>         Minimum lines required per node log (default: 20)
  --diag-tail-lines=<n>       Tail lines to keep from diagnostics before verifiers (default: 6000)
  --skip-update               Do not pass --update to run5.sh
  --skip-mobile-deploy        Skip Android+iOS deploy script before each attempt
  --require-receipt-gate      Treat receipt convergence verifier as required (default: warn-only)
  --android-diag=<path>       Use this Android diagnostics log for receipt gate
  --ios-diag=<path>           Use this iOS diagnostics log for receipt gate
  --run5-script=<path>        Override run5 script path (default: ./run5.sh)
  --output-root=<path>        Override output root (default: logs/live-verify)
  -h, --help                  Show this help text

Environment overrides:
  OUTPUT_ROOT, RUN5_SCRIPT
  IOS_DIAG_PULL_ATTEMPTS (default: 3) - Number of iOS diagnostics pull attempts
  IOS_DIAG_STABILITY_MAX_DELTA_BYTES (default: 256) - Max size delta for stable capture
  IOS_DIAG_COPY_TIMEOUT (default: 60) - Timeout in seconds for individual copy attempts
USAGE
}

for arg in "$@"; do
  case "$arg" in
    --step=*) STEP_ID="${arg#*=}" ;;
    --time=*) TIME_MIN="${arg#*=}" ;;
    --attempts=*) MAX_ATTEMPTS="${arg#*=}" ;;
    --tail-lines=*) TAIL_LINES="${arg#*=}" ;;
    --min-log-lines=*) MIN_LOG_LINES="${arg#*=}" ;;
    --diag-tail-lines=*) DIAG_TAIL_LINES="${arg#*=}" ;;
    --skip-update) UPDATE_EACH=0 ;;
    --skip-mobile-deploy) DEPLOY_MOBILE=0 ;;
    --require-receipt-gate) REQUIRE_RECEIPT_GATE=1 ;;
    --android-diag=*) ANDROID_DIAG_SOURCE="${arg#*=}" ;;
    --ios-diag=*) IOS_DIAG_SOURCE="${arg#*=}" ;;
    --run5-script=*) RUN5_SCRIPT="${arg#*=}" ;;
    --output-root=*) OUTPUT_ROOT="${arg#*=}" ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "Unknown argument: $arg" >&2
      usage
      exit 2
      ;;
  esac
done

if ! [[ "$TIME_MIN" =~ ^[0-9]+$ ]] || [ "$TIME_MIN" -le 0 ]; then
  echo "--time must be a positive integer" >&2
  exit 2
fi
if ! [[ "$MAX_ATTEMPTS" =~ ^[0-9]+$ ]] || [ "$MAX_ATTEMPTS" -le 0 ]; then
  echo "--attempts must be a positive integer" >&2
  exit 2
fi
if ! [[ "$TAIL_LINES" =~ ^[0-9]+$ ]] || [ "$TAIL_LINES" -le 0 ]; then
  echo "--tail-lines must be a positive integer" >&2
  exit 2
fi
if ! [[ "$MIN_LOG_LINES" =~ ^[0-9]+$ ]] || [ "$MIN_LOG_LINES" -le 0 ]; then
  echo "--min-log-lines must be a positive integer" >&2
  exit 2
fi
if ! [[ "$DIAG_TAIL_LINES" =~ ^[0-9]+$ ]] || [ "$DIAG_TAIL_LINES" -le 0 ]; then
  echo "--diag-tail-lines must be a positive integer" >&2
  exit 2
fi

sanitize_step_id() {
  local raw="$1"
  local clean
  clean="$(echo "$raw" | tr ' /:' '___' | tr -cd 'A-Za-z0-9._-')"
  if [ -z "$clean" ]; then
    clean="adhoc"
  fi
  echo "$clean"
}

STEP_ID="$(sanitize_step_id "$STEP_ID")"

log() {
  printf '[%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*"
}

phase() {
  echo
  echo "================================================================"
  log "PHASE: $*"
  echo "================================================================"
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    return 1
  fi
}

absolute_path() {
  local p="$1"
  if [ -d "$p" ]; then
    (cd "$p" && pwd)
  else
    local dir
    dir="$(cd "$(dirname "$p")" && pwd)"
    echo "$dir/$(basename "$p")"
  fi
}

resolve_latest_mesh_logdir() {
  local latest_link="$ROOT_DIR/logs/5mesh/latest"
  if [ ! -L "$latest_link" ]; then
    return 1
  fi
  local target
  target="$(readlink "$latest_link")"
  if [ -z "$target" ]; then
    return 1
  fi
  if [[ "$target" = /* ]]; then
    echo "$target"
  else
    echo "$ROOT_DIR/logs/5mesh/$target"
  fi
}

collect_android_diagnostics() {
  local out_file="$1"
  local stderr_file="$2"
  : > "$stderr_file"

  if ! command -v adb >/dev/null 2>&1; then
    return 1
  fi

  local serial
  serial="$(adb devices | awk 'NR>1 && $2=="device" {print $1; exit}')"
  if [ -z "$serial" ]; then
    echo "No connected Android device for diagnostics pull" >> "$stderr_file"
    return 1
  fi

  if adb -s "$serial" shell run-as com.scmessenger.android cat files/mesh_diagnostics.log > "$out_file" 2>> "$stderr_file"; then
    if [ -s "$out_file" ]; then
      return 0
    fi
  fi

  # Release builds may not allow run-as. Fall back to filtered logcat
  # delivery markers so convergence/ordering verifiers still have signal.
  local raw_logcat="$out_file.raw.logcat"
  if adb -s "$serial" logcat -d -v time > "$raw_logcat" 2>> "$stderr_file"; then
    grep -Ei "delivery_state|delivery_attempt|Receipt for|msg_rx|msg_rx_processed|delivery_receipt" "$raw_logcat" > "$out_file" || true
    rm -f "$raw_logcat"
    if [ -s "$out_file" ]; then
      return 0
    fi
  else
    rm -f "$raw_logcat"
  fi

  echo "Android diagnostics pull failed for serial $serial" >> "$stderr_file"
  return 1
}

collect_ios_diagnostics() {
  local out_file="$1"
  local stderr_file="$2"
  : > "$stderr_file"

  if ! command -v xcrun >/dev/null 2>&1; then
    return 1
  fi

  local devices_json
  devices_json="$(mktemp)"
  if ! xcrun devicectl list devices --json-output "$devices_json" >/dev/null 2>>"$stderr_file"; then
    rm -f "$devices_json"
    echo "Unable to list iOS devices via devicectl" >> "$stderr_file"
    return 1
  fi

  local device_id
  device_id="$(python3 - "$devices_json" <<'PY'
import json
import sys

path = sys.argv[1]
try:
    with open(path, "r", encoding="utf-8") as f:
        payload = json.load(f)
except Exception:
    print("")
    raise SystemExit(0)

devices = payload.get("result", {}).get("devices", [])
for d in devices:
    identifier = d.get("identifier") or ""
    if not identifier:
        continue
    pairing_state = (d.get("connectionProperties", {}).get("pairingState") or "").lower()
    if pairing_state == "paired":
        print(identifier)
        raise SystemExit(0)
print("")
PY
)"
  rm -f "$devices_json"

  if [ -z "$device_id" ]; then
    echo "No paired iOS device found for diagnostics pull" >> "$stderr_file"
    return 1
  fi

  rm -f "$out_file"
  local previous_size=0
  local stable_size=0
  local attempts="${IOS_DIAG_PULL_ATTEMPTS:-3}"
  local max_delta="${IOS_DIAG_STABILITY_MAX_DELTA_BYTES:-256}"
  if ! [[ "$attempts" =~ ^[0-9]+$ ]] || [ "$attempts" -le 0 ]; then
    echo "Invalid IOS_DIAG_PULL_ATTEMPTS='$attempts'; defaulting to one-shot mode (1)" >> "$stderr_file"
    attempts=1
  fi

  for attempt in $(seq 1 "$attempts"); do
    local attempt_file="${out_file}.attempt${attempt}"
    rm -f "$attempt_file"

    # Add timeout to prevent hanging on large file transfers (IOS-DIAG-001 fix)
    if timeout ${IOS_DIAG_COPY_TIMEOUT:-60} xcrun devicectl device copy from \
        --device "$device_id" \
        --domain-type appDataContainer \
        --domain-identifier SovereignCommunications.SCMessenger \
        --source Documents/mesh_diagnostics.log \
        --destination "$attempt_file" >>"$stderr_file" 2>&1; then
      :
    else
      echo "iOS diagnostics copy timed out after ${IOS_DIAG_COPY_TIMEOUT:-60} seconds" >> "$stderr_file"
    fi

    if [ ! -s "$attempt_file" ]; then
      echo "iOS diagnostics pull attempt ${attempt}/${attempts} produced no bytes" >> "$stderr_file"
      continue
    fi

    local current_size
    current_size="$(wc -c < "$attempt_file" | tr -d ' ')"
    echo "iOS diagnostics pull attempt ${attempt}/${attempts} size=${current_size}" >> "$stderr_file"

    if [ "$attempts" -eq 1 ]; then
      mv "$attempt_file" "$out_file"
      rm -f "${out_file}.attempt"*
      echo "iOS diagnostics pull accepted in one-shot mode (attempts=1)" >> "$stderr_file"
      return 0
    fi

    if [ "$previous_size" -gt 0 ]; then
      local delta
      delta=$(( current_size > previous_size ? current_size - previous_size : previous_size - current_size ))
      if [ "$delta" -le "$max_delta" ]; then
        stable_size="$current_size"
        mv "$attempt_file" "$out_file"
        rm -f "${out_file}.attempt"*
        echo "iOS diagnostics pull stabilized across retries (size delta=${delta} bytes)" >> "$stderr_file"
        return 0
      fi
      echo "iOS diagnostics pull attempt ${attempt} size delta=${delta} bytes (waiting for stable capture)" >> "$stderr_file"
    fi

    previous_size="$current_size"
    mv "$attempt_file" "$out_file"
    sleep 1
  done

  # Fail-fast if we could not get two near-identical pulls to avoid treating truncated copies as valid.
  if [ "$stable_size" -eq 0 ]; then
    if [ -s "$out_file" ]; then
      echo "iOS diagnostics pull could not confirm non-truncated stability after ${attempts} attempts" >> "$stderr_file"
    else
      echo "iOS diagnostics pull failed for device $device_id" >> "$stderr_file"
    fi
  fi
  # Ensure callers do not consume an untrusted/truncated capture on failed stability.
  rm -f "$out_file"
  rm -f "${out_file}.attempt"*
  return 1
}

tail_diagnostics_window() {
  local input_file="$1"
  local output_file="$2"
  if [ ! -f "$input_file" ]; then
    return 1
  fi
  tail -n "$DIAG_TAIL_LINES" "$input_file" > "$output_file"
  return 0
}

pair_matrix_gate() {
  local logdir="$1"
  local out_file="$2"

  python3 - "$logdir" > "$out_file" <<'PY'
import os
import re
import sys

logdir = sys.argv[1]
logs = {
    "gcp": os.path.join(logdir, "gcp.log"),
    "osx": os.path.join(logdir, "osx.log"),
    "android": os.path.join(logdir, "android.log"),
    "ios_dev": os.path.join(logdir, "ios-device.log"),
    "ios_sim": os.path.join(logdir, "ios-sim.log"),
}
node_order = ["gcp", "osx", "android", "ios_dev", "ios_sim"]

peer_pat = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")
own_patterns = [
    re.compile(r"local_peer_id\s*=\s*(12D3KooW[a-zA-Z0-9]{44,})"),
    re.compile(r"Starting Swarm with PeerID:\s*(12D3KooW[a-zA-Z0-9]{44,})"),
    re.compile(r"SwarmBridge with peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})"),
    re.compile(r"agent:\s*scmessenger/[^/]+/headless/relay/(12D3KooW[a-zA-Z0-9]{44,})"),
    re.compile(r"local peer id:?\s*(12D3KooW[a-zA-Z0-9]{44,})", re.I),
]


def read_text(path):
    try:
        with open(path, "r", errors="ignore") as f:
            return f.read()
    except Exception:
        return ""


def strip_ansi(s):
    return re.sub(r"\x1b\[[0-9;]*m", "", s)


contents = {name: strip_ansi(read_text(path)) for name, path in logs.items()}
own_ids = {}

for name in node_order:
    text = contents[name]
    found = None
    for pat in own_patterns:
        m = pat.search(text)
        if m:
            found = m.group(1)
            break
    if not found:
        # Fallback: most frequent peer ID in that node log
        counts = {}
        for pid in peer_pat.findall(text):
            counts[pid] = counts.get(pid, 0) + 1
        if counts:
            found = max(counts, key=counts.get)
    own_ids[name] = found

seen_ids = {name: set(peer_pat.findall(contents[name])) for name in node_order}

missing_ids = [name for name in node_order if not own_ids[name]]
if missing_ids:
    print("FAIL: could not derive own peer IDs for node(s): " + ", ".join(missing_ids))
    sys.exit(1)

missing_pairs = []
for src in node_order:
    for dst in node_order:
        if src == dst:
            continue
        if own_ids[dst] not in seen_ids[src]:
            missing_pairs.append((src, dst))

print("pair_matrix_summary:")
print(f"  logdir: {logdir}")
print("  own_ids:")
for name in node_order:
    print(f"    - {name}: {own_ids[name]}")

print("  directed_visibility:")
for src in node_order:
    missing_for_src = [dst for s, dst in missing_pairs if s == src]
    seen = (len(node_order) - 1) - len(missing_for_src)
    status = "PASS" if not missing_for_src else "FAIL"
    miss = ", ".join(missing_for_src) if missing_for_src else "none"
    print(f"    - {src}: {status} {seen}/{len(node_order)-1} missing={miss}")

if missing_pairs:
    print("FAIL: missing directed node pair visibility edges:")
    for src, dst in missing_pairs:
        print(f"  - {src} -> {dst}")
    sys.exit(1)

print("PASS: full directed visibility matrix achieved (20/20 edges)")
sys.exit(0)
PY
}

error_scan_gate() {
  local logdir="$1"
  local out_file="$2"

  local ios_hits android_hits
  ios_hits=$(grep -Eci "SIGTRAP|EXC_BREAKPOINT|cpu_resource_fatal|fatal error|assertion failed" "$logdir/ios-device.log" 2>/dev/null || true)
  android_hits=$(grep -Eci "FATAL EXCEPTION|java\.lang\.NullPointerException|java\.lang\.IllegalStateException" "$logdir/android.log" 2>/dev/null || true)

  ios_hits=$(echo "${ios_hits:-0}" | awk -F: '{sum += $NF} END {print sum + 0}')
  android_hits=$(echo "${android_hits:-0}" | awk -F: '{sum += $NF} END {print sum + 0}')

  {
    echo "error_scan_summary:"
    echo "  ios_crash_markers: $ios_hits"
    echo "  android_fatal_markers: $android_hits"
  } > "$out_file"

  if [ "$ios_hits" -gt 0 ] || [ "$android_hits" -gt 0 ]; then
    return 1
  fi
  return 0
}

log_health_gate() {
  local logdir="$1"
  local out_file="$2"

  local required=("gcp.log" "osx.log" "android.log" "ios-device.log" "ios-sim.log")
  local fail=0

  {
    echo "log_health_summary:"
    echo "  logdir: $logdir"
    echo "  min_log_lines: $MIN_LOG_LINES"
  } > "$out_file"

  for f in "${required[@]}"; do
    local p="$logdir/$f"
    if [ ! -f "$p" ]; then
      echo "  - $f: MISSING" >> "$out_file"
      fail=1
      continue
    fi

    local lines
    lines="$(wc -l < "$p" | tr -d ' ')"
    echo "  - $f: ${lines} lines" >> "$out_file"
    if [ "$lines" -lt "$MIN_LOG_LINES" ]; then
      fail=1
    fi
  done

  [ "$fail" -eq 0 ]
}

tail_snapshot() {
  local logdir="$1"
  local out_file="$2"

  {
    echo "tail_snapshot:"
    for name in gcp.log osx.log android.log ios-device.log ios-sim.log; do
      echo ""
      echo "===== $name (last $TAIL_LINES lines) ====="
      if [ -f "$logdir/$name" ]; then
        tail -n "$TAIL_LINES" "$logdir/$name"
      else
        echo "missing"
      fi
    done
  } > "$out_file"
}

run_verifier() {
  local label="$1"
  local out_file="$2"
  shift 2

  {
    echo "verifier: $label"
    echo "command: $*"
    echo ""
    "$@"
  } > "$out_file" 2>&1
}

phase "Preflight"
require_cmd bash
require_cmd python3
require_cmd awk
require_cmd sed

if [ ! -x "$RUN5_SCRIPT" ]; then
  echo "run5 script not executable: $RUN5_SCRIPT" >&2
  exit 1
fi

if [ "$DEPLOY_MOBILE" -eq 1 ] && [ ! -x "$DEPLOY_SCRIPT" ]; then
  echo "deploy script not executable: $DEPLOY_SCRIPT" >&2
  exit 1
fi

if [ -n "$ANDROID_DIAG_SOURCE" ] && [ ! -f "$ANDROID_DIAG_SOURCE" ]; then
  echo "Android diagnostics source not found: $ANDROID_DIAG_SOURCE" >&2
  exit 1
fi
if [ -n "$IOS_DIAG_SOURCE" ] && [ ! -f "$IOS_DIAG_SOURCE" ]; then
  echo "iOS diagnostics source not found: $IOS_DIAG_SOURCE" >&2
  exit 1
fi

SESSION_TS="$(date '+%Y%m%d_%H%M%S')"
SESSION_DIR="$OUTPUT_ROOT/${STEP_ID}_${SESSION_TS}"
mkdir -p "$SESSION_DIR"
SESSION_MANIFEST="$SESSION_DIR/session_manifest.txt"

{
  echo "step_id=$STEP_ID"
  echo "session_ts=$SESSION_TS"
  echo "run5_script=$(absolute_path "$RUN5_SCRIPT")"
  echo "time_min=$TIME_MIN"
  echo "max_attempts=$MAX_ATTEMPTS"
  echo "update_each=$UPDATE_EACH"
  echo "deploy_mobile=$DEPLOY_MOBILE"
  echo "require_receipt_gate=$REQUIRE_RECEIPT_GATE"
  echo "diag_tail_lines=$DIAG_TAIL_LINES"
  echo "output_root=$(absolute_path "$OUTPUT_ROOT")"
} > "$SESSION_MANIFEST"

log "Session directory: $SESSION_DIR"
log "Step ID: $STEP_ID"
log "Update each attempt: $UPDATE_EACH"
log "Deploy mobile each attempt: $DEPLOY_MOBILE"

# Pre-flight gate
phase "Pre-Flight Validation"
if ! "$SCRIPT_DIR/preflight.sh" 2>&1 | tee "$SESSION_DIR/preflight.log"; then
  echo "Pre-flight checks failed. Aborting verification loop." >&2
  exit 1
fi

final_status=1
successful_attempt=""

for attempt in $(seq 1 "$MAX_ATTEMPTS"); do
  ATTEMPT_TS="$(date '+%Y%m%d_%H%M%S')"
  ATTEMPT_DIR="$SESSION_DIR/attempt_${attempt}_${ATTEMPT_TS}"
  mkdir -p "$ATTEMPT_DIR"

  phase "Attempt ${attempt}/${MAX_ATTEMPTS} - Build/Deploy"

  attempt_fail=0
  fail_reasons=()

  if [ "$DEPLOY_MOBILE" -eq 1 ]; then
    if ! (cd "$ROOT_DIR" && "$DEPLOY_SCRIPT" both | tee "$ATTEMPT_DIR/deploy_mobile.log"); then
      attempt_fail=1
      fail_reasons+=("mobile deploy failed")
      log "Attempt $attempt: mobile deploy failed"
    else
      log "Attempt $attempt: mobile deploy succeeded"
    fi
  else
    log "Attempt $attempt: mobile deploy skipped"
  fi

  phase "Attempt ${attempt}/${MAX_ATTEMPTS} - 5-Node Run/Log Capture"

  run5_args=("--time=$TIME_MIN")
  if [ "$UPDATE_EACH" -eq 1 ]; then
    run5_args+=("--update")
  fi

  if ! (cd "$ROOT_DIR" && "$RUN5_SCRIPT" "${run5_args[@]}" | tee "$ATTEMPT_DIR/run5.stdout.log"); then
    attempt_fail=1
    fail_reasons+=("run5 execution failed")
    log "Attempt $attempt: run5 execution failed"
  fi

  mesh_logdir=""
  if mesh_logdir="$(resolve_latest_mesh_logdir)" && [ -d "$mesh_logdir" ]; then
    echo "mesh_logdir=$(absolute_path "$mesh_logdir")" > "$ATTEMPT_DIR/mesh_logdir.txt"
    log "Attempt $attempt: captured logdir $(absolute_path "$mesh_logdir")"
  else
    attempt_fail=1
    fail_reasons+=("unable to resolve logs/5mesh/latest")
    log "Attempt $attempt: unable to resolve latest 5mesh logdir"
  fi

  if [ -n "$mesh_logdir" ] && [ -d "$mesh_logdir" ]; then
    phase "Attempt ${attempt}/${MAX_ATTEMPTS} - Gate: Log Health"
    if log_health_gate "$mesh_logdir" "$ATTEMPT_DIR/log_health_gate.txt"; then
      log "Attempt $attempt: log health gate PASS"
    else
      attempt_fail=1
      fail_reasons+=("log health gate failed")
      log "Attempt $attempt: log health gate FAIL"
    fi

    phase "Attempt ${attempt}/${MAX_ATTEMPTS} - Gate: Pair Matrix (All Node Pairings)"
    if pair_matrix_gate "$mesh_logdir" "$ATTEMPT_DIR/pair_matrix_gate.txt"; then
      log "Attempt $attempt: pair matrix gate PASS"
    else
      attempt_fail=1
      fail_reasons+=("pair matrix gate failed")
      log "Attempt $attempt: pair matrix gate FAIL"
    fi

    phase "Attempt ${attempt}/${MAX_ATTEMPTS} - Gate: Crash/Fatal Marker Scan"
    if error_scan_gate "$mesh_logdir" "$ATTEMPT_DIR/error_scan_gate.txt"; then
      log "Attempt $attempt: crash/fatal marker gate PASS"
    else
      attempt_fail=1
      fail_reasons+=("crash/fatal marker gate failed")
      log "Attempt $attempt: crash/fatal marker gate FAIL"
    fi

    phase "Attempt ${attempt}/${MAX_ATTEMPTS} - Gate: Deterministic Verifiers"

    # Android diagnostics source precedence:
    # 1) explicit --android-diag, 2) pulled device diagnostics, 3) run5 android log
    android_receipt_log="$mesh_logdir/android.log"
    if [ -n "$ANDROID_DIAG_SOURCE" ]; then
      cp "$ANDROID_DIAG_SOURCE" "$ATTEMPT_DIR/android_diag_input.log"
      android_receipt_log="$ATTEMPT_DIR/android_diag_input.log"
    else
      if collect_android_diagnostics "$ATTEMPT_DIR/android-mesh_diagnostics-device.log" "$ATTEMPT_DIR/android-mesh_diagnostics.stderr"; then
        android_receipt_log="$ATTEMPT_DIR/android-mesh_diagnostics-device.log"
      fi
    fi

    # iOS diagnostics source precedence:
    # 1) explicit --ios-diag, 2) pulled device diagnostics, 3) repo ios_diagnostics_latest.log (if present), 4) run5 ios-device log
    ios_receipt_log="$mesh_logdir/ios-device.log"
    if [ -n "$IOS_DIAG_SOURCE" ]; then
      cp "$IOS_DIAG_SOURCE" "$ATTEMPT_DIR/ios_diag_input.log"
      ios_receipt_log="$ATTEMPT_DIR/ios_diag_input.log"
    elif collect_ios_diagnostics "$ATTEMPT_DIR/ios-mesh_diagnostics-device.log" "$ATTEMPT_DIR/ios-mesh_diagnostics.stderr"; then
      ios_receipt_log="$ATTEMPT_DIR/ios-mesh_diagnostics-device.log"
    elif [ -f "$ROOT_DIR/ios_diagnostics_latest.log" ] && [ -s "$ROOT_DIR/ios_diagnostics_latest.log" ]; then
      cp "$ROOT_DIR/ios_diagnostics_latest.log" "$ATTEMPT_DIR/ios_diag_input.log"
      ios_receipt_log="$ATTEMPT_DIR/ios_diag_input.log"
    fi

    # Restrict convergence/ordering verifiers to recent diagnostics lines to avoid
    # stale historical messages in long-running device logs.
    if tail_diagnostics_window "$android_receipt_log" "$ATTEMPT_DIR/android_receipt_window.log"; then
      android_receipt_log="$ATTEMPT_DIR/android_receipt_window.log"
    fi
    if tail_diagnostics_window "$ios_receipt_log" "$ATTEMPT_DIR/ios_receipt_window.log"; then
      ios_receipt_log="$ATTEMPT_DIR/ios_receipt_window.log"
    fi

    if run_verifier "relay_flap_regression" "$ATTEMPT_DIR/verify_relay_flap_regression.log" \
      bash "$ROOT_DIR/scripts/verify_relay_flap_regression.sh" "$mesh_logdir/ios-device.log"; then
      log "Attempt $attempt: relay flap regression verifier PASS"
    else
      attempt_fail=1
      fail_reasons+=("relay flap regression verifier failed")
      log "Attempt $attempt: relay flap regression verifier FAIL"
    fi

    if run_verifier "ble_only_pairing" "$ATTEMPT_DIR/verify_ble_only_pairing.log" \
      bash "$ROOT_DIR/scripts/verify_ble_only_pairing.sh" "$mesh_logdir/android.log" "$mesh_logdir/ios-device.log"; then
      log "Attempt $attempt: BLE-only pairing verifier PASS"
    else
      attempt_fail=1
      fail_reasons+=("BLE-only pairing verifier failed")
      log "Attempt $attempt: BLE-only pairing verifier FAIL"
    fi

    if run_verifier "receipt_convergence" "$ATTEMPT_DIR/verify_receipt_convergence.log" \
      bash "$ROOT_DIR/scripts/verify_receipt_convergence.sh" "$android_receipt_log" "$ios_receipt_log"; then
      log "Attempt $attempt: receipt convergence verifier PASS"
    else
      if [ "$REQUIRE_RECEIPT_GATE" -eq 1 ]; then
        attempt_fail=1
        fail_reasons+=("receipt convergence verifier failed")
        log "Attempt $attempt: receipt convergence verifier FAIL (required gate)"
      else
        log "Attempt $attempt: receipt convergence verifier WARN (non-blocking)"
      fi
    fi

    if run_verifier "delivery_state_monotonicity" "$ATTEMPT_DIR/verify_delivery_state_monotonicity.log" \
      bash "$ROOT_DIR/scripts/verify_delivery_state_monotonicity.sh" "$android_receipt_log" "$ios_receipt_log"; then
      log "Attempt $attempt: delivery-state monotonicity verifier PASS"
    else
      attempt_fail=1
      fail_reasons+=("delivery-state monotonicity verifier failed")
      log "Attempt $attempt: delivery-state monotonicity verifier FAIL"
    fi

    tail_snapshot "$mesh_logdir" "$ATTEMPT_DIR/tail_snapshot.txt"
  fi

  {
    echo "attempt=$attempt"
    echo "attempt_dir=$(absolute_path "$ATTEMPT_DIR")"
    echo "result=$([ "$attempt_fail" -eq 0 ] && echo PASS || echo FAIL)"
    if [ "${#fail_reasons[@]}" -gt 0 ]; then
      echo "fail_reasons=${fail_reasons[*]}"
    fi
  } > "$ATTEMPT_DIR/attempt_result.txt"

  if [ "$attempt_fail" -eq 0 ]; then
    final_status=0
    successful_attempt="$ATTEMPT_DIR"
    log "Attempt $attempt PASSED all required gates"
    break
  fi

  log "Attempt $attempt FAILED required gates"
  if [ "${#fail_reasons[@]}" -gt 0 ]; then
    log "Failure reasons: ${fail_reasons[*]}"
  fi

done

phase "Session Summary"

if [ "$final_status" -eq 0 ]; then
  log "LIVE VERIFICATION PASS: SUCCESS"
  log "Successful attempt: $(absolute_path "$successful_attempt")"
  echo "success_attempt=$(absolute_path "$successful_attempt")" >> "$SESSION_MANIFEST"
  echo "session_result=PASS" >> "$SESSION_MANIFEST"
else
  log "LIVE VERIFICATION PASS: FAILED after $MAX_ATTEMPTS attempt(s)"
  log "Session dir for investigation: $(absolute_path "$SESSION_DIR")"
  echo "session_result=FAIL" >> "$SESSION_MANIFEST"
fi

log "Session manifest: $(absolute_path "$SESSION_MANIFEST")"
exit "$final_status"
