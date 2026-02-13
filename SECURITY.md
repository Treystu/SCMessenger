# Security Policy

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you discover a security vulnerability in SCMessenger, please report it privately to help us address it before public disclosure.

### How to Report

1. **Email**: Send details to the repository maintainers via GitHub private vulnerability reporting
2. **GitHub Security Advisories**: Use GitHub's "Report a vulnerability" feature in the Security tab
3. **Include**: 
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: We will acknowledge receipt within 48 hours
- **Updates**: We will provide regular updates on our progress
- **Timeline**: We aim to address critical vulnerabilities within 7 days
- **Credit**: We will credit you in the security advisory (unless you prefer to remain anonymous)

## Security Considerations

SCMessenger is designed with security and privacy as core principles:

### Cryptography

- **Identity**: Ed25519 signing keys
- **Key Exchange**: X25519 ECDH (ephemeral per-message)
- **Encryption**: XChaCha20-Poly1305 with authenticated encryption
- **Hashing**: Blake3 for identity derivation and KDF
- **Sender Auth**: AAD binding + Ed25519 envelope signatures

### Privacy Features

- **No Accounts**: No phone numbers, email addresses, or accounts
- **Identity Ownership**: You control your Ed25519 keypair
- **Onion Routing**: Multi-hop routing for metadata protection
- **Cover Traffic**: Padding and timing obfuscation
- **No Central Servers**: Fully peer-to-peer mesh network

### Known Security Considerations

1. **Local Storage**: Identity keys stored on device - protect your device
2. **Physical Access**: An attacker with device access can read stored messages
3. **Network Analysis**: While onion routing helps, sophisticated attackers may attempt traffic correlation
4. **Bluetooth/WiFi**: Local transports have inherent range limitations and exposure

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |
| < 1.0   | :x: (pre-release)  |

## Security Best Practices for Users

- Keep your device secure with strong passwords/biometrics
- Keep SCMessenger updated to the latest version
- Verify contact public keys through out-of-band channels
- Be cautious about the physical security of devices running SCMessenger
- Understand that peer-to-peer networks have different threat models than server-based systems

## Audit Status

SCMessenger is an open-source project under active development. We welcome security audits and encourage responsible disclosure of any vulnerabilities discovered.

Current status:
- Regular internal security reviews
- Community code review via pull requests
- CodeQL security scanning in CI/CD
- No formal third-party security audit yet

## Contact

For security concerns, please use GitHub's private vulnerability reporting feature or contact the maintainers directly.
