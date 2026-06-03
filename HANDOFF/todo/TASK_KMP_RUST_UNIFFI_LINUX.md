# TASK: KMP Scaffolding & Rust Integration — Rust & UniFFI Linux Specialist

## Agent Role
Agent 1: Rust & UniFFI Linux Specialist

## Context (Compressed)
SCMessenger is a Rust mesh networking engine with:
- `core/` — scmessenger-core (lib + cdylib): identity, crypto, transport, store, relay, identity, privacy, abuse, notification, drift, routing
- `mobile/` — scmessenger-mobile: UniFFI bridge crate for Android/iOS (cdylib/staticlib)
- `cli/` — scmessenger-cli: headless daemon + embedded web server (warp HTTP + WebSocket on 127.0.0.1:9002)
- `wasm/` — scmessenger-wasm: WASM bindings for browser thin-client via WebSocket /ws (JSON-RPC)

UniFFI version: 0.31. Current mobile bridge uses `gen-bindings` feature to generate Kotlin/Swift bindings.

## Your Mission
Design and implement the Rust/UniFFI integration layer for a Kotlin Multiplatform (KMP) Compose Multiplatform Ubuntu desktop client. The desktop client will consume the Rust core via UniFFI-generated Kotlin bindings, same as mobile.

### Specific Tasks
1. **Analyze current UniFFI binding surface**: Catalog all exported UDL interfaces in `mobile/` and `core/`. Identify which are desktop-relevant (crypto, identity, transport, store) vs mobile-specific (BLE, camera, push).
2. **Design desktop-specific UniFFI extensions**: BlueZ/BLE D-Bus integration via `libdbus` or `zbus` crate. Avahi/mDNS integration. XDG Base Directory compliance for sled store paths.
3. **Create `desktop_bridge/` module**: New crate inside the workspace that exports desktop-specific FFI functions (system tray status, native notification triggers, power management, socket activation).
4. **Update `Cargo.toml` workspace** to include the new desktop bridge crate with proper `cfg(target_os = "linux")` feature gates.
5. **Generate Kotlin bindings** for desktop using the existing `gen_kotlin` binary. Verify the output compiles with KMP targeting `linuxX64`.

### Output Format
- Rust source files in `desktop_bridge/` (or `core/src/desktop_bridge/`)
- Updated `Cargo.toml` entries
- Generated `.kt` binding files
- Verification: `cargo build -p scmessenger-core --features gen-bindings` succeeds

### Constraints
- Must NOT break existing Android/iOS mobile build
- Must use `cfg` flags to gate desktop-specific code
- Must maintain UniFFI ABI compatibility with existing Kotlin consumers
- sled store paths must use XDG directories on Linux (`~/.local/share/scmessenger`, `~/.config/scmessenger`)
- BlueZ integration via `zbus` (pure Rust D-Bus bindings, no C dependencies)
