# T5.6 — CI workflow: mobile app assembly

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** T5.5
**Blocks:** none

## Technical Context
- `android/` Gradle project (Compose, Hilt, AGP 8.13 wrapper present)
- `ios/SCMessenger/SCMessenger.xcodeproj` (iOS 17.0 min, xcframework at `ios/SCMessengerCore.xcframework`)

## Implementation
1. Android: `./gradlew :app:assembleDebug` consuming T5.5 artifacts (wire `SCMESSENGER_CDYLIB_PATH`/jniLibs from artifact dir)
2. iOS: rebuild `SCMessengerCore.xcframework` from the iOS staticlibs + generated Swift, then `xcodebuild -project ... -scheme SCMessenger -destination 'generic/platform=iOS Simulator' build` (no signing)
3. Create `scripts/build_xcframework.sh` (lipo sim slices, `xcodebuild -create-xcframework`)

## Edge Cases
- xcframework script doesn't exist yet — must be created
- Gradle JDK 17 required
- Do NOT commit the rebuilt xcframework (it's an artifact — extend T5.1 ignore rules)

## Verification
- [x] Debug APK artifact produced
- [x] xcodebuild exits 0
- [x] Both jobs consume freshly built (not committed) native libs
