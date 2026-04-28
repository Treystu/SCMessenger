# Script Hygiene Guidelines

Status: Active  
Last updated: 2026-03-16  
Purpose: Canonical standards for script quality, reliability, and maintainability across the SCMessenger project.

---

## 1. Shell Script Standards

### 1.1 Required Header

Every shell script MUST begin with:

```bash
#!/usr/bin/env bash
set -euo pipefail

# scripts/example.sh — Brief description of purpose
# 
# Usage:
#   ./scripts/example.sh [options]
#
# Environment variables:
#   VAR_NAME — description (default: value)
```

### 1.2 Error Handling

**REQUIRED**: All scripts must use strict mode:
```bash
set -euo pipefail
```

**REQUIRED**: All scripts must register cleanup handlers:
```bash
cleanup() {
  # Kill background processes
  # Remove temp files
  # Restore original state if needed
}
trap cleanup EXIT INT TERM
```

**REQUIRED**: All command failures must be handled:
```bash
# For commands that may legitimately fail:
command_that_might_fail || {
  echo "Warning: command failed, continuing..." >&2
}

# For commands that must succeed:
if ! command_that_must_succeed; then
  echo "Error: critical command failed" >&2
  exit 1
fi
```

**FORBIDDEN**: Unchecked command substitution:
```bash
# BAD: result=$(command)  # command failure is silent
# GOOD: result=$(command) || { echo "Failed"; exit 1; }
```

### 1.3 Device Detection

**STANDARD PATTERN** — Use this for all device detection:

```bash
# Android
ANDROID_SERIAL="${ANDROID_SERIAL:-}"
if [ -z "$ANDROID_SERIAL" ]; then
  ANDROID_SERIAL=$(adb devices 2>/dev/null | awk 'NR>1 && $2=="device" {print $1; exit}')
fi
if [ -z "$ANDROID_SERIAL" ]; then
  echo "error: no Android device connected" >&2
  exit 1
fi

# iOS Physical Device
IOS_DEVICE_UDID="${IOS_DEVICE_UDID:-}"
if [ -z "$IOS_DEVICE_UDID" ]; then
  IOS_DEVICE_UDID=$(xcrun devicectl list devices \
    --hide-default-columns --columns Identifier --columns State --hide-headers 2>/dev/null | \
    awk '$2 ~ /(available|connected)/ {print $1; exit}')
fi

# iOS Simulator
IOS_SIM_UDID="${IOS_SIM_UDID:-}"
if [ -z "$IOS_SIM_UDID" ]; then
  IOS_SIM_UDID=$(xcrun simctl list devices 2>/dev/null | awk -F '[()]' '/Booted/{print $2; exit}')
fi
```

**FORBIDDEN**: Hardcoded device IDs or serials

### 1.4 Path Handling

**REQUIRED**: All scripts must derive paths relative to their own location:
```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
```

**FORBIDDEN**: Hardcoded absolute paths (except for system commands)

**FORBIDDEN**: Using `~` for home directory expansion in scripts

### 1.5 Logging

**REQUIRED**: All scripts must log actions with timestamps:
```bash
log() {
  printf '[%s] %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$*"
}

log_info()  { echo -e "\033[0;32m[INFO]\033[0m  $(date '+%H:%M:%S') $*"; }
log_warn()  { echo -e "\033[1;33m[WARN]\033[0m  $(date '+%H:%M:%S') $*"; }
log_error() { echo -e "\033[0;31m[ERROR]\033[0m $(date '+%H:%M:%S') $*" >&2; }
```

### 1.6 Dry-Run Mode

**REQUIRED**: Scripts that modify state MUST support dry-run mode:
```bash
DRY_RUN="${DRY_RUN:-0}"

dry_run() {
  if [ "$DRY_RUN" = "1" ]; then
    echo "[DRY-RUN] $*" >&2
    return 0
  fi
  "$@"
}

# Usage:
dry_run adb install -r "$apk_path"
dry_run rm -rf "$build_dir"
```

---

## 2. Python Script Standards

### 2.1 Required Header

```python
#!/usr/bin/env python3
"""
scripts/example.py — Brief description

Usage:
    python3 scripts/example.py <required_arg> [--optional-flag]

Environment variables:
    VAR_NAME — description (default: value)
"""
import sys
import os
import re
import argparse
```

### 2.2 Error Handling

**REQUIRED**: All scripts must handle file I/O errors:
```python
def read_log(path: str) -> str:
    try:
        with open(path, 'r', errors='ignore') as f:
            return f.read()
    except OSError as e:
        print(f"Error reading {path}: {e}", file=sys.stderr)
        return ""
```

