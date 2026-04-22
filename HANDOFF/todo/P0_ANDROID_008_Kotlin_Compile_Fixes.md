# P0_ANDROID_008_Kotlin_Compile_Fixes

**Priority:** P0
**Type:** BUILD
**Platform:** Android
**Estimated Effort:** 2–4 hours

## Objective
Fix 30+ Kotlin compilation errors preventing `./gradlew :app:compileDebugKotlin` from passing. These errors block the Android build entirely.

## Background
The Android project has 225 source files but 30+ compilation errors across 6 files. The errors fall into four categories:

1. **Suspend functions called from non-suspend contexts** (12 errors)
   - `MeshRepository.kt:260` — `relayCircuitBreaker.allowRequest(addr)` called in regular function
   - `MeshRepository.kt:2368` — `bleScanner?.startScanning()` called in regular function
   - `MeshRepository.kt:2912` — `bleScanner?.stopScanning()` called in regular function
   - `TransportManager.kt:91, 116, 367, 391, 436, 460` — `startScanning` / `stopScanning`
   - `BleScanner.kt:164, 174, 188, 206, 207, 272, 319, 484, 505` — suspend calls inside non-suspend callbacks

2. **Coroutine scope / lambda mismatches** (6 errors)
   - `MeshRepository.kt:6902` — `it[1].ifEmpty { _ -> it[2] }` has a parameter `_` but `String.ifEmpty` expects `() -> String`
   - `MeshRepository.kt:6918` — `async` called without a `CoroutineScope` receiver inside `map` lambda
   - `MeshRepository.kt:6935` — `await()` / `isCompleted` / `cancel()` scope issues
   - `NetworkDetector.kt:278` — `kotlinx.coroutines.async` unresolved because the `map` lambda does not have a `CoroutineScope` receiver (needs `this@coroutineScope.async`)

3. **Missing Compose Material 2 imports** (7 errors)
   - `ContactsScreen.kt:278–312` — `SwipeToDismiss`, `DismissValue`, `rememberDismissState` are unresolved
   - These APIs are from `androidx.compose.material` (Material 2), but the file only imports `androidx.compose.material3.*`. In Material 3 the API is `SwipeToDismissBox` / `rememberSwipeToDismissBoxState`. Either add Material 2 imports or migrate to Material 3 APIs.

4. **Unresolved reference `getMeshStats`** (1 error)
   - `MeshRepository.kt:6968` — `getMeshStats` is not found. Check if this method exists in the Rust UniFFI bindings or if it was renamed/removed.

## Constraints
- ZERO REGRESSIONS: Every change must preserve runtime behavior
- Do NOT change the Rust core or UniFFI bindings unless specifically required for `getMeshStats`
- Prefer `suspend` function annotations over `lifecycleScope.launch` when the caller is already in a coroutine context
- For `BleScanner.kt` callbacks, the scanning APIs are inherently asynchronous — consider converting the callback interfaces to suspend functions or wrapping calls with `suspendCoroutine`
- Material 3 migration is preferred over Material 2 fallback if it doesn't change UX

## Verification Checklist
- [ ] `./gradlew :app:compileDebugKotlin` passes with zero errors
- [ ] `./gradlew :app:assembleDebug` produces APK successfully
- [ ] No new warnings introduced (or document each new warning)
- [ ] All existing Kotlin tests pass (`./gradlew :app:testDebugUnitTest`)

## Rollback
If any test fails after a change: `git restore` the file and re-examine the approach.

[NATIVE_SUB_AGENT: RESEARCH] — Use native sub-agents to identify the exact line/column of each error and whether the caller is truly non-suspend or just missing a `suspend` modifier.
[NATIVE_SUB_AGENT: LINT_FORMAT] — Use native sub-agents to format Kotlin files after edits and run Android Lint to verify no new issues.