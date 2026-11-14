# PR #139 Adversarial Security Review -- AGENTS.md Rule 8 Gate

Status: Active
Last updated: 2026-08-08
Branch: `tracking/pre-v040-tag-work` @ `6cb7033a`
Diff base: `origin/main...origin/tracking/pre-v040-tag-work`
Reviewer role: adversarial security auditor (read-only, Opus)

## Verdict summary

**BLOCK.** One CRITICAL finding (F1) reintroduces the exact
internal-network-disclosure vulnerability that the code comment three lines
above it documents as previously fixed. Three HIGH findings compound it.

---

## F1 -- CRITICAL -- disclosure -- RFC1918 gate never checks the requester

Location:
- `core/src/transport/addr_filter.rs:456-504`
- `core/src/store/ledger_entry.rs:1250-1268`
- `core/src/transport/swarm.rs:3983-3997` and `:5695-5710`

The docstring on `exchange_response_entries` claims private entries are shared
"only to a peer on the same private class as one of our own listeners
(`my_addrs`)". The implementation does not check the peer.
`is_disclosable_on_rfc1918_network(entry_addr, my_addrs)` compares the **ledger
entry's** address class against **our own** listener addresses:

```rust
fn has_matching_private_class(ip: &Ipv4Addr, my_addrs: &[String]) -> bool {
    my_addrs.iter().any(|a| {
        extract_ipv4(a).is_some_and(|my_ip| {
            rfc1918_class(&my_ip) == rfc1918_class(ip) && rfc1918_class(ip).is_some()
        })
    })
}
```

`my_addrs` is built from `swarm.listeners().chain(swarm.external_addresses())`.
The node binds `/ip4/0.0.0.0/tcp/...` (swarm.rs:2466-2471, 2489-2491), and
libp2p 0.56 + `if-watch` 3.2.2 expand an unspecified bind into one
`NewListenAddr` per interface -- so `my_addrs` always contains the node's own
LAN IP (e.g. `/ip4/192.168.1.50/tcp/9001`). Our LAN neighbours in the ledger are
`/ip4/192.168.1.x/...`. Same class => `true`, unconditionally, for every
requester. Pushing `conn.remote_addr` into `my_addrs` cannot restrict anything
-- it is one more element inside an `.any()`, so it can only ever *add* matches.

Proof (concrete):

1. Victim is a normal home node: `swarm.listeners()` yields
   `/ip4/192.168.1.50/tcp/9001`, `/ip4/127.0.0.1/tcp/9001`.
2. Victim dialed LAN neighbours via mDNS; `record_connection` is deliberately
   unfiltered (swarm.rs:4694-4706), so the ledger holds
   `/ip4/192.168.1.7/tcp/9001` -> `12D3KooW...`, `/ip4/192.168.1.22/tcp/9001`.
3. Attacker anywhere on the internet completes a Noise handshake and sends one
   `/sc/ledger-exchange/1.0.0` request (rate limit permits 1).
4. Filter: `is_disclosable_multiaddr("/ip4/192.168.1.7/tcp/9001")` = false ->
   `is_private_or_cgnat_multiadr` = true -> `is_disclosable_on_rfc1918_network`
   compares class(192.168.1.7)=2 against class(192.168.1.50)=2 -> **true**.
5. Attacker receives up to `LEDGER_EXCHANGE_MAX_RESPONSE_PEERS`
   `SharedPeerEntry { multiaddr, last_peer_id, last_seen }`
   (ledger_entry.rs:1278-1287) -- internal subnet, live host:port, and each
   neighbour's libp2p PeerId, with recency.

That payload is an internal network map plus a peer-identity-to-private-address
binding, delivered to an unauthenticated remote. For a privacy-focused messenger
this is both internal-network reconnaissance and a deanonymization primitive.
The comment at swarm.rs:3976-3982 describes this identical bug as previously
closed:

> "...this call site used to pass `Local`, which skips the RFC1918 check, so
> every LAN neighbour we had dialed -- subnet, host:port and peer id -- was
> disclosed to internet peers."

Both call sites are affected: the exchange RESPONSE (swarm.rs:3997) and the
outbound `ShareLedger` REQUEST (swarm.rs:5710).

Fix: the gate must be a predicate on the **requester's** observed address, not
on ours. Derive it from `connection_tracker.get_connection(&peer).remote_addr`
as a separate, mandatory argument; require it to be private; require it to match
the entry's subnet. Reject the path outright when the connection is relayed
(`/p2p-circuit`), since a circuit remote address does not establish LAN
adjacency.

