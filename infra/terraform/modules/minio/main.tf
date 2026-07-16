# Sovereign MinIO module.
# Deploys Bitnami MinIO into the target Kubernetes cluster via Helm.

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

variable "bucket" {
  type        = string
  description = "Default bucket to create"
  default     = "helixforge"
}

variable "access_key" {
  type        = string
  description = "MinIO root user"
  default     = "helixminio"
}

variable "secret_key" {
  type        = string
  description = "MinIO root password"
  sensitive   = true
}

variable "chart_version" {
  type        = string
  description = "Bitnami MinIO chart version"
  default     = "14.0.0"
}

resource "helm_release" "minio" {
  name             = "${var.name}-minio"
  repository       = "https://charts.bitnami.com/bitnami"
  chart            = "minio"
  version          = var.chart_version
  namespace        = var.namespace
  create_namespace = true

  set {
    name  = "fullnameOverride"
    value = "${var.name}-minio"
  }

  set {
    name  = "auth.rootUser"
    value = var.access_key
  }

  set_sensitive {
    name  = "auth.rootPassword"
    value = var.secret_key
  }

  set {
    name  = "defaultBuckets"
    value = var.bucket
  }
}

output "endpoint" {
  description = "MinIO S3 endpoint"
  value       = "http://${var.name}-minio.${var.namespace}.svc.cluster.local:9000"
}

output "console_endpoint" {
  description = "MinIO console endpoint"
  value       = "http://${var.name}-minio.${var.namespace}.svc.cluster.local:9001"
}

output "bucket" {
  description = "Default bucket"
  value       = var.bucket
}

output "access_key" {
  description = "MinIO root user"
  value       = var.access_key
}

output "secret_key" {
  description = "MinIO root password"
  value       = var.secret_key
  sensitive   = true
}
