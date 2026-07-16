variable "kubeconfig_path" {
  type        = string
  description = "Path to the kubeconfig file used by the Helm provider"
  default     = "~/.kube/config"
}

variable "namespace" {
  type        = string
  description = "Kubernetes namespace for HelixCore dependencies"
  default     = "helix-core-prod"
}

variable "postgres_password" {
  type        = string
  description = "Postgres application password"
  sensitive   = true
}

variable "minio_access_key" {
  type        = string
  description = "MinIO root user"
  default     = "helixminio"
}

variable "minio_secret_key" {
  type        = string
  description = "MinIO root password"
  sensitive   = true
}