---

## F2 -- HIGH -- disclosure -- "same network" is class-granular, not subnet-granular

Location: `core/src/transport/addr_filter.rs:497-521`

`rfc1918_class` collapses every private address into one of four buckets (10/8,
172.16/12, 192.168/16, 100.64/10). The function's own doc comment claims "same
/8, /16 **or /24** block". No /24 comparison exists. Even after F1 is fixed,
`192.168.0.0/16` covers essentially every consumer LAN, so "same class" is close
to a tautology: a peer at `192.168.99.7` on another continent, reached over a
VPN or internet relay, matches `192.168.1.x`.

Proof: `is_disclosable_on_rfc1918_network("/ip4/192.168.1.7/tcp/9001",
&["/ip4/192.168.99.7/tcp/9001"])` -> `rfc1918_class` = `Some(2)` for both ->
`true`. The unit test `rfc1918_not_disclosable_when_different_private_class`
only exercises 10.x vs 192.168.x -- the one case the coarse check happens to
catch. No test for same-class/different-subnet.

Fix: compare actual subnets (mask to /24, or better, compare against the
interface prefix length reported by the listener). Correct the doc comment.

---

## F3 -- HIGH -- disclosure -- "verified contact" discloses loopback and link-local cross-subnet

Location: `core/src/store/ledger_entry.rs:1204-1211`, `:238-258`, `:1263-1265`

`is_verified_contact` returns true for any peer with a ledger entry where
`success_count > 0` and no `/p2p-circuit`. `success_count` is incremented by
`record_connection` on every successful **dial** -- not a contact relationship,
not an accepted invite, no user action. "Verified contact" means "we once
successfully dialed this peer directly."

When that holds, the filter returns `true` for **any** entry classed private by
`is_private_or_cgnat_multiadr` -- and that classifier deliberately includes
loopback and link-local:

```rust
if ip.is_private() || ip.is_loopback() || ip.is_link_local() || cgnat { return true; }
```

So a "verified contact" receives our `/ip4/127.0.0.1/tcp/<port>` entries (which
local services we connect to, on which ports) and our `/ip4/169.254.x.y/...`
entries -- the range containing `169.254.169.254`, described in this same
module's header as "the single highest-value SSRF target in existence".
`is_disclosable_multiaddr` treats those as absolutely non-disclosable; this new
path routes around that absolute.

Proof: get us to dial the attacker once (Identify `listen_addrs` or a prior
exchange entry; our dial sweep does the rest) -> `is_verified_contact(attacker)`
= true -> next exchange returns every loopback and link-local entry in our
ledger with no class check, since `my_addrs` is not consulted on that branch.

Fix: (a) exclude loopback, link-local, unspecified, multicast and broadcast from
the contact-chaining branch unconditionally -- route them through
`is_unconditionally_routable_ipv4` before any disclosure decision. (b) Redefine
"verified contact" against the contact store (a mutual, user-accepted
relationship), not dial success in the ledger.

---

## F4 -- HIGH -- unproven, remote-supplied addresses became disclosable

Location: `core/src/store/ledger_entry.rs:1245-1248`; ingest at
`core/src/mobile_bridge.rs:1018-1042` and `:1144`

The disclosure filter was relaxed from `success_count > 0` to
`(e.success_count > 0 || e.public_key.is_some())`. `public_key` is populated by
`annotate_identities_batch` from the Identify path, where `listen_addrs` is
"whatever the remote chose to put in its Identify response"
(mobile_bridge.rs:1004-1006) and is filtered with `NetworkMode::Local`, which
deliberately permits RFC1918. Those entries have `success_count == 0` -- we never
dialed them, we have no evidence they exist.

This converts the node into an amplifier: an attacker's chosen private
host:port pairs, bound to the attacker's PeerId, are redistributed by us to
every exchange partner. Receivers ingest them (swarm.rs:3906-3930), add them to
Kademlia when `is_discoverable_multiaddr` passes -- which permits RFC1918
(swarm.rs:106-120) -- then dial them under `NetworkMode::Local`. That is a
mesh-distributed internal port-scan / SSRF oracle, the precise threat
`addr_filter` exists to prevent (module header, addr_filter.rs:363-372).

