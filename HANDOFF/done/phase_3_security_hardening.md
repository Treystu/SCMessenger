# Phase 3: Security Hardening (Adversarial Review)

**Priority:** P0 (Security-critical)
**Assigned Agent:** precision-validator (deepseek-v3.2:cloud)
**Fallback:** deep-analyst (deepseek-v4-pro:cloud)
**Status:** TODO
**Depends On:** phase_1c_integration_tests

## 3A: Crypto Module Review
- [ ] Adversarial review of `core/src/crypto/` — all files
- [ ] Verify X25519 ECDH constant-time operations
- [ ] Verify XChaCha20-Poly1305 authenticated encryption
- [ ] Verify key lifecycle management
- [ ] Verify ratcheting implementation
- [ ] Run Kani proofs (`kani-proofs` feature)

## 3B: Transport & Routing Review
- [ ] Adversarial review of `core/src/transport/` — BLE, relay, QUIC
- [ ] Adversarial review of `core/src/routing/` — TTL budgets, multipath, reputation
- [ ] Verify transport race conditions
- [ ] Verify negative cache correctness

## 3C: Privacy & Abuse Review
- [ ] Adversarial review of `core/src/privacy/` — onion routing, cover traffic
- [ ] Adversarial review of `core/src/abuse/` — spam detection, auto-block
- [ ] Verify timing obfuscation

## 3D: Supply Chain Audit
- [ ] Audit `Cargo.lock` for unexpected additions
- [ ] Verify no secrets/keys in committed files (`git diff --cached`)
- [ ] Run `scripts/docs_sync_check.sh`

## Success Criteria
- All adversarial reviews produce zero CRITICAL or HIGH findings
- Kani proofs pass (if `kani-proofs` feature)

## Rules
- Use `.claude/prompts/adversarial-reviewer.md` prompt template
- Use `.claude/skills/adversarial_review.sh` skill
- All findings must have severity rating and proof-of-concept
