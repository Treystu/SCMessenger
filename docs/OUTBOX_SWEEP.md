# Outbox sweep: what runs, how often, and what it cannot do

Status: current for `origin/main` at the time of writing (`cb06e777`).
Scope: the native store-and-forward sweep (`sweep_outbox_over_swarm`) and its
absence on WASM. Records the facts #495 asked for as a written note rather than
as a one-line issue comment.

## Where it runs

The sweep is a native transport-loop timer, not a per-message retry:

- interval: **120 s**, `core/src/transport/swarm.rs:5065`
  (`tokio::time::interval(Duration::from_secs(120))`)
- tick handler: `core/src/transport/swarm.rs:5396`, which calls
  `sweep_outbox_over_swarm` at `:5398`
- implementation: `core/src/transport/swarm.rs:2828`

It sweeps only peers that are **both registered and currently connected** - the
function returns `0` when that set is empty (`swarm.rs:2841-2844`), so a node
with no connected peer does no outbox work at all, however long messages sit in
the queue.

## How much it can move, and the "64 / 120 s" figure

Two budgets cap one tick:

| Budget | Value | Source |
|---|---|---|
| total messages per tick, across all peers | 64 | `core/src/iron_core.rs:3862` (`OUTBOX_SWEEP_MAX_PER_TICK`) |
| messages per peer per tick | 16 | `core/src/iron_core.rs:3861` (`OUTBOX_SWEEP_MAX_PER_PEER`) |

Peers are ordered by public key and then rotated by a per-tick offset, so the
64-message budget is spread fairly over ticks rather than always reaching the
same peers first (`swarm.rs:2846-2849`).

Consequences worth stating plainly, because they are easy to misread as a bug:

- the ceiling is **64 messages per 120 s** (about 0.53 messages/second) for the
  whole node, no matter how many peers are connected;
- with a single connected peer the ceiling is **16 per 120 s**, because the
  per-peer cap binds first;
- a large backlog therefore drains in `ceil(backlog / 64)` ticks, i.e. two
  minutes per 64 messages. That is the intended store-and-forward cadence, not
  a stall.

## WASM has no sweep at all

`git grep sweep_outbox -- wasm` returns nothing, and `wasm/src` contains no
sweep reference: the WASM build enqueues but never drains on a timer. This is a
real gap, not a documentation omission, and it is tracked as **#583** (WASM
outbox sweep). It is recorded here rather than "verified" as working, because it
does not work.

## What this document is not

It is not a claim that delivery depends on the sweep alone: direct-link delivery
releases the outbox entry on a transport ACK
(`cli/src/api.rs:1062` `should_release_outbox`), and the sweep exists for the
store-and-forward case where no direct link was available. The sweep is the
floor rate, not the delivery mechanism.