Proof: attacker advertises `/ip4/10.0.0.1/tcp/22`, `/ip4/10.0.0.2/tcp/22`, ...
in Identify. We store them with `public_key = Some(...)`, `success_count = 0`.
Every subsequent exchange where the class check passes (per F1, every exchange)
ships them onward. Dial-outcome timing on receivers is the oracle.

Fix: keep `success_count > 0` as the disclosure predicate. A public key is an
identity annotation, not evidence of reachability. If unproven entries must be
shared, give them a distinct lower-confidence wire tier that receivers do not
auto-dial.

---

## F5 -- HIGH -- T1's libp2p-peer-id claim is not implemented; transport block gate is dead and fails open

Location: `core/src/store/blocked.rs:107-116`; `core/src/transport/swarm.rs:3138`
and `:6339`

The PR claims dual-flavor blocking across "identity_id and libp2p peer id".
`resolved_identifiers` handles exactly one relation -- Ed25519 public-key-hex ->
blake3 identity_id:

```rust
pub fn resolved_identifiers(&self, peer_id: &str) -> Vec<String> {
    let mut identifiers = vec![peer_id.to_string()];
    if let Some(derived_id) = identity_id_from_public_key_hex(peer_id) { ... }
    identifiers
}
```

No libp2p PeerId handling. Meanwhile both transport-layer block gates pass a
base58 libp2p PeerId:

```rust
core_handle.is_peer_blocked(peer.to_string(), None).unwrap_or(false)
```

`"12D3KooW..."` is not 64-hex, so `resolved_identifiers` returns only itself,
and the lookup key `blocked:12D3KooW...` is never written by `block_peer`
(called with public-key hex or identity_id). The gate never fires. It also
**fails open** on storage error (`unwrap_or(false)`), contradicting the
fail-closed policy this PR asserts at every other block site.

Proof: block a peer from the UI. That peer's libp2p connection still passes the
swarm-level gate at swarm.rs:3138; the block only takes effect later at
`receive_message`, after decryption. "Neither flavor can bypass a block" does
not hold at the transport boundary.

Fix: (a) add libp2p PeerId <-> Ed25519 public key derivation to
`resolved_identifiers` (derivable in that direction -- see cli/src/main.rs:2224-2226),
or (b) withdraw the claim and change `unwrap_or(false)` to `unwrap_or(true)` at
both swarm gates.

---

## F6 -- MEDIUM -- T2's ~50% gate makes `list()` surface a bogus double-hash and drop the real row

Location: `core/src/store/blocked.rs:212-256` and `:425-457`;
`core/src/identity/keys.rs:39-48`

The commit's own comment admits the residual: a blake3 output is a valid Ed25519
curve point roughly half the time, so
`identity_id_from_public_key_hex(<an identity_id>)` returns
`Some(blake3(identity_id))` about 50% of the time. Two new paths treat
"derivable" as "is a public key":

1. `block()` writes an alias row under the double-hash `D = blake3(ID)`.
2. `list()` then drops the real row: for the `ID` row, `derived == D`,
   `D != ID`, and `D` is in the set (we just wrote it) -> `retain` returns false.

Net effect: the blocked list surfaces `D` -- a value corresponding to no public
key and no identity_id -- and hides `ID`.

Proof: `manager.block(BlockedIdentity::new(id))` where `id` is an on-curve
identity_id. `list()` returns one peer-level entry with `peer_id == blake3(id)`.
Enforcement still works (`is_blocked_resolved` checks the physically-present
`ID` row) and `unblock` recovers via the reverse scan, so this is not an ingress
bypass. The damage: the UI shows an unrecognizable identifier, and
`blocked_only_peer_ids()` (blocked.rs:486-497, built on `list()`) returns `{D}`
instead of `{ID}` -- so any consumer matching by identity_id misses the block.
History records are keyed by `canonical_peer_id` (= identity_id), so a
list-based filter over that set would fail.

The existing test `block_identity_id_writes_no_pk_alias` deliberately uses
`"7f"*32`, an off-curve value, so it never exercises this branch.

Fix: do not use curve-point validity as a public-key-vs-identity_id
discriminator. Tag the flavor explicitly in the stored row (the `pk:` / `id:`
prefixes at keys.rs:9-11 exist for exactly this), or make `list()` dedupe on an
explicit alias link rather than re-derivation.

---

## F7 -- MEDIUM -- `get_canonical_peer_id` returns a double-hash for ~50% of identity_ids, and it is a new FFI surface

