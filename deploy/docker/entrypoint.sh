#!/usr/bin/env sh
# Map Helm-style service names to Cargo bin names (underscores).
set -e
name="${1:-gateway}"
case "$name" in
  gateway) bin=gateway ;;
  agent-hub|agent_hub) bin=agent_hub ;;
  vault-service|vault_service) bin=vault_service ;;
  billing-service|billing_service) bin=billing_service ;;
  observability-service|observability_service) bin=observability_service ;;
  auth-adapter|auth_adapter) bin=auth_adapter ;;
  *)
    echo "unknown service: $name" >&2
    echo "expected: gateway|agent-hub|vault-service|billing-service|observability-service|auth-adapter" >&2
    exit 1
    ;;
esac
# Fail closed if secrets missing outside local.
if [ "${HELIX_ENV:-local}" != "local" ]; then
  if [ -z "${DATABASE_URL:-}" ]; then
    echo "DATABASE_URL required when HELIX_ENV!=local" >&2
    exit 1
  fi
  if [ -z "${HELIX_VAULT_MASTER_KEY:-}" ]; then
    echo "HELIX_VAULT_MASTER_KEY required when HELIX_ENV!=local" >&2
    exit 1
  fi
fi
exec "/app/bin/$bin"
