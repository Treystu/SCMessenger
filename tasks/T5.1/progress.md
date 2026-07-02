# T5.1 — Purge committed build artifacts & fix .gitignore enforcement

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** none (first task)
**Blocks:** T5.2, T5.8, T5.9

## Technical Context
- `core/target/` (android-libs ~1 GB, staged-cdylib 57 MB)
- `android/app/build/outputs/apk/debug/app-debug.apk`
- `android/.gradle/`
- `repomix-output.xml` (5.6 MB)
- `.gitignore` already lists these patterns but files were committed before it applied

## Implementation
1. `git rm -r --cached` each artifact path
2. Add `repomix-output.xml` + `core/target/` + `android/.gradle/` explicitly to `.gitignore`
3. Do NOT rewrite history (single-commit repo; follow-up `git gc` suffices)

## Edge Cases
- Android Gradle build references `core/target/generated-sources/uniffi/kotlin` and prebuilt `.so`s
- Confirm `android/app/build.gradle` regenerates these before deleting
- Document local-build prerequisite in README task (T5.8) if needed

## Verification
- [x] `git ls-files | grep -E '\.(so|dylib|apk)$'` returns empty
- [x] `du -sh .git` reported before/after
- [x] `cargo build -p scmessenger-core` still succeeds