Location: `core/src/iron_core.rs:1407-1411`; FFI surface added at
`scripts/ffi-snapshots/kotlin-symbols.txt` / `swift-symbols.txt`

```rust
pub fn get_canonical_peer_id(&self, peer_id: &str) -> Option<String> {
    crate::identity::keys::identity_id_from_public_key_hex(peer_id)
        .filter(|derived| derived != peer_id)
        .or(Some(peer_id.to_string()))
}
```

The doc comment says the input is a pending-request peerId -- i.e.
`message_request_key`'s output, an identity_id. For on-curve identity_ids this
returns `blake3(identity_id)`, not the input. Exported to Kotlin and Swift as
`getCanonicalPeerId` and documented as the mapping mobile clients use to match
pending requests against the blocked list. About half of all peers map to a
value that matches nothing.

Proof: `get_canonical_peer_id(id)` for any on-curve identity_id returns
`Some(blake3(id))`. Mobile compares that against a blocked list containing `id`
-> no match -> a blocked sender renders as an unblocked pending request.

Fix: return the input unchanged unless the caller explicitly asserts the input
is a public key; or take a typed/prefixed identifier. Also `.or(Some(...))`
allocates eagerly on every call -- `.or_else` is correct, though cosmetic next
to the semantic bug.

---

## F8 -- MEDIUM -- `RejectMessageRequest` silently does nothing if the message is gone

Location: `cli/src/server.rs:1543-1592`

Reject previously blocked `request_id` unconditionally. It now requires finding
a live inbox message whose `message_request_key` matches and which carries a
`sender_public_key_hex`; if none is found it returns JSON-RPC `-32002` and
**blocks nobody**.

Proof (TOCTOU): user opens the pending list at T0. Between T0 and the reject at
T1, the retention sweeper (`core/src/store/sweeper.rs`) expires the inbox
message, or a caller drains `drain_received_messages`. At T1
`peek_received_messages()` no longer contains it -> `-32002` -> no block
written. The user believes they blocked a harasser; the block store is
untouched, and the same sender's next message produces a fresh pending request.
The `filter_map(|m| m.sender_public_key_hex)` also drops any message whose
envelope key is absent -- precisely the legacy class the `message_request_key`
fallback exists to serve.

Fix: on lookup failure, fall back to blocking the canonical `request_id`
directly (the pre-change behaviour). The dual-flavor write already handles the
identity_id-only case correctly.

---

## F9 -- MEDIUM -- block store is dual-flavor; its side effects are not

Location: `core/src/iron_core.rs:1469-1508`

`unblock_peer` calls `history_manager.unhide_messages_for_peer(&peer_id)` with
the raw caller-supplied flavor. `block_and_delete_peer` calls
`history_manager.remove_conversation(peer_id)` and
`outbox.drain_for_peer(&peer_id)` the same way. History records are keyed by
`canonical_peer_id` (identity_id, set at iron_core.rs:3381). The block store now
resolves both flavors; these three side effects do not.

Proof: `block_and_delete_peer(<public key hex>)` -- now the natural flavor to
pass, since the reject handler was changed to block by public key -- marks
`is_deleted = true` under both flavors but purges **nothing**, because
`remove_conversation("<pk hex>")` finds no rows keyed by identity_id. The user
is told the conversation was deleted; every message is still on disk.
Symmetrically, `unblock_peer("<pk hex>")` removes both block rows but leaves the
messages permanently hidden.

Fix: resolve identifiers once at the `IronCore` layer and apply every side
effect across all resolved flavors, or canonicalize to identity_id before all
three calls.

---

## F10 -- MEDIUM -- one unparseable blocked row makes every unblock fail forever

Location: `core/src/store/blocked.rs:332-372`

```rust
for (_, value) in blocked_entries {
    let blocked: BlockedIdentity =
        serde_json::from_slice(&value).map_err(|_| IronCoreError::Internal)?;
```

`unblock` now depends on `identifiers_for_unblock`, which scans the full
`blocked:` prefix and hard-fails on the first row it cannot deserialize.
Previously `unblock` performed direct key removals with no scan and no parse. A
single corrupt row, partial write, or forward-incompatible schema field
permanently bricks unblocking for **every** peer. One-way door: the user can
always add blocks but never remove one.

Verified the prefixes do not collide (`"blocked:"` vs `"blocked_devs:"`,
blocked.rs:15-17), so registry rows are not caught by the blocked scan. That
specific failure mode does not apply.

