# SCMessenger Issues Burndown Script Guide

## Overview

The `burndown_issues.sh` script provides a systematic approach to addressing open issues in the SCMessenger codebase. It features:

- **Baseline build/test** at the start to establish current state
- **Gatekeeper verification** before and after each fix attempt
- **Test/logging validation** for each fix to ensure resolution
- **Graceful skip** for issues that are too challenging or require unavailable resources
- **Priority-based execution** (P0 → P1 → P2)
- **Perfect implementation preference** - fixes are either complete or skipped
- **Final verification pass** to confirm all changes are stable
- **Comprehensive reporting** with session artifacts

## Usage

### Basic Usage

```bash
# Run full burndown on all open issues
./scripts/burndown_issues.sh

# Run on specific issues only
./scripts/burndown_issues.sh --issues "AND-SEND-BTN-001,AND-DELIVERY-001"

# Skip issues requiring physical devices
./scripts/burndown_issues.sh --skip-hard

# Dry run (show what would be done)
./scripts/burndown_issues.sh --dry-run

# Skip baseline build/test
./scripts/burndown_issues.sh --skip-baseline
```

### Options

| Option | Description |
|--------|-------------|
| `--issues <ids>` | Comma-separated list of specific issue IDs to fix |
| `--skip-hard` | Skip issues requiring special environments/hardware |
| `--dry-run` | Show planned actions without making changes |
| `--skip-baseline` | Skip the initial baseline build/test |
| `--help` | Show help message |

## Workflow

### 1. Baseline Phase

The script starts by running a baseline verification:

```bash
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
./scripts/docs_sync_check.sh
./gradlew :app:lintDebug  # Android (if available)
```

This establishes the current state and identifies pre-existing issues.

### 2. Issue Execution Phases

Issues are executed in priority order:

#### Phase 1: P0 Critical Issues
- `AND-SEND-BTN-001` - Send Button Not Responding
- `FIELD-BINARY-001` - Field iOS Binary Stale

#### Phase 2: P1 High Priority Issues
- `AND-DELIVERY-001` - Delivery State Tracking Broken
- `AND-MSG-VIS-001` - Message Visibility Issues
- `CROSS-PAIR-001` - iOS/Android Cross-Device Continuity
- `IOS-DIAG-001` - iOS Diagnostics Extraction Unreliable
- `IOS-FREEZE-001` - App Freezing and Hanging
- `AND-CELLULAR-001` - Android Cellular Cannot Send

#### Phase 3: P2 Medium Priority Issues
- `AND-PERMISSION-001` - Permission Request Loop
- `OPS-ADB-001` - Android Wireless ADB Stability
- `TEST-ENV-001` - Docker Simulation Not Executed
- `VALIDATION-001` - Required Closure Evidence Missing

### 3. Per-Issue Workflow

For each issue, the script follows this workflow:

```
┌─────────────────────────────────────────────────────────────┐
│                    Issue Attempt Workflow                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ Gatekeeper Pre  │
                    │   Check         │
                    └────────┬────────┘
                              │
           ┌──────────────────┼──────────────────┐
           │                  │                  │
           ▼                  ▼                  ▼
    ┌────────────┐     ┌────────────┐     ┌────────────┐
    │   Check    │     │  Identify  │     │  Check     │
    │  Exists    │     │  Files     │     │  Env Avail │
    └─────┬──────┘     └─────┬──────┘     └─────┬──────┘
          │                  │                  │
          └──────────────────┼──────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │  Can Fix Issue? │
                    └────────┬────────┘
                              │
           ┌──────────────────┼──────────────────┐
           │ Yes              │ No               │
           ▼                  ▼                  │
    ┌────────────┐     ┌────────────┐           │
    │  Execute   │     │   Skip     │           │
    │    Fix     │     │  (Graceful)│           │
    └─────┬──────┘     └────────────┘           │
          │                                      │
          ▼                                      │
    ┌────────────┐                               │
    │ Gatekeeper │                               │
    │  Post-Check│                               │
    └─────┬──────┘                               │
          │                                      │
    ┌─────┴──────┐                               │
    │ Pass?      │                               │
    └─────┬──────┘                               │
          │                                      │
    ┌─────┴──────┐                               │
    │ Yes    No  │                               │
    ▼         ▼  │                               │
 FIXED    FAILED │                               │
                  │                               │
          ┌──────┴───────────────────────────────┘
          │
          ▼
    Next Issue
```

### 4. Gatekeeper Verification

