---
task_id: "MICRO_004"
priority: "P0"
assigned_agent: "triage-router"
model: "gemini-3-flash-preview:cloud"
token_budget: 200
time_limit_ms: 60000
phase: "MICRO"
---

# MODEL: gemini-3-flash-preview:cloud
# BUDGET: 60

# MICRO_004: Install Missing Windows GNU Rust Target

## Objective
Install the `x86_64-pc-windows-gnu` Rust target to immediately unblock the Android UniFFI binding generation, which hardcodes this target in `build.gradle`.

## Background
The Android build fails at `:app:generateUniFFIBindings` with:
```
error[E0463]: can't find crate for `core`
  = note: the `x86_64-pc-windows-gnu` target may not be installed
```
The active toolchain is `1.95.0-x86_64-pc-windows-msvc`. The `x86_64-pc-windows-gnu` target is missing.

## Exact Command
```bash
rustup target add x86_64-pc-windows-gnu
```

## Verification
```bash
rustup target list --installed | grep x86_64-pc-windows-gnu
```
Output must contain `x86_64-pc-windows-gnu`.

## Success Criteria
- [ ] `rustup target add x86_64-pc-windows-gnu` completes without error
- [ ] `rustup target list --installed` confirms the target is present
- [ ] No code changes required

## Failure Protocol
If the command fails (e.g., network error, toolchain mismatch), report the exact error output. Do NOT attempt alternative fixes.

## Note
This is an immediate unblocker. A follow-up task (`FIX_ANDROID_BUILD_001`) will align the build scripts with the MSVC toolchain to eliminate the GNU dependency entirely.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` command to move this task markdown file from `HANDOFF/todo/` to `HANDOFF/done/`. If you do not move the file, the Orchestrator assumes you failed.
