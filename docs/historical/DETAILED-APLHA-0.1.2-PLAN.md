# SCMessenger v0.1.2-alpha Plan Update: WASM libp2p Swarm for Full Cross-Version Compatibility

## Summary
- Compatibility verdict: **No**. Current WebSocket/WebRTC in web is not full-compatible with other app versions because web bypasses libp2p swarm and uses a separate relay receive loop.
- Root gap: [core/src/transport/swarm.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/transport/swarm.rs) still hard-fails on `wasm32`; [wasm/src/lib.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/src/lib.rs) and [wasm/src/transport.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/src/transport.rs) use custom transport paths instead of swarm protocol parity.
- Decision: **v0.1.2-alpha ships with wasm libp2p swarm as required path**. Legacy web relay loop remains only as temporary compatibility wrapper.

## Locked Compatibility Contract
1. Keep protocol IDs unchanged across versions:
   - `/sc/message/1.0.0`
   - `/sc/address-reflection/1.0.0`
   - `/sc/relay/1.0.0`
   - `/sc/ledger-exchange/1.0.0`
   - `/sc/id/1.0.0`
2. Keep default topics unchanged:
   - `sc-lobby`
   - `sc-mesh`
3. Compatibility target matrix is explicit:
   - Browser `v0.1.2-alpha` ↔ native tag `v0.1.0`
   - Browser `v0.1.2-alpha` ↔ native tag `v0.1.1`
   - Browser `v0.1.2-alpha` ↔ current head (`v0.1.2-alpha` branch)

## Change Plan (Specific by Implementation Area)

### 1) Dependency and feature topology
Files:
- [Cargo.toml](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/Cargo.toml)
- [core/Cargo.toml](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/Cargo.toml)
- [wasm/Cargo.toml](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/Cargo.toml)

Changes:
1. Enable libp2p `wasm-bindgen` and `websocket-websys` features for wasm target compilation.
2. Keep native tokio/tcp/quic features scoped to non-wasm target.
3. Do not bump libp2p major/minor for alpha unless blocked; stay on `0.53.2` to reduce regression risk.

Acceptance:
1. `cargo check -p scmessenger-core --target wasm32-unknown-unknown` succeeds with swarm code compiled.
2. Native build behavior remains unchanged for iOS/Android/CLI.

### 2) Implement wasm swarm path in core transport
Files:
- [core/src/transport/swarm.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/transport/swarm.rs)
- [core/src/transport/behaviour.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/transport/behaviour.rs)

Changes:
1. Replace wasm bail-out in `start_swarm_with_config` with real wasm swarm builder path using `with_wasm_bindgen()`.
2. Use browser-capable transport via `with_other_transport(...)` using libp2p websocket-websys transport upgraded with noise + yamux.
3. Reuse existing `IronCoreBehaviour` and existing request-response/gossipsub/kad logic for protocol parity.
4. Keep mdns excluded on wasm (`cfg` already present).
5. Keep relay client behavior enabled so browser can route through relay-capable bootstrap nodes.
6. Preserve command/event API shape (`SwarmHandle`, `SwarmCommand`, `SwarmEvent2`) so mobile/CLI surfaces stay stable.

Acceptance:
1. `start_swarm_with_config` on wasm returns `SwarmHandle` (no panic/bail).
2. Browser peer discovers and exchanges request-response messages with native swarm peers.

### 3) Wasm runtime/event-loop adaptation for swarm
Files:
- [core/src/transport/swarm.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/transport/swarm.rs)

Changes:
1. Add wasm-specific event-loop spawning strategy (`wasm_bindgen_futures::spawn_local`) while preserving tokio path for native.
2. Keep retry/bootstrap timers operational in wasm branch with wasm-compatible timer strategy.
3. Define explicit wasm command semantics:
   - `Listen`: returns unsupported error.
   - `GetListeners`: returns empty list.
   - `Dial`, `SendMessage`, `GetPeers`, `SubscribeTopic`, `PublishTopic`: fully supported.

Acceptance:
1. Command behavior is deterministic and documented for wasm limitations.
2. No deadlocks or dropped command channel under browser runtime tests.

### 4) Replace web app transport entrypoint with swarm
Files:
- [wasm/src/lib.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/src/lib.rs)
- [wasm/src/transport.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/src/transport.rs)

Changes:
1. Add swarm lifecycle exports in wasm bindings:
   - `startSwarm(bootstrapAddrs)`
   - `stopSwarm()`
   - `sendPreparedEnvelope(peerId, envelopeBytes)`
   - `getPeers()`
2. Route inbound swarm `MessageReceived` events into the existing JS-drain buffer path so UI consumption pattern remains stable.
3. Keep `startReceiveLoop(relayUrl)` for one alpha cycle as deprecated shim that calls swarm bootstrap with converted address.
4. Remove release dependency on legacy `WebRtcPeer` stub path; keep code only behind non-default fallback flag if needed for rollback.

Acceptance:
1. Browser JS can send and receive through swarm APIs without direct use of legacy relay helper.
2. Existing UI using drain-based receive model still functions.

