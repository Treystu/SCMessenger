# Network Simulation Environment

This directory contains resources for simulating a multi-node SCMessenger network using Docker. This setup allows for testing network routing, relay functionality, and connectivity across simulated NAT boundaries.

## Architecture

The simulation consists of three nodes:

1.  **Relay Node (`scm-relay`)**:
    - Acts as a bootstrap node and relay server.
    - Exposes port `4001` to the simulated network.
    - Sits on both `network-a` and `network-b`.

2.  **Alice (`scm-alice`)**:
    - Represents a client behind "NAT A" (`network-a`).
    - Can only communicate with Bob via the Relay (initially) or hole punching.

3.  **Bob (`scm-bob`)**:
    - Represents a client behind "NAT B" (`network-b`).
    - Can only communicate with Alice via the Relay (initially) or hole punching.

## Prerequisites

- Docker
- Docker Compose

## Usage

### 1. Start the Environment

```bash
docker compose up -d --build
```

### 2. Verify Connectivity (Automated)

Run the verification script from the root of the repository:

```bash
./verify_simulation.sh
```

### 3. Manual Verification

**Check Status of Alice:**

```bash
docker exec -it scm-alice scm status
```

**Check Status of Bob:**

```bash
docker exec -it scm-bob scm status
```

**Send a Message (Alice -> Bob):**

1.  Get Bob's Peer ID:
    ```bash
    docker exec scm-bob scm identity show
    ```
2.  Send message from Alice:
    ```bash
    docker exec scm-alice scm send <BOB_PEER_ID> "Hello World"
    ```
3.  Check Bob's history:
    ```bash
    docker exec scm-bob scm history
    ```

## Development

- **Modify the App**: Re-run `docker-compose build` to include changes from the host source code.
- **Logs**: Use `docker-compose logs -f` to follow logs from all nodes.
