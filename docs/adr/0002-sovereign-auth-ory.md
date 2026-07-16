# ADR-0002: Self-hosted Ory Kratos/Hydra for identity

## Status

Accepted — 2026-07-14

## Context

Sovereign by design forbids SaaS IdP lock-in. Enterprise customers require data
residency and self-hosting.

## Decision

- Use **Ory Kratos** for identity/session and **Ory Hydra** for OAuth2/OIDC.
- HelixCore `auth-adapter` is the façade; products never call Kratos directly.
- Local/dev allows `X-Helix-Dev-User` when `HELIX_ENV=local|dev`.

## Consequences

- Full control of identity data.
- Operational burden of running Ory (mitigated by docker-compose + Helm).
