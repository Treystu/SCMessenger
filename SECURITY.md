> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

# Security Policy

## [Needs Revalidation] Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you discover a security vulnerability in SCMessenger, please report it privately to help us address it before public disclosure.

### [Needs Revalidation] How to Report

1. **Email**: Send details to the repository maintainers via GitHub private vulnerability reporting
2. **GitHub Security Advisories**: Use GitHub's "Report a vulnerability" feature in the Security tab
3. **Include**: 
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### [Needs Revalidation] What to Expect

- **Acknowledgment**: We will acknowledge receipt within 48 hours
- **Updates**: We will provide regular updates on our progress
- **Timeline**: We aim to address critical vulnerabilities within 7 days
- **Credit**: We will credit you in the security advisory (unless you prefer to remain anonymous)

## [Needs Revalidation] Security Considerations

SCMessenger is designed with security and privacy as core principles:

### [Needs Revalidation] Cryptography

- **Identity**: Ed25519 signing keys
- **Key Exchange**: X25519 ECDH (ephemeral per-message)
- **Encryption**: XChaCha20-Poly1305 with authenticated encryption
- **Hashing**: Blake3 for identity derivation and KDF
- **Sender Auth**: AAD binding + Ed25519 envelope signatures

### [Needs Revalidation] Privacy Features

- **No Accounts**: No phone numbers, email addresses, or accounts
- **Identity Ownership**: You control your Ed25519 keypair
- **Onion Routing**: Multi-hop routing for metadata protection
- **Cover Traffic**: Padding and timing obfuscation
- **No Central Servers**: Fully peer-to-peer mesh network

### [Needs Revalidation] Known Security Considerations

1. **Local Storage**: Identity keys stored on device - protect your device
2. **Physical Access**: An attacker with device access can read stored messages
3. **Network Analysis**: While onion routing helps, sophisticated attackers may attempt traffic correlation
4. **Bluetooth/WiFi**: Local transports have inherent range limitations and exposure

## [Needs Revalidation] Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |
| < 1.0   | :x: (pre-release)  |

## [Needs Revalidation] Security Best Practices for Users

- Keep your device secure with strong passwords/biometrics
- Keep SCMessenger updated to the latest version
- Verify contact public keys through out-of-band channels
- Be cautious about the physical security of devices running SCMessenger
- Understand that peer-to-peer networks have different threat models than server-based systems

## [Needs Revalidation] Audit Status

SCMessenger is an open-source project under active development. We welcome security audits and encourage responsible disclosure of any vulnerabilities discovered.

Current status:
- Regular internal security reviews
- Community code review via pull requests
- CodeQL security scanning in CI/CD
- No formal third-party security audit yet

## [Needs Revalidation] Contact

For security concerns, please use GitHub's private vulnerability reporting feature or contact the maintainers directly.
