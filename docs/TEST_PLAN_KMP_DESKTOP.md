# SCMessenger KMP Desktop Test Plan

> **Status**: Draft
> **Last updated**: 2026-06-03
> **Target**: SCMessenger Compose Multiplatform (KMP) desktop client on Linux (linuxX64) and JVM

---

## 1. Test Infrastructure Overview

### 1.1 Testing Pyramid for KMP Desktop

```
┌─────────────────────────────────────────────────────────────────┐
│                    KMP DESKTOP TESTING PYRAMID                  │
├─────────────────────────────────────────────────────────────────┤
│  Level 4: Cross-Platform Mesh Interop (Manual / Multi-device)  │
│    └── desktop ↔ Android (libp2p mesh)                         │
│    └── desktop ↔ WASM   (WebSocket relay)                       │
│    └── desktop ↔ iOS     (relay custody)                        │
├─────────────────────────────────────────────────────────────────┤
│  Level 3: UI Parity Tests (compose-ui-test)                     │
│    └── Compose Multiplatform UI tests on linuxX64/JVM          │
│    └── Desktop composable rendering parity vs Android           │
├─────────────────────────────────────────────────────────────────┤
│  Level 2: Integration Tests                                      │
│    └── Rust: cargo test -p scmessenger-desktop-bridge          │
│    └── Kotlin: ./gradlew :shared:testLinuxX64                  │
├─────────────────────────────────────────────────────────────────┤
│  Level 1: Unit Tests                                             │
│    └── Rust: cargo test -p scmessenger-core (existing)         │
│    └── Kotlin: commonTest source set (kotlin-test)             │
│    └── Android: ./gradlew :app:testDebugUnitTest (existing)    │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Test Source Sets

| Source Set | Purpose | Framework | Run Command |
|---|---|---|---|
| `shared/src/commonTest` | Shared KMP unit tests | `kotlin-test` | `./gradlew :shared:allTests` |
| `shared/src/linuxX64Test` | Linux-native desktop tests | `kotlin-test` | `./gradlew :shared:testLinuxX64` |
| `shared/src/jvmTest` | JVM desktop tests | `kotlin-test` | `./gradlew :shared:jvmTest` |
| `desktop_bridge/tests/` | Rust integration tests | `#[test]` | `cargo test -p scmessenger-desktop-bridge` |
| `desktop_bridge/src/` (inline) | Rust unit tests | `#[test]` | `cargo test -p scmessenger-desktop-bridge` |
| `app/src/test/` | Android unit tests (Existing) | JUnit | `./gradlew :app:testDebugUnitTest` |
| `core/tests/` | Rust core integration tests (Existing) | `#[test]` | `cargo test -p scmessenger-core` |

### 1.3 Build Configuration Requirements

`shared/build.gradle.kts` must declare:

```kotlin
kotlin {
    sourceSets {
        val commonTest by getting {
            dependencies {
                implementation(kotlin("test"))
            }
        }
        val linuxX64Test by getting {
            dependencies {
                dependsOn(commonTest)
                implementation(kotlin("test"))
            }
        }
        val jvmTest by getting {
            dependencies {
                dependsOn(commonTest)
                implementation(kotlin("test"))
            }
        }
    }
}
```

### 1.4 CI Test Gate (ubuntu-latest)

```bash
# Gate 1: Core Rust tests
cargo test -p scmessenger-core --workspace

# Gate 2: Desktop bridge tests
cargo test -p scmessenger-desktop-bridge

# Gate 3: KMP shared LinuxX64 tests
./gradlew :shared:testLinuxX64

# Gate 4: Android parity (existing)
./gradlew :app:testDebugUnitTest
```

All four gates must pass before merge.

### 1.5 Mocking Strategy

| External Dependency | CI Mock Approach |
|---|---|
| BlueZ D-Bus (BLE) | Mock with `zbus` test double or `@Ignore` on CI; `#[cfg(not(test))]` gate |
| XDG directories | Override `XDG_DATA_HOME` / `XDG_CONFIG_HOME` env vars in tests |
| System notifications | Stub `NotificationProxy` returning in-memory log |
| libp2p mesh | Replace with loopback transport or in-process swarm relay |
| Sled DB (store) | Use temporary directory `tempfile::TempDir` in tests |

---

## 2. UI Parity Matrix

### 2.1 Screen Mapping: Android → Desktop

