# Peer-Served Self-Updating / Self-Healing Capability

**Version:** 0.1 (feasibility study)  
**Date:** 2026-06-04  
**Status:** DRAFT — security & feasibility review

---

## 1. Vision

When a peer joins the SCMessenger mesh, it should be able to:

1. **Auto-discover** other peers on the local network and via relay bootstrap — with zero manual config
2. **Serve the current app binary** to other peers over the mesh transport, enabling:
   - **Self-update**: A peer with an older version can pull a newer version from a peer that already has it
   - **Self-heal**: A peer with a corrupted/missing installation can restore from a healthy peer
   - **Zero-infrastructure deployment**: No app store, no CDN, no central server required — the mesh is the distribution network

This is peer-to-peer software distribution, scoped to the SCMessenger mesh itself.

---

## 2. Architecture

### 2.1 Auto-Discovery (Zero-Config Bootstrap)

**Current state:** Static bootstrap multiaddrs in `config.json`, mDNS (broken in WSL2), manual peer ID entry.

**Proposed layers (priority order):**

| Layer | Mechanism | Scope | Latency | Trust |
|-------|-----------|-------|---------|-------|
| L1 | **mDNS/DNS-SD** | Same LAN subnet | <1s | Local network trust |
| L2 | **LAN subnet scan** | /24 subnet, TCP probe on 9001 | 1-5s | Same as L1 |
| L3 | **Relay bootstrap** | Pre-configured relay nodes (internet) | 1-10s | Relay operator trust |
| L4 | **Peer exchange (PX)** | Connected peers share their peer lists | Instant after L1-L3 | Transitively trusted |
| L5 | **Invite protocol** | Out-of-band share (QR, NFC, deeplink) | Manual | Explicit |

**Implementation for L2 (LAN scan):**
- On startup, if no peers connected within 5s, spawn a background task
- Enumerate the local subnet (from default interface gateway)
- TCP connect probe port 9001 on each /24 address (max 254, timeout 300ms each, ~8s total with 32 parallel tasks)
- For each open port, attempt noise handshake + identify protocol
- If peer responds with `/sc/1.0.0` protocol, add to peer table

### 2.2 App Binary Distribution Protocol

**New Drift frame type:** `FrameType::AppBinary` (extends existing `DriftFrame`)

```
AppBinaryOffer {
    version: SemVer,           // e.g. "0.2.1"
    target: TargetTriple,      // e.g. "aarch64-linux-android"
    sha256: [u8; 32],          // content hash
    size: u64,                 // uncompressed size
    signature: Ed25519Sig,     // signed by release key (NOT by serving peer)
    min_version: SemVer,       // minimum version that can receive this update
}

AppBinaryRequest {
    version: SemVer,
    target: TargetTriple,
    offset: u64,               // for resumable/chunked download
}

AppBinaryChunk {
    version: SemVer,
    target: TargetTriple,
    offset: u64,
    data: Vec<u8>,             // max 64KB per chunk (FRAME_MAX_PAYLOAD)
    final: bool,
}
```

**Flow:**
1. Peer A (new version) broadcasts `AppBinaryOffer` on gossipsub topic `sc-updates`
2. Peer B (old version) receives offer, checks:
   - Is `version` > my current version? → Skip if not
   - Is `target` my platform? → Skip if not  
   - Is `signature` valid against the **hardcoded release public key**? → REJECT if not
   - Does `sha256` match a known-good manifest? (optional: pinned versions)
3. Peer B opens a direct stream to Peer A via libp2p protocol `/sc/update/1`
4. Chunked transfer with CRC32 per chunk (existing Drift integrity)
5. On completion: verify full SHA-256, verify signature over the entire binary
6. Stage the binary in app-local storage (not live-replace yet)
7. Prompt user to apply update (or auto-apply if configured)

### 2.3 Self-Healing

Same protocol, triggered differently:
- **Corruption detection:** On startup, compute SHA-256 of own binary. If it doesn't match the known-good hash, mark self as "needs repair"
- **Missing binary:** If a critical shared library / resource bundle is missing (e.g., native .so on Android), request it from a peer
- **Heal flow:** Peer B sends `AppBinaryRequest` for its own version (not an upgrade, a repair). Peer A serves the matching binary if it has it.

---

## 3. Security Analysis

### 3.1 Threat Model

