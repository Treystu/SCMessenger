# SCMessenger: Complete Implementation Plan for Claude Code

> Verified against commit `5b1927df` (2026-02-21T12:56:44Z) — the actual HEAD of `main`.
> All file paths, function signatures, and line references are from the real codebase.

---

## How To Use This Document

Feed this entire file to Claude Code along with the repository. Each task group is
designed to be executed as a single unit — make all changes within a group, then run
the verification commands at the end of that group before moving on. Do NOT run tests
after every individual file change.

---

## TASK GROUP 1: Housekeeping & Repo Cleanup

These are trivial but should be done first to avoid noise in diffs.

### 1.1 Remove accidentally committed build artifacts

**Files to delete:**
- `core/log.txt` — contains a "No space left on device" error from a local build
- `core/out.txt` — full `cargo build` stdout (321 lines of compiler output)

**Action:** `git rm core/log.txt core/out.txt`

### 1.2 Remove duplicate generated bindings

Commit `81bf4209` accidentally checked in a second copy of the iOS generated bindings
under `core/iOS/` (in addition to the canonical `iOS/SCMessenger/SCMessenger/Generated/`).

**Files to delete (entire tree):**
- `core/iOS/SCMessenger/Generated/api.swift` (4374 lines, duplicate)
- `core/iOS/SCMessenger/Generated/apiFFI.h` (1747 lines, duplicate)
- `core/iOS/SCMessenger/Generated/apiFFI.modulemap` (6 lines, duplicate)
- `core/iOS/SCMessenger/SCMessenger/Generated/api.swift` (4374 lines, duplicate)
- `core/iOS/SCMessenger/SCMessenger/Generated/apiFFI.h` (1747 lines, duplicate)
- `core/iOS/SCMessenger/SCMessenger/Generated/apiFFI.modulemap` (6 lines, duplicate)

**Action:** `git rm -r core/iOS/`

### 1.3 Add `core/iOS/` to `.gitignore`

Append to `.gitignore`:
```
core/iOS/
core/log.txt
core/out.txt
*.xcuserstate
```

### Verification (Group 1)

```bash
git status  # confirm only intended deletions
```

---

## TASK GROUP 2: Rust Core — Fix `extract_public_key_from_peer_id`

The newest commit (`5b1927df`) added `extract_public_key_from_peer_id()` to `core/src/lib.rs`.
This function is called from both iOS (`CoreDelegateImpl.swift`) and Android (`MeshRepository.kt`)
in the `onPeerIdentified` callback to extract the Ed25519 public key from a libp2p PeerId.

### 2.1 Audit the current implementation

**File:** `core/src/lib.rs` lines ~297-313

Current code:
```rust
pub fn extract_public_key_from_peer_id(&self, peer_id: String) -> Result<String, IronCoreError> {
    let bytes = bs58::decode(&peer_id)
        .into_vec()
        .map_err(|_| IronCoreError::InvalidInput)?;
    if bytes.len() >= 32 {
        let pub_key_bytes = &bytes[bytes.len() - 32..];
        Ok(hex::encode(pub_key_bytes))
    } else {
        Err(IronCoreError::InvalidInput)
    }
}
```

**Problem:** This naively takes the last 32 bytes of the base58-decoded PeerId. A libp2p
Ed25519 PeerId is actually a multihash-encoded identity: `0x00 (identity hash) + 0x24 (length 36) +
0x08 0x01 (protobuf: Ed25519 key type) + 0x12 0x20 (protobuf: 32 bytes follow) + <32 bytes pubkey>`.
The current code happens to work because the last 32 bytes of a 38-byte payload ARE the key,
but it should be validated properly and will break on non-Ed25519 PeerIds.

**Fix in `core/src/lib.rs`:**