| # | Android Screen | Desktop Equivalent | Parity Level | Desktop Adaptation Notes |
|---|---|---|---|---|
| 1 | `OnboardingScreen` | `OnboardingScreen` | **Full parity** | Same composable; responsive layout adjusts to window size |
| 2 | `DashboardScreen` | `DashboardScreen` | **Full parity** | Same content; sidebar + detail pane layout on wide windows |
| 3 | `ChatScreen` | `ChatScreen` | **Full parity** | Message list, input field, delivery state identical |
| 4 | `ConversationsScreen` | `ConversationsScreen` | **Full parity** | Thread list same; desktop gets keyboard shortcuts |
| 5 | `ContactsScreen` | `ContactsScreen` | **Full parity** | Contact manager identical |
| 6 | `AddContactScreen` | `AddContactDialog` | **Adapted** | Modal dialog instead of full screen on desktop |
| 7 | `ContactDetailScreen` | `ContactDetailPane` | **Adapted** | Side pane or master-detail split on desktop |
| 8 | `SettingsScreen` | `SettingsScreen` | **Full parity** | Same sections; desktop uses wider layout |
| 9 | `MeshSettingsScreen` | `MeshSettingsScreen` | **Full parity** | Same mesh configuration options |
| 10 | `PowerSettingsScreen` | `PowerSettingsScreen` | **Adapted** | No battery section on desktop; CPU/network throttling only |
| 11 | `IdentityScreen` | `IdentityScreen` | **Full parity** | Key management identical |
| 12 | `JoinMeshScreen` | `JoinMeshDialog` | **Adapted** | Dialog with QR scan or manual code entry |
| 13 | `BlockedPeersScreen` | `BlockedPeersScreen` | **Full parity** | Same block/unblock functionality |
| 14 | `RequestsInboxScreen` | `RequestsInboxScreen` | **Full parity** | Same request approval flow |
| 15 | `DiagnosticsScreen` | `DiagnosticsScreen` | **Full parity** | Same diagnostic output; desktop adds copy-to-clipboard |
| 16 | `PeerListScreen` | `PeerListPane` | **Adapted** | Embedded in dashboard sidebar on desktop |
| 17 | `TopologyScreen` | `TopologyScreen` | **Full parity** | Same topology visualization; desktop gets larger canvas |
| 18 | `NetworkStatusDialog` | `NetworkStatusDialog` | **Full parity** | Same network info display |
| 19 | `MessageBubble` | `MessageBubble` | **Full parity** | Identical rendering |
| 20 | `MessageInput` | `MessageInput` | **Full parity** | Same input; desktop adds Enter-to-send, Shift+Enter newline |
| 21 | `DeliveryStateSurface` | `DeliveryStateSurface` | **Full parity** | Same delivery indicators |
| 22 | `ErrorBanner` | `ErrorBanner` | **Full parity** | Same error display |
| 23 | `StatusIndicator` | `StatusIndicator` | **Full parity** | Same connection status dots |
| 24 | `Identicon` | `Identicon` | **Full parity** | Same identicon generation |
| 25 | `CopyableText` | `CopyableText` | **Full parity** | Same copy-to-clipboard |
| 26 | `StorageWarningBanner` | `StorageWarningBanner` | **Full parity** | Same storage warning |
| 27 | `EntropyCanvas` | `EntropyCanvas` | **Full parity** | Same entropy visualization |

### 2.2 Mobile-Only Screens (No Desktop Equivalent)

| Android Screen | Reason | Desktop Handling |
|---|---|---|
| Camera composables | No camera assumption on desktop | N/A — file picker instead |
| GPS / Location features | No GPS on desktop | N/A — manual location entry if needed |
| Foreground service notification | Android-specific lifecycle | Replaced by background process / system tray |
| Bluetooth permissions dialog | Android BLE permission model | Replaced by BlueZ D-Bus integration |
| Push notification channels | Android notification channels | Replaced by XDG desktop notifications |

### 2.3 Desktop-Only Adaptations

| Feature | Android | Desktop |
|---|---|---|
| Background operation | Foreground service + notification | System tray icon + background process |
| Notifications | Android notification manager | XDG `org.freedesktop.Notifications` via `notify-rust` |
| BLE discovery | Android `BluetoothAdapter` | BlueZ D-Bus via `zbus` |
| Storage paths | `Context.getFilesDir()` | XDG Base Directory spec (`$XDG_DATA_HOME/scmessenger`) |
| App lifecycle | Activity lifecycle | Window close → minimize to tray; explicit quit |
| Permissions | Runtime permission dialogs | Polkit / D-Bus authorization |
| Auto-update | Play Store / manual APK | Built-in update checker or package manager |

---

## 3. Cross-Platform Mesh Interoperability Test Scenarios

### 3.1 Desktop ↔ Android (libp2p Mesh)

**Test Environment**: Ubuntu desktop (linuxX64 KMP client) + Android phone on same LAN

