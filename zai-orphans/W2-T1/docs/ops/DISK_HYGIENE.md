# Disk Hygiene Practices

**Status:** Complete
**Last Updated:** 2026-07-26

## Overview

This document captures measured constraints and practices for managing disk space on
development and CI machines. The Windows development drive (237 GB total) regularly
approaches full capacity during multi-gate build sweeps.

## Measured Cost Per Build Gate

Full five-gate sweep (fmt, clippy default, clippy --all-features, test compile-only,
WASM release) regrows `target/` to approximately **40-47 GB**:

- Run 1: reclaimed 42.7 GB from clean
- Run 2: reclaimed 35.7 GB from clean
- Run 3: reclaimed 47.2 GB from clean

Single gate times vary; clippy and cargo-test use separate caches (clippy-driver vs
rustc), and the two clippy feature variants do not share compiled artifacts either.

## Critical Traps

### Trap 1: `cargo clean --target <triple>` Wipes ALL Targets

**Problem:** Specifying `--target x86_64-linux-android` does not scope to that target.
**Evidence:** Intended reclaim was ~4 GB; actual deletion was 44.7 GB (the entire `target/`).

**Fix:** Do not use `--target` flag with `cargo clean`. Only `cargo clean` without
arguments or specify partial paths:
```bash
cargo clean                           # Wipe all
cargo clean --release                 # Wipe release only
cargo clean --doc                     # Wipe generated docs
```

### Trap 2: `cargo clean` Destroys Generated Bindings

**Problem:** `cargo clean` removes `core/target/generated-sources/`, which is a
dependency of `scripts/ffi_surface.sh`. Running that script after `cargo clean`
silently succeeds (reports "Updated ... snapshot") even though bindings are absent.

**Evidence:** Two consecutive CI failures today when `ffi_surface.sh` was run
post-clean.

**Fix:** After any `cargo clean`:
1. Regenerate Swift bindings: `cargo run --manifest-path core/Cargo.toml --bin gen_swift --features gen-bindings`
2. Regenerate Kotlin bindings: `cargo run --manifest-path core/Cargo.toml --bin gen_kotlin --features gen-bindings`
3. Verify files exist: `ls core/target/generated-sources/`
4. Then run `scripts/ffi_surface.sh --update` if needed.

### Trap 3: Multi-Cache Artifact Duplication

**Status:** Confirmed via inspection.
**Hypothesis:** Clippy and cargo build/test use separate artifact caches (clippy-driver
vs rustc), and the two clippy feature variants (default vs --all-features) compile and
cache independently. This results in near-complete duplicate copies of the workspace
in `target/` during a full sweep.

**Mitigation:** This is a cargo/rustc design constraint; no local workaround exists.
Ensure sufficient free space (25 GB minimum) before any full sweep.

## Recommended Routine

### Before a Full Gate Sweep

1. Run `scripts/preflight_disk.sh` (or `.ps1` on Windows).
   - Reports free space and current `target/` size.
   - Exits non-zero if free space < 25 GB.
2. If insufficient space, run `cargo clean` and retry the preflight check.
3. Proceed with build gates.

### When Disk Space Runs Low (< 5 GB free)

1. Run `cargo clean` to free 35-47 GB.
2. Run `cd android && ./gradlew clean` to free an additional 3+ GB.
3. Empty browser cache and `C:\Users\<USER>\AppData\Local\Temp` if needed.

### What NEVER to Do

- Do NOT run `cargo clean --target <triple>` — it deletes all targets, not the one specified.
- Do NOT run `ffi_surface.sh --update` immediately after `cargo clean` without regenerating bindings first.
- Do NOT commit `target/`, `android/app/build/`, `.gradle/`, or generated bindings (they are in `.gitignore`).

## Current Cache Sizes

Measured on 2026-07-26 after a full sweep:

- `target/` (if present): ~1.5-3.1 GB per platform
- `android/app/build/`: 3.1 GB
- `core/target/`: 1.5 GB
- `~/.gradle/`: 3.2 GB (global Android gradle cache)
- `~/.cargo/registry/`: 1.1 GB (global Rust package cache)

## Disk Space State

- Drive size: 237 GB total
- Free space (after preflight): 45.5 GB (18% free)
- Threshold for safe builds: 25 GB minimum

## CI Enforcement

Enforced checks in `.github/workflows/hygiene.yml`:

1. **Tracked File Size Guard:** Warns if any tracked file exceeds 2 MB (excludes
   `Cargo.lock`, generated bindings, and xcframework headers). This prevents
   accidental commits of large artifacts.
2. **Mixed Line Endings:** Warns if any tracked files have mixed CRLF/LF line endings.
3. **Root Layout:** Fails if any `.log`, `.pid`, or `.logcat` files are tracked.

## .gitignore Coverage

All critical patterns are covered:

- `/target/` (line 1, 134)
- `android/app/build/` (line 43)
- `.gradle/` (line 45, 136)
- `dist/` (line 122)
- `tmp/` (line 5, 92)
- `core/target/generated-sources/` (line 54)
- `**/*.log`, `**/*.pid`, `**/*.logcat` (lines 9, 11, 13)
- `local.properties` (line 46, 350)

No gaps found requiring fixes.
