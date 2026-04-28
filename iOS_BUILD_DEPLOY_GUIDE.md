# iOS Build and Deployment Guide

## Current Status (2026-04-20)

**Task:** P0_IOS_001 - Field iOS Binary Deployment

**Problem:** Field iOS binary version is stale (v0.2.0 build 4) vs current source hardening with WS12.22+ crash fixes.

## Environment Requirements

To build and deploy the latest iOS binary, you need:

| Requirement | Details |
|-------------|---------|
| macOS | macOS 14+ (Sonoma) recommended |
| Xcode | Xcode 15.4+ with iOS 17.0+ SDK |
| Apple ID | Developer Program membership required |
| Physical Device | iOS 17+ device for field testing |

## Build Instructions

### Step 1: Generate UniFFI Bindings

```bash
cd /path/to/SCMessenger

# Build the Rust mobile library
cargo build --target aarch64-apple-ios --release

# Copy bindings to iOS project
bash iOS/copy-bindings.sh
```

### Step 2: Build the iOS App

```bash
# Set your Apple Team ID (get from Apple Developer portal)
export APPLE_TEAM_ID="YOUR_TEAM_ID_HERE"

# Build for device
bash iOS/build-device.sh
```

### Step 3: Deploy to Physical Device

```bash
# Get your device UDID
xcrun devicectl list devices

# Deploy with install script
export DEVICE_UDID="YOUR_DEVICE_UDID"
bash iOS/install-device.sh
```

### Step 4: Alternative - Xcode GUI Build

1. Open `iOS/SCMessenger/SCMessenger.xcodeproj` in Xcode
2. Select "Generic iOS Device" as target
3. Sign the project with your Apple Developer account
4. Click "Product > Build" or "Product > Archive"
5. Use "Distribute App" to export for testing

## WS12.22+ Crash Fixes Included

The source code includes the following critical fixes:

### IOS-CRASH-001: SIGTRAP in BLE Peripheral Send Path
- **File:** `iOS/SCMessenger/SCMessenger/Transport/BLEPeripheralManager.swift`
- **Fix:** Added `peripheralManager.state == .poweredOn` guard before every `updateValue` call
- **Lines:** 71, 312-314, 624-625

### IOS-PERF-001: CPU Watchdog Kill Under Retry Pressure  
- **File:** `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- **Fix:** Added `Task.yield()` in outbox flush loop
- **Line:** 4676

### LOG-AUDIT-001: Retry Storm Fix
- **File:** `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- **Fix:** Exponential backoff (1s → 2s → 4s → 8s → 16s → 32s cap) and circuit breaker
- **Lines:** 134-152, 4005-4086

## Verification Steps

After deploying to field devices:

1. **Launch the app** - Verify no crash on startup
2. **Send test message** - Verify message delivery works
3. **Monitor for 24 hours** - Check for crashes in Settings > Diagnostics
4. **Capture logs** - Use `ios_extractor.py` to extract diagnostic logs

## Current Source Code Status

| Check | Status |
|-------|--------|
| Rust core build | ✅ Pass (dev profile) |
| iOS source files | ✅ All present with WS12.22+ fixes |
| UniFFI bindings | ✅ Generated in `core/target/aarch64-apple-ios/debug/uniffi/` |
| Xcode project | ✅ Available at `iOS/SCMessenger/SCMessenger.xcodeproj` |

## Manual Build Summary (for field technician)

```
On macOS machine:

1. Clone repo: git clone <this-repo>
2. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
3. Add iOS target: rustup target add aarch64-apple-ios
4. Build Rust lib: cargo build --target aarch64-apple-ios --release
5. Copy bindings: bash iOS/copy-bindings.sh
6. Build iOS: APPLE_TEAM_ID=<YOUR_TEAM_ID> bash iOS/build-device.sh
7. Install: bash iOS/install-device.sh (with device connected)
```

## Next Steps for Field Testing

1. Build binary using instructions above
2. Deploy to physical iOS devices (minimum 3 for representative testing)
3. Run comprehensive testing matrix:
   - BLE connectivity
   - Relay connections
   - WiFi Direct transfers
   - Cross-platform messaging (iOS ↔ Android)
4. Capture crash-free evidence logs
5. Update MASTER_BUG_TRACKER.md with deployment completion

## Notes

- The iOS app requires iOS 17.0+ (deployment target set in Info.plist)
- The current version string is 0.2.1 (build 5) as per Info.plist
- All crash fixes from WS12.22+ are present in the source code
- No OTA distribution available - requires physical connection for deployment
