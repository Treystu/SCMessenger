# Script Sanity Check Agentic Prompt

Status: Active  
Last updated: 2026-03-16  
Purpose: Canonical prompt for sanity-checking the deployment → log capture → analysis → iteration workflow. Produces implementation instructions for a less-capable model.

---

## The Prompt (Copy-Paste Ready)

```text
You are a senior DevOps/Platform engineer performing a comprehensive sanity check of the SCMessenger app's debugging, testing, and verification script ecosystem. Your job is to think through the ENTIRE chain of events—from deploying new variants on Android and iOS devices, to capturing logs, analyzing logs, and iterating on issues until complete—then produce a tightly-scoped implementation plan that a less-capable model can execute.

## Context

SCMessenger is a peer-to-peer encrypted messenger with a 5-node mesh test topology:
1. GCP — headless relay (Docker on cloud VM)
2. OSX — headless relay (local cargo binary)
3. Android — full node (Pixel 6a via adb)
4. iOS Device — full node (physical device via devicectl/xcodebuild)
5. iOS Simulator — full node (simulator via simctl)

The Rust core (`core/`) compiles to Android (via UniFFI), iOS (via UniFFI XCFramework), and headless CLI (`cli/`). Mobile apps are in `android/` (Kotlin/Gradle) and `iOS/` (Swift/Xcode).

## Your Mission

### Phase 1: Mental Model Construction (Think Deeply)

Walk through the ENTIRE lifecycle of a debugging session. For each step, identify the exact scripts involved, their inputs/outputs, failure modes, and gaps. Think about:

**Step 1: Pre-Flight Checks**
- What device detection happens? (adb devices, xcrun devicectl, xcrun simctl)
- Are there scripts that verify build prerequisites? (`verify_all_builds.sh`, `verify_ios_bindings.sh`)
- What happens if a device is missing or disconnected mid-session?
- Are there wireless ADB reconnect mechanisms? (Yes, in `live-smoke.sh` lines 48-76)

**Step 2: Build Phase**
- Android: `./gradlew clean assembleDebug` (in `deploy_to_device.sh`, `android/install-clean.sh`)
- iOS: `xcodebuild clean build` + UniFFI binding generation (`iOS/copy-bindings.sh`, `iOS/install-device.sh`, `iOS/install-sim.sh`)
- Rust core: `cargo build --workspace` (implicit in headless nodes)
- GCP: `gcloud builds submit` + Docker container update (`deploy_gcp_node.sh`)
- What happens if the Rust core hasn't been rebuilt? (iOS XCFramework gets stale)
- What happens if Gradle daemon is stale? (`install-clean.sh` stops it)

**Step 3: Deploy Phase**
- Android: `adb install -r` or `./gradlew installDebug`
- iOS Device: `xcrun devicectl device install app`
- iOS Sim: `xcrun simctl install` + `xcrun simctl launch`
- GCP: Docker container restart via SSH
- OSX: Binary launch or `cargo run`
- What are the launch commands? Are they correct? (`adb shell am start`, `xcrun simctl launch`)

**Step 4: Log Capture Phase**
- Android: `adb logcat -v time` (multiple variants in `capture_both_logs.sh`, `comprehensive_log_capture.sh`, `live-smoke.sh`, `run5.sh`)
- iOS Simulator: `xcrun simctl spawn <udid> log stream --predicate 'subsystem == "com.scmessenger"'`
- iOS Device: Requires `xcrun devicectl` or filtered syslog
- GCP/OSX: File-based logging or stdout capture
- Log sanitization: Are timestamps stripped? Are peer IDs redacted? (`mince_logs.py`)
- Log rotation: Are old logs pruned? (`prune_sim_logs.sh`)
- What's the log directory structure? (`logs/5mesh/<timestamp>/`, `logs/live-smoke/<timestamp>/`, `logs/live-verify/<step>/`)

**Step 5: Analysis Phase**
- Peer ID extraction: Multiple regex patterns in `check_logs.py`, `analyze_mesh.py`
- Visibility matrix: Which nodes see which peers? (directed graph)
- Error extraction: `grep -iE "error|exception|crash"` patterns
- Transport evidence: BLE, direct, relay, WiFi markers
- Delivery state tracking: `delivery_state` markers for message flow
- LogSankey visualizer: `log-visualizer/server.mjs` for Sankey diagram generation
- `analyze_issues.sh`: Quick error summary
- `diagnose_message_issue.sh`: Message persistence debugging

**Step 6: Verification Phase (Deterministic Verifiers)**
- `verify_relay_flap_regression.sh`: iOS relay dial-loop detection
- `verify_ble_only_pairing.sh`: BLE-only strict mode validation
- `verify_receipt_convergence.sh`: Message receipt convergence
- `verify_delivery_state_monotonicity.sh`: Delivery state ordering
- `verify_cross_pair_local.sh`: Cross-pair local transport
- `verify_ws12_matrix.sh`: WS12 parity validation
- `correlate_relay_flap_windows.sh`: Relay churn correlation

**Step 7: Iteration Loop**
- The `run5-live-feedback.sh` script orchestrates: deploy → run5 → analyze → gate check → retry
- Phase gates: log health, pair matrix, crash markers, deterministic verifiers
- Max attempts (default 3) with strict pass/fail
- Output goes to `logs/live-verify/<step>/<attempt>/`

### Phase 2: Gap Analysis (Be Brutally Honest)

For each step above, identify:
1. **Missing error handling**: What happens when a command fails silently?
2. **Race conditions**: Are there timing issues between deploy and log capture start?
3. **Platform asymmetries**: Do Android and iOS handle the same scenario differently?
4. **Log format inconsistencies**: Are log parsers aligned with actual log output?
5. **Stale artifact risks**: Could old builds/XCFrameworks/APKs cause false positives?
6. **Device detection gaps**: What if multiple devices are connected?
7. **Cleanup gaps**: Are temp files, old logs, stale processes cleaned up?
8. **Documentation drift**: Do scripts match what docs say they do?

### Phase 3: Workflow Tightening (Make It Bulletproof)

Design improvements to make the workflow "super tight":
1. **Unified entry point**: Should there be a single `debug-session.sh` that orchestrates everything?
2. **Pre-flight validation**: Comprehensive device/dependency check before any work begins
3. **Atomic log capture**: Guarantee log files are complete and uncorrupted
4. **Standardized log format**: Ensure all platforms emit logs in a parseable format
5. **Automated triage**: Scripts should classify issues by severity and affected component
6. **Regression detection**: Compare against baseline runs to catch new failures
7. **Cleanup guarantees**: Always clean up, even on failure (trap handlers)

### Phase 4: Implementation Plan (For a Less-Capable Model)

Produce a detailed, step-by-step implementation plan that a less-capable/expensive model can follow. Each step should be:
- **Atomic**: One clear action per step
- **Specific**: Exact file paths, exact commands, exact line numbers when editing
- **Testable**: Clear success criteria for each step
- **Ordered**: Dependencies respected, no circular requirements

Format each step as:
```
### Step N: <Title>
**File**: <path to file>
**Action**: <create|modify|delete>
**Rationale**: <why this change is needed>
**Implementation**:
<exact code/commands to execute>
**Verification**:
<how to verify this step succeeded>
**LOC Estimate**: <approximate lines of code changed>
```

### Phase 5: Hygiene Guidelines

Document new guidelines for improved script hygiene:
1. **Error handling standards**: Every script must use `set -euo pipefail`
2. **Logging standards**: Every script must log its actions with timestamps
3. **Cleanup standards**: Every script must register trap handlers for cleanup
4. **Device detection standards**: Single canonical device detection pattern
5. **Log format standards**: Structured logging for machine-parseable output
6. **Documentation standards**: Every script must have a header comment with usage
7. **Testing standards**: Every script must have a `--dry-run` mode

## Constraints

- Do NOT modify the Rust core, Android Kotlin, or iOS Swift application code
- Do NOT modify existing CI/CD workflows (`.github/workflows/`)
- DO modify scripts in `scripts/`, `iOS/`, `android/`, and root-level `.sh` files
- DO create new scripts if needed
- DO update documentation in `docs/` to reflect changes
- Follow the project's existing conventions (see `AGENTS.md`, `.roo/rules/`)
- All changes must pass `./scripts/docs_sync_check.sh`

## Deliverables

1. **Analysis Report**: Detailed gap analysis for each step in the lifecycle
2. **Implementation Plan**: Step-by-step instructions for a less-capable model
3. **Hygiene Guidelines**: New standards document for script quality
4. **Updated Scripts**: The actual script improvements (if time permits)

## Reading Order

Before starting, read these files in order:
1. `docs/TESTING_GUIDE.md` — testing pyramid and validation ladder
2. `docs/WS14_AUTOMATION_HANDOFF.md` — automation handoff patterns
3. `scripts/README.md` — script inventory
4. `run5.sh` — 5-node mesh harness
5. `scripts/run5-live-feedback.sh` — live verification loop
6. `scripts/deploy_to_device.sh` — device deployment
7. `scripts/live-smoke.sh` — live smoke test
8. `scripts/check_logs.py` — log analysis
9. `scripts/analyze_mesh.py` — mesh monitoring
10. `scripts/verify_all_builds.sh` — build verification
11. All scripts matching `scripts/verify_*.sh` — deterministic verifiers
12. `log-visualizer/server.mjs` — Sankey visualizer
```

---

## Usage Instructions

1. Copy the prompt above (between the ```text fences)
2. Paste it into a capable reasoning model (GPT-4, Claude, Gemini)
3. Let it think through all phases deeply
4. Review the deliverables it produces
5. Use the implementation plan to guide a less-capable model
6. Iterate until the workflow is bulletproof

---

## Expected Output Structure

The agent should produce:

```
# SCMessenger Script Sanity Check Results

## 1. Mental Model Summary
[Diagram or description of the entire workflow]

## 2. Gap Analysis
### Step 1: Pre-Flight Checks
- Gap 1: [description]
- Gap 2: [description]
...

## 3. Implementation Plan
### Step 1: [Title]
**File**: scripts/example.sh
**Action**: modify
**Rationale**: ...
**Implementation**: ...
**Verification**: ...
**LOC Estimate**: ~30 LOC

## 4. Hygiene Guidelines
[New standards document]

## 5. Summary
[Executive summary of findings and recommendations]
```
