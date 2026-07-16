# OAuth2 / OIDC with Ory Hydra

## Start

```powershell
docker compose --profile ory up -d
# Hydra public :4444  admin :4445
# Discovery: http://127.0.0.1:4444/.well-known/openid-configuration
```

## Auth-adapter APIs

| Endpoint | Purpose |
|----------|---------|
| `GET /v1/oidc/status` | Issuer URLs + health |
| `POST /v1/oidc/clients` | Register OAuth client (Admin) |
| `POST /v1/oidc/introspect` | Map token → Helix principal |

## Client credentials (machine)

1. Create client via `/v1/oidc/clients` (returns `client_id` / `client_secret`).  
2. Token:  
   `POST http://127.0.0.1:4444/oauth2/token`  
   `grant_type=client_credentials&scope=read write`  
3. Call Helix APIs: `Authorization: Bearer <access_token>`  
   Hybrid auth tries **Kratos whoami first**, then **Hydra introspect**.

## Browser OIDC

Use authorization code + PKCE against Hydra with console UI login/consent pages
(point `urls.login` / `urls.consent` in `deploy/local/hydra/hydra.yml` at your console).

## Sovereignty

All tokens are issued by self-hosted Hydra — no Auth0/Okta required.