#### Pre-Check Gatekeeper
Verifies:
- Issue exists in tracker
- Affected files can be identified
- Current build state

#### Post-Check Gatekeeper
Verifies:
- `cargo build --workspace` passes
- `cargo test --workspace` passes
- Platform-specific checks (Android lint, iOS verification)
- Documentation sync passes

### 5. Final Verification

After all issues are processed, a final verification runs:

```bash
cargo build --workspace
cargo test --workspace
./gradlew :app:compileDebugKotlin  # Android
./iOS/verify-test.sh               # iOS
./scripts/docs_sync_check.sh
```

## Issue-Specific Fix Strategies

### AND-SEND-BTN-001: Send Button Not Responding

**Root Cause:** UI thread blocked, Compose recomposition issue, or coroutine scope cancellation.

**Fix Strategy:**
1. Verify click handler exists in `ChatScreen.kt`
2. Check coroutine scope configuration
3. Add defensive logging for debugging
4. Ensure proper ViewModel scope usage

**Files:**
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt`

### AND-DELIVERY-001: Delivery State Tracking Broken

**Root Cause:** Message ID propagation issues, missing retry limits.

**Fix Strategy:**
1. Check for `msg=unknown` patterns
2. Verify message ID is properly propagated
3. Add max retry limit (10 attempts)
4. Ensure delivery state enum exists

**Files:**
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `core/src/lib.rs`

### AND-MSG-VIS-001: Message Visibility Issues

**Root Cause:** UI filters hiding pending messages.

**Fix Strategy:**
1. Identify filters that only show "delivered" messages
2. Update filter to include pending messages
3. Add delivery state indicator to message list

**Files:**
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt`

### IOS-FREEZE-001: App Freezing and Hanging

**Root Cause:** Main thread blocking, excessive debug logging, SwiftUI state thrashing.

**Fix Strategy:**
1. Add `Task.yield()` in retry loops
2. Reduce debug logging on hot paths
3. Optimize SwiftUI state updates
4. Move heavy operations to background

**Files:**
- `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- `iOS/SCMessenger/SCMessenger/Transport/BLEPeripheralManager.swift`

### AND-CELLULAR-001: Android Cellular Cannot Send

**Root Cause:** Cellular transport not properly configured or missing relay fallback.

**Fix Strategy:**
1. Verify cellular transport handling exists
2. Check relay fallback for NAT traversal
3. Ensure SmartTransportRouter includes cellular path

**Files:**
- `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt`

### AND-PERMISSION-001: Permission Request Loop

**Root Cause:** Multiple code paths requesting same permissions without deduplication.

**Fix Strategy:**
1. Add permission state tracking
2. Deduplicate permission requests in MainActivity
3. Coordinate all permission sources into single request
4. Add request state machine + backoff timer

**Files:**
- `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt`
- `android/app/src/main/java/com/scmessenger/android/utils/Permissions.kt`

### OPS-ADB-001: Android Wireless ADB Stability

**Root Cause:** ADB endpoint drifts during reconnect cycles.

**Fix Strategy:**
1. Add stable reconnect logic
2. Implement exponential backoff
3. Add auto-recovery for connection drops

**Files:**
- `scripts/deploy_to_device.sh`

### IOS-DIAG-001: iOS Diagnostics Extraction Unreliable

**Root Cause:** Socket closes on large file transfer.

**Fix Strategy:**
1. Implement chunked transfer for large files
2. Add buffer management
3. Improve error handling for socket closure

**Files:**
- `scripts/capture_logs.sh`

### CROSS-PAIR-001: Cross-Device Continuity

**Root Cause:** Synchronized evidence not captured for cross-device delivery.

**Fix Strategy:**
1. Run cross-pair verification script
2. Capture synchronized artifacts
3. Verify bidirectional delivery + receipt convergence

**Files:**
- `scripts/verify_cross_pair_local.sh`

### TEST-ENV-001: Docker Simulation Not Executed

**Root Cause:** Docker prerequisites not resolved.

**Fix Strategy:**
1. Check Docker availability
2. Run Docker simulation tests
3. Archive results

**Files:**
- `docker/run-all-tests.sh`

### VALIDATION-001: Required Closure Evidence Missing

**Root Cause:** Live network matrix, ACK-safe switch, reinstall continuity evidence not captured.

**Fix Strategy:**
1. Run receipt convergence verification
2. Run BLE pairing verification
3. Run relay flap verification
4. Archive all evidence

**Files:**
- `scripts/verify_receipt_convergence.sh`
- `scripts/verify_ble_only_pairing.sh`
- `scripts/verify_relay_flap_regression.sh`

### FIELD-BINARY-001: Field iOS Binary Stale

**Root Cause:** Deployed iOS binary doesn't contain latest source fixes.

**Fix Strategy:**
1. Check for iOS device connection
2. Build latest iOS binary
3. Install on device
4. Capture post-deploy evidence

**Files:**
- `iOS/build-device.sh`

## Output and Artifacts

### Session Directory Structure

```
tmp/burndown-results/
└── YYYYMMDD_HHMMSS/
    ├── session-info.txt           # Session metadata
    ├── BURNDOWN_REPORT.md         # Main report
    ├── baseline/                  # Baseline build/test logs
    │   └── baseline_*.log
    ├── fixes/                     # Fix attempt details
    │   ├── AND-SEND-BTN-001_details.txt
    │   └── ...
    ├── gatekeeper/                # Gatekeeper verification results
    │   ├── pre_AND-SEND-BTN-001.json
    │   ├── post_AND-SEND-BTN-001.json
    │   ├── build_*.log
    │   ├── test_*.log
    │   └── exit_verification.json
    ├── validation/                # Validation test results
    │   ├── receipt_convergence.log
    │   ├── ble_pairing.log
    │   └── relay_flap.log
    └── final/                     # Final verification logs
        └── final_verification_*.log
