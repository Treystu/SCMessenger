# SCMessenger Cross-Platform Unification Plan

> Generated: 2026-02-17 | Status: In Progress

## Executive Summary

Full audit of iOS and Android codebases reveals **one critical networking bug** and **multiple settings parity gaps**. This document tracks the plan and execution status for achieving complete unification.

---

## 🔴 CRITICAL: BLE UUID Mismatch (Networking Broken)

**Impact:** iOS and Android devices **cannot discover each other** via BLE. This is the #1 blocker for cross-platform P2P connectivity.

### Current State

| Component           | Android UUID                           | iOS UUID                               |
| ------------------- | -------------------------------------- | -------------------------------------- |
| **Service**         | `0000DF01-0000-1000-8000-00805F9B34FB` | `6E400001-B5A3-F393-E0A9-E50E24DCCA9E` |
| **Identity Char**   | `0000DF02-...`                         | `6E400004-...` (ID)                    |
| **Message/TX Char** | `0000DF03-...`                         | `6E400002-...` (TX)                    |
| **Sync/RX Char**    | `0000DF04-...`                         | `6E400003-...` (RX)                    |

### Decision: Unify on Android UUIDs

The iOS UUIDs are Nordic UART Service (NUS) UUIDs, which are intended for Nordic's specific UART service. The Android UUIDs are custom and more appropriate for SCMessenger's custom GATT service.

### Fix Plan

- [x] **Step 1:** Update `iOS/Transport/MeshBLEConstants.swift` to use Android UUIDs
- [x] **Step 2:** Map characteristic roles consistently:
  - `DF02` = Identity (read) — peer identity beacon
  - `DF03` = Message (write) — central writes messages to peripheral
  - `DF04` = Sync (notify) — peripheral notifies central of incoming data
- [x] **Step 3:** Update iOS `BLECentralManager.swift` to use new char names
- [x] **Step 4:** Update iOS `BLEPeripheralManager.swift` to use new char names

---

## 🟡 Settings Parity Gaps

### iOS Missing Features (present on Android)

| Feature                                   | Android Location           | Status                               |
| ----------------------------------------- | -------------------------- | ------------------------------------ |
| Service Control (Start/Stop/Status/Stats) | `SettingsScreen.kt`        | [x] Add to `SettingsView.swift`      |
| Transport Toggles (BLE, WiFi, Internet)   | `MeshSettingsScreen.kt`    | [x] Add to `MeshSettingsView`        |
| Relay Budget Slider                       | `MeshSettingsScreen.kt`    | [x] Add to `MeshSettingsView`        |
| Battery Floor Slider                      | `MeshSettingsScreen.kt`    | [x] Add to `MeshSettingsView`        |
| Onion Routing Toggle                      | `PrivacySettingsScreen.kt` | [x] Add to `PrivacySettingsView`     |
| Privacy by Design Notice                  | `PrivacySettingsScreen.kt` | [x] Add to `PrivacySettingsView`     |
| Cover Traffic Placeholder                 | `PrivacySettingsScreen.kt` | [x] Add to `PrivacySettingsView`     |
| Message Padding Placeholder               | `PrivacySettingsScreen.kt` | [x] Add to `PrivacySettingsView`     |
| Timing Obfuscation Placeholder            | `PrivacySettingsScreen.kt` | [x] Add to `PrivacySettingsView`     |
| App Preferences (Notifications)           | `SettingsScreen.kt`        | [x] Add to `SettingsView.swift`      |
| Info Section (Contacts/Messages Count)    | `SettingsScreen.kt`        | [x] Add to `SettingsView.swift`      |
| Power Settings (AutoAdjust, Profiles)     | `PowerSettingsScreen.kt`   | [x] Create `PowerSettingsView.swift` |
| BLE Scan Interval Override                | `PowerSettingsScreen.kt`   | [x] In `PowerSettingsView.swift`     |
| Relay Max Override                        | `PowerSettingsScreen.kt`   | [x] In `PowerSettingsView.swift`     |

### Android Missing Features (present on iOS)

| Feature                       | iOS Location          | Status                                |
| ----------------------------- | --------------------- | ------------------------------------- |
| BLE Identity Rotation Toggle  | `PrivacySettingsView` | [x] Add to `PrivacySettingsScreen.kt` |
| BLE Rotation Interval Display | `PrivacySettingsView` | [x] Add to `PrivacySettingsScreen.kt` |

---

## 🟢 Networking Parity (Already Achieved)

| Transport          | Android                                                 | iOS                                     | Cross-Platform?                      |
| ------------------ | ------------------------------------------------------- | --------------------------------------- | ------------------------------------ |
| **BLE GATT**       | BleScanner, BleGattClient, BleGattServer, BleAdvertiser | BLECentralManager, BLEPeripheralManager | 🔴 Broken (UUID mismatch) → Fixed    |
| **BLE L2CAP**      | BleL2capManager                                         | BLEL2CAPManager                         | ✅ PSM matches (`0x1001`)            |
| **WiFi P2P**       | WiFiAwareTransport, WiFiDirectTransport                 | MultipeerTransport                      | ⚠️ Platform-specific APIs (expected) |
| **Internet/Swarm** | SwarmBridge (via MeshRepository)                        | SwarmBridge (via MeshRepository)        | ✅ Same Rust core                    |
| **Multipeer**      | N/A (Android equivalent: WiFi Direct)                   | MultipeerTransport                      | ⚠️ Apple-only (expected)             |

### WiFi Transport Notes

WiFi Direct (Android) and Multipeer Connectivity (iOS) are platform-specific equivalents. They serve the same role (high-throughput local P2P) but use different APIs. This is **expected and correct** — there is no cross-platform WiFi Direct standard. Cross-platform connectivity for non-BLE is handled via the Internet/Swarm transport using TCP/IP.

---

## Execution Order

1. **🔴 Fix BLE UUID mismatch** (critical networking fix)
2. **Add BLE Identity Rotation to Android** (small, contained change)
3. **Expand iOS SettingsViewModel** (data layer for new settings)
4. **Expand iOS SettingsView** (service control, app prefs, info)
5. **Expand iOS MeshSettingsView** (transport toggles, sliders)
6. **Expand iOS PrivacySettingsView** (onion routing, notice, placeholders)
7. **Create iOS PowerSettingsView** (new view)
8. **Update FEATURE_PARITY.md** (documentation)
9. **Verify builds** (both platforms)

---

## Architecture Notes

### Shared Core (Rust via UniFFI)

Both platforms share the same Rust core providing:

- `IronCore` — Identity & crypto
- `MeshService` — Service lifecycle
- `MeshSettingsManager` — Settings persistence
- `AutoAdjustEngine` — Power management
- `ContactManager` — Contact storage
- `HistoryManager` — Message history
- `LedgerManager` — Connection reputation
- `SwarmBridge` — TCP/IP messaging

### Platform Preferences

|                     | Android                             | iOS                               |
| ------------------- | ----------------------------------- | --------------------------------- |
| **Mesh Settings**   | `MeshSettingsManager` (Rust)        | `MeshSettingsManager` (Rust)      |
| **App Preferences** | `PreferencesRepository` (DataStore) | `UserDefaults` / `@AppStorage`    |
| **ViewModel**       | `SettingsViewModel` (Hilt)          | `SettingsViewModel` (@Observable) |
