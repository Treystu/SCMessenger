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
gcloud compute instances update-container scmessenger-bootstrap \
    --zone=us-central1-a \
    --container-image=us-central1-docker.pkg.dev/scmessenger-bootstrapnode/scmessenger-repo/scmessenger-cli:latest \
    --container-arg="relay" \
    --container-arg="--listen" \
    --container-arg="/ip4/0.0.0.0/tcp/9001" \
    --container-arg="--http-port" \
    --container-arg="9000" \
    --container-arg="--name" \
    --container-arg="gcp-bootstrap-1"
echo -e "${GREEN}✓ Container updated on the VM.${NC}"

echo -e "${BLUE}3. Restarting the VM to ensure the newly built image is pulled...${NC}"
gcloud compute instances stop scmessenger-bootstrap --zone=us-central1-a
gcloud compute instances start scmessenger-bootstrap --zone=us-central1-a

echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  GCP Relay Node Updated successfully!                    ${NC}"
echo -e "${GREEN}  Running: scm relay --listen /ip4/0.0.0.0/tcp/9001      ${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
