# Sovereign NATS module.
# Deploys the official NATS Helm chart into the target Kubernetes cluster.

terraform {
  required_version = ">= 1.6.0"

  required_providers {
    helm = {
      source  = "hashicorp/helm"
      version = ">= 2.12.0"
    }
  }
}

variable "name" {
  type        = string
  description = "Release name prefix"
}

variable "region" {
  type        = string
  description = "Data residency region tag"
}

variable "namespace" {
  type        = string
  description = "Kubernetes namespace"
  default     = "helixforge"
}

variable "chart_version" {
  type        = string
  description = "NATS Helm chart version"
  default     = "1.2.0"
}

resource "helm_release" "nats" {
  name             = "${var.name}-nats"
  repository       = "https://nats-io.github.io/k8s/helm/charts/"
  chart            = "nats"
  version          = var.chart_version
  namespace        = var.namespace
  create_namespace = true

  set = [
    { name = "fullnameOverride", value = "${var.name}-nats" },
  ]
}

output "url" {
  description = "NATS client URL"
  value       = "nats://${var.name}-nats.${var.namespace}.svc.cluster.local:4222"
}

output "monitor_url" {
  description = "NATS monitoring URL"
  value       = "http://${var.name}-nats.${var.namespace}.svc.cluster.local:8222"
}
