# MODEL: qwen3-coder-next:cloud
# BUDGET: 1200
# token_budget: 12000

# P1_CLI_024_mDNS_TxtRecordTooLong_For_Circuit_Addresses

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04, log scm.log.2026-06-04-21 lines 1–4)
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P1 — local-discovery quality
**Source:** Live log inspection of running scmessenger-cli.exe (HEAD `14ea6d61`)
**Depends on:** P0_BUILD_001

---

## Verified Gap (with reproduction)

First 4 lines of `C:/Users/SCMessenger/AppData/Local/scmessenger/logs/scm.log.2026-06-04-21`:

```
WARN libp2p_mdns::behaviour::iface::dns: Excluding address from response: TxtRecordTooLong
  address=/ip4/172.26.154.211/tcp/9002/ws/p2p/12D3KooW…/p2p-circuit/p2p/12D3KooW…/p2p-circuit/p2p/12D3KooW…
```

The libp2p `swarm` advertises the local peer over mDNS with **every** listen address,
including relayed/circuit addresses. Once the peer has used libp2p relay (the WSL↔Android
mesh produces p2p-circuit addresses), the multiaddrs inflate past the 1300-byte mDNS TXT
record limit and the mDNS daemon **silently drops** them.

Effect: a phone that joins the LAN later and tries to discover this node via mDNS sees
NOTHING. Direct TCP works (port 9101 is in listeners), but the on-LAN, zero-config path
is broken for any node that has previously used relay.

## Scope (~80 LoC across 2 files)

### Part A: Filter mDNS-advertised addresses (LOC: ~50)

In `core/src/transport/swarm.rs` (the function that builds the mDNS-behaviour addresses,
search for `behaviour::mdns::Mdns` or the `MdnsConfig`):

```rust
fn build_mdns_external_addrs(all_listeners: &[Multiaddr]) -> Vec<Multiaddr> {
    all_listeners
        .iter()
        .filter(|a| {
            // Only advertise addresses a LAN peer can actually reach us on:
            //  - /ip4/ or /ip6/ direct
            //  - NO p2p-circuit
            //  - NO /ws/ websocket relay
            let s = a.to_string();
            !s.contains("/p2p-circuit/") && !s.contains("/ws/") && !s.contains("/wss/")
        })
        .cloned()
        .collect()
}
```

Use the result when constructing `MdnsConfig` (or whatever variant is in the swarm init).
If the swarm doesn't have a hook for "advertise subset of addresses", fall back to
rebuilding the swarm with a `mDNS::with_addrs(...)` if available in libp2p 0.56, or
move the filter into the local mDNS-behaviour wrapper.

### Part B: Test that the filter excludes circuit addresses (LOC: ~30)

In `core/src/transport/swarm.rs` (add a `#[cfg(test)] mod tests`):

```rust
#[test]
fn mdns_filter_drops_circuit_addresses() {
    let addrs = vec![
        "/ip4/192.168.0.230/tcp/9101".parse().unwrap(),
        "/ip4/172.26.144.1/tcp/9101/p2p/12D3K…/p2p-circuit/p2p/12D3K…".parse().unwrap(),
        "/ip4/172.26.154.211/tcp/9002/ws/p2p/12D3K…/p2p-circuit/p2p/12D3K…".parse().unwrap(),
    ];
    let filtered = build_mdns_external_addrs(&addrs);
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].to_string().starts_with("/ip4/192.168.0.230/tcp/9101"));
}
```

## File Targets

- `core/src/transport/swarm.rs` [EDIT — add `build_mdns_external_addrs`, wire it into mDNS init, add test]

## Build Verification Commands

```bash
cargo check -p scmessenger-core
cargo test -p scmessenger-core --lib transport::swarm
```

## Acceptance Gates

1. Test `mdns_filter_drops_circuit_addresses` passes
2. After redeploy, the first 100 log lines on a fresh start contain ZERO `TxtRecordTooLong` warnings
3. Manual: a phone joining the LAN within 10 seconds sees the Windows node via mDNS

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001]
