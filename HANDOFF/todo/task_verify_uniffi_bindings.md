# Task: Verify UniFFI Annotations Complete

## Background
Commit `6d528dc4` added 8 interfaces to `core/src/api.udl` but the corresponding Rust structs lacked `#[derive(uniffi::Object)]` and `#[uniffi::export]` annotations, causing Android crash:
```
java.lang.UnsatisfiedLinkError: undefined symbol: uniffi_scmessenger_core_fn_clone_autoadjustengine
```

A previous agent added annotations to:
- `core/src/iron_core.rs` — `IronCore`
- `core/src/mobile_bridge.rs` — `MeshService`, `AutoAdjustEngine`, `MeshSettingsManager`, `HistoryManager`, `LedgerManager`, `SwarmBridge`
- `core/src/contacts_bridge.rs` — `ContactManager`

## Goals
1. Verify ALL 8 UDL interfaces have proper annotations
2. Verify ALL methods in UDL match Rust method signatures
3. Verify constructors have `#[uniffi::constructor]`
4. Verify `cargo check -p scmessenger-core` passes (or would pass if linker worked)
5. Check for any missing types that need `uniffi::Record` or other derives

## UDL Interfaces to Verify
1. `IronCore` — `core/src/iron_core.rs`
2. `MeshService` — `core/src/mobile_bridge.rs`
3. `AutoAdjustEngine` — `core/src/mobile_bridge.rs`
4. `MeshSettingsManager` — `core/src/mobile_bridge.rs`
5. `ContactManager` — `core/src/contacts_bridge.rs`
6. `HistoryManager` — `core/src/mobile_bridge.rs`
7. `LedgerManager` — `core/src/mobile_bridge.rs`
8. `SwarmBridge` — `core/src/mobile_bridge.rs`

## Verification Steps
1. Read `core/src/api.udl` and extract all interface definitions
2. For each interface, find the Rust struct and verify:
   - `#[derive(uniffi::Object)]` is present
   - `#[uniffi::export]` is on the impl block
   - Methods match UDL signatures
3. Check `core/src/lib.rs` to ensure correct modules are exported
4. Check if any types referenced in UDL need additional derives

## Report
- Which interfaces are fully annotated
- Which have mismatches or missing methods
- Recommended fixes for any gaps
- Whether the Kotlin bindings (`api.kt`) would generate correctly
