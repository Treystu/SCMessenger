# Implementation Tasks

## Phase 1: Bootstrap Wiring Fix

- [ ] 1.1 Create baseline commit and tag
- [ ] 1.2 Wire `get_preferred_relays()` into Android `primeRelayBootstrapConnections()`
- [ ] 1.3 Wire `get_preferred_relays()` into CLI `cmd_start()`
- [ ] 1.4 Remove dead hardcoded IPs from `CORE_BOOTSTRAP_NODES`
- [ ] 1.5 Verify CLI ↔ Android connection via mDNS + ledger

## Phase 2: QR IP Sharing

- [ ] 2.1 Expose `AddressObserver` to `IronCore` bridge
- [ ] 2.2 Extend `InviteToken` with address fields
- [ ] 2.3 Add `with_current_addresses()` method
- [ ] 2.4 Wire into Android QR generation
- [ ] 2.5 Wire into CLI invite commands

## Phase 3: Bluetooth Verification & Parity

### Current Status (2026-05-06)
- ✅ Android: Full BLE implementation (GATT Server + Client)
- ⚠️ Windows CLI: Partial (scanning only, no advertising)
- ❌ **Gap**: btleplug doesn't support peripheral mode on desktop

### Phase 3A: Bluetooth Verification (Current Capabilities)
- [ ] 3.1 Verify Android GATT advertising starts
- [ ] 3.2 Verify CLI BLE central scanning starts  
- [ ] 3.3 Test unidirectional messaging (Windows → Android via BLE)
- [ ] 3.4 Test graceful degradation when BLE disabled

### Phase 3B: Bluetooth Parity Implementation (Future Work)
**See**: `HANDOFF/BLUETOOTH_DISCOVERY_PARITY_PLAN.md` for full details

**Option 1 (Recommended)**: Windows GATT Server via WinRT APIs
- [ ] 3.5 Add `windows` crate dependency for BLE APIs
- [ ] 3.6 Implement `cli/src/ble_windows.rs` with GATT server
- [ ] 3.7 Add BLE advertising via `BluetoothLEAdvertisementPublisher`
- [ ] 3.8 Implement characteristics (Identity, Message, Sync)
- [ ] 3.9 Test bidirectional BLE messaging (Windows ↔ Android)
- [ ] 3.10 Verify Android can discover Windows via BLE

**Option 2 (Quick Win)**: Hybrid mDNS + BLE approach
- [ ] 3.11 Use mDNS for discovery (already working)
- [ ] 3.12 Use BLE for low-latency messaging only
- [ ] 3.13 Keep TCP/IP for bidirectional reliability

**Estimated Effort**: 
- Option 1: 2-3 weeks (full parity)
- Option 2: 1 week (hybrid approach)

## Verification

### Build & Code Quality
- [ ] 4.1 `cargo check --workspace` passes (0 errors)
- [ ] 4.2 `cargo test --workspace` passes (all tests)
- [ ] 4.3 Android debug build succeeds
- [ ] 4.4 No hardcoded IPs remain

### Discovery & Connectivity
- [ ] 4.5 CLI ↔ Android connect via mDNS (✅ mDNS enabled on Windows as of 2026-05-06)
- [ ] 4.6 CLI ↔ Android connect via DHT (bootstrap nodes)
- [ ] 4.7 CLI can scan and discover Android via BLE (unidirectional)
- [ ] 4.8 Android ↔ Windows bidirectional BLE (requires Phase 3B)

### Features
- [ ] 4.9 QR code contains IPs
- [ ] 4.10 Ledger persists across restarts
- [ ] 4.11 Discovery status API shows all transports
- [ ] 4.12 Graceful degradation when transports unavailable

### Documentation
- [ ] 4.13 Update README with discovery status
- [ ] 4.14 Document BLE limitations (Windows peripheral mode)
- [ ] 4.15 Add troubleshooting guide for discovery issues