# SCMessenger Remaining Work Tracking

Status: Active
Last updated: 2026-07-03 (Post-Quantum Migration workstream opened)

---

## 2026-07-03 POST-QUANTUM MIGRATION WORKSTREAM (PQC-01..14)

**Status:** OPEN — task files staged in `HANDOFF/todo/`, human-approved for implementation
**Reference:** `docs/QUANTUM_READINESS_AUDIT.md` (verdict: not quantum-proof; all asymmetric crypto is Curve25519) and `HANDOFF/todo/PQC_00_MASTER_PLAN.md` (dependency graph, suite registry, global rules, standard gates)

Goal: hybrid X25519+ML-KEM-768 for all new-session confidentiality (closes harvest-now-decrypt-later), Ed25519+ML-DSA-65 dual signatures for identity operations. Symmetric layer (XChaCha20-Poly1305 / Blake3 / Argon2id) is already quantum-safe — unchanged.

| Wave | Tasks | Notes |
|------|-------|-------|
| 0 | PQC-01 (ML-KEM dep), PQC-02 (Envelope v2), PQC-03 (Identity v2 bundle) | Parallelizable; PQC-03 needs PQC-01 |
| 1 | PQC-04 (suite negotiation), PQC-05 (hybrid KEM module) | PQC-05 requires adversarial review |
| 2 | PQC-06 (hybrid session init) | Adversarial review |
| 3 | PQC-07 (PQ ratchet — Sonnet-tier only), PQC-09 (hybrid onion), PQC-10 (ML-DSA) | PQC-07 is highest-risk; auditor + gatekeeper |
| 4 | PQC-08 (legacy path retirement), PQC-11 (relay/invite dual-sig), PQC-12 (TLS PQ groups) | |
| 5 | PQC-13 (Kani/proptest/cross-version matrix), PQC-14 (docs + risk register closure) | Workstream exit gates |

Standing rules for all PQC tasks: hybrid never pure; never remove legacy decrypt/verify paths; bincode format-tag discipline for any wire/sled struct change; adversarial review for `crypto/`/`privacy/` changes per `.claude/rules/security.md`. Per-task Definition of Done includes the standard build gates and moving the task file to `HANDOFF/done/`.

---

## 2026-07-02 V1.0.0 RELEASE READINESS ASSESSMENT

**Status:** IN PROGRESS
**Reference:** `docs/release-readiness-2026-07-02.md`

Based on the latest PR merge (`cbec1f4`), the following tasks are the final remaining items for v1.0.0 perfect code:

### Human-only / Infrastructure Blockers
- **H1:** Restore GitHub Actions runners (Runners failing immediately without logs due to billing/quota issues). This blocks all CI validation.
- **H2:** Physical-device procedures (WiFi Aware/Direct, BLE tests, DTN mule test). Requires hardware.

### Completed Code & Script Fixes (Verified 2026-07-02)
- ✅ **S-Tasks (S2-S8):** All core automation and script tasks have been resolved in the codebase.
- ✅ **T-Tasks (T1-T17):** All Rust, CLI, Android, and iOS codebase bug fixes have been completed and merged.

*Note: S9 (Cross-platform workflow validation) is still pending, blocked by H1.*

---

For historical entries prior to 2026-07-02, see docs/historical/REMAINING_WORK_TRACKING_ARCHIVE_2026.md
