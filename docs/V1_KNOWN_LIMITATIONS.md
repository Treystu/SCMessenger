# V1.0.0 Known Limitations

Status: Active. Last updated: 2026-07-17.

This document records scope waivers and known limitations for the v1.0.0
release of SCMessenger. All items below were deliberate decisions, not
oversights.

---

## Desktop BLE Peripheral Mode (T1.8 waiver)

The Windows/Linux CLI daemon does not operate as a BLE peripheral. It can
initiate BLE connections as a central to Android devices, but cannot be
discovered by other BLE centrals. This is a platform limitation: Windows BLE
peripheral role requires WinRT APIs not yet wired.

**Impact:** Windows CLI nodes are not BLE-discoverable. Android-to-Android
BLE discovery works normally.

**Workaround:** Use mDNS/LAN or TCP bootstrap for Windows CLI discovery.

**Future:** Scheduled for v1.1 (WinRT BLE peripheral lane).

---

## WiFi Direct (v1.0.0 scope waiver)

WiFi Direct (P2P group formation) between Android devices is deferred to v1.1.
Android-to-Android direct pairing uses mDNS/LAN + TCP. Windows WiFi Direct is
waived entirely (no WFD driver API wired).

**Decision date:** 2026-07-09 (operator-confirmed waiver).
**Future:** Android-to-Android WiFi Direct in v1.1. Windows WiFi Direct
post-v1.1 pending demand.

---

## GitHub Actions CI (H-01 billing gate)

CI runners are blocked pending billing resolution on the personal GitHub
account. All build verification runs locally on the Windows host. iOS XCTest
gates require a macOS runner, blocked until H-01 resolves.

**Workaround:** Local `cargo check/clippy/fmt/test` gates are run on the
Windows host before every commit.

---

## Physical Device WiFi Aware / BLE Field Verification (H-02)

The operator's primary test device (Pixel 6a) was unavailable. All v1.0.0
Android verification used the `scm_pixel_34` AVD emulator. Physical
two-device WiFi Aware and BLE sneakernet scenarios are verified by design but
not field-tested. Scheduled for the v1.0.0 farm pilot (F3 phase).

---

## AWS/B4 Cloud Relay (H-04 paused)

Cloud relay infrastructure (`infra/aws/`) is committed and ready but was not
activated. Credential injection was not completed. The relay daemon runs
correctly in the Docker farm-sim. Cloud activation deferred to operator
decision.

---

## Resolved in v1.0.0

The following items were open concerns that are resolved in v1.0.0:

- Message delivery pipeline (A1/A2 outbox flush + receipt round-trip)
- Post-Quantum Cryptography waves 0-3 (PQC-01 through PQC-07 chain)
- Receipt acknowledgment pipeline (core-side encode/decode unified)
- mDNS discovery protocol (self-loopback filter, LAN feed to MeshRepository)
- Adaptive port selection (P1-11/P1-12/P1-13 hardcode sweep)
- BLE TX path (CLI outbound, P1-16)
- DriftFrame custody relay dispatch (Fix-1, 2026-07-17)
- Outbox flush on peer identity confirmation (Site-3, 2026-07-17)
