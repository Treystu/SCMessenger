# Agent Task: Android Gradle Build Verification

**Model:** gemma4:31b:cloud (lightweight)
**Priority:** P1 — Needed for Android deployment

## Context
The SCMessenger Android build needs verification. The core Rust crate compiles cleanly (`cargo check --workspace` passes). The build.gradle already has the GNU target fix for UniFFI binding generation.

## Steps

1. Set environment: `export ANDROID_HOME="$LOCALAPPDATA/Android/Sdk"`
2. Run: `cd android && ./gradlew assembleDebug -x lint --quiet`
3. If the build fails, read the error output and fix the issue
4. Key known issues:
   - If `cargo ndk` fails, check that `ANDROID_NDK_HOME` points to the NDK dir
   - If `dlltool` is not found, set `PATH="/c/msys64/mingw64/bin:$PATH"` before building
   - If UniFFI binding generation fails, the build.gradle already uses `--target x86_64-pc-windows-gnu`
5. On success, report the APK path and size

## File Domains
- `android/app/build.gradle`
- `android/app/src/`
- `android/build.gradle`

## Completion
Write COMPLETION marker to `.claude/agents/<your_id>/COMPLETION` with STATUS, CHANGED_FILES, BUILD_STATUS.