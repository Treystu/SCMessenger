# SCMessenger Docker Testing Environment

Comprehensive Docker-based testing infrastructure for verifying all SCMessenger features in a simulated multi-network environment.

## Quick Start

### Run All Tests
```bash
./run-all-tests.sh
```

### Run Specific Tests
```bash
./run-all-tests.sh --rust-only      # Rust core tests
./run-all-tests.sh --android-only   # Android unit tests
./run-all-tests.sh --integration-only # Integration tests
```

**📖 For detailed testing guide, see [TESTING_GUIDE.md](TESTING_GUIDE.md)**

## Overview

This directory contains Docker configurations and test scripts to validate SCMessenger's core functionality including:

- **Unit Tests**: Rust core library and Android app with MockK
- **Integration Tests**: Multi-node mesh networking with real containers
- **Mock Infrastructure**: Configurable NAT, latency, and network simulation
- Peer-to-peer messaging
- Relay-based routing
- Multi-hop relay chains
- DHT/Kademlia peer discovery
- Mesh network formation
- NAT traversal simulation
- Message delivery tracking
- Network partition resilience

## Architecture

### Network Topology

```
Network-A (172.20.0.0/24)          Network-B (172.21.0.0/24)          Network-C (172.22.0.0/24)
┌──────────────────┐               ┌──────────────────┐               ┌──────────────────┐
│                  │               │                  │               │                  │
│  Alice           │               │  Bob             │               │  Eve             │
│  Carol           │               │  David           │               │                  │
│                  │               │                  │               │                  │
│       ╲          │               │          ╱       │               │                  │
│        ╲         │               │         ╱        │               │                  │
│         ╲        │               │        ╱         │               │         ╱        │
│          Relay1──┼───────────────┼───────Relay2─────┼───────────────┼────────╱         │
│                  │               │                  │               │                  │
└──────────────────┘               └──────────────────┘               └──────────────────┘

Direct P2P:     Alice ↔ Carol (same network)
Single Relay:   Alice ↔ Bob (via Relay1)
Multi-hop:      Alice ↔ Eve (via Relay1 → Relay2)
```

### Node Configuration

| Node    | Network(s)      | Role                    | Port |
|---------|-----------------|-------------------------|------|
| relay1  | A, B            | Primary bootstrap relay | 4001 |
| relay2  | B, C            | Secondary relay         | 4002 |
| alice   | A               | Client node             | -    |
| bob     | B               | Client node             | -    |
| carol   | A               | Client node             | -    |
| david   | B               | Client node             | -    |
| eve     | C               | Client node             | -    |

## Files

### Docker Images
- `Dockerfile` - Multi-stage build for SCMessenger CLI (production)
- `Dockerfile.android-test` - Android test environment with SDK, NDK, Rust
- `Dockerfile.rust-test` - Rust test environment with all tooling

### Docker Compose Configurations
- `docker-compose.yml` - Basic 3-node setup (relay, alice, bob)
- `docker-compose-extended.yml` - Full 7-node testing environment
- `docker-compose.test.yml` - **NEW: Comprehensive test infrastructure**
- `docker-compose.network-test.yml` - Network simulation with NAT

### Scripts
- `run-all-tests.sh` - **NEW: Main test runner for all test suites**
- `run-tests.sh` - Quick start script for Docker environments
- `setup-android-test-mocks.sh` - **NEW: Set up Android test mocks**
- `entrypoint.sh` - Container initialization script
- `manage.sh` - Docker management utilities

### Documentation
- `README.md` - This file
- `TESTING_GUIDE.md` - **NEW: Comprehensive testing guide**
- `test-scripts/` - Integration test scripts
- `test-results/` - Output directory for test logs and results

## Usage

### Basic Setup (3 nodes)

Start the basic environment with one relay and two clients:

```bash
cd docker
docker-compose up --build
```

This creates:
- 1 relay node (bridges network-a and network-b)
- Alice on network-a
- Bob on network-b

### Extended Setup (7 nodes)

Start the full testing environment:

```bash
cd docker
docker-compose -f docker-compose-extended.yml up --build
```

This creates:
- 2 relay nodes (relay1, relay2)
- 5 client nodes across 3 networks
- Full mesh topology for comprehensive testing

### Running Automated Tests

Execute the full integration test suite:

```bash
cd docker
docker-compose -f docker-compose-extended.yml --profile test up --build
```

The test runner will:
1. Wait for all nodes to initialize
2. Verify network connectivity
3. Test direct P2P messaging
4. Test single-relay routing
5. Test multi-hop relay chains
6. Verify DHT peer discovery
7. Test bidirectional messaging
8. Verify mesh routing
9. Check connection stability
10. Validate relay load distribution

Test results are saved to `test-results/` with timestamps.

### Manual Testing

Access individual node shells:

```bash
# Alice's shell
docker exec -it scm-alice /bin/bash

# Inside container, use CLI commands
scm identity show
scm peers list
scm send <peer-id> "Hello!"
scm history
```

### Viewing Logs

Monitor logs from all nodes:

```bash
docker-compose -f docker-compose-extended.yml logs -f
```

View logs from specific node:

```bash
docker logs -f scm-alice
```

### Stopping Services

Stop all containers:

```bash
docker-compose -f docker-compose-extended.yml down
```

Clean up volumes and networks:

```bash
docker-compose -f docker-compose-extended.yml down -v
```

## Test Suite Details

### Test 1: Relay Node Operational Status
Verifies both relay nodes start successfully and listen on configured ports.

### Test 2: Client Node Connectivity
Ensures all 5 client nodes can connect to the network and establish libp2p swarms.

### Test 3: Identity Creation
Validates that each node successfully generates Ed25519 keypairs and peer IDs.

### Test 4: Direct P2P Messaging
Tests message delivery between two nodes on the same network (Alice → Carol).

### Test 5: Single-Relay Routing
Tests message delivery across networks via one relay (Alice → Bob).

### Test 6: Multi-Hop Relay
Tests message delivery requiring multiple relay hops (Alice → Eve).

### Test 7: DHT Peer Discovery
Validates Kademlia DHT functionality and peer discovery.

### Test 8: Bidirectional Messaging
Tests message delivery in both directions (Bob → Alice).

### Test 9: Mesh Routing
Tests messaging between nodes on the same network (Bob ↔ David).

### Test 10: Message Queueing
Simulates network delays and verifies message queuing.

### Test 11: Connection Stability
Monitors persistent connections over time.

### Test 12: Relay Load Distribution
Verifies both relay nodes are active and handling traffic.

## Troubleshooting

### Nodes fail to connect

Check network configuration:
```bash
docker network ls
docker network inspect docker_network-a
```

### Messages not delivered

Check logs for routing errors:
```bash
docker logs scm-alice | grep -i "routing\|relay\|send"
```

### Tests fail

View detailed test logs:
```bash
cat docker/test-results/test_run_*.log
```

## Development

- **Modify the App**: Re-run `docker-compose build` to include changes from the host source code.
- **Logs**: Use `docker-compose logs -f` to follow logs from all nodes.
