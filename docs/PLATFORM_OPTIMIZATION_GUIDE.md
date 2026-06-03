# SCMessenger Per-Platform Optimization Guide

> Version: v0.2.1-alpha  
> Scope: UX, interoperability, setup simplification, and OS-specific optimizations for each platform variant.

---

## Table of Contents

1. [Ubuntu/Debian Linux (Dedicated Hardware)](#1-ubuntudebian-linux-dedicated-hardware)
2. [Windows (Desktop/Laptop)](#2-windows-desktoplaptop)
3. [macOS (Desktop/Laptop)](#3-macos-desktoplaptop)
4. [Android (Mobile)](#4-android-mobile)
5. [iOS (Mobile)](#5-ios-mobile)
6. [WASM/Browser]((#6-wasmbrowser)
7. [Cross-Platform Interoperability Matrix](#7-cross-platform-interoperability-matrix)

---

## 1. Ubuntu/Debian Linux (Dedicated Hardware)

### Primary Use Pattern

A headless or minimally graphical node running 24/7 as a **stable relay and DHT anchor**. Typically deployed on a home server, VPS, or Raspberry Pi-class device. This is the backbone of the mesh — always-on, always-reachable, always-relaying. Users expect `apt install` simplicity and `systemd` reliability.

### Key Optimizations

- **Full transport stack**: TCP + QUIC + mDNS + DNS-based discovery — all enabled. The `IronCoreBehaviour` in `core/src/transport/behaviour.rs` compiles mDNS and UPnP for `not(wasm32) and not(android)` targets, so Linux gets both.
- **Aggressive relay custody**: `max_relay_budget: 500`, `relay_enabled: true`. Every node is a mandatory relay server (`relay_server: relay::Behaviour` always present).
- **Persistent Kademlia DHT**: The DHT record store (`kad::store::MemoryStore`) should survive restarts. Currently in-memory — needs sled-backed persistence (see Missing Features).
- **mDNS + DNS-SD auto-discovery**: libp2p mDNS (`Toggle<mdns::tokio::Behaviour>`) auto-discovers LAN peers. On headless servers with no multicast (containers, cloud VMs), mDNS should be toggled off via `DiscoveryConfig`.
- **UPnP auto port-mapping**: `upnp: upnp::tokio::Behaviour` attempts automatic port forwarding for NAT traversal. Critical for home-server nodes behind consumer routers.
- **Cover traffic + timing obfuscation**: Server nodes are ideal candidates for running cover traffic and timing obfuscation since they have spare CPU/bandwidth and no battery concern.
- **Auto-adjust profile**: `AdjustmentProfile::Maximum` at all times (no battery, always on AC).

### Setup Simplification

1. **`.deb` package** with `apt` repository — single command install.
2. **`systemd` unit file** shipped with the package:
   - `Type=notify` with `sd_notify` readiness signaling.
   - `WatchdogSec=60` for automatic restart on hang.
   - `Restart=on-failure` with `RestartSec=5`.
   - `CapabilityBoundingSet=CAP_NET_BIND_SERVICE CAP_NET_RAW` (for QUIC/UDP).
   - `AmbientCapabilities` for non-root port binding.
3. **Socket activation** support (already scaffolded in `desktop_bridge` via `check_socket_activation()`): launch on incoming connection to save idle resources.
4. **Auto-detect BLE adapter**: The `desktop_bridge/src/ble.rs` module uses BlueZ D-Bus (`zbus`) to detect and manage BLE. On headless servers without Bluetooth, gracefully skip BLE setup.
5. **Firewall auto-config**: Post-install script that offers to open TCP/4001 and UDP/4001 (libp2p defaults) via `ufw` or `iptables`.
6. **XDG paths**: `desktop_bridge/src/xdg_paths.rs` already resolves `$XDG_DATA_HOME/scmessenger` and `$XDG_CONFIG_HOME/scmessenger` — no custom paths needed.

### Interoperability Concerns

- **QUIC support**: Linux `sysctl` may need `net.core.rmem_max` and `net.core.wmem_max` increased for QUIC UDP buffers. The `patch/libp2p-quic/` crate handles this at the transport level but OS-level tuning helps.
- **Docker/container networking**: mDNS multicast does not traverse Docker bridge networks. Nodes in containers must use DNS-based bootstrap peers or `--network=host`.
- **BlueZ version**: BLE D-Bus API varies across BlueZ versions (5.50+ required for LEAdvertisingManager). D-Bus method calls may fail on older distributions.

### Missing Features

- [ ] **systemd unit file** — not yet shipped with the build.
- [ ] **sled-backed DHT store** — currently `MemoryStore`, loses all DHT records on restart.
- [ ] **Auto-update mechanism** — no self-update or package repo automation.
- [ ] **`apt` repository** — no `.deb` packaging pipeline.
- [ ] **Log rotation config** — no `logrotate` snippet shipped.
- [ ] **QUIC transport** — scaffolded in `patch/libp2p-quic/` but not yet integrated into swarm setup.

### Recommended Config

```json
{
  "relay_enabled": true,
  "max_relay_budget": 500,
  "battery_floor": 0,
  "ble_enabled": true,
  "wifi_aware_enabled": false,
  "wifi_direct_enabled": false,
  "internet_enabled": true,
  "discovery_mode": "Normal",
  "onion_routing": false,
  "cover_traffic_enabled": true,
  "message_padding_enabled": true,
  "timing_obfuscation_enabled": true,
  "notifications_enabled": true,
  "notify_dm_enabled": true,
  "notify_dm_request_enabled": true,
  "notify_dm_in_foreground": true,
  "notify_dm_request_in_foreground": true,
  "sound_enabled": false,
  "badge_enabled": false
}
```

> WiFi Aware/Direct are Android-specific; `false` on Linux. Sound/badge off for headless. Cover traffic + padding + timing obfuscation enabled as server nodes are ideal privacy mixers.

---

## 2. Windows (Desktop/Laptop)

### Primary Use Pattern

A **desktop client** that the user opens occasionally, likely behind home/office NAT. The user is not a network operator — they want install-and-forget messaging. Two deployment modes:

1. **Native CLI daemon** (`scmessenger-cli`) running as a Windows service or tray app, with WASM browser UI connecting via WebSocket to `127.0.0.1:9002`.
2. **WSL2 bridge** — running the Linux CLI daemon inside WSL2, with the browser UI connecting across the WSL networking boundary.

### Key Optimizations

- **Zero-config startup**: The WASM client defaults to `IronCoreMode::Daemon` and connects to `ws://127.0.0.1:9002/ws`. The CLI daemon should auto-start on login.
- **Windows Firewall auto-config**: The installer must add a firewall rule for TCP/9002 (daemon WebSocket) and TCP/4001 + UDP/4001 (libp2p). No manual `netsh` by the user.
- **NAT traversal via relay**: Windows nodes behind NAT rely on Circuit Relay v2 (`relay_client` + `dcutr` for hole punching). The `autonat` behaviour probes reachability.
- **System tray integration**: The CLI daemon should minimize to system tray with connectivity status. Currently the `DesktopBridge` has `TrayStatus` and `TrayIconState` abstractions but no Windows implementation.
- **No BLE on Windows**: The `desktop_bridge` BLE module returns `"BLE adapter info is only supported on Linux"` on non-Linux targets. BLE mesh transport is unavailable.
- **AdjustmentProfile**: Desktop assumed AC power → `Maximum` when screen on, `High` when charging, `Standard` on battery.

### Setup Simplification

1. **MSIX or NSIS installer** that:
   - Installs the CLI daemon binary.
   - Creates a startup shortcut (or registers as Windows Service).
   - Adds firewall exceptions.
   - Starts the daemon on port 9002.
   - Opens `localhost:9002` in the default browser.
2. **QR-pairing flow**: For initial peer discovery, generate a QR code with the node's PeerId + relay addresses. The mobile app scans it. Zero typing.
3. **WSL2 auto-detect**: If no native daemon is found, the WASM client could probe `wsl.localhost` addresses for a running daemon. The `DaemonBridge` reconnection logic (exponential backoff, 5 max attempts) already handles transient disconnections.

### Interoperability Concerns

- **WebSocket port conflict**: Port 9002 may be in use. The daemon should support configurable port or auto-increment on bind failure.
- **WSL2 networking changes**: Microsoft has altered WSL2 networking across Windows builds (mirrored mode vs. NAT bridge). The WASM `DaemonBridge` URL must adapt.
- **No mDNS on Windows**: libp2p mDNS is compiled for `not(wasm32) and not(android)` but uses Tokio-based mDNS which may not work reliably on Windows. Bonjour/mDNS responder may need to be installed.
- **Path separators**: Configuration and storage paths use Windows `\` separators. The Rust `std::path` handles this, but the WASM `MeshSettingsManager` reads/writes JSON with `std::fs` — works fine on Windows.

### Missing Features

- [ ] **Windows installer** (MSIX/NSIS) — no packaging pipeline.
- [ ] **Windows Service registration** — CLI daemon can't run as a proper Windows service.
- [ ] **System tray icon** — `TrayStatus` exists in `desktop_bridge` but no Windows systray implementation.
- [ ] **Windows Firewall auto-config** — no `netsh` automation.
- [ ] **Native Windows BLE** — not supported; `desktop_bridge` returns errors.
- [ ] **Windows push notifications** — no integration with Windows Notification Centre.

### Recommended Config

```json
{
  "relay_enabled": true,
  "max_relay_budget": 200,
  "battery_floor": 15,
  "ble_enabled": false,
  "wifi_aware_enabled": false,
  "wifi_direct_enabled": false,
  "internet_enabled": true,
  "discovery_mode": "Normal",
  "onion_routing": false,
  "cover_traffic_enabled": false,
  "message_padding_enabled": true,
  "timing_obfuscation_enabled": false,
  "notifications_enabled": true,
  "notify_dm_enabled": true,
  "notify_dm_request_enabled": true,
  "notify_dm_in_foreground": false,
  "notify_dm_request_in_foreground": true,
  "sound_enabled": true,
  "badge_enabled": true
}
```

> BLE/WiFi transports unavailable on Windows. `battery_floor: 15` for laptop users. Cover traffic disabled (intermittent usage, don't waste bandwidth). Message padding always on (minimal overhead, strong privacy).

---

## 3. macOS (Desktop/Laptop)

### Primary Use Pattern

Similar to Windows — a desktop/laptop client that runs as a background daemon with a browser UI. The key differentiators are Apple-specific APIs: Keychain, Bonjour, CoreBluetooth, and App Sandbox.

### Key Optimizations

- **Bonjour (mDNS) discovery**: macOS has native Bonjour support. The libp2p mDNS behaviour works well on macOS without additional software (unlike Windows). LAN peer discovery is automatic.
- **Keychain for identity storage**: The `IdentityKeys` (Ed25519 keypair) should be stored in macOS Keychain via `Security.framework`, not on disk. This provides hardware-backed protection and survives app reinstallation. Currently, identity is stored via sled — needs Keychain bridge.
- **CoreBluetooth for BLE**: macOS supports CoreBluetooth, but the `desktop_bridge` BLE module only works on Linux (BlueZ D-Bus). A macOS BLE bridge using `IOBluetooth` or CoreBluetooth framework is needed.
- **Apple Silicon native build**: `aarch64-apple-darwin` target for M1/M2/M3 Macs. Must be in the CI matrix. The Rust build should produce a universal binary or architecture-specific.
- **App Sandbox / Hardened Runtime**: If distributed via Mac App Store, the app runs in a sandbox. Network access requires `com.apple.security.network.client` and `com.apple.security.network.server` entitlements. BLE requires `com.apple.security.device.bluetooth`.
- **Power management integration**: `desktop_bridge/src/power.rs` doesn't have macOS-specific battery detection. Should use `IOKit` (`IOPSCopyPowerSourcesInfo`) instead of `/sys/class/power_supply`.
- **NEPacketTunnelProvider**: macOS 12+ supports Network Extensions. Could provide a VPN-style always-on mesh tunnel similar to iOS.

### Setup Simplification

1. **`.app` bundle**: Drag-to-Applications install. The daemon binary and WASM UI bundled together.
2. **LaunchAgent plist**: Auto-start on login via `~/Library/LaunchAgents/com.scmessenger.daemon.plist`.
2. **Xcode project**: For the native shell that hosts the daemon process and system tray.
3. **Keychain-backed identity**: On first launch, generate identity and store in Keychain. No files on disk.

### Interoperability Concerns

- **App Store vs. DMG distribution**: App Store requires sandbox; DMG does not. BLE mesh and local server binding may be restricted in sandboxed builds.
- **Code signing + notarization**: Required for Gatekeeper. The Rust binary must be signed and notarized.
- **Rosetta compatibility**: x86_64 builds work on Apple Silicon via Rosetta, but native ARM is preferred for libp2p crypto operations.

### Missing Features

- [ ] **macOS Keychain identity storage** — identity currently in sled.
- [ ] **macOS-native BLE bridge** — `IOBluetooth`/CoreBluetooth bridge not implemented.
- [ ] **macOS power management** — no `IOKit` battery detection.
- [ ] **LaunchAgent auto-start** — no plist generation.
- [ ] **`.app` bundle packaging** — no macOS packaging pipeline.
- [ ] **Code signing/notarization** — no CI step for this.

### Recommended Config

```json
{
  "relay_enabled": true,
  "max_relay_budget": 200,
  "battery_floor": 15,
  "ble_enabled": true,
  "wifi_aware_enabled": false,
  "wifi_direct_enabled": false,
  "internet_enabled": true,
  "discovery_mode": "Normal",
  "onion_routing": false,
  "cover_traffic_enabled": false,
  "message_padding_enabled": true,
  "timing_obfuscation_enabled": false,
  "notifications_enabled": true,
  "notify_dm_enabled": true,
  "notify_dm_request_enabled": true,
  "notify_dm_in_foreground": false,
  "notify_dm_request_in_foreground": true,
  "sound_enabled": true,
  "badge_enabled": true
}
```

> BLE enabled (CoreBluetooth available on most Macs). mDNS works natively via Bonjour. Message padding on, cover traffic off (desktop usage pattern).

---

## 4. Android (Mobile)

### Primary Use Pattern

A **mobile mesh node** carried by the user. Intermittent connectivity, battery-constrained, screen-off most of the time. The mesh must stay alive in the background via a Foreground Service, but minimize CPU/radio usage. BLE and/or WiFi-Direct provide local mesh transport when internet is unavailable.

### Key Optimizations

- **Foreground Service**: `MeshForegroundService` (Kotlin) keeps the mesh alive with a persistent notification. Android 12+ requires `FOREGROUND_SERVICE` permission and type declaration (`connectedDevice`, `dataSync`, or `specialUse`).
- **BLE transport**: Android's `BLETransportManager` uses Android BLE APIs. The Rust `AutoAdjustEngine` dynamically tunes BLE scan/advertise intervals:
  - `Maximum`: scan 100ms, advertise 20ms (screen on, charging, WiFi)
  - `Minimal`: scan 5120ms, advertise 2000ms (battery < 10%)
- **WiFi Aware (Nan)**: For peer-to-peer messaging without infrastructure. Tuned similarly to BLE.
- **No libp2p mDNS on Android**: The `IronCoreBehaviour` gates mDNS behind `not(target_os = "android")` — Android uses `NsdManager` for LAN discovery instead.
- **FCM push for relay wake**: When the app is in background, FCM can wake it for incoming relay messages. Without FCM, messages queue on the relay until the next app open. The core supports relay custody store (`RelayCustodyStore` with `CustodyEnforcement` and `CustodyCompatMode`).
- **Auto-adjust engine**: `AutoAdjustEngine::compute_adjustments()` reads `DeviceProfile` (battery, charging, WiFi, motion, screen) and selects `AdjustmentProfile`. This drives BLE scan window, relay rate, and transport selection.
- **Boot receiver**: `BootReceiver` auto-starts the foreground service on device boot.
- **ANR watchdog**: `AnrWatchdog` monitors for Application Not Responding states that could trigger system kill.

### Setup Simplification

1. **One-time permission flow**: Request `BLUETOOTH_SCAN`, `BLUETOOTH_ADVERTISE`, `BLUETOOTH_CONNECT` (Android 12+), `ACCESS_FINE_LOCATION` (for BLE), `FOREGROUND_SERVICE`, `POST_NOTIFICATIONS` (Android 13+).
2. **QR-onboarding**: Scan QR code from a desktop node → instantly get relay addresses + bootstrap peers.
3. **Automatic transport selection**: No user choice needed — `SmartTransportRouter` (iOS has equivalent) selects best transport. Android's `PlatformNetworking` detects WiFi/cellular automatically.
4. **Battery optimization exemption**: Prompt user to exempt SCMessenger from battery optimization ("Allow background activity").

### Interoperability Concerns

- **Android 12+ BLE permissions**: The new fine-grained Bluetooth permissions (`BLUETOOTH_SCAN` with `neverForLocation` attribute) affect discovery behavior.
- **WiFi Aware availability**: Not available on all devices. `MeshServiceConfig::enable_wifi_aware` defaults to `true` but runtime check must disable gracefully.
- **Doze mode**: Even with foreground service, Doze can restrict network access. High-priority FCM messages bypass Doze.
- **Process death**: Android may kill the service under extreme memory pressure. `ServiceHealthMonitor` tracks service health, but WALA guarantees require the service to self-restart.
- **NsdManager vs mDNS**: Android uses its own NsdManager for LAN discovery; this may not be directly compatible with libp2p mDNS protocol. Cross-platform LAN discovery between Android and desktop nodes may fail.

### Missing Features

- [ ] **FCM integration** — no Firebase Cloud Messaging push notification receiver.
- [ ] **WiFi-Direct transport** — `wifi_direct_enabled` field exists but no implementation.
- [ ] **Background work manager integration** — currently raw Foreground Service; should use WorkManager for periodic sync.
- [ ] **BLE L2CAP** — available on Android 8.1+ for higher throughput; not used.
- [ ] **NearbyShare/Android Beam** — for out-of-band key exchange.

### Recommended Config

```json
{
  "relay_enabled": true,
  "max_relay_budget": 100,
  "battery_floor": 20,
  "ble_enabled": true,
  "wifi_aware_enabled": true,
  "wifi_direct_enabled": false,
  "internet_enabled": true,
  "discovery_mode": "Normal",
  "onion_routing": false,
  "cover_traffic_enabled": false,
  "message_padding_enabled": true,
  "timing_obfuscation_enabled": false,
  "notifications_enabled": true,
  "notify_dm_enabled": true,
  "notify_dm_request_enabled": true,
  "notify_dm_in_foreground": false,
  "notify_dm_request_in_foreground": true,
  "sound_enabled": true,
  "badge_enabled": true
}
```

> Conservative relay budget (100/hour). WiFi-Direct disabled (unstable). Cover traffic off (battery). Message padding on. Battery floor 20% — stop non-essential mesh below this.

---

## 5. iOS (Mobile)

### Primary Use Pattern

Very similar to Android but under **much stricter background execution constraints**. iOS has no true foreground service. The mesh must survive using CoreBluetooth background modes, BGTaskScheduler, and optionally location-based triggers. This is the most challenging platform for always-on mesh.

### Key Optimizations

- **CoreBluetooth background modes**: The `IosBackgroundStrategy` (Rust core) manages `BackgroundMode::BluetoothCentral` and `BackgroundMode::BluetoothPeripheral`. iOS allows BLE to operate in the background if the app declares `bluetooth-central` and `bluetooth-peripheral` in `UIBackgroundModes`. The BLE scan/advertise intervals are automatically throttled by iOS when backgrounded.
- **BGTaskScheduler**: `MeshBackgroundService` (Swift) registers:
  - `BGAppRefreshTask` (quick sync, ~30s, minimum interval 15 minutes via `fetch_interval_secs: 900` in `IosBackgroundConfig`)
  - `BGProcessingTask` (longer maintenance, ~minutes, system decides when to run)
- **SmartTransportRouter**: iOS has a full Swift transport router that races BLE, Multipeer, and internet in parallel with 500ms timeout fallback. Transport health tracking biases future selections toward historically successful transports.
- **Adaptive reporting**: `IosPlatformBridge` throttles battery and motion reports adaptively (5-minute intervals at 95%+ battery, 30-second intervals below 20%).
- **BLE L2CAP**: `BLEL2CAPManager.swift` provides L2CAP channel-based transport for higher throughput (6–10x faster than GATT) when both peers support it.
- **MultipeerConnectivity**: `MultipeerTransport.swift` provides WiFi+Bluetooth-based local mesh via Apple's framework. No manual WiFi-Direct needed.
- **Location-based background**: Optional `BackgroundMode::Location` with `LocationAccuracy::ReducedAccuracy` can keep the app alive in background at very low power cost.
- **Apple Push (APNs)**: For relay wake-up. More reliable than BGTaskScheduler for real-time delivery.
- **mDNS via Bonjour**: `mDNSServiceDiscovery.swift` wraps `NetServiceBrowser` for LAN peer discovery. Works in background if `bluetooth-central` mode is active.

### Setup Simplification

1. **One-time permission flow**: Bluetooth (with `CBPeripheralManager` advertisement), Notifications, and optional Location ("Allow While Using" or "Always" for background mesh).
2. **Background mode guidance**: `NotificationGuidanceView.swift` explains to users why certain permissions are needed.
3. **QR-onboarding**: Same as Android — scan from desktop.
4. **Auto-connect on app open**: The `MeshRepository` auto-starts BLE and mesh operations when the app enters foreground.

### Interoperability Concerns

- **Background execution time**: iOS strictly limits background execution. BLE central/peripheral modes keep the app alive but with throttled scan intervals. The 15-minute minimum for BGAppRefreshTask means real-time mesh can't rely on it.
- **BLE state restoration**: `CoreBluetoothState::Restricted` means the user has denied Bluetooth access. The `IosBackgroundStrategy` checks `can_run_ble_central_background()` and `can_run_ble_peripheral_background()` to gate operations.
- **No WiFi Aware on iOS**: Apple has no equivalent to Android WiFi Aware (NAN). MultipeerConnectivity is the closest but requires user interaction to accept connections in some configurations.
- **App Store review**: Apple may reject apps that use location purely for background keepalive. The `IosBackgroundConfig` defaults to `LocationAccuracy::Disabled` and `allow_always_location: false`.
- **NEPacketTunnelProvider**: Could provide a "VPN" tunnel for always-on mesh. This requires a Network Extension target and special entitlements. Not yet implemented.

### Missing Features

- [ ] **APNs integration** — no Apple Push Notification service receiver.
- [ ] **NEPacketTunnelProvider** — no VPN-style always-on mesh tunnel.
- [ ] **Sign in with Apple** — optional identity verification not implemented.
- [ ] **Background transfer (URLSession)** — for downloading queued messages while suspended.
- [ ] **CoreSpotlight indexing** — for searching messages in iOS Spotlight.

### Recommended Config

```json
{
  "relay_enabled": true,
  "max_relay_budget": 50,
  "battery_floor": 25,
  "ble_enabled": true,
  "wifi_aware_enabled": false,
  "wifi_direct_enabled": false,
  "internet_enabled": true,
  "discovery_mode": "Cautious",
  "onion_routing": false,
  "cover_traffic_enabled": false,
  "message_padding_enabled": true,
  "timing_obfuscation_enabled": false,
  "notifications_enabled": true,
  "notify_dm_enabled": true,
  "notify_dm_request_enabled": true,
  "notify_dm_in_foreground": false,
  "notify_dm_request_in_foreground": true,
  "sound_enabled": true,
  "badge_enabled": true
}
```

> Very conservative relay budget (50/hour). Higher battery floor (25%). Discovery Cautious (less aggressive scanning). WiFi Aware/Direct N/A on iOS. Cover traffic off entirely. Message padding on.

---

## 6. WASM/Browser

### Primary Use Pattern

A **thin client** that runs in a browser tab. Two modes:

1. **Daemon mode** (`IronCoreMode::Daemon`): The WASM client delegates all operations to a local CLI daemon via JSON-RPC over WebSocket (`ws://127.0.0.1:9002/ws`). The browser is just a UI shell.
2. **Full mode** (`IronCoreMode::Full`): The WASM client runs the core directly in-browser, with WebRTC for direct peer connections and WebSocket relay for server-mediated transport.

The WASM layer has its own `MeshSettings` that mirrors the core's, with web-specific defaults.

### Key Optimizations

- **WebRTC data channels**: `WebRtcTransport` (in `wasm/src/transport.rs`) creates `RtcPeerConnection` + `RtcDataChannel` for browser-to-browser direct messaging. SDP offer/answer exchange happens via the signalling channel (relay WebSocket). ICE candidates are gathered via `onicecandidate`.
- **WebSocket relay**: `WebSocketRelay` connects to known relay servers (`wss://relay.scmessenger.local` by default). Binary frames (`ArrayBuffer`) for efficiency.
- **Daemon bridge**: `DaemonBridge` (in `wasm/src/daemon_bridge.rs`) provides full JSON-RPC 2.0 client with:
  - Automatic request ID management (`AtomicU64`).
  - Pending request tracking with response callbacks.
  - Exponential backoff reconnection (5 max attempts, 1s initial interval).
  - Server-push notification handling.
- **Web-specific defaults**: The WASM `MeshSettings::default()` explicitly disables BLE, WiFi Aware, WiFi Direct, and sets `battery_floor: 0` (web = always plugged in).
- **IndexedDB persistence**: Currently identity and settings use in-memory storage or `localStorage`. IndexedDB would provide structured, larger-capacity persistence.
- **Service Worker**: For background message receipt when the tab is hidden/closed. Not yet implemented.
- **`tracing_wasm`**: Browser console logging via `tracing_wasm::set_as_global_default()`.

### Setup Simplification

1. **One-click**: Open `https://app.scmessenger.local` or `file:///path/to/ui/index.html`. No installation.
2. **Auto-daemon detection**: Probe `ws://127.0.0.1:9002/ws` — if a daemon responds, switch to Daemon mode automatically.
3. **Identity in localStorage**: On first visit, generate identity and store in `localStorage` (base64-encoded). No server-side storage.

### Interoperability Concerns

- **No direct P2P without WebRTC**: WASM cannot open TCP/UDP sockets. WebRTC is the only browser P2P path, and it requires STUN/TURN for NAT traversal (unless both peers are on the same LAN).
- **Daemon bridge requires local daemon**: In Daemon mode, the CLI daemon must already be running. If not, the `DaemonBridge` reconnection will exhaust attempts (5) and stay `Disconnected`.
- **CORS**: WebSocket connections to `127.0.0.1:9002` from a `file://` or `https://` origin may be blocked by browser security. The daemon must set appropriate CORS headers.
- **Tab lifecycle**: When the tab is backgrounded, browsers throttle `setTimeout`/`setInterval`. The `DaemonBridge` reconnection may stall. Service Workers are needed for reliable background operation.
- **WebRTC SDP size limits**: Very large SDP offers (many ICE candidates) may hit browser limits. Trickle ICE (sending candidates individually) mitigates this.

### Missing Features

- [ ] **Service Worker background mode** — no service worker for message receipt when tab is closed.
- [ ] **IndexedDB persistence** — identity and settings are in-memory or localStorage.
- [ ] **STUN/TURN configuration UI** — WebRTC ICE servers config not exposed in UI.
- [ ] **End-to-end encryption for WebRTC** — WebRTC has DTLS at the transport level, but message-level encryption must be applied on top.
- [ ] **Push API** — for server-push notifications when the tab is closed (requires service worker + Push API subscription).
- [ ] **WebRTC stats** — no `RTCPeerConnection.getStats()` monitoring for transport health.

### Recommended Config

```json
{
  "relay_enabled": true,
  "max_relay_budget": 100,
  "battery_floor": 0,
  "ble_enabled": false,
  "wifi_aware_enabled": false,
  "wifi_direct_enabled": false,
  "internet_enabled": true,
  "discovery_mode": "Normal",
  "onion_routing": false,
  "cover_traffic_enabled": false,
  "message_padding_enabled": true,
  "timing_obfuscation_enabled": false,
  "notifications_enabled": true,
  "notify_dm_enabled": true,
  "notify_dm_request_enabled": true,
  "notify_dm_in_foreground": false,
  "notify_dm_request_in_foreground": true,
  "sound_enabled": true,
  "badge_enabled": true
}
```

> All local transports disabled (browser can't access them). Internet-only. Battery floor 0 (always on AC). Message padding on for consistent ciphertext size.

---

## 7. Cross-Platform Interoperability Matrix

| Feature | Linux | Windows | macOS | Android | iOS | WASM |
|---|---|---|---|---|---|---|
| **TCP transport** | ✅ | ✅ | ✅ | ❌ (NsdManager) | ❌ (Multipeer) | ❌ |
| **QUIC transport** | 🟡 (scaffold) | 🟡 | 🟡 | ❌ | ❌ | ❌ |
| **mDNS/Bonjour** | ✅ libp2p | ❌ (needs Bonjour) | ✅ NetService | ❌ (NsdManager) | ✅ NetServiceBrowser | ❌ |
| **BLE** | ✅ BlueZ D-Bus | ❌ | ❌ (needs IOBluetooth) | ✅ Android BLE | ✅ CoreBluetooth | ❌ |
| **WiFi Aware/NAN** | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ |
| **WiFi Direct** | ❌ | ❌ | ❌ | 🟡 (planned) | ❌ (Multipeer) | ❌ |
| **WebRTC** | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Circuit Relay v2** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ (via WS) |
| **Kademlia DHT** | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ (daemon mode) |
| **UPnP** | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **Foreground Service** | systemd | Win Service | LaunchAgent | ✅ | ❌ (BGTask) | ❌ |
| **Push wake** | ❌ | ❌ | ❌ | FCM 🟡 | APNs 🟡 | Push API 🟡 |
| **Keychain/Keystore** | Keystore (file) | DPAPI | Keychain 🟡 | Android Keystore | Keychain 🟡 | IndexedDB 🟡 |
| **Auto-adjust** | ✅ (always Max) | ✅ | ✅ | ✅ | ✅ | ✅ (always Max) |
| **Cover traffic** | ✅ recommended | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Onion routing** | ✅ optional | ✅ optional | ✅ optional | 🟡 (CPU heavy) | 🟡 (CPU heavy) | ❌ |

> ✅ = implemented and working  
> 🟡 = planned/partial/stub  
> ❌ = not available on this platform

---

## Appendix A: AutoAdjustEngine Profile Summary

The `AutoAdjustEngine` (in `core/src/mobile/auto_adjust.rs`) selects profiles based on `DeviceProfile`:

| Profile | BLE Scan (ms) | BLE Advertise (ms) | Relay/Hour | Pri Threshold | When |
|---|---|---|---|---|---|
| **Maximum** | 100 | 20 | 500 | 10 | Screen on + battery >80% + WiFi, or charging + WiFi |
| **High** | 500 | 50 | 300 | 30 | Screen on + battery >60%, or charging, or automotive + WiFi |
| **Standard** | 1280 | 100 | 100 | 50 | Default, screen off + battery >40% |
| **Reduced** | 2560 | 500 | 30 | 70 | Battery 15-40% screen off, or walking + battery <30% |
| **Minimal** | 5120 | 2000 | 5 | 90 | Battery <10%, any condition |

## Appendix B: iOS Background Mode Summary

The `IosBackgroundConfig` (in `core/src/mobile/ios_strategy.rs`) default:

- **Enabled modes**: `BluetoothCentral`, `BluetoothPeripheral`, `BackgroundFetch`
- **Fetch interval**: 900 seconds (15 minutes, iOS minimum)
- **Location accuracy**: `ReducedAccuracy` (disabled by default)
- **Always-location**: `false`

To enable persistent background mesh on iOS, the recommended mode set is:
`{ BluetoothCentral, BluetoothPeripheral, BackgroundFetch, Location }` with `ReducedAccuracy`.

## Appendix C: Transport Selection Priority

The `TransportBridge` (in `cli/src/transport_bridge.rs`) detects CLI capabilities as:
- `Internet` (WebSocket relay + daemon UI bridge) — always
- `Local` (TCP/QUIC/mDNS) — always on native
- `BLE` — on Linux, Windows, macOS (via `cfg(any(linux, windows, macos))`)

On mobile, transport selection is platform-native:
- **Android**: BLE → WiFi Aware → Internet (managed by Kotlin transport managers)
- **iOS**: BLE → Multipeer → Internet (managed by `SmartTransportRouter.swift`)

The transport health tracking in `SmartTransportRouter` biases toward transports with:
- Higher success rate (70% weight)
- Lower latency (30% weight)
- Recent success within 5 seconds
