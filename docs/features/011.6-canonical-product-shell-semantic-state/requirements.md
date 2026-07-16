# 011.6 — One canonical product shell and semantic state system

## Status

Closed on 2026-07-15. Implementation complete.

## Outcome

Every HelixForge product is represented through one shared semantic state
vocabulary and one canonical shell primitive. The console and future product UIs
reuse the same state tokens, colors, and layout components instead of
reinventing them per product.

## Scope

1. **Semantic state vocabulary**
   - Rust `SemanticState` enum in `shared_core`.
   - TypeScript `SemanticState` enum in `@helixforge/sdk-ts`.
   - Mappings from `JobStatus` and `CheckState` into `SemanticState`.
2. **Product maturity**
   - `ProductMaturity` enum (`Scaffold`, `Prototype`, `Alpha`, `Beta`,
     `Production`).
   - Added to every `ProductMeta` in `shared_core::project`.
3. **Runtime state endpoint**
   - Gateway `/v1/catalog/{slug}/state` probes the product's `/healthz` and
     returns its current `SemanticState`.
4. **Shared UI shell**
   - `@helixforge/ui` package exporting `Shell`, `SemanticBadge`, and a CSS
     token layer.
5. **Shared TS SDK**
   - `@helixforge/sdk-ts` package exporting catalog types and a minimal gateway
     client.
6. **Console reference integration**
   - Console imports `@helixforge/ui` and `@helixforge/sdk-ts`.
   - Catalog cards display tier, maturity, and a live semantic state badge.
   - Layout wraps pages in the canonical `<Shell>`.

## Allowed edits

- `crates/shared-core/src/semantic_state.rs` (new)
- `crates/shared-core/src/project.rs`
- `crates/shared-core/src/lib.rs`
- `crates/shared-core/src/health.rs` (mapping)
- `crates/helix-db/src/jobs.rs` (mapping)
- `services/gateway/src/main.rs`
- `packages/ui/**/*` (new)
- `packages/sdk-ts/**/*` (new)
- `apps/console/**/*`
- This packet directory.

## Forbidden edits

- No changes to product-domain logic outside shared abstractions.
- No changes to service-kit auth or middleware.
- No production runtime state changes.

## EARS acceptance

### Semantic state

- The system SHALL define one `SemanticState` vocabulary: `active`,
  `waiting_human`, `waiting_external`, `completed`, `failed`, `unknown`.
- The system SHALL map `JobStatus` values into `SemanticState`.
- The system SHALL map `CheckState` values into `SemanticState`.

### Product maturity

- The system SHALL tag every product with a `ProductMaturity` value.
- The system SHALL expose maturity in gateway catalog responses.

### Runtime state

- The system SHALL provide `/v1/catalog/{slug}/state` returning the product's
  current semantic state derived from a live health probe.

### Shared shell

- The system SHALL provide a canonical `Shell` React component.
- The system SHALL provide a `SemanticBadge` component that renders state color,
  label, and optional dot.
- The console SHALL use the shared `Shell` and `SemanticBadge` components.

## Test plan

| Check | Evidence |
|---|---|
| SemanticState mappings | Rust unit tests for JobStatus and CheckState conversions |
| Catalog includes maturity | `cargo test --workspace` + gateway catalog response inspection |
| Runtime state endpoint | `curl /v1/catalog/helix-flow/state` with product up/down |
| Shared packages build | `pnpm --filter @helixforge/ui typecheck` and `pnpm --filter @helixforge/sdk-ts typecheck` |
| Console typecheck + build | `pnpm --filter @helixforge/console typecheck` and `pnpm --filter @helixforge/console build` |

## Dependencies

- `011.1`–`011.5` closed.
- Existing gateway catalog, shared-core, service-kit health, helix-db jobs.

## Rollback / compensation

- New fields are additive; existing catalog consumers ignore unknown fields.
- Shared packages are new; removing them reverts only console imports.
