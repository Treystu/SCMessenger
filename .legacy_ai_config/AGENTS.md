# SCMessenger — Agentic Swarm Protocol & Architecture Rules
# Status: Active | Last updated: 2026-04-15
# Description: Single Source of Truth for Swarm Assignments and Domain Knowledge.

## ⚠️ MANDATORY: Log Extraction Standard
**All AI agents MUST read and follow:** `LOG_EXTRACTION_STANDARD.md`
- **iOS:** Use `ios_extractor.py` (mandatory)
- **Android:** Use `adb_extractor.py` (mandatory)
- **❌ Do NOT** create ad-hoc log extraction commands or ask users to manually run adb/idevicesyslog.

---

## 1. AGENTIC SWARM PROTOCOL — MODEL ASSIGNMENTS
Each role maps to an exact Ollama Cloud Pro model. NEVER substitute models without owner approval.

| Role | Ollama Cloud Model | Responsibilities |
|---|---|---|
| **Architect** | `kimi-k2.5:cloud`, `kimi-k2-thinking:cloud` | System design, planning, architecture decisions, PHILOSOPHY_CANON enforcement. |
| **Backend Coder** | `glm-5.1:cloud`, `qwen3.5:397b:cloud` | Massive refactors, Rust implementation, Drift/Routing/Privacy wiring, core API surface. |
| **Mobile Executer** | `glm-5.1:cloud`, `qwen3.5:397b:cloud` | Android Kotlin/Compose, iOS SwiftUI, UniFFI binding regeneration, BLE/Multipeer. |
| **Security Auditor** | `deepseek-v3.2:cloud` | Crypto audit, protocol review, identity model verification, envelope validation. |
| **Fast Executer** | `gemma4:31b:cloud`, `minimax-m2.7:cloud` | CLI commands, quick tests, lint fixes, doc sync checks, minor edits <50 LOC. |

---

## 2. CORE ARCHITECTURAL CONSTRAINTS

**2.1 DRIFTNET MESH**
- Every node IS the network. No third-party relays, no external servers.
- Relay = Messaging. Non-negotiable coupling.
- Internet is a transport, not a dependency (BLE, WiFi Direct, physical proximity are equal).

**2.2 IDENTITY UNIFICATION**
- `public_key_hex` (Ed25519, 64 hex chars) is the canonical identity.
- `identity_id` (Blake3 hash) and `libp2p_peer_id` are DERIVED METADATA. Never use as primary keys.
- Identity keys NEVER leave the device.

**2.3 RUST CORE DOMINANCE**
- ALL core protocol/security logic lives in Rust core (`core/src/`).
- Platform adapters (Kotlin/Swift/JS) are thin UI layers calling Rust via UniFFI.
- `IronCore` (`core/src/lib.rs`) is THE public API surface.

**2.4 PLATFORM PARITY**
- Critical-path behavior MUST be identical across Android, iOS, and Web/WASM.

---

## 3. CRYPTOGRAPHY — NON-NEGOTIABLE
*NEVER substitute algorithms without Security Auditor and owner approval.*

| Layer | Algorithm | Notes |
|---|---|---|
| **Identity signing** | Ed25519 | Keys never leave device. |
| **Identity hash** | Blake3(ed25519_pubkey) | This is `identity_id`. |
| **Key exchange** | X25519 ECDH | Ephemeral per-message. |
| **KDF** | Blake3 derive_key | |
| **Encryption** | XChaCha20-Poly1305 | 24-byte nonce, authenticated. |
| **Sender auth** | AAD + Ed25519 sig | Envelope signature binding. |

---

## 4. MANDATORY BUILD VERIFICATION
If you push code updates or patches, you MUST run the appropriate compiler/builder to prove that the edits compile successfully before concluding the conversation.

* **Rust Core:** `cargo test --workspace --lib` and `cargo clippy --workspace --lib --bins --examples -- -D warnings`
* **Android:** `cd android && ./gradlew assembleDebug`
* **WASM:** `cd wasm && cargo build --target wasm32-unknown-unknown`
* **iOS:** *(Bypassed on Windows host as of April 2026. Keep architecture parity, skip compilation).*

---

## 5. RUST CODE CONVENTIONS
* **Async:** Use `tokio` runtime.
* **Errors:** Use `thiserror` (library) and `anyhow` (binary). Propagate with `?`. NEVER panic or `unwrap()` in library code.
* **State:** `IronCore` uses `Arc<RwLock<_>>`. Use `parking_lot::RwLock`.
* **UniFFI:** All public API functions must be in `core/src/api.udl`.

---

## 6. FILE STORAGE RULES
* **NEVER** use system `/tmp`, `/var/tmp`, `/dev/shm`.
* **ALWAYS** use repo-local `tmp/` directory.
    * Session files: `tmp/session_logs/YYYYMMDD_HHMM/`
    * Debug logs: `tmp/work_files/debug_logs/`

---

## 7. PROHIBITED ACTIONS
- Decouple relaying from messaging.
- Introduce Nostr relays or external centralized servers.
- Use `unwrap()` in library code.
- Write docs longer than the code they document.
- Use time-based estimates.