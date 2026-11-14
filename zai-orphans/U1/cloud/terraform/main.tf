###############################################################################
# SCMessenger Cloud Infrastructure — Main Terraform Configuration
#
# Provisions:
#   1. Orchestrator VM  (e2-micro, always-on, Debian 12)
#   2. Firewall rules   (external access + inter-node communication)
#   3. Worker instance templates (Spot/preemptible, standard + Android)
#
# Usage:
#   terraform init
#   terraform plan  -var-file="prod.tfvars"
#   terraform apply -var-file="prod.tfvars"
###############################################################################

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
  }
}

# ---------------------------------------------------------------------------
# Provider
# ---------------------------------------------------------------------------
provider "google" {
  project = var.project_id
  region  = var.region
  zone    = var.zone
}

# ---------------------------------------------------------------------------
# Data: latest Debian 12 image
# ---------------------------------------------------------------------------
data "google_compute_image" "debian12" {
  family  = "debian-12"
  project = "debian-cloud"
}

# ---------------------------------------------------------------------------
# Service Account for orchestrator & workers
# ---------------------------------------------------------------------------
resource "google_service_account" "scm_orchestrator" {
  account_id   = "scm-orchestrator"
  display_name = "SCMessenger Orchestrator Service Account"
}

# ---------------------------------------------------------------------------
# Orchestrator VM
# ---------------------------------------------------------------------------
resource "google_compute_instance" "orchestrator" {
  name         = "scm-orchestrator"
  machine_type = "e2-micro"
  zone         = var.zone

  tags = ["scm-orchestrator", "http-server"]

  boot_disk {
    initialize_params {
      image = data.google_compute_image.debian12.self_link
      size  = 20
      type  = "pd-standard"
    }
    auto_delete = true
  }

  network_interface {
    network = "default"

    # Ephemeral external IP
    access_config {}
  }

  service_account {
    email  = google_service_account.scm_orchestrator.email
    scopes = [
      "https://www.googleapis.com/auth/compute",
      "https://www.googleapis.com/auth/devstorage.read_only",
      "https://www.googleapis.com/auth/logging.write",
    ]
  }

  # Pass secrets + config via instance metadata
  metadata = {
    TELEGRAM_BOT_TOKEN = var.telegram_bot_token
    OPENROUTER_API_KEY = var.openrouter_api_key
    GCP_PROJECT        = var.project_id
    GITHUB_REPO        = var.github_repo
  }

  # Startup script — bootstraps the orchestrator
  metadata_startup_script = file("${path.module}/../scripts/orchestrator_startup.sh")

  # Allow Terraform to stop the VM for updates
  allow_stopping_for_update = true

  labels = {
    app  = "scmessenger"
    role = "orchestrator"
  }
}

# ---------------------------------------------------------------------------
# Firewall: external access to orchestrator (SSH + API)
# ---------------------------------------------------------------------------
resource "google_compute_firewall" "orchestrator_external" {
  name    = "scm-orchestrator-external"
  network = "default"

  allow {
    protocol = "tcp"
    ports    = ["22", "8080"]
  }

  # Allow from anywhere (tighten in production)
  source_ranges = ["0.0.0.0/0"]
  target_tags   = ["scm-orchestrator"]

  description = "Allow SSH and API access to the SCMessenger orchestrator"
}

# ---------------------------------------------------------------------------
# Firewall: orchestrator <-> worker internal communication
# ---------------------------------------------------------------------------
resource "google_compute_firewall" "orchestrator_worker_internal" {
  name    = "scm-orchestrator-worker-internal"
  network = "default"

  allow {
    protocol = "tcp"
    ports    = ["22", "8080", "9000-9010"]
  }

  # Only allow traffic between orchestrator and worker tagged instances
  source_tags = ["scm-orchestrator", "scm-worker"]
  target_tags = ["scm-orchestrator", "scm-worker"]

  description = "Allow internal communication between orchestrator and worker nodes"
}

# ---------------------------------------------------------------------------
# Worker Instance Template — Standard (e2-standard-8, Spot)
# ---------------------------------------------------------------------------
resource "google_compute_instance_template" "worker_standard" {
  name_prefix  = "scm-worker-std-"
  machine_type = "e2-standard-8"

  tags = ["scm-worker"]

  disk {
    source_image = data.google_compute_image.debian12.self_link
    disk_size_gb = 50
    disk_type    = "pd-ssd"
    auto_delete  = true
    boot         = true
  }

  network_interface {
    network = "default"
    access_config {} # Ephemeral IP for pulling packages
  }

  scheduling {
    preemptible                 = true
    automatic_restart           = false
    on_host_maintenance         = "TERMINATE"
    provisioning_model          = "SPOT"
    instance_termination_action = "DELETE"
  }

  service_account {
    email  = google_service_account.scm_orchestrator.email
    scopes = [
      "https://www.googleapis.com/auth/compute",
      "https://www.googleapis.com/auth/devstorage.read_only",
      "https://www.googleapis.com/auth/logging.write",
    ]
  }

  labels = {
    app  = "scmessenger"
    role = "worker"
    type = "standard"
  }

  lifecycle {
    create_before_destroy = true
  }
}

# ---------------------------------------------------------------------------
# Worker Instance Template — Android (n2-standard-8, nested virt, Spot)
# ---------------------------------------------------------------------------
resource "google_compute_instance_template" "worker_android" {
  name_prefix  = "scm-worker-android-"
  machine_type = "n2-standard-8"

  tags = ["scm-worker"]

  disk {
    source_image = data.google_compute_image.debian12.self_link
    disk_size_gb = 50
    disk_type    = "pd-ssd"
    auto_delete  = true
    boot         = true
  }

  network_interface {
    network = "default"
    access_config {}
  }

  # Enable nested virtualization for Android emulators
  advanced_machine_features {
    enable_nested_virtualization = true
  }

  scheduling {
    preemptible                 = true
    automatic_restart           = false
    on_host_maintenance         = "TERMINATE"
    provisioning_model          = "SPOT"
    instance_termination_action = "DELETE"
  }

  service_account {
    email  = google_service_account.scm_orchestrator.email
    scopes = [
      "https://www.googleapis.com/auth/compute",
      "https://www.googleapis.com/auth/devstorage.read_only",
      "https://www.googleapis.com/auth/logging.write",
    ]
  }

  labels = {
    app  = "scmessenger"
    role = "worker"
    type = "android"
  }

  lifecycle {
    create_before_destroy = true
  }
}
