# SCMessenger Unified Global App Plan

Last updated: **2026-02-23**

## 1) Goal

Deliver one **singular, dependable SCMessenger product** that works consistently for:

- regional communication (same city/country, limited connectivity)
- international communication (cross-region, cross-continent, mixed NAT/firewall conditions)
- all primary clients: **Android, iOS, Web**
- global organic adoption (no geo-targeted rollout gates)

## 2) Current Baseline (Verified)

- Full repo context review complete: **393/393 files checked**
- Triple-check artifact: `docs/TRIPLE_CHECK_REPORT.md`
- Canonical tracker pending count: **0** (`docs/DOC_PASS_TRACKER.md`)

This plan covers completion work from that baseline.

## 2.1) Owner Directives (2026-02-23)

1. Rollout model: global and organic, not region-targeted.
2. Infrastructure model: community-operated network (self-hosted and third-party relays/bootstrap nodes are both valid).
3. Legal/compliance gate: none required for alpha launch.
4. Reliability objective: app should stay available while open/relaying, and messages must not be dropped; durable store-and-forward is mandatory.
5. Device strategy: 80/20 coverage (focus on the smallest platform set that covers most users).
6. Language scope: English-only for alpha; broader i18n is a backlog track.
7. Storage policy: bounded retention so local storage cannot grow unbounded.
8. Abuse controls: not an alpha blocker (post-alpha hardening track).
9. UX policy: mandatory first-run consent gate explaining security/privacy boundaries.
10. Bootstrap governance mode: pending final product choice (tracked as an explicit decision item).

## 3) Non-Negotiable Product Invariants

1. One identity contract across all platforms:
   - canonical persisted/exchange identity: `public_key_hex`
   - `identity_id` and `libp2p_peer_id` are derived/operational metadata
2. Relay semantics are identical everywhere:
   - relay ON: inbound+outbound messaging works
   - relay OFF: inbound+outbound blocked, local history readable offline
3. Bootstrap strategy is deterministic and unified:
   - startup env/config override
   - dynamic remote bootstrap source
   - signed/static fallback list
4. Offline-first behavior is consistent:
   - local history, drafts, and queued outbox available without network
5. Cross-platform UX intent parity:
   - same mental model and critical controls on Android/iOS/Web
6. Retention is bounded by policy:
   - history/outbox storage uses enforceable limits and pruning behavior
7. First-run consent gate is mandatory:
   - users must explicitly acknowledge privacy/security model before first use

## 4) Unified System Contract

## 4.1 Core

- Rust core is the single protocol and security authority.
- UniFFI/WASM bindings expose one coherent API contract.
- No platform-specific cryptographic behavior divergence.

## 4.2 Platform Adapters

- Android/iOS/Web adapters implement platform mechanics only:
  - lifecycle
  - transport hooks
  - notifications
  - UI
- Platform adapters do not redefine protocol invariants.

## 4.3 Settings

- Converge current settings drift (`mobile_bridge`, `mobile/settings`, `platform/settings`) into one canonical schema.
- Enforce shared validation rules in core and bind once into all clients.

## 5) Global Viability Workstreams

## A. Identity and Contact Interop

- Canonicalize contact import/export payloads to `public_key_hex` first.
- Keep `identity_id` and `libp2p_peer_id` as optional metadata fields.
- Add migration and dedup logic for legacy contacts on all clients.

Exit gate:

- same contact payload round-trips Android <-> iOS <-> Web with no identity ambiguity.

## B. Relay, Routing, and Delivery Reliability

- Keep relay=messaging coupling enforced in core and client repositories.
- Add parity tests proving OFF blocks both inbound and outbound everywhere.
- Harden retry/backoff and delivery receipts under packet loss and roaming.

Exit gate:

- deterministic relay behavior under automated parity test suite on all three clients.

## C. Bootstrap and Community-Operated Connectivity

- Implement bootstrap resolution order:
  - env/startup overrides
  - remote bootstrap config fetch
  - static signed fallback
- Support mixed operator models:
  - self-hosted nodes (for example Raspberry Pi/home servers)
  - third-party-hosted nodes (for example cloud relay services)
- Add failover and stale-config TTL behavior.

Exit gate:

- node startup reaches healthy peers through configured resolution order under mixed operator topologies.

## D. Web Promotion from Experimental to First-Class

- Move Web from “experimental” to parity roadmap track.
- Complete missing Web feature surface for:
  - identity export/import parity
  - relay/bootstrap controls
  - conversation/history parity
  - QR-equivalent import UX path (camera/file/manual)
