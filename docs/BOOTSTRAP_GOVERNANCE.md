# Bootstrap Governance Model (Alpha)

> **Status:** Locked for v0.1.2-alpha  
> **Last updated:** 2026-02-25

## Decision

SCMessenger uses a **static-first, env/URL-override** bootstrap governance model
for the alpha release. This is the simplest trustworthy model that satisfies both
self-hosted and centrally-operated deployment scenarios.

## Resolution Priority Chain

When a client starts, bootstrap nodes are resolved in strict priority order:

1. **Environment variable override** (`SC_BOOTSTRAP_NODES`)  
   Comma-separated list of multiaddr strings. If set and non-empty, this is the
   only source used. Designed for operators and CI.

2. **Remote URL fetch** (`remote_url` in `BootstrapConfig`)  
   HTTP GET to a JSON endpoint returning an array of multiaddr strings.
   Timeout: configurable (default 5 seconds). On failure, falls through.

3. **Static fallback list** (`static_nodes` in `BootstrapConfig`)  
   Hardcoded multiaddr strings compiled into the binary. Always available.
   Current default: GCP relay at `34.135.34.73:9001`.

## Trust Model

- **Alpha:** Bootstrap nodes are trusted implicitly. The static list is maintained
  by the project maintainers and compiled into each release.

- **Identity flexibility:** Bootstrap nodes may rotate their libp2p PeerId without
  breaking clients. Clients connect by IP:port and accept whichever valid Noise
  identity the remote presents. This supports infrastructure key rotation and
  multi-node deployments behind a single IP.

- **No PKI or certificate pinning** in alpha. Trust is based on:
  1. The compiled static node list (auditable in source).
  2. The operator's env/URL override (self-custodied).

## Self-Hosted Operator Guide

Self-hosted relay operators can configure their clients to use their own
bootstrap nodes:

```bash
# Option 1: Environment variable (highest priority)
export SC_BOOTSTRAP_NODES="/ip4/YOUR_IP/tcp/9001,/ip4/YOUR_IP2/tcp/9001"

# Option 2: Remote URL (second priority)
# Configure your client's BootstrapConfig with:
#   remote_url: "https://your-domain.com/bootstrap.json"
#
# The endpoint should return a JSON array:
# ["/ip4/YOUR_IP/tcp/9001", "/ip4/YOUR_IP2/tcp/9001"]
```

## Future Considerations (Post-Alpha)

- **Signed bootstrap lists:** Remote URL responses signed with a project key.
- **Gossip-based discovery:** Peers share known bootstrap nodes via ledger exchange.
- **Reputation-weighted selection:** Prioritize bootstrap nodes with higher uptime.
- **Decentralized registry:** Community-curated bootstrap list via on-chain or
  distributed registry.

These are tracked as post-alpha enhancements and will not block the 0.1.2 release.

## References

- Implementation: `core/src/transport/bootstrap.rs` (BootstrapResolver)
- UDL definition: `core/src/api.udl` (BootstrapConfig, BootstrapResolver)
- CLI usage: `cli/src/bootstrap.rs`