Fix: `continue` on parse failure instead of `?`, logging the key.

---

## F11 -- LOW -- block-store writes performed under a READ lock

Location: `core/src/iron_core.rs:4455-4465`; `core/src/store/blocked.rs:507-545`

`IronCore::register_device_id` takes `self.blocked_manager.read()` and calls
`BlockedManager::register_device_id`, which now calls `self.block(blocked)` -- a
multi-row write. Every other write path (`block_peer` :1466, `unblock_peer`
:1474, `block_and_delete_peer` :1503) takes `.write()`. The `RwLock` no longer
serializes all writers: two concurrent `register_device_id` calls run their
check-then-act in parallel, and a concurrent reader can observe a half-written
device alias pair.

Impact is limited today (writes idempotent; a partially-written device alias
still leaves the peer blocked under the first-written flavor), but the invariant
"all block-store mutation happens under the write lock" is now false.

Fix: make `IronCore::register_device_id` take `.write()`. Note iron_core.rs:2240-2244
already uses `.write()` for the same underlying call -- the two sites are
internally inconsistent.

---

## F12 -- LOW -- non-atomic dual-flavor block write

Location: `core/src/store/blocked.rs:212-256`

`block()` writes N rows sequentially with `?`. A storage error after the primary
write leaves a half-applied block. If the primary is the identity_id flavor and
the alias write fails, `is_blocked_resolved(<pk>)` still resolves and matches.
If the primary is the public key and the alias fails,
`is_blocked_resolved(<identity_id>)` returns false -- identity_id does not
derive back to the public key -- so an identity_id-flavored check bypasses the
block. The caller does receive an `Err`, limiting this to silent-caller
scenarios (`let _ = ... register_device_id(...)` at iron_core.rs:1436, 1443,
2241, 4460 discards it).

Fix: batch the rows into a single atomic backend write, or write the identity_id
(canonical) row first so a partial failure fails safe.

---

## F13 -- LOW -- `is_disclosable_on_rfc1918_network` fails OPEN by default

Location: `core/src/transport/addr_filter.rs:456-495`

The protocol loop falls through to `true` when the multiaddr contains no `Ip4`,
`Ip6` or DNS component (`/p2p/QmX`, `/memory/...`, `/unix/...`, or an empty
multiaddr -- note `"".parse::<Multiaddr>()` is `Ok(<empty>)`, documented as a
live hazard at ledger_entry.rs). It is also `pub`, so a future call site
inherits the fail-open default. This contradicts the module's stated convention,
where `DnsPolicy::Reject` is `#[default]` specifically "so that a future call
site which forgets to think about provenance fails closed"
(addr_filter.rs:424-428). Currently unreachable from the single caller because
`is_private_or_cgnat_multiadr` gates it -- one refactor away.

Fix: return `false` at the end of the loop; require an explicit IP component.

---

## F14 -- LOW -- string-split address parsing diverges from the module's single-parser discipline

Location: `core/src/transport/addr_filter.rs:529-553`

`extract_ipv4` / `extract_ipv6` re-implement multiaddr parsing by `split('/')`,
taking the first parsable component and ignoring encapsulation order -- in a
module whose header states it exists so there is "exactly ONE definition".
Consequence: a listener expressed as `/ip6/::ffff:192.168.1.50/tcp/9001` yields
`None` from `extract_ipv4`, so it never matches an `/ip4/192.168.1.x` entry.
Conservative today, but the same address in two encodings gets two different
verdicts -- the class of inconsistency that produced the NAT64 bypass this
module was hardened against.

[OK] The IPv4-mapped-IPv6 bypass raised in the threat model (`::ffff:10.0.0.1`)
**is** correctly handled on the entry side: the `Ip6` branch routes through the
shared `embedded_ipv4` (addr_filter.rs:227-253), covering `::ffff:`, `::`,
NAT64 `64:ff9b::/96` and `64:ff9b:1::/48`, and 6to4 `2002::/16`.

Fix: parse `my_addrs` into `Multiaddr` once and reuse `embedded_ipv4`.

---

## F15 -- INFO -- relay reputation gate is live but keyed on the literal `"unknown"`

Location: `core/src/drift/relay.rs:231-246`; wired at
`core/src/iron_core.rs:4040-4042`, called from `initialize_identity` at `:680-686`

