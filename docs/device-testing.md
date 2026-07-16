# SCMessenger Physical Device Testing Guide

This document describes the manual field procedures used to verify that SCMessenger's proximity transports work on real hardware. These procedures are the release evidence for Track 1 (native hardware) and Track 2 (DTN) work.

## Prerequisites

### Hardware
- Two Android devices (BLE; optionally Wi-Fi Aware and/or Wi-Fi Direct capable).
- Two iOS devices (BLE + Apple Multipeer).
- One "mule" device (Android or iOS) for the 3-device DTN scenario.
- All devices charged to > 50% and left awake during tests.

### Build artifacts
- Android: `android/app/build/outputs/apk/debug/app-debug.apk` (see `android/README.md`).
- iOS: `SCMessengerCore.xcframework` + Xcode build of `ios/SCMessenger/SCMessenger.xcodeproj`.

### Environment
- Devices physically near each other (0.5–3 m for BLE, 5–30 m for Wi-Fi Aware/Direct).
- Wi-Fi and Bluetooth enabled.
- Location permission granted on Android (required for BLE scans and Wi-Fi Aware).
- Nearby Devices / Bluetooth permission granted on iOS.

---

## Test 1 — Two-Device BLE Message Exchange

### Goal
Verify that two SCMessenger peers discover each other over BLE and exchange an encrypted text message end-to-end.

### Procedure
1. Install the app on Device A and Device B.
2. On each device, create a new identity and note the public Safety Number.
3. Enable **BLE only** in transport settings (disable Wi-Fi Aware / Multipeer if available).
4. Place both devices within 1 m of each other.
5. On Device A, compose a message to Device B's contact and send.
6. Wait up to 60 seconds for BLE discovery + transport escalation.

### Expected Result
- Device B receives the message.
- Message status on Device A changes from `Sending` → `Sent` → `Delivered`.
- Tapping the message on either device shows the same `message_id` and transport `Ble`.

### Verification command
```bash
# On each Android device via adb:
adb -s <serial> logcat -d | grep -E "BleTransport|on_proximity_data_received|send_proximity_packet" | tail -20
```

---

## Test 2 — Android Wi-Fi Aware Data Path

### Goal
Verify that Android Wi-Fi Aware discovery and data-path establishment work and that libp2p traffic can ride the Aware IPv6 link.

### Requirements
- Both Android devices must support Wi-Fi Aware (`PackageManager.FEATURE_WIFI_AWARE`).
- Android 9+ (API 28+); Android 15+ recommended.

### Procedure
1. Install the debug APK on both devices.
2. Enable **Wi-Fi Aware** in transport settings.
3. Keep BLE enabled for initial peer discovery.
4. Place devices 2–5 m apart with clear line of sight.
5. Send a message from Device A to Device B.

### Expected Result
- Logcat shows `WifiAwareTransport: publish/session` and `WifiAwareTransport: dataPathInfo` events.
- After data-path establishment, `SwarmBridge.get_peers()` on each device lists the other's `PeerId`.
- Message is marked as delivered over transport `WifiAware`.

### Verification commands
```bash
adb -s <serial-a> shell am start-foreground-service \
  -n com.scmessenger.android/.service.MeshForegroundService
adb -s <serial-a> logcat -d | grep -iE "WifiAware|DataPathActive|SwarmBridge" | tail -30
adb -s <serial-b> logcat -d | grep -iE "WifiAware|DataPathActive|SwarmBridge" | tail -30
```

---

## Test 3 — Android Wi-Fi Direct Group + Message

### Goal
Verify Wi-Fi Direct group formation and message exchange.

### Procedure
1. Enable **Wi-Fi Direct** in transport settings on both Android devices.
2. Initiate a message from Device A.
3. Observe group-owner negotiation and peer connection.

### Expected Result
- `WifiDirectTransport` logs show group created/joined and peer connected.
- Message delivers over `WifiDirect` transport.

---

## Test 4 — iOS Multipeer Message Exchange

### Goal
Verify iOS-to-iOS messaging over Apple Multipeer Connectivity.

