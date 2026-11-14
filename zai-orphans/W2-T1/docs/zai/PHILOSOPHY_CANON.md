# SCMessenger Philosophy Canon — Completion Edition (v2)

Status: Active for the zai track. Supersedes the 2026-02-24 draft
(byte-identical copy: `docs/zai/reference/PHILOSOPHY_CANON_2026-02-24.md`)
everywhere that draft has drifted from repository reality. Where this canon
and the old draft conflict, this file wins; where they agree, the agreement
is deliberate.

This edition exists so the v1.0.0 completion effort is judged against what
the repo actually is (mined 2026-08-16 from README, docs/, the parity
audit, known-limitations register, and the governance experiment's lessons),
not what a February draft imagined.

## Mission and product intent

- Build a sovereign, end-to-end encrypted mesh messenger: users own identity
  and communication with no centralized account dependency and no server to
  trust — messages move over whatever path exists (BLE, Wi-Fi, LAN, internet
  relay) and transports race.
- Deliver one dependable product across the platform tiers below with
  aligned critical-path behavior.
- Support global organic adoption through community-operated infrastructure.
- **Completion intent (zai track):** v1.0.0 means the wiring is finished,
  the duplicates are collapsed, every decision has one authority, and the
  honesty of the README ("intentions, not guarantees" until external audit)
  extends to every gate and metric we publish.

## Platform tiers (updated from the tri-platform draft)

| Tier | Platforms | Parity bar |
|---|---|---|
| First-class | Android, iOS, Web (wasm) | PHIL-006/010: identical critical-path identity, send/receive, history, settings, relay semantics. Non-negotiable for GA. |
| Expanding | KMP desktop (WS-D), CLI (Windows/Linux/macOS) | Parity on identity/crypto/history via the shared Rust core from day one; UI parity follows platform maturity. BLE-peripheral and WiFi-Direct waivers live in `docs/V1_KNOWN_LIMITATIONS.md` — the canonical waiver register — never in ambiguity. |

**Known canon-vs-repo tension, stated not hidden:** the iOS/Android parity
audit (2026-07-27) documents iOS gaps (blocked peers, message requests,
delivery-state surfaces, diagnostics, test coverage). PHIL-006 makes
first-class parity non-waivable at GA. Therefore the zai track's v1.0.0
definition of done includes closing first-class parity or explicitly
re-classifying the gap through the limitations register with an owner and
trigger — "leave ambiguous" is the one option the canon forbids.

## User promises and trust boundaries

- No phone number, email, or centralized account for core messaging identity.
- Cryptographic authority lives in the Rust core; platform adapters never
  redefine protocol guarantees.
- First-run consent explains security and privacy boundaries before first
  use (user-facing consent — distinct from lane-participation consent, which
  is an org concern, not a product promise).
- The README's threat model is the contract: content protected end to end;
  metadata made expensive, not impossible; nothing protects a compromised
  or unlocked device.

## Cryptography posture (updated 2026-08: hybrid PQC is the baseline)

| Purpose | Primitive |
|---|---|
| Key exchange | X25519 + ML-KEM-768 (hybrid classical + post-quantum) |
| Signatures | Ed25519 + ML-DSA-65 |
| Message encryption | ChaCha20-Poly1305 |
| Forward secrecy | Double Ratchet |
| Derivation/hash | Blake3 |

The February draft's pre-PQC table is historical. PQC enforcement must be
uniform on every path before 1.0.0 (PQC-08 legacy retirement is
last-but-one for exactly this reason); "real but not uniformly enforced" is
a v0.x condition, not a GA condition.

## Engineering canon (new in v2 — paid for by the 2026 experiment)

1. **One authority per decision.** A concept with N decision-makers is a
   defect even when all N currently agree (see: escalation's three rival
   authorities). Executors execute; authorities decide; inputs feed.
2. **Two-copies-one-fixed is forbidden.** Any concept implemented at N sites
   must either be collapsed or carry a named, tested contract binding all
   copies ("verified on BOTH call paths" is the closure bar, not a
   flourish). This caused three consecutive adversarial BLOCKs.
3. **Fail closed, print provenance.** Gates and tooling derive from the
   system of record, announce degradation loudly, and tripwire at known
   caps. Silent truncation is a defect class, not a style choice.
4. **Evidence over claims.** [V-RUN]/[V-READ] classes; bare "PASSED" is
   banned; a delegated verification is a claim until the Integrator re-runs
   it. Reports quote terminal output.
5. **Honesty ordering for releases:** claims that outrun evidence are the
   canonical failure (v0.2.1's five-minute rollback; v0.3.5's ghost
   release). Tags are cut deliberately from green gates.
6. **Bounded state everywhere:** storage retention (carried from v1),
   log emission (the 2.24M-line BLE accept-spin), disk (shared target
   discipline), dispatch budgets (90-minute floor), branch life (<48h).

## UX and interaction principles

- Relay semantics stay user-understandable and identical everywhere: relay
  ON enables relay messaging; OFF blocks inbound and outbound relay
  messaging while local history remains readable offline.
- Mental model and critical controls aligned across first-class platforms.
- English-only for alpha, i18n path preserved (post-alpha expansion).

## Reliability posture

- Durable store-and-forward: messages are not dropped under transient
  connectivity loss; delivery/reconnect/availability converge to 100% over
  a sufficient horizon (eventual-consistency targets, measurable gates).
- The app stays available during active sessions when open/relaying.
- CI-backed parity checks and measurable gates are readiness criteria — and
  the gates themselves are canon (see engineering canon 3/4).

## Explicit anti-goals

- No region-gated rollout sequencing.
- No abuse-control/compliance mapping as alpha-blocking gates (post-alpha
  hardening; tracked, not forgotten).
- No unbounded local storage growth.
- No platform-specific protocol drift.
- No governance sprawl: one charter, one backlog, one canon, one lessons
  register (org lesson; the 1,695-file coordination corpus was its own
  failure surface).
- No metric without a working measurement pipeline behind it (the burndown
  lesson: a scoreboard counting ghosts drives ghost work).

## Prioritized tradeoff matrix (unchanged — it held)

1. User trust and security boundaries
2. Reliable delivery and durability
3. Cross-platform interoperability and parity consistency
4. Performance optimization
5. Delivery velocity

## Enforceable rules

Revised from v1; IDs preserved where intent is unchanged. Verification
column names the zai org mechanism that enforces it.

| Rule ID | Rule text | Criticality | Verification (zai mechanism) |
|---|---|---|---|
| PHIL-001 | `public_key_hex` is the canonical persisted/exchanged identity across all platforms; `identity_id`/`libp2p_peer_id` are derived metadata. | non-negotiable | interop tests; Validator review on identity-touching diffs |
| PHIL-002 | Relay ON/OFF semantics identical across clients (OFF = no relay messaging in either direction; local history readable offline). | non-negotiable | parity tests; UX flow validation |
| PHIL-003 | Protocol and cryptographic authority stays in the Rust core; adapters never redefine crypto behavior. | non-negotiable | merge-blocked-dir gate + Validator verdict |
| PHIL-004 | First-run consent gate communicates security/privacy boundaries before first use. | non-negotiable | UI flow tests; release checklist |
| PHIL-005 | Bounded retention prevents unbounded local growth. | non-negotiable | storage policy tests; config inspection |
| PHIL-006 | First-class parity (Android, iOS, Web) on critical-path identity, send/receive, history, settings before GA; gaps close or re-classify through the limitations register with owner+trigger — never linger ambiguous. | non-negotiable | parity audit refresh at GA; limitations register review |
| PHIL-007 | Global organic rollout with community-operated infrastructure. | negotiable | rollout docs; config behavior |
| PHIL-008 | Alpha is English-only with i18n scaffolding preserved. | negotiable | localization config; backlog check |
| PHIL-009 | Anti-abuse controls non-blocking in alpha, required before beta. | non-negotiable | release checklist; gating tests |
| PHIL-010 | No temporary lead platform for critical UX behavior. | non-negotiable | tri-platform UX parity checklist; release sign-off |
| PHIL-011 | PQC hybrid is enforced uniformly on every message path before 1.0.0; legacy suites retire last-but-one (PQC-08) with a cross-version matrix (PQC-13). | non-negotiable | verification suite; PQC-13 matrix |
| PHIL-012 | One decision-maker per decision; executors and inputs never act as rival authorities. | non-negotiable | architecture review in chunk briefs; COPIES report line |
| PHIL-013 | No duplicated implementation of a concept without a tested contract binding every copy; consolidation plan tracked until closed. | non-negotiable | unification plan burndown; COPIES report line |
| PHIL-014 | Gates fail closed, print provenance, tripwire at caps; no piped/silent-truncated verification anywhere. | non-negotiable | AGENTS.md R-Z1/R-Z2; scripts/gate.sh |
| PHIL-015 | Release claims never outrun evidence: tags cut from green gates; known limitations recorded in the register, not in memory. | non-negotiable | exit-criteria checklist; limitations register |
| PHIL-016 | Scoreboards count the real tree: any metric regenerates from committed inputs, with dropped-ghost counts printed. | non-negotiable | measurement pipeline (W2-M); gate 5 of verify_incremental_gate.py |

## Exception handling

An exception to a non-negotiable rule requires all of: rationale tied to a
higher canon priority, a time-bound window, a named owner and rollback
plan, and operator approval — recorded in `docs/V1_KNOWN_LIMITATIONS.md`
(product waivers) or the chunk file (process waivers). The enforcement
rubric (`skills/philosophy-enforcer/references/enforcement-rubric.md`,
kept) defines PASS/CONDITIONAL/FAIL and the verdict logic; the Validator
role applies it to every merge-blocked diff and every release gate.

## Canon alignment with the zai exit criteria

Charter exit criteria 1–6 map to canon rules: (1)↔PHIL-016/001,
(2)↔PHIL-013, (3)↔PHIL-012, (4)↔PHIL-011 + reliability posture,
(5)↔PHIL-006/010, (6)↔PHIL-015. A charter criterion cannot be closed in a
way that fails its canon rule, and vice versa a canon-blocking gap is an
open charter criterion by construction.
