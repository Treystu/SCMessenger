# SCMessenger: Remaining Work Tracking

This document identifies all known TODOs, FIXMEs, placeholders, and incomplete implementations across the SCMessenger repository.

## 🛠️ High-Priority Implementation Gaps

### Rust Core (`core/`)

1.  **Relay Protocol Enforcement**
    - **File**: `core/src/mobile_bridge.rs:433`
    - **Sentiment**: Incomplete
    - **Description**: `set_relay_budget` is implemented as a logging-only function. The actual logic to block or throttle messages based on the hourly budget is not yet wired to the relay protocol.

2.  **Stateless Device Engine**
    - **File**: `core/src/mobile_bridge.rs:427`
    - **Sentiment**: Placeholder
    - **Description**: `update_device_state` (battery, motion, network) currently only logs the update. The auto-adjustment engine needs to be made stateful or integrated to react to these changes (e.g., slowing down scans when battery is low).

3.  **Cryptographic Binding for Sender ID**
    - **Source**: `PRODUCTION_READINESS_AUDIT.md` (Referencing `AUDIT_DRIFTNET.md:364`)
    - **Sentiment**: Security Gap
    - **Description**: "The sender_public_key is NOT cryptographically bound". This implies a potential impersonation risk if not addressed.

4.  **Cover Traffic (Dummy Messages)**
    - **Source**: `SOVEREIGN_MESH_PLAN.md:545`
    - **Sentiment**: Planned/Missing
    - **Description**: Integration of `core/src/privacy/cover.rs` for generating dummy traffic to obfuscate communication patterns.

### WASM / Web Transport (`wasm/`)

1.  **WebRTC Implementation Gaps**
    - **File**: `wasm/src/transport.rs:193, 368-378`
    - **Sentiment**: Significant TODOs
    - **Description**: `set_remote_answer()` and ICE candidate gathering logic are currently body-less "TODO" prescriptions. WASM transport is not yet functional for WebRTC.

2.  **WebSocket Handle Safety**
    - **File**: `wasm/src/transport.rs:305`
    - **Sentiment**: Missing logic
    - **Description**: Return error if WebSocket handle is missing despite `Connected` state.

---

## ⚙️ Maintenance & Refactoring TODOs

### iOS Project (`iOS/`)

1.  **Multipeer Connectivity Stability**
    - **Status**: Skeleton implemented in `MultipeerTransport.swift`.
    - **Remaining**: Verify session reliable/unreliable settings and ensure robust reconnection logic for WiFi-Direct equivalents.

2.  **Generated Code Efficiency**
    - **File**: `iOS/SCMessenger/SCMessenger/Generated/api.swift:53`
    - **Description**: `// TODO: This copies the buffer. Can we read directly from a pointer?` (Performance optimization).

### UI & User Experience Placeholders

1.  **Privacy Features Placeholder**
    - **File**: `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift:355`
    - **Description**: "Future Privacy Features (mirrors Android placeholders)". Includes toggles or settings that are not yet wired to core privacy modules.

2.  **Onboarding Identity Fail-safe**
    - **File**: `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift:49`
    - **Description**: Resetting onboarding state if identity is missing after start. Needs better UX flow than just a log print.

---

## 📋 Tracking Summary

| Component | Status                           | Priority | Category          |
| :-------- | :------------------------------- | :------- | :---------------- |
| Core      | `Relay Budget Enforcement`       | High     | Security/Protocol |
| Core      | `Stateful Device Profile Engine` | Medium   | Optimization      |
| Core      | `Cover Traffic Generation`       | Low      | Privacy           |
| iOS       | `Multipeer Reliability`          | Medium   | Transport         |
| iOS       | `Privacy UI Integration`         | Low      | UI/UX             |
| Android   | `Test Runner/Wrapper`            | Medium   | CI/CD             |
| WASM      | `WebRTC Handshake`               | High     | Transport         |

---

## 🔍 Audit Methodology

This list was compiled by auditing:

- Source code comments (`TODO`, `FIXME`, `HACK`).
- Function stubs with `tracing::info!` placeholders.
- Internal audit documents (`PRODUCTION_READINESS_AUDIT.md`, `AUDIT_SUMMARY_QUICK_REFERENCE.md`).
- Build logs indicating missing imports or architectural placeholders.
- **Corrected Audit Script**: `scripts/repo_audit.sh` was created to perform this search efficiently using `find` and `grep` while avoiding massive build directories.
