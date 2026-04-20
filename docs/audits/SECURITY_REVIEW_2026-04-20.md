# SCMessenger Security Review (April 20, 2026)

## Scope

This review focused on static code inspection across core crypto/storage, CLI/local control surfaces, and Android runtime entry points.

## Executive Summary

I found multiple high-impact issues, including command injection in installer script generation, missing cryptographic verification in invite tokens, and insecure BLE channel usage. There are also several medium-risk hardening gaps around local trust boundaries, secret persistence, and diagnostic logging.

## Findings

### 1) High — Command Injection via `host` query parameter in installer script generation

**Where**
- `InstallParams` accepts untrusted `host` query input directly.  
- The untrusted value is interpolated into shell script content (including a URL and echoed strings) without sanitization.

**Evidence**
- `cli/src/server.rs` defines `InstallParams { host: Option<String> }` and consumes it in install handlers.  
- `handle_install_native` injects `{host}` into `URL="http://{host}/api/download/scm-linux-amd64"`.  
- `handle_install_docker` derives `hostname` from user input and injects it into script output.

**Why this matters**
Users are explicitly encouraged to run `curl ... | bash` install flows. Unsanitized interpolation means crafted query strings can produce malicious shell content and execute arbitrary commands on the victim machine.

**Fix guidance**
- Strictly validate `host` against an allowlist format (`[a-zA-Z0-9.-]+(:[0-9]{1,5})?`) before interpolation.
- Reject or default when invalid.
- Prefer emitting structured data and a local installer binary over `curl | bash` pipelines.

---

### 2) High — Invite token validation does not verify cryptographic signatures

**Where**
- `InviteToken::is_valid()` only checks expiration and whether signature bytes are non-empty.

**Evidence**
- `core/src/relay/invite.rs` uses `now < expires_at && !signature.is_empty()` in `is_valid`.
- Tests assert that any non-empty signature is "valid" (`with_signature(vec![1,2,3])`).

**Why this matters**
An attacker can forge invite tokens by attaching arbitrary bytes as signature and bypass trust gating if this check is used in admission logic.

**Fix guidance**
- Implement explicit Ed25519 verification over canonical signable bytes from `get_signable_data()` using `inviter_public_key`.
- Rename current method to `is_structurally_valid()` if needed, and add a mandatory `verify_signature()` path.

---

### 3) High — BLE L2CAP transport uses insecure channels

**Where**
- Android BLE L2CAP manager opens insecure server and client L2CAP channels.

**Evidence**
- `listenUsingInsecureL2capChannel()` for incoming.
- `createInsecureL2capChannel(psm)` for outgoing.

**Why this matters**
"Insecure" BLE channels weaken peer authentication at the transport layer and increase MITM/impersonation exposure (especially relevant pre-session bootstrap).

**Fix guidance**
- Use secure L2CAP channel APIs where available.
- Enforce authenticated pairing/bond requirements before accepting sensitive traffic.
- Treat unauthenticated L2CAP as discovery-only, never trust-carrying.

---

### 4) Medium — Backup key derivation uses deterministic salt derived from passphrase

**Where**
- Backup encryption derives PBKDF2 salt as `blake3(passphrase)`.

**Evidence**
- `core/src/crypto/backup.rs` comment and code derive salt from passphrase bytes.

**Why this matters**
A deterministic passphrase-derived salt defeats per-backup uniqueness and improves attacker economics for large offline cracking campaigns (no per-record salt diversity).

**Fix guidance**
- Generate a random per-backup salt (16–32 bytes) and store it with nonce+ciphertext.
- Keep PBKDF2 iteration count high (or consider Argon2id).

---

### 5) Medium — Ratchet secrets are serialized as plaintext hex and persisted in general backend

**Where**
- Session serialization stores DH/root/chain secret material as hex strings.
- Sessions are saved into backend key `ratchet_sessions_v1`.

**Evidence**
- `SerializableRatchetSession` includes `our_dh_secret_hex`, `root_key_hex`, `chain_key_hex`, optional `identity_secret_hex`.
- `save()` writes raw JSON to `StorageBackend`.
- `SledStorage` has no at-rest encryption configuration.

**Why this matters**
Local filesystem compromise yields ratchet state compromise, affecting confidentiality of future/pending messages and session integrity.

**Fix guidance**
- Encrypt serialized session blobs before persistence using OS keystore-bound keys.
- Add explicit key rotation and secure wipe strategy.

---

### 6) Medium — Local control API has no authentication and includes process-shutdown endpoint

**Where**
- Local HTTP control API (`127.0.0.1:9876`) processes privileged operations without auth.
- `/api/shutdown` exits process.

**Evidence**
- API routes include send/history/contacts/diagnostics and `/api/shutdown` calling `std::process::exit(0)`.

**Why this matters**
Any local process in user context can command the node (DoS, privacy impact, message abuse). Loopback-only is not a complete trust boundary against local malware or untrusted local apps.

**Fix guidance**
- Require unguessable bearer token or Unix-domain socket permissions.
- Add CSRF-style nonce / origin-like protections for local browser clients.
- Consider disabling shutdown endpoint in production builds.

---

### 7) Medium — Exported BootReceiver increases abuse surface for forced service starts

**Where**
- Android receiver is exported and can trigger foreground service startup when actions match boot intents.

**Evidence**
- Manifest marks `BootReceiver` as `android:exported="true"`.
- Receiver checks action strings and starts service based on preference.

**Why this matters**
Third-party apps may spoof broadcasts (platform/version dependent behavior) to trigger repeated starts and battery/resource abuse.

**Fix guidance**
- Prefer `exported="false"` if possible with system delivery model.
- Validate sender/package where feasible.
- Add rate-limits / debounce on service starts.

---

### 8) Medium — Diagnostics logging includes sensitive identifiers and persists to disk

**Where**
- Chat screen logs conversation IDs, nicknames, display names.
- File logger persists logs in app files and rotates archives.

**Evidence**
- `Timber.d("CHAT_SCREEN: conversationId=..., normalizedPeerId=..., displayName=..., localNick=..., fedNick=...")`.
- `FileLoggingTree` writes raw log lines to `mesh_diagnostics.log` and rotates history files.

**Why this matters**
Identifiers and social-graph metadata can be exfiltrated from diagnostics bundles or on-device compromise; logs can outlive user expectations.

**Fix guidance**
- Redact peer IDs/nicknames in production logs.
- Gate detailed logs behind debug build flags.
- Encrypt diagnostics artifacts or require explicit one-time export consent with scrub pass.

## Priority Remediation Plan

1. **Immediate (P0)**: Fix installer command injection and invite signature verification.
2. **Short-term (P1)**: Replace insecure BLE L2CAP usage; harden local API auth.
3. **Near-term (P1/P2)**: Move to random KDF salt + encrypted ratchet persistence.
4. **Ongoing (P2)**: Reduce receiver abuse surface and scrub diagnostic logging.
