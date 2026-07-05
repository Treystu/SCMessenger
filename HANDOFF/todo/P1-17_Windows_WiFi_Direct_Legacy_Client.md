# TASK: P1-17-IMPL — Windows joins an Android WiFi Direct group as a legacy client and TCP-dials the group owner

**Tier:** [SONNET] [AUDIT-GATE] [DEVICE] [HUMAN-gated]
**Phase:** v1.0.0 Phase 1, Stage D (transport-matrix WiFi Direct cell, 2.6).
**Design source:** `HANDOFF/plans/P1-17_windows_wifi_direct_design.md` (read it in full first — Sections 1, 2, 3, 4).
**GATED BY (do not start until all true):**
- **[HUMAN] operator BUILD decision.** Section 2 of the design note requires an operator sign-off (build vs. waiver) taken *after* the hardware pre-check. If the operator waived, this ticket is void — narrow the matrix cell to Android <-> Android [BLOCKED-HW] instead and delete this ticket.
- **Hardware pre-check PASSED.** The manual Windows-join-to-Android-GO smoke test (design note Section 6, steps 1-3) must have confirmed this Windows adapter/driver will associate to a Pixel `DIRECT-*` SoftAP and reach the GO IP. If it did not, waive — do not attempt to code around a driver-level rejection.
- **P1-13 landed** (negotiated/laddered port; `mobile_bridge.rs:1398` no longer hardcodes `9001`). P1-13 depends on P1-11/P1-12, which queue behind P1-04. This ticket consumes the negotiated port and MUST NOT introduce a second hardcode.

## Source

`HANDOFF/V1_0_0_EXECUTION_PLAN.md` P1-17 (Stage D). Feasibility settled in `HANDOFF/plans/P1-17_windows_wifi_direct_design.md` this session ([V-READ], no toolchain in sandbox — re-verify with a real build + device pass on the Windows box). Root gap surfaced by `HANDOFF/plans/P1-15_transport_matrix_audit.md` section (d) and ticket `P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md`.

## Problem (exact, verified)

