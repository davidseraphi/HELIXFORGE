# HelixCode threat model (end-state)

## Trust boundaries

| Zone | Trust | Notes |
|------|-------|-------|
| Client (web/Electron) | Untrusted | XSS → steal dev headers; CSP recommended in prod |
| API (`helix_code_api`) | Trusted compute | Auth via Ory / API keys / local dev headers |
| Postgres | Trusted data | Tenancy isolation by `tenant_id` columns |
| MinIO | Trusted object | Ciphertext-only sealed; CI artifacts may be plaintext logs |
| NATS | Trusted bus | Internal events |
| Docker CI | Semi-trusted | `--network none`; mounts only workdir |

## Assets

- Git objects (IP), sealed secrets, MLS blobs, audit chain, tenant quotas/billing meters.

## Key threats & controls

1. **Cross-tenant read/write** — All queries bind `tenant_id`; ACL optional on `repo`.
2. **Direct push to protected branch** — `branch_protections` enforced on:
   - REST `POST /commits` and batch commits (`require_pr`)
   - Smart HTTP `git-receive-pack` (pkt-line ref updates parsed **before** pack apply)
   - Break-glass: `HELIX_CODE_ALLOW_DIRECT_PUSH=1`
3. **Force push** — `deny_force_push` uses `git merge-base --is-ancestor`; break-glass `HELIX_CODE_ALLOW_FORCE_PUSH=1`.
4. **PR merge gates** — `require_approvals` + `required_status_checks` (pipeline name must have succeeded run for PR head SHA).
5. **CI RCE** — Command allowlist; optional Docker isolation; deny curl/wget/rm -rf by default.
6. **Terminal abuse** — **Allowlist** (`terminal_policy.rs`); blocks shell metacharacters / PowerShell / network tools; break-glass `HELIX_CODE_TERM_ALLOW_ALL=1`; workdir jailed under `.data/helix-code/terminals`.
7. **Webhook SSRF** — `webhook_policy.rs`: http(s) only; blocks metadata IPs/hosts + private ranges (private allowed only when `HELIX_ENV=local` / `HELIX_CODE_WEBHOOK_ALLOW_PRIVATE=1` / dev headers); no redirects; HMAC signature header; 5s timeout.
8. **Quotas** — Enforced: max_repos, max_pipeline_runs_month, **max_agent_jobs_day**, **max_sealed_bytes**.
9. **MLS key backup** — Server stores **opaque** client ciphertext only.
10. **Deploy keys** — `x-helix-deploy-key`; hashed at rest; repo-scoped read/write; still subject to branch protection on push.
11. **Dev headers** — `HELIX_ALLOW_DEV_HEADERS` must be off outside local.

## Residual risk

- Terminal is still process-based (not a full VM); allowlist + relative paths reduce blast radius only.
- HTTPS webhooks connect by hostname after DNS policy (SNI); http pin-to-IP. Production must set `HELIX_CODE_WEBHOOK_ALLOW_HOSTS`.
- Host docker-fallback is gated (`HELIX_CODE_ALLOW_HOST_FALLBACK`); intentional `isolation=host` still runs allowlisted cmds on host.
- Break-glass envs remain global process flags (not per-tenant) but are logged to breakglass ring + tracing.
- Self-signed org Electron cert until OV/EV swap (`ENTERPRISE_CODESIGN.md`).
- DAP host-toolchain dependent; multi-instance needs sticky LB (`HA_STICKY.md`).
