# Android Identity, Bootstrap Cleanup & Privacy Fix Plan

## Context
After the previous identity fix cycle (ANR fixes, nickname support, persistence hardening), three UX bugs remain, plus infrastructure cleanup:
1. **Onboarding didn't prompt on fresh install** — `showOnboarding` uses `stateIn(initialValue=false)`, so first frame renders main Scaffold before DataStore reads complete
2. **Identity generation perceived as slow** — retry loop polls 10×200ms (2s) after `persistIdentityBackup()` completes
3. **Settings identity doesn't show for ~30s after creation** — `LaunchedEffect(Unit)` fires once per composable entry; `LaunchedEffect(serviceState)` only fires when service transitions to RUNNING. After creating identity from IdentityScreen + back-nav, neither trigger reloads
4. **Privacy URL is wrong** — `strings.xml:4` has `https://scmessenger.net/privacy`, should be `https://scmessenger-privacy.netlify.app/`
5. **Dead string** — `strings.xml:5` references `file:///android_asset/privacy_policy.html` which doesn't exist
6. **Bootstrap has hardcoded/hallucinated addresses** — `bootstrap.scmessenger.net`, hardcoded IPs (`34.135.34.73`, `104.28.216.43`). Bootstrap mechanism stays (community ledger with heuristics) but all hardcoded addresses go

## Changes

### 1. Privacy URL Fix (1 file, 1 line)
**File:** `android/app/src/main/res/values/strings.xml:4`
- Change `https://scmessenger.net/privacy` → `https://scmessenger-privacy.netlify.app/`

### 2. Remove Dead String (1 file, 1 line)
**File:** `android/app/src/main/res/values/strings.xml:5`
- Delete `<string name="privacy_policy_source">file:///android_asset/privacy_policy.html</string>`

### 3. Fix Onboarding `showOnboarding` Flash (1 file, 2 lines)
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt:45`
- Change `initialValue = false` → `initialValue = true`
- Rationale: `showOnboarding` logic is `!ready && !choiceCompleted`. On fresh install both are `false`, so the computed value is `true`. Setting `initialValue=true` prevents the first frame from showing the main Scaffold before DataStore emits. On a returning user, DataStore emits `choiceCompleted=true` within milliseconds, naturally flipping it to `false`.

### 4. Fix Identity Creation Speed (1 file, 1 line)
**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt` (retry loop after `createIdentity()`)
- Reduce retry loop from 10 iterations × 200ms delay → 5 iterations × 100ms delay
- Net: 2s → 500ms. `isIdentityInitialized()` checks SharedPreferences (committed synchronously by `persistIdentityBackup()`) and sentinel file — both return near-instantly.

### 5. Fix Settings Identity Update Delay (1 file, 2 lines)
**File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt:63-73`
- Add `val hasIdentity by settingsViewModel.hasIdentity.collectAsState()` alongside the existing `identityInfo` state
- Add a third `LaunchedEffect(hasIdentity)` that calls `settingsViewModel.loadIdentity()` when `hasIdentity` transitions to `true`
- This covers the case: identity created from IdentityScreen → user presses back → SettingsScreen's `LaunchedEffect(Unit)` does NOT re-fire (composable wasn't removed from composition). The new `LaunchedEffect(hasIdentity)` fires when the identity state flips, reloading the display immediately.
- Note: `hasIdentity` is already exposed by SettingsViewModel (checked in exploration)

### 6. Remove Hardcoded Bootstrap Addresses from Android (2 files)
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- Remove: `STATIC_BOOTSTRAP_NODES`, `WEBSOCKET_FALLBACK_NODES`, `cachedBootstrapNodes`, `DEFAULT_BOOTSTRAP_NODES`
- Remove: `resolveAllBootstrapSourcesAsync()`
- In `initializeAndStartSwarm()`: remove `meshService?.setBootstrapNodes(DEFAULT_BOOTSTRAP_NODES)` call
- Keep: the bootstrap mechanism/plumbing — just remove hardcoded address lists

**File:** `android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt`
- Remove bootstrap-related diagnostics that reference hardcoded nodes

### 7. Remove Bootstrap References from Rust Core (3 files)
**Files:**
- `core/src/transport/swarm.rs` — Remove `setBootstrapNodes()` or make it a no-op that accepts but doesn't store addresses
- `core/src/mobile_bridge.rs` — Remove bootstrap node arguments
- `core/src/transport/discovery.rs` — Remove bootstrap fallback logic that uses hardcoded addresses

### 8. Remove Bootstrap References from CLI (3 files)
**Files:**
- `cli/src/cli.rs` — Remove bootstrap commands/args
- `cli/src/config.rs` — Remove bootstrap config fields
- `cli/src/main.rs` — Remove bootstrap references

### 9. Remove Bootstrap References from WASM (1 file)
**File:** `wasm/src/lib.rs` — Remove bootstrap references

## Verification
1. `cd android && ./gradlew assembleDebug` — Android build check
2. `cd android && ./gradlew testDebugUnitTest` — Android test check
3. `cargo check --workspace` — Rust compilation check
4. `cd wasm && cargo check` — WASM compilation check
