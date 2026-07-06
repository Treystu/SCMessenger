# Quarantined Unit Tests

This directory is NOT a registered Gradle source set — files here are not
compiled or executed. It exists so drifted tests can be tracked (not deleted)
without blocking the rest of the re-enabled unit-test suite.

Context: `./gradlew :app:testDebugUnitTest` was force-disabled from 2026-06-06
to 2026-07-06 (see `HANDOFF/done/P1_ANDROID_Unit_Tests_Force_Disabled_Since_2026-06-06.md`).
When re-enabled, two files no longer compiled against a month of main-source
drift and were moved here. Restoring them is tracked in
`HANDOFF/todo/NEXT_ITER_01_Compile_Gates_And_Test_Triage.md`.

## Files and what they need

1. `IdentityViewModelTest.kt`
   - `IdentityViewModel` moved to `com.scmessenger.android.ui.viewmodels` (test
     package/imports are stale).
   - The ViewModel now takes an `identityCreationCoordinator` dependency and its
     `error` flow comes from the coordinator — the mock setup and the
     `viewModel._error` access are invalid against the current API.
   - `uniffi.api.IdentityInfo` gained `deviceId` and `seniorityTimestamp` params.
   - `ServiceState.INITIALIZING` no longer exists (use `STARTING`).
   - `verify {}` on suspend `createIdentity`/`setNickname` must be `coVerify {}`;
     missing imports (`CompletableFuture`, `delay`).
2. `IdentityCreationFlowTest.kt`
   - A Compose UI test (`createEmptyComposeRule`, semantics matchers) sitting in
     the JVM unit-test source set. It has NEVER compiled here — it needs either
     (a) moving to `androidTest` with the existing compose-test deps, or
     (b) Robolectric + `testImplementation` compose-ui-test deps.
   - This is a structural decision, not a mechanical fix.
