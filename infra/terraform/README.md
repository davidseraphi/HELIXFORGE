# HelixCore infrastructure

Terraform modules for deploying the sovereign HelixCore data-plane dependencies
(Postgres, NATS, MinIO) onto a Kubernetes cluster.

## Layout

- `modules/` — reusable, provider-agnostic modules.
  - `network` — VPC/network intent (replace with AWS/GCP/Azure/on-prem Calico).
  - `kubernetes` — cluster metadata (replace with provider-specific cluster resource).
  - `postgres` — deploys Bitnami PostgreSQL via Helm.
  - `nats` — deploys the official NATS Helm chart.
  - `minio` — deploys Bitnami MinIO via Helm.
- `environments/` — concrete environment roots.
  - `prod` — wires dependencies together and exposes connection URLs.
  - `dev` — local intent; for local Docker Compose use `docker compose up -d` at the repo root.

## Deploy prod dependencies

Requires:
- Terraform >= 1.6.0
- `kubectl` configured for the target cluster
- Helm provider fetched by Terraform

```bash
cd infra/terraform/environments/prod
terraform init
terraform plan \
  -var="postgres_password=$(openssl rand -hex 32)" \
  -var="minio_secret_key=$(openssl rand -hex 32)" \
  -out=tfplan
terraform apply tfplan
```

After dependencies are running, deploy HelixCore itself via the ArgoCD
application in `infra/argocd/applications/helix-core-prod.yaml`, or via Helm
using the URLs from the Terraform outputs.
