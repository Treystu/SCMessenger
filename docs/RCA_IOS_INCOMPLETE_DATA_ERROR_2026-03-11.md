# RCA: iOS API Fatal Error - `incompleteData`

Status: Active
Last updated: 2026-03-11
RCA ID: RCA-IOS-001

## 1. Issue Description

**Error:** `SCMessenger/api.swift:1454: Fatal error: 'try!' expression unexpectedly raised an error: SCMessenger.(unknown context at $10ef60f44).UniffiInternalError.incompleteData`

**Symptom:** The iOS application crashes immediately when calling `getIdentityInfo()` on the `IronCore` instance.

## 2. Root Cause Analysis

### A. Context: WS13.1 Tight-Pairing Implementation
On 2026-03-10, architectural changes were introduced to the Rust core to support **WS13.1 (Tight-Pairing)**. This involved adding new fields to the `IdentityInfo` struct and changing the signature of `SwarmBridge::send_message`.

### B. The Defect
The `scmessenger-core` Rust library was updated, and the `api.udl` file was modified to include:
- `device_id: string?`
- `seniority_timestamp: u64?`
inside the `IdentityInfo` dictionary.

However, the **iOS Swift bindings** (`iOS/SCMessenger/SCMessenger/Generated/api.swift`) were **not regenerated** in that session because the environment where the core changes were landed lacked the necessary toolchains (`xcodebuild`, etc.) to verify the iOS build.

### C. Technical Breakdown
1.  **Binary Mismatch:** The compiled Rust library (XCFramework) used by the iOS app contained the new `IdentityInfo` layout (with 7 fields).
2.  **Swift Binding Drift:** The existing `api.swift` file expected the old `IdentityInfo` layout (with 5 fields).
3.  **UniFFI Deserialization Failure:** When `getIdentityInfo()` was called, the Rust side returned a `RustBuffer` containing the serialized 7-field struct. The Swift `FfiConverterTypeIdentityInfo.read` method only consumed 5 fields from the buffer.
4.  **Incomplete Data:** Because the Swift side finished reading before the buffer was empty (or vice-versa, depending on how UniFFI detects it), it raised `UniffiInternalError.incompleteData`.
5.  **Fatal Crash:** The call site in `api.swift` used `try!`, which turned this error into a fatal crash.

## 3. Remediation

1.  **Bindings Regeneration:** Re-ran the Swift binding generation using `cargo run --bin gen_swift --features gen-bindings` on a Mac host.
2.  **Layout Alignment:** The newly generated `api.swift` correctly includes headers and converters for all 7 fields of `IdentityInfo`.
3.  **Sync-to-Project:** Copied the regenerated `api.swift`, `apiFFI.h`, and `apiFFI.modulemap` to the iOS project directory.
4.  **Verification:** Validated the iOS build using `xcodebuild` to ensure the generated bindings are compatible with the current Swift project settings.

## 4. Prevention

1.  **Mandatory Regeneration Gate:** Any change to `api.udl` MUST trigger a regeneration of both Android (Kotlin) and iOS (Swift) bindings.
2.  **Automated Binding Verification:** Added `scripts/verify_ios_bindings.sh` to cross-check the UDL definition against the generated Swift code to detect drift before a crash occurs at runtime.
3.  **Swift 6 Compatibility:** Updated the `gen_swift` binary to handle `nonisolated(unsafe)` more robustly, ensuring all `FfiConverter` implementations stay compatible with the project's strict concurrency settings.

## 5. Verification Status

- [x] RCA Documented
- [x] Swift Bindings Regenerated
- [x] iOS Project Synced
- [ ] Build Verified (In Progress)
- [ ] Runtime Verified (Pending physical device)
