#!/bin/bash
# Example: Custom Test Scenario
# Demonstrates how to use the Docker test infrastructure for custom testing scenarios

set -e

cd "$(dirname "$0")"

echo "========================================="
echo "Custom Test Scenario Example"
echo "========================================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Step 1: Start mock infrastructure
log_step "1. Starting mock infrastructure (relay + 2 clients)..."
docker compose -f docker-compose.test.yml --profile test up -d mock-relay mock-client-a mock-client-b

# Step 2: Wait for infrastructure to be ready
log_step "2. Waiting for infrastructure to initialize..."
sleep 10

# Check relay health
for i in {1..30}; do
    if docker compose -f docker-compose.test.yml ps mock-relay | grep -q "healthy"; then
        log_info "Mock relay is healthy"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "ERROR: Mock relay failed to become healthy"
        docker compose -f docker-compose.test.yml logs mock-relay
        exit 1
    fi
    sleep 2
done

# Step 3: Get peer IDs from both clients
log_step "3. Retrieving peer IDs..."

echo "Client A identity:"
docker exec scm-mock-client-a scm identity show || true

echo ""
echo "Client B identity:"
docker exec scm-mock-client-b scm identity show || true

# Step 4: Show connected peers
log_step "4. Checking peer connections..."
sleep 5

echo "Client A peers:"
docker exec scm-mock-client-a scm peers list || true

echo ""
echo "Client B peers:"
docker exec scm-mock-client-b scm peers list || true

# Step 5: Demonstrate message sending (if CLI supports it)
log_step "5. Testing message delivery..."
echo "NOTE: Actual message sending depends on CLI implementation"
echo "This is where you would test your custom scenarios"
echo ""
echo "Example commands to run manually:"
echo "  docker exec -it scm-mock-client-a scm send <peer-id> 'Hello!'"
echo "  docker exec -it scm-mock-client-b scm history"

# Step 6: View logs
log_step "6. Viewing recent logs (Ctrl+C to stop)..."
echo "Press Ctrl+C after reviewing logs..."
sleep 2
docker compose -f docker-compose.test.yml logs --tail=30 mock-relay mock-client-a mock-client-b || true

# Cleanup prompt
echo ""
echo "========================================="
echo "Test scenario complete!"
echo "========================================="
echo ""
echo "Mock infrastructure is still running."
echo "Options:"
echo "  1. Access client A: docker exec -it scm-mock-client-a /bin/bash"
echo "  2. Access client B: docker exec -it scm-mock-client-b /bin/bash"
echo "  3. View relay logs: docker compose -f docker-compose.test.yml logs -f mock-relay"
echo "  4. Stop all: docker compose -f docker-compose.test.yml down"
echo ""

read -p "Stop infrastructure now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    log_info "Stopping mock infrastructure..."
    docker compose -f docker-compose.test.yml down
    log_info "Done!"
else
    log_info "Infrastructure still running. Use 'docker compose -f docker-compose.test.yml down' to stop."
fi