Replace the function body with proper multihash parsing:
```rust
pub fn extract_public_key_from_peer_id(&self, peer_id: String) -> Result<String, IronCoreError> {
    let bytes = bs58::decode(&peer_id)
        .into_vec()
        .map_err(|_| IronCoreError::InvalidInput)?;
    // libp2p Ed25519 PeerId: 0x00 0x24 0x08 0x01 0x12 0x20 <32 bytes>
    // Total = 38 bytes. Verify the protobuf prefix.
    if bytes.len() == 38
        && bytes[0] == 0x00  // identity multihash
        && bytes[1] == 0x24  // length 36
        && bytes[2] == 0x08  // protobuf field 1 (key type)
        && bytes[3] == 0x01  // Ed25519
        && bytes[4] == 0x12  // protobuf field 2 (key data)
        && bytes[5] == 0x20  // 32 bytes
    {
        Ok(hex::encode(&bytes[6..38]))
    } else if bytes.len() >= 32 {
        // Fallback for non-standard PeerIds: take last 32 bytes
        Ok(hex::encode(&bytes[bytes.len() - 32..]))
    } else {
        Err(IronCoreError::InvalidInput)
    }
}
```

### 2.2 Add unit test

**File:** `core/src/lib.rs` — inside the existing `#[cfg(test)] mod tests` block

```rust
#[test]
fn test_extract_public_key_from_peer_id() {
    let core = IronCore::new();
    core.initialize_identity().unwrap();
    let info = core.get_identity_info();
    let libp2p_peer_id = info.libp2p_peer_id.unwrap();
    let extracted_hex = core.extract_public_key_from_peer_id(libp2p_peer_id).unwrap();
    let original_hex = info.public_key_hex.unwrap();
    assert_eq!(extracted_hex, original_hex,
        "Extracted public key must match the identity's own public key");
}
```

### Verification (Group 2)

```bash
cargo test -p scmessenger-core -- test_extract_public_key_from_peer_id
cargo test -p scmessenger-core -- tests  # all lib.rs tests
cargo clippy -p scmessenger-core -- -D warnings
```

---

## TASK GROUP 3: Rust Core — Wire PlatformBridge Callbacks + Fix Stats

Per PRODUCTION_READINESS_AUDIT.md P2 items #2 and #3: PlatformBridge methods are never
called, and `get_stats()` returns all zeros.

### 3.1 Wire `on_data_received` to actually process incoming BLE data

**File:** `core/src/mobile_bridge.rs`

The `on_data_received` method already increments `bytes_transferred` and calls
`core.receive_message(data)`. But the `receive_message` result isn't used to update
`messages_relayed`. The `IronCore::receive_message()` already calls the delegate internally.

**Fix:** After `core.receive_message(data)` succeeds in `on_data_received()`, increment
`stats.messages_relayed`:

```rust
pub fn on_data_received(&self, peer_id: String, data: Vec<u8>) {
    let mut stats = self.stats.lock();
    stats.bytes_transferred += data.len() as u64;
    drop(stats);

    if let Some(core) = self.get_core() {
        match core.receive_message(data) {
            Ok(msg) => {
                tracing::info!("Message received from {}: {:?}", peer_id, msg.id);
                let mut stats = self.stats.lock();
                stats.messages_relayed += 1;
            }
            Err(e) => {
                tracing::error!("Failed to process received message: {:?}", e);
            }
        }
    }
}
```

### 3.2 Wire `get_stats()` to include real peer count from SwarmBridge

**File:** `core/src/mobile_bridge.rs` — `get_stats()` method

Replace the placeholder augmentation comment with real data:

```rust
pub fn get_stats(&self) -> ServiceStats {
    let mut stats = self.stats.lock().clone();

    // Get real peer count from SwarmBridge
    let handle_guard = self.swarm_bridge.handle.lock();
    if handle_guard.is_some() {
        if let Ok(peers) = self.swarm_bridge.get_peers_sync() {
            stats.peers_discovered = peers.len() as u32;
        }
    }

    // Calculate uptime
    // (uptime_secs is tracked elsewhere or we can derive from start time)
    stats
}
```

Note: `get_peers_sync()` needs to exist. Check if `SwarmBridge::get_peers()` is already
synchronous (it is — it blocks on the runtime handle). So just use:

```rust
pub fn get_stats(&self) -> ServiceStats {
    let mut stats = self.stats.lock().clone();
    let peers = self.swarm_bridge.get_peers();
    stats.peers_discovered = peers.len() as u32;
    stats
}
```

