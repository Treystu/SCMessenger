# Docker build environment for reproducible Linux builds
# Requirement 7.7: Use Docker containers for Linux builds to ensure consistent environment
# Requirement 7.1: Pinned Rust toolchain version (rust-toolchain.toml)

FROM rust:1.75.0-slim

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set up non-root builder user (Requirement 7.7)
RUN useradd -m -u 1000 -s /bin/bash builder && \
    mkdir -p /workspace && \
    chown -R builder:builder /workspace

USER builder
WORKDIR /workspace

# Pre-install Rust components specified in rust-toolchain.toml
# This layer will be cached unless rust-toolchain.toml changes
COPY --chown=builder:builder rust-toolchain.toml ./
RUN rustup show

# Pre-download dependencies for faster builds
# This layer will be cached unless Cargo.toml or Cargo.lock changes
COPY --chown=builder:builder Cargo.toml Cargo.lock ./
COPY --chown=builder:builder core/Cargo.toml ./core/
COPY --chown=builder:builder mobile/Cargo.toml ./mobile/
COPY --chown=builder:builder cli/Cargo.toml ./cli/
COPY --chown=builder:builder wasm/Cargo.toml ./wasm/

# Create dummy source files to allow dependency download
RUN mkdir -p core/src mobile/src cli/src wasm/src && \
    echo "fn main() {}" > core/src/lib.rs && \
    echo "fn main() {}" > mobile/src/lib.rs && \
    echo "fn main() {}" > cli/src/main.rs && \
    echo "fn main() {}" > wasm/src/lib.rs && \
    cargo fetch --locked

# Build stage: Copy actual source code and build
# This layer will be rebuilt when source code changes
COPY --chown=builder:builder . .

# Build release binary for CLI
RUN cargo build --release --bin scmessenger-cli --locked

# Verify build output
RUN test -f target/release/scmessenger-cli && \
    echo "Build successful: $(ls -lh target/release/scmessenger-cli)"

# Default command: run tests
CMD ["cargo", "test", "--workspace", "--locked"]
