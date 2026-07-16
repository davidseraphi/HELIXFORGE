# Review configuration

## Four-tier review (portfolio doctrine)

| Tier | When | Gate |
|------|------|------|
| 0 | Every commit | `cargo test --workspace`, gitleaks, no secrets in tree |
| 1 | Feature PR | Architecture fit + product domain boundaries |
| 2 | Cross-cutting | Auth, vault, audit, billing changes |
| 3 | Pre-release | Full CI + manual smoke of gateway + one product |

## Gated paths (require Tier 2+)

- `crates/service-kit/**`
- `crates/auth-client/**`
- `crates/audit-log/**`
- `services/auth-adapter/**`
- `infra/**`
- `.github/workflows/**`
