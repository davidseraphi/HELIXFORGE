# Terraform network module (skeleton)

Provider-agnostic residency-tagged network variables for HelixForge.

## Apply dry-run

```bash
cd infra/terraform/environments/dev
terraform init
terraform plan
```

## Variables

| Name | Description |
|------|-------------|
| `name` | Network name prefix |
| `cidr` | Default `10.42.0.0/16` |
| `region` | Data residency region tag (e.g. `eu-west`) |

Wire outputs to AWS VPC / GCP / Azure / on-prem Calico as needed. No cloud vendor lock-in.
