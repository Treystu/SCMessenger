# Contributing to SCMessenger

Thank you for considering contributing to SCMessenger! This document provides guidelines and instructions for contributing.

## Philosophy

SCMessenger is the world's first truly sovereign messenger — works everywhere, owned by no one, unstoppable by design. Our core principles guide all development:

1. **Relay = Messaging** — Non-negotiable coupling. Want to talk? You relay for others. No free riders.
2. **Every node IS the network** — No third-party relays, no external infrastructure.
3. **Internet is a transport, not a dependency** — Use when available, never require it.
4. **Privacy + Infrastructure independence + Identity ownership** — All three, always.
5. **Mass market UX** — Grandma should be able to use this.

## Code of Conduct

We are committed to providing a welcoming and inspiring community for all. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

- Rust 1.93.1 or later
- Cargo (comes with Rust)
- Git

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger

# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run CLI
cargo run -p scmessenger-cli -- --help
```

## Development Workflow

### 1. Fork and Branch

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/SCMessenger.git
cd SCMessenger
git remote add upstream https://github.com/Treystu/SCMessenger.git

# Create a feature branch
git checkout -b feature/your-feature-name
```

### 2. Make Changes

- Write clear, concise code following our conventions (see below)
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 3. Code Style

**All new code is Rust. No TypeScript, no JavaScript.**

#### Key Conventions

- Use `thiserror` for error types, `anyhow` for error propagation in binaries
- Use `tracing` for logging (not `println!` in library code)
- Use `parking_lot::RwLock` over `std::sync::RwLock`
- Async runtime is `tokio`
- Network layer is `libp2p` 0.53
- Serialization: `bincode` for wire format, `serde_json` for human-readable
- Tests go in `#[cfg(test)] mod tests` in the same file

#### Code Quality

```bash
# Format code (REQUIRED before commit)
cargo fmt --all

# Run linter
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments

# Run tests
cargo test --workspace
```

**Do NOT:**
- Add unnecessary abstractions where concrete types work
- Use `unwrap()` in library code (use `?` or `expect()` with context)
- Add new dependencies without checking if an existing workspace dep covers the need
- Use time-based estimates in plans or roadmaps (use LoC estimates only)
- Decouple relaying from messaging — they are permanently bound

### 4. Commit Messages

Write clear commit messages following these guidelines:

```
Short (50 chars or less) summary

More detailed explanatory text, if necessary. Wrap it to about 72
characters. The blank line separating the summary from the body is
critical.

- Bullet points are okay
- Use present tense ("Add feature" not "Added feature")
- Reference issues: "Fixes #123" or "Closes #456"
```

### 5. Testing

```bash
# Run all tests
cargo test --workspace

# Run specific module tests
cargo test -p scmessenger-core

# Run with logging
RUST_LOG=debug cargo test --workspace -- --nocapture
```

All tests must pass before submitting a PR. We currently have ~638 tests across all modules.

### 6. Documentation

- Update relevant documentation in `docs/` if you change architecture
- Update README.md if you change setup/usage instructions
- Add inline comments for complex logic (but prefer self-documenting code)
- Don't write docs longer than the code they document

### 7. Submit Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create PR on GitHub
```

**PR Requirements:**
- All CI checks must pass (formatting, linting, build, tests)
- Include description of changes and motivation
- Reference any related issues
- Add tests for new functionality
- Update documentation as needed

## Project Structure

```
core/        scmessenger-core    Rust library (~29K LoC)
cli/         scmessenger-cli     Interactive CLI
mobile/      scmessenger-mobile  iOS/Android bindings (UniFFI)
wasm/        scmessenger-wasm    Browser bindings (wasm-bindgen)
reference/   —                   V1 TypeScript (porting guides only)
docs/        —                   Architecture and design docs
```

### Core Modules

- `identity` — Ed25519 keys, Blake3 hashing, sled persistence
- `crypto` — X25519 ECDH + XChaCha20-Poly1305 encryption
- `message` — Message types, envelope format, bincode codec
- `store` — Outbox/inbox with quotas and deduplication
- `transport` — BLE, WiFi Aware, WiFi Direct, Internet, NAT traversal
- `drift` — Drift Protocol for mesh synchronization
- `routing` — Mycorrhizal routing engine
- `relay` — Self-relay network, bootstrap, peer exchange
- `privacy` — Onion routing, cover traffic, padding
- `mobile` — Mobile service lifecycle, auto-adjust
- `platform` — Platform-specific implementations
- `wasm_support` — Browser mesh participation

## Areas for Contribution

### High Priority

- Mobile platform optimizations (battery, connectivity)
- BLE transport improvements
- NAT traversal enhancements
- Routing algorithm refinements
- Privacy feature additions

### Documentation

- Architecture diagrams
- Protocol specifications
- Platform-specific guides
- Example applications

### Testing

- Integration tests
- Network scenario testing
- Load/stress testing
- Security audits

## Security

If you discover a security vulnerability, please follow our [Security Policy](SECURITY.md). Do NOT open a public issue.

## Questions?

- Open a [GitHub Discussion](https://github.com/Treystu/SCMessenger/discussions)
- Check existing [Issues](https://github.com/Treystu/SCMessenger/issues)
- Review the [Documentation](docs/)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

Thank you for making SCMessenger better! 🚀
