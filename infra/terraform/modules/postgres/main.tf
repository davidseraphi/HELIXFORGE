# Sovereign Postgres module.
# Deploys Bitnami PostgreSQL into the target Kubernetes cluster via Helm.
# Replace with a managed-DB module (RDS, Cloud SQL, etc.) if provider lock-in is
# acceptable for a given environment.

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

variable "database" {
  type        = string
  description = "Database name"
  default     = "helixforge"
}

variable "username" {
  type        = string
  description = "Application database user"
  default     = "helix"
}

variable "password" {
  type        = string
  description = "Application database password"
  sensitive   = true
}

variable "chart_version" {
  type        = string
  description = "Bitnami PostgreSQL chart version"
  default     = "15.0.0"
}

resource "helm_release" "postgresql" {
  name             = "${var.name}-postgres"
  repository       = "https://charts.bitnami.com/bitnami"
  chart            = "postgresql"
  version          = var.chart_version
  namespace        = var.namespace
  create_namespace = true

  set {
    name  = "global.postgresql.auth.database"
    value = var.database
  }

  set {
    name  = "global.postgresql.auth.username"
    value = var.username
  }

  set_sensitive {
    name  = "global.postgresql.auth.password"
    value = var.password
  }

  set_sensitive {
    name  = "global.postgresql.auth.postgresPassword"
    value = var.password
  }

  set {
    name  = "fullnameOverride"
    value = "${var.name}-postgres"
  }
}

output "host" {
  description = "Postgres service hostname inside the cluster"
  value       = "${var.name}-postgres.${var.namespace}.svc.cluster.local"
}

output "port" {
  description = "Postgres port"
  value       = 5432
}

output "database" {
  description = "Database name"
  value       = var.database
}

output "username" {
  description = "Application user"
  value       = var.username
}

output "password" {
  description = "Application password"
  value       = var.password
  sensitive   = true
}
