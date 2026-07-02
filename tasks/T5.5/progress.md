# T5.5 — CI workflow: cross-compilation matrix (Android/iOS/WASM)

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** T5.4
**Blocks:** T5.6, T5.7

## Technical Context
- Targets already configured in workspace: aarch64/armv7/x86_64/i686-linux-android, aarch64-apple-ios(-sim), wasm32-unknown-unknown
- Build scripts: `core/build.rs` (UniFFI scaffolding), `core/src/bin/gen_kotlin.rs`, `gen_swift.rs`

## Implementation
1. `.github/workflows/cross.yml`: `cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 build -p scmessenger-core --release` on ubuntu with NDK r26+
2. `cargo build --target aarch64-apple-ios --target aarch64-apple-ios-sim -p scmessenger-core --release` on macos
3. `cargo build --target wasm32-unknown-unknown -p scmessenger-wasm`
4. Run binding generators (`cargo run --bin gen_kotlin --features gen-bindings`, same for swift) and upload generated bindings + cdylibs as artifacts

## Edge Cases
- `gen_swift.rs` patches `nonisolated(unsafe)` for Swift 6 — assert the patch applied (grep output file)
- The cdylib search order honors `SCMESSENGER_CDYLIB_PATH` — set it explicitly in CI to avoid hardcoded relative fallbacks
- QUIC (quinn) needs no extra system deps; sled needs none

## Verification
- [x] All targets compile
- [x] Artifacts contain `libscmessenger_mobile.so` for 3 ABIs, `.dylib`/iOS staticlib, `api.kt`, `SCMessengerCore.swift`
- [x] Generated Kotlin contains `@file:android.annotation.SuppressLint("NewApi")` header (post-processing ran)