`abb32d45` turns a previously dead branch live: `shared_reputation_manager` is
now always set at identity init, so `get_enhanced_score(sender_peer_id)`
executes on every relayed envelope -- with `let sender_peer_id = "unknown";`
hardcoded. The gate provides zero per-sender abuse protection while appearing
enabled in code review and telemetry. If anything ever records a signal under
the literal `"unknown"`, `is_abusive() && spam_confidence > 0.8` becomes a
fleet-wide relay kill switch.

New lock edge: `drift_engine.write()` -> `abuse_manager.read()`. Searched the
workspace and found **no** `abuse_manager.write()` anywhere
(`EnhancedAbuseReputationManager` uses interior mutability -- `record_signal`
takes `&self`), so no inverse edge and no deadlock today. [OK] The ordering is
not recorded in the `IronCore` lock-ordering comment block.

Fix: extract the real sender from the envelope, or leave the branch dead until
it can be. Document the new lock edge.

---

## F16 -- INFO -- `///` doc comment on a match arm, masked by a crate-wide allow

Location: `cli/src/server.rs:1476`

`/// Accept = add the sender as a contact...` sits on the
`ClientIntent::AcceptMessageRequest` match arm. This normally trips
`unused_doc_comments` under the repo's `-D warnings` clippy gate; it is silent
only because `cli/src/main.rs:5` carries `#![allow(dead_code, unused)]`, and
`unused` is the lint group containing `unused_doc_comments`. The same blanket
allow is why `sender_is_resolvable` (server.rs:191) does not trip `dead_code`.

---

## F17 -- INFO -- legacy orphaned block rows from the argument-swap bug

Location: `cli/src/server.rs:794-796` (the fix)

`core.block_peer(peer_id, reason, None)` -> `core.block_peer(peer_id, None, reason)`
is a correct and important fix: the signature is
`block_peer(peer_id, device_id, reason)`, so every prior UI block that supplied a
reason was written as a device-specific block keyed `blocked:<peer>:<reason>`
with no peer-level row -- it never blocked anything. Migration note: those
orphaned rows persist, the peers they name are still unblocked, and `unblock`
cannot reach them without knowing the reason string.

---

## Claims verified -- explicit results

| Claim | Result | Evidence |
|---|---|---|
| T1 dual-flavor symmetry (public key <-> identity_id) | [OK] for the derivable direction | `block()` mirrors pk -> identity_id for peer-level, known-device and device-specific rows (blocked.rs:229-255). `is_blocked_resolved` / `is_blocked_and_deleted_resolved` / `get` all iterate `resolved_identifiers` (blocked.rs:393-421, 471-479). `unblock` reverse-maps by scanning for rows whose derived id equals the target (blocked.rs:332-372). The identity_id -> pk direction is correctly documented as underivable and handled at the check side. Tested by `block_under_public_key_matches_inbound_identity_id` and `unblock_canonical_identity_id_removes_public_key_devices_and_registry`. |
| T1 dual-flavor for libp2p peer id | [FAIL] | Not implemented at all -- F5. |
| T2 curve-point gate applied at every derivation site | [OK] site coverage, [WARN] strength | Exactly one derivation function (`identity_id_from_public_key_hex`, keys.rs:39-48), now gated; no path derives an identity_id from an unvalidated key. But the gate is ~50% effective by construction and three call sites treat it as a discriminator -- F6, F7. |
| T3 fail-closed pending requests | [OK] at core ingress and CLI listing | Every error arm in `receive_message` denies with no log-and-continue: blocked+deleted `Err` -> `return Err(Blocked)` (iron_core.rs:3303-3312); block lookup `Err` -> `true`, message hidden (`:3320-3330`); contact lookup `Err` -> `return Err(e)` (`:3271-3282`). CLI listing: unresolvable key -> `continue` (server.rs:1414-1418); block lookup error -> `.unwrap_or(true)` (`:1428-1430`). |
| T4 unified canonical request key | [OK] today, [WARN] conditionally | All three handlers call the same `message_request_key` (server.rs:1415, 1489, 1554). For every message that can reach the inbox today, `sender_public_key_hex` is `Some(<authenticated key>)` and `sender_id` is the canonical identity_id (iron_core.rs:3366-3373), so the key is a pure function of the authenticated key: no two senders collide, one sender cannot produce two keys. Residual: `is_valid_identity_id` accepts any 64-hex string (keys.rs:27-29), so if a message with `sender_public_key_hex: None` ever reaches the inbox (legacy bincode decode path, inbox.rs:36-42), the same sender yields a different key and `resolve_sender_public_key` hands the accept handler an unauthenticated plaintext key. Currently unreachable (no `IronCore` constructor uses the persistent inbox; inbox.rs:531-546) -- latent, not live. |
| addr_filter safety | [FAIL] | F1 (critical), F2, F3, F4, F13, F14. |
| Ingress canonicalization is authenticated | [OK] genuine improvement | `canonical_peer_id` now derives solely from the envelope key with a hard `ok_or(IronCoreError::CryptoError)?` (iron_core.rs:3258-3261); the previous `unwrap_or_else(|| message.sender_id.clone())` plaintext fallback is gone. The key is genuinely authenticated: the ratchet session index is `blake3(sender_public_key)` (crypto/encrypt.rs:598, 629), so a forged key selects a session whose chain keys or X25519 secret the attacker does not hold, and decryption fails. |
| No panic on attacker-controlled envelope input | [OK] | `copy_from_slice` at encrypt.rs:604, 638, 640 preceded by strict length validation in `decode_wire_envelope` (codec.rs:238-256: 32/32/24/32/1088). No `unwrap()`/`expect()` on attacker-influenced input in reviewed paths. |
| Lifecycle mutex | [OK] | `lifecycle_transition` (iron_core.rs:174, 616, 643) taken strictly outside both `running` and the drift locks, serializing start/stop without altering the documented running->drift order. `drift_activate`/`drift_deactivate` never take it, so no cycle. `parking_lot::Mutex` non-reentrancy safe here. Covered by three new concurrency tests. |
| Crypto module untouched (X25519 / XChaCha20-Poly1305) | [OK] | `git diff --stat origin/main...HEAD -- core/src/crypto core/src/privacy core/src/routing` is empty. No hard-project-rule violation. |
| Kani proofs | [OK] | No `kani-proofs`-covered code moved or changed. |
| `unsafe` blocks | [OK] | Zero `unsafe` added. |
| Supply chain | [OK] | No `Cargo.toml` or `Cargo.lock` delta in the entire branch. |
| Build / compile gate | [WARN] not verified | Reviewer was read-only and the machine had disk/concurrency constraints. Static review found no obvious compile breaks. **The gate must still be run before merge.** |

