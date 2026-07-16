# Next action

## Latest: HELIXCOLLAB-FULL

**Goal:** bring HelixCollab (product 01) to the same sovereign-ready depth as
HELIXCore, with tests, deep smoke, and CI proof.

See `docs/goals/HELIXCOLLAB_FULL.md` for the full definition of done.

### Closed in this session

1. **Goal document** — `docs/goals/HELIXCOLLAB_FULL.md` created and aligned with
   `HELIXCORE_FULL.md` structure.
2. **Runtime blocker fixed** — `projects/helix-collab/backend/src/main.rs` now
   uses `fallback_service` instead of `nest_service("/", …)` to satisfy Axum 0.8.
   Without this the API panicked at startup.
3. **Local proof green**:
   - `cargo test -p helix_collab_api` ✅
   - `cargo clippy -p helix_collab_api -- -D warnings` ✅
   - `@helixforge/helix-collab-web` typecheck ✅
   - `@helixforge/helix-collab-web` build ✅
   - `scripts/helix_collab_smoke.ps1` ✅

### Still open

- Add HelixCollab smoke job and web build to `.github/workflows/ci.yml`.
- Push and verify the full CI run is green.

## Active goal: HELIXCOLLAB-FULL

Do not resume other product 1–20 depth work or activate HelixAnvil until
HELIXCOLLAB-FULL is CI-proven.

## Open founder decisions

- Managed-service commercial model and final custody providers (per HelixCore
  spec) — does not block G0, but must be resolved before G1 capability broker.
- HelixAnvil canonical home is `projects/helix-anvil`; sequencing remains
  portfolio-last.

## Paste-ready continuation prompt

```text
Continue in C:\Users\divin\PROJECTS\HELIXFORGE. HELIXCORE-FULL is CI-proven.
The new active goal is HELIXCOLLAB-FULL.
HelixCollab backend already compiles and passes `scripts/helix_collab_smoke.ps1`
locally after fixing an Axum 0.8 root-nesting panic in
`projects/helix-collab/backend/src/main.rs`.
Next step is adding the HelixCollab smoke job and web build to CI and pushing
for a green run.
Do not resume other product depth work or activate HelixAnvil until
HELIXCOLLAB-FULL is CI-proven.
```
