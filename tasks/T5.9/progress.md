# T5.9 — Resolve duplicate PlatformBridge trait (G4)

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** T5.1
**Blocks:** none

## Technical Context
- Live trait: `mobile_bridge.rs:1436` (UniFFI-exported, used by both apps)
- Legacy: `mobile/service.rs:87` + `platform/service.rs` (only consumer is its own `MockPlatformBridge` tests)

## Implementation
1. Confirm via grep that nothing outside `mobile/` + `platform/` consumes the legacy trait
2. Delete or `#[deprecated]`-and-quarantine the legacy `mobile/service.rs` service path
3. Keep `mobile/auto_adjust.rs`, `ios_strategy.rs` which are referenced by the live bridge
4. If deletion ripples, minimum bar: rename legacy trait to `LegacyPlatformBridge` so agents can't wire the wrong one

## Edge Cases
- `MeshService` in `mobile/service.rs` vs the UniFFI `MeshService` in `mobile_bridge.rs:153` are different types with the same name — ensure `lib.rs` re-exports stay unambiguous

## Verification
- [x] `cargo test --workspace` passes
- [x] `grep -rn "trait PlatformBridge" core/src` yields exactly one non-deprecated definition
