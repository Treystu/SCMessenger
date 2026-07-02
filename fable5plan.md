# SCMessenger v1.0.0 — Master Engineering Backlog (Gaps-Only)

## Context

**Goal:** production-ready v1.0.0 autonomous survival mesh: direct P2P over BLE/Wi-Fi proximity transports, mycorrhizal routing, and delay-tolerant data-muling on commodity iOS/Android with zero ISP/grid dependence.

**Why this backlog looks different from the original request:** a three-agent deep audit (Rust core, FFI/mobile, CI/hygiene) established that most of the "build" tracks are **already implemented and tested**:

- **Routing** — full 3-layer mycorrhizal engine (`core/src/routing/`: `engine.rs`, `local.rs`, `neighborhood.rs`, `global.rs`) with adaptive TTL, smart retry, negative cache, reputation/multipath (phase2 features). Integration-tested (`core/tests/integration_mycorrhizal_routing.rs`).
- **Drift/DTN** — **already wired live**, not dormant: `iron_core.rs:132-133` owns `drift_store`/`drift_engine`; `transport/swarm.rs:2046-2047, 3331+` activates a `SyncSession` per peer on connect and ships `DriftFrame`s. IBLT sketch, CRDT MeshStore, LZ4, policy engine all real (`core/src/drift/`).
- **Crypto** — Ed25519/X25519/XChaCha20-Poly1305 + full Double Ratchet (`core/src/crypto/ratchet.rs`), Kani proofs, proptests. Zero `todo!()`/`unimplemented!()` in the core (one `// TODO` at `iron_core.rs:3316`).
- **Native transports** — real Swift (`ios/SCMessenger/SCMessenger/Transport/`: BLECentral/Peripheral/L2CAP, Multipeer, mDNS, SmartTransportRouter) and Kotlin (`android/app/.../transport/`: BLE, WifiAware, WifiDirect, mDNS, SmartTransportRouter) implementations with correct manifests/entitlements.
- **PlatformBridge BLE path is wired**: `mobile_bridge.rs:1126` (`on_ble_data_received` → core ingest), `mobile_bridge.rs:1249` (`send_ble_packet` → native egress).

