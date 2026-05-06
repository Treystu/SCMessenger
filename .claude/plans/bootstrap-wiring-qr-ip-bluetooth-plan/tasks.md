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

## Phase 3: Bluetooth Verification

- [ ] 3.1 Verify Android GATT advertising starts
- [ ] 3.2 Verify CLI BLE central scanning starts
- [ ] 3.3 Test bidirectional messaging
- [ ] 3.4 Test graceful degradation when BLE disabled

## Verification

- [ ] 4.1 `cargo check --workspace` passes (0 errors)
- [ ] 4.2 `cargo test --workspace` passes (all tests)
- [ ] 4.3 Android debug build succeeds
- [ ] 4.4 No hardcoded IPs remain
- [ ] 4.5 CLI ↔ Android connect via mDNS
- [ ] 4.6 CLI ↔ Android connect via BLE
- [ ] 4.7 QR code contains IPs
- [ ] 4.8 Ledger persists across restarts