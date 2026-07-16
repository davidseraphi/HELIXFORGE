terraform {
  required_version = ">= 1.6.0"

  required_providers {
    helm = {
      source  = "hashicorp/helm"
      version = ">= 2.12.0"
    }
  }
}

# Assumes the active kubeconfig context points at the target cluster.
# For CI, set KUBE_CONFIG_PATH or pass kubeconfig_path.
provider "helm" {
  kubernetes = {
    config_path = var.kubeconfig_path
  }
}

module "network" {
  source = "../../modules/network"
  name   = "helixforge-prod"
  region = "eu-west"
  cidr   = "10.0.0.0/16"
}

module "kubernetes" {
  source     = "../../modules/kubernetes"
  name       = "helixforge-prod"
  region     = module.network.residency_region
  node_count = 5
}

module "postgres" {
  source    = "../../modules/postgres"
  name      = "helixforge-prod"
  region    = module.network.residency_region
  namespace = var.namespace
  database  = "helixforge"
  username  = "helix"
  password  = var.postgres_password
}

module "nats" {
  source    = "../../modules/nats"
  name      = "helixforge-prod"
  region    = module.network.residency_region
  namespace = var.namespace
}

module "minio" {
  source     = "../../modules/minio"
  name       = "helixforge-prod"
  region     = module.network.residency_region
  namespace  = var.namespace
  bucket     = "helixforge"
  access_key = var.minio_access_key
  secret_key = var.minio_secret_key
}

output "residency_region" {
  value = module.network.residency_region
}

output "cluster_name" {
  value = module.kubernetes.cluster_name
}

output "database_host" {
  value = module.postgres.host
}

output "database_url" {
  value     = "postgres://${module.postgres.username}:${urlencode(var.postgres_password)}@${module.postgres.host}:${module.postgres.port}/${module.postgres.database}"
  sensitive = true
}

output "nats_url" {
  value = module.nats.url
}

output "minio_endpoint" {
  value = module.minio.endpoint
}

output "minio_credentials" {
  value = {
    access_key = module.minio.access_key
    secret_key = module.minio.secret_key
  }
  sensitive = true
}
