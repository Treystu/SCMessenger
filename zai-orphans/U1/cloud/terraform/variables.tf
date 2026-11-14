###############################################################################
# SCMessenger Cloud Infrastructure — Input Variables
#
# Required variables (no default):
#   - project_id
#   - telegram_bot_token
#   - openrouter_api_key
#
# Supply via terraform.tfvars, -var flags, or environment variables.
###############################################################################

variable "project_id" {
  description = "GCP project ID where all resources will be provisioned"
  type        = string
}

variable "region" {
  description = "Default GCP region for resources"
  type        = string
  default     = "us-central1"
}

variable "zone" {
  description = "Default GCP zone for the orchestrator VM"
  type        = string
  default     = "us-central1-a"
}

variable "telegram_bot_token" {
  description = "Telegram Bot API token for the SCMessenger bot"
  type        = string
  sensitive   = true
}

variable "openrouter_api_key" {
  description = "OpenRouter API key for LLM access"
  type        = string
  sensitive   = true
}

variable "github_repo" {
  description = "GitHub repository in owner/repo format (e.g. user/SCMessenger_Clean)"
  type        = string
  default     = "user/SCMessenger_Clean"
}
