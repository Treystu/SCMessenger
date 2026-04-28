# Transport Failure Analysis - 2026-03-15

## Executive Summary

**Status Update**: Both iOS and Android apps have been rebuilt and reinstalled as clean release builds. iOS installed successfully. Android release build failed due to missing keystore - debug build attempted instead.

## Build Status
- **iOS**: ✅ Built and installed successfully (release configuration)
- **Android**: ⚠️ Release build failed (missing keystore), debug build in progress

## Additional Issues Reported

### Issue 1: Nickname Update Causing Crashes
- **Severity**: HIGH
- **Description**: Updating nickname causes issues - edits try to propagate in real time for each character edit, causing crashes
- **Root Cause**: Real-time character-by-character propagation instead of debounced/batched updates
- **Fix Needed**: Implement debounced nickname update function that waits for user to finish typing before propagating

### Issue 2: Stale Contact After Deletion
- **Severity**: MEDIUM
- **Description**: Stale contact showed up after being deleted on Android
- **Root Cause**: Contact cache not properly invalidated after deletion
- **Fix Needed**: Ensure contact deletion clears all caches and triggers UI refresh

## Log Analysis Findings

### 1. iOS Physical Device (iPhone)
**Status**: ❌ CRASHED (Signal 9 - SIGKILL)
- **Log**: `ios-device.log` shows "App terminated due to signal 9"
- **Impact**: App is completely non-functional, cannot receive or send messages

### 2. iOS Simulator
**Status**: ❌ CRASHED (Signal 15 - SIGTERM)
- **Log**: `ios-sim.log` shows "Child process terminated with signal 15: Terminated"
- **Impact**: Simulator app also non-functional

### 3. Android Device (Pixel 6a)
**Status**: ✅ RUNNING but ❌ TRANSPORT BROKEN
- **Log**: `android.log` shows app running, discovering peers, but all transport methods failing

**Key Android Log Entries**:
```
03-13 04:13:58.886 W MeshRepository: Core-routed delivery failed for 12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198: Network error; trying alternative transports
03-13 04:13:58.887 I MeshRepository: delivery_attempt msg=unknown medium=core phase=direct outcome=failed detail=ctx=send route=12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198 reason=Network error
03-13 04:13:58.896 D MeshRepository: 🔀 Transport: relay-circuit route=12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198 connected=true timeout=1500ms
03-13 04:13:59.898 W MeshRepository: Relay-circuit retry failed for 12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198: Network error
03-13 04:13:59.906 I MeshRepository: delivery_attempt msg=unknown medium=final phase=aggregate outcome=failed detail=ctx=send reason=all_transports_failed ble_only=false
03-13 04:13:58.491 I MeshRepository: delivery_attempt msg=unknown medium=ble phase=local_fallback outcome=skipped detail=ctx=send requested_target=6F:02:6E:60:0B:32 reason=stale_ble_hint_no_fresh_observation
```

### 4. OSX Relay Node
**Status**: ✅ RUNNING
- **Log**: `osx.log` shows relay running, making dial attempts, discovering peers
- **Note**: Connections are being dropped frequently but relay is functional

### 5. GCP Relay Node
**Status**: ✅ RUNNING
- **Log**: `gcp.log` shows relay running, accepting connections, acting as relay server
- **Warning**: "Dropping inbound identify push stream because we are at capacity" - indicates high load

## Transport Failure Chain

```
Android tries to send message to iOS peer (12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198)
    ↓
1. Core Transport (Direct) → FAILED (Network error)
    ↓
2. Relay-Circuit Transport → FAILED (Network error)
    ↓
3. BLE Fallback → SKIPPED (stale_ble_hint_no_fresh_observation)
    ↓
4. Final Result → all_transports_failed
```

## Root Cause Analysis

### Why BLE Fallback Failed
The BLE fallback was skipped with reason `stale_ble_hint_no_fresh_observation`. This means:
- Android has a BLE address hint for the iOS device (6F:02:6E:60:0B:32)
- But this hint is stale - no recent BLE advertisement has been observed from the iOS device
- **Reason**: The iOS app is not running, so it's not advertising BLE beacons

### Why Core/Relay Transports Failed
The "Network error" for core and relay transports suggests:
1. The iOS peer identity is known (discovered via relay network)
2. But the actual iOS device is not reachable
3. Dial attempts to the iOS device's addresses fail because the app is not listening

### Why iOS App Crashed
**iOS Physical Device**: Signal 9 (SIGKILL) - This is typically caused by:
- Memory pressure (iOS killed the app to free memory)
- Background app refresh timeout
- User force-quit the app
- Crash due to unhandled exception

**iOS Simulator**: Signal 15 (SIGTERM) - This is typically caused by:
- Simulator being shut down
- App being terminated by Xcode
- Memory pressure on the Mac

## Critical Issues Identified

### Issue 1: iOS App Not Running
- **Severity**: CRITICAL
- **Impact**: Complete message delivery failure
- **Fix**: Ensure iOS app is running on physical device

### Issue 2: BLE Hints Stale
- **Severity**: HIGH
- **Impact**: BLE fallback cannot be used even if iOS app starts
- **Fix**: Implement BLE hint refresh mechanism or reduce hint staleness timeout

### Issue 3: GCP Relay At Capacity
- **Severity**: MEDIUM
- **Impact**: May cause connection drops under load
- **Fix**: Increase identify stream capacity or add more relay nodes

## Recommended Fixes

### Immediate Actions
1. **Restart iOS App on Physical Device**
   - Open Xcode
   - Build and run SCMessenger on the physical iPhone
   - Verify app stays running

2. **Check iOS App Crash Logs**
   - Open Xcode → Window → Devices and Simulators
   - Select the physical device
   - View crash logs for SCMessenger
   - Identify the crash reason

3. **Verify BLE Advertising**
   - Once iOS app is running, verify BLE beacons are being advertised
   - Check Android log for "BLE GATT identity beacon updated" messages

### Long-term Fixes
1. **Implement BLE Hint Refresh**
   - Add mechanism to refresh BLE hints periodically
   - Reduce staleness timeout for BLE hints
   - Implement BLE reconnection logic

2. **Add Transport Health Monitoring**
   - Monitor transport success rates
   - Alert when all transports fail
   - Implement automatic retry with exponential backoff

3. **Improve Error Handling**
   - Add specific error messages for "peer not reachable"
   - Implement user notification when messages cannot be delivered
   - Add message queue with retry mechanism

## Verification Steps

After restarting iOS app:
1. Check iOS device log for app startup messages
2. Check Android log for BLE beacon observations
3. Send test message from Android to iOS
4. Verify message delivery success in both logs
5. Check that BLE fallback is working (if core transport fails)

## Log Files Reference

- **iOS Device Log**: `logs/5mesh/20260313_041301/ios-device.log`
- **iOS Device System Log**: `logs/5mesh/20260313_041301/ios-device-system.log`
- **iOS Simulator Log**: `logs/5mesh/20260313_041301/ios-sim.log`
- **Android Log**: `logs/5mesh/20260313_041301/android.log`
- **OSX Relay Log**: `logs/5mesh/20260313_041301/osx.log`
- **GCP Relay Log**: `logs/5mesh/20260313_041301/gcp.log`

## Conclusion

The transport is broken because the iOS app is not running. Once the iOS app is restarted and verified to be running, the transport should work. The BLE fallback issue (stale hints) is a secondary problem that may need to be addressed for better reliability.
