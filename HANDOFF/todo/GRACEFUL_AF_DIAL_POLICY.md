# TASK: Graceful-AF Dial Policy — self-dial prevention + network-aware RFC1918 filtering + backoff

Status: READY FOR DELEGATION (Qwen THINK → CODER)
Source: Direct observation 2026-07-20 (Lucas CLI connected to alpha relay via
ss verified; then proceeded to promiscuously dial its own LAN IP, emulator
10.0.2.x junk from ledger, and all 48 ledger entries with 5s backoff each)

## Problem

After the initial relay-connection ledger exchange, the CLI received 48 peer
addresses via the ledger and began dialing ALL of them with ~5s backoff in a
"promiscuous" loop. This included:
- Its own LAN IP (192.168.0.121) — self-dial, wastes resources
- Emulator-internal 10.0.2.x addresses (RFC1918 private, but the CLI is on a
  public internet network, not on 10.0.0.0/8)
- Other private/internal addresses that cannot route from this machine

## Required Changes

### 1. Self-dial prevention
Track the local node's own known addresses (listen addrs + external addrs).
Before dialing any address, check if it resolves to an address this node
already holds. Skip if yes.

Location: `core/src/transport/swarm.rs` (where dial is initiated) or
`cli/src/ledger.rs` (where dial targets are enumerated). Ledger-side is
preferred since the CLI ledger dial loop is the promiscuous source.

### 2. Private-range awareness
The existing `is_dialable_multiaddr` filter (commit 36635cb0) rejects
loopback/link-local/site-local. It should also reject RFC1918 private
addresses (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16) UNLESS the local
node itself is on that private network.

How to determine "my network": compare each private-range address against
the node's own listen/external addresses. If ANY listen address is in the
same private range, allow dialing that range. Otherwise, reject.

Location: `cli/src/ledger.rs` is_dialable_multiaddr or the caller.

### 3. Backoff discipline
The promiscuous dial loop uses 5s backoff per address but chains them
sequentially with no per-peer cap. Add:
- Per-peer backoff state (dedup by peer ID, not address)
- At most 3 concurrent outbound dials to unknown peers
- Exponential backoff per peer: 5s, 30s, 120s, 5min, 30min
- After the initial relay connection, prefer circuit-relay routing over
  direct promiscuous dials to unknown addresses

### 4. Success-based pruning
If the relay circuit is working (circuit-reservation granted, relay forwarding
established), use it for message delivery. Only fall back to direct dials
when a message needs to reach a specific peer that doesn't respond via relay.

## Files to Modify

- `cli/src/ledger.rs` — add self-dial check + private-range-awareness + backoff
- Possibly `cli/src/main.rs` — pass own-address list into the ledger filter

## Verification

1. Start CLI, connect to relay, observe ledger exchange → should NOT dial
   own IP or junk private addresses
2. Should dial the relay only (bootstrap seed) and learn peers via ledger
   without promiscuous looping
3. Relay circuit should be used for routing instead of direct dials

## Security Review

Mandatory crypto-security-auditor (transport/ routing concerns: modified dial
behavior affects connection-layer security properties).
