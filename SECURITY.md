# Security Policy

Status: Active  
Last updated: 2026-03-07  
Version: v0.2.1 (alpha)

SCMessenger is in active pre-release development. Security is a top priority, and we take all vulnerability reports seriously.

## Table of Contents

- [Supported Versions](#supported-versions)
- [Reporting a Vulnerability](#reporting-a-vulnerability)
- [Vulnerability Response Process](#vulnerability-response-process)
- [Severity Classification](#severity-classification)
- [Response Time Expectations](#response-time-expectations)
- [Public Disclosure Policy](#public-disclosure-policy)
- [Security Posture Notes](#security-posture-notes)
- [Security Best Practices](#security-best-practices)

## Supported Versions

| Version line | Supported for security reports | Status |
| --- | --- | --- |
| `main` / `v0.2.1` alpha baseline | :white_check_mark: | Active development |
| Tagged `v0.2.0` releases | :white_check_mark: | Supported |
| Tagged `v0.1.x` releases | :warning: | Historical only; verify against current `main` |
| Older unsupported snapshots | :x: | Not supported |

**Note:** `WS13` and `WS14` are planned follow-on workstreams for future releases; they are not the current alpha baseline.

## Reporting a Vulnerability

**IMPORTANT:** Please **do not** report security vulnerabilities in public GitHub issues.

### Preferred Reporting Methods

1. **GitHub Security Advisories** (Recommended)
   - Go to: https://github.com/Treystu/SCMessenger/security/advisories
   - Click "Report a vulnerability"
   - Fill out the advisory form with details

2. **Private Maintainer Contact**
   - If the advisory flow is unavailable, contact maintainers directly
   - Use encrypted communication when possible

### What to Include

Please provide as much information as possible:

- **Clear description** of the vulnerability
- **Affected component(s)** and version/commit
- **Reproduction steps** or proof-of-concept details
- **Impact assessment** (what can an attacker do?)
- **Suggested mitigation** (if you have one)
- **Your contact information** for follow-up questions

### Example Report Template

```
## Vulnerability Description
[Clear description of the security issue]

## Affected Components
- Component: [e.g., core/crypto, android/app, iOS/SCMessenger]
- Version: [e.g., v0.2.1, commit abc123]
- Platform: [e.g., All, Android only, iOS only]

## Reproduction Steps
1. [Step 1]
2. [Step 2]
3. [Observe vulnerability]

## Impact Assessment
- Severity: [Critical/High/Medium/Low]
- Attack Vector: [Network/Local/Physical]
- Privileges Required: [None/Low/High]
- User Interaction: [None/Required]
- Impact: [Confidentiality/Integrity/Availability]

## Suggested Mitigation
[Your suggested fix, if any]

## Additional Context
[Any other relevant information]
```

## Vulnerability Response Process

1. **Acknowledgment** (Within 48 hours)
   - We will acknowledge receipt of your report
   - Assign a tracking identifier
   - Confirm we are investigating

2. **Initial Assessment** (Within 7 days)
   - Validate the vulnerability
   - Assess severity and impact
   - Determine affected versions
   - Provide initial timeline

3. **Fix Development** (Timeline varies by severity)
   - Develop and test fix
   - Prepare security advisory
   - Coordinate disclosure timeline

4. **Release and Disclosure** (Coordinated)
   - Release patched version
   - Publish security advisory
   - Credit reporter (if desired)
   - Notify affected users

## Severity Classification

We use a modified CVSS-based severity classification:

### Critical (CVSS 9.0-10.0)

**Characteristics:**
- Remote code execution
- Complete system compromise
- Cryptographic key extraction
- Mass data breach

**Response Time:** Fix within 7 days

**Examples:**
- Remote code execution in message parser
- Private key extraction vulnerability
- Authentication bypass allowing full access

### High (CVSS 7.0-8.9)

**Characteristics:**
- Significant data exposure
- Authentication bypass
- Privilege escalation
- Denial of service (persistent)

**Response Time:** Fix within 14 days

**Examples:**
- Message content decryption without key
- Identity spoofing vulnerability
- Persistent crash causing data loss

### Medium (CVSS 4.0-6.9)

**Characteristics:**
- Limited data exposure
- Denial of service (temporary)
- Information disclosure
- Security feature bypass

**Response Time:** Fix within 30 days

**Examples:**
- Metadata leakage
- Temporary service disruption
- Timing side-channel

### Low (CVSS 0.1-3.9)

**Characteristics:**
- Minimal impact
- Requires significant preconditions
- Theoretical vulnerability
- Best practice violation

**Response Time:** Fix in next release

**Examples:**
- Minor information disclosure
- Hardening opportunity
- Configuration issue

## Response Time Expectations

| Severity | Acknowledgment | Initial Assessment | Fix Target | Disclosure |
|----------|----------------|-------------------|------------|------------|
| Critical | 24 hours | 48 hours | 7 days | Coordinated |
| High | 48 hours | 7 days | 14 days | Coordinated |
| Medium | 72 hours | 14 days | 30 days | Coordinated |
| Low | 7 days | 30 days | Next release | Coordinated |

**Note:** These are target timelines. Actual response may vary based on complexity and resources.

## Public Disclosure Policy

### Coordinated Disclosure

We follow responsible disclosure practices:

1. **Private Reporting**: Vulnerabilities reported privately
2. **Fix Development**: Fix developed and tested privately
3. **Coordinated Release**: Fix released before public disclosure
4. **Public Advisory**: Security advisory published after fix
5. **Credit**: Reporter credited (if desired)

### Disclosure Timeline

- **Standard**: 90 days from initial report
- **Critical**: Expedited (7-14 days)
- **Extensions**: By mutual agreement

### What We Publish

Security advisories include:

- Vulnerability description
- Affected versions
- Fixed versions
- Severity rating
- Mitigation steps
- Credit to reporter (if desired)

### What We Don't Publish

- Detailed exploitation techniques
- Proof-of-concept code (initially)
- Information that could aid attackers

## Security Posture Notes

### Current Security Status

SCMessenger is designed around end-to-end encryption, identity ownership, and infrastructure independence. However, as an alpha-stage project, the following caveats apply:

**Cryptographic Design:**
- ✅ Ed25519 for identity and signatures
- ✅ X25519 ECDH for key exchange
- ✅ XChaCha20-Poly1305 for encryption
- ✅ Blake3 for hashing
- ⚠️ Cryptographic implementation under active review

**Security Features:**
- ✅ End-to-end encryption
- ✅ Forward secrecy (planned)
- ✅ Identity verification
- ⚠️ Onion routing (in development)
- ⚠️ Cover traffic (planned)

**Platform Security:**
- ✅ Android: ProGuard/R8 enabled for release builds
- ✅ iOS: App Transport Security (ATS) configured
- ✅ WASM: Sandboxed execution
- ⚠️ Secure key storage (platform-dependent)

### Known Limitations

1. **Alpha Status**: Not yet recommended for production use
2. **Audit Status**: No formal security audit completed
3. **Key Management**: Secure key storage varies by platform
4. **Network Security**: Transport security under active development
5. **Side Channels**: Timing and traffic analysis mitigations in progress

### Security Roadmap

See current release and risk posture in:
- [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md)
- [docs/MILESTONE_PLAN_V0.2.0_ALPHA.md](docs/MILESTONE_PLAN_V0.2.0_ALPHA.md)
- [docs/V0.2.0_RESIDUAL_RISK_REGISTER.md](docs/V0.2.0_RESIDUAL_RISK_REGISTER.md)

## Security Best Practices

### For Users

1. **Keep Updated**: Always use the latest version
2. **Verify Identity**: Verify contact identities out-of-band
3. **Secure Device**: Use device encryption and strong passwords
4. **Network Security**: Be cautious on untrusted networks
5. **Backup Keys**: Securely backup identity keys

### For Developers

1. **Code Review**: All security-sensitive code requires review
2. **Testing**: Comprehensive tests for security features
3. **Dependencies**: Regular dependency audits
4. **Secrets**: Never commit secrets to git
5. **Unsafe Code**: All `unsafe` blocks require `// SAFETY:` comments

### For Operators

1. **Relay Security**: Secure relay node configurations
2. **Monitoring**: Monitor for suspicious activity
3. **Updates**: Keep relay nodes updated
4. **Logging**: Secure log storage and rotation
5. **Access Control**: Restrict administrative access

## Security Resources

### Documentation

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Cryptographic Design](docs/PROTOCOL.md)
- [Testing Guide](docs/TESTING_GUIDE.md)
- [Deployment Guide](docs/DEPLOYMENT.md)

### Tools and Scripts

- `scripts/audit_unsafe.sh` - Audit unsafe Rust code
- `scripts/verify_platform_security.sh` - Verify platform security configs
- `.gitleaks.toml` - Secret scanning configuration
- `deny.toml` - Dependency security policy

### External Resources

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Mobile Security](https://owasp.org/www-project-mobile-security/)
- [libp2p Security](https://docs.libp2p.io/concepts/security/)

## Contact

### Security Team

- **GitHub Security Advisories**: https://github.com/Treystu/SCMessenger/security/advisories
- **Email**: security@scmessenger.org (if available)

### General Support

- **Bug Reports**: Use GitHub Issues (non-security bugs only)
- **Questions**: Use GitHub Discussions
- **Support**: See [SUPPORT.md](SUPPORT.md)

---

**Thank you for helping keep SCMessenger secure!**

For non-security bugs and feature requests, please use the normal GitHub issue templates.
