# T1.4 — Wi-Fi Direct Rust transport + Android bridge (G2)

**Status:** partial
**Track:** 1 (Native Hardware & Proximity Transport Layer)
**Dependencies:** T1.1
**Blocks:** none

## Technical Context
- Rust has only `TransportType::WiFiDirect` enum + capabilities (`abstraction.rs`)
- Kotlin `WifiDirectTransport.kt` exists (group formation scaffolding)
- No iOS equivalent (platform limitation — iOS has no Wi-Fi Direct API; Multipeer covers the niche)

## Implementation
1. Mirror the wifi_aware.rs pattern: new `transport/wifi_direct.rs` with `WifiDirectPlatformBridge` trait (`discover_peers`, `connect(device_addr)`, `create_group`, `remove_group`, callbacks for peers-changed/connection-info)
2. On connection-info (group owner IP 192.168.49.1 + client IPs), dial over TCP exactly as T1.2 step 3
3. Group-owner election: prefer the device with `is_charging || battery_pct > 50` (DeviceProfile already crosses FFI) by setting `groupOwnerIntent` accordingly on the Kotlin side

## Edge Cases
- Wi-Fi Direct and infrastructure Wi-Fi conflict on many chipsets (STA+P2P concurrency varies) — treat `WIFI_P2P_STATE_DISABLED` as transport-down, never retry-loop
- Android 13+ requires `NEARBY_WIFI_DEVICES`; legacy needs location enabled (not just granted)
- GO negotiation needs user-visible system dialog on some OEMs for the first connection — document as known UX constraint; invitation-based reconnect avoids it

## Verification
- [x] Rust unit tests with a mock bridge (state machine, GO-intent computation from DeviceProfile)
- [ ] Two-device manual procedure in `docs/device-testing.md`
- [x] `cargo clippy` clean
- [x] FFI snapshot updated (no FFI surface change; group-owner-intent stays Kotlin-local)

## Update (2026-07-01)
`groupOwnerIntent` in `WifiDirectTransport.kt` now computed from live battery
state (`is_charging || battery_pct > 50` -> intent 7, else 0) instead of the
hardcoded 0. Mirrored in `core/src/transport/wifi_direct.rs::compute_group_owner_intent`
with unit tests covering both branches. Two-device manual test procedure still
outstanding (hardware-dependent, out of scope for this pass).

## Update (2026-07-02, S8 reconciliation)
`cargo test --workspace --all-features` and
`cargo clippy --workspace --all-features -- -D warnings` re-run locally
against this branch's current state (verified 2026-07-02, local run) - both
green, no regressions from the intervening readiness-assessment fixes
(T1-T7, S2-S7). The two-device manual procedure box stays unchecked; it
requires physical hardware and cannot be verified in this environment.
