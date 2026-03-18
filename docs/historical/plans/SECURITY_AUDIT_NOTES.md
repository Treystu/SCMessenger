> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# Security Audit Notes

**Date:** 2026-02-21
**Branch:** `claude/swarm-optimizer-setup-R5UCu`
**Tool:** `cargo-audit v0.22.1` (926 advisories loaded)
**Scope:** 498 crate dependencies in Cargo.lock

---

## [Current] Section Action Outcome (2026-02-23)

- `move`: active security posture and enforcement policy belong in `SECURITY.md` and `docs/CURRENT_STATE.md`.
- `move`: unresolved remediation tasks belong in `REMAINING_WORK_TRACKING.md`.
- `rewrite`: dependency advisory snapshots here require fresh rerun before operational use.
- `keep`: retain this audit as a historical evidence snapshot.

## [Needs Revalidation] Summary

- 1 vulnerability (error-level)
- 6 warnings (unmaintained / unsound)
- `cargo-deny` not installed; advisories check skipped

---

## [Needs Revalidation] Vulnerability (cargo audit error)

### [Needs Revalidation] ring 0.16.20 — AES panic on overflow check
| Field | Value |
|-------|-------|
| Advisory | RUSTSEC-2025-0009 |
| Severity | **Vulnerability** (cargo audit exits non-zero) |
| Issue | Some AES functions may panic when overflow checking is enabled |
| Fix | Upgrade to `ring >= 0.17.12` |
| Path | `ring 0.16.20` → `rcgen 0.11.3` → `libp2p-tls 0.4.1` → `libp2p-quic 0.10.3` → `libp2p 0.53.2` |
| Notes | This is a transitive dep pulled by libp2p's QUIC/TLS stack. Not a direct SCMessenger dep. Upgrading requires libp2p to drop rcgen 0.11 (libp2p 0.53 is pinned). A newer `ring 0.17.14` **is already present** in the lock file (used by another path), so the issue is the 0.16.x line remaining alongside it. |

**Action:** Track libp2p upgrade to a version that uses `rcgen >= 0.12` (which depends on `ring 0.17`). Cannot patch independently without breaking the libp2p 0.53 pin. Separate PR when libp2p is bumped.

---

## [Needs Revalidation] Warnings (unmaintained / unsound)

### [Needs Revalidation] bincode 1.3.3 — Unmaintained
| Field | Value |
|-------|-------|
| Advisory | RUSTSEC-2025-0141 |
| Severity | Warning (unmaintained) |
| Workspace version spec | `bincode = "1.3"` in workspace `Cargo.toml` |
| Locked version | 1.3.3 (confirmed) |
| Direct dep | Yes — used in `scmessenger-core` and `scmessenger-cli` for wire format |
| Also pulled by | `uniffi_macros 0.27.3` (transitive) |
| Replacement | `bincode 2.x` (breaking API) or `postcard` |

**Action:** Migration to `bincode 2.x` or `postcard` is a **breaking wire-format change** — must be its own PR with protocol version bump. Do not mix with other changes.

---

### [Needs Revalidation] sled 0.34.7 — Transitively pulls unmaintained deps
| Field | Value |
|-------|-------|
| Locked version | 0.34.7 (confirmed, matches expected 0.34.x) |
| Workspace version spec | `sled = "0.34"` in workspace `Cargo.toml` |
| Direct dep | Yes — storage backend for inbox/outbox |
| Pulled-in issues | `fxhash 0.2.1` (RUSTSEC-2025-0057, unmaintained) and `instant 0.1.13` (RUSTSEC-2024-0384, unmaintained) via `parking_lot 0.11.2` |
| Root cause | `sled 0.34` itself is unmaintained; no 0.35+ exists |
| Replacement | `redb`, `heed`, or `fjall` |

**Action:** Replacement of `sled` is a **large storage-layer refactor** — must be its own PR. All three unmaintained warnings (`sled`, `fxhash`, `instant`) will clear together when sled is replaced.

---

