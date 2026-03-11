# Security Policy

Status: Active
Last updated: 2026-03-07

SCMessenger is in active pre-release development. Please **do not** report security issues in public GitHub issues.

## Supported Versions

| Version line | Supported for security reports |
| --- | --- |
| `main` / `v0.2.0` alpha baseline | :white_check_mark: |
| Tagged `v0.1.x` releases | :warning: historical only; please verify against current `main` |
| Older unsupported snapshots | :x: |

`WS13` and `WS14` are planned follow-on workstreams for `v0.2.1`; they are not the current alpha baseline.

## Reporting a Vulnerability

Use one of the private GitHub security channels:

1. GitHub Security Advisories / the repository's **Report a vulnerability** flow
2. GitHub private maintainer contact if the advisory flow is unavailable

Please include:

- a clear description of the issue,
- affected component(s) and version/commit,
- reproduction steps or proof-of-concept details,
- impact assessment,
- and any suggested mitigation if you have one.

## Public Issue Policy

- Security vulnerabilities: **private report only**
- Non-sensitive defects: use the normal GitHub issue templates
- Questions or operator/support requests: use `SUPPORT.md`

## Security Posture Notes

- SCMessenger is designed around end-to-end encryption, identity ownership, and infrastructure independence.
- The repository is still in alpha-readiness work, so interfaces and mitigations may continue to change before broader release hardening.
- Current release and risk posture are tracked in:
  - `docs/CURRENT_STATE.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
