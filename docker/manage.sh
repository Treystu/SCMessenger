#!/bin/bash
set -e

# Configuration
IMAGE_NAME="scmessenger"
DEFAULT_PORT=8080

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

function log_info {
    echo -e "${GREEN}[INFO]${NC} $1"
}

function log_warn {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

function check_docker {
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}Error: docker is not installed.${NC}"
        exit 1
    fi
}

function build {
    log_info "Building Docker image '$IMAGE_NAME'..."
    docker build -f docker/Dockerfile -t $IMAGE_NAME .
    log_info "Build complete."
}

function run_local {
    check_docker
    PORT=${2:-$DEFAULT_PORT}
    
    log_info "Running '$IMAGE_NAME' locally on port $PORT..."
    log_info "Access the UI at http://localhost:$PORT"
    log_info "Press Ctrl+C to stop."
    
    # Run interactive so user can see logs and stop it easily
    docker run -it --rm \
        -p $PORT:$PORT \
        -e PORT=$PORT \
        -e RUST_LOG=info \
        $IMAGE_NAME
}

function simulate {
    log_info "Running full simulation verification..."
    ./verify_simulation.sh
}

function deploy {
    log_info "Starting GCP Deployment..."
    
    # Check for gcloud
    if ! command -v gcloud &> /dev/null; then
        echo -e "${RED}Error: gcloud SDK is not installed.${NC}"
        echo "Please install it: https://cloud.google.com/sdk/docs/install"
        exit 1
    fi

    # Get Project ID
    PROJECT_ID=$(gcloud config get-value project 2>/dev/null)
    if [ -z "$PROJECT_ID" ]; then
        echo -e "${RED}Error: No GCP project configured.${NC}"
        echo "Run: gcloud auth login && gcloud config set project [PROJECT_ID]"
        exit 1
    fi

    REGION="us-central1"
    REPO="scmessenger-repo"
    IMAGE_TAG="$REGION-docker.pkg.dev/$PROJECT_ID/$REPO/$IMAGE_NAME:latest"

    # Confirm
    echo -e "Project: ${GREEN}$PROJECT_ID${NC}"
    echo -e "Region:  ${GREEN}$REGION${NC}"
    echo -e "Image:   ${GREEN}$IMAGE_TAG${NC}"
    echo ""
    read -p "Continue with deployment? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi

    # 1. Ensure Repository Exists (idempotent-ish check/create)
    log_info "Ensuring Artifact Registry repository exists..."
    if ! gcloud artifacts repositories describe $REPO --location=$REGION &>/dev/null; then
        log_warn "Creating repository '$REPO'..."
        gcloud artifacts repositories create $REPO \
            --repository-format=docker \
            --location=$REGION \
            --description="SCMessenger Docker Repository" || true
    fi

    # 2. Configure Docker Auth
    gcloud auth configure-docker $REGION-docker.pkg.dev --quiet

    # 3. Build & Push
    log_info "Building specific platform image (linux/amd64)..."
    docker build --platform linux/amd64 -f docker/Dockerfile -t $IMAGE_TAG .
    
    log_info "Pushing image to GCP..."
    docker push $IMAGE_TAG

    # 4. Deploy
    log_info "Deploying to Cloud Run..."
    gcloud run deploy scmessenger-service \
        --image $IMAGE_TAG \
        --region $REGION \
        --platform managed \
        --allow-unauthenticated \
        --port 8080 \
        --session-affinity

    echo -e "${GREEN}Deployment Complete!${NC}"
}

# Main Dispatch
case "$1" in
    "build")
        build
        ;;
    "run")
        run_local "$@"
        ;;
    "simulate")
        simulate
        ;;
    "deploy")
        deploy
        ;;
    *)
        echo "SCMessenger Docker Manager"
        echo "Usage: ./docker/manage.sh [command]"
        echo ""
        echo "Commands:"
        echo "  build           Build the Docker image"
        echo "  run [port]      Run locally (default port 8080). Example: ./docker/manage.sh run 9000"
        echo "  simulate        Run the multi-node network simulation"
        echo "  deploy          Deploy to Google Cloud Run (requires gcloud configured)"
        exit 1
        ;;
esac
