# Phase 2: Platform Clients

**Priority:** P1
**Assigned Agent:** implementer (qwen3-coder-next:cloud) + worker (gemma4:31b:cloud)
**Status:** TODO
**Depends On:** phase_1c_integration_tests

## 2A: CLI Daemon
- [ ] Verify `scmessenger-cli` builds and serves on 127.0.0.1:9002
- [ ] Wire HTTP + WebSocket server (`server/` module)
- [ ] Wire `transport_bridge` and `transport_api`
- [ ] Wire `ble_daemon` and `ble_mesh`
- [ ] Wire `config`, `ledger`, `bootstrap`, `contacts`, `history`

## 2B: WASM Thin Client
- [ ] Verify `cargo build -p scmessenger-wasm --target wasm32-unknown-unknown`
- [ ] Wire `mesh`, `daemon_bridge`, `connection_state`, `transport`
- [ ] Wire `notification_manager`, `storage`, `worker`
- [ ] Test `wasm-pack build --target web`

## 2C: Mobile Bridge
- [ ] Generate UniFFI Kotlin bindings
- [ ] Generate UniFFI Swift bindings
- [ ] Verify `scmessenger-mobile` compiles for `aarch64-linux-android` and `x86_64-linux-android`
- [ ] Verify iOS target compilation

## 2D: Android App
- [ ] `./gradlew assembleDebug -x lint --quiet` passes
- [ ] Wire `MeshRepository` → ViewModels → Compose UI
- [ ] Wire BLE/WiFi transport managers
- [ ] Wire foreground service, notification channels
- [ ] Pass `RoleNavigationPolicyTest`

## Success Criteria
- CLI daemon serves on 127.0.0.1:9002
- WASM builds successfully
- Android debug APK compiles
