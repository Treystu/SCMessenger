# P0_ANDROID_008: Kotlin Compile Fixes

**Priority:** P0 (Build Blocking)
**Platform:** Android
**Estimated LoC Impact:** 150–250 LoC
**Source:** PRODUCTION_ROADMAP_PRIORITIZED.md — P0 Blocker #1

## Objective
Fix 30+ Kotlin compilation errors preventing `./gradlew :app:compileDebugKotlin` from passing.

## Error Inventory

### 1. Suspend Function in Non-Suspend Context
**Files:** `MeshRepository.kt`, `TransportManager.kt`
**Errors:** 10+ locations
- `MeshRepository.kt:260, 2368, 2912`
- `TransportManager.kt:91, 116, 367, 391, 436, 460`
**Fix:** Mark caller as `suspend` or wrap in `lifecycleScope.launch`

### 2. Suspend Function in Non-Suspend Callback
**File:** `BleScanner.kt`
**Errors:** 9 locations (164, 174, 188, 206, 207, 272, 319, 484, 505)
**Fix:** Use `suspend` lambdas or `CoroutineScope.launch`

### 3. Lambda Parameter Syntax
**File:** `MeshRepository.kt:6902`
**Error:** `String.ifEmpty` lambda with `_` parameter
**Fix:** `it[1].ifEmpty { it[2] }` (remove `_` param)

### 4. Async Scope Issues
**File:** `MeshRepository.kt:6918, 6935`
**Error:** `async` without CoroutineScope receiver, `await()` scope issues
**Fix:** Add `coroutineScope { async { ... } }`

### 5. Material 2 vs Material 3
**File:** `ContactsScreen.kt:278–312`
**Error:** `SwipeToDismiss` in Material 3 project
**Fix:** Add `androidx.compose.material:material` import OR migrate to `SwipeToDismissBox`

### 6. UniFFI Binding Gap
**File:** `MeshRepository.kt:6968`
**Error:** `getMeshStats` unresolved
**Fix:** Check Rust UniFFI bindings — may need to expose `mesh_stats()` in Rust core

## Verification Checklist
- [ ] `./gradlew :app:compileDebugKotlin` passes with zero errors
- [ ] `./gradlew :app:assembleDebug` produces APK
- [ ] No regressions in existing functionality

## Files to Modify
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ContactsScreen.kt`
- Potentially: `core/src/lib.rs` (UniFFI bindings)

## Rollback
`git restore` if build breaks native targets.

[NATIVE_SUB_AGENT: RESEARCH] — Map all suspend/non-suspend boundaries before editing.
