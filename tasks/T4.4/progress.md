# T4.4 — Zero-status UI hardening (honest state surfacing, no fake liveness)

**Status:** completed
**Track:** 4 (Cryptographic Identity, Anti-Entropy & UI Hardening)
**Dependencies:** T5.7
**Blocks:** none

## Technical Context
- `ConnectionPathState` enum already models honesty levels (Disconnected/Bootstrapping/DirectPreferred/RelayFallback/RelayOnly — api.udl:144)
- `ServiceState`; delivery vs sent receipts (`MessageRecord.delivered`)
- Survival-context requirement: UI must NEVER imply connectivity/delivery that isn't cryptographically confirmed

## Implementation
1. Define the canonical message-state machine exposed over FFI: `Queued -> InCustody(mule) | Sent(transport) -> Delivered(receipt verified) -> Read(optional)`
2. Add `MessageStatus` enum to UDL replacing the bare `delivered` bool (keep bool as derived for compat)
3. Custody state explicitly distinct from sent ("being carried by the mesh" != "reached recipient")
4. Swift `ChatViewModel.swift` + Kotlin equivalents render: no checkmark until receipt-verified; explicit "carried by N hops" indeterminate state; `ConnectionPathState.Disconnected` shows mesh-only mode prominently, never a spinner implying imminent internet

## Edge Cases
- Receipt forgery — receipts must be signature-verified against recipient pubkey before flipping Delivered (`on_receipt_received` path at `mobile_bridge.rs` — audit that verification happens in Rust, not trusted from transport)
- Status regression (Delivered never downgrades)
- WASM/CLI parity for the enum

## Verification
- [x] Rust state-machine property test (no illegal transitions, monotone progress)
- [x] Receipt-forgery test: unsigned/wrong-key receipt does NOT flip status
- [x] FFI snapshot updated
- [x] Swift+Kotlin unit tests for render mapping (status -> glyph) committed alongside