| Test ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| DA-001 | Peer discovery (LAN) | 1. Launch desktop client<br>2. Launch Android app<br>3. Wait for mDNS discovery | Both peers appear in peer list within 10s | P0 |
| DA-002 | Direct message send | 1. Select Android peer from desktop<br>2. Send text message | Message delivered; delivery state shows confirmed | P0 |
| DA-003 | Direct message receive | 1. Send message from Android to desktop | Desktop shows notification; message appears in chat | P0 |
| DA-004 | File transfer (desktop → Android) | 1. Attach file on desktop<br>2. Send to Android peer | File received on Android; checksum matches | P1 |
| DA-005 | File transfer (Android → desktop) | 1. Send file from Android to desktop | File saved to XDG data dir; checksum matches | P1 |
| DA-006 | Offline queue (desktop offline) | 1. Disconnect desktop from network<br>2. Send message from Android<br>3. Reconnect desktop | Message received after reconnection; order preserved | P1 |
| DA-007 | Offline queue (Android offline) | 1. Disconnect Android<br>2. Send message from desktop<br>3. Reconnect Android | Message delivered after Android comes online | P1 |
| DA-008 | Relay custody (3-node) | 1. Desktop + Android + third relay node<br>2. Disconnect direct path<br>3. Send message | Message routed through relay; custody receipt confirmed | P2 |
| DA-009 | BLE discovery fallback | 1. Disable WiFi on both<br>2. Enable BLE<br>3. Initiate discovery | Peers discovered via BLE; connection established | P2 |
| DA-010 | Identity verification | 1. Exchange identities across platforms<br>2. Verify fingerprint | Fingerprint matches on both platforms | P0 |

### 3.2 Desktop ↔ WASM (WebSocket Relay)

**Test Environment**: Ubuntu desktop (KMP client) + Browser (WASM client) via WebSocket relay

| Test ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| DW-001 | Relay connection | 1. Desktop connects to relay<br>2. WASM client connects to same relay | Both clients show connected status | P0 |
| DW-002 | Message via relay | 1. Desktop sends message via relay<br>2. WASM client receives | Message content identical; timestamp within tolerance | P0 |
| DW-003 | Message via relay (reverse) | 1. WASM sends message<br>2. Desktop receives | Message content identical | P0 |
| DW-004 | Relay custody proof | 1. Send message with custody request<br>2. Verify relay receipt | Custody receipt generated and verified | P1 |
| DW-005 | Relay failover | 1. Primary relay goes down<br>2. Clients reconnect to backup relay | Messages resume after reconnection; no data loss | P2 |
| DW-006 | Large message via relay | 1. Send message > 64KB via relay | Message chunked and reassembled correctly | P2 |
| DW-007 | Concurrent relay sessions | 1. Multiple WASM clients connect<br>2. Desktop sends broadcast | All WASM clients receive broadcast | P2 |

### 3.3 Desktop ↔ iOS (Relay Custody)

**Test Environment**: Ubuntu desktop (KMP client) + iOS device via relay (no direct P2P)

| Test ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| DI-001 | Relay-mediated connection | 1. Desktop connects to relay<br>2. iOS connects to relay<br>3. Initiate session | Session established via relay | P0 |
| DI-002 | Message send (desktop → iOS) | 1. Desktop sends message to iOS peer | Message delivered; iOS shows notification | P0 |
| DI-003 | Message receive (iOS → desktop) | 1. iOS sends message to desktop | Desktop receives; message content intact | P0 |
| DI-004 | Custody chain verification | 1. Send message with full custody chain<br>2. Verify at each hop | Custody chain intact end-to-end | P1 |
| DI-005 | iOS background handling | 1. Send message to iOS while app backgrounded<br>2. iOS comes to foreground | Message queued and delivered; no loss | P1 |
| DI-006 | Desktop background handling | 1. Minimize desktop to tray<br>2. Send message from iOS<br>3. Restore desktop | Message received while minimized; tray notification shown | P1 |
| DI-007 | Identity cross-verification | 1. Compare identity fingerprints across platforms | Fingerprints match; no MITM | P0 |

### 3.4 Common Interoperability Test Scenarios

| Test ID | Scenario | Platforms | Expected Result |
|---|---|---|---|
| CI-001 | Message ordering | All pairs | Messages delivered in send order per conversation |
| CI-002 | Duplicate suppression | All pairs | Duplicate messages (same msg-id) deduplicated |
| CI-003 | Unicode content | All pairs | Emoji, CJK, RTL text rendered correctly on all platforms |
| CI-004 | Clock skew tolerance | All pairs | Messages accepted with clock skew ≤ 5 minutes |
| CI-005 | Network partition recovery | All pairs | Messages sync after partition heals; no data loss |
| CI-006 | Key rotation | All pairs | New keys propagated; old messages still decryptable |

