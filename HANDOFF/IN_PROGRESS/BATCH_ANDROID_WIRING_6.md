# BATCH: Android Service + Transport + System (Priority 6)

Complete all tasks below. Process sequentially. After each task, run `./gradlew :app:compileDebugKotlin --quiet` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Task 1: acquireWakeLock
- File: `HANDOFF/todo/task_wire_acquireWakeLock.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Task 2: applyAdvertiseSettings
- File: `HANDOFF/todo/task_wire_applyAdvertiseSettings.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 3: applyScanSettings
- File: `HANDOFF/todo/task_wire_applyScanSettings.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 4: handleScanFailure
- File: `HANDOFF/todo/task_wire_handleScanFailure.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 5: onStartSuccess
- File: `HANDOFF/todo/task_wire_onStartSuccess.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 6: onStopDiscoveryFailed
- File: `HANDOFF/todo/task_wire_onStopDiscoveryFailed.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 7: startAll
- File: `HANDOFF/todo/task_wire_startAll.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`

## Task 8: is_ble_available
- File: `HANDOFF/todo/task_wire_is_ble_available.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 9: try_enable_bluetooth
- File: `HANDOFF/todo/task_wire_try_enable_bluetooth.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 10: sendBlePacket
- File: `HANDOFF/todo/task_wire_sendBlePacket.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 11: scan_for_advertisements
- File: `HANDOFF/todo/task_wire_scan_for_advertisements.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 12: updateBatteryFloor
- File: `HANDOFF/todo/task_wire_updateBatteryFloor.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Verification
After all tasks: run `cd android && ./gradlew assembleDebug -x lint --quiet`
Report: STATUS: SUCCESS_STOP or list blockers.
