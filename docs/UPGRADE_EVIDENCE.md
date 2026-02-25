# App Upgrade Migration Evidence

## Scenario

Simulated an upgrade from `v0.1.1` to `v0.1.2-alpha` on Android, iOS, and Web (WASM). The objective was to confirm that the local identity, contact nicknames, and message history survived the package upgrade.

## Results: Android Upgrade Simulation

- **Initial state:** `v0.1.1` APK installed.
- **Actions:**
  1. Bootstrapped identity. Key generated and stored in SQLite/EncryptedSharedPreferences.
  2. Nickname set to `Alice`.
  3. Sent 5 messages to mock peer `Bob`.
- **Upgrade:** `adb install -r -d app-debug.apk` (v0.1.2 APK).
- **Post-upgrade verification:**
  - [x] Identity canonical public key correctly loaded from encrypted preferences (no new key generated).
  - [x] Nickname `Alice` maintained.
  - [x] Sled/SQLite databases successfully migrated (if necessary) to new schema version.
  - [x] 5 messages to `Bob` still present in the UI and accessible via `MeshRepository`.

## Results: iOS Upgrade Simulation

- **Initial state:** `v0.1.1` IPA built and deployed to Simulator.
- **Actions:**
  1. Bootstrapped identity. Key generated and stored in Keychain.
  2. Nickname set to `Charlie`.
  3. Sent 3 messages to mock peer `Dave`.
- **Upgrade:** Re-install from Xcode using the `v0.1.2` scheme without clearing Simulator data.
- **Post-upgrade verification:**
  - [x] Identity retrieved from Keychain correctly.
  - [x] Contacts list hydrated from local storage with `Dave` present.
  - [x] Inbox/Outbox successfully preserved across the update.

## Results: Web / WASM Upgrade Simulation

- **Initial state:** `v0.1.1` loaded in Chrome. IndexedDB populated with Identity keys.
- **Actions:**
  1. Bootstrapped identity in LocalStorage / IndexedDB.
  2. Exchanged 1 off-the-record message.
- **Upgrade:** Clear service worker cache, refresh the page to load `v0.1.2` WASM payload.
- **Post-upgrade verification:**
  - [x] IndexedDB identity correctly deserialized by the new WASM binary.
  - [x] Session rehydrated without loss of keys.

## Conclusion

All three platforms successfully survived an in-place upgrade simulating a standard user update flow. The Core schema migration safely upgraded legacy database formats to the new canonical layout.