---

## Verdict

**BLOCK -- must not merge.**

F1 is a CRITICAL internal-network-disclosure regression that reopens a
previously-closed vulnerability, documented as closed in a comment three lines
above the new code. F3 and F4 independently route around `addr_filter`'s
absolute disclosure rules. F5 means the PR's headline T1 claim is not delivered
at the transport boundary.

### Minimum conditions to lift the block

1. **F1** -- the RFC1918 disclosure gate must take the requester's observed
   address as a mandatory, separate argument and must reject relayed
   (`/p2p-circuit`) connections outright. `my_addrs` alone cannot gate anything.
2. **F2** -- subnet comparison, not four-value class comparison. Fix the doc
   comment to match.
3. **F3** -- loopback, link-local, unspecified, multicast and broadcast excluded
   from the contact-chaining branch unconditionally; "verified contact" must not
   mean "we dialed them once".
4. **F4** -- revert the disclosure predicate to `success_count > 0`.
5. **F5** -- implement libp2p PeerId resolution, or withdraw the claim and
   change both swarm gates to `unwrap_or(true)`.

### Required regression tests before re-review

The current suite passes while the vulnerability is live, which is itself a
finding.

- A remote-internet requester (non-private `conn.remote_addr`) must receive
  **zero** RFC1918 entries even when our own listeners are RFC1918. No existing
  test models the requester at all.
- Same-class/different-subnet must be blocked (`192.168.1.x` entry vs
  `192.168.99.x` peer).
- A "verified contact" must receive zero loopback and zero `169.254.0.0/16`
  entries.
- An entry with `success_count == 0 && public_key.is_some()` must not appear in
  any exchange response.

### Should fix before the v0.4.0 tag, but not merge-blocking

F6-F10: block-list integrity, `getCanonicalPeerId` FFI semantics, the reject
TOCTOU, dual-flavor side-effect asymmetry, and the unblock-bricking parse
failure.

### Genuine improvements worth preserving

The authenticated-key ingress canonicalization (iron_core.rs:3258-3261), the
complete fail-closed error handling at ingress (`:3266-3330`), the `block_peer`
argument-order fix (cli/src/server.rs:796), and the lifecycle transition mutex
(iron_core.rs:174).