### [Needs Revalidation] ring 0.16.20 — Unmaintained (separate from the vulnerability)
| Field | Value |
|-------|-------|
| Advisory | RUSTSEC-2025-0010 |
| Severity | Warning (unmaintained, ring < 0.17) |
| Notes | Same crate/path as the vulnerability above; resolved by the same libp2p upgrade |

---

### [Needs Revalidation] lru 0.12.5 — Unsound (`IterMut` stacked borrows violation)
| Field | Value |
|-------|-------|
| Advisory | RUSTSEC-2026-0002 |
| Severity | Warning (unsound) |
| Issue | `IterMut` invalidates an internal pointer, violating Stacked Borrows |
| Path | `lru 0.12.5` → `libp2p-swarm 0.44.2` / `libp2p-identify 0.44.2` → `libp2p 0.53.2` |
| Notes | Transitive dep through libp2p. SCMessenger code does not call `lru::LruCache::iter_mut()` directly. Risk is low in practice but theoretically unsound. Resolved by libp2p upgrade. |

---

### [Needs Revalidation] paste 1.0.15 — Unmaintained
| Field | Value |
|-------|-------|
| Advisory | RUSTSEC-2024-0436 |
| Severity | Warning (unmaintained) |
| Paths | `uniffi_core 0.27.3` (mobile bindings) and `netlink-packet-utils 0.5.2` (via `if-watch`, libp2p Linux networking) |
| Notes | Proc-macro-only dep with no runtime security surface. Low priority. Clears when uniffi and/or libp2p are updated. |

---

## [Needs Revalidation] Direct Dependency Version Status

| Crate | Expected | Locked | Workspace Pin | Status |
|-------|----------|--------|---------------|--------|
| `ed25519-dalek` | >= 2.1 | **2.2.0** | `version = "2.1"` | OK — pinned correctly, resolved to 2.2.0 |
| `ring` (direct) | n/a | 0.17.14 (direct path) | not a direct dep | OK — direct uses get 0.17.14; 0.16.20 only via rcgen/libp2p-quic |
| `sled` | 0.34.x | **0.34.7** | `sled = "0.34"` | Expected version; crate itself is unmaintained |
| `bincode` | 1.3.3 | **1.3.3** | `bincode = "1.3"` | Expected version; crate now flagged unmaintained |

---

## [Needs Revalidation] Action Items (priority order)

1. **[Separate PR — libp2p bump]** Upgrade libp2p beyond 0.53 once a version ships that uses `rcgen >= 0.12`. This resolves: `ring 0.16.20` vulnerability (RUSTSEC-2025-0009), `ring` unmaintained warning (RUSTSEC-2025-0010), `lru` unsound warning (RUSTSEC-2026-0002), `instant` unmaintained warning (RUSTSEC-2024-0384, partial), and `paste` unmaintained warning (partial).

2. **[Separate PR — sled replacement]** Replace `sled 0.34` with a maintained embedded DB (`redb`, `heed`, or `fjall`). This resolves: `fxhash` unmaintained (RUSTSEC-2025-0057), `instant` unmaintained (RUSTSEC-2024-0384, remainder), and sled's own maintenance status.

3. **[Separate PR — bincode replacement]** Migrate wire format from `bincode 1.3` to `bincode 2.x` or `postcard`. Requires protocol version bump and careful backward-compat handling. Resolves RUSTSEC-2025-0141.

4. **[No action needed]** `ed25519-dalek 2.2.0` — workspace already pins `>= 2.1`; running version is compliant.

5. **[Install cargo-deny]** Add `cargo-deny` to CI (`cargo install cargo-deny`) and configure `deny.toml` to enforce advisory checks on every PR. Advisories check could not be run in this audit due to missing installation.

---

## [Needs Revalidation] Notes on workspace Cargo.toml pinning

`ed25519-dalek` is already correctly pinned at `version = "2.1"` in the workspace `Cargo.toml`, preventing regression to the vulnerable 1.x line (which was previously present via `cesride`, now removed per the comment in `Cargo.toml`). No change needed here.

`bincode` and `sled` use loose `"1.3"` / `"0.34"` specs; these are acceptable since no newer major versions of these crates exist in their maintained lines — the advisory warnings are about the crates being abandoned entirely, not about a patch-level vulnerability that a version bump would fix.
