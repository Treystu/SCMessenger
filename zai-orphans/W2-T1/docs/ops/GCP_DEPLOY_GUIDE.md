> **Component Status Notice (2026-03-16)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

Status: Current
Last updated: 2026-07-25

## [Current] Deployed Node Record

A GCP-hosted node is an ordinary SCMessenger node that happens to have a public
address. It is not a relay role or a bootstrap role -- every node is a full relay.
Record your own deployment's values here; they are deployment-specific and must
not be copied from documentation into any client or build.

- **VM**: `<VM_NAME>` in `<GCP_ZONE>` (e.g. `us-central1-a`)
- **Container**: `<CONTAINER_NAME>` (ports 9000-9001 exposed)
- **Image**: `<REGION>-docker.pkg.dev/<GCP_PROJECT>/<REPO>/scmessenger-cli:latest`
- **Peer ID**: `<PEER_ID>` -- read with `docker exec <CONTAINER_NAME> scm identity`
- **External IP**: `<NODE_IP>` -- prefer a reserved static IP; an ephemeral IP
  changes on stop/start and invalidates every address already shared with peers

Its multiaddr, for clients that need a cold-start entry point, is
`/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>`, supplied via `SC_BOOTSTRAP_NODES` (the
only variable name the code reads). See `docs/BOOTSTRAP.md` and
`docs/RELAY_OPERATOR_GUIDE.md`.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

# Deploying SCMessenger to Google Cloud Platform (Cloud Run)

This guide explains how to deploy SCMessenger as a scalable, serverless container service on GCP Cloud Run.

## [Needs Revalidation] [WARNING] Important: State Persistence

By default, **Cloud Run is stateless**. When your container restarts or scales down, **all local data (Identity, Contacts, Message History)** will be lost because SCMessenger uses an embedded database (`sled`) stored in the container.

**For Testing**: This is fine. You will get a fresh identity every time you deploy.
**For Production**: You must mount a persistent volume (Cloud Storage FUSE or NFS) or refactor the app to use an external database (which is outside the scope of this quick guide).

---

## [Needs Revalidation] 1. Prerequisites

- Google Cloud SDK (`gcloud`) installed and logged in.
- A GCP Project created.

## [Needs Revalidation] 2. One-Time Setup

Run these commands once to configure your environment.

```bash
# 1. Login
gcloud auth login

# 2. Set your project ID
export PROJECT_ID="your-project-id"  # <--- REPLACE THIS
gcloud config set project $PROJECT_ID

# 3. Enable required APIs
gcloud services enable artifactregistry.googleapis.com run.googleapis.com

# 4. Create an Artifact Registry Repository (to store your Docker images)
gcloud artifacts repositories create scmessenger-repo \
    --repository-format=docker \
    --location=us-central1 \
    --description="SCMessenger Docker Repository"

# 5. Configure Docker to authenticate with GCP
gcloud auth configure-docker us-central1-docker.pkg.dev
```

## [Current] 3. Build & Deploy

You can run this sequence every time you want to update the GCP node. (The
resource names below -- `scmessenger-bootstrap`, `scmessenger-relay` -- are just
the existing VM and container names; they do not denote node roles.)

```bash
# Option 1: Use the deploy script (recommended)
./scripts/deploy_gcp_node.sh

# Option 2: Manual steps
# Variables
export PROJECT_ID=$(gcloud config get-value project)
export REGION="us-central1"
export IMAGE_TAG="us-central1-docker.pkg.dev/$PROJECT_ID/scmessenger-repo/scmessenger-cli:latest"

# 1. Build the image for Cloud Run (Linux x86_64)
docker build --platform linux/amd64 -f docker/Dockerfile -t $IMAGE_TAG .

# 2. Push to Artifact Registry
docker push $IMAGE_TAG

# 3. Update container on GCP Compute Instance
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a --tunnel-through-iap --command="
    sudo docker stop scmessenger-relay 2>/dev/null || true
    sudo docker rm scmessenger-relay 2>/dev/null || true
    sudo docker run -d --restart=unless-stopped \
        --name scmessenger-relay \
        -p 9001:9001 \
        -p 9000:9000 \
        $IMAGE_TAG \
        scm relay --listen /ip4/0.0.0.0/tcp/9001 --http-port 9000
"
```

## [Current] 4. Verify

After deployment, verify the node is running:

```bash
# Test GCP node connectivity
./scripts/test_gcp_node.sh

# Or manually check via gcloud
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a --tunnel-through-iap --command="sudo docker ps --format 'table {{.ID}}\t{{.Status}}\t{{.Ports}}'"
```

The node should show:
- Status: `Up X seconds/minutes`
- Ports: `0.0.0.0:9000-9001->9000-9001/tcp`

To view logs:
```bash
gcloud compute ssh scmessenger-bootstrap --zone=us-central1-a --tunnel-through-iap --command="sudo docker logs --tail 50 scmessenger-relay"
```