There is no Windows participant in WiFi Direct today. `core/src/transport/wifi_direct.rs` compiles on Windows but is inert (its `PlatformWifiDirectBridge` delegates every action to `crate::PlatformBridge`, implemented in production only by Android's `AndroidPlatformBridge.kt`). The CLI has no `PlatformBridge` impl and no WiFi Direct transport. `desktop_bridge/` is Linux-only. So Windows <-> Android over a Direct group cannot happen. [V-READ, P1-15(d)]

The Android side already creates/joins a group (GO-intent logic landed, T1.4) and, on the client, TCP-dials the GO IP (`core/src/mobile_bridge.rs:1394-1400`, currently hardcoded `/ip4/{go_ip}/tcp/9001`). An Android GO is, at the radio layer, a WPA2 SoftAP with an SSID + passphrase; a Windows box can join it as an ordinary Wi-Fi station (legacy client) and then reach the GO over normal IP — no Windows WiFiDirect P2P API required. What is missing is (a) getting the group credentials + GO IP:port to Windows, and (b) Windows joining + dialing.

## Root Cause

WiFi Direct was built Android-native (Android `WifiP2pManager`). The Rust `wifi_direct.rs` is a thin platform-bridge shim, not a cross-platform transport. Windows was never given a peer role. The legacy-client approach (Windows joins the GO's SoftAP over its normal Wi-Fi stack) was never implemented because there was no credential-ingress path to Windows and no CLI join/dial step.

## Scope / What to do

Smallest-surface slice first (design note Section 3, "manual/QR credential ingress + CLI dial + reuse negotiated port"):

1. **Credential ingress (no wire-protocol change).** Add a CLI path to accept group credentials `{ssid, passphrase, group_owner_ip, port}` as a pasted string and/or scanned QR. Reuse the existing identity-QR/string plumbing shape; do NOT add a new `peer_exchange` wire message (that is P1-12's audit-gated decision — design note §4 item 4). The `port` value is the GO's negotiated port from P1-13; treat it as authoritative, never default to `9001`.

2. **Windows Wi-Fi join.** Associate Windows to the group SSID with the passphrase.
   - **First slice: manual OR `netsh wlan` shell-out** (zero new dependency). Add a WLAN profile from the passphrase and `netsh wlan connect`, or document a manual "join this SSID in Settings" step and have the CLI proceed once the GO IP is reachable.
   - **Do NOT** add the `windows`-crate native Wi-Fi join (option 2) or any WiFiDirect P2P (`Windows.Devices.WiFiDirect`) module in this slice — the native `Devices_WiFi` dependency is a stack-addition escalation that requires separate operator sign-off (design note §4 item 2). It is a *follow-up refinement*, not part of closing the cell.

3. **Dial the group owner.** Once associated and the GO IP is reachable, TCP-dial `group_owner_ip:port` via the existing CLI dial path (`cli/src/main.rs` dial loop ~1462-1494 -> `SwarmCommand::Dial`, `core/src/transport/swarm.rs:4104`). Reuse the promiscuous dialer; do not add a parallel dial mechanism.

4. **Port handling.** Consume the negotiated/laddered port from P1-13. If the GO port is not carried in `GroupInfo` (i.e. the `GroupInfo.port` field from P1-10 §4 item 2 was not approved), use the no-wire-change fallback: try the port ladder against `group_owner_ip` (443/80/8080/9090/negotiated), per the design note §1.3 / §4 item 3. Pick whichever the operator approved in P1-10/P1-12; do not add a fresh hardcode.

Android-side coordination (NOT this ticket's Rust scope, track as a dependency): Android must expose the group SSID/passphrase (`WifiP2pGroup.getNetworkName()`/`getPassphrase()`) for the credential export. File/link an Android coordination task; do not modify generated UniFFI files.

## Blast Radius

`cli/src/` (new join + credential-ingress + dial wiring), `core/src/transport/wifi_direct.rs` (only if `GroupInfo.port` is approved), `core/src/mobile_bridge.rs:1398` (must already be de-hardcoded by P1-13 — verify, do not re-hardcode). Touches the live transport dial path -> behavioral. `desktop_bridge/` unchanged (stays Linux-BLE-only). No change to Android generated bindings.

## Adversarial Review Requirement

**[AUDIT-GATE].** Touches `core/src/transport/` and the CLI dial path. Mandatory `crypto-security-auditor` pass before done, `release-gatekeeper` before merge, per `.claude/rules/security.md` and plan §1.1. Not test-only — no skip. Auditor should specifically probe: does joining an attacker-controlled `DIRECT-*` SSID + dialing its IP expose the node to anything the libp2p Noise handshake doesn't already authenticate? (The GO IP link is untrusted transport; identity auth must still gate the session.)

## Files to Touch

- `cli/src/main.rs` — credential-ingress command + Wi-Fi join step + reuse dial loop (~1462-1494).
- `cli/src/config.rs` / a new small CLI module — parse/hold the group credential struct.
- `core/src/mobile_bridge.rs:1398` — verify P1-13 de-hardcoded it; consume negotiated port. Do NOT re-hardcode.
- `core/src/transport/wifi_direct.rs:49` — `GroupInfo` gains `port` ONLY if approved in P1-10/P1-12; otherwise untouched (use port-ladder fallback).
- (coordination, not this ticket) `android/.../transport/WifiDirectTransport.kt`, `android/.../service/AndroidPlatformBridge.kt` — expose SSID/passphrase for export.

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo test --workspace --no-run
```
Device (the cell's real exit test, design note Section 6):
- Android creates group (GO); read SSID/passphrase; export credentials (QR/string) to Windows CLI.
- Windows joins the SSID (manual or `netsh`), gets a GO-subnet IP, TCP-dials `go_ip:<negotiated port>`.
- Confirm E2E message BOTH directions (phone->Windows and Windows->phone) over the Direct group link, cold-start included, twice. Evidence to ledger + matrix 2.6.

## Acceptance Tests

1. CLI accepts a `{ssid, passphrase, go_ip, port}` credential (paste and/or QR) and parses it correctly. (unit)
2. Given a reachable GO IP, the CLI dials `go_ip:port` using the negotiated port, NOT `9001`. (unit/integration on the dial-target construction)
3. Port fallback: if no `GroupInfo.port`, the ladder tries 443/80/8080/9090 against `go_ip`. (unit)
4. Device: message lands both directions over the Direct group link (design note Section 6). ([DEVICE])

## Do NOT

- Do NOT start before the [HUMAN] BUILD decision AND the hardware pre-check pass AND P1-13. If the pre-check failed, the correct output is the waiver (Android <-> Android [BLOCKED-HW] in matrix 2.6), not code.
- Do NOT add a second hardcoded port for the Windows dial — consume the negotiated/actual port from P1-13 (ticket's original "Do NOT").
- Do NOT introduce the `windows`-crate native Wi-Fi join or any `Windows.Devices.WiFiDirect` P2P module in this slice — that is a stack-addition escalation requiring separate operator sign-off (design note §4 item 2). Manual/`netsh` join only for the first slice.
- Do NOT add a `peer_exchange` wire message for credentials — QR/manual ingress only; the wire path is P1-12's audit-gated decision.
- Do NOT claim the matrix WiFi Direct cell closable without either this implementation passing the device test OR an operator-recorded waiver.
- Mandatory `crypto-security-auditor` before done (transport path). `release-gatekeeper` before merge.
