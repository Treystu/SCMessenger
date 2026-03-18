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

# SCMessenger Network Testing Guide

This document explains how to use the comprehensive network testing tools for SCMessenger.

## [Needs Revalidation] Overview

SCMessenger includes three levels of testing scripts:

1. **`verify_simulation.sh`** - Core functionality and quick network validation (basic Docker network)
2. **`run_comprehensive_network_tests.sh`** - **NEW!** Enhanced testing with real network conditions (NAT, latency, packet loss)
3. **`test_network_scenarios.sh`** - Advanced network scenario testing with detailed pass/fail reporting

## [Needs Revalidation] 🆕 Enhanced Network Testing (Recommended)

### [Needs Revalidation] What's New

The enhanced testing suite simulates **real-world network conditions**:

- ✅ **NAT Simulation**: Cone NAT for Alice, Symmetric NAT for Bob
- ✅ **Bandwidth Limits**: 10 Mbps for Alice, 5 Mbps for Bob
- ✅ **Network Latency**: 50ms for Alice, 100ms for Bob
- ✅ **Packet Loss**: 2% packet loss for Bob
- ✅ **Network Isolation**: Separate networks with NAT gateways
- ✅ **Traffic Control**: Real `tc` and `iptables` rules

### [Needs Revalidation] Why This Matters

The basic Docker network (bridge mode) doesn't test:
- NAT hole punching (no actual NAT)
- Transport escalation (no network constraints)
- Circuit relay fallback (direct connections always work)
- Address reflection (no external addresses to observe)

The enhanced setup creates **realistic conditions** where these features are actually needed and tested.

## [Needs Revalidation] Prerequisites

- Docker (automatically installed on macOS if missing)
- At least 4GB of available RAM
- Internet connection for initial Docker image build

## [Needs Revalidation] Quick Start

### [Needs Revalidation] Option A: Enhanced Network Testing (Recommended)

Run the enhanced simulation with real network conditions:

```bash
./run_comprehensive_network_tests.sh
```

This will:
- ✅ Set up NAT gateways and network isolation
- ✅ Apply bandwidth limits and latency
- ✅ Configure traffic control rules
- ✅ Start all nodes with realistic constraints
- ✅ Run connectivity verification

**Duration:** ~60 seconds setup + testing

Then run the comprehensive scenarios:

```bash
./test_network_scenarios.sh
```

### [Needs Revalidation] Option B: Basic Simulation & Validation

Run the basic simulation (simpler, but less realistic):

```bash
./verify_simulation.sh
```

This script will:
- ✅ Build Docker images
- ✅ Start 3 nodes (Relay/Charlie, Alice, Bob)
- ✅ Verify instance isolation
- ✅ Test peer discovery
- ✅ Validate crypto operations
- ✅ Test end-to-end message delivery
- ✅ Check NAT traversal capabilities
- ✅ Analyze connection types
- ✅ Verify network resilience
- ✅ Test transport protocols
- ✅ Validate privacy features
- ✅ Check Drift protocol sync

**Duration:** ~30-45 seconds

### [Needs Revalidation] 2. Advanced Network Scenarios

Once the basic simulation is running, test advanced scenarios:

```bash
./test_network_scenarios.sh
```

This script tests:
- 🔄 Network partition and recovery
- 🌐 NAT hole punching and traversal
- 🔗 Circuit relay protocols
- 🕸️ Mesh routing and multi-hop forwarding
- 🚀 Transport protocol escalation
- 🔐 Privacy and onion routing
- 💾 Drift protocol offline delivery
- 📊 Performance metrics and error analysis

**Duration:** ~60-90 seconds

## [Needs Revalidation] Test Scenarios Explained

### [Needs Revalidation] Scenario 1: Network Partition Recovery
Tests mesh resilience when the relay node fails:
- Pauses the relay container
- Attempts to send messages during partition
- Verifies message queueing in outbox
- Restores relay and confirms message delivery

### [Needs Revalidation] Scenario 2: NAT Traversal & Address Discovery
Analyzes NAT behavior:
- Address observation protocol
- Hole punching attempts
- Connection type analysis (direct vs relayed)

### [Needs Revalidation] Scenario 3: Circuit Relay Protocol
Tests relay-based message forwarding:
- Relay node activity monitoring
- Circuit reservation tracking
- Message delivery through relay

### [Needs Revalidation] Scenario 4: Mesh Routing & Multi-hop
Tests mycorrhizal mesh routing:
- Routing table updates
- DHT operations
- Peer exchange protocol

### [Needs Revalidation] Scenario 5: Transport Layer Analysis
Tests protocol capabilities:
- TCP/QUIC/WebSocket usage
- Protocol escalation
- Transport optimization

### [Needs Revalidation] Scenario 6: Privacy Features
Tests privacy-preserving features:
- Onion routing circuits
- Cover traffic and padding
- Multi-hop encryption

### [Needs Revalidation] Scenario 7: Drift Protocol & Offline Support
Tests asynchronous message delivery:
- Offline message queueing
- Drift synchronization
- Store-and-forward reliability

### [Needs Revalidation] Scenario 8: Performance Metrics
Analyzes system performance:
- Message throughput (10 messages)
- Error rate analysis
- Performance classification

## [Needs Revalidation] Network Capabilities Tested