### 3.3 Wire `update_device_state` to propagate to swarm relay budget

This is ALREADY DONE as of commit `81bf4209`. The `update_device_state()` method:
- Stores the profile in `current_device_profile`
- Computes relay budget based on battery level
- Calls `set_relay_budget()` which forwards to swarm

**No action needed here.** Verified in current code.

### Verification (Group 3)

```bash
cargo test -p scmessenger-core -- mobile_bridge
cargo test --workspace
```

---

## TASK GROUP 4: Rust Core — Rebuild xcframework & Regenerate Bindings

The xcframework checked into the repo only contains headers (76KB `.h` file). The actual
`.a` static library must be built locally. After Groups 2-3 modified `lib.rs` and
`mobile_bridge.rs`, the bindings must be regenerated.

### 4.1 Build for iOS targets

```bash
cd mobile
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
```

### 4.2 Regenerate UniFFI bindings

```bash
cd core
cargo run --bin uniffi-bindgen generate src/api.udl --language swift --out-dir ../iOS/SCMessenger/SCMessenger/Generated/
cargo run --bin uniffi-bindgen generate src/api.udl --language kotlin --out-dir ../android/app/src/main/java/
```

### 4.3 Recreate xcframework

```bash
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libscmessenger_mobile.a \
  -headers iOS/SCMessenger/SCMessenger/Generated/apiFFI.h \
  -library target/aarch64-apple-ios-sim/release/libscmessenger_mobile.a \
  -headers iOS/SCMessenger/SCMessenger/Generated/apiFFI.h \
  -output SCMessengerCore.xcframework
```

### 4.4 Verify iOS builds

```bash
cd iOS/SCMessenger
xcodebuild -scheme SCMessenger -sdk iphonesimulator -arch arm64 build
```

**Known issue from `build_new.txt`:** The build log shows compilation succeeds through
all Swift files. If there are linker errors about undefined `uniffi_scmessenger_core_fn_*`
symbols, it means the xcframework binary is stale. Step 4.3 fixes this.

### 4.5 Verify Android builds

```bash
cd android
./gradlew :app:clean :app:assembleDebug
./gradlew test
```

### Verification (Group 4)

All of steps 4.4 and 4.5 above constitute the verification.

---

## TASK GROUP 5: Complete WebRTC Transport (WASM)

Per CLAUDE.md "Known Remaining Gaps": 4 specific items totaling ~140 LOC.

### 5.1 Add `RtcSdpType` to web-sys features

**File:** `wasm/Cargo.toml`

Find the `[dependencies.web-sys]` section and add `"RtcSdpType"` to the features list.

### 5.2 Implement `set_remote_answer`

**File:** `wasm/src/transport.rs`

Find the TODO comment for `set_remote_answer`. Implement:
- Parse the answer SDP JSON string
- Create `RtcSessionDescriptionInit` with type = `RtcSdpType::Answer`
- Call `peer_connection.set_remote_description()` via `JsFuture`
- ~50 LOC

### 5.3 Implement ICE trickle candidate exchange

**File:** `wasm/src/transport.rs`

- Add `ice_candidates: Vec<String>` to `WebRtcInner`
- Buffer candidates from `onicecandidate` events
- Expose `get_ice_candidates()` / `add_ice_candidate()` methods
- ~30 LOC

### 5.4 Implement WebRTC answerer path

**File:** `wasm/src/transport.rs`

- `set_remote_offer()`: set remote description with type = Offer
- `create_answer()`: call `peer_connection.create_answer()` via JsFuture, then `set_local_description()`
- Mirrors `create_offer()` exactly
- ~60 LOC

### Verification (Group 5)

```bash
cd wasm && wasm-pack build --target web
cd wasm && wasm-pack test --headless --chrome  # if chromedriver available
```

---

## TASK GROUP 6: Dependency Security Audit

Per Issue #42 and AUDIT_RESOLUTIONS.md findings.

### 6.1 Run `cargo audit` and document current state

```bash
cargo audit
```

### 6.2 Evaluate vulnerable dependency chain