- Add browser runtime CI with `wasm-pack` and integration tests.

Exit gate:

- Web passes the same critical-path parity suite as mobile for identity/send/receive/history/settings.

## E. Transport and Network Matrix (Regional + International)

- Execute field matrix:
  - LAN, carrier NAT, CGNAT, enterprise WiFi, captive portals, IPv4/IPv6 mixed
  - same-region and cross-region relay paths
- Record latency, delivery success, reconnect success, and battery impact.

Exit gate:

- pass-rate thresholds met for each matrix tier with no P0 delivery failures.

## F. Localization, Accessibility, and International UX

- Alpha language scope: English-only across Android/iOS/Web.
- Add i18n pipeline scaffolding now (string catalog extraction + locale plumbing), but defer non-English copy launch.
- Keep RTL/language expansion as a post-alpha track.
- Enforce accessibility baseline (screen readers, dynamic text, contrast).

Exit gate:

- English locale and accessibility test suites pass on all clients.

## G. Security and Abuse Resistance

- Keep E2E boundaries strict and testable.
- Add key-rotation and compromise-recovery flows.
- Keep abuse controls (relay spam/flood controls) explicitly tracked for post-alpha.
- Expand threat model and red-team style protocol tests.

Exit gate:

- alpha security acceptance checklist passed for all three clients and relay layer (without abuse-control hard gate).

## H. Compliance and Data Governance

- Define data retention defaults and user-controlled deletion behavior.
- Ensure no platform stores plaintext outside approved boundaries.
- Compliance mapping is a post-alpha expansion track, not an alpha launch gate.

Exit gate:

- retention and deletion policy is enforced and documented on all clients.

## I. Observability and Operations

- Unified telemetry model (privacy-preserving) across Android/iOS/Web.
- Error taxonomy aligned to operator actions.
- Reliability targets include active-session availability and durable eventual delivery semantics.

Exit gate:

- operational dashboards and runbooks exist for launch tiers, including queue-durability and delivery-recovery signals.

## J. Release Engineering and CI/CD

- CI gates must include:
  - Rust workspace tests
  - Android build + tests
  - iOS build + tests
  - browser/WASM runtime tests
  - parity contract tests
- Remove placeholder scripts and stale test claims.

Exit gate:

- all required gates are mandatory on mainline and release branches.

## K. Documentation and Governance

- Keep canonical chain synchronized:
  - `README.md`
  - `docs/REPO_CONTEXT.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/GLOBAL_ROLLOUT_PLAN.md`
  - this plan
- Continue section-level status tagging for mixed historical docs.

Exit gate:

- no contradictory “current state” claims across canonical docs.

## 6) Delivery Phases

## Phase 0: Contract Freeze

- lock invariants (identity, relay semantics, bootstrap resolution, offline behavior)
- freeze API/schema targets for parity implementation

## Phase 1: Core Convergence

- settings schema unification
- bootstrap resolver implementation
- identity payload normalization

## Phase 2: Tri-Platform Parity

- Android/iOS/Web parity for core user journeys and controls
- relay OFF/ON and history offline guarantees

## Phase 3: Global Network Hardening

- community-operated bootstrap/relay matrix and field matrix execution
- routing/delivery reliability tuning

## Phase 4: Operational Readiness

- observability, runbooks, incident response, and reliability readiness
- CI gate completion and release rehearsals

## Phase 5: Organic Global Rollout

1. Internal dogfood (all 3 clients)
2. Open global alpha (organic community growth)
3. Community scale-up with reliability guardrails
4. GA readiness review and release

## 7) Definition of Done (Strict)

The app is considered fully unified and launch-ready only when all are true:

1. Android, iOS, and Web pass identical critical-path parity tests.
2. Identity/contact payload interoperability is lossless across all clients.
3. Relay ON/OFF semantics are behaviorally identical everywhere.
4. Bootstrap resolution (env + dynamic + fallback) is implemented and tested.
5. Regional/international network matrix targets are met.
6. CI enforces tri-platform gates by default.
7. Retention bounds and consent gate behavior are verified on all clients.
8. Canonical docs and backlog accurately match runtime behavior.

## 8) Active Execution Source

Implementation queue and owners should be managed in:

- `REMAINING_WORK_TRACKING.md`

Status verification should continue in:

- `docs/CURRENT_STATE.md`
- `docs/TRIPLE_CHECK_REPORT.md`
- `docs/DOC_PASS_TRACKER.md`
