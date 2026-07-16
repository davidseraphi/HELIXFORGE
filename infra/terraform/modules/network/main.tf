variable "name" {
  type        = string
  description = "Network name prefix"
}

variable "cidr" {
  type    = string
  default = "10.42.0.0/16"
}

variable "region" {
  type        = string
  description = "Data residency region tag"
}

output "network_name" {
  value = var.name
}

output "cidr" {
  value = var.cidr
}

output "residency_region" {
  value = var.region
}

# Provider-agnostic network module skeleton.
# Wire to AWS VPC / GCP VPC / Azure VNet / on-prem Calico as needed.
# Sovereignty principle: no hard dependency on a single cloud vendor.
