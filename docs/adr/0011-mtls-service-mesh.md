# ADR-0011: mTLS / service mesh strategy

## Status

Accepted — 2026-07-14

## Context

Enterprise zero-trust requires encrypted service-to-service traffic. HelixCore
runs many processes (6 core + 20 products). A full mesh is operationally heavy
for local Windows dev.

## Decision

1. **Local / bare metal:** optional mutual TLS between gateway and core via
   `HELIX_TLS_CERT_FILE` / `HELIX_TLS_KEY_FILE` / `HELIX_TLS_CLIENT_CA` when set.
   Scripts under `deploy/local/mtls/` generate a dev CA + server certs.
2. **Kubernetes:** prefer **Linkerd** or **Istio** ambient/sidecar mode — do not
   reimplement mesh in Rust. Helm chart annotates pods for mesh injection.
3. **Gateway reverse-proxy** remains the application edge; mesh provides
   transport identity (`spiffe://…` later).
4. Product WebSockets (Collab) may still use direct ports until WS mTLS is
   validated end-to-end.

## Consequences

- No mandatory mesh for local smoke.
- Production K8s path is documented and chart-ready.
- Gateway can terminate TLS when certs are mounted.