| Threat | Severity | Mitigation |
|--------|----------|------------|
| **Malicious binary injection** |  CRITICAL | Release key signature verification. The serving peer CANNOT forge a valid signature — they can only serve binaries signed by the project's release key. This is identical to how Android verifies APK signatures. |
| **Replay attack (old vulnerable version)** |  HIGH | Minimum version floor: peers reject offers where `version < min_supported_version`. The `min_version` field in the offer encodes the floor. Additionally, the release key signatures are versioned — a v0.2.1 binary signed by the release key is valid, but if v0.2.1 has a known CVE, the `min_version` in v0.2.2+ offers will be set to 0.2.2, preventing downgrade. |
| **Man-in-the-middle on transport** |  HIGH | Mitigated by libp2p Noise XX handshake — all streams are end-to-end encrypted and authenticated. The serving peer's libp2p peer ID is verified. An MITM would need to break the Noise channel. |
| **Denial of service (bandwidth exhaustion)** |  HIGH | Rate limits per peer on `AppBinaryRequest` (e.g., max 1 concurrent transfer, max 100MB/hour served). Existing `abuse/spam_detection` module tracks peer reputation — peers with low reputation are denied update serving. |
| **Supply chain (compromised release key)** |  CRITICAL | This would be catastrophic — same threat as any signed distribution system (Play Store, F-Droid, APT). Mitigations: (1) Multi-sig release key (2-of-3 threshold), (2) Air-gapped signing ceremony, (3) Key rotation mechanism with on-chain/in-mesh announcement. |
| **Peer fingerprinting / network mapping** |  LOW | `AppBinaryOffer` broadcasts version + platform, which reveals what software a peer runs. This is equivalent to User-Agent. Acceptable for a mesh where peer IDs are already public. |
| **Binary size amplification** |  MEDIUM | A 278MB APK served peer-to-peer uses significant bandwidth. Mitigations: (1) delta updates (bsdiff) instead of full binary, (2) transfer only after user confirmation, (3) serve from LAN peers first (avoid upstream bandwidth). |

### 3.2 Trust Architecture

```
┌─────────────────────────────────────────────┐
│         HARDWARE ROOT OF TRUST              │
│   (device secure boot, OS app verification) │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│       RELEASE SIGNING KEY (Ed25519)         │
│  - Signs every released binary              │
│  - Embedded in app at compile time          │
│  - 2-of-3 threshold sig (future)            │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│      TRANSPORT SECURITY (libp2p Noise)      │
│  - Authenticates peer identity              │
│  - Encrypts all update traffic              │
│  - Prevents MITM on LAN/WAN                │
└──────────────────┬──────────────────────────┘
                   │
┌──────────────────▼──────────────────────────┐
│      APPLICATION POLICY                      │
│  - Version floor (no downgrade)             │
│  - User confirmation before apply           │
│  - Reputation gate (spam threshold)         │
│  - Platform target match                    │
└─────────────────────────────────────────────┘
```

**Key insight:** The serving peer is **NOT** trusted. They are a dumb pipe. Trust derives solely from the **release key signature** on the binary. A compromised or malicious peer can only: (a) refuse to serve, (b) serve garbage (detected by hash/signature mismatch), or (c) serve an old version (blocked by version floor). They **cannot** inject arbitrary code.

### 3.3 Critical Security Requirements

