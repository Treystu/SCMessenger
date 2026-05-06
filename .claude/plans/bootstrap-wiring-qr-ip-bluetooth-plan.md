# Bootstrap Wiring, QR IP Sharing & Bluetooth Verification Plan

## Executive Summary

Three workstreams to fix connectivity between CLI and Android:

1. **Bootstrap Wiring Fix** — The ledger infrastructure exists but is not wired into dial paths. Android's `primeRelayBootstrapConnections()` builds `addresses = emptyList()` and dials nothing.
2. **QR IP Sharing** — InviteTokens currently only contain identity data. Extend to include the node's current LAN/WAN IPs for direct peer-to-peer connections.
3. **Bluetooth Verification** — BLE central (CLI scanning) is implemented but peripheral advertising is a stub. Verify CLI can discover Android over BLE.

---

## Workstream 1: Bootstrap Wiring Fix

### Problem Statement

The connection ledger (`ConnectionLedger`, `RelayDiscovery`, `get_preferred_relays()`) is fully implemented in core, but:

- **Android**: `primeRelayBootstrapConnections()` builds `addresses = emptyList<String>()` and dials nothing
- **Android**: `racingBootstrapWithFallback()` builds `prioritizedAddresses = emptyList<String>()`
- **Core**: `CORE_BOOTSTRAP_NODES` contains dead/hallucinated addresses (`34.135.34.73`, `bootstrap.scmessenger.net`, etc.)
- **CLI**: Config has stale `34.135.34.73` from initial setup

### Solution Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Bootstrap Priority Order                    │
├─────────────────────────────────────────────────────────────────┤
│ 1. ENV override (SC_BOOTSTRAP_NODES) — highest priority         │
│ 2. Remote URL fetch (operator-configurable, not yet impl)       │
│ 3. Ledger-first — peers from previous sessions, ranked by:      │
│    - recency (last_seen)                                        │
│    - reliability (success_count)                                │
│    - transport type (LAN > WebSocket > TCP)                     │
│ 4. mDNS local discovery (already working)                       │
│ 5. Static fallback — ONLY for development, NOT production       │
└─────────────────────────────────────────────────────────────────┘
```

### Files to Modify

#### Core (Rust)

| File | Change |
|------|--------|
| `core/src/transport/bootstrap.rs` | Remove dead IPs from `CORE_BOOTSTRAP_NODES`. Replace with comment about ledger-first. |
| `core/src/mobile_bridge.rs` | Add `get_preferred_relays()` bridge method that returns `Vec<String>` multiaddrs |
| `core/src/transport/ledger.rs` | Ensure `to_shared_entries()` and `merge_shared_entries()` are exposed to bridge |

#### Android (Kotlin)

| File | Change |
|------|--------|
| `MeshRepository.kt` | In `primeRelayBootstrapConnections()`, call `ledgerManager.getPreferredRelays(limit=5)` and use those addresses |
| `MeshRepository.kt` | In `racingBootstrapWithFallback()`, same — populate `prioritizedAddresses` from ledger |
| `MeshRepository.kt` | Remove hardcoded port probe list (`34.135.34.73`, `104.28.216.43`) |
| `LedgerManager.kt` | Ensure `getPreferredRelays()` is wired to core bridge |

#### CLI (Rust)

| File | Change |
|------|--------|
| `cli/src/main.rs` | In `cmd_start()`, call `ledger.get_preferred_relays()` before falling back to static |
| `cli/src/config.rs` | Remove `bootstrap_nodes` field or mark as deprecated |

### Verification Steps

1. Fresh install on Android — verify ledger populates after first mDNS discovery
2. Restart app — verify ledger is loaded and used for bootstrap
3. CLI start — verify it reads `peers.json` and attempts ledger entries first
4. CLI ↔ Android on same LAN — verify direct connection via mDNS + ledger

---

## Workstream 2: QR IP Sharing

### Problem Statement

Current `InviteToken` contains:
- `inviter_id`, `invitee_id`
- `inviter_public_key`
- `created_at`, `expires_at`
- `signature`

**Missing**: The inviter's current network addresses (LAN IP, WAN IP) so the invitee can dial directly.

### Solution Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Extended InviteToken                         │
├─────────────────────────────────────────────────────────────────┤
│ Existing fields:                                                │
│   - inviter_id, invitee_id, inviter_public_key                  │
│   - created_at, expires_at, signature, metadata                 │
│ NEW fields:                                                     │
│   - lan_addresses: Vec<String>   // /ip4/192.168.1.x/tcp/9001  │
│   - wan_addresses: Vec<String>   // /ip4/1.2.3.4/tcp/9001      │
│   - relay_hints: Vec<String>     // known relays if direct fail│
└─────────────────────────────────────────────────────────────────┘
```