**REQUIRED**: All scripts must use argparse for argument parsing:
```python
parser = argparse.ArgumentParser(description="...")
parser.add_argument("input", help="Input file path")
parser.add_argument("--verbose", "-v", action="store_true")
args = parser.parse_args()
```

### 2.3 Log Parsing

**STANDARD**: Always strip ANSI codes before parsing:
```python
def strip_ansi(s: str) -> str:
    return re.sub(r'\x1b\[[^m]*m', '', s)
```

**STANDARD**: Use consistent peer ID pattern:
```python
PEER_ID_PAT = re.compile(r"(12D3KooW[1-9A-HJ-NP-Za-km-z]{44,})")
```

---

## 3. Log Format Standards

### 3.1 Structured Log Markers

All scripts that capture or emit logs MUST use these standard markers:

```bash
# Session markers
=== TEST_START_MARKER: <ISO-8601 timestamp> ===
=== CAPTURE_START: <timestamp> ===
=== CAPTURE_END: <timestamp> ===

# Device identification
=== OWN_IDENTITY: <peer_id> ===
=== DEVICE_INFO: <platform> <model> <os_version> ===

# Phase markers
=== PHASE_START: <phase_name> ===
=== PHASE_END: <phase_name> <pass|fail> ===
```

### 3.2 Log File Naming

**STANDARD**: Use timestamped directories:
```
logs/<category>/<YYYYMMDD_HHMMSS>/
  ├── android.log
  ├── ios-device.log
  ├── ios-sim.log
  ├── gcp.log
  ├── osx.log
  ├── harness.log
  └── analysis.log
```

**REQUIRED**: Maintain a `latest` symlink:
```bash
ln -sfn "$TIMESTAMP" "logs/5mesh/latest"
```

### 3.3 Log Rotation

**REQUIRED**: All log capture scripts must support size limits:
```bash
MAX_LOG_SIZE_MB="${MAX_LOG_SIZE_MB:-100}"

# Check before capture
LOG_SIZE_KB=$(du -sk "$LOGDIR" 2>/dev/null | awk '{print $1}')
if [ "$LOG_SIZE_KB" -gt $((MAX_LOG_SIZE_MB * 1024)) ]; then
  log_warn "Log directory exceeds ${MAX_LOG_SIZE_MB}MB — consider pruning"
fi
```

---

## 4. Testing Standards

### 4.1 Script Testing

**REQUIRED**: Every new script must have a `--help` flag:
```bash
usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Description of what this script does.

Options:
  --option=<value>  Description (default: value)
  -h, --help        Show this help text
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac
```

### 4.2 Verification Commands

**REQUIRED**: Every script that modifies state must document its verification command:

```bash
# At the end of the script header comment:
# Verification:
#   ./scripts/example.sh --dry-run
#   ./scripts/example.sh --help
```

### 4.3 Integration with Test Pyramid

Scripts must align with the testing pyramid levels:

| Level | Script Type | Verification |
|-------|------------|--------------|
| L1 | Unit test runners | `cargo test --workspace` |
| L2 | Platform smoke | `live-smoke.sh`, `verify_ws12_matrix.sh` |
| L3 | Live verification | `run5-live-feedback.sh` |
| L4 | Full mesh | `run5.sh` |

---

## 5. Documentation Standards

### 5.1 Script README

**REQUIRED**: `scripts/README.md` must be updated when adding or removing scripts:

```markdown
| Script | Purpose | Level | Input |
|--------|---------|-------|-------|
| `example.sh` | Does X | L2 | Device logs |
```

### 5.2 Inline Documentation

**REQUIRED**: Complex logic must be documented:
```bash
# P5: CRITICAL — Always clean before device deploy to prevent stale APK issues
# This prevents Hilt NoClassDefFoundError caused by stale generated code.
log_info "Cleaning build artifacts..."
```

### 5.3 Cross-References

**REQUIRED**: Scripts that depend on other scripts must document the dependency:
```bash
# Depends on: scripts/preflight.sh, scripts/deploy_to_device.sh
# Called by: scripts/run5-live-feedback.sh, scripts/debug-session.sh
```

---

## 6. Cleanup Standards

### 6.1 Temp File Management

**REQUIRED**: All temp files must use `mktemp` and be cleaned up:
```bash
TEMP_FILE="$(mktemp)"
cleanup() {
  rm -f "$TEMP_FILE"
}
trap cleanup EXIT
```

**FORBIDDEN**: Using `/tmp` directly — use `mktemp` or repo-local `tmp/`:
```bash
# BAD:  > /tmp/myfile.txt
# GOOD: > "$(mktemp)"
# GOOD: > "$ROOT_DIR/tmp/myfile.txt"
```

### 6.2 Process Cleanup

