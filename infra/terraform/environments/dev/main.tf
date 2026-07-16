terraform {
  required_version = ">= 1.6.0"
}

module "network" {
  source = "../../modules/network"
  name   = "helixforge-dev"
  region = "local"
  cidr   = "10.42.0.0/16"
}

output "residency_region" {
  value = module.network.residency_region
}