### External IP Resolution (Already Implemented!)

The mesh already has **peer-assisted NAT discovery** via `AddressReflectionService`:

```
core/src/transport/reflection.rs:
  - AddressReflectionRequest: "What's my external address?"
  - AddressReflectionResponse: "I see you as 1.2.3.4:9001"

core/src/transport/observation.rs:
  - AddressObserver: Aggregates observations from multiple peers
  - primary_external_address(): Returns consensus WAN IP
```

This is the **sovereign mesh equivalent of STUN** — no external servers needed. Any connected peer can tell you your external IP.

**What needs wiring:**
1. Expose `AddressObserver::primary_external_address()` to `IronCore`
2. Wire it into `InviteToken::with_current_addresses()`
3. Android: Call reflection service before generating QR

### IP Detection Strategy

```rust
// In core/src/relay/invite.rs

impl InviteToken {
    pub fn with_current_addresses(mut self, core: &IronCore) -> Self {
        // 1. Get LAN IPs from libp2p swarm (already detected for mDNS)
        self.lan_addresses = core.get_lan_multiaddrs();
        
        // 2. Get WAN IP via peer-assisted reflection (no STUN needed!)
        if let Some(wan) = core.address_observer().primary_external_address() {
            self.wan_addresses = vec![format!("/ip4/{}/tcp/{}", wan.ip(), wan.port())];
        }
        
        // 3. Get known relays from ledger
        self.relay_hints = core.get_preferred_relays(3);
        
        self
    }
}
```

### QR Code Format

**Decision: Binary (bincode)** — native SCMessenger-to-SCMessenger pairing only.

- Compact (no Base64 overhead)
- Direct deserialization via `InviteToken::from_bytes()`
- Already implemented in `core/src/relay/invite.rs`

### QR Code Flow

```
1. User A taps "Share Invite" in app
2. App:
   a. Creates InviteToken with identity
   b. Calls reflection service to get WAN IP (async, cached)
   c. Gets LAN IPs from swarm
   d. Serializes to bytes via InviteToken::to_bytes()
   e. Renders as QR code
3. User B scans QR code with SCMessenger
4. B's app deserializes InviteToken
5. B attempts connections in order:
   a. LAN addresses (if same subnet)
   b. WAN addresses (if reachable)
   c. Relay hints (fallback)
6. On success, B adds A to contacts and ledger
```

### Files to Modify

| File | Change |
|------|--------|
| `core/src/relay/invite.rs` | Add `lan_addresses`, `wan_addresses`, `relay_hints` fields to `InviteToken` |
| `core/src/relay/invite.rs` | Add `with_current_addresses()` method |
| `core/src/iron_core.rs` | Expose `address_observer()` and add `get_lan_multiaddrs()` helper |
| `core/src/mobile_bridge.rs` | Add bridge methods for address reflection and invite generation |
| `android/.../ui/identity/IdentityScreen.kt` | Add "Share Invite" button, generate QR with IPs |
| `android/.../data/QRInviteManager.kt` | New file — QR generation/parsing with IP addresses |
| `cli/src/main.rs` | Add `invite create` and `invite accept` commands with IP handling |

### Verification Steps

1. Generate QR on Android — verify it contains LAN IP
2. Scan QR from CLI — verify it extracts IP and attempts direct dial
3. Cross-network test — verify WAN IP is included (from peer reflection)
4. Verify reflection service works: connect two nodes, check each sees other's external IP

---

## Workstream 3: Bluetooth Verification

### Current State

| Component | Status |
|-----------|--------|
| CLI BLE Central (scanning) | ✅ Implemented in `cli/src/ble_mesh.rs` |
| CLI BLE Peripheral (advertising) | ⚠️ Stub only — logs intention, no actual advertising |
| Core BLE Protocol | ✅ GATT service, fragmentation, reassembly in `core/src/transport/ble/` |
| Android BLE GATT Server | ✅ `GattServer` trait defined, platform impl needed |