---

## 4. Rust Integration Tests (desktop_bridge)

### 4.1 XDG Path Tests

| Test | File | Description |
|---|---|---|
| `test_xdg_data_dir` | `desktop_bridge/tests/xdg_paths_test.rs` | Verify `xdg_data_dir()` returns absolute path |
| `test_xdg_data_dir_contains_scmessenger` | `desktop_bridge/tests/xdg_paths_test.rs` | Verify path ends with `scmessenger` |
| `test_xdg_config_dir` | `desktop_bridge/tests/xdg_paths_test.rs` | Verify `xdg_config_dir()` returns absolute path |
| `test_xdg_env_override` | `desktop_bridge/tests/xdg_paths_test.rs` | Override `XDG_DATA_HOME` env var; verify respected |
| `test_ensure_directories` | `desktop_bridge/tests/xdg_paths_test.rs` | Verify `ensure_directories` creates all required dirs |

### 4.2 Desktop Bridge FFI Tests

| Test | File | Description |
|---|---|---|
| `test_desktop_version` | `desktop_bridge/src/lib.rs` (inline) | Verify version string non-empty |
| `test_xdg_data_dir_returns_path` | `desktop_bridge/src/lib.rs` (inline) | Verify path returned |
| `test_xdg_config_dir_returns_path` | `desktop_bridge/src/lib.rs` (inline) | Verify config path returned |

### 4.3 Notification Tests (Mocked)

| Test | File | Description |
|---|---|---|
| `test_notification_send` | `desktop_bridge/tests/notification_test.rs` | Mock notification proxy; verify send succeeds |
| `test_notification_with_actions` | `desktop_bridge/tests/notification_test.rs` | Verify action buttons serialized correctly |

---

## 5. Kotlin KMP Tests

### 5.1 commonTest

| Test | File | Description |
|---|---|---|
| `testPlatformName` | `shared/src/commonTest/.../PlatformTest.kt` | Verify `platformName()` returns non-empty string |
| `testGreet` | `shared/src/commonTest/.../PlatformTest.kt` | Verify `greet()` includes platform name |

### 5.2 linuxX64Test

| Test | File | Description |
|---|---|---|
| `testLinuxPlatform` | `shared/src/linuxX64Test/.../LinuxPlatformTest.kt` | Verify `platformName()` returns `"Linux"` |
| `testLinuxStorePath` | `shared/src/linuxX64Test/.../LinuxPlatformTest.kt` | Verify store path under XDG data dir |

---

## 6. UI Parity Test Procedures

### 6.1 Automated UI Tests (compose-ui-test)

| Test | Description |
|---|---|
| `testOnboardingRenders` | Verify onboarding screen composables render without crash |
| `testChatScreenMessageList` | Verify message list displays messages |
| `testChatScreenInputField` | Verify input field accepts text and sends on Enter |
| `testContactsScreenList` | Verify contacts list renders contact items |
| `testSettingsScreenSections` | Verify all settings sections present |
| `testNavigationFlow` | Verify navigation between screens works |
| `testSystemTrayIntegration` | Verify tray icon appears and responds to clicks |

### 6.2 Manual UI Parity Checklist

For each screen in the UI Parity Matrix (Section 2.1):

- [ ] All text content matches Android
- [ ] All interactive elements present (buttons, inputs, toggles)
- [ ] Layout adapts correctly to window resize
- [ ] Keyboard navigation works (Tab, Enter, Escape)
- [ ] Right-click context menus present where applicable
- [ ] Drag-and-drop supported for file attachments
- [ ] Copy/paste works (Ctrl+C / Ctrl+V)
- [ ] Scroll behavior matches (scrollbar vs. touch scroll)

---

## 7. Test Execution Schedule

| Phase | Tests | Trigger |
|---|---|---|
| Pre-commit | `cargo test --workspace`, `./gradlew :shared:testLinuxX64` | Every commit |
| PR gate | All Level 1 + Level 2 tests | Every PR |
| Nightly | Level 3 UI tests + Level 4 interop (manual) | Nightly cron |
| Release | Full matrix: all levels + all interop scenarios | Release candidate |

---

## 8. Known Limitations

1. **BLE on CI**: BlueZ D-Bus tests require `@Ignore` or `#[cfg(not(test))]` on CI runners without Bluetooth hardware
2. **iOS interop**: Requires physical iOS device; cannot be automated on CI
3. **WASM relay**: Requires running relay server; documented as manual test procedure
4. **compose-ui-test on linuxX64**: Requires X11 or Wayland display; use `xvfb-run` on headless CI
5. **File transfer large files**: Limited to < 100MB in CI due to resource constraints
