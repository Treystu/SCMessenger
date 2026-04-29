# Agent Task: Integrate Security Tooling

**Delegated To:** triage-router (gemini-3-flash-preview:cloud)  
**Priority:** P1 (Phase 3D prerequisite)  
**Status:** pending_agent_dispatch  
**Depends On:** Phase 1A compilation baseline passing

## Objective
Integrate 3 automated security tools to complement the manual adversarial review protocol in Phase 3.

## Tasks

### 1. cargo-deny — Supply Chain Audit
- Install: `cargo install cargo-deny`
- Create `deny.toml` config in repo root
- Run: `cargo deny check advisories`
- Document any findings in HANDOFF/done/

### 2. cargo-audit — CVE Scan  
- Install: `cargo install cargo-audit`
- Run: `cargo audit`
- Document any CVEs found

### 3. miri — Unsafe Block Validation
- Install via rustup: `rustup +nightly component add miri`
- Target files with `// SAFETY:` comments in:
  - core/src/crypto/
  - core/src/transport/
- Run: `cargo +nightly miri test` on applicable test targets

## Success Criteria
- All three tools installed and configured
- Zero actionable CRITICAL/HIGH findings (or documented with justification)
- Handoff report filed to HANDOFF/done/security_tooling_integration.md
