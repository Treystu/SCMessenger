# P0_ANDROID_004: Transport Identity Resolution Fix

**Priority:** P0 CRITICAL
**Platform:** Android
**Status:** REJECTED - QA Gatekeeper returned to todo
**Routing Tags:** [REQUIRES: TECH_DEBT] [REQUIRES: FINALIZATION]
**QA Review Date:** 2026-04-20

## QA Gatekeeper Findings

### COMPILATION ERRORS (FIXED by Gatekeeper)
1. `MeshRepository.kt`: Bare `Contact` type in `createEmergencyContact()` and `logIdentityResolutionDetails()` — fixed to `uniffi.api.Contact`
2. `MeshRepository.kt`: Missing `Permissions` import
3. `MeshRepository.kt`: `handleCancellationCascade()` tried to reassign read-only `coroutineContext` — replaced with cancel-only

### CRITICAL IMPLEMENTATION GAPS (Still need fresh coder)
1. **4 of 5 planned methods are DEAD CODE** — never called from any code path:
   - `createEmergencyContact()` — exists but never invoked
   - `isBootstrapRelayPeerFromKey()` — exists but never invoked
   - `validatePeerBeforeContactCreation()` — exists but never invoked
   - `logIdentityResolutionDetails()` — exists but never invoked

2. **`PeerKeyUtils.kt` is ALL STUBS**:
   - `extractPublicKeyFromPeerId()` returns `null` for libp2p peer IDs (the format the app actually uses)
   - `extractPeerIdFromPublicKey()` generates non-standard `peer_<hex>` IDs that won't match any real peer IDs
   - Comment acknowledges: "For now, return null to signal we need external processing"

3. **`resolveTransportIdentity()` only partially works** — returns non-null identity but does NOT create emergency contacts. The downstream caller is supposed to trigger contact creation but the pipeline is incomplete.

### Success Criteria Status
- ❌ `resolveTransportIdentity()` does NOT create emergency contacts for unknown peers
- ❌ Auto-contact creation pipeline broken — `createEmergencyContact()` is dead code
- ❌ `PeerKeyUtils` core methods are stubs returning null or non-standard IDs
- ❌ Peer discovery → contact creation pipeline NOT functional
- ✅ `resolveTransportIdentity()` no longer returns null (returns identity with null nickname)

## Files to Modify (for fresh coder)
1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` — Wire `createEmergencyContact()` into `resolveTransportIdentity()`, connect `validatePeerBeforeContactCreation()` to the auto-contact flow
2. `android/app/src/main/java/com/scmessenger/android/utils/PeerKeyUtils.kt` — Implement real libp2p peer ID ↔ public key conversion (SHA256 + multihash + base58)

## Key Technical Requirement
Peer ID conversion MUST implement proper libp2p identity:
- Ed25519 public key → multihash SHA-256 → base58btc encoding → `12D3KooW...` peer ID
- This is the inverse operation for extracting public keys from peer IDs
- The `peer_<hex>` fallback format is NOT acceptable for production

## Estimated Remaining LOC: ~200-250 LOC