1. **Release key MUST be embedded at compile time** — it cannot be fetched from the mesh (circular trust). This means key rotation requires at least one out-of-band update (via app store, USB, or the old key signing the new key's introduction).

2. **Signature verification MUST happen before staging** — never write unsigned bytes to a location the app launcher can reach.

3. **Version floor MUST be enforced** — the binary must contain a hardcoded `MIN_UPDATE_VERSION` that cannot be overridden by mesh config. This prevents downgrade attacks even if the attacker has a legitimately-signed old binary.

4. **Delta updates MUST be verifiable independently** — if using bsdiff/bspatch, the patched binary's final SHA-256 must match the signed manifest hash. The delta itself is not signed — the reconstructed whole is verified.

5. **Transfer MUST be rate-limited** — prevent a malicious peer from exhausting bandwidth by requesting binary chunks in a loop.

---

## 4. Implementation Phases

### Phase 1: Auto-Discovery (v0.2.2)
- Implement LAN subnet scanner in `core/src/relay/` (new file `lan_scanner.rs`)
- Add scan trigger on startup when peer count = 0
- Wire into `IronCore::start_swarm_with_config()` — scan runs in background
- **Effort:** ~2-3 days
- **Risk:** Low — purely additive discovery mechanism

### Phase 2: Update Protocol Definition (v0.2.2)
- Extend `DriftFrame` with `FrameType::AppBinaryOffer/Request/Chunk`
- Define the `/sc/update/1` libp2p protocol handler
- Implement `AppBinaryOffer` gossipsub broadcast on `sc-updates` topic
- **Effort:** ~3-5 days
- **Risk:** Medium — changes to wire format, need backward compat

### Phase 3: Binary Transfer & Verification (v0.3.0)
- Implement chunked transfer with resume (offset-based `AppBinaryRequest`)
- SHA-256 verification of received binary
- Ed25519 signature verification against hardcoded release key
- Version floor enforcement
- Stage to temp directory (app-local, not executable path)
- **Effort:** ~5-7 days
- **Risk:** Medium — crypto correctness is critical

### Phase 4: Delta Updates (v0.3.0)
- Integrate bsdiff for generating delta patches between versions
- Serving peer computes delta on-the-fly (or caches pre-computed deltas)
- Receiving peer applies bspatch, verifies reconstructed whole
- **Effort:** ~3-5 days
- **Risk:** Medium — bsdiff memory usage for large binaries

### Phase 5: Self-Heal & Auto-Apply (v0.3.1)
- Binary integrity check on startup (SHA-256 self-hash)
- Auto-request repair from connected peers if corrupted
- Platform-specific apply mechanism:
  - Android: Download APK → trigger `PackageInstaller` session (requires user confirmation per Android security model)
  - iOS: Not possible — Apple prohibits side-loading from app context (must go through App Store or TestFlight)
  - Desktop (CLI): Replace binary in-place, restart daemon
  - WASM: Service worker update pattern
- **Effort:** ~5-7 days
- **Risk:** High — platform-specific, Android `PackageInstaller` is fiddly

---

## 5. Platform Compatibility

| Platform | Self-Update | Self-Heal | Notes |
|----------|-------------|-----------|-------|
| **Android** |  (with user confirmation) |  | `PackageInstaller` API. Cannot silently replace — Android security model requires user to confirm install. Can prompt with "Update available from peer X". |
| **iOS** |  |  | **Apple prohibits installation of binaries from app context.** Must use App Store / TestFlight. Self-heal limited to non-binary resources (e.g., config, databases). |
| **CLI (Linux/Windows)** |  |  | Can replace own binary (with care for running process). Must: (1) write new binary to temp path, (2) rename over old path, (3) exec() or restart daemon. |
| **WASM/Browser** |  (service worker) |  | Standard web update pattern: service worker caches new version, activates on next page load. No special protocol needed — HTTP Cache-Control + SW lifecycle. |

**iOS is the only hard blocker.** On iOS, peer-served updates can only distribute non-code resources ( themes, language packs, config bundles). The app binary itself must go through Apple.

---

## 6. Feasibility Assessment

### Feasible 
- **Auto-discovery** (LAN scan, peer exchange, relay bootstrap) — straightforward, builds on existing transport
- **Peer-served binary transfer** — Drift protocol already has chunking, CRC32, compression
- **Signature-based trust** — Ed25519 release signing is well-understood
- **Self-heal** — binary integrity check + repair request is simple and high-value
- **Android update prompt** — `PackageInstaller` is a standard Android API

### Hard but Doable ️
- **Delta updates** — bsdiff works but is memory-hungry for large binaries (278MB APK → ~500MB peak memory for diff generation). Need streaming bsdiff or pre-computed deltas on the serving side.
- **CLI live binary replacement** — Must handle: in-use file locks (Windows), process self-replacement, atomicity. Doable with temp-file + rename.
- **Rate limiting / abuse prevention** — Need careful tuning to prevent bandwidth exhaustion while allowing legitimate updates

### Not Feasible on All Platforms 
- **iOS binary distribution** — Apple's walled garden. No workaround without jailbreak.
- **Silent auto-apply on Android** — Google requires user confirmation for non-Play-Store installs (unless using `DEVICE_OWNER` managed profile — enterprise-only)

---

## 7. Release Key Management

### Initial Setup
```bash
# Generate release signing key (Ed25519)
openssl genpkey -algorithm Ed25519 -out release_key.pem

# Extract public key for embedding
openssl pkey -in release_key.pem -pubout -out release_pub.pem

# Sign a release binary
openssl pkeyutl -sign -inkey release_key.pem -pkeyopt digest:SHA256 -in app.apk -out app.apk.sig
```

### Key Rotation (Future)
1. New key pair generated
2. Old key signs a "key rotation" message: `{new_pub_key, effective_version, signature_by_old_key}`
3. Peers running old version accept binaries signed by either old or new key during transition window
4. After `effective_version`, only new key accepted
5. This requires the old key's trust to be forwardABLE — which is why it must be embedded, not fetched

### Multi-Sig (Future)
- Require 2-of-3 signatures from different key holders (release, security, ops)
- Threshold signature scheme (e.g., FROST for EdDSA)
- Prevents single key compromise from allowing malicious update distribution

---

## 8. Recommended Next Steps

1. **Implement LAN auto-discovery first** (Phase 1) — this unblocks all mesh connectivity issues without any security complexity
2. **Define the update protocol** (Phase 2) — get the wire format right before implementing transfer
3. **Release key ceremony** — generate and embed the release public key before v0.3.0
4. **Start with full-binary transfer only** — skip delta updates (Phase 4) for v0.3.0, add later
5. **iOS: accept App Store only** — don't fight Apple. Focus peer-served updates on Android + Desktop

---

## Appendix A: Comparison with Existing Systems

| System | Distribution | Trust Model | Delta Updates | Auto-Apply |
|--------|-------------|-------------|---------------|------------|
| **SCMessenger (proposed)** | P2P mesh | Release key signature | bsdiff (future) | Android: prompt, CLI: auto |
| **F-Droid** | HTTP repo | Repo signing key | n/a | Yes (with prompt) |
| **Play Store** | Google CDN | Google signing | Google Play patch API | Yes (background) |
| **Tailscale** | HTTP CDN | Tailscale signing | n/a | Yes (on Linux) |
| **Tor Browser** | HTTP CDN + Tor | Mozilla signing | n/a | Yes (on restart) |
| **IPFS** | P2P content-addressed | CID integrity | n/a | Manual |
| **Brave Browser** | HTTP CDN | Brave signing | n/a | Yes (background) |

SCMessenger's approach is closest to F-Droid's trust model (known signing key, community-verifiable) but with a P2P distribution channel instead of a centralized repo.