```

### Report Format

The generated `BURNDOWN_REPORT.md` includes:

1. **Summary Table** - Attempted, Fixed, Skipped, Failed counts
2. **Detailed Results** - Issue-by-issue results with status
3. **Issues Fixed** - Details on successfully fixed issues
4. **Issues Skipped** - Reasons for skipped issues
5. **Artifacts** - Location of all session artifacts
6. **Next Steps** - Recommended follow-up actions

## Graceful Skip Behavior

The script gracefully skips issues when:

1. **Environment not available** - Physical device required but not connected
2. **Hardware required** - iOS device, Android device, or specific hardware needed
3. **Docker not available** - Docker daemon not running or not installed
4. **Pre-check fails** - Issue cannot be identified or located
5. **--skip-hard flag** - User explicitly skips hard issues

Skipped issues are logged with reasons and do not cause the script to fail.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success - all attempted fixes passed |
| 1 | Baseline failed |
| 2 | One or more fixes failed |
| 3 | Gatekeeper verification failed |
| 4 | All issues were skipped |

## Best Practices

### Before Running

1. **Commit current changes** - Ensure working directory is clean
2. **Check device connections** - Ensure required devices are connected
3. **Verify Docker** - If testing Docker simulation, ensure Docker is running
4. **Review issues** - Understand what issues will be attempted

### During Execution

1. **Monitor output** - Watch for warnings or errors
2. **Check gatekeeper results** - Verify pre/post checks pass
3. **Note skipped issues** - Plan to address with proper environment

### After Running

1. **Review report** - Read `BURNDOWN_REPORT.md`
2. **Check artifacts** - Review logs in session directory
3. **Address failures** - Manually fix issues that failed
4. **Re-run if needed** - Address environment issues and re-run

## Troubleshooting

### Issue: Script fails at baseline

**Solution:** Baseline failures are warnings, not errors. The script continues even if baseline has issues. Review the baseline log to understand pre-existing problems.

### Issue: All issues skipped

**Solution:** This typically means the environment doesn't have required hardware. Use `--skip-hard` to skip hardware-dependent issues, or connect required devices.

### Issue: Gatekeeper fails after fix

**Solution:** The fix may have introduced regressions. Review the gatekeeper logs in `gatekeeper/build_*.log` and `gatekeeper/test_*.log` to identify issues.

### Issue: Fix doesn't resolve the problem

**Solution:** Some issues may require deeper investigation or physical device testing. The script provides a starting point but may not resolve all edge cases.

## Integration with CI/CD

The burndown script can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
jobs:
  burndown:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run burndown
        run: ./scripts/burndown_issues.sh --skip-hard
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: burndown-results
          path: tmp/burndown-results/
```

## See Also

- [`MASTER_BUG_TRACKER.md`](../MASTER_BUG_TRACKER.md) - Centralized bug tracking
- [`REMAINING_WORK_TRACKING.md`](../REMAINING_WORK_TRACKING.md) - Work item tracking
- [`scripts/README.md`](../scripts/README.md) - Scripts operations guide
- [`docs/SCRIPT_HYGIENE_GUIDE.md`](./SCRIPT_HYGIENE_GUIDE.md) - Script development guidelines
