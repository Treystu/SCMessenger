> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# Handoff: Nearby Peers Feature Implementation

This document summarizes the current progress and pending tasks for the "Nearby Peers" feature across Rust Core, iOS, and Android.

## [Needs Revalidation] Objective

Enable automatic discovery of peers on the local network (LAN) using libp2p's Identify protocol, allowing users to easily add contacts by tapping on discovered nearby users.

---

## [Needs Revalidation] 🏗️ What has been done

### [Needs Revalidation] 1. Rust Core (`core/`)

- **API Extension**: Added `extract_public_key_from_peer_id` to `IronCore` to derive the Ed25519 public key from a libp2p PeerID.
- **Delegate Update**: Added `on_peer_identified(peer_id, listen_addrs)` to the `CoreDelegate` trait and UniFFI UDL.
- **Event Wiring**: Updated `mobile_bridge.rs` to catch `SwarmEvent::PeerIdentified` and notify the mobile delegates with the peer's listen addresses.
- **Dependencies**: Added `bs58` for PeerID decoding.

### [Needs Revalidation] 2. iOS Development (`iOS/`)

- **UI**: Modified `AddContactView` in `ContactsListView.swift` to include a section showing "Nearby Peers" with "Pencil" (Fill) and "Plus" (Quick Add) buttons.
- **Logic**:
  - Implemented `quickAddNearbyPeer` for one-tap contact creation.
  - Updated `MeshRepository.swift` to auto-connect to LAN peers upon identification.
  - Increased GATT identity data limit to 512 bytes in `MeshBLEConstants.swift`.
- **Build Fixes**:
  - Regenerated Swift bindings and updated `api.swift` in all project locations.
  - Rebuilt Rust static library for iOS simulator (`aarch64-apple-ios-sim`) and updated `SCMessengerCore.xcframework`.

### [Needs Revalidation] 3. Android Development (`android/`)

- **Update**: Added auto-connect for LAN peers in `MeshRepository.kt`.
- **Optimization**: Limited beacon listeners to top 3 to keep payload size manageable.
- **Verification**: Confirmed successful build via `./gradlew assembleDebug`.

---

## [Needs Revalidation] 🏗️ Current Status

### [Needs Revalidation] ✅ iOS Build Error (Resolved)

- Rebuilt static library and matched bindings. The `extractPublicKeyFromPeerId` should now be available.

### [Needs Revalidation] ✅ Android Build (Verified)

- Build successful.

### [Needs Revalidation] ✅ BLE Identity Optimization (Completed)

- Payload size managed via GATT and listener truncation.

---

## [Needs Revalidation] 🏁 Next Steps

1. **Simulated/Real Testing**: Run both apps on the same network and verify that they "see" each other in the Nearby list and can "Quick Add" each other.
2. **Relay Verification**: Test messaging between two devices that are NOT on the same network using the GCP relay node.
3. **UI Polish**: Ensure the "Quick Add" feedback (e.g., success toast) is clear to the user.
