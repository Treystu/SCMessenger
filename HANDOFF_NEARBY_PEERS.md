# Handoff: Nearby Peers Feature Implementation

This document summarizes the current progress and pending tasks for the "Nearby Peers" feature across Rust Core, iOS, and Android.

## Objective

Enable automatic discovery of peers on the local network (LAN) using libp2p's Identify protocol, allowing users to easily add contacts by tapping on discovered nearby users.

---

## 🏗️ What has been done

### 1. Rust Core (`core/`)

- **API Extension**: Added `extract_public_key_from_peer_id` to `IronCore` to derive the Ed25519 public key from a libp2p PeerID.
- **Delegate Update**: Added `on_peer_identified(peer_id, listen_addrs)` to the `CoreDelegate` trait and UniFFI UDL.
- **Event Wiring**: Updated `mobile_bridge.rs` to catch `SwarmEvent::PeerIdentified` and notify the mobile delegates with the peer's listen addresses.
- **Dependencies**: Added `bs58` for PeerID decoding.

### 2. iOS Development (`iOS/`)

- **UI**: Modified `AddContactView` in `ContactsListView.swift` to include a section showing "Nearby Peers" with "Pencil" (Fill) and "Plus" (Quick Add) buttons.
- **Logic**:
  - Implemented `quickAddNearbyPeer` for one-tap contact creation.
  - Updated `MeshRepository.swift` to auto-connect to LAN peers upon identification.
  - Increased GATT identity data limit to 512 bytes in `MeshBLEConstants.swift`.
- **Build Fixes**:
  - Regenerated Swift bindings and updated `api.swift` in all project locations.
  - Rebuilt Rust static library for iOS simulator (`aarch64-apple-ios-sim`) and updated `SCMessengerCore.xcframework`.

### 3. Android Development (`android/`)

- **Update**: Added auto-connect for LAN peers in `MeshRepository.kt`.
- **Optimization**: Limited beacon listeners to top 3 to keep payload size manageable.
- **Verification**: Confirmed successful build via `./gradlew assembleDebug`.

---

## 🏗️ Current Status

### ✅ iOS Build Error (Resolved)

- Rebuilt static library and matched bindings. The `extractPublicKeyFromPeerId` should now be available.

### ✅ Android Build (Verified)

- Build successful.

### ✅ BLE Identity Optimization (Completed)

- Payload size managed via GATT and listener truncation.

---

## 🏁 Next Steps

1. **Simulated/Real Testing**: Run both apps on the same network and verify that they "see" each other in the Nearby list and can "Quick Add" each other.
2. **Relay Verification**: Test messaging between two devices that are NOT on the same network using the GCP relay node.
3. **UI Polish**: Ensure the "Quick Add" feedback (e.g., success toast) is clear to the user.
