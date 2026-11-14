# SCMessenger v0.4.0-alpha.1 — Release Notes

**Release Date:** August 2026
**Target Builds:** Android APK (`app-release.apk`) / Desktop CLI (`scm-*`) / Web Daemon
**Previous Tag:** `v0.1.9` (March 2026)

---

## [IMPORTANT] READ THIS FIRST: What This Release Is (and Is Not)

**SCMessenger is pre-release alpha software. It is NOT yet suitable for anyone who requires privacy from a capable adversary, nation-state actor, or forensic examination.**

The most recent published release prior to this milestone was `v0.1.9` (March 2026). This `v0.4.0-alpha.1` release closes a five-month development gap, introducing major architectural overhauls across the core library, desktop clients, and Android application.

Before installing or testing this build, please review these explicit limitations:

1. **No Independent Security Audit:**
   The cryptographic primitives, Double Ratchet implementation, and transport protocols have been reviewed only by the developers and automated analysis tooling who built them. No independent, third-party security firm has audited this codebase. Treat all privacy and security properties as development intentions, not certified guarantees.

2. **Post-Quantum Cryptography is Hybrid & Partial:**
   While hybrid post-quantum key encapsulation (X25519 + ML-KEM-768) and signatures (Ed25519 + ML-DSA-65) are implemented in the Rust core, post-quantum suites are not yet uniformly enforced across every platform adapter and fallback path.

3. **Threat Model Boundaries (What It Does NOT Protect):**
   SCMessenger assumes the network is hostile and that no centralized server can be trusted. It protects message **content** end-to-end and aims to make **who is talking to whom** expensive to determine through transport racing and relayed store-and-forward custody. It does **not** protect against a compromised operating system, physical access to an unlocked device, keyboard loggers, or an adversary observing physical radio emanations in your immediate room.

4. **Alpha Quality & Battery Optimizations:**
   Background BLE connections can be throttled or terminated by aggressive OS battery optimizations on certain Android vendor skins (e.g., MIUI, OneUI). If a message does not deliver immediately, ensure the app remains open or background battery restrictions are disabled.

**Do not use this build for sensitive communications where a software defect could cause physical, legal, or financial harm.** Use it to experiment, test the decentralized mesh, verify offline transports, and report bugs.

---

## Overview

SCMessenger requires **no phone numbers, no email addresses, no central servers, and no user accounts**. Identity is rooted entirely in an on-device cryptographic key pair (Ed25519 / ML-DSA-65).

Messages move across whatever physical transport is available — Bluetooth Low Energy, local Wi-Fi / LAN, or Internet Cloud Nodes — and transports race concurrently so a message takes whichever pathway is actively functioning. When recipients are offline, intermediate nodes provide sealed store-and-forward custody without the ability to inspect payload contents or inner metadata.

---

## What's New in v0.4.0-alpha.1

### 1. Multi-Transport Mesh Racing
- **Simultaneous Path Racing:** Messages race concurrently across whatever physical pathways exist: Bluetooth Low Energy (GATT & L2CAP), local Wi-Fi / LAN (mDNS + direct RFC 1918 TCP/IP connect), and Internet Cloud Nodes.
- **Offline Direct Messaging:** Two phones within Bluetooth or local Wi-Fi range communicate directly without internet connectivity or cellular data.
- **Store-and-Forward Custody:** When recipients are offline, intermediate nodes store encrypted, sealed envelopes and forward them upon reconnection without decrypting payload contents or determining conversation metadata.

### 2. Post-Quantum Hybrid Cryptography
- **Hybrid Key Encapsulation:** Integrated classical X25519 ECDH with NIST ML-KEM-768 (Kyber) to defend against "Harvest Now, Decrypt Later" quantum adversary attacks.
- **Double Ratchet Forward Secrecy:** Continuous per-message re-keying augmented with post-quantum ratchet mixing.
- **Authenticated Identities:** 256-bit cryptographic identity IDs derived via Blake3 over Ed25519 public keys with dual-flavor address bindings.

