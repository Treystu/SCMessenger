# TASK: P1-CORE-WIN-WIFIDIRECT — No Windows-side WiFi Direct path exists; `wifi_direct.rs` compiles on Windows but is inert

**Tier:** [OPUS+] spec/design first, then [SONNET] implementation. Feeds and de-risks P1-17.
**Gates:** [AUDIT-GATE] (touches `core/src/transport/`), [DEVICE] (Pixel + Windows Wi-Fi stack required), [HUMAN] (operator scope call per plan Section 5.3).

## Source

P1-15 transport-matrix ground-truth audit (`HANDOFF/plans/P1-15_transport_matrix_audit.md`, section (d)), 2026-07-04 [V-READ]. Settles the P1-15 question "(d) Windows-side WiFi Direct reality" and feeds the P1-17 scope decision.

## Problem (exact, verified)

Android<->Windows WiFi Direct has **no Windows participant** today.

- `core/src/transport/wifi_direct.rs` is gated `#[cfg(not(target_arch = "wasm32"))]` (`:85-212`) and declared unconditionally (`core/src/transport/mod.rs:28`), so it **compiles on Windows** — but that is not an implementation. Every action delegates to `crate::PlatformBridge`: `discover_peers` -> `wifi_direct_discover_peers()` (`:154`), `connect` -> `wifi_direct_connect()` (`:174`), `create_group` -> `wifi_direct_create_group()` (`:187`). `is_available()` returns `true` only if a platform bridge is present (`:149`).
- `PlatformBridge` (`core/src/mobile_bridge.rs:1789`, with `wifi_direct_*` at `:1808-1812`) is implemented in production **only on Android** (`android/.../service/AndroidPlatformBridge.kt:517-555`). The CLI has **no** `PlatformBridge` impl and **no** WiFi Direct transport instantiation (`cli/src/` has only `enable_wifi_direct`-style config flags for status). Grep of `desktop_bridge/` for `wifi_direct`/`WifiDirect` is clean — the only native Windows-ish bridge crate has no WiFi Direct at all.
- The data link, when it does form (Android GO), is a plain TCP dial: the client hardcodes `/ip4/{group_owner_ip}/tcp/9001` (`core/src/mobile_bridge.rs:1398`). `set_on_message_received` is a no-op even in the real Rust bridge (`wifi_direct.rs:211`) — data does not flow through a discrete callback, it flows over the group IP link.

So: Android can create/join a WiFi Direct group and TCP-dial the group owner, but on the Windows side there is no code that joins an Android P2P group or dials into it.

## Root Cause

WiFi Direct was built as an Android-native capability (Android `WifiP2pManager`, GO-intent logic already landed per plan T1.4). The Rust `wifi_direct.rs` is a thin platform-bridge shim, not a cross-platform transport. Windows was never given a peer role because Windows Wi-Fi Direct requires a different mechanism (WinRT `WiFiDirect*` APIs, or joining the Android group's SoftAP as a legacy client over the normal Wi-Fi stack).

## Blast Radius

- Design-dominated. The [OPUS+] output is a 1-2 page spec deciding the Windows role:
  - **Recommended (per plan P1-17):** Windows joins the Android-created P2P group as a **legacy client** over its ordinary Wi-Fi stack (the group is a SoftAP with a passphrase Android controls), then TCP-dials the group-owner IP on the negotiated port (post-P1-13, not hardcoded 9001). This keeps the change to CLI dial logic + credential exchange, avoiding a WinRT WiFiDirect native module.
  - **Alternative:** a WinRT `WiFiDirectDevice` native path in `desktop_bridge`/CLI — substantially larger, [DEVICE]-heavy, and arguably out of scope vs. the legacy-client approach.
- Implementation, once the spec lands, touches the CLI dial path + how the group passphrase/GO-IP reaches Windows (peer_exchange or QR/manual). Touches `core/src/transport/` -> `crypto-security-auditor` gate.
- [HUMAN] scope call is embedded: this is the "WiFi Direct Windows-side scope" open decision in plan Section 5.3. If the operator waives, the matrix cell narrows to Android<->Android [BLOCKED-HW] with a recorded waiver (do NOT silently downgrade — plan P1-17 rule).

## Files to Touch

- `HANDOFF/plans/` — the [OPUS+] design note first (Windows WiFi Direct role decision).
- Then (implementation): `cli/src/` dial path (join-as-legacy-client + TCP dial to GO IP), credential/GO-IP ingress (coordinate with P1-12 peer_exchange address propagation and P1-13 port de-hardcoding).
- `core/src/mobile_bridge.rs:1398` — the hardcoded `tcp/9001` client dial is a P1-13 item; ensure the Windows path uses the negotiated port, not a second hardcode.

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
# Device (P1-17): Android creates group (GO); Windows joins group SoftAP as legacy client;
# Windows TCP-dials GO IP on negotiated port; confirm E2E message both directions.
```

## Do NOT

- Do NOT claim the matrix WiFi Direct cell is closable Windows<->Android without either this implementation or an operator-recorded waiver. Absence of a Windows peer is the finding, not a bug in `wifi_direct.rs`.
- Do NOT add a second hardcoded port for the Windows dial — consume the negotiated/actual port from P1-13.
- Do NOT start the WinRT WiFiDirect native module without operator sign-off — the legacy-client approach is the recommended, smaller path; a native module is a stack-addition escalation (`.claude/rules` / CLAUDE.md escalation rule).
- Mandatory `crypto-security-auditor` review before implementation is done (transport path). `release-gatekeeper` before merge.
