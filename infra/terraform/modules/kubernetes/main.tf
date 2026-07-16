# Sovereign Kubernetes cluster module.
# This is a placeholder wiring harness for a self-hosted control plane (e.g. k3s/rke2).
# Replace with your provider-specific resources (AWS EKS, GCP GKE, on-prem metal, etc.).

terraform {
  required_version = ">= 1.6.0"
}

variable "name" {
  description = "Cluster / environment name"
  type        = string
}

variable "region" {
  description = "Residency region"
  type        = string
}

variable "node_count" {
  description = "Number of control-plane + worker nodes"
  type        = number
  default     = 3
}

output "cluster_name" {
  value = var.name
}

output "residency_region" {
  value = var.region
}

output "node_count" {
  value = var.node_count
}