### BLE Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    BLE Connection Model                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  SCM GATT Service UUID: 0xDF010000-0000-1000-8000-00805F9B34FB  │
│                                                                 │
│  Characteristics:                                               │
│    0xDF02: Write   (send data to peripheral)                    │
│    0xDF03: Notify  (receive data from peripheral)               │
│    0xDF04: Status  (connection state)                           │
│                                                                 │
│  Message Flow:                                                  │
│    1. Central (CLI) scans for GATT service UUID                 │
│    2. Peripheral (Android) advertises service                   │
│    3. Central connects, subscribes to Notify (0xDF03)           │
│    4. Messages flow both ways via Write/Notify                  │
│    5. Large messages fragmented (512-byte characteristic limit) │
│                                                                 │
│  Platform Constraints:                                          │
│    - Windows/macOS/Linux CLI: Central mode only (btleplug)      │
│    - Android: Both Central and Peripheral modes                 │
│    - iOS: Background advertising restricted                     │
│                                                                 │
│  Power/Battery Considerations:                                  │
│    - BLE may be disabled by system power saving                 │
│    - Handle graceful degradation when BLE unavailable           │
│    - mDNS fallback when BLE fails                               │
└─────────────────────────────────────────────────────────────────┘
```

### Connection Flow

```
┌──────────────────┐         ┌──────────────────┐
│   CLI (Central)  │         │ Android (Periph) │
└────────┬─────────┘         └────────┬─────────┘
         │                            │
         │  1. Scan for GATT service  │
         │ ◄───────────────────────────│  Advertise 0xDF01...
         │                            │
         │  2. Connect                │
         │ ──────────────────────────►│
         │                            │
         │  3. Discover characteristics│
         │ ──────────────────────────►│
         │                            │
         │  4. Subscribe to Notify    │
         │ ──────────────────────────►│
         │                            │
         │  5. Message via Notify     │
         │ ◄───────────────────────────│  (push decrypted msgs)
         │                            │
         │  6. Message via Write      │
         │ ──────────────────────────►│
         │                            │
```

### Verification Test Plan

1. **Start Android app** — verify GATT advertising starts (check logs for "BLE advertising")
2. **Start CLI** — `scmessenger-cli start`
3. **Check CLI logs** — should see "BLE scan active (filtered to SCM service)"
4. **Check connection** — CLI should log "BLE GATT notify subscribed on <MAC>"
5. **Send test message Android → CLI** — verify arrives via BLE
6. **Send test message CLI → Android** — verify arrives via BLE Write
7. **Disable BLE on Android** — verify graceful fallback to mDNS

### Files to Verify/Modify

| File | Action |
|------|--------|
| `cli/src/ble_mesh.rs` | Verify central scanning starts by default |
| `cli/src/ble_daemon.rs` | Verify graceful handling when BLE unavailable |
| `core/src/transport/ble/gatt.rs` | Verify fragmentation/reassembly works |
| `android/.../BluetoothManager.kt` | Verify GATT server starts on app launch |
| `android/.../BlePeripheralService.kt` | Verify advertising starts with correct UUID |

### Known Limitations & Mitigations

| Limitation | Mitigation |
|------------|------------|
| CLI cannot advertise (btleplug limitation) | Android advertises, CLI scans |
| BLE may be disabled by power saving | Graceful fallback to mDNS |
| BLE range limited (~10m) | Works for proximity pairing |
| iOS background advertising restricted | Focus on Android for BLE MVP |

---

## Execution Order

### Phase 1: Bootstrap Wiring (Critical Path)
1. Wire `get_preferred_relays()` into Android `primeRelayBootstrapConnections()`
2. Wire `get_preferred_relays()` into CLI `cmd_start()`
3. Remove dead hardcoded IPs from `CORE_BOOTSTRAP_NODES`
4. Verify CLI ↔ Android connection via mDNS + ledger

### Phase 2: QR IP Sharing
1. Expose `AddressObserver` to `IronCore` bridge
2. Extend `InviteToken` with address fields
3. Add `with_current_addresses()` method
4. Wire into Android QR generation
5. Wire into CLI invite commands

### Phase 3: Bluetooth Verification
1. Verify Android GATT advertising starts
2. Verify CLI BLE central scanning starts
3. Test bidirectional messaging
4. Test graceful degradation when BLE disabled

---

## Regression Prevention Plan

### Pre-Implementation Baseline

Before making any changes, capture current state:

```bash
# 1. Build verification
cargo check --workspace
cargo test --workspace --no-run

