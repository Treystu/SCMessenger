#!/bin/bash
set -e

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}1. Submitting build to GCP Cloud Build...${NC}"
gcloud builds submit --config cloudbuild.yaml .
echo -e "${GREEN}✓ Build successful and image pushed to Artifact Registry.${NC}"

echo -e "${BLUE}2. Updating the container on GCP Compute Instance...${NC}"
# Stop and remove existing container
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a --tunnel-through-iap --command="
    sudo docker stop scmessenger-relay 2>/dev/null || true
    sudo docker rm scmessenger-relay 2>/dev/null || true
" 2>&1 | grep -v "^WARNING:" | grep -v "^bash:" || true

# Start new container with correct command
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a --tunnel-through-iap --command="
    sudo docker run -d --restart=unless-stopped \
        --name scmessenger-relay \
        -p 9001:9001 \
        -p 9000:9000 \
        us-central1-docker.pkg.dev/scmessenger-bootstrapnode/scmessenger-repo/scmessenger-cli:latest \
        scm relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000 --name GCP-headless
" 2>&1 | grep -v "^WARNING:" | grep -v "^bash:" || true
echo -e "${GREEN}✓ Container updated on the VM.${NC}"

echo -e "${BLUE}3. Verifying container is running...${NC}"
sleep 3
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a --tunnel-through-iap --command="sudo docker ps --filter name=scmessenger-relay --format 'table {{.ID}}\t{{.Status}}\t{{.Ports}}'" 2>&1 | grep -v "^WARNING:" | grep -v "^bash:" || true

echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  GCP Relay Node Updated successfully!                    ${NC}"
echo -e "${GREEN}  Container: scmessenger-relay                            ${NC}"
echo -e "${GREEN}  Running: scm relay --listen /ip4/0.0.0.0/tcp/9001      ${NC}"
echo -e "${GREEN}  Ports: 9000 (HTTP), 9001 (libp2p)                      ${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
