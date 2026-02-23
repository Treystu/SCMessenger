#!/bin/bash
# Quick start script for running SCMessenger Docker tests

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}SCMessenger Docker Test Runner${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}Error: Docker is not running${NC}"
    exit 1
fi

# Parse arguments
MODE="extended"
DETACH=""
PROFILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --basic)
            MODE="basic"
            shift
            ;;
        --extended)
            MODE="extended"
            shift
            ;;
        --test)
            MODE="test"
            PROFILE="--profile test"
            shift
            ;;
        --detach|-d)
            DETACH="-d"
            shift
            ;;
        --clean)
            echo -e "${YELLOW}Cleaning up containers and volumes...${NC}"
            docker compose down -v 2>/dev/null || true
            docker compose -f docker-compose-extended.yml down -v 2>/dev/null || true
            echo -e "${GREEN}Cleanup complete${NC}"
            exit 0
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --basic       Start basic 3-node setup (default: extended)"
            echo "  --extended    Start extended 7-node setup"
            echo "  --test        Run automated integration tests"
            echo "  --detach, -d  Run in detached mode"
            echo "  --clean       Stop and remove all containers and volumes"
            echo "  --help, -h    Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 --extended          # Start 7-node environment"
            echo "  $0 --test              # Run full test suite"
            echo "  $0 --basic --detach    # Start basic setup in background"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Build and start based on mode
case $MODE in
    basic)
        echo -e "${YELLOW}Starting basic 3-node setup...${NC}"
        docker compose up --build $DETACH
        ;;
    extended)
        echo -e "${YELLOW}Starting extended 7-node setup...${NC}"
        docker compose -f docker-compose-extended.yml up --build $DETACH
        ;;
    test)
        echo -e "${YELLOW}Building containers...${NC}"
        docker compose -f docker-compose-extended.yml build

        echo -e "${YELLOW}Starting test environment...${NC}"
        docker compose -f docker-compose-extended.yml up -d

        echo -e "${YELLOW}Waiting for nodes to initialize...${NC}"
        sleep 20

        echo -e "${YELLOW}Running integration tests...${NC}"
        docker compose -f docker-compose-extended.yml run --rm test-runner

        echo ""
        echo -e "${GREEN}Test results saved to: test-results/${NC}"

        # Show summary
        LATEST_SUMMARY=$(ls -t test-results/summary_*.json 2>/dev/null | head -1)
        if [ -n "$LATEST_SUMMARY" ]; then
            echo -e "${GREEN}Test Summary:${NC}"
            cat "$LATEST_SUMMARY" | jq '.'
        fi

        # Cleanup
        echo -e "${YELLOW}Stopping containers...${NC}"
        docker compose -f docker-compose-extended.yml down
        ;;
esac

if [ -z "$DETACH" ] && [ "$MODE" != "test" ]; then
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Environment stopped${NC}"
    echo -e "${GREEN}========================================${NC}"
fi
