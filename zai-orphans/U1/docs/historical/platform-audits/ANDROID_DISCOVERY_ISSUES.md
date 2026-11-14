> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

# Android Peer Discovery & Mesh Issues - Status Report

## [Needs Revalidation] Primary Issue: Peer Visibility Parity

**Symptoms:** Android client does not display connected peers (Bootstrap, iOS, etc.) while iOS client correctly shows them.

## [Needs Revalidation] Investigated Causes & Findings

### [Needs Revalidation] 1. Connection Instability

- **Log Evidence:** Android logs show frequent `Disconnected` events shortly after `Identified` or `Connected` events.
- **Root Cause:** The GCP relay logs show `Listen error: Failed to negotiate transport protocol(s)`. This suggests a handshake failure or protocol version mismatch during the `libp2p` upgrade phase $(noise/yamux)$.
- **Impact:** Peers appear momentarily and are then removed from the `NearbyPeers` list after the 30-second grace period.

### [Needs Revalidation] 2. Bootstrap Relay Filtering

- **Log Evidence:** `Ignoring bootstrap relay peer discovery event: 12D3KooWL6Ke...`
- **Logic:** `MeshRepository.isBootstrapRelayPeer` returns `true` for the GCP node. This causes the code to intentionally skip emitting `PeerEvent.Discovered`. While it still processes `onPeerIdentified`, the initial "Discovered" entry in the UI never appears for the bootstrap node itself.

### [Needs Revalidation] 3. WiFi Transport Failures

- **Log Evidence:** `WiFi P2P Discovery failed: 0`.
- **Impact:** Local peer discovery via WiFi Aware or WiFi Direct is likely non-functional on the current Android test device, forcing all discovery to happen via BLE or the Internet (GCP Relay).

### [Needs Revalidation] 4. Identity Resolution Race Conditions

- **Observation:** `onPeerIdentified` calls `resolveTransportIdentity(peerId)`. If the identity hasn't been gossip-propagated yet, this returns `null`, and `IdentityDiscovered` is skipped.
- **Parity Note:** On iOS, the `handleTransportPeerDiscovered` logic might be more aggressive in re-trying identity resolution or maintaining the "Discovered" state even without full identity.

---

## [Needs Revalidation] Technical Debt / Documented Issues

| Area          | Issue Description                                                                                                      | Priority |
| :------------ | :--------------------------------------------------------------------------------------------------------------------- | :------- |
| **Transport** | `Failed to negotiate transport protocol(s)` on Internet relay connections.                                             | High     |
| **Discovery** | Bootstrap relays are hidden from the "Nearby" list by design; user wants them visible for confirmation.                | Med      |
| **local-net** | WiFi P2P Discovery failing on specific Android versions/hardware.                                                      | Med      |
| **UI/VM**     | `ContactsViewModel` disconnect grace period (30s) may be hiding temporary connection flickers that should be "Online". | Low      |
| **BLE**       | `BleGattServer` exceptions observed during startup (potential permission or state race).                               | Low      |

## [Needs Revalidation] Next Steps (Morning Plan)

1. **Relax Bootstrap Filtering:** Allow bootstrap nodes to appear in search/nearby lists if they provide a valid identity.
2. **Protocol Negotiation Audit:** Compare `libp2p` transport configurations between Android (Kotlin Bridge) and iOS (Swift Bridge) to ensure identical multiaddr handling.
3. **Identity Gossip Verification:** Ensure `IdentityDiscovered` events are re-emitted if an identity is resolved _after_ the initial connection.
