# SCMessenger Unified Global App Plan

Last updated: **2026-02-23**

## 1) Goal

Deliver one **singular, dependable SCMessenger product** that works consistently for:

- regional communication (same city/country, limited connectivity)
- international communication (cross-region, cross-continent, mixed NAT/firewall conditions)
- all primary clients: **Android, iOS, Web**

## 2) Current Baseline (Verified)

- Full repo context review complete: **392/392 files checked**
- Triple-check artifact: `docs/TRIPLE_CHECK_REPORT.md`
- Canonical tracker pending count: **0** (`docs/DOC_PASS_TRACKER.md`)

This plan covers completion work from that baseline.

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

## C. Bootstrap and Multi-Region Connectivity

- Implement bootstrap resolution order:
  - env/startup overrides
  - remote bootstrap config fetch
  - static signed fallback
- Introduce multi-region bootstrap pools (Americas, EMEA, APAC) with health scoring.
- Add failover and stale-config TTL behavior.

Exit gate:

- node startup reaches healthy peers from any region using configured resolution order.

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

- Add i18n pipeline (string catalogs for Android/iOS/Web).
- Ship at least tier-1 languages and RTL readiness.
- Enforce accessibility baseline (screen readers, dynamic text, contrast).

Exit gate:

- locale and accessibility test suites pass on all clients.

## G. Security and Abuse Resistance

- Keep E2E boundaries strict and testable.
- Add key-rotation and compromise-recovery flows.
- Add abuse controls for relay spam/flood and malformed payloads.
- Expand threat model and red-team style protocol tests.

Exit gate:

- security acceptance checklist passed for all three clients and relay layer.

## H. Compliance and Data Governance

- Define data retention defaults and user-controlled deletion behavior.
- Ensure no platform stores plaintext outside approved boundaries.
- Prepare regional compliance map (store only required metadata).

Exit gate:

- compliance checklist signed for target regions before GA expansion.

## I. Observability and Operations

- Unified telemetry model (privacy-preserving) across Android/iOS/Web.
- Error taxonomy aligned to operator actions.
- SLOs for delivery, startup, discovery, and reconnect.

Exit gate:

- operational dashboards and runbooks exist for launch tiers.

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

- multi-region bootstrap and field matrix execution
- routing/delivery reliability tuning

## Phase 4: Operational Readiness

- observability, runbooks, incident response, compliance readiness
- CI gate completion and release rehearsals

## Phase 5: Staged Global Rollout

1. Internal dogfood (all 3 clients)
2. Regional closed beta
3. International open beta
4. General availability

## 7) Definition of Done (Strict)

The app is considered fully unified and launch-ready only when all are true:

1. Android, iOS, and Web pass identical critical-path parity tests.
2. Identity/contact payload interoperability is lossless across all clients.
3. Relay ON/OFF semantics are behaviorally identical everywhere.
4. Bootstrap resolution (env + dynamic + fallback) is implemented and tested.
5. Regional/international network matrix targets are met.
6. CI enforces tri-platform gates by default.
7. Canonical docs and backlog accurately match runtime behavior.

## 8) Active Execution Source

Implementation queue and owners should be managed in:

- `REMAINING_WORK_TRACKING.md`

Status verification should continue in:

- `docs/CURRENT_STATE.md`
- `docs/TRIPLE_CHECK_REPORT.md`
- `docs/DOC_PASS_TRACKER.md`
