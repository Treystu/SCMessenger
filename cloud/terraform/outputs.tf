###############################################################################
# SCMessenger Cloud Infrastructure — Outputs
###############################################################################

output "orchestrator_ip" {
  description = "External (ephemeral) IP of the orchestrator VM"
  value       = google_compute_instance.orchestrator.network_interface[0].access_config[0].nat_ip
}

output "orchestrator_internal_ip" {
  description = "Internal IP of the orchestrator VM (used by workers)"
  value       = google_compute_instance.orchestrator.network_interface[0].network_ip
}

output "orchestrator_service_account" {
  description = "Email of the orchestrator service account"
  value       = google_service_account.scm_orchestrator.email
}

output "worker_standard_template" {
  description = "Self-link of the standard worker instance template"
  value       = google_compute_instance_template.worker_standard.self_link
}

output "worker_android_template" {
  description = "Self-link of the Android worker instance template"
  value       = google_compute_instance_template.worker_android.self_link
}
