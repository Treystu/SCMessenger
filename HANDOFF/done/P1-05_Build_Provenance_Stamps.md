# P1-05: Build-provenance stamps

**Priority:** P1
**Recommended worker:** Qwen-Plus (Orchestrator)
**Source:** Stage A Execution Plan

## Spec
Small Rust fn (env!/option_env via build.rs) surfaced in CLI `--version` + startup log + Android BuildConfig/About. Prevents the P1-04 artifact-skew class entirely.

## Execution
1. Modified `core/build.rs` to generate `SCM_BUILD_STAMP` capturing `git rev-parse --short HEAD`, `git rev-parse --abbrev-ref HEAD`, and a Unix timestamp.
2. Exported `string get_build_provenance()` via `core/src/api.udl` and implemented it in `core/src/lib.rs`.
3. Updated `cli/src/main.rs` to log `scmessenger_core::get_build_provenance()` during the CLI startup process.
4. Added `getBuildProvenance()` wrapper in `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` and `SettingsViewModel.kt`.
5. Displayed the Core Provenance stamp within the Settings > Info > Version UI `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt`.

## Completion
- All builds passed (Windows core crate + Android assembleDebug).
- Swarm worker Qwen-Plus contributed to file generation.