### 3. Ledger-Based Mesh Discovery
- **No Central Directory:** Replaced legacy bootstrap relay lists with gossip-propagated ledger sharing. Nodes exchange authenticated neighbor ledgers over local discovery to form ad-hoc peer meshes dynamically.
- **Address Reflection & Candidate Ladders:** Intelligent connection candidate management that prioritizes direct local-area addresses before escalating to WAN custody.

### 4. Asynchronous Delivery Proofs & UI Cleanups
- **End-to-End Delivery Receipts:** Real receiver-side decryption confirmation that propagates back through the mesh and updates message states from Sent to Delivered.
- **Housekeeping Isolation:** Internal delivery receipts, custody acknowledgments, and routing frames are encrypted and filtered out of user conversation views.

### 5. Android Client Enhancements
- **Modern Jetpack Compose UI:** Complete interface overhaul featuring thread management, QR code contact exchange, and dark mode support.
- **Diagnostics Screen:** Built-in mesh routing inspection, peer discovery tables, transport state monitors, and local log exporters.
- **Mesh APK Sharing:** Direct peer-to-peer APK sharing enabling nearby devices to install SCMessenger without Google Play Store or internet access via native share intents and an ephemeral local HTTP server.
- **Provenance Verification:** Build provenance stamped into `BuildConfig.SCM_GIT_HASH` for transparent auditing against public repository commits.

### 6. Desktop CLI & Web Daemon
- Full-featured node daemon for Windows, macOS, and Linux with local REST API and embedded Web UI on `http://127.0.0.1:9876`.

---

## Known Issues & Limitations

- **NAT Traversal & Cloud Node Relay Dependency:** Circuit-relay NAT traversal had a critical bug fixed in this release (PR #157: restored missing transport prefixes on circuit-relay dial addresses). In addition, UPnP automatic port-mapping was intentionally removed. Consequently, two peers on different private networks without a direct route rely on intermediate cloud node custody; alpha testers behind restrictive symmetric NATs may experience connection hurdles if the relay path is impaired.
- **iOS Availability:** iOS builds currently require manual compilation via Xcode on a Mac. App Store and TestFlight distribution are planned for a future beta milestone.
- **Background BLE Throttling:** On certain Android devices, background BLE advertising and scanning may experience intermittent latency when the screen is off for extended periods.
- **Message Retry Backoff:** When both peers are offline and cloud custody is unreachable, messages back off exponentially up to a 300-second retry interval.

---

## Installation & Verification

### Android
1. Download `app-release.apk` (or candidate APKs) from the [GitHub Releases](https://github.com/Sovereign-Communication/SCMessenger/releases) page.
2. Enable "Install Unknown Apps" in your Android settings for your browser/file manager.
3. Install the APK and launch SCMessenger.
4. Verify build SHA in **Settings > Diagnostics > Build Info**.

### Desktop CLI
Standalone CLI executables are published with each release:
- Linux: `scm-linux-amd64`
- macOS (Intel): `scm-macos-amd64`
- macOS (Apple Silicon): `scm-macos-arm64`
- Windows: `scm-windows-amd64.exe`

Verify asset integrity against `SHA256SUMS.txt`:
```bash
sha256sum -c SHA256SUMS.txt
```

### Source Verification
To build from source:
```bash
git clone https://github.com/Sovereign-Communication/SCMessenger.git
cd SCMessenger
cargo build --release -p scmessenger-cli
cargo test --workspace
```

---

## Reporting Vulnerabilities & Bugs

- **Bug Reports:** Open an issue on our [GitHub Issue Tracker](https://github.com/Sovereign-Communication/SCMessenger/issues).
- **Security Vulnerabilities:** Please use [GitHub Private Vulnerability Reporting](https://github.com/Sovereign-Communication/SCMessenger/security/advisories/new) to disclose vulnerabilities responsibly.
