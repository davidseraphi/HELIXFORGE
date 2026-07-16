# ADR-0001: Rust-first monorepo with Cargo workspaces + Turborepo

## Status

Accepted — 2026-07-14

## Context

HelixForge is a 21-surface ecosystem (HelixCore + 20 products). Shared auth,
vault, agents, billing, and audit must be identical across products. Teams need
one source of truth, high performance, and sovereign deployability.

## Decision

1. **Single monorepo** under `HELIXFORGE/`.
2. **Rust Cargo workspace** for all backends and shared crates.
3. **pnpm + Turborepo** for Next.js UIs and TS packages.
4. **HelixCore** services + `service-kit` crate as the only path products use to
   reach platform capabilities.
5. **NATS JetStream** for async inter-service events; HTTP/JSON for request paths
   initially (gRPC optional later).

## Consequences

- Maximum code reuse; product APIs are thin domain layers.
- CI can quality-gate the whole platform.
- Repo size grows; default-members keep local builds focused on core.