**Check these specific items:**
- `ed25519-dalek` — should be 2.1+ (verify in `Cargo.lock`)
- `ring` version via libp2p 0.53 — check if 0.17+ is available
- `sled 0.34` — used by `identity/store.rs`, `contacts_bridge.rs`, `cli/src/ledger.rs`,
  `cli/src/contacts.rs`, `cli/src/history.rs`. The CLAUDE.md acknowledges sled as the
  current storage backend. Replacing it is a significant change.
- `bincode 1.3.3` — used throughout for wire format serialization

**Decision point:** These are real vulnerabilities but replacing `sled` and `bincode` touches
~15 files. This should be a separate branch/PR, not mixed with feature work.

### 6.3 Recommended approach

Create a tracking issue with:
- Pin `ed25519-dalek >= 2.1` in workspace `Cargo.toml` (verify already done)
- Evaluate `redb` as sled replacement: write a spike migration of `identity/store.rs`
- Evaluate `postcard` as bincode replacement: write a spike migration of `message/codec.rs`
- Do NOT replace sled/bincode in the same PR as feature work

### Verification (Group 6)

```bash
cargo audit
cargo deny check advisories  # if cargo-deny is installed
```

---

## TASK GROUP 7: Integration Test Validation

### 7.1 Run all existing Rust integration tests

```bash
cargo test --test integration_e2e
cargo test --test integration_all_phases
cargo test --test integration_ironcore_roundtrip
cargo test --test integration_nat_reflection
cargo test --test test_address_observation
cargo test --test test_mesh_routing
cargo test --test test_multiport
```

### 7.2 Add message persistence restart test

**New file:** `core/tests/test_persistence_restart.rs`

```rust
use scmessenger_core::contacts_bridge::ContactManager;

#[test]
fn test_contact_persistence_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    // First instance: add a contact
    {
        let manager = ContactManager::new(path.to_string()).unwrap();
        let contact = scmessenger_core::contacts_bridge::Contact {
            peer_id: "test-peer-001".to_string(),
            nickname: Some("Alice".to_string()),
            public_key: "a".repeat(64),
            added_at: 1000,
            last_seen: None,
            notes: None,
        };
        manager.add(contact).unwrap();
        assert_eq!(manager.count(), 1);
    }
    // manager dropped here — sled should flush

    // Second instance: verify data survived
    {
        let manager2 = ContactManager::new(path.to_string()).unwrap();
        assert_eq!(manager2.count(), 1);
        let retrieved = manager2.get("test-peer-001".to_string()).unwrap().unwrap();
        assert_eq!(retrieved.nickname, Some("Alice".to_string()));
    }
}
```

Add `tempfile` as a dev-dependency in `core/Cargo.toml` if not present:
```toml
[dev-dependencies]
tempfile = "3"
```

### 7.3 Run Docker test suite (if Docker available)

```bash
cd docker
./run-all-tests.sh --rust-only
./run-all-tests.sh --integration-only
```

### Verification (Group 7)

All test commands above constitute the verification.

---

## TASK GROUP 8: Forward Secrecy — Double Ratchet Port (Future)

This is the largest remaining crypto gap. Reference implementations exist in
`reference/double-ratchet.ts` (14KB) and `reference/x3dh.ts` (19KB).

### 8.1 Scope

**New files:**
- `core/src/crypto/ratchet.rs` (~400-500 LOC)
- `core/src/crypto/x3dh.rs` (~300-400 LOC)
- `core/tests/test_double_ratchet.rs` (~200 LOC)

**Modified files:**
- `core/src/crypto/mod.rs` — add `pub mod ratchet; pub mod x3dh;`
- `core/src/crypto/encrypt.rs` — add `encrypt_with_session()` / `decrypt_with_session()`
- `core/src/message/types.rs` — add `MessageType::KeyExchange` variant
- `core/src/lib.rs` — add `establish_session()`, `get_or_create_session()` to `IronCore`
- `core/src/api.udl` — expose session management to mobile

### 8.2 Implementation approach