# 2. Android build verification
cd android && ./gradlew assembleDebug

# 3. Create baseline commit
git add -A
git commit -m "baseline: pre-bootstrap-wiring state"
git tag pre-bootstrap-wiring
```

### Per-Phase Verification

Each phase must pass these checks before proceeding:

#### Phase 1: Bootstrap Wiring

| Check | Command | Expected |
|-------|---------|----------|
| Core compiles | `cargo check -p scmessenger-core` | 0 errors |
| WASM compiles | `cargo check -p scmessenger-core --target wasm32-unknown-unknown --features wasm` | 0 errors |
| CLI compiles | `cargo check -p scmessenger-cli` | 0 errors |
| Core tests pass | `cargo test -p scmessenger-core` | All pass |
| Android builds | `cd android && ./gradlew assembleDebug` | BUILD SUCCESSFUL |
| No hardcoded IPs | `grep -r "34.135.34.73" core/src/` | No matches |
| Ledger wired | `grep -r "getPreferredRelays" android/` | Matches in MeshRepository |

#### Phase 2: QR IP Sharing

| Check | Command | Expected |
|-------|---------|----------|
| InviteToken compiles | `cargo check -p scmessenger-core` | 0 errors |
| InviteToken serializes | `cargo test -p scmessenger-core invite` | Tests pass |
| Bridge exposed | `grep -r "address_observer" core/src/iron_core.rs` | Match found |
| Android QR generates | Manual test on device | QR renders with IPs |

#### Phase 3: Bluetooth Verification

| Check | Command | Expected |
|-------|---------|----------|
| CLI BLE starts | `./target/debug/scmessenger-cli start 2>&1 \| grep -i ble` | "BLE scan active" |
| Android advertises | Check logcat for GATT service | "Advertising service 0xDF01" |
| Connection works | Manual test CLI ↔ Android | Messages flow both ways |
| Graceful degradation | Disable BLE on Android, restart | mDNS fallback works |

### Rollback Procedure

If any phase fails and cannot be fixed within 30 minutes:

```bash
# Rollback to baseline
git checkout pre-bootstrap-wiring
git checkout pre-bootstrap-wiring -- .
cargo build

# Or rollback to specific phase
git checkout phase-1-complete
```

### Continuous Verification

After each file modification:

```bash
# Quick compile check (30s)
cargo check -p scmessenger-core

# If core passes, check full workspace (2min)
cargo check --workspace
```

### Integration Test Script

Create `scripts/verify-bootstrap-wiring.sh`:

```bash
#!/bin/bash
set -e

echo "=== Phase 1: Bootstrap Wiring Verification ==="

# Check for hardcoded IPs
echo "Checking for hardcoded IPs..."
if grep -r "34.135.34.73" core/src/ --include="*.rs"; then
    echo "ERROR: Hardcoded IP still present"
    exit 1
fi
echo "✓ No hardcoded IPs"

# Check ledger wiring
echo "Checking ledger wiring..."
if ! grep -q "getPreferredRelays" android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt; then
    echo "ERROR: Ledger not wired in MeshRepository"
    exit 1
fi
echo "✓ Ledger wired in Android"

# Build check
echo "Building workspace..."
cargo check --workspace
echo "✓ Workspace compiles"

echo "=== All checks passed ==="
```

---

## Verification Checklist

After all phases complete:

- [ ] `cargo check --workspace` passes (0 errors)
- [ ] `cargo test --workspace` passes (all tests)
- [ ] `cargo check -p scmessenger-core --target wasm32-unknown-unknown --features wasm` passes
- [ ] Android debug build succeeds (`./gradlew assembleDebug`)
- [ ] No hardcoded IPs remain in `core/src/transport/bootstrap.rs`
- [ ] CLI ↔ Android connect via mDNS on same LAN
- [ ] CLI ↔ Android connect via BLE (Android advertises, CLI scans)
- [ ] QR code contains LAN IP and WAN IP (if available)
- [ ] Ledger persists across app restarts
- [ ] Bootstrap uses ledger entries first, not hardcoded IPs
- [ ] BLE gracefully falls back to mDNS when disabled

---

## Estimated Effort

| Workstream | Files | Effort |
|------------|-------|--------|
| Bootstrap Wiring | 6 | 2-3 hours |
| QR IP Sharing | 7 | 2-3 hours |
| Bluetooth Verification | 4 | 1-2 hours |
| **Total** | **17** | **5-8 hours** |
