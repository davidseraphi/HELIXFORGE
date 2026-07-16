# ADR-0007: Gateway reverse-proxy edge (direct ports still allowed)

## Status

Superseded in-part — 2026-07-14 (proxy implemented; mesh still optional)

## Context

Products bind 8101–8120. Operators also want a single gateway entry for HTTP
APIs without full service mesh.

## Decision

1. Gateway proxies:
   - `/p/{slug}/…` → product default port
   - `/core/{service}/…` → core service URLs from config
2. Direct product ports remain valid for local smoke and WebSocket-heavy apps.
3. Catalog entries include `gateway_prefix` for clients that prefer one origin.
4. Kubernetes mesh / mTLS: see **ADR-0011** (Linkerd/Istio annotations; local optional TLS certs).

## Consequences

- Browser consoles can target gateway only for REST.
- Collab WebSocket may still use direct product port until WS proxy is hardened.