**REQUIRED**: All background processes must be tracked and cleaned:
```bash
declare -a BACKGROUND_PIDS=()

start_background_process() {
  some_command &
  BACKGROUND_PIDS+=($!)
}

cleanup() {
  for pid in "${BACKGROUND_PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
  done
}
trap cleanup EXIT
```

### 6.3 Log Pruning

**REQUIRED**: Scripts that create logs must document pruning:
```bash
# Log pruning:
#   KEEP_HOURS=24 ./scripts/prune_sim_logs.sh  # Keep last 24 hours
#   ./scripts/prune_sim_logs.sh --dry-run       # Preview what would be deleted
```

---

## 7. Platform Parity Standards

### 7.1 Consistent Behavior

All platform-specific scripts must document parity points:

```bash
# Parity: Android and iOS must handle these identically:
# - Device detection failure (exit 1 with clear message)
# - Log capture timeout (graceful cleanup)
# - App launch failure (retry with backoff)
```

### 7.2 Cross-Platform Verification

**REQUIRED**: Scripts that touch multiple platforms must verify all:
```bash
verify_platform_parity() {
  echo "Verifying Android..."
  # Android checks
  
  echo "Verifying iOS..."
  # iOS checks
  
  echo "Verifying headless..."
  # GCP/OSX checks
}
```

---

## 8. Security Standards

### 8.1 Credential Handling

**FORBIDDEN**: Hardcoded credentials or API keys:
```bash
# BAD:  GCP_KEY="AIzaSy..."
# GOOD: GCP_KEY="${GCP_KEY:?GCP_KEY not set}"
```

### 8.2 Log Sanitization

**REQUIRED**: Scripts that export logs must sanitize sensitive data:
```bash
sanitize_log() {
  sed -E \
    -e 's/12D3KooW[a-zA-Z0-9]{45}/<PEER_ID>/g' \
    -e 's/[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}/<UUID>/g' \
    -e 's/\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}/<IP>/g'
}
```

---

## 9. Performance Standards

### 9.1 Timeout Handling

**REQUIRED**: All network operations must have timeouts:
```bash
# SSH operations
ssh -o ConnectTimeout=10 -o ServerAliveInterval=15 ...

# Network checks
nc -z -w 5 "$host" "$port"

# Log capture
timeout "$DURATION_SEC" adb logcat ...
```

### 9.2 Resource Limits

**REQUIRED**: Scripts that may consume significant resources must document limits:
```bash
# Resource limits:
#   MAX_LOG_SIZE_MB=100    # Maximum log directory size
#   MAX_ATTEMPTS=3         # Maximum retry attempts
#   DURATION_SEC=300       # Maximum capture duration
```

---

## 10. Enforcement

### 10.1 Pre-Commit Checks

Before committing any script changes:

1. Run `shellcheck` on all `.sh` files
2. Run `python3 -m py_compile` on all `.py` files
3. Verify `set -euo pipefail` is present
4. Verify cleanup handlers are registered
5. Run `./scripts/docs_sync_check.sh`

### 10.2 CI Integration

Script hygiene is enforced by CI:
- `.github/workflows/ci.yml` runs `shellcheck`
- All scripts must pass `--help` without errors
- All scripts must exit cleanly with `--dry-run`

### 10.3 Review Checklist

When reviewing script changes:

- [ ] Header comment with usage and environment variables
- [ ] `set -euo pipefail` at the top
- [ ] Cleanup trap handler registered
- [ ] No hardcoded paths or device IDs
- [ ] Proper error messages to stderr
- [ ] Log output with timestamps
- [ ] Dry-run mode for state-modifying scripts
- [ ] Documentation updated in `scripts/README.md`
- [ ] Cross-references updated in dependent scripts

---

## Appendix: Script Inventory Compliance

Current compliance status (as of 2026-03-16):

| Script | Header | Strict Mode | Cleanup | Dry-Run | Status |
|--------|--------|-------------|---------|---------|--------|
| `deploy_to_device.sh` | ✓ | ✓ | ✗ | ✗ | ⚠️ |
| `live-smoke.sh` | ✓ | ✓ | ✓ | ✗ | ⚠️ |
| `run5-live-feedback.sh` | ✓ | ✓ | ✓ | ✗ | ⚠️ |
| `check_logs.py` | ✗ | N/A | N/A | N/A | ⚠️ |
| `analyze_mesh.py` | ✓ | N/A | N/A | N/A | ✓ |
| `verify_all_builds.sh` | ✗ | ✓ | ✗ | ✗ | ⚠️ |
| `capture_both_logs.sh` | ✗ | ✗ | ✗ | ✗ | ❌ |
| `comprehensive_log_capture.sh` | ✗ | ✗ | ✗ | ✗ | ❌ |

Legend: ✓ = Compliant, ⚠️ = Partial, ❌ = Non-compliant, N/A = Not applicable

---

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-03-16 | — | Initial creation |
