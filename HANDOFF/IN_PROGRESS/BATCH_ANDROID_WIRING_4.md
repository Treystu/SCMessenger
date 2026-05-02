# BATCH: Android BLE + Transport + Foreground Service (Priority 4)

Complete all tasks below. Process sequentially. After each task, run `./gradlew :app:compileDebugKotlin --quiet` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

## Task 1: handleScanFailure
- File: `HANDOFF/todo/task_wire_handleScanFailure.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 2: onStartSuccess
- File: `HANDOFF/todo/task_wire_onStartSuccess.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 3: onStopDiscoveryFailed
- File: `HANDOFF/todo/task_wire_onStopDiscoveryFailed.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 4: startAll
- File: `HANDOFF/todo/task_wire_startAll.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`

## Task 5: is_ble_available
- File: `HANDOFF/todo/task_wire_is_ble_available.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 6: try_enable_bluetooth
- File: `HANDOFF/todo/task_wire_try_enable_bluetooth.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 7: sendBlePacket
- File: `HANDOFF/todo/task_wire_sendBlePacket.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 8: scan_for_advertisements
- File: `HANDOFF/todo/task_wire_scan_for_advertisements.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 9: updateBatteryFloor
- File: `HANDOFF/todo/task_wire_updateBatteryFloor.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Task 10: updateContactDeviceId
- File: `HANDOFF/todo/task_wire_updateContactDeviceId.md`
- Target: `android/app/src/main/java/com/scmessenger/android/data/ContactRepository.kt`

## Task 11: clearSearch
- File: `HANDOFF/todo/task_wire_clearSearch.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/search/SearchViewModel.kt`

## Task 12: buildForegroundServiceNotification
- File: `HANDOFF/todo/task_wire_buildForegroundServiceNotification.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Verification
After all tasks: run `cd android && ./gradlew assembleDebug -x lint --quiet`
Report: STATUS: SUCCESS_STOP or list blockers.
