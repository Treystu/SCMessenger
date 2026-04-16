# P0_SECURITY_006: Consent Gate Enforcement

**Priority:** P0 (Security/Compliance)
**Platform:** Core/Rust
**Status:** ALREADY IMPLEMENTED
**Verified:** 2026-04-15

## Findings

The consent gate is **already enforced** at the Rust core API level:

1. **`core/src/lib.rs` line 944-946**: `initialize_identity()` checks `if !*self.consent_granted.read()` and returns `IronCoreError::ConsentRequired` if consent hasn't been granted.
2. **`core/src/lib.rs` line 932-936**: `grant_consent()` sets the `consent_granted` flag and emits an audit event.
3. **`core/src/lib.rs` line 938-941**: `is_consent_granted()` provides a query method.
4. **`core/src/api.udl`**: Both `grant_consent()` and `is_consent_granted()` are exposed via UniFFI.
5. **`core/src/lib.rs` line 297**: `consent_granted: Arc<RwLock<bool>>` is stored on the `IronCore` struct and initialized to `false`.
6. **Test coverage** (lines 2583-2596): Confirms that `initialize_identity()` fails with `ConsentRequired` before consent is granted.

No additional implementation is needed. The task description stated "consent gate is implemented at UI level but NOT enforced at Rust core API level" — this is **incorrect**. The enforcement is already in place.

## No Changes Required