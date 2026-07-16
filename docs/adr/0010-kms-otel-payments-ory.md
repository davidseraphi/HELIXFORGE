# ADR-0010: KMS, OTEL, marketplace payments, live Ory

## Status

Accepted — 2026-07-14

## Decision

1. **KMS**: Pluggable `KeyManagement` — `LocalSoftwareKms` (default) or `HttpKms`
   (`HELIX_VAULT_KMS_MODE=http`, `HELIX_VAULT_KMS_URL`). New secrets use **HVA3**
   (random DEK wrapped by KMS). Vault-service exposes `/v1/kms/wrap|unwrap` as
   software HSM endpoints.
2. **OTEL**: `OTEL_EXPORTER_OTLP_ENDPOINT` enables OTLP/**HTTP** JSON export to
   collector/Jaeger (`:4318`). Request middleware emits spans; boot span on start.
3. **Payments**: Durable `helix_core.payment_intents` + local_sim confirm that
   activates plan. Real Stripe/PSP adapters later.
4. **Ory**: Compose profile `ory` runs Kratos; auth-adapter `POST /v1/ory/register`
   and `/v1/ory/login` for live session tokens. Dev headers remain for offline local.

## Compose

```bash
docker compose --profile ory --profile observability up -d
```
