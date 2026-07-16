# Contributing to HelixForge

## Review rule

All changes to `main` require human review through a pull request. Direct pushes
to `main` are not allowed.

## Change scope

- One active foundation gate and at most three active product gates at a time.
- Each change must link to an approved feature packet under `docs/features/`.
- High-impact changes (identity, signing, payments, physical control, permanent
  deletion, real sensitive data) require a separate founder decision.

## Checks before review

```bash
RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-msvc cargo build --workspace
RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-msvc cargo test --workspace
RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-msvc cargo clippy --workspace --all-targets -- -D warnings
RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-msvc cargo fmt --all -- --check
pnpm --filter @helixforge/console typecheck
helm lint infra/helm/helix-core
```

## Secrets

Never commit secrets, `.env` files, or signing material. Local secrets live in
`~/Desktop/.keys/helixforge/.env.local` and are loaded by operator-owned scripts.
