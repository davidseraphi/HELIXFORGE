# Kimi full HelixCore review prompt

**Goal:** HELIXCORE-FULL — HelixCore must be a fully built sovereign foundation
for 20 product forges.

**Repo:** `C:\Users\divin\PROJECTS\HELIXFORGE`

## Read first

- `docs/goals/HELIXCORE_FULL.md`
- `docs/features/010-helix-core-deep/requirements.md`
- `docs/features/000-helix-core-bootstrap/requirements.md`
- `constitution.md`
- `VISION.md`
- `AGENTS.md`
- `BUILD_SPEC.md`
- `crates/service-kit`
- `crates/shared-core`
- `crates/helix-db`
- `crates/auth-client`
- `crates/audit-log`
- `crates/agent-framework`
- `crates/vault-client`
- `crates/billing-client`
- `crates/observability`
- `crates/nats-client`
- `services/gateway`
- `services/auth-adapter`
- `services/agent-hub`
- `services/vault-service`
- `services/billing-service`
- `services/observability-service`
- `docker-compose.yml`
- `infra/helm`
- `infra/argocd`
- `infra/terraform`

## Review job

1. Assess how complete HelixCore is vs the HELIXCORE-FULL definition of done.
2. For each capability (AetherID, agents, vault, billing, audit, observability,
   gateway, data plane, infra), label items **DONE**, **PARTIAL**, or **MISSING**.
3. Call out overclaims, security risks, sovereignty gaps, and missing tests.
4. Prioritize the top work stream to finish HelixCore fully.
5. Produce a structured report with:
   - **Verdict:** `PASS` or `PASS_WITH_FOLLOWUPS` or `FAIL` or `NOT_COMPLETE`
   - Executive summary
   - Capability matrix with evidence paths
   - Findings with severity, path, issue, fix suggestion
   - Recommended build order to reach FULL
   - Retest commands

Be harsh about honesty. Scaffold is not done.
