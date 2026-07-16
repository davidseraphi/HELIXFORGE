# OpenTelemetry (OTEL) for HelixCore

## Status

- Structured tracing via `tracing` on every `service_kit` boot.
- **`OTEL_EXPORTER_OTLP_ENDPOINT`** enables OTLP/**HTTP** JSON export (`/v1/traces`).
- gRPC `:4317` is auto-mapped to HTTP `:4318` for Jaeger all-in-one.
- Request middleware emits spans; boot span on process start.
- Healthz reports `otlp.active=true` when export is enabled.
- Metrics: in-process counters + Prometheus text on observability-service.

## Local

```powershell
docker compose --profile observability up -d jaeger
$env:OTEL_EXPORTER_OTLP_ENDPOINT = "http://127.0.0.1:4318"
# start core services
# UI: http://127.0.0.1:16686
```

## Kubernetes

```yaml
otel:
  endpoint: "http://otel-collector.observability:4318"
```

## Non-goals

- Mandatory SaaS APM vendor.
- gRPC OTLP only (HTTP JSON is the portable path).
