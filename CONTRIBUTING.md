# Contributing to SCMessenger

**Status**: Active  
**Last updated**: 2026-03-07

Thank you for considering contributing to SCMessenger! This document provides guidelines and instructions for contributing.

Current release line: **v0.2.1 is the active alpha baseline** for repository work and bug reporting. Planned follow-on workstreams `WS13` and `WS14` remain future scope and should not be treated as part of the current alpha closeout unless maintainers explicitly retarget them.

## Table of Contents

- [Philosophy](#philosophy)
- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Commit Message Format](#commit-message-format)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Project Structure](#project-structure)
- [Areas for Contribution](#areas-for-contribution)
- [Security](#security)
- [Questions and Support](#questions-and-support)
- [License](#license)

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

**Required:**
- Rust 1.75.0 or later (see `rust-toolchain.toml`)
- Cargo (comes with Rust)
- Git 2.30+

**Platform-Specific:**
- **Android**: Android SDK, NDK r26b, Java 17+ - See [Android Setup Guide](docs/platform/ANDROID_SETUP.md)
- **iOS**: macOS, Xcode 15+, CocoaPods - See [iOS Setup Guide](docs/platform/IOS_SETUP.md)
- **WASM**: Node.js 20+, wasm-pack - See [WASM Setup Guide](docs/platform/WASM_SETUP.md)

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/Treystu/SCMessenger.git
cd SCMessenger

# Install pre-commit hooks (recommended)
./scripts/install_hooks.sh  # Unix/macOS/Git Bash
# OR
powershell -ExecutionPolicy Bypass -File scripts/install_hooks.ps1  # Windows PowerShell

# Build the workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run CLI
cargo run --release --bin scmessenger-cli -- --help
```


## Development Workflow

### Branch Protection Rules

The `main` branch is protected with the following rules to ensure code quality:

**Required Configuration (via GitHub Settings → Branches → Branch protection rules):**

1. **Prevent force pushes** — Enabled
   - Protects commit history integrity
   
2. **Prevent deletion** — Enabled
   - Prevents accidental branch deletion
   
3. **Require status checks to pass before merging** — Enabled
   - Required checks:
     - `ci / rust-core` — Core Rust tests
     - `ci / rust-android` — Android build (if android/ changed)
     - `ci / rust-ios` — iOS build (if iOS/ changed)
     - `ci / rust-wasm` — WASM build (if wasm/ changed)
     - `ci / rust-cli` — CLI build (if cli/ changed)

**Note:** On GitHub Free tier, we cannot enforce:
- Required reviewers (requires GitHub Pro)
- CODEOWNERS enforcement (requires GitHub Pro)
- Private security advisories (requires GitHub Pro)

**To configure branch protection:**
1. Go to: `Settings → Branches → Add branch protection rule`
2. Branch name pattern: `main`
3. Enable: "Require status checks to pass before merging"
4. Select required checks from the list above
5. Enable: "Do not allow bypassing the above settings"
6. Enable: "Restrict who can push to matching branches" (optional, for team repos)
7. Save changes

### 1. Fork and Branch

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/SCMessenger.git
cd SCMessenger
git remote add upstream https://github.com/Treystu/SCMessenger.git

# Create a feature branch
git checkout -b feat/your-feature-name
```

### 2. Make Changes

- Write clear, concise code following our conventions (see below)
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 3. Code Style

Rust is the primary implementation language for shared/core logic, but this repository also contains active Android (Kotlin), iOS (Swift), and GitHub/doc surfaces that should be updated when a task requires them.

#### Rust Conventions

**Required:**
- Use `thiserror` for error types, `anyhow` for error propagation in binaries
- Use `tracing` for logging (not `println!` in library code)
- Use `parking_lot::RwLock` over `std::sync::RwLock`
- Async runtime is `tokio`
- Network layer is `libp2p` 0.53
- Serialization: `bincode` for wire format, `serde_json` for human-readable
- Tests go in `#[cfg(test)] mod tests` in the same file

**Code Quality:**

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

#### Kotlin Style (Android)

- Follow [Kotlin Coding Conventions](https://kotlinlang.org/docs/coding-conventions.html)
- Use 4 spaces for indentation
- Maximum line length: 120 characters
- Use camelCase for variables and functions
- Use PascalCase for classes

**Linting:**
```bash
cd android
./gradlew ktlintCheck
./gradlew ktlintFormat  # Auto-fix
```

#### Swift Style (iOS)

- Follow [Swift API Design Guidelines](https://swift.org/documentation/api-design-guidelines/)
- Use 4 spaces for indentation
- Maximum line length: 120 characters
- Use camelCase for variables and functions
- Use PascalCase for types

**Linting:**
```bash
cd iOS
swiftlint lint
```

#### JavaScript/TypeScript Style (WASM)

- Follow [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript)
- Use 2 spaces for indentation
- Maximum line length: 100 characters
- Use camelCase for variables and functions
- Use PascalCase for classes

**Linting:**
```bash
cd wasm
npm run lint
```

### 4. Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements
- `ci`: CI/CD changes
- `build`: Build system changes
- `revert`: Revert a previous commit

**Examples:**

```bash
# Simple feature
git commit -m "feat: add message encryption"

# Bug fix with scope
git commit -m "fix(android): resolve BLE connection timeout"

# Breaking change
git commit -m "feat!: change message format to protobuf

BREAKING CHANGE: Message format changed from bincode to protobuf.
Clients must upgrade to maintain compatibility."

# Reference issue
git commit -m "fix: resolve relay connection race condition

Fixes #123"
```

**Commit Message Hook:**

The repository includes a commit-msg hook that enforces this format. Install it:

```bash
./scripts/install_hooks.sh  # Unix/macOS/Git Bash
# OR
powershell -ExecutionPolicy Bypass -File scripts/install_hooks.ps1  # Windows PowerShell
```

### 5. Testing

All changes must include appropriate tests.

#### Unit Tests

```bash
# Run all tests
cargo test --workspace

# Run specific module tests
cargo test -p scmessenger-core

# Run with logging
RUST_LOG=debug cargo test --workspace -- --nocapture

# Run specific test
cargo test -p scmessenger-core test_message_encryption
```

#### Integration Tests

```bash
# Run integration tests
cargo test --workspace --test '*'

# Run specific integration test
cargo test -p scmessenger-core --test integration_offline_partition_matrix
```

#### Platform-Specific Tests

```bash
# Android
cd android && ./gradlew test

# iOS
cd iOS && xcodebuild test -workspace SCMessenger.xcworkspace -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 15'

# WASM
cd wasm && wasm-pack test --headless --firefox
```

**Test Requirements:**
- All new features must include tests
- Bug fixes must include regression tests
- Maintain or improve code coverage (target: 80% for core modules)
- Property-based tests for parsers, serializers, and cryptographic operations

See [Testing Guide](docs/TESTING_GUIDE.md) for comprehensive testing documentation.

### 6. Documentation

- Update relevant documentation in `docs/` if you change architecture
- Update README.md if you change setup/usage instructions
- Add inline comments for complex logic (but prefer self-documenting code)
- Don't write docs longer than the code they document
- Run documentation sync check:
  ```bash
  ./scripts/docs_sync_check.sh  # Unix/macOS/Git Bash
  # OR
  powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs_sync_check.ps1  # Windows PowerShell
  ```

### 7. Submit Pull Request

```bash
# Push to your fork
git push origin feat/your-feature-name

# Create PR on GitHub
```

**PR Requirements:**
- All CI checks must pass (formatting, linting, build, tests)
- Include description of changes and motivation
- Reference any related issues (e.g., "Fixes #123")
- Add tests for new functionality
- Update documentation as needed
- Fill out the PR template completely

**PR Template Checklist:**
- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published

## Testing Requirements

### Test Coverage

- **Core modules**: Minimum 80% line coverage
- **Unit tests**: Test specific examples and edge cases
- **Integration tests**: Test component interactions
- **Property-based tests**: Test universal properties (parsers, serializers, crypto)
- **Regression tests**: Test for all previously fixed critical/high bugs

### Property-Based Testing

For parsers, serializers, and cryptographic operations, use property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_message_serialization_roundtrip(msg in any::<Message>()) {
        let encoded = bincode::serialize(&msg).unwrap();
        let decoded: Message = bincode::deserialize(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }
}
```

### Running Tests Locally

Before submitting a PR, ensure all tests pass:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --workspace -- -D warnings

# Unit tests
cargo test --workspace

# Integration tests
cargo test --workspace --test '*'

# Documentation sync
./scripts/docs_sync_check.sh
```

## Pull Request Process

1. **Create PR**: Open a pull request from your fork to `main`
2. **CI Checks**: Wait for automated checks to complete
3. **Review**: Address any feedback from maintainers
4. **Approval**: PR must pass all checks before merge
5. **Merge**: Maintainer will merge using squash or rebase

**PR Review Criteria:**
- Code quality and style compliance
- Test coverage and passing tests
- Documentation updates
- No breaking changes (unless justified)
- Performance considerations
- Security implications

## Project Structure

```
SCMessenger/
├── core/           # scmessenger-core - Rust library (~29K LoC)
│   ├── src/
│   │   ├── identity/      # Ed25519 keys, Blake3 hashing
│   │   ├── crypto/        # X25519 ECDH + XChaCha20-Poly1305
│   │   ├── message/       # Message types, envelope format
│   │   ├── store/         # Outbox/inbox with quotas
│   │   ├── transport/     # BLE, WiFi, Internet, NAT traversal
│   │   ├── drift/         # Drift Protocol for mesh sync
│   │   ├── routing/       # Mycorrhizal routing engine
│   │   ├── relay/         # Self-relay network
│   │   └── privacy/       # Onion routing, cover traffic
│   └── tests/
│       ├── unit/          # Fast unit tests
│       ├── integration/   # Integration tests
│       ├── property/      # Property-based tests
│       └── regression/    # Bug-specific regression tests
├── mobile/         # scmessenger-mobile - UniFFI bindings
├── cli/            # scmessenger-cli - Interactive CLI
├── wasm/           # scmessenger-wasm - Browser bindings
├── android/        # Android app (Kotlin + Jetpack Compose)
├── iOS/            # iOS app (Swift + SwiftUI)
├── docs/           # Architecture and design docs
├── scripts/        # Build and utility scripts
└── .github/        # GitHub Actions workflows

```

### Core Modules

- **identity**: Ed25519 keys, Blake3 hashing, sled persistence
- **crypto**: X25519 ECDH + XChaCha20-Poly1305 encryption
- **message**: Message types, envelope format, bincode codec
- **store**: Outbox/inbox with quotas and deduplication
- **transport**: BLE, WiFi Aware, WiFi Direct, Internet, NAT traversal
- **drift**: Drift Protocol for mesh synchronization
- **routing**: Mycorrhizal routing engine
- **relay**: Self-relay network, bootstrap, peer exchange
- **privacy**: Onion routing, cover traffic, padding
- **mobile**: Mobile service lifecycle, auto-adjust
- **platform**: Platform-specific implementations
- **wasm_support**: Browser mesh participation

## Areas for Contribution

### High Priority

- Mobile platform optimizations (battery, connectivity)
- BLE transport improvements
- NAT traversal enhancements
- Routing algorithm refinements
- Privacy feature additions
- Test coverage improvements
- Documentation improvements

### Documentation

- Architecture diagrams
- Protocol specifications
- Platform-specific guides
- Example applications
- API documentation
- Troubleshooting guides

### Testing

- Integration tests
- Network scenario testing
- Load/stress testing
- Security audits
- Property-based tests
- Regression tests

### Infrastructure

- CI/CD improvements
- Build system optimizations
- Release automation
- Monitoring and observability
- Performance profiling

## Security

If you discover a security vulnerability, please follow our [Security Policy](SECURITY.md). 

**DO NOT** open a public issue for security vulnerabilities.

**Reporting:**
- Use GitHub Security Advisories (private disclosure)
- Or email: security@scmessenger.org (if available)

See [SECURITY.md](SECURITY.md) for details on:
- Supported versions
- Vulnerability reporting procedures
- Response time expectations
- Severity classification

## Questions and Support

- **Documentation**: Start with [DOCUMENTATION.md](DOCUMENTATION.md) and `docs/` directory
- **Build Issues**: Check [Build Issues Guide](docs/troubleshooting/BUILD_ISSUES.md)
- **CI Failures**: Check [CI Failures Guide](docs/troubleshooting/CI_FAILURES.md)
- **Runtime Issues**: Check [Runtime Issues Guide](docs/troubleshooting/RUNTIME_ISSUES.md)
- **GitHub Issues**: Search existing issues or create a new one
- **GitHub Discussions**: For questions and general discussion
- **Support**: See [SUPPORT.md](SUPPORT.md) for routing guidance

## License

By contributing, you agree that your contributions will be licensed under The Unlicense.

---

**Thank you for making SCMessenger better!** 🚀

For the current verified workspace snapshot and implementation status, see [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md).
