# HelixCode production environment

## Required

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | Postgres |
| `HELIX_ENV` | Must **not** be `local` / `dev` in production |
| `HELIX_ALLOW_DEV_HEADERS` | Must be `0` or unset |
| `HELIX_CODE_INSTANCE_ID` | Unique per API replica (sticky) |
| `HELIX_CODE_WEBHOOK_ALLOW_HOSTS` | Comma list, e.g. `hooks.example.com,.trusted.org` |

## Strongly recommended

| Variable | Purpose |
|----------|---------|
| `HELIX_CODE_ISOLATION` | `docker` (or `auto` with Docker available) |
| `HELIX_CODE_DOCKER_IMAGE` | `helixforge/helix-code-ci:local` or registry image |
| `HELIX_CODE_TERM_ISOLATION` | `docker` when possible |
| `CSC_LINK` / `CSC_KEY_PASSWORD` | Enterprise OV/EV PFX (see ENTERPRISE_CODESIGN.md) |
| `HELIX_CODE_UPDATE_URL` | Electron generic update feed base |

## Break-glass (avoid in prod)

Process env (global, audited):

- `HELIX_CODE_ALLOW_DIRECT_PUSH`
- `HELIX_CODE_ALLOW_FORCE_PUSH`
- `HELIX_CODE_CI_ALLOW_ALL`
- `HELIX_CODE_TERM_ALLOW_ALL`
- `HELIX_CODE_ALLOW_HOST_FALLBACK`
- `HELIX_CODE_ALLOW_HOST_ISOLATION`

Per-tenant (preferred for temporary grants): `PUT /v1/me/breakglass` (Write scope).

## Sticky / HA

See `HA_STICKY.md`. LB must pin `/v1/terminals/*`, `/v1/debug/*`, `/v1/lsp/*` by instance.
