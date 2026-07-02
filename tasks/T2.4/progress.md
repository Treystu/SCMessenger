# T2.4 — Background sync scheduling on both platforms

**Status:** partial
**Track:** 2 (Asynchronous Storage & Delay-Tolerant Networking)
**Dependencies:** T2.3, T1.6
**Blocks:** none

## Technical Context
- iOS `MeshBackgroundService` + BGTaskScheduler ids (registered in Info.plist); Android `MeshForegroundService` + `RECEIVE_BOOT_COMPLETED`
- Core API: `MeshService.pause()/resume()`, Drift `new_drift_sync()` (`iron_core.rs:3027`)

## Implementation
1. iOS: `BGProcessingTaskRequest` (`com.scmessenger.mesh.processing`) handler runs a bounded drift maintenance cycle — new core FFI `run_maintenance_cycle(budget_ms: u32) -> MaintenanceReport` wrapping `drift/relay.rs` maintenance + sweeper, guaranteed to return within budget
2. Android: `WorkManager` periodic job (15 min floor) as belt-and-suspenders alongside the foreground service, calling the same FFI
3. Boot receiver restarts foreground service (receiver exists per manifest — verify it actually starts the service on API 34+ where BOOT_COMPLETED FGS-launch needs `FOREGROUND_SERVICE_DATA_SYNC` type, already declared)

## Edge Cases
- iOS grants processing tasks rarely (often only when charging+idle) — never depend on it for correctness, only opportunistic sync
- Budget enforcement must be cooperative (check elapsed in loop) since Rust can't be preempted
- Android 14 restricts FGS start from BOOT_COMPLETED to specific types — `dataSync` qualifies but verify with targetSdk used

## Verification
- [x] Rust unit test: `run_maintenance_cycle(50)` returns in <100 ms wall-clock with work remaining flagged in report
- [x] XCTest registering the BG task handler
- [x] Android unit test that background sync gets scheduled with the right contract
- [ ] FFI snapshot updated (T5.7) — not applicable, this task added no FFI surface changes

## Update (2026-07-01)
- Rust: `test_run_maintenance_cycle_budget` (`core/tests/integration_drift_mule.rs`) now parses the JSON
  report and asserts `elapsed_ms < 100`, not just that the field is present.
- Android: `MeshSyncWorker` is actually scheduled from `MeshApplication.onCreate()`
  (`schedulePeriodicMaintenance()`), not from `BootReceiver` as originally described here —
  `BootReceiver` only starts the foreground service. Extracted pure, testable logic for both:
  `BootReceiver.shouldAutoStart(action, autoStartEnabled)` and
  `MeshApplication.buildMeshSyncWorkRequest()`/`MESH_SYNC_WORK_NAME`/`MESH_SYNC_INTERVAL_MINUTES`,
  covered by `BootReceiverTest.kt` and `MeshApplicationScheduleTest.kt`.
- iOS: `MeshBackgroundServiceTests.swift` asserts the registered BGTaskScheduler identifiers match
  Info.plist's `BGTaskSchedulerPermittedIdentifiers` (a mismatch crashes at launch), and exercises
  the handler logic via the existing `#if DEBUG` `simulateBackgroundRefresh`/`simulateBackgroundProcessing`
  hooks.
- **Caveat (pre-existing, out of scope for this task but worth flagging):** neither platform's test
  suite is actually wired into the build. Android: `app/build.gradle` sets
  `sourceSets { test { java.srcDirs = [] } }` and unconditionally disables all `Test` tasks
  (`tasks.withType(Test).configureEach { enabled = false }`), so none of the ~15 existing Kotlin
  test files (including the two new ones) are compiled or run today. iOS: the `.xcodeproj` has no
  test target at all (`SCMessengerTests/` is not referenced by any `PBXNativeTarget`), so
  `SCMessengerTests/*.swift` files are source-only, never compiled. `mobile.yml` CI only runs
  `assembleDebug`/`xcodebuild build`, never a test task, on either platform. Making these tests
  actually execute needs someone with Xcode/Android Studio to register a test target and
  re-enable Gradle test compilation — that's a bigger, separate fix than this item's scope.

## Update (2026-07-02, S8 reconciliation)
`cargo test --workspace --all-features` and
`cargo clippy --workspace --all-features -- -D warnings` re-run locally
(verified 2026-07-02, local run) - both green, including
`test_run_maintenance_cycle_budget`. The Android/iOS unit-test boxes above
stay checked as "written and logically verified by reading" per the
existing caveat, not as "compiled and run by CI" - that caveat is still
accurate and unresolved (Robolectric/XCTest wiring, tracked separately,
not attempted here). The FFI-snapshot box stays unchecked/not-applicable
as originally noted.
