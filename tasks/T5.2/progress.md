# T5.2 — Remove phantom `mobile` workspace member & fix portability of cargo config

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** T5.1
**Blocks:** T5.3

## Technical Context
- Root `Cargo.toml` `members = ["core", "mobile", "cli", "desktop_bridge", "wasm"]` — `mobile/` does not exist
- `core/.cargo/config.toml` hardcodes `/c/Users/kanal/...NDK...clang.cmd` (Windows path)

## Implementation
1. Drop `mobile` from workspace members
2. Verify `desktop_bridge` exists; drop if phantom
3. Replace hardcoded NDK linker with env-var-driven config (`[env]` + documented `ANDROID_NDK_HOME`)
4. Or move linker selection into `cargo-ndk` invocation documented in scripts

## Verification
- [x] `cargo metadata --format-version 1 > /dev/null` exits 0
- [x] `grep -r "kanal" core/.cargo/` empty
- [x] `cargo check --workspace` passes
