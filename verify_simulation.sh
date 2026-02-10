#!/bin/bash
set -e

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PREF_FILE=".docker_pref"
INSTALLED_THIS_SESSION=false

# Function to check if Docker daemon is running
check_docker_running() {
    if docker info > /dev/null 2>&1; then
        return 0
    fi
    return 1
}

# Function to install Docker on macOS
install_docker_macos() {
    echo -e "${YELLOW}Docker not found. Installing intelligently...${NC}"
    
    # Check for Homebrew
    if ! command -v brew &> /dev/null; then
        echo "Homebrew not found. Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    
    echo "Installing Docker Desktop via Homebrew..."
    brew install --cask docker
    
    echo "Starting Docker Desktop..."
    open -a Docker
    
    echo -e "${YELLOW}Waiting for Docker Engine to start...${NC}"
    echo "NOTE: You may need to interact with the Docker Desktop window to accept terms or grant permissions."
    
    # Wait for Docker to be ready
    local retries=0
    while ! docker info > /dev/null 2>&1; do
        sleep 5
        echo -n "."
        retries=$((retries + 1))
        if [ $retries -gt 60 ]; then # Wait up to 5 minutes
            echo -e "\n${RED}Timed out waiting for Docker. Please ensure Docker Desktop is running.${NC}"
            exit 1
        fi
    done
    echo -e "\n${GREEN}Docker is running!${NC}"
}

# --- Docker Detection & Installation ---

if ! command -v docker &> /dev/null; then
    # Check OS
    if [[ "$OSTYPE" == "darwin"* ]]; then
        install_docker_macos
        INSTALLED_THIS_SESSION=true
    else
        echo -e "${RED}Docker is not installed and automatic installation is only supported on macOS.${NC}"
        echo "Please install Docker manually and re-run this script."
        exit 1
    fi
else
    # Docker binary exists, check if daemon is running
    if ! check_docker_running; then
        echo -e "${YELLOW}Docker is installed but not running.${NC}"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo "Attempting to start Docker Desktop..."
            open -a Docker || true
            
            echo "Waiting for Docker Engine to start..."
            while ! docker info > /dev/null 2>&1; do
                sleep 2
                echo -n "."
            done
            echo -e "\n${GREEN}Docker started.${NC}"
        else
            echo "Please start the Docker daemon and re-run this script."
            exit 1
        fi
    fi
fi

# --- Simulation Logic ---

echo -e "${GREEN}🚀 Building Docker images...${NC}"
docker compose -f docker/docker-compose.yml build

echo "---------------------------------------------------"
echo -e "${GREEN}🌐 Starting Network Simulation (Relay + Alice + Bob)${NC}"
echo "---------------------------------------------------"
docker compose -f docker/docker-compose.yml up -d

echo "⏳ Waiting for nodes to initialize and discover each other (15s)..."
sleep 15

# Get Peer IDs
echo "📋 Retrieving Peer IDs..."

# Helper function to get ID with retry
get_peer_id() {
    local container=$1
    local id
    # Try multiple times to get ID in case service is slow
    for i in {1..3}; do
        id=$(docker exec $container scm identity show 2>/dev/null | grep "ID:" | awk '{print $2}')
        if [ ! -z "$id" ]; then
            echo "$id"
            return
        fi
        sleep 2
    done
}

ALICE_ID=$(get_peer_id scm-alice)
BOB_ID=$(get_peer_id scm-bob)

echo "👤 Alice ID: $ALICE_ID"
echo "👤 Bob ID:   $BOB_ID"

if [ -z "$ALICE_ID" ] || [ -z "$BOB_ID" ]; then
    echo -e "${RED}✗ Failed to retrieve Peer IDs. Check container logs.${NC}"
    docker compose -f docker/docker-compose.yml logs
    exit 1
fi

echo "---------------------------------------------------"
echo "📨 Test 1: Alice -> Bob (Message Send)"
echo "---------------------------------------------------"

# Add Bob as contact
docker exec scm-alice scm contact add "$BOB_ID" "test-key-placeholder" --name Bob > /dev/null 2>&1 || true

# Send message
MESSAGE="Hello from Alice $(date +%s)"
echo "Sending: '$MESSAGE'"
docker exec scm-alice scm send "$BOB_ID" "$MESSAGE"

echo "⏳ Waiting for message delivery (5s)..."
sleep 5

# Check Bob's history
echo "---------------------------------------------------"
echo "📥 Test 1: Verifying Receipt on Bob"
echo "---------------------------------------------------"
BOB_HISTORY=$(docker exec scm-bob scm history --limit 5)

if echo "$BOB_HISTORY" | grep -q "$MESSAGE"; then
    echo -e "${GREEN}✓ Message received successfully!${NC}"
else
    echo -e "${RED}✗ Message not found in Bob's history${NC}"
    echo "Bob's History:"
    echo "$BOB_HISTORY"
    docker compose -f docker/docker-compose.yml logs
    exit 1
fi

echo "---------------------------------------------------"
echo -e "${GREEN}✅ Simulation Verified Successfully${NC}"
echo "---------------------------------------------------"

# --- Cleanup Logic ---

if [ "$INSTALLED_THIS_SESSION" = true ]; then
    echo ""
    echo -e "${YELLOW}Docker was installed specifically for this simulation.${NC}"
    
    ACTION=""
    if [ -f "$PREF_FILE" ]; then
        ACTION=$(cat "$PREF_FILE")
    fi
    
    if [ -z "$ACTION" ]; then
        echo "What would you like to do with Docker?"
        echo "1) Remove it (clean up)"
        echo "2) Keep it"
        echo "3) Remove it (and remember this choice)"
        echo "4) Keep it (and remember this choice)"
        read -p "Select an option [1-4]: " choice
        
        case $choice in
            1) ACTION="remove" ;;
            2) ACTION="keep" ;;
            3) ACTION="remove"; echo "remove" > "$PREF_FILE" ;;
            4) ACTION="keep"; echo "keep" > "$PREF_FILE" ;;
            *) ACTION="keep" ;; # Default
        esac
    fi
    
    if [ "$ACTION" = "remove" ]; then
        echo "Tearing down containers..."
        docker compose -f docker/docker-compose.yml down
        echo "Uninstalling Docker..."
        brew uninstall --cask docker
        echo -e "${GREEN}Docker removed.${NC}"
    else
        echo -e "${GREEN}Docker kept installed.${NC}" 
        echo "To tear down the simulation manually:"
        echo "  docker compose -f docker/docker-compose.yml down"
    fi
else
    echo "To tear down the simulation:"
    echo "  docker compose -f docker/docker-compose.yml down"
fi