### Procedure
1. Build and install the iOS app on two devices (see `ios/README.md`).
2. Enable **Multipeer** in transport settings.
3. Send a message from Device A to Device B.

### Expected Result
- `MultipeerTransport` discovers the peer via `MCSession` invitation.
- Message delivers over `Multipeer` transport.
- The app remains responsive when backgrounded for up to 30 seconds.

---

## Test 5 — Three-Device Sneakernet / Mule (DTN)

### Goal
Verify Store-and-Carry: Device A sends a message to Device B while B is offline; a mule device carries the message and delivers it when B comes back in range.

### Roles
- **Sender** (Device A): online, can reach the Mule.
- **Mule** (Device M): moves between A and B.
- **Receiver** (Device B): offline during send, later comes online near Mule.

### Procedure
1. All three devices create identities and are contacts of each other.
2. Disable all transports on Device B or power it off.
3. On Device A, compose a message to Device B and send.
4. Wait until Device A shows the message as `StoredForCarry` or `Pending`.
5. Move Device M within BLE range of Device A and wait for custody transfer (log shows `DriftCustody` or `StoreAndCarry` handoff).
6. Move Device M away from Device A and within BLE range of Device B.
7. Re-enable transports on Device B.

### Expected Result
- Device M logs show `on_drift_custody_received` for the message.
- Device B eventually receives the message after Device M comes in range.
- Message status on Device A eventually transitions to `Delivered`.
- `drift custody` telemetry shows one custody transfer event.

### Verification
```bash
adb -s <mule-serial> logcat -d | grep -iE "Drift|StoreAndCarry|custody" | tail -40
adb -s <receiver-serial> logcat -d | grep -iE "Drift|on_proximity_data_received|MessageStatus::Delivered" | tail -40
```

---

## Test 6 — Process Death & Resume

### Goal
Verify that custody survives app kill and reboot (T2.3).

### Procedure
1. Start the 3-device DTN scenario above.
2. After Device M receives custody, force-stop the SCMessenger app (or reboot Device M).
3. Relaunch the app and move Device M near Device B.

### Expected Result
- Device M restores custody from persistent storage.
- Device B still receives the message after resume.

---

## Test 7 — Transport Escalation & Fallback

### Goal
Verify that SmartTransportRouter escalates from BLE to a higher-bandwidth transport when available and falls back gracefully.

### Procedure
1. Pair two Android devices via BLE.
2. Enable both BLE and Wi-Fi Aware.
3. Send a large attachment (e.g., a 50 KB photo) or a burst of messages.
4. After delivery, disable Wi-Fi Aware mid-session.

### Expected Result
- Initial large payload uses `WifiAware`.
- After Wi-Fi Aware is disabled, subsequent messages fall back to `Ble`.
- No duplicate messages are delivered (dedup verified).

---

## Logging & Telemetry

All device tests should capture:

```bash
adb logcat -d > scmessenger-<device>-<test>.log
```

Key log tags:
- `BleTransport`
- `WifiAwareTransport`
- `WifiDirectTransport`
- `MultipeerTransport`
- `SwarmBridge`
- `DriftService`
- `SmartTransportRouter`
- `MessageRepository`

---

## Pass/Fail Criteria

A release candidate passes physical device testing when:

1. Test 1 (BLE exchange) passes on Android → Android, iOS → iOS, and Android → iOS.
2. Test 2 passes on two Wi-Fi Aware capable Android devices.
3. Test 5 (sneakernet) passes with at least one Android mule.
4. Test 6 passes (custody survives process death).
5. No crashes or ANRs during any test.
6. No duplicate message delivery.
7. Message content and status are consistent on all devices.

---

## Known Limitations

- iOS does not support Wi-Fi Aware or Wi-Fi Direct; use Multipeer or BLE.
- Wi-Fi Aware availability varies by OEM and Android version.
- BLE background delivery on iOS is best-effort and may be delayed by iOS scheduling.
- Battery optimization dialogs on Android may need to be accepted for reliable background operation.
