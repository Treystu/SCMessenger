# Build stage
FROM rust:latest AS builder

# Optional: Override bootstrap nodes at build time
# Format: comma-separated multiaddrs
# Example: /ip4/1.2.3.4/tcp/9001/p2p/12D3Koo...,/ip4/5.6.7.8/tcp/9001/p2p/12D3Koo...
ARG SCMESSENGER_BOOTSTRAP_NODES=""
ENV SCMESSENGER_BOOTSTRAP_NODES=${SCMESSENGER_BOOTSTRAP_NODES}

WORKDIR /usr/src/scmessenger

# Copy manifests (must include every workspace member from the root
# Cargo.toml: core, cli, wasm, mobile, desktop_bridge - a missing member's
# Cargo.toml makes the workspace build fail with "failed to read
# <member>/Cargo.toml").
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml core/
COPY cli/Cargo.toml cli/
COPY mobile/Cargo.toml mobile/
COPY wasm/Cargo.toml wasm/
COPY desktop_bridge/Cargo.toml desktop_bridge/

# Copy source code
COPY core/src core/src
COPY cli/src cli/src
COPY mobile/src mobile/src
COPY wasm/src wasm/src
COPY desktop_bridge/src desktop_bridge/src

# Copy build scripts
COPY core/build.rs core/
COPY mobile/build.rs mobile/
COPY desktop_bridge/build.rs desktop_bridge/

# Build the CLI
RUN cargo build --release --bin scmessenger-cli

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies and network tools
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    iproute2 \
    iputils-ping \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/scmessenger/target/release/scmessenger-cli /usr/local/bin/scm

# Copy entrypoint script
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Environment variables
ENV RUST_LOG=info
ENV SCM_CONFIG_DIR=/root/.config/scmessenger
ENV SCM_DATA_DIR=/root/.local/share/scmessenger
ENV LISTEN_PORT=9000

# Expose ports
EXPOSE 9000 9001

ENTRYPOINT ["entrypoint.sh"]
CMD ["scm", "start"]
