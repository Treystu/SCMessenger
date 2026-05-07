# Docker Build Environment

This directory contains Docker configurations for reproducible Linux builds of SCMessenger.

## Purpose

The Docker build environment ensures:
- **Consistent build environment** across different machines and CI systems
- **Reproducible builds** with pinned Rust toolchain (1.75.0) and dependencies
- **Isolated builds** that don't depend on host system configuration
- **Build verification** for release artifacts

## Requirements

- Docker installed and running
- At least 4GB of available disk space
- Internet connection for initial image build

## Quick Start

### Build the Docker Image

```bash
./docker/build.sh build-image
```

This creates a Docker image with:
- Rust 1.75.0 (pinned via rust-toolchain.toml)
- Build dependencies (build-essential, pkg-config, libssl-dev)
- Pre-downloaded Cargo dependencies (cached layer)
- Non-root builder user

### Build CLI Binary

```bash
./docker/build.sh build-cli
```

This builds the `scmessenger-cli` binary in the Docker container and outputs it to `target/docker-release/scmessenger-cli`.

### Run Tests

```bash
./docker/build.sh test
```

This runs the full test suite in the Docker container.

### Clean Up

```bash
./docker/build.sh clean
```

This removes the Docker image and build artifacts.

## Manual Docker Commands

If you prefer to use Docker directly:

### Build Image

```bash
docker build -f docker/build.Dockerfile -t scmessenger-builder:1.75.0 .
```

### Build CLI

```bash
docker run --rm \
  -v $(pwd)/target/docker-release:/workspace/target/release \
  scmessenger-builder:1.75.0 \
  cargo build --release --bin scmessenger-cli --locked
```

### Run Tests

```bash
docker run --rm scmessenger-builder:1.75.0 cargo test --workspace --locked
```

### Interactive Shell

```bash
docker run --rm -it scmessenger-builder:1.75.0 /bin/bash
```

## Build Reproducibility

The Docker environment ensures reproducible builds by:

1. **Pinned Rust version**: Uses `rust:1.75.0-slim` base image
2. **Pinned dependencies**: Uses `Cargo.lock` with `--locked` flag
3. **Consistent system libraries**: Debian slim with specific package versions
4. **Isolated environment**: No host system dependencies
5. **Non-root user**: Builds run as user `builder` (UID 1000)

## Caching Strategy

The Dockerfile uses multi-layer caching to speed up builds:

1. **Base layer**: Rust toolchain and system dependencies (rarely changes)
2. **Dependency layer**: Cargo dependencies (changes when Cargo.toml/Cargo.lock changes)
3. **Source layer**: Application source code (changes frequently)

This means:
- First build: ~10-15 minutes (downloads everything)
- Subsequent builds with code changes only: ~2-3 minutes
- Subsequent builds with dependency changes: ~5-7 minutes

## CI/CD Integration

This Docker environment can be used in GitHub Actions:

```yaml
- name: Build with Docker
  run: |
    docker build -f docker/build.Dockerfile -t scmessenger-builder:1.75.0 .
    docker run --rm \
      -v ${{ github.workspace }}/target/docker-release:/workspace/target/release \
      scmessenger-builder:1.75.0 \
      cargo build --release --bin scmessenger-cli --locked
```

## Troubleshooting

### Build fails with "permission denied"

Make sure the builder user has correct permissions:
```bash
docker run --rm -it scmessenger-builder:1.75.0 /bin/bash
# Inside container:
whoami  # Should be "builder"
ls -la /workspace
```

### Image build is slow

The first build downloads all dependencies. Subsequent builds use Docker's layer cache. To force a clean build:
```bash
docker build --no-cache -f docker/build.Dockerfile -t scmessenger-builder:1.75.0 .
```

### Out of disk space

Clean up old Docker images and containers:
```bash
docker system prune -a
```

## Security Considerations

- The Docker image runs as non-root user `builder` (UID 1000)
- No secrets or credentials are included in the image
- The image is based on official Rust slim image from Docker Hub
- System packages are from Debian stable repositories

## Related Documentation

- [Build Reproducibility Requirements](../docs/ARCHITECTURE.md#build-reproducibility)
- [Release Pipeline](../.github/workflows/release.yml)
- [Deployment Guide](../docs/DEPLOYMENT.md)
