# BATCH: Android UI Components + System Tasks (Priority 3)

Complete all tasks below. Process sequentially. After each task, run `./gradlew :app:compileDebugKotlin --quiet` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

## Task 1: ContactDetailScreen
- File: `HANDOFF/todo/task_wire_ContactDetailScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt`

## Task 2: PeerListScreen
- File: `HANDOFF/todo/task_wire_PeerListScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/mesh/PeerListScreen.kt`

## Task 3: TopologyScreen
- File: `HANDOFF/todo/task_wire_TopologyScreen.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/mesh/TopologyScreen.kt`

## Task 4: WarningBanner
- File: `HANDOFF/todo/task_wire_WarningBanner.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/WarningBanner.kt`

## Task 5: InfoBanner
- File: `HANDOFF/todo/task_wire_InfoBanner.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/InfoBanner.kt`

## Task 6: ErrorState
- File: `HANDOFF/todo/task_wire_ErrorState.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/ErrorState.kt`

## Task 7: IdenticonFromHex
- File: `HANDOFF/todo/task_wire_IdenticonFromHex.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/IdenticonFromHex.kt`

## Task 8: LabeledCopyableText
- File: `HANDOFF/todo/task_wire_LabeledCopyableText.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/LabeledCopyableText.kt`

## Task 9: TruncatedCopyableText
- File: `HANDOFF/todo/task_wire_TruncatedCopyableText.md`
- Target: `android/app/src/main/java/com/scmessenger/android/ui/components/TruncatedCopyableText.kt`

## Task 10: acquireWakeLock
- File: `HANDOFF/todo/task_wire_acquireWakeLock.md`
- Target: `android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`

## Task 11: applyAdvertiseSettings
- File: `HANDOFF/todo/task_wire_applyAdvertiseSettings.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Task 12: applyScanSettings
- File: `HANDOFF/todo/task_wire_applyScanSettings.md`
- Target: `android/app/src/main/java/com/scmessenger/android/transport/BleTransportManager.kt`

## Verification
After all tasks: run `cd android && ./gradlew assembleDebug -x lint --quiet`
Report: STATUS: SUCCESS_STOP or list blockers.