### [Needs Revalidation] Core Network Features
- ✅ **Address Observation** - Peers observe each other's external addresses
- ✅ **NAT Type Detection** - Cone vs Symmetric NAT identification
- ✅ **Hole Punching** - Direct connection attempts through NAT
- ✅ **Circuit Relay** - Fallback routing when direct fails
- ✅ **Peer Exchange** - Bootstrap & discovery mechanisms
- ✅ **Multi-hop Routing** - Mycorrhizal mesh routing

### [Needs Revalidation] Advanced Features
- ✅ **Transport Escalation** - Automatic protocol upgrades
- ✅ **Connection Resilience** - Exponential backoff & retry
- ✅ **Onion Routing** - Privacy-preserving multi-hop circuits
- ✅ **Drift Synchronization** - Efficient message sync protocol
- ✅ **Store-and-Forward** - Reliable offline message delivery
- ✅ **Network Partition Recovery** - Handles relay failures gracefully

## [Needs Revalidation] Network Topology

```
┌─────────────────┐         ┌─────────────────┐
│   Network A     │         │   Network B     │
│                 │         │                 │
│   ┌─────────┐   │         │   ┌─────────┐   │
│   │  Alice  │───┼────┐    │   │   Bob   │   │
│   └─────────┘   │    │    │   └─────────┘   │
│                 │    │    │        │         │
└─────────────────┘    │    └────────┼─────────┘
                       │             │
                  ┌────▼─────────────▼────┐
                  │   Charlie (Relay)     │
                  │  Bridges Networks     │
                  └───────────────────────┘
```

- **Alice**: Network A participant, NAT traversal capable
- **Bob**: Network B participant, address reflection active
- **Charlie (Relay)**: Bridges both networks, circuit relay provider

## [Needs Revalidation] Understanding Test Output

### [Needs Revalidation] Success Indicators
- ✅ **Green checkmarks** - Feature working as expected
- ℹ️ **Blue info** - Feature not needed in current scenario
- ⚠️ **Yellow warnings** - Non-critical issues or pending operations

### [Needs Revalidation] Failure Indicators
- ✗ **Red X marks** - Critical failures requiring attention

### [Needs Revalidation] Common Warnings (Non-critical)
- "No address observations detected" - Normal in local Docker networks
- "No onion routing detected" - Optional for small 3-node networks
- "Cover traffic disabled" - Often disabled in test mode
- "Message delivery pending" - May take extra time on slow systems

## [Needs Revalidation] Cleanup

Tear down the simulation:

```bash
docker compose -f docker/docker-compose.yml down
```

Remove all containers and networks:

```bash
docker compose -f docker/docker-compose.yml down -v
```

## [Needs Revalidation] Troubleshooting

### [Needs Revalidation] "Containers not running" Error
Run `verify_simulation.sh` first before running `test_network_scenarios.sh`.

### [Needs Revalidation] "Failed to retrieve node IDs"
Wait 5-10 seconds after starting containers, then retry.

### [Needs Revalidation] Port Conflicts
If port 4001 is in use, modify `docker/docker-compose.yml` to use different ports.

### [Needs Revalidation] Docker Permission Errors
On Linux, add your user to the docker group:
```bash
sudo usermod -aG docker $USER
newgrp docker
```

### [Needs Revalidation] Out of Memory
Ensure at least 4GB RAM is available. Close other applications if needed.

## [Needs Revalidation] Advanced Usage

### [Needs Revalidation] Run Specific Scenarios Only

Edit `test_network_scenarios.sh` and comment out unwanted scenario sections.

### [Needs Revalidation] Custom Network Topologies

Modify `docker/docker-compose.yml` to add more nodes or change network configurations.

### [Needs Revalidation] Extended Testing

Run the scenarios multiple times to test consistency:
```bash
for i in {1..5}; do
  echo "=== Test iteration $i ==="
  ./test_network_scenarios.sh
  sleep 10
done
```

### [Needs Revalidation] Continuous Monitoring

Watch logs in real-time while tests run:
```bash
# In one terminal
./test_network_scenarios.sh

# In another terminal
docker logs -f scm-relay
# or
docker logs -f scm-alice
# or
docker logs -f scm-bob
```

## [Needs Revalidation] Integration with CI/CD

Both scripts can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run Network Tests
  run: |
    ./verify_simulation.sh
    ./test_network_scenarios.sh
```

## [Needs Revalidation] Performance Benchmarks

Expected performance on modern hardware:

| Test | Duration | Messages | Success Rate |
|------|----------|----------|--------------|
| Basic Simulation | 30-45s | 1-2 | 100% |
| Advanced Scenarios | 60-90s | 15-20 | 95-100% |
| Total Suite | 90-135s | 16-22 | 95-100% |

## [Needs Revalidation] Contributing

When adding new network tests:

1. Add test scenario to `test_network_scenarios.sh`
2. Document the scenario in this guide
3. Update the summary section with the new capability
4. Test on multiple platforms (macOS, Linux)
5. Verify cleanup works correctly

## [Needs Revalidation] Related Documentation

- [CLAUDE.md](CLAUDE.md) - Project philosophy and architecture
- [docker/README.md](docker/README.md) - Docker setup details
- [core/README.md](core/README.md) - Core library documentation

## [Needs Revalidation] Support

For issues or questions:
1. Check the Troubleshooting section above
2. Review container logs: `docker compose -f docker/docker-compose.yml logs`
3. Open an issue on GitHub with logs and error messages

---

**Last Updated:** 2026-02-10
**SCMessenger Version:** 0.1.0
**Minimum Docker Version:** 20.10+
