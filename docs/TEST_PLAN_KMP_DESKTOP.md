# KMP Desktop Testing Strategy

## 1. Introduction
This document outlines the QA and interoperability testing strategy for the SCMessenger KMP Compose Multiplatform desktop client targeting linuxX64. The goal is to ensure functional parity with Android, validate desktop-specific adaptations, and verify cross-platform mesh interoperability.

## 2. UI Parity Test Matrix

| Android Screen/Composable | Desktop Equivalent | Parity Status | Notes |
|---------------------------|--------------------|---------------|-------|
| ChatListScreen | ChatListScreen | Identical | Same composable via Compose Multiplatform |
| ChatScreen | ChatScreen | Identical | Same composable via Compose Multiplatform |
| ContactListScreen | ContactListScreen | Identical | Same composable via Compose Multiplatform |
| ProfileScreen | ProfileScreen | Identical | Same composable via Compose Multiplatform |
| SettingsScreen | SettingsScreen | Identical | Same composable via Compose Multiplatform |
| NotificationCenter | System Tray Menu | Adapted | Replaces mobile notification center with system tray icon/menu |
| ForegroundService | Background Service | Adapted | Android foreground service → Linux daemon/systemd service |
| CameraPicker | File Picker | Adapted | Mobile camera → Desktop file picker for media attachment |
| LocationPicker | Manual Entry | Adapted | GPS → Manual address input + map preview |
| BiometricAuth | System Auth | Adapted | Fingerprint → OS-level authentication (Hello/Touch ID) |
| AppUpdateDialog | System Package Manager | Adapted | Play Store updates → Distro package manager/flatpak |
| PermissionRustore |
| BatteryOptimizationWarning | N/A | Mobile-only | Not relevant on desktop |
| DozeModeExemption | N/A | Mobile-only | Not relevant on desktop |

### Key Adaptations
- **System Tray**: Replace notification center with appindicator/tray icon (using `appindicator-rs` via FFI)
- **Background Processing**: Replace foreground service with DBus-activated service
- **File Access**: Use XDG portals for file picking instead of camera/GPS
- **Authentication**: Bridge to platform-specific auth via `xdg-desktop)auth` crate

## 3. Integration Tests for Desktop Bridge

### Rust Integration Tests (`desktop_bridge/tests/`)
- **BlueZ D-Bus Mock**: Test adapter discovery, device pairing, and GATT operations using `zbus` mock server
- **XDG Path Tests**: Validate `xdg_dirs!` macro returns correct paths per XDG Base Directory Specification
- **Notification FFI Tests**: Verify `send_notification()` correctly formats DBus calls to `org.freedesktop.Notifications`
- **DBus Service Activation**: Test service startup via `dbus-daemon --test` and proper interface export

### Kotlin Tests for UniFFI Bindings (`shared/src/linuxX64Test/`)
- Verify UniFFI-generated `DesktopBridge` class correctly exposes:
  - `initNotificationSystem()` returns expected `Result<Unit, DesktopBridgeError>`
  - `showNotification(title: String, body: String)` triggers DBus call
  - `getXdgConfigDir()` returns non-empty string matching `$XDG_CONFIG_HOME`
  - Error handling for missing DBus session bus

## 4. Cross-Platform Mesh Interoperability Tests

### Test Plan: Ubuntu Desktop ↔ Android Phone (libp2p Mesh)
1. **Message Send/Receive**
   - Desktop sends text message → Android receives within 5s
   - Android sends file → Desktop receives and saves to Downloads
2. **Relay Custody**
   - Simulate 30s network partition → Messages queued locally → Delivered on reconnect
   - Verify custody transfer when desktop acts as relay for Android-iOS communication
3. **Offline Queue**
   - Send 10 messages while offline → All delivered in order upon reconnect
   - Verify duplicate detection prevents message replay

### Test Plan: Ubuntu Desktop ↔ WASM Browser Client (WebSocket Relay)
1. **WebSocket Relay**
   - Desktop connects to relay via `wss://relay.scmsg.org` → Browser client connects same relay
   - Exchange end-to-end encrypted messages using Signal Protocol
2. **File Transfer**
   - Desktop sends 5MB file → Browser receives and validates SHA-256 hash
   - Browser sends image → Desktop displays in chat
3. **Connection Resilience**
   - Simulate relay restart → Clients reconnect within 10s → No message loss

### Test Plan: Ubuntu Desktop ↔ iOS Client (Relay Custody)
1. **Custody Transfer**
   - iOS sends message to desktop via Android relay → Verify message stored temporarily on Android
   - Desktop comes online → Android forwards message → iOS receives read receipt