### 5) Bootstrap and addressing standardization
Files:
- [core/src/mobile_bridge.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/mobile_bridge.rs)
- [wasm/src/lib.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/src/lib.rs)

Changes:
1. Standardize bootstrap config to full libp2p multiaddrs for web (`/dns4/.../tcp/.../wss/...` plus `/p2p/<peer_id>` where available).
2. Require at least one browser-reachable bootstrap relay endpoint in alpha environment config.
3. Enforce deterministic path policy in web matching native: bootstrap -> direct probe if available -> relay settle.

Acceptance:
1. Browser can dial at least one configured bootstrap node from public internet.
2. Path transitions are surfaced in telemetry and match native state model.

### 6) Public API/interfaces/types updates (must document and freeze)
Files:
- [wasm/src/lib.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/src/lib.rs)
- [wasm/README.md](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/README.md)
- [core/src/api.udl](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/src/api.udl)

Changes:
1. Add wasm swarm lifecycle/send/peer methods listed above.
2. Mark `startReceiveLoop` as deprecated in docs and runtime warnings.
3. Keep existing message envelope format and receipt semantics unchanged.
4. Keep UniFFI API stable unless parity gap is identified; if changed, regenerate bindings as one atomic change.

Acceptance:
1. API docs explicitly distinguish deprecated relay helper from required swarm path.
2. No breaking changes for existing native UniFFI consumers in `v0.1.2-alpha`.

### 7) Testing and compatibility scenarios (release blockers)
Files:
- [core/tests](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/core/tests)
- [wasm/src/lib.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/src/lib.rs)
- [wasm/src/transport.rs](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/src/transport.rs)
- [.github/workflows/ci.yml](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/.github/workflows/ci.yml)

Required test cases:
1. Wasm swarm startup test proves no wasm bail-out path remains.
2. Browser->native send/receive against `v0.1.0` node.
3. Browser->native send/receive against `v0.1.1` node.
4. Browser->native send/receive against current head.
5. Relay-only scenario with direct route blocked.
6. Mid-send network interruption and retry reconciliation.
7. Duplicate suppression and ACK integrity across path changes.
8. Resume/reconnect after browser tab suspend.

Quality gates:
1. Compatibility gate: all three version pairs pass bidirectional messaging with zero loss/duplication.
2. WASM gate: wasm-pack browser tests pass with swarm path enabled.
3. Regression gate: existing core/native suites remain green.

### 8) Documentation and plan artifact updates
Files:
- [APP_VERSION_0.1.2_ALPHA_PLAN.md](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/APP_VERSION_0.1.2_ALPHA_PLAN.md)
- [docs/STUBS_AND_UNIMPLEMENTED.md](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/STUBS_AND_UNIMPLEMENTED.md)
- [docs/REPO_CONTEXT.md](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/REPO_CONTEXT.md)
- [wasm/README.md](/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/wasm/README.md)

Changes:
1. Update alpha plan sections for “WASM swarm parity” as non-negotiable.
2. Close the S0 wasm swarm stub item once implemented.
3. Document new wasm swarm APIs and deprecation window for legacy receive loop.

Acceptance:
1. No docs still claim web is relay-loop-only after implementation.
2. Release checklist includes cross-version compatibility matrix results.

## Revised LOC Estimate (implementation + tests)
1. Dependency/feature and build graph updates: 120-220 LOC
2. Core wasm swarm implementation and event loop: 650-1,050 LOC
3. Wasm binding/API integration: 300-550 LOC
4. Legacy transport shim/deprecation cleanup: 120-260 LOC
5. Tests/CI/docs updates: 500-900 LOC
6. Added total for this change: **1,690-2,980 LOC**
7. Revised overall v0.1.2-alpha total: **5,070-8,810 LOC**

## Assumptions and defaults (explicit)
1. Default transport for browser compatibility is libp2p swarm over websocket-websys + relay path.
2. WebRTC direct browser transport is not a release blocker for `v0.1.2-alpha`; it is optional follow-on once compatibility gate is green.
3. At least one bootstrap relay endpoint is browser-reachable via WSS in alpha infra.
4. Protocol versions remain at `1.0.0` for this alpha to preserve `v0.1.0`/`v0.1.1` compatibility.
5. If `0.53.2` blocks implementation unexpectedly, only then upgrade libp2p and rerun full matrix before merge.

## External research references used
- [libp2p 0.53.2 features (wasm-bindgen/websocket-websys)](https://docs.rs/crate/libp2p/0.53.2/features)
- [rust-libp2p browser-webrtc wasm swarm example](https://raw.githubusercontent.com/libp2p/rust-libp2p/master/examples/browser-webrtc/src/lib.rs)
- [libp2p websocket-websys transport API](https://raw.githubusercontent.com/libp2p/rust-libp2p/master/transports/websocket-websys/src/lib.rs)
- [libp2p-webrtc-websys README (wasm swarm usage)](https://raw.githubusercontent.com/libp2p/rust-libp2p/master/transports/webrtc-websys/README.md)
