# Deploying SCMessenger to Google Cloud Platform (Cloud Run)

This guide explains how to deploy SCMessenger as a scalable, serverless container service on GCP Cloud Run.

## ⚠️ Important: State Persistence

By default, **Cloud Run is stateless**. When your container restarts or scales down, **all local data (Identity, Contacts, Message History)** will be lost because SCMessenger uses an embedded database (`sled`) stored in the container.

**For Testing**: This is fine. You will get a fresh identity every time you deploy.
**For Production**: You must mount a persistent volume (Cloud Storage FUSE or NFS) or refactor the app to use an external database (which is outside the scope of this quick guide).

---

## 1. Prerequisites

- Google Cloud SDK (`gcloud`) installed and logged in.
- A GCP Project created.

## 2. One-Time Setup

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

## 3. Build & Deploy

You can run this sequence every time you want to update the app.

```bash
# Variables
export PROJECT_ID=$(gcloud config get-value project)
export REGION="us-central1"
export IMAGE_TAG="us-central1-docker.pkg.dev/$PROJECT_ID/scmessenger-repo/scmessenger:latest"

# 1. Build the image for Cloud Run (Linux x86_64)
# We use --platform linux/amd64 to ensure compatibility even if building from a Mac M1/M2
docker build --platform linux/amd64 -f docker/Dockerfile -t $IMAGE_TAG .

# 2. Push to Artifact Registry
docker push $IMAGE_TAG

# 3. Deploy to Cloud Run
gcloud run deploy scmessenger-service \
    --image $IMAGE_TAG \
    --region $REGION \
    --platform managed \
    --allow-unauthenticated \
    --port 8080 \
    --session-affinity  # Recommended for WebSockets to stick to one instance
```

## 4. Verify

After deployment, GCP will output a URL (e.g., `https://scmessenger-service-xyz.a.run.app`).

1. Open that URL in your browser.
2. The UI should load.
3. Open the **Browser Console** (F12) to verify the WebSocket connection:
   - It should connect to `wss://scmessenger-service-xyz.a.run.app/ws`.
