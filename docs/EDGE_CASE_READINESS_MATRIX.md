# SCMessenger Edge-Case Readiness Matrix (v0.2.x)

Status: Active planning artifact  
Last updated: 2026-03-02  
Baseline used: v0.2.0 execution tracking through WS8 evidence in `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

---

## 1) Scope and invariants

This matrix evaluates resilience for the current product shape:

1. Text messaging only (no large media transfer feature assumptions).
2. Every node is a relay; headless/identity-less mode must preserve relay behavior parity.
3. Messages should retry until delivery (no delivery TTL termination).
4. Internet should improve throughput/reachability but not be a hard dependency for local relay participation.

---

## 2) Current readiness summary

Strong foundations already in place:

1. Direct-first route attempts plus relay fallback are implemented.
2. Persistent outbox + relay custody store-and-forward are active.
3. Retry behavior is non-terminal for undelivered messages.
4. Receipt convergence exists to stop duplicate forward attempts.
5. Android WiFi Direct and iOS Multipeer local-path integrations are wired.
6. Headless-default startup and role parity are in place.

Known open/deferred constraints (from residual register):

1. `R-WS3-01` - live socket-enabled custody reconnect test cannot run in restricted environments.
2. `R-WS3-02` - relay custody default persistence path currently uses temp-dir.
3. `R-WS4-02` - convergence marker anti-abuse/trust policy not yet hardened.
4. `R-WS5-01` - storage-pressure snapshot can degrade to no-op on some platforms.
5. `R-WS6-01` - Android WiFi hint staleness can reduce local fast-path hit rate.
6. `R-WS7-01` - iOS Multipeer prefix-handle collisions can reduce local fast-path hit rate.
7. `R-WS8-01` - headless-to-full promotion currently re-keys network identity with a transient restart.

---

## 3) Environment and movement edge-case matrix

| Scenario | Real-world condition | Current handling | Readiness | Primary gap |
| --- | --- | --- | --- | --- |
| Dense local outage (city blackout, no internet) | Nearby peers exist but WAN is unavailable | Local transports + relay logic can continue hop-by-hop forwarding | Medium | Encounter quality and stale local route hints can reduce throughput consistency |
| Sparse offline region (rural, disaster, long gaps) | Long periods without any reachable next hop | Durable retry semantics preserve eventual delivery intent | Medium | Delivery latency can become unbounded without encounter heuristics and delay-tolerant sync scoring |
| Airplane mode / radios disabled | Device cannot use BLE/WiFi/cellular at all | Messages queue locally for later retry when radios return | Medium | No user-visible "deferred by radio-off state" diagnostics policy is standardized yet |
| In-flight WiFi (restricted egress) | Captive or filtered internet with blocked P2P ports/protocols | Fallback paths can help when at least one route remains reachable | Low-Medium | No explicit captive-portal/filtered-egress detection and adaptive policy routing layer |
| Subway/tunnel commuting | Repeated short connect/disconnect cycles | Infinite retry + custody convergence protect eventual delivery | Medium | Burst-window prioritization is not yet tuned for short-lived opportunities |
| High-speed travel (train/car border hopping) | Rapid cell tower/network changes and NAT churn | Retry/fallback recovers over time | Medium | Route-recency quality can decay under rapid topology churn |
| Carrier-grade NAT / symmetric NAT | Direct inbound often impossible | Relay custody path covers unreachable direct peers | Medium-High | Relay custody durability path should move from temp-dir to durable app data |
| Enterprise/school network filtering | Strict firewall/proxy constraints | Multi-path attempts may still succeed through allowed paths | Medium | No explicit transport-profile policy for highly restricted enterprise environments |
| IPv6-only / NAT64 transitions | Mixed v4/v6 reachability asymmetry | Core supports mixed transport addressing | Medium | Need explicit NAT64/IPv6-only validation matrix and acceptance gates |
| Satellite / high-latency links | Very high RTT, jitter, burst loss | Non-terminal retry preserves eventual behavior | Medium | Retry/backoff is not yet profile-tuned for high-latency links |
| OS background kill (iOS/Android power management) | App suspended or terminated by OS | Persistence protects message state across restarts | Medium | Wake/reconnect reliability and delegate-style wake strategy remain future work |
| Battery-critical or thermal throttling | Aggressive OS limits on scanning/transports | Existing fallbacks keep functionality when available | Medium | No explicit power-mode routing profile with measurable degradation policy |
| Disk nearly full | Storage pressure threatens queue durability | Dynamic pressure controls exist with purge prioritization | Medium | Snapshot unavailability on some platforms can disable pressure policy (`R-WS5-01`) |
| Clock skew / wrong system time | Timestamp ordering and recency quality can drift | Core dedup + persisted history reduce hard failure risk | Low-Medium | Need clock-skew-tolerant ordering/recency normalization policy |
| High churn crowd events | Many transient peers, duplicate path attempts | Receipt convergence and deterministic fallbacks reduce loops | Medium | Convergence marker trust policy needs hardening (`R-WS4-02`) |
| Recycled identity or multi-device collisions | Sender may target stale recipient instance | WS13 plan defines strict active-device pairing | Low (today) | Requires WS13 implementation (v0.2.1) for robust resolution |
| Heavily censored/hostile jurisdiction (for example North Korea-like constraints) | Internet blocked/monitored, legal and personal risk is high | Local/offline behavior can still function where radios are allowed | Low | Needs explicit "restricted environment mode" guidance, transport hardening strategy, and user safety/legal warnings |

---

## 4) Improvement backlog (documented hardening actions)

### 4.1 v0.2.0 closure-focused hardening (WS11/WS12 + release gate)

1. `EC-01` - Move relay custody default storage path from temp-dir to durable app data path (`R-WS3-02`).
2. `EC-02` - Ensure all platform adapters provide `DeviceStorageSnapshot` so pressure policy cannot silently no-op (`R-WS5-01`).
3. `EC-03` - Replace volatile local transport hints with authenticated stable alias mapping on Android/iOS (`R-WS6-01`, `R-WS7-01`).
4. `EC-04` - Harden delivery convergence marker acceptance with additional anti-abuse validation (`R-WS4-02`).
5. `EC-05` - Add socket-enabled CI/host lane for ignored live custody reconnect integration tests (`R-WS3-01`).
6. `EC-06` - Standardize sender-visible delivery states (`queued`, `pending retry`, `delivered`) to reduce ambiguity from first-pass failures (`R-WS2-01`).

### 4.2 v0.2.1 execution (WS13 stream)

7. `EC-07` - Implement strict `(identity_public_key, active_device_id)` destination pairing (`WS13` plan).
8. `EC-08` - Add signed registration/deregistration lifecycle and stale takeover/abandon rules (`WS13.3`-`WS13.5`).
9. `EC-09` - Enforce intended-device routing semantics and compatibility migration path (`WS13.2`-`WS13.6`).

### 4.3 Post-v0.2.1 global hardening track

10. `EC-10` - Add captive-portal and filtered-egress detection with transport-profile adaptation.
11. `EC-11` - Introduce high-latency profile tuning (satellite/remote links) for retry and batching behavior.
12. `EC-12` - Add censorship-resilience strategy (protocol obfuscation/port agility) with explicit legal and safety guidance.
13. `EC-13` - Implement decentralized wake/delegate strategy for suspended mobile nodes.
14. `EC-14` - Add encounter-aware delay-tolerant forwarding heuristics for sparse/offline mobility.
15. `EC-15` - Add clock-skew tolerant recency/order normalization and validation tests.
16. `EC-16` - Publish restricted-environment operator/user guidance (risk disclosures, safe-use recommendations, and failure expectations).

---

## 5) Recommended next documentation gates

1. Keep this matrix linked from `DOCUMENTATION.md` and `REMAINING_WORK_TRACKING.md` as the canonical edge-case planning artifact.
2. Require each new residual risk entry to map to at least one edge-case scenario category here.
3. At v0.2.0 release gate, explicitly classify each `EC-01` to `EC-06` as `Closed`, `Accepted`, or `Deferred`.
