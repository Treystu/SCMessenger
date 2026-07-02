# T5.7 — UniFFI surface contract test (FFI stability gate)

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** T5.4, T5.5
**Blocks:** T1.1, T4.2, T4.4

## Technical Context
- `core/src/api.udl` + proc-macro exports in `mobile_bridge.rs`/`contacts_bridge.rs`/`blocked_bridge.rs`
- uniffi 0.31. Breaking the surface silently breaks both apps.

## Implementation
1. Snapshot test: check in a canonical copy of the generated `api.kt` and `SCMessengerCore.swift` public-symbol list
2. Extract `fun |class |interface |enum ` signatures via a small script `scripts/ffi_surface.sh` (not full file — symbol list only)
3. CI job diffs freshly generated surface against snapshot and fails on unapproved change
4. Update procedure documented in the script header

## Edge Cases
- uniffi version bumps regenerate cosmetically different code — symbol-list extraction (not byte diff) makes the gate robust
- Two `PlatformBridge` traits exist (G4) — snapshot only the UniFFI one

## Verification
- [x] CI fails when an agent adds/removes/renames any exported fn/record/enum without updating the snapshot
- [x] Passes on no-op rebuild

## Update (2026-07-01)
Found two real bugs during a re-audit: (1) `extract_swift_symbols` only matched
`public func`/`public class`/etc, silently missing all 231 `open func` lines
UniFFI emits for instance methods on exported objects (including
`runMaintenanceCycle`) - fixed the regex to also match `open func `. (2) the
`ffi-surface` CI job only ran `gen_kotlin`, so `swift-symbols.txt` was never
regenerated or diffed in CI at all - added a `gen_swift` step so both
surfaces are actually gated on every push/PR. Both snapshots regenerated and
verified clean (`scripts/ffi_surface.sh` exits 0).