1. Port `reference/x3dh.ts` → `core/src/crypto/x3dh.rs`:
   - `X3DHKeys` struct (identity key, signed prekey, one-time prekeys)
   - `compute_shared_secret()` using triple DH
   - `X3DHBundle` for publishing prekeys

2. Port `reference/double-ratchet.ts` → `core/src/crypto/ratchet.rs`:
   - `RatchetState` struct (root key, chain keys, message keys)
   - `ratchet_encrypt()` / `ratchet_decrypt()`
   - `dh_ratchet_step()` for Diffie-Hellman ratchet advance
   - Skipped message key storage

3. Wire into `IronCore`:
   - On first message to a new contact: perform X3DH, establish session
   - On subsequent messages: use Double Ratchet
   - Fallback: if no session exists, use current static ECDH (backwards compat)

### 8.3 This is a separate PR

Do NOT mix this with Groups 1-7. This changes the crypto protocol and needs its own
review and testing cycle.

### Verification (Group 8)

```bash
cargo test -p scmessenger-core -- crypto::ratchet
cargo test -p scmessenger-core -- crypto::x3dh
cargo test --test test_double_ratchet
cargo test --test integration_ironcore_roundtrip  # backwards compat
```

---

## TASK GROUP 9: Group Messaging (Future)

No implementation exists. This requires:

- MLS (RFC 9420) or Sender Keys protocol implementation
- New `core/src/crypto/group.rs` (~600-800 LOC)
- New `core/src/message/group.rs` (~300-400 LOC)
- Updates to `api.udl` for group session management
- UI changes on both iOS and Android

This is a separate milestone. Do not attempt alongside Groups 1-8.

---

## TASK GROUP 10: Protocol Specification Document (Future)

**File:** `docs/PROTOCOL.md` (currently 85 lines — needs expansion to ~3000+ lines)

Formalize:
- Wire format (bincode envelope structure, field order, sizes)
- Drift Protocol framing (DriftFrame CRC32, DriftEnvelope 186-byte header)
- Key exchange handshake
- BLE beacon format (identity JSON payload)
- Gossipsub topic naming conventions
- Relay budget enforcement protocol
- Cover traffic generation parameters

---

## EXECUTION ORDER SUMMARY

| Order | Group | Scope | Blocking? |
|-------|-------|-------|-----------|
| 1 | Housekeeping | Delete junk files | No |
| 2 | Fix extract_public_key_from_peer_id | ~30 LOC Rust | No |
| 3 | Wire PlatformBridge + Stats | ~50 LOC Rust | No |
| 4 | Rebuild xcframework + bindings | Build scripts | Blocks iOS/Android testing |
| 5 | Complete WebRTC | ~140 LOC Rust/WASM | No |
| 6 | Security audit | Analysis + tracking issue | No |
| 7 | Integration tests | ~100 LOC Rust + run suite | No |
| 8 | Double Ratchet | ~1200 LOC Rust (separate PR) | No |
| 9 | Group Messaging | ~1500 LOC Rust (separate milestone) | No |
| 10 | Protocol Spec | ~3000 lines docs | No |

**Groups 1-4** should be done as a single commit/PR (they're all about getting the
current code to build and work correctly).

**Group 5** is independent and can be done in parallel.

**Group 6** is analysis only — creates issues, doesn't change code.

**Group 7** validates everything.

**Groups 8-10** are future milestones.

---

## FINAL CHECKLIST (Run After Groups 1-7)

```bash
# 1. Rust workspace builds clean
cargo build --workspace

# 2. All unit tests pass
cargo test --workspace

# 3. Clippy clean
cargo clippy --workspace -- -D warnings

# 4. Integration tests pass
cargo test --test integration_e2e
cargo test --test integration_ironcore_roundtrip

# 5. Docker test suite (if available)
cd docker && ./run-all-tests.sh --rust-only

# 6. Android builds
cd android && ./gradlew :app:assembleDebug && ./gradlew test

# 7. iOS builds (requires macOS with Xcode)
cd iOS/SCMessenger && xcodebuild -scheme SCMessenger -sdk iphonesimulator -arch arm64 build

# 8. WASM builds
cd wasm && wasm-pack build --target web
```