**Per user decision:** backlog covers **only the genuine gaps**; **acoustic transport is deferred post-v1.0.0** (zero code exists; it's a multi-month DSP effort).

**The verified genuine gaps:**

| # | Gap | Evidence |
|---|-----|----------|
| G1 | `WifiAwareTransport` is **orphaned** — complete + tested but referenced nowhere outside `transport/wifi_aware.rs`; only impl of `WifiAwarePlatformBridge` is `#[cfg(test)] MockWifiAwareBridge` | grep: zero external references |
| G2 | Wi-Fi Direct has **no Rust-side transport** — only `TransportType::WiFiDirect` enum + Kotlin `WifiDirectTransport.kt` with no FFI path | `transport/abstraction.rs:11-22` |
| G3 | `PlatformBridge` FFI carries **only BLE** (`send_ble_packet`/`on_ble_data_received`) — Wi-Fi Aware/Direct native code can't deliver bytes into the core | `mobile_bridge.rs:1436-1443`, `api.udl` callback interface |
| G4 | **Duplicate `PlatformBridge` traits** — `mobile_bridge.rs:1436` (UniFFI, live) vs `mobile/service.rs:87` (legacy, mock-only) | grep |
| G5 | `SwarmHandle` async-command path incomplete — `iron_core.rs:3314-3318` `update_keepalive` is a stub; `SwarmCommand` enum exists (`swarm.rs:1238`) but lacks the variant | code read |
| G6 | 7 `#[ignore]`d NAT tests requiring live `SwarmHandle` (`transport/nat.rs:634-699`) | code read |
| G7 | **No CI/CD at all** — no workflows, no rustfmt/clippy/deny config; `core/.cargo/config.toml` hardcodes a Windows NDK path | audit |
| G8 | **~1 GB committed build artifacts** (`core/target/android-libs/*.so` ~1 GB, `staged-cdylib/*.dylib` 57 MB, `app-debug.apk` 47 MB, `android/.gradle/`, `repomix-output.xml` 5.6 MB) | audit |
| G9 | Phantom workspace member `mobile` in root `Cargo.toml` (directory doesn't exist) | audit |
| G10 | No top-level README/CHANGELOG/LICENSE file; no on-device multi-node verification harness | audit |

---

## Master Backlog

Ordering is strict technical dependency: **Track 5 (CI/hygiene) executes FIRST** — a swarm of agents cannot safely verify work without automated gates. Then Track 1 (FFI seam), then 2–4 which consume it.

Task IDs: `T<track>.<seq>`. Each task is atomic, agent-executable, and independently verifiable.

---

### TRACK 5 — CI/CD, FFI Stability & Repo Hygiene  ⟵ EXECUTE FIRST

#### T5.1 Purge committed build artifacts & fix .gitignore enforcement
- **Technical context:** `core/target/` (android-libs ~1 GB, staged-cdylib 57 MB), `android/app/build/outputs/apk/debug/app-debug.apk`, `android/.gradle/`, `repomix-output.xml`. `.gitignore` already lists these patterns but the files were committed before it applied.
- **Implementation:** `git rm -r --cached` each artifact path; add `repomix-output.xml` + `core/target/` + `android/.gradle/` explicitly to `.gitignore`. Do NOT rewrite history in this task (single-commit repo; a follow-up `git gc` after the removal commit suffices to shrink clones going forward).
- **Edge cases:** the Android Gradle build references `core/target/generated-sources/uniffi/kotlin` and the prebuilt `.so`s — confirm `android/app/build.gradle` regenerates these (it has a task that triggers `gen_kotlin` when missing) before deleting, otherwise document the local-build prerequisite in the README task (T5.8).
- **Verification:** `git ls-files | grep -E '\.(so|dylib|apk)$'` returns empty; `du -sh .git` reported before/after; `cargo build -p scmessenger-core` still succeeds.

#### T5.2 Remove phantom `mobile` workspace member & fix portability of cargo config
- **Technical context:** root `Cargo.toml` `members = ["core", "mobile", "cli", "desktop_bridge", "wasm"]` — `mobile/` does not exist. `core/.cargo/config.toml` hardcodes `/c/Users/kanal/...NDK...clang.cmd` (a Windows path) as the x86_64-linux-android linker.
- **Implementation:** drop `mobile` from members (verify `desktop_bridge` exists; drop it too if phantom). Replace hardcoded NDK linker with env-var-driven config (`[env]` + documented `ANDROID_NDK_HOME`) or move linker selection into a `cargo-ndk` invocation documented in scripts (T5.4).
- **Edge cases:** `cargo metadata` must succeed on macOS/Linux/Windows; do not break the `gen-bindings` feature builds.
- **Verification:** `cargo metadata --format-version 1 > /dev/null` exits 0; `grep -r "kanal" core/.cargo/` empty; `cargo check --workspace` passes.

#### T5.3 Add rustfmt + clippy + cargo-deny baseline
- **Technical context:** no `rustfmt.toml`, `clippy.toml`, or `deny.toml` exist. 53k LOC core.
- **Implementation:** add `rustfmt.toml` (default style, `edition = "2021"`), workspace-level `[workspace.lints.clippy]` (warn-level: `all`; allow existing violations via one `cargo clippy --fix` pass or targeted `allow`s — do not hand-edit 126 files), `deny.toml` (advisories + licenses: MIT/Apache-2.0/BSD allowlist; libp2p tree is large — run `cargo deny check` first and codify the actual license set found).
- **Edge cases:** `cargo fmt` on generated/UDL-adjacent code; exclude `core/target/generated-sources`. Kani proof modules and proptest harnesses may trip clippy — scope lints per-module if needed.
- **Verification:** `cargo fmt --check`, `cargo clippy --workspace --all-features -- -D warnings`, `cargo deny check` all exit 0.

#### T5.4 CI workflow: core build + test matrix
- **Technical context:** create `.github/workflows/ci.yml`. Tests: 21 integration files in `core/tests/`, 1,145+ unit tests, proptests. `gen-bindings` feature gates `gen_kotlin.rs`/`gen_swift.rs`.
- **Implementation:** jobs — (1) `fmt`+`clippy`+`deny` (after T5.3); (2) `cargo test --workspace` on ubuntu + macos; (3) `cargo test -p scmessenger-core --features phase2_apis`; (4) doc build. Cache with `Swatinem/rust-cache`. Pin toolchain via `rust-toolchain.toml` (new file, stable channel).
- **Edge cases:** mDNS/network-touching integration tests may fail in CI containers — the core already "gracefully disables mDNS in containers" per `behaviour.rs`; mark genuinely network-dependent tests `#[ignore]` with a dedicated `-- --ignored` nightly job rather than letting them flake. proptest/Kani: run Kani in a separate optional job (it needs `cargo kani` install, slow).
- **Verification:** workflow green on a test PR; total wall time < 20 min; failure of any single test fails the pipeline.

#### T5.5 CI workflow: cross-compilation matrix (Android/iOS/WASM)
- **Technical context:** targets already configured in workspace: aarch64/armv7/x86_64/i686-linux-android, aarch64-apple-ios(-sim), wasm32-unknown-unknown. Build scripts: `core/build.rs` (UniFFI scaffolding), `core/src/bin/gen_kotlin.rs`, `gen_swift.rs`.
- **Implementation:** `.github/workflows/cross.yml`: (1) `cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 build -p scmessenger-core --release` on ubuntu with NDK r26+; (2) `cargo build --target aarch64-apple-ios --target aarch64-apple-ios-sim -p scmessenger-core --release` on macos; (3) `cargo build --target wasm32-unknown-unknown -p scmessenger-wasm`; (4) run binding generators (`cargo run --bin gen_kotlin --features gen-bindings`, same for swift) and upload generated bindings + cdylibs as artifacts.
- **Edge cases:** `gen_swift.rs` patches `nonisolated(unsafe)` for Swift 6 — assert the patch applied (grep output file). The cdylib search order honors `SCMESSENGER_CDYLIB_PATH` — set it explicitly in CI to avoid the hardcoded relative fallbacks. QUIC (quinn) needs no extra system deps; sled needs none.
- **Verification:** all targets compile; artifacts contain `libscmessenger_mobile.so` for 3 ABIs, `.dylib`/iOS staticlib, `api.kt`, `SCMessengerCore.swift`; generated Kotlin contains `@file:android.annotation.SuppressLint("NewApi")` header (post-processing ran).

#### T5.6 CI workflow: mobile app assembly
- **Technical context:** `android/` Gradle project (Compose, Hilt, AGP 8.13 wrapper present), `ios/SCMessenger/SCMessenger.xcodeproj` (iOS 17.0 min, xcframework at `ios/SCMessengerCore.xcframework`).
- **Implementation:** (1) Android: `./gradlew :app:assembleDebug` consuming T5.5 artifacts (wire `SCMESSENGER_CDYLIB_PATH`/jniLibs from artifact dir); (2) iOS: rebuild `SCMessengerCore.xcframework` from the iOS staticlibs + generated Swift, then `xcodebuild -project ... -scheme SCMessenger -destination 'generic/platform=iOS Simulator' build` (no signing).
- **Edge cases:** xcframework script doesn't exist yet — create `scripts/build_xcframework.sh` (lipo sim slices, `xcodebuild -create-xcframework`); Gradle JDK 17; do NOT commit the rebuilt xcframework (it's an artifact — extend T5.1 ignore rules).
- **Verification:** debug APK artifact produced; xcodebuild exits 0; both jobs consume freshly built (not committed) native libs.

#### T5.7 UniFFI surface contract test (FFI stability gate)
- **Technical context:** `core/src/api.udl` + proc-macro exports in `mobile_bridge.rs`/`contacts_bridge.rs`/`blocked_bridge.rs`. uniffi 0.31. Breaking the surface silently breaks both apps.
- **Implementation:** snapshot test: check in a canonical copy of the generated `api.kt` and `SCMessengerCore.swift` public-symbol list (not full file — extract `fun |class |interface |enum ` signatures via a small script `scripts/ffi_surface.sh`); CI job diffs freshly generated surface against snapshot and fails on unapproved change. Update procedure documented in the script header.
- **Edge cases:** uniffi version bumps regenerate cosmetically different code — symbol-list extraction (not byte diff) makes the gate robust. Two `PlatformBridge` traits exist (G4) — snapshot only the UniFFI one.
- **Verification:** CI fails when an agent adds/removes/renames any exported fn/record/enum without updating the snapshot; passes on no-op rebuild.

#### T5.8 Top-level README, LICENSE, CHANGELOG, agent map
- **Technical context:** module READMEs exist (`core/`, `cli/`, `wasm/`, `ios/`, `android/`) but no root docs. Workspace says MIT but no LICENSE file.
- **Implementation:** root `README.md` (architecture diagram from the audit: bridge → IronCore → message/crypto/routing/transport/drift/store layers; build prerequisites incl. NDK env var from T5.2), `LICENSE` (MIT), `CHANGELOG.md` seeded at 0.3.4, and `ARCHITECTURE.md` mapping every subsystem to its files — this is the swarm agents' navigation chart.
- **Edge cases:** keep claims accurate to code (no aspirational features — acoustic is explicitly listed as post-v1.0 in a roadmap section).
- **Verification:** `test -f README.md LICENSE CHANGELOG.md ARCHITECTURE.md`; every path referenced in ARCHITECTURE.md exists (scriptable link-check).

#### T5.9 Resolve duplicate `PlatformBridge` trait (G4)
- **Technical context:** live trait: `mobile_bridge.rs:1436` (UniFFI-exported, used by both apps). Legacy: `mobile/service.rs:87` + `platform/service.rs` (only consumer is its own `MockPlatformBridge` tests).
- **Implementation:** confirm via grep that nothing outside `mobile/` + `platform/` consumes the legacy trait; delete or `#[deprecated]`-and-quarantine the legacy `mobile/service.rs` service path (keep `mobile/auto_adjust.rs`, `ios_strategy.rs` which are referenced by the live bridge). If deletion ripples, minimum bar: rename legacy trait to `LegacyPlatformBridge` so agents can't wire the wrong one.
- **Edge cases:** `MeshService` in `mobile/service.rs` vs the UniFFI `MeshService` in `mobile_bridge.rs:153` are different types with the same name — ensure `lib.rs` re-exports stay unambiguous.
- **Verification:** `cargo test --workspace` passes; `grep -rn "trait PlatformBridge" core/src` yields exactly one non-deprecated definition.

---

### TRACK 1 — Native Hardware & Proximity Transport Layer

*(Acoustic: DEFERRED post-v1.0.0 per decision. iOS/Android BLE stacks already exist; this track closes the Wi-Fi Aware/Direct gaps and hardens BLE.)*

#### T1.1 Generalize the FFI proximity-data plane: `send_ble_packet` → transport-tagged packets (G3)
- **Technical context:** `core/src/api.udl` `PlatformBridge` callback interface; `mobile_bridge.rs:1126` (`on_ble_data_received`), `:1249` (`send_ble_packet`); Swift `SmartTransportRouter.swift`; Kotlin `SmartTransportRouter.kt`. Today only BLE bytes can cross the FFI.
- **Implementation:** add to UDL: `enum ProximityTransport { Ble, WifiAware, WifiDirect, Multipeer }`, plus `on_proximity_data_received(string peer_id, ProximityTransport transport, bytes data)` and `send_proximity_packet(string peer_id, ProximityTransport transport, bytes data)`. Keep the BLE-named methods as thin delegating wrappers (FFI surface gate T5.7 gets a snapshot update). Internally route by `TransportType` (`transport/abstraction.rs:11-22`) so `TransportCapabilities::max_payload_size` per transport (BLE 512 vs WiFiDirect 4096) is enforced at the bridge with explicit `IronCoreError::InvalidInput` on oversize.
- **Edge cases:** UniFFI 0.31 callback interfaces are sync and must not block — dispatch ingest onto the global runtime (`mobile_bridge.rs:2267-2297` pattern). Duplicate delivery when a peer is reachable over two transports — dedup already exists (`store/dedup.rs`), but verify message-id dedup fires before decrypt cost.
- **Verification:** new Rust unit tests: oversize payload per transport rejected; round-trip via a mock `PlatformBridge` for each enum variant; T5.7 snapshot updated in same change; both binding generators succeed.

#### T1.2 Wire `WifiAwareTransport` into the live core (de-orphan, G1)
- **Technical context:** `transport/wifi_aware.rs` is complete (state machine, data paths, RSSI bandwidth model, `wire_discovery_callback`) but unreferenced. Consumer seam: `MeshService.start()` (`mobile_bridge.rs:227`) and the swarm event loop (`transport/swarm.rs`). Settings flag already exists: `MeshSettings.wifi_aware_enabled` (api.udl:222 block).
- **Implementation:** (1) implement a production `WifiAwarePlatformBridge` whose methods forward over the T1.1 FFI plane (publish/subscribe/data-path requests become `PlatformBridge` calls; new UDL methods: `wifi_aware_publish(service_name, info)`, `wifi_aware_subscribe(...)`, `wifi_aware_create_data_path(peer_id, pmk)` — or fold into a generic `transport_control(transport, op, payload)` to keep the surface small (preferred). (2) Instantiate `WifiAwareTransport` inside `MeshService.start()` when `wifi_aware_enabled && bridge.is_available()`. (3) On `DataPathInfo` confirmation (IP+port), dial that socket via the existing libp2p TCP transport (`SwarmHandle.dial` path used by `SwarmBridge::dial`, `mobile_bridge.rs:2428`) so Noise/Yamux/Gossipsub ride the Aware data path with zero new protocol code. (4) PMK derivation: blake3-derive a 32-byte PMK from the DarkBLE group key (`transport/ble/beacon.rs`) so only mesh members can join data paths.
- **Edge cases:** Android-only (iOS has no Wi-Fi Aware API — bridge `is_available()` must return false on iOS; `MultipeerTransport.swift` is the iOS analog and stays on its existing path). Android requires `NEARBY_WIFI_DEVICES` (API 31+) / fine-location (≤30) at runtime — already in manifest, but the Kotlin bridge must check grant state before `is_available()=true`. Aware sessions die on Wi-Fi toggle/Doze: `on_network_changed` (existing PlatformBridge callback) must tear down `DataPathActive` state.
- **Verification:** `cargo test -p scmessenger-core wifi_aware` (existing 15 tests still pass); new integration test with `MockWifiAwareBridge` proving: discovery event → `create_data_path` → dial issued to `SwarmHandle` (assert via command-channel inspection); Kotlin unit test (Robolectric) for permission-gated availability.

#### T1.3 Android `WifiAwarePlatformBridge` native implementation
- **Technical context:** `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt` exists (publish/subscribe scaffolding) but has no FFI connection. Target: implement the Kotlin side of T1.2's bridge methods using `WifiAwareManager`/`WifiAwareSession`/`PublishDiscoverySession`/`SubscribeDiscoverySession` + `ConnectivityManager.NetworkRequest` with `WifiAwareNetworkSpecifier` (PMK variant).
- **Implementation:** wire `attach()` lifecycle to `MeshForegroundService` start/stop; on `onServiceDiscovered` → call core `on_proximity_data_received`-adjacent discovery callback (the T1.2 control channel); on network-available callback with `WifiAwareNetworkInfo` → report `(ipv6, port)` back to core. Use the link-local IPv6 + the peer's announced port from service-info TLV.
- **Edge cases:** Aware unavailable on huge swath of devices (`PackageManager.FEATURE_WIFI_AWARE` optional — manifest already `required=false`); `WifiAwareManager.isAvailable()` flaps with Wi-Fi state — register `ACTION_WIFI_AWARE_STATE_CHANGED` receiver. Doze/App Standby suspends sessions: foreground service (already present, `FOREGROUND_SERVICE_CONNECTED_DEVICE`) keeps it alive; document that battery-optimization exemption is user-prompted, never silently assumed. IPv6 link-local requires scope-id when dialing — multiaddr must be `/ip6/<addr>%<scope>/tcp/<port>` (verify libp2p multiaddr scope-id support; if unsupported, bind a local TCP proxy socket).
- **Verification:** instrumented test on two physical Android devices (documented manual procedure in `docs/device-testing.md` + an `adb`-scripted check): both report `DataPathActive`, then `SwarmBridge.get_peers()` on each shows the other's PeerId. CI-side: Robolectric tests for state machine, lint passes.

#### T1.4 Wi-Fi Direct Rust transport + Android bridge (G2)
- **Technical context:** Rust has only `TransportType::WiFiDirect` enum + capabilities (`abstraction.rs`). Kotlin `WifiDirectTransport.kt` exists (group formation scaffolding). No iOS equivalent (platform limitation — iOS has no Wi-Fi Direct API; Multipeer covers the niche).
- **Implementation:** mirror the wifi_aware.rs pattern: new `transport/wifi_direct.rs` with `WifiDirectPlatformBridge` trait (`discover_peers`, `connect(device_addr)`, `create_group`, `remove_group`, callbacks for peers-changed/connection-info). On connection-info (group owner IP 192.168.49.1 + client IPs), dial over TCP exactly as T1.2 step 3. Group-owner election: prefer the device with `is_charging || battery_pct > 50` (DeviceProfile already crosses FFI) by setting `groupOwnerIntent` accordingly on the Kotlin side.
- **Edge cases:** Wi-Fi Direct and infrastructure Wi-Fi conflict on many chipsets (STA+P2P concurrency varies) — treat `WIFI_P2P_STATE_DISABLED` as transport-down, never retry-loop. Android 13+ requires `NEARBY_WIFI_DEVICES`; legacy needs location enabled (not just granted). GO negotiation needs user-visible system dialog on some OEMs for the first connection — document as known UX constraint; invitation-based reconnect avoids it.
- **Verification:** Rust unit tests with a mock bridge (state machine, GO-intent computation from DeviceProfile); two-device manual procedure in `docs/device-testing.md`; `cargo clippy` clean; FFI snapshot updated.

#### T1.5 BLE L2CAP throughput & fragmentation hardening
- **Technical context:** Rust framing: `transport/ble/l2cap.rs` (framing/reassembly); Swift `BLEL2CAPManager.swift`; Kotlin `BLEL2CAPManager.kt`. BLE capability says 512-byte payloads (`abstraction.rs`), but DriftFrames go to 65,536 (`drift/frame.rs` MAX_PAYLOAD) — fragmentation correctness under loss is the survival-critical path.
- **Implementation:** add to `l2cap.rs`: explicit reassembly timeout (drop partial after 30 s), per-peer reassembly memory cap (e.g. 256 KiB) with `DropReason` accounting into `drift/relay.rs` stats, and CRC32 verification on reassembled frame (frame.rs already carries CRC — assert it's checked post-reassembly, add if not). Property test: random fragment loss/reorder/duplication never panics and never yields a corrupt-but-accepted frame.
- **Edge cases:** iOS L2CAP MTU negotiation differs from Android (iOS up to ~2048-byte SDU typical; Android `BluetoothSocket` L2CAP CoC API 29+); peripheral-role L2CAP listen on Android requires API 29+ — Kotlin must gate with `Build.VERSION` and fall back to GATT characteristic writes (the `gatt.rs` path) below that.
- **Verification:** `proptest` in `core/src/transport/ble/`: 10k randomized fragment streams, zero panics, corrupt frames always rejected (CRC); memory cap test: oversized partial stream evicted with logged DropReason.

#### T1.6 iOS background BLE survival audit & hardening
- **Technical context:** `ios/.../Transport/BLECentralManager.swift`, `BLEPeripheralManager.swift`; Info.plist has `bluetooth-central`/`bluetooth-peripheral` background modes + BGTaskScheduler ids (`com.scmessenger.mesh.refresh`/`.processing`); `MeshBackgroundService` calls Rust `pause()`/`resume()`.
- **Implementation:** enforce the three iOS background-BLE realities in code: (1) backgrounded advertising drops the local name and moves service UUIDs to the overflow area — central-side scan must therefore scan by service UUID (`CBCentralManagerScanOptionAllowDuplicatesKey` is ignored in background; dedupe accordingly); (2) use `CBCentralManager` state restoration (`CBCentralManagerOptionRestoreIdentifierKey`) so the OS relaunches the app on peripheral events — implement `centralManager(_:willRestoreState:)`; (3) BGProcessingTask drives periodic Drift `SyncSession` flushes — budget work to <30 s and reschedule.
- **Edge cases:** iOS kills L2CAP channels on suspend — Rust side must treat BLE peers as intermittently connected (the routing engine's `PeerStatus::Stale` path covers this; verify the staleness timeout aligns with iOS suspend cadence ~10 s). DarkBLE rotating beacons (`beacon.rs` rotation_epoch) vs. iOS overflow-area advertising: confirm the encrypted beacon fits the 28-byte overflow payload — if not, move rotation material into the scan-response/GATT read.
- **Verification:** XCTest for willRestoreState handling; documented two-device procedure: message delivered while receiving iPhone is backgrounded ≥10 min; beacon payload size statically asserted ≤ legal advertisement length in a Rust test.

#### T1.7 Transport escalation policy unification (SmartTransportRouter parity)
- **Technical context:** escalation logic exists in THREE places: Rust `transport/escalation.rs`, Swift `SmartTransportRouter.swift`, Kotlin `SmartTransportRouter.kt`. Risk: divergent decisions (e.g., Swift prefers Multipeer while Rust expects BLE), causing dual-send waste.
- **Implementation:** make Rust authoritative: expose `recommended_transport(peer_id) -> ProximityTransport` through the FFI (consumes `TransportCapabilities` + `TransportHealthMonitor` from `transport/health.rs` + DeviceProfile battery state). Native routers demote to executors: they report link availability up (`on_network_changed` extension or new `on_transport_availability(transport, available)`) and obey downward picks.
- **Edge cases:** native layer has information Rust lacks mid-flight (e.g., L2CAP channel just died) — allow native veto with mandatory report-back so the health monitor learns. Don't break existing BLE-only flows during migration: feature-flag via `MeshSettings`.
- **Verification:** Rust unit tests: given (battery, link set, payload size) → deterministic transport pick matching a documented decision table in `ARCHITECTURE.md`; grep proves Swift/Kotlin routers no longer contain independent preference ordering (only availability checks).

---

### TRACK 2 — Asynchronous Storage & Delay-Tolerant Networking

*(Drift is already live in the swarm path — this track verifies, completes the async-command seam, and proves sneakernet.)*

#### T2.1 Complete the `SwarmCommand` async-command seam (G5)
- **Technical context:** `transport/swarm.rs:1238` (`SwarmCommand` enum), `:1394` (`SwarmHandle`); stub at `iron_core.rs:3314-3318` (`update_keepalive`); `SwarmBridge` (`mobile_bridge.rs:2245+`) already wraps handle ops sync-over-async.
- **Implementation:** add `SwarmCommand::UpdateKeepalive { peer_id, interval }` (and audit the enum for other externally-needed-but-missing variants: per-peer disconnect, transport-pref hint from T1.7); plumb `IronCore::update_keepalive` through a held `SwarmHandle` (IronCore currently has no handle field — inject via the same wiring `SwarmBridge::set_handle` uses, `mobile_bridge.rs:2566`). Remove the TODO.
- **Edge cases:** command channel full/closed (swarm shut down) must return `Err`, not block — use `try_send` with explicit error mapping; WASM target excludes this path (`#[cfg(not(target_arch = "wasm32"))]` already present).
- **Verification:** `grep -n "TODO" core/src/iron_core.rs` empty; new integration test: start swarm, issue `update_keepalive`, observe keepalive change via swarm event inspection; the 7 `#[ignore]`d NAT tests in `transport/nat.rs:634-699` get a live-SwarmHandle harness and are un-ignored (or moved to the CI `--ignored` network job from T5.4 with the harness).

#### T2.2 Drift sync end-to-end verification under partition (prove the DTN claim)
- **Technical context:** `drift/sync.rs` (SyncSession state machine), `drift/sketch.rs` (IBLT), `drift/store.rs` (CRDT MeshStore), swarm activation at `transport/swarm.rs:3331+`; existing test `integration_offline_partition_matrix.rs`.
- **Implementation:** new integration test `core/tests/integration_drift_mule.rs` simulating the canonical sneakernet scenario with three in-process nodes A, M(ule), B and a partition harness (no common connectivity between A and B ever): (1) A queues messages for B while only A↔M connected; (2) A↔M disconnect, M↔B connect; (3) assert B receives, decrypts (Double Ratchet out-of-order tolerance — `MAX_SKIP_KEYS=64`), and dedup holds when M later re-syncs with A. Include IBLT failure path: difference count exceeding sketch capacity must fall back to full-list sync, not silently lose messages.
- **Edge cases:** TTL expiry during custody (envelope `expires_in_seconds` from `TtlConfig`) — expired messages must be dropped by M with `DropReason` recorded, never delivered stale; `SyncRateLimiter` (`drift/rate_limit.rs`) must not starve a short BLE contact window — verify a 10 s contact transfers ≥ N messages.
- **Verification:** `cargo test --test integration_drift_mule` green; test asserts: delivery, decryption, dedup count == 0 duplicates surfaced to history, expired-message drop with reason, and sync completes within simulated 10 s contact.

#### T2.3 Custody persistence across process death (mule survives reboot)
- **Technical context:** `MeshStore` (`drift/store.rs`) appears in-memory (`MeshStore::new()` at `iron_core.rs:264` with no path); persistent stores exist via `StorageBackend`/`SledStorage` (`store/backend.rs`) and `RelayCustodyStore` (`store/relay_custody.rs`); `test_persistence_restart.rs` covers the sled stores.
- **Implementation:** back `MeshStore` with the `StorageBackend` trait (sled on native, memory on WASM): persist drift envelopes under a `drift/` key prefix with TTL metadata; hydrate on `IronCore` construction when a storage path is provided (`MeshService::with_storage`, `mobile_bridge.rs:178`). Sweep expired envelopes in the existing `store/sweeper.rs` retention pass.
- **Edge cases:** mobile storage pressure — cap custody store (configurable, default e.g. 64 MiB / 10k envelopes) with eviction order: expired → lowest-priority → oldest; eviction must record `DropReason` for the relay stats. sled low-space mode already used by managers — reuse the same tree/config. Android `MeshForegroundService` crash-handler stops the service — ensure flush-on-stop (sled flush in `MeshService.stop()`, `mobile_bridge.rs:310`).
- **Verification:** extend `test_persistence_restart.rs`: queue 100 drift envelopes → drop and reopen `IronCore` on same path → all 100 present, expired ones swept; eviction test at cap; `cargo test -p scmessenger-core persistence` green.

#### T2.4 Background sync scheduling on both platforms
- **Technical context:** iOS `MeshBackgroundService` + BGTaskScheduler ids (registered in Info.plist); Android `MeshForegroundService` + `RECEIVE_BOOT_COMPLETED`. Core API: `MeshService.pause()/resume()`, Drift `new_drift_sync()` (`iron_core.rs:3027`).
- **Implementation:** iOS: `BGProcessingTaskRequest` (`com.scmessenger.mesh.processing`) handler runs a bounded drift maintenance cycle — new core FFI `run_maintenance_cycle(budget_ms: u32) -> MaintenanceReport` wrapping `drift/relay.rs` maintenance + sweeper, guaranteed to return within budget. Android: `WorkManager` periodic job (15 min floor) as belt-and-suspenders alongside the foreground service, calling the same FFI; boot receiver restarts foreground service (receiver exists per manifest — verify it actually starts the service on API 34+ where BOOT_COMPLETED FGS-launch needs `FOREGROUND_SERVICE_DATA_SYNC` type, already declared).
- **Edge cases:** iOS grants processing tasks rarely (often only when charging+idle) — never depend on it for correctness, only opportunistic sync; budget enforcement must be cooperative (check elapsed in loop) since Rust can't be preempted. Android 14 restricts FGS start from BOOT_COMPLETED to specific types — `dataSync` qualifies but verify with targetSdk used.
- **Verification:** Rust unit test: `run_maintenance_cycle(50)` returns in <100 ms wall-clock with work remaining flagged in report; XCTest registering the BG task handler; Android instrumented test (or Robolectric) that boot receiver schedules the service; FFI snapshot updated (T5.7).

#### T2.5 Outbox retry × Drift custody convergence audit
- **Technical context:** two queuing systems coexist: `store/outbox.rs` (QueuedMessage + `SmartRetryManager`, `routing/smart_retry.rs`) and drift custody (T2.3). Risk: same message retried over live swarm AND muled via drift → duplicate sends, double battery cost.
- **Implementation:** define and enforce a single ownership rule in `iron_core.rs` send path (`iron_core.rs:602` is where legacy→drift envelope conversion happens): when `RoutingDecision.primary == NextHop::StoreAndCarry`, message moves to drift custody and is *removed* from active outbox retry (state-marked `InCustody`); a delivery receipt (`integration_receipt_convergence.rs` machinery) clears both. Document the state machine in `ARCHITECTURE.md`.
- **Edge cases:** receipt arrives via a different transport than delivery (likely in mesh) — receipt handling is already transport-agnostic by message_id, verify; custody→live transition when a direct route appears (routing engine `resume_prefetch.rs`) must atomically re-claim from drift store to outbox without a window where both own it.
- **Verification:** new test in `integration_retry_lifecycle.rs`: force StoreAndCarry, assert outbox stops retrying; restore route, assert exactly-one delivery (dedup count 0 at recipient); state-transition property test (no state where both systems own the message).

---

### TRACK 3 — Mycorrhizal Routing & Hardware-Aware Heuristics

*(Engine is built and tested. Gaps: phase2 features are off by default, hardware signals don't yet modulate routing, and the engine's StoreAndCarry decision needs to drive Track 2.)*

#### T3.1 Promote `phase2_apis` (reputation + multipath) into default build
- **Technical context:** `routing/reputation.rs` + `routing/multipath.rs` gated behind `phase2_apis` cargo feature; `transport/mesh_routing.rs` (ReputationTracker, MultiPathDelivery) ships unconditionally — audit overlap between the two reputation systems (`abuse/reputation.rs` is a third, abuse-scoped one).
- **Implementation:** run full suite with `--features phase2_apis`; reconcile the routing-reputation vs mesh_routing-ReputationTracker duplication (pick the routing-layer one as authoritative for path choice, abuse one stays for blocking); then move `phase2_apis` code into default features (delete the gate or invert to an opt-out).
- **Edge cases:** WASM build must still compile (check the feature isn't accidentally pulling tokio-full into wasm32); multipath duplicate-send interacts with T2.5 ownership rule — multipath counts as ONE owner (the live path) with internal redundancy.
- **Verification:** `cargo test --workspace --all-features` and `cargo build --target wasm32-unknown-unknown` both green; `grep -rn "phase2_apis" core/` only in CHANGELOG; one authoritative routing-reputation source asserted in ARCHITECTURE.md.

#### T3.2 Hardware-aware routing cost function (battery/charging/motion → route choice)
- **Technical context:** `DeviceProfile` (battery_pct, is_charging, has_wifi, MotionState) reaches Rust via `update_device_state` (`mobile_bridge.rs:839`) and feeds `AutoAdjustEngine` (BLE scan intervals, relay budgets) — but NOT the `RoutingEngine`'s next-hop choice (`routing/engine.rs:128`, `route_message`). Peers' device states partially propagate via gossip (`neighborhood.rs` gateway info).
- **Implementation:** extend `RoutingDecision` scoring: when choosing among `alternatives`, weight gateway/relay candidates by advertised energy class. Concretely: add a 2-bit energy class (Charging/High/Low/Critical) to the neighborhood gossip record (`NeighborhoodGossip` — bump gossip schema version with backward-compat decode); cost function: `cost = base_hop_cost * energy_multiplier[class] * (1/confidence)` with multipliers {Charging:0.5, High:1.0, Low:2.0, Critical:8.0}. A Critical-battery peer is chosen only when it is the sole route.
- **Edge cases:** gossip schema versioning — old peers send records without energy class: default to High (neutral), never reject; energy class is adversarially spoofable — cap its influence (multiplier bounds above) and let `ReputationTracker` delivery-failure feedback dominate over time; do not leak precise battery % over the mesh (privacy) — 2-bit class only.
- **Verification:** unit tests in `routing/engine.rs`: given equal-hop alternatives, charging peer wins; critical peer chosen only as sole route; gossip decode of old-schema record defaults correctly (round-trip test both directions); `integration_mycorrhizal_routing.rs` extended with an energy-skewed topology asserting route selection.

#### T3.3 `StoreAndCarry` decision → Drift handoff wiring (close routing→DTN loop)
- **Technical context:** `NextHop::StoreAndCarry` (`routing/engine.rs:18-36`) exists, and drift is live (Track 2) — verify the actual code path from `route_message` returning StoreAndCarry to envelope landing in `MeshStore` custody. Grep suggests `iron_core.rs:602` converts envelopes but the decision→custody linkage needs proof.
- **Implementation:** trace and (if missing) implement: send path consults `RoutingEngine.route_message()`; on `StoreAndCarry`, invoke the T2.5 custody handoff; on `RouteDiscovery{hint}`, trigger neighborhood route request (`routing/global.rs` `request_route`) with `timeout_budget.rs` phase budget, falling back to StoreAndCarry on exhaustion. This task is *verification-first*: write the failing test, then add only the missing glue.
- **Edge cases:** routing engine optional (`iron_core.rs:3277` shows `Option<RoutingEngine>`) — when None (e.g., WASM minimal), send path must default to direct-or-custody, never panic; priority field (u8) must map to drift `RelayProfile` priority thresholds consistently (one mapping table, tested).
- **Verification:** integration test: node with zero peers sends → message in drift custody with correct TTL/priority; peer appears later → delivered (this is T2.2's scenario driven through the public send API rather than drift internals — both must pass).

#### T3.4 Routing telemetry for field debugging (zero-infrastructure observability)
- **Technical context:** `observability.rs` (audit events), `transport/diagnostics.rs` (`NetworkDiagnosticsReport`), `DiagnosticsReporter.kt` export path. Survival deployments can't attach debuggers — the device must self-report why a message took the path it took.
- **Implementation:** ring buffer (last 256) of `RoutingDecision`s (already serializable-shaped: decided_by layer, confidence, primary/alternatives) attached to `NetworkDiagnosticsReport`; expose via existing diagnostics FFI; include drift custody stats (count, oldest age, drop reasons) and per-transport health from `health.rs`. No new persistent storage — memory ring only (privacy: cleared on app kill).
- **Edge cases:** report must redact recipient hints (4-byte hints are already privacy-preserving, but don't include message_ids alongside peer_ids in the same record — keep them unjoinable); bound report size (<256 KiB) for export-via-QR/file use.
- **Verification:** unit test: 300 decisions → ring holds last 256; report JSON schema-validated and size-bounded; redaction asserted (no message_id+peer_id co-occurrence).

---

### TRACK 4 — Cryptographic Identity, Anti-Entropy & UI Hardening

*(Crypto core is production-grade. Gaps are at the edges: session persistence, key verification UX, and zero-status honesty.)*

#### T4.1 Ratchet session persistence audit & restart safety
- **Technical context:** `crypto/session_manager.rs` (ratchet session persistence), `crypto/ratchet.rs` (`MAX_SKIP_KEYS=64`, zeroizing keys), `identity/store.rs` (sled-backed). Risk class: app restart mid-conversation losing ratchet state → permanent decrypt failure.
- **Implementation:** verification-first: integration test that (1) establishes ratchet A↔B, exchanges 10 messages, (2) serializes/persists A's session, drops A's process state, rehydrates, (3) continues conversation both directions including an out-of-order message from before the restart. Audit that persisted session material is encrypted-at-rest or at minimum that the threat model (device storage = trusted) is documented; confirm `Zeroize` on the serialization buffers (a `Zeroizing<Vec<u8>>` wrapper on the encode path).
- **Edge cases:** skipped-keys map across restart (out-of-order buffer must survive persistence or the test above fails); concurrent session mutation during flush (parking_lot guards exist — verify no deadlock with sled flush reentrance, `identity/store.rs` reopens-on-drop pattern is suspicious here).
- **Verification:** new `core/tests/integration_ratchet_persistence.rs` green; proptest: random persist/restore points in a 200-message exchange never produce decrypt failure beyond the documented MAX_SKIP_KEYS window.

#### T4.2 Out-of-band identity verification (safety numbers + QR)
- **Technical context:** identity = Ed25519, `identity_id = blake3(pubkey)` (`identity/keys.rs`); iOS already has `NSCameraUsageDescription` for QR contact scanning; `InviteSystem`/`InviteToken` (`relay/invite.rs`) exists for bootstrap.
- **Implementation:** Rust: `safety_number(our_pubkey, their_pubkey) -> String` — 60-digit numeric (Signal-style: blake3 over sorted pubkeys, chunked to 5-digit groups), exposed via FFI; QR payload format: versioned CBOR {version, pubkey, device_id, sig-over-payload} reusing invite-token signing; `Contact` record (api.udl:249) gains `verified_at: Option<u64>` (storage-side; UDL field addition = FFI snapshot update). Mark-verified API on `ContactManager`.
- **Edge cases:** key change after verification (new device) MUST flip verified→unverified and surface a UI event — hook `last_known_device_id` change detection in `contacts_bridge.rs`; QR payload must not embed nickname (privacy at scan time); safety number must be order-independent (sorted keys) so both sides display identically.
- **Verification:** Rust unit tests: same number both directions; differs on any key change; QR payload round-trip + signature verify + reject-tampered; UDL snapshot updated; Kotlin/Swift compile.

#### T4.3 Anti-entropy for contact/block state (CRDT reconciliation of social graph across own devices)
- **Technical context:** `ContactManager` (`contacts_bridge.rs`), `BlockedIdentity` store (`store/blocked.rs`, `blocked_bridge.rs`), `reconcile_from_history` exists (`contacts_bridge.rs:234`). Multi-device same-identity sync has registration machinery (`store/relay_custody.rs` RegistrationState, seniority_timestamp) but contacts/blocks don't sync.
- **Implementation:** model contacts/blocks as LWW-register CRDTs keyed by peer_id (timestamp = sender_timestamp, tiebreak = device seniority): serialize deltas into drift envelopes addressed to own identity (self-addressed custody — the mesh mules your own profile between your devices); merge on receipt with deletion-tombstones (BlockedIdentity already has `is_deleted` — extend Contact with tombstone). **Blocks must win conflicts**: a block from ANY device overrides a concurrent unblock (safety-first merge bias, documented).
- **Edge cases:** tombstone GC (sweeper integration, retain ≥90 days matching RetentionConfig); clock skew between own devices — LWW uses sender_timestamp but bound acceptance to ±24 h of local clock with seniority tiebreak beyond; encrypted with own-identity keys (self-addressed envelopes already encrypt to recipient = self).
- **Verification:** unit tests: concurrent block+unblock → blocked; add+remove → tombstone wins per LWW; integration: two IronCore instances same identity, partitioned edits, drift-merge → identical contact/block sets (extend `integration_contact_block.rs`).

#### T4.4 Zero-status UI hardening (honest state surfacing, no fake liveness)
- **Technical context:** `ConnectionPathState` enum already models honesty levels (Disconnected/Bootstrapping/DirectPreferred/RelayFallback/RelayOnly — api.udl:144); `ServiceState`; delivery vs sent receipts (`MessageRecord.delivered`). Survival-context requirement: UI must NEVER imply connectivity/delivery that isn't cryptographically confirmed.
- **Implementation:** define the canonical message-state machine exposed over FFI: `Queued → InCustody(mule) | Sent(transport) → Delivered(receipt verified) → Read(optional)` — add `MessageStatus` enum to UDL replacing the bare `delivered` bool (keep bool as derived for compat); custody state explicitly distinct from sent ("being carried by the mesh" ≠ "reached recipient"). Swift `ChatViewModel.swift` + Kotlin equivalents render: no checkmark until receipt-verified; explicit "carried by N hops" indeterminate state; `ConnectionPathState.Disconnected` shows mesh-only mode prominently, never a spinner implying imminent internet.
- **Edge cases:** receipt forgery — receipts must be signature-verified against recipient pubkey before flipping Delivered (`on_receipt_received` path at `mobile_bridge.rs` — audit that verification happens in Rust, not trusted from transport); status regression (Delivered never downgrades); WASM/CLI parity for the enum.
- **Verification:** Rust state-machine property test (no illegal transitions, monotone progress); receipt-forgery test: unsigned/wrong-key receipt does NOT flip status; FFI snapshot updated; Swift+Kotlin unit tests for render mapping (status → glyph) committed alongside.

#### T4.5 Key backup/recovery flow verification
- **Technical context:** `crypto/backup.rs` exists (key backup/recovery); AuditEventType has BackupExported/BackupImported; no evidence of end-to-end test or mobile UX wiring.
- **Implementation:** verification-first: integration test exporting identity+ratchet sessions+contacts to an encrypted backup blob (passphrase-derived key — audit `backup.rs` KDF: must be Argon2id or scrypt, NOT bare blake3 of passphrase; add if missing, this is the one likely real crypto gap), importing on a fresh IronCore, asserting full conversational continuity (can decrypt next ratchet message). Wire export/import through FFI if not present.
- **Edge cases:** backup of a *registered* device must handle seniority (imported device re-registers, doesn't clone seniority — interaction with `RegistrationState`); partial import (corrupt blob) must be atomic — all-or-nothing with explicit `CorruptionDetected` error (enum variant exists); passphrase KDF parameters must be embedded in blob header for forward-compat.
- **Verification:** roundtrip integration test; tampered-blob test → CorruptionDetected, no partial state; KDF asserted memory-hard (test that derivation takes >100 ms or checks algorithm tag); audit events emitted both directions.

---

## Dependency Graph (execution order for the swarm)

```
T5.1 → T5.2 → T5.3 → T5.4 → T5.5 → T5.6
                        ↘ T5.7 (after T5.5 artifacts)
T5.8, T5.9 — parallel anytime after T5.1

T1.1 (requires T5.7 gate live) → T1.2 → T1.3
                               → T1.4
T1.5, T1.6 — parallel after T5.4 (need CI to verify)
T1.7 after T1.1

T2.1 — after T5.4
T2.2 — after T2.1
T2.3 — after T2.2
T2.4 — after T2.3 + T1.6
T2.5 — after T2.3

T3.1 — after T5.4
T3.2 — after T3.1
T3.3 — after T2.5 + T3.1
T3.4 — after T3.3

T4.1 — after T5.4 (parallel with Track 1)
T4.2 — after T5.7
T4.3 — after T2.3
T4.4 — after T5.7
T4.5 — after T4.1

v1.0.0 gate: ALL above + two-device field procedures in docs/device-testing.md executed and logged.
```

## Deferred post-v1.0.0 (explicit non-goals now)
- **Acoustic/ultrasonic transport** (zero existing code; FSK/GGWave-class modem, FEC, AVAudioSession/AAudio integration) — revisit after BLE/Wi-Fi mesh is field-proven.
- Git history rewrite for artifact purge (T5.1 removes from HEAD only).
- iOS Wi-Fi Aware (no OS API — permanent platform constraint; Multipeer is the iOS proximity answer).

## Verification strategy (global)
1. Every task lands with its named automated test; CI (T5.4-T5.7) is the gate for all subsequent tracks.
2. `cargo test --workspace --all-features` + cross-compile matrix + FFI surface snapshot must be green on every merge.
3. Physical two-device procedures (T1.3, T1.4, T1.6) are documented, scripted where adb/xcrun allows, and their logs committed to `docs/device-testing/` as release evidence.
4. v1.0.0 release criterion: the T2.2 sneakernet scenario passes as an automated test AND as a physical three-device field test (A → mule → B with no shared connectivity).