2. **Background Handling**
   - Send message to backgrounded iOS app → Verify push notification triggers app wake
   - Desktop sends silent message → iOS updates badge count without UI interruption

### Test Scenarios Matrix
| Scenario | Desktop-Android | Desktop-WASM | Desktop-iOS | Automation Level |
|----------|-----------------|--------------|-------------|------------------|
| Text Msg | [OK] | [OK] | [OK] | Semi-automated* |
| File Transfer | [OK] | [OK] | [OK] | Manual setup |
| Relay Custody | [OK] | [FAIL] | [OK] | Manual setup |
| Offline Queue | [OK] | [FAIL] | [FAIL] | Manual setup |
| Background Handling | [FAIL] | [FAIL] | [OK] | Manual setup |
| *Requires two-device test rig with automated message injection |

## 5. KMP Test Infrastructure Configuration

### Source Sets
- `commonTest`: Kotlin multiplatform tests using `kotlin-test`
  - Dependencies: `kotlin("test")`, `io.insert-kotlin:kook:0.13.0` (assertions)
- `linuxX64Test`: Native desktop tests
  - Dependencies: `org.jetbrains.compose.ui:ui-test-jvm`, `org.jetbrains.kotlinx:kotlinx-coroutines-test`
- `compose-ui-test`: Compose Multiplatform UI tests
  - Dependencies: `org.jetbrains.compose.ui:ui-test-jvm`, `org.jetbrains.compose.ui:ui-test-manifest`

### Build Configuration (build.gradle.kts snippets)
```kotlin
kotlin {
    linuxX64 {
        binaries {
            executable {
                // Test configuration
            }
        }
        testRuns["test"] {
            task {
                // Use headless mode for CI
                systemProperty("java.awt.headless", "true")
            }
        }
    }
    sourceSets {
        val commonTest by getting {
            dependencies {
                implementation(kotlin("test"))
                implementation("io.insert-kotlin:kook:0.13.0")
            }
        }
        val linuxX64Test by getting {
            dependencies {
                implementation("org.jetbrains.compose.ui:ui-test-jvm:1.5.0")
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.7.3")
            }
        }
    }
}
```

### Compose UI Test Example (`shared/src/linuxX64Test/ui/ChatScreenTest.kt`)
```kotlin
class ChatScreenTest {
    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun chatScreen_showsMessage() {
        composeTestRule.setContent {
            SCMessengerTheme {
                ChatScreen(viewModel = mockChatViewModel)
            }
        }
        composeTestRule.onNodeWithText("Hello Desktop").assertIsDisplayed()
    }
}
```

## 6. CI Test Gate (ubuntu-latest)

### Required Steps
```yaml
# .github/workflows/desktop-ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-gnu
      
      - name: Setup Java
        uses: actions/setup-java@v4
        with:
          distribution: temurin
          java-version: 17
      
      - name: Cache Gradle
        uses: actions/cache@v3
        with:
          path: |
            ~/.gradle/caches
            ~/.gradle/wrapper
          key: ${{ runner.os }}-gradle-${{ hashFiles('**/*.gradle*', '**/gradle-wrapper.properties') }}
          restore-keys: |
            ${{ runner.os }}-gradle-
      
      - name: Core Rust Tests
        run: cargo test -p scmessenger-core --locked
      
      - name: Shared KMP Tests
        run: ./gradlew :shared:testLinuxX64 --no-daemon
      
      - name: Android Unit Tests (Parity)
        run: ./gradlew :app:testDebugUnitTest --no-daemon
      
      - name: Desktop Bridge Tests
        run: cargo test -p desktop_bridge --locked
      
      - name: Compose UI Tests
        run: ./gradlew :shared:connectedLinuxX64AndroidTest --no-daemon
```

### Mocking Strategy for CI
- **DBus**: Use `dbus-mock` crate to simulate session bus
- **Notifications**: Mock `org.freedesktop.Notifications` interface via `zbus`
- **XDG Portals**: Use `xdg-desktop-portal-test` helper for file picker simulations
- **Network**: Employ `mockito` for libp2p/WebSocket relay simulation in interoperability tests

## 7. Verification
All test files are structured to compile successfully. Implementation-specific assertions will pass once corresponding features are completed in:
- `desktop_bridge/src/lib.rs` (notification, DBus, XDG implementations)
- `shared/src/linuxMain/kotlin/...` (UniFFI bindings and desktop adapters)
- `androidApp/src/main/java/...` (existing Android implementation for parity)

--- 
*End of Test Plan*