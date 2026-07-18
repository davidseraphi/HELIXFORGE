# Decision log (append-only)

## 2026-07-18 — HELIXVITAPRIME-FULL closed and CI-proven

- Completed the HelixVita Prime second-wave depth packet:
  - Migration `0053_vita_depth.sql` + down migration: `recruiting_at`/
    `completed_at`/`terminated_at`/`deleted_at` lifecycle columns on
    `vita.studies`; `updated_at`, `enrolled_at`, `withdrawn_at`, `deleted_at`
    on `vita.cohorts`; legacy `open` cohort status backfilled to `draft`;
    partial active indexes.
  - Extended `VitaRepo` (`crates/helix-db/src/vita.rs`) with `update_study`,
    `recruit_study`, `complete_study` (rejected while draft cohorts remain),
    `terminate_study`, `soft_delete_study`, `restore_study`; parent-verified
    `update_cohort`, `enroll_cohort`, `withdraw_cohort`,
    `soft_delete_cohort`, `restore_cohort`; and `get_vita_summary`.
  - Added routes in `projects/helix-vita-prime/backend/src/main.rs`:
    `PATCH /v1/studies/{id}`, `POST /v1/studies/{id}/recruit`,
    `POST /v1/studies/{id}/complete`, `POST /v1/studies/{id}/terminate`,
    `POST /v1/studies/{id}/delete`, `POST /v1/studies/{id}/restore`,
    `PATCH /v1/studies/{id}/cohorts/{cohort_id}`,
    `POST /v1/studies/{id}/cohorts/{cohort_id}/enroll`,
    `POST /v1/studies/{id}/cohorts/{cohort_id}/withdraw`,
    `POST /v1/studies/{id}/cohorts/{cohort_id}/delete`,
    `POST /v1/studies/{id}/cohorts/{cohort_id}/restore`,
    `GET /v1/reports/vita-summary`, and `GET /v1/domain/status` with planes.
  - In-process tests: study and cohort status transition guards.
  - Ignored Postgres integration test for the completion guard, study/cohort
    lifecycle, and vita summary.
  - PowerShell smoke `scripts/helix_vita_prime_smoke.ps1` and
    `vita-prime-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29655268193` is all green, including the new
    **HelixVita Prime smoke** job.
- Commits `da9d339` (activation) and `7bad969` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXVITAPRIME-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXQUANTUMFORGE-FULL closed and CI-proven

- Completed the HelixQuantum Forge second-wave depth packet:
  - Migration `0052_quantum_depth.sql` + down migration: `submitted_at`/
    `completed_at`/`failed_at`/`deleted_at` lifecycle columns on
    `quantum.jobs`; `updated_at`, `validated_at`, `archived_at`, `deleted_at`
    on `quantum.circuits`; legacy `open` circuit status backfilled to
    `draft`; partial active indexes.
  - Extended `QuantumRepo` (`crates/helix-db/src/quantum.rs`) with
    `update_job`, `submit_job` (requires at least one non-deleted circuit),
    `complete_job`, `fail_job`, `soft_delete_job`, `restore_job`;
    parent-verified `update_circuit`, `validate_circuit`, `archive_circuit`,
    `soft_delete_circuit`, `restore_circuit`; and `get_quantum_summary`.
  - Added routes in `projects/helix-quantum-forge/backend/src/main.rs`:
    `PATCH /v1/jobs/{id}`, `POST /v1/jobs/{id}/submit`,
    `POST /v1/jobs/{id}/complete`, `POST /v1/jobs/{id}/fail`,
    `POST /v1/jobs/{id}/delete`, `POST /v1/jobs/{id}/restore`,
    `PATCH /v1/jobs/{id}/circuits/{circuit_id}`,
    `POST /v1/jobs/{id}/circuits/{circuit_id}/validate`,
    `POST /v1/jobs/{id}/circuits/{circuit_id}/archive`,
    `POST /v1/jobs/{id}/circuits/{circuit_id}/delete`,
    `POST /v1/jobs/{id}/circuits/{circuit_id}/restore`,
    `GET /v1/reports/quantum-summary`, and `GET /v1/domain/status` with
    planes.
  - In-process tests: job and circuit status transition guards.
  - Ignored Postgres integration test for the submit guard, job/circuit
    lifecycle, and quantum summary.
  - PowerShell smoke `scripts/helix_quantum_forge_smoke.ps1` and
    `quantum-forge-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29652895313` is all green, including the new
    **HelixQuantum Forge smoke** job.
- Commits `9c9d300` (activation) and `928b04f` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXQUANTUMFORGE-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXORBITPRIME-FULL closed and CI-proven

- Completed the HelixOrbit Prime second-wave depth packet:
  - Migration `0051_orbit_depth.sql` + down migration: `commissioned_at`/
    `decommissioned_at`/`deleted_at` lifecycle columns on `orbit.assets`;
    `updated_at`, `planned_at`, `completed_at`, `cancelled_at`, `deleted_at`
    on `orbit.passes`; legacy `open` pass status backfilled to `draft`;
    partial active indexes.
  - Extended `OrbitRepo` (`crates/helix-db/src/orbit.rs`) with
    `update_asset`, `commission_asset`, `decommission_asset` (rejected while
    draft or planned passes remain), `recommission_asset`,
    `soft_delete_asset`, `restore_asset`; parent-verified `update_pass`,
    `plan_pass`, `complete_pass`, `cancel_pass`, `soft_delete_pass`,
    `restore_pass`; and `get_orbit_summary`.
  - Added routes in `projects/helix-orbit-prime/backend/src/main.rs`:
    `PATCH /v1/assets/{id}`, `POST /v1/assets/{id}/commission`,
    `POST /v1/assets/{id}/decommission`, `POST /v1/assets/{id}/recommission`,
    `POST /v1/assets/{id}/delete`, `POST /v1/assets/{id}/restore`,
    `PATCH /v1/assets/{id}/passes/{pass_id}`,
    `POST /v1/assets/{id}/passes/{pass_id}/plan`,
    `POST /v1/assets/{id}/passes/{pass_id}/complete`,
    `POST /v1/assets/{id}/passes/{pass_id}/cancel`,
    `POST /v1/assets/{id}/passes/{pass_id}/delete`,
    `POST /v1/assets/{id}/passes/{pass_id}/restore`,
    `GET /v1/reports/orbit-summary`, and `GET /v1/domain/status` with planes.
  - In-process tests: asset and pass status transition guards.
  - Ignored Postgres integration test for the decommission guard, asset/pass
    lifecycle, and orbit summary.
  - PowerShell smoke `scripts/helix_orbit_prime_smoke.ps1` and
    `orbit-prime-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29651383990` is all green, including the new
    **HelixOrbit Prime smoke** job.
- Commits `1931c91` (activation) and `7bb8244` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXORBITPRIME-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXCLIMATEPRIME-FULL closed and CI-proven

- Completed the HelixClimate Prime second-wave depth packet:
  - Migration `0050_climate_depth.sql` + down migration: `activated_at`/
    `archived_at`/`deleted_at` lifecycle columns on `climate.scenarios`;
    `updated_at`, `assessed_at`, `dismissed_at`, `deleted_at` on
    `climate.risk_scores`; legacy `open` score status backfilled to `draft`;
    partial active indexes.
  - Extended `ClimateRepo` (`crates/helix-db/src/climate.rs`) with
    `update_scenario`, `activate_scenario`, `archive_scenario` (rejected
    while draft scores remain), `reopen_scenario`, `soft_delete_scenario`,
    `restore_scenario`; parent-verified `update_score`, `assess_score`,
    `dismiss_score`, `soft_delete_score`, `restore_score`; and
    `get_climate_summary`.
  - Added routes in `projects/helix-climate-prime/backend/src/main.rs`:
    `PATCH /v1/scenarios/{id}`, `POST /v1/scenarios/{id}/activate`,
    `POST /v1/scenarios/{id}/archive`, `POST /v1/scenarios/{id}/reopen`,
    `POST /v1/scenarios/{id}/delete`, `POST /v1/scenarios/{id}/restore`,
    `PATCH /v1/scenarios/{id}/risk_scores/{score_id}`,
    `POST /v1/scenarios/{id}/risk_scores/{score_id}/assess`,
    `POST /v1/scenarios/{id}/risk_scores/{score_id}/dismiss`,
    `POST /v1/scenarios/{id}/risk_scores/{score_id}/delete`,
    `POST /v1/scenarios/{id}/risk_scores/{score_id}/restore`,
    `GET /v1/reports/climate-summary`, and `GET /v1/domain/status` with
    planes.
  - In-process tests: scenario and score status transition guards.
  - Ignored Postgres integration test for the archive guard, scenario/score
    lifecycle, and climate summary.
  - PowerShell smoke `scripts/helix_climate_prime_smoke.ps1` and
    `climate-prime-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29650054052` is all green, including the new
    **HelixClimate Prime smoke** job.
- Commits `2d0db63` (activation) and `db899a8` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXCLIMATEPRIME-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXTERRAPRIME-FULL closed and CI-proven

- Completed the HelixTerra Prime second-wave depth packet:
  - Migration `0049_terra_depth.sql` + down migration: `activated_at`/
    `retired_at`/`deleted_at` lifecycle columns on `terra.fields`;
    `updated_at`, `confirmed_at`, `dismissed_at`, `deleted_at` on
    `terra.observations`; legacy `open` observation status backfilled to
    `draft`; partial active indexes.
  - Extended `TerraRepo` (`crates/helix-db/src/terra.rs`) with
    `update_field`, `activate_field`, `retire_field` (rejected while draft
    observations remain), `reopen_field`, `soft_delete_field`,
    `restore_field`; parent-verified `update_observation`,
    `confirm_observation`, `dismiss_observation`, `soft_delete_observation`,
    `restore_observation`; and `get_terra_summary`.
  - Added routes in `projects/helix-terra-prime/backend/src/main.rs`:
    `PATCH /v1/fields/{id}`, `POST /v1/fields/{id}/activate`,
    `POST /v1/fields/{id}/retire`, `POST /v1/fields/{id}/reopen`,
    `POST /v1/fields/{id}/delete`, `POST /v1/fields/{id}/restore`,
    `PATCH /v1/fields/{id}/observations/{obs_id}`,
    `POST /v1/fields/{id}/observations/{obs_id}/confirm`,
    `POST /v1/fields/{id}/observations/{obs_id}/dismiss`,
    `POST /v1/fields/{id}/observations/{obs_id}/delete`,
    `POST /v1/fields/{id}/observations/{obs_id}/restore`,
    `GET /v1/reports/terra-summary`, and `GET /v1/domain/status` with planes.
  - In-process tests: field and observation status transition guards.
  - Ignored Postgres integration test for the retire guard, field/observation
    lifecycle, and terra summary.
  - PowerShell smoke `scripts/helix_terra_prime_smoke.ps1` and
    `terra-prime-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29648775239` is all green, including the new
    **HelixTerra Prime smoke** job.
- Commits `60f5eef` (activation) and `2eb2347` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXTERRAPRIME-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXCURAPRIME-FULL closed and CI-proven

- Completed the HelixCura Prime second-wave depth packet:
  - Migration `0048_cura_depth.sql` + down migration: `activated_at`/
    `discharged_at`/`deleted_at` lifecycle columns on `cura.care_cases`;
    `updated_at`, `signed_at`, `voided_at`, `deleted_at` on `cura.notes`;
    legacy `open` note status backfilled to `draft`; partial active indexes.
  - Extended `CuraRepo` (`crates/helix-db/src/cura.rs`) with `update_case`,
    `activate_case`, `discharge_case` (rejected while draft notes remain),
    `reopen_case`, `soft_delete_case`, `restore_case`; parent-verified
    `update_note` (rejected once signed or voided), `sign_note`, `void_note`,
    `soft_delete_note`, `restore_note`; and `get_cura_summary`.
  - Added routes in `projects/helix-cura-prime/backend/src/main.rs`:
    `PATCH /v1/care_cases/{id}`, `POST /v1/care_cases/{id}/activate`,
    `POST /v1/care_cases/{id}/discharge`, `POST /v1/care_cases/{id}/reopen`,
    `POST /v1/care_cases/{id}/delete`, `POST /v1/care_cases/{id}/restore`,
    `PATCH /v1/care_cases/{id}/notes/{note_id}`,
    `POST /v1/care_cases/{id}/notes/{note_id}/sign`,
    `POST /v1/care_cases/{id}/notes/{note_id}/void`,
    `POST /v1/care_cases/{id}/notes/{note_id}/delete`,
    `POST /v1/care_cases/{id}/notes/{note_id}/restore`,
    `GET /v1/reports/cura-summary`, and `GET /v1/domain/status` with planes.
  - In-process tests: case and note status transition guards.
  - Ignored Postgres integration test for the discharge guard, signed-note
    immutability, case/note lifecycle, and cura summary.
  - PowerShell smoke `scripts/helix_cura_prime_smoke.ps1` and
    `cura-prime-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29647567869` is all green, including the new
    **HelixCura Prime smoke** job.
- Commits `60ba77f` (activation) and `cd279bb` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXCURAPRIME-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXLEXPRIME-FULL closed and CI-proven

- Completed the HelixLex Prime second-wave depth packet:
  - Migration `0047_lex_depth.sql` + down migration: `opened_at`/`closed_at`/
    `deleted_at` lifecycle columns on `lex.matters`; `updated_at`, `filed_at`,
    `withdrawn_at`, `deleted_at` on `lex.filings`; legacy `open` filing status
    backfilled to `draft`; partial active indexes.
  - Extended `LexRepo` (`crates/helix-db/src/lex.rs`) with `update_matter`,
    `open_matter`, `close_matter` (rejected while draft filings remain),
    `reopen_matter`, `soft_delete_matter`, `restore_matter`; parent-verified
    `update_filing`, `file_filing`, `withdraw_filing`, `soft_delete_filing`,
    `restore_filing`; and `get_lex_summary`.
  - Added routes in `projects/helix-lex-prime/backend/src/main.rs`:
    `PATCH /v1/matters/{id}`, `POST /v1/matters/{id}/open`,
    `POST /v1/matters/{id}/close`, `POST /v1/matters/{id}/reopen`,
    `POST /v1/matters/{id}/delete`, `POST /v1/matters/{id}/restore`,
    `PATCH /v1/matters/{id}/filings/{filing_id}`,
    `POST /v1/matters/{id}/filings/{filing_id}/file`,
    `POST /v1/matters/{id}/filings/{filing_id}/withdraw`,
    `POST /v1/matters/{id}/filings/{filing_id}/delete`,
    `POST /v1/matters/{id}/filings/{filing_id}/restore`,
    `GET /v1/reports/lex-summary`, and `GET /v1/domain/status` with planes.
  - In-process tests: matter and filing status transition guards.
  - Ignored Postgres integration test for the close guard, matter/filing
    lifecycle, and lex summary.
  - PowerShell smoke `scripts/helix_lex_prime_smoke.ps1` and
    `lex-prime-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29646308966` is all green, including the new
    **HelixLex Prime smoke** job.
- Commits `082f548` (activation) and `0e42fce` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXLEXPRIME-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXSYNTHBIO-FULL closed and CI-proven

- Completed the HelixSynthBio second-wave depth packet:
  - Migration `0046_synthbio_depth.sql` + down migration: `submitted_at`/
    `approved_at`/`deleted_at` lifecycle columns on `synthbio.designs`;
    `updated_at`, `started_at`, `completed_at`, `failed_at`, `deleted_at` on
    `synthbio.sims`; partial active indexes.
  - Extended `SynthbioRepo` (`crates/helix-db/src/synthbio.rs`) with
    `update_design`, `submit_design`, `approve_design` (requires at least one
    completed sim), `return_design`, `soft_delete_design`, `restore_design`;
    parent-verified `update_sim`, `start_sim`, `complete_sim`, `fail_sim`,
    `soft_delete_sim`, `restore_sim`; and `get_synthbio_summary`.
  - Added routes in `projects/helix-synthbio/backend/src/main.rs`:
    `PATCH /v1/designs/{id}`, `POST /v1/designs/{id}/submit`,
    `POST /v1/designs/{id}/approve`, `POST /v1/designs/{id}/return`,
    `POST /v1/designs/{id}/delete`, `POST /v1/designs/{id}/restore`,
    `PATCH /v1/designs/{id}/sims/{sim_id}`,
    `POST /v1/designs/{id}/sims/{sim_id}/start`,
    `POST /v1/designs/{id}/sims/{sim_id}/complete`,
    `POST /v1/designs/{id}/sims/{sim_id}/fail`,
    `POST /v1/designs/{id}/sims/{sim_id}/delete`,
    `POST /v1/designs/{id}/sims/{sim_id}/restore`,
    `GET /v1/reports/synthbio-summary`, and `GET /v1/domain/status` with
    planes.
  - In-process tests: design and sim status transition guards.
  - Ignored Postgres integration test for the approval guard, design/sim
    lifecycle, and synthbio summary.
  - PowerShell smoke `scripts/helix_synthbio_smoke.ps1` and `synthbio-smoke`
    CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29644975351` is all green, including the new
    **HelixSynthBio smoke** job.
- Commits `9497ec1` (activation) and `1774e92` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXSYNTHBIO-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXFORGESTUDIO-FULL closed and CI-proven

- Completed the HelixForge Studio second-wave depth packet:
  - Migration `0045_studio_depth.sql` + down migration: `published_at`/
    `deleted_at` lifecycle columns on `studio.apps`; `updated_at`,
    `archived_at`, `deleted_at` on `studio.pages`; partial active indexes.
  - Extended `StudioRepo` (`crates/helix-db/src/studio.rs`) with `update_app`,
    `publish_app` (requires at least one non-deleted page), `unpublish_app`,
    `soft_delete_app`, `restore_app`; parent-verified `update_page`,
    `archive_page`, `reopen_page`, `soft_delete_page`, `restore_page`; and
    `get_studio_summary`.
  - Added routes in `projects/helix-forge-studio/backend/src/main.rs`:
    `PATCH /v1/apps/{id}`, `POST /v1/apps/{id}/publish`,
    `POST /v1/apps/{id}/unpublish`, `POST /v1/apps/{id}/delete`,
    `POST /v1/apps/{id}/restore`, `PATCH /v1/apps/{id}/pages/{page_id}`,
    `POST /v1/apps/{id}/pages/{page_id}/archive`,
    `POST /v1/apps/{id}/pages/{page_id}/reopen`,
    `POST /v1/apps/{id}/pages/{page_id}/delete`,
    `POST /v1/apps/{id}/pages/{page_id}/restore`,
    `GET /v1/reports/studio-summary`, and `GET /v1/domain/status` with planes.
  - In-process tests: app and page status transition guards.
  - Ignored Postgres integration test for the publish guard, app/page
    lifecycle, and studio summary.
  - PowerShell smoke `scripts/helix_forge_studio_smoke.ps1` and
    `forge-studio-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29643838956` is all green, including the new
    **HelixForge Studio smoke** job.
- Commits `d1d00ed` (activation) and `d5204ce` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark
  HELIXFORGESTUDIO-FULL closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXNETWORK-FULL closed and CI-proven

- Completed the HelixNetwork second-wave depth packet:
  - Migration `0044_network_depth.sql` + down migration: `deactivated_at`/
    `deleted_at` lifecycle columns on `network.profiles`, `responded_at`/
    `blocked_by` on `network.connections`, `closed_at`/`deleted_at` on
    `network.opportunities`, partial active indexes, connection pair index.
  - Extended `NetworkRepo` (`crates/helix-db/src/network.rs`) with owner-scoped
    `update_profile`, `deactivate_profile`, `reactivate_profile`,
    `soft_delete_profile`, `restore_profile`; a `request_connection` rework
    (revives declined/removed pairs, rejects blocked pairs in both directions,
    requires active profiles); `decline_connection`, `remove_connection`,
    `block_connection`; owner-scoped `update_opportunity`,
    `close_opportunity`, `reopen_opportunity`, `soft_delete_opportunity`,
    `restore_opportunity`; and `get_network_summary`.
  - Added routes in `projects/helix-network/backend/src/main.rs`:
    `PATCH /v1/profiles/{id}`, `POST /v1/profiles/{id}/deactivate`,
    `POST /v1/profiles/{id}/reactivate`, `POST /v1/profiles/{id}/delete`,
    `POST /v1/profiles/{id}/restore`, `POST /v1/connections/{id}/decline`,
    `POST /v1/connections/{id}/remove`, `POST /v1/connections/{id}/block`,
    `PATCH /v1/opportunities/{id}`, `POST /v1/opportunities/{id}/close`,
    `POST /v1/opportunities/{id}/reopen`, `POST /v1/opportunities/{id}/delete`,
    `POST /v1/opportunities/{id}/restore`, `GET /v1/reports/network-summary`,
    and `GET /v1/domain/status` with planes.
  - In-process tests: profile transitions, opportunity transitions, connection
    revival eligibility.
  - Ignored Postgres integration test for the full connection lifecycle,
    blocking, profile lifecycle, opportunity lifecycle, and network summary.
  - PowerShell smoke `scripts/helix_network_smoke.ps1` (two dev users) and
    `network-smoke` CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29642796843` is all green, including the new
    **HelixNetwork smoke** job.
- Commits `5be6550` (activation) and `eae2367` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark HELIXNETWORK-FULL
  closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXWELL-FULL closed and CI-proven

- Completed the HelixWell second-wave depth packet:
  - Migration `0043_well_depth.sql` + down migration: `paused_at`/`ended_at`/
    `deleted_at` lifecycle columns on `well.habits`; nullable `mood`/`energy`
    on `well.checkins` (a skipped field is missing, never zero) plus
    `updated_at`, `deleted_at`, `edit_version`; append-only
    `well.checkin_edits` history table; partial active indexes.
  - Extended `WellRepo` (`crates/helix-db/src/well.rs`) with `update_habit`,
    `pause_habit`, `resume_habit`, `end_habit`, `soft_delete_habit`,
    `restore_habit`, optional-field `create_checkin`, transactional
    `update_checkin` (pre-edit snapshot + version bump),
    `soft_delete_checkin`, `list_checkin_edits`, and `get_habit_summary`.
  - Added routes in `projects/helix-well/backend/src/main.rs`:
    `PATCH /v1/habits/{id}`, `POST /v1/habits/{id}/pause`,
    `POST /v1/habits/{id}/resume`, `POST /v1/habits/{id}/end`,
    `POST /v1/habits/{id}/delete`, `POST /v1/habits/{id}/restore`,
    `GET /v1/checkins/{id}`, `PATCH /v1/checkins/{id}`,
    `POST /v1/checkins/{id}/delete`, `GET /v1/checkins/{id}/edits`,
    `GET /v1/reports/habit-summary`, and `GET /v1/domain/status` with planes.
  - In-process tests: scale validation, skipped fields, habit status
    transitions, empty-update detection.
  - Ignored Postgres integration test for habit lifecycle + check-in edit
    history + habit summary.
  - PowerShell smoke `scripts/helix_well_smoke.ps1` and `well-smoke`
    CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29641402713` is all green, including the new
    **HelixWell smoke** job.
- Commits `90b89cf` (activation) and `2183f7d` (implementation) pushed to
  `main`.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark HELIXWELL-FULL
  closed and clear the active goal.
- Next action: founder selects the next explicit named goal.

## 2026-07-17 — HELIXEDU-FULL second-wave depth packet

- Activated goal: `HELIXEDU-FULL` (catalog order 6, port 8106).
- Added migration `0041_edu_depth.sql` for course soft-delete, active indexes,
  and the `edu.progress_history` audit side table.
- Extended `EduRepo` with course update/soft-delete/restore/unpublish,
  published-only enrollment guard, enrollment withdrawal, and progress history
  recording with `completed_at` transitions.
- Added backend routes: `PATCH /v1/courses/{id}`, `POST /v1/courses/{id}/delete`,
  `POST /v1/courses/{id}/restore`, `POST /v1/courses/{id}/unpublish`,
  `GET /v1/enrollments/{id}`, and `POST /v1/enrollments/{id}/withdraw`.
- Wired audit, metering, and NATS events for course/enrollment lifecycle and
  progress updates.
- Added `GET /v1/domain/status` returning `phase: wave2_w4` and capability
  planes.
- Added in-process validation/boundary tests plus an ignored Postgres
  integration test for progress history persistence.
- Added `scripts/helix_edu_smoke.ps1` and an `edu-smoke` CI job.
- Verification: `cargo test --workspace --all-features` pass,
  `cargo clippy --workspace --all-targets -- -D warnings` clean,
  local smoke PASS, CI run `29607668365` green.
- Out of scope for this packet: lessons, modules, assessments, rubrics,
  submissions, feedback, credentials, learner UI, offline sync, mastery graph,
  and certification issuance.

## 2026-07-17 — HelixCollab integration-test tenant seed fix

- `Run HelixCollab integration tests` failed on fresh CI Postgres with
  `audit_events_tenant_fk` violation because the deterministic local-dev tenant
  used by `dev_principal` was not seeded.
- Updated `projects/helix-collab/backend/src/domain/mod.rs` test harness to
  upsert the local-dev tenant before each ignored integration test.
- Verification: all 11 ignored Collab integration tests pass on a freshly
  migrated database; full CI run `29607668365` green.

## 2026-07-17 — HELIXCOMMERCE-FULL second-wave depth packet

- Activated goal: `HELIXCOMMERCE-FULL` (catalog order 5, port 8105).
- Added migration `0040_commerce_depth.sql` for order cancel tracking and indexes.
- Extended `CommerceRepo` with mixed-currency order rejection, atomic inventory
  reservation, order cancel with inventory restoration, and product update.
- Added backend routes: `PATCH /v1/products/{id}`, `POST /v1/orders/{id}/cancel`.
- Wired audit, metering, and NATS events for product update, order create, and
  order cancel.
- Added `scripts/helix_commerce_smoke.ps1` and a `commerce-smoke` CI job.
- Added unit test for checked money arithmetic and ignored integration test for
  the two-buyer race for the last unit.
- Verification: `cargo test --workspace --all-features` pass,
  `cargo clippy --workspace --all-targets -- -D warnings` clean,
  local smoke PASS, CI run green.
- Out of scope for this packet: carts, payment intents, fulfilment, returns,
  refunds, buyer UI, channels, and reconciliation.

## 2026-07-17 — HELIXINSIGHTS-FULL second-wave depth packet

- Activated goal: `HELIXINSIGHTS-FULL` (catalog order 4, port 8104).
- Added migration `0039_insights_depth.sql` for soft-delete columns and query indexes.
- Extended `InsightsRepo` with soft delete, tenant-wide metric list, filtered point
  queries, and in-process SQL aggregates (`sum`, `avg`, `min`, `max`, `count`).
- Added backend routes: `GET /v1/metrics`, `GET /v1/metrics/{id}`,
  `DELETE /v1/datasets/{id}`, `DELETE /v1/metrics/{id}`,
  `POST /v1/metrics/{id}/aggregate`, and filtered `GET /v1/metrics/{id}/points`.
- Wired audit, metering, and NATS events for dataset/metric lifecycle, point
  recording, and aggregate queries.
- Added `scripts/helix_insights_smoke.ps1` and an `insights-smoke` CI job.
- Verification: `cargo test --workspace --all-features` pass,
  `cargo clippy --workspace --all-targets -- -D warnings` clean,
  local smoke PASS, CI run `29597119407` green.
- Out of scope for this packet: decision records, alerts, reports, dashboards,
  forecasts, federated aggregates, and web UI changes.

## 2026-07-15 — Recorded smoke passes do not equal current Foundation Integrity

- The 2026-07-14 HelixCore and HelixCode results remain valid only for their
  recorded scope and tree. Later source changes require fresh reproduction.
- Foundation Integrity is currently `NOT_PROVEN`; no product is listed as having
  passed the new durability gate.
- Catalog registration, a migration, or a persistence scaffold is not proof of
  crash safety, concurrency safety, clean restore, or cross-platform release.
- Direct environment-secret handling is a legacy bootstrap surface. The target
  is a user-owned capability broker that never exposes raw values to agents.

## 2026-07-15 — Anvil location is unresolved and implementation-blocked

- The intended standalone root `C:\Users\divin\PROJECTS\HELIXANVIL` does not
  currently exist.
- A substantial scaffold exists at `projects/helix-anvil` even though older
  records describe the monorepo copy as reverted.
- This decision supersedes the older 2026-07-15 entry that called the external
  path the "Correct home." It is now only the intended path until the founder
  chooses the canonical home.
- No agent may create, move, merge, rename, delete, activate, or implement either
  location until the founder chooses the canonical home. Portfolio-last
  sequencing remains.

## 2026-07-15 — HelixCore and Aether identity boundary clarified

- HelixCore owns the canonical logical identities, bindings, policy decisions,
  and provider-neutral capability contracts used by HelixForge.
- Aether may attest those identities and act as the preferred external proof or
  capability-broker provider. It does not create a second canonical identity.
- A local HelixCore-compatible provider remains mandatory for offline and
  standalone operation.

## 2026-07-15 — Category-defining product program and five-year target sheets

- Founder directive: raise every HelixForge product to a category-defining,
  near-frontier but buildable ambition, and remove design guesswork from Kimi's
  execution path.
- Canonical target-state program: `docs/product-program/README.md`, shared
  contract, five-year roadmap, individual product sheets, and a gated Kimi
  master prompt.
- Target sheets describe future contracts, not present completion. Live source,
  fresh tests, and runtime evidence remain the truth for current maturity.
- Portfolio rule: repair the truthful shared foundation before deepening another
  thin product; at most one foundation gate and three product gates may be active.
- Existing sequence remains: HelixPulse is catalog-last and HelixAnvil remains a
  separate portfolio project activated last unless the founder explicitly
  changes that decision.
- Aether is the preferred provider-neutral proof, stable-project-identity, and
  capability-broker integration. HelixForge keeps a local fallback so Aether is
  not a hard runtime dependency.
- Every product now shares: stable identity, exact workload capabilities,
  all-or-nothing writes, durable visible jobs, truthful release gates, 30-day
  recovery, portable exit, and native Windows/macOS/Linux proof.
- This planning pass changes documentation only. It does not authorize real
  clinical, biological, financial, physical-control, payment, signing, permanent
  deletion, repository initialization, commit, or push actions.

## 2026-07-15 — Second-wave product depth starts at HelixFlow

- Program: `docs/SECOND_WAVE.md` — deepen thin catalog products in order; pulse monorepo-last; Anvil portfolio-last.
- W1 HelixFlow: migration `0032_flow_depth.sql` (step_events, run fields); get/list runs; in-process execute (`echo`/`set`/`fail`/`noop`); cancel; domain status; smoke `scripts/helix_flow_smoke.ps1`.
- Active goal `SECOND-WAVE-PRODUCT-DEPTH` / phase `w1_helix_flow`.

## 2026-07-15 — Portfolio sequencing: Anvil last

- User directive: **HELIXANVIL is the last portfolio project**, not the next after HelixCode.
- HelixCode checkpoint closed in monorepo.
- Next monorepo options: second-wave depth on thinner products, or **helix-pulse** when ready for catalog build-last (order 21).
- Anvil 001 stays parked until HelixForge product line (through pulse) is done.

## 2026-07-15 — HelixCode optionals + intentional limits

- Migration `0031_code_limits_ha.sql`: per-tenant breakglass + process_sessions sticky registry.
- Terminal: docker-prefer isolation via `run_isolated`; sticky register; relative path policy retained.
- Host isolation intentional: requires `HELIX_CODE_ALLOW_HOST_ISOLATION` (else docker if available).
- Per-tenant breakglass API `GET/PUT /v1/me/breakglass` merged with env in `tenant_policy`.
- DAP/terminal sticky_miss via `require_process_local`; HA_STICKY + PROD_ENV docs; update feed script.
- Smoke: DAP program launch attempt + breakglass GET; endstate+base PASS docker.

## 2026-07-15 — HelixCode close Kimi residuals (injection, host fallback, webhook, breakglass)

- `cmd_policy.rs`: shared isolation allowlist + shell metachar ban; enforced in `container::run_isolated`.
- Host fallback after docker failure requires `HELIX_CODE_ALLOW_HOST_FALLBACK` or `CI_ALLOW_ALL` + breakglass record.
- Terminal: relative-only file args; absolute/`..` denied.
- Webhooks: fail-closed host allowlist outside local; private targets decoupled from `HELIX_ALLOW_DEV_HEADERS`.
- `breakglass.rs`: process ring + tracing; DIRECT_PUSH/FORCE_PUSH/TERM/CI/HOST_FALLBACK recorded; exposed on domain status.
- Unit tests 26 PASS.

## 2026-07-15 — HelixCode A+C polish: Docker bind paths, SSRF pin, Electron, HA, Kimi re-run

- Windows Docker volume binds: strip `\\?\` and convert `C:\…` → `/c/…` (`container::normalize_docker_host_path`); proven with `isolation=docker` smoke PASS.
- Webhook SSRF: HTTPS-required outside local; optional `HELIX_CODE_WEBHOOK_ALLOW_HOSTS`; re-resolve at deliver; http pin-to-IP + Host header.
- Electron: package description/author, `electron-updater` dep, icon via `helix_code_make_icon.ps1`, auto-update feed env.
- Docs: `HA_STICKY.md`, `ENTERPRISE_CODESIGN.md` (OV/EV swap); packaging notes.
- Status-check green path in endstate smoke (docker CI succeeded then merge).
- Unit tests: 23 passed; endstate + base smoke PASS on docker isolation.

## 2026-07-15 — HelixCode P1: terminal allowlist, webhook SSRF, quota edges

- `terminal_policy.rs`: allowlist + hard denylist + shell metachar block; `HELIX_CODE_TERM_ALLOW_ALL` break-glass.
- `webhook_policy.rs`: SSRF checks at create + delivery; block metadata/private unless local allow; no redirects; timeout.
- Quotas: `max_agent_jobs_day` on agent job create; `max_sealed_bytes` on sealed put + MLS seal; usage exposes `sealed_bytes`.
- Smoke: SSRF metadata block, terminal deny powershell; unit tests for policies.

## 2026-07-15 — HelixCode P0: branch protection on smart HTTP + force push + status checks

- New `branch_protection.rs`: parse receive-pack pkt-lines; `require_pr` / `deny_force_push` shared rules; `required_status_checks` against pipeline runs by name + commit SHA.
- Smart HTTP `git-receive-pack` enforces protections **before** pack apply (deploy keys included).
- REST single/batch commits use shared `enforce_rest_commit`.
- PR merge validates approvals + required CI checks; pipeline trigger accepts `commit_sha`.
- Break-glass: `HELIX_CODE_ALLOW_DIRECT_PUSH`, `HELIX_CODE_ALLOW_FORCE_PUSH`.
- Smoke: required_status_checks block then satisfy path; unit tests for pkt-line parse.

## 2026-07-15 — HelixCode residual wave: org codesign + full DAP + Kimi CLI

- Org code-signing material under `Desktop/.keys/helixforge/code-signing/` (never in-repo); `scripts/helix_code_org_codesign.ps1` generates self-signed CodeSigning PFX or uses enterprise cert; `-Pack` runs electron-builder with CSC_* and signs `HelixCode.exe`.
- Full DAP client (`dap_client.rs`): Content-Length framing; prefers `lldb-dap` then `gdb --interpreter=dap`; detects Windows gdb builds without DAP UI; full control surface (step/pause/scopes/variables/evaluate); spawn PATH/PYTHONHOME for LLVM liblldb + Python 3.11.
- HTTP: `/v1/debug/adapters`, sessions continue/next/stepIn/stepOut/pause/threads/stack/scopes/variables/evaluate.
- Host note: scoop gdb 17.1 lacks dap; LLVM 22 lldb-dap works with python311.dll.
- External review: `scripts/kimi_helixcode_endstate_review.ps1` → `docs/reviews/HELIXCODE_ENDSTATE/KIMI_REPORT.md`.

## 2026-07-15 — HelixCode residuals: deploy keys, sticky LSP, DAP, web panels, Electron signing

- Migration `0030_code_residuals.sql`: deploy_keys, lsp_session_registry, debug_sessions.
- Smart HTTP accepts `x-helix-deploy-key` (hashed at rest, repo-scoped read/write).
- LSP registers sessions with `HELIX_CODE_INSTANCE_ID` for LB sticky routing; sticky_miss on wrong node.
- Debug: durable sessions + breakpoints + continue/stop; web Debug activity.
- Web: collab/terminal/debug/ext/settings activity tabs.
- Electron: `ELECTRON_PACKAGING.md` (CSC_* signing), optional auto-update via `HELIX_CODE_UPDATE_URL`.
- Smoke endstate extended for deploy key + debug + instance_id.

## 2026-07-15 — HelixCode ENDSTATE: close gaps 1–9

- Migration `0029_code_endstate.sql`: issues, PRs, reviews, protections, webhooks, CI fleet columns, runners, agent events, MLS devices/backups, user_settings, tenant_quotas.
- APIs: collab_api + endstate_api (list/cancel runs, artifact download, terminals, settings, quotas, LSP servers, extensions, debug launch).
- Branch protection enforced on direct commit; break-glass `HELIX_CODE_ALLOW_DIRECT_PUSH=1`.
- Quotas on repo create + pipeline runs; meters commits/issues/prs.
- Docs: THREAT_MODEL, BACKUP_RESTORE, `docs/reviews/HELIXCODE_ENDSTATE/*`.
- Smoke: `scripts/helix_code_endstate_smoke.ps1`.
- Phase `endstate`; Anvil still out of product.

## 2026-07-15 — HelixCode H6: CI Docker image with git + cargo

- Image: `projects/helix-code/docker/Dockerfile.ci` → tag `helixforge/helix-code-ci:local` (rust bookworm + git).
- Build: `projects/helix-code/docker/build-ci-image.ps1`.
- `container::docker_image()` prefers CI image when present; else `alpine:3.20`.
- `image_has_forge_tools()` — sandbox no longer force-hosts git/cargo when full CI image is active.
- Electron: `electron/wait-and-launch.cjs` polls web UI before spawning desktop shell.
- Domain: `docker_ci_image_preferred`, `docker_has_forge_tools`, `docker_image`.

## 2026-07-15 — HelixCode H5: split editor groups + Electron desktop shell

- **Split groups:** primary/secondary with shared document store; tab move; sash resize; Ctrl+\\ / Ctrl+Shift+\\; focus Ctrl+1/2.
- **Electron:** `web/electron/main.cjs` + `preload.cjs` (contextIsolation, no nodeIntegration); loads Next UI URL; menu actions via IPC `helix-menu`.
- Not a VS Code/Code-OSS binary embed — sovereign forge chrome around Monaco web shell.
- Domain planes: `split_editor_groups`, `electron_shell`.
- Prove: web typecheck; electron package install; smoke domain flags (code_oss remains).

## 2026-07-15 — HelixCode H4: Code-OSS depth (web shell, not VS Code fork)

- Not embedding Code-OSS/Electron; sovereign **Code-OSS-class** forge workspace on Next+Monaco.
- Backend: `GET /v1/repos/{id}/files` recursive index; `GET …/search` bounded content search; `POST …/commits/batch` multi-file one commit.
- Web shell: activity bar, multi-tab dirty tracking, breadcrumbs, status bar, command palette (Ctrl+Shift+P), quick open (Ctrl+P), Search/SCM/CI/Agents/MLS side views, problems/output/history panel.
- Domain phase `H_code_oss`; planes `code_oss`, `files_index`, `content_search`, `batch_commit`.
- Prove: unit files/search/batch in git_store; smoke files+search+batch; typecheck web.

## 2026-07-15 — HelixCode horizons (except Anvil): OpenMLS + Docker isolation + Monaco UX

- User: optional horizons **except Anvil** (native IDE stays HELIXANVIL).
- Migration `0028_code_openmls_docker.sql`: `code.mls_user_blobs`, `code.mls_groups_meta`, `isolation` on pipeline_runs/agent_jobs.
- **OpenMLS** forge API: identity, key-package, create/add/join group, encrypt/decrypt, `POST …/mls-sealed` (export-secret DEK + AES pack). Durable hydrate/persist via CodeRepoStore.
- **Container isolation**: `domain/container.rs` — `HELIX_CODE_ISOLATION=host|docker|auto`; CI steps via `run_isolated`; git/cargo steps force host when image is alpine; agent jobs run isolation probe.
- **Monaco UX**: completion + definition + hover providers; side tabs Problems/History/CI/Agents/MLS; top-bar run CI/agent/OpenMLS.
- Prove: unit mls alice/bob + durable blob + sandbox host isolation; smoke extended OpenMLS + isolation fields + domain planes.
- Anvil not registered as monorepo product #22.

## 2026-07-15 — HelixCode E5: sealed objects + crypto groups

- Migration `0027_code_e5_sealed.sql`: sealed metadata depth + `code.crypto_groups` + members.
- **Never store cleartext in MinIO** — HVA4 (`vault_seal_tenant`) or group AES-GCM with HVA4-wrapped DEK.
- Full CRUD: put/list/get(decrypt)/delete; group create + member add (DEK re-wrap).
- Classification: internal|confidential|secret|top_secret|mls; public rejected.
- Prove: smoke HVA4 roundtrip + group seal/decrypt + delete; phase `E5_sealed_crypto`.

## 2026-07-15 — HelixCode E4: multi-agent sandbox mesh

- Migration `0026_code_e4_agents.sql`: workdir, commit_sha, log_text, files_changed, agent_run_ids, mesh_steps.
- Isolated worktree per job (`.data/helix-code/agent-jobs/job-*`).
- Structured `patches[]` full-file apply + optional `unified_diff` via `git apply --check` then apply.
- Multi-agent: `helix-code-assistant` + `helix-code-patcher` via `agent_framework` (tool sandbox).
- Commit+push to bare origin when `commit: true`; audit `agent.job`.
- Prove: unit apply_patch_and_commit; smoke mesh job commits `src/e4_marker.rs`.

## 2026-07-15 — HelixCode E3: LSP bridge (rust-analyzer)

- JSON-RPC 2.0 over stdio; session checkout under `.data/helix-code/lsp-sessions`.
- API: open/close session, didOpen/didChange, diagnostics, hover, completion, definition.
- Resolve **toolchain-native** `rust-analyzer.exe` (MSVC) — rustup cargo/bin proxy fails under gnu host.
- Web: Monaco markers + hover strip; debounced didChange.
- Prove: unit path checks; smoke opens session, did-open, close; typecheck green.
- Install: `rustup component add rust-analyzer --toolchain stable-x86_64-pc-windows-msvc`.

## 2026-07-15 — HelixCode E2: sandbox CI + MinIO artifacts

- Migration `0025_code_e2_runners.sql`: run `workdir`/`artifacts`/`exit_code` + `code.pipeline_artifacts`.
- Runner clones bare repo to `.data/helix-code/ci-runs/run-*`, executes allowlisted steps with timeout (`HELIX_CODE_CI_STEP_TIMEOUT_SECS`, default 60s).
- Allowlist: echo, cargo test/check/build/clippy, git status/log/rev-parse/show, dir/ls/type/cat, rustc --version, etc. Deny curl/wget/rm -rf. Break-glass: `HELIX_CODE_CI_ALLOW_ALL=1`.
- Artifacts (incl. `helix-ci.log`) uploaded to MinIO; listed via `GET /v1/pipeline-runs/{id}/artifacts`.
- Prove: unit allowlist + sandbox_runs_echo_step; smoke PASS with `exit=0` + artifacts≥1.

## 2026-07-15 — HelixCode E1: gitoxide + Monaco workspace

- **gix 0.85** for object-plane **reads** (refs, head, tree, blob, log) with system-git fallback.
- Writes (init/commit) + smart HTTP packs remain system `git`.
- Domain: `git_backend=gix+cli`, phase `E1_workspace_web_and_gix`.
- Web: `@helixforge/helix-code-web` (Next 15 + Monaco) on port **3102**.
- Prove: unit roundtrip; smoke PASS with gix backend; web typecheck PASS.

## 2026-07-15 — HelixCode extreme E0: git forge foundation

- Started HelixCode extreme in-monorepo (`projects/helix-code`), not underscoped Monaco demo.
- Migration `0024_code_extreme.sql`: refs, workspaces, pipelines, pipeline_runs, agent_jobs, sealed_objects + repo head/visibility.
- Dual-plane git: bare repos; E0 object plane via system `git`; smart HTTP pack servers.
- CI: pipeline definition + in-process echo-safe runner (arbitrary shell deferred E2).
- Sealed objects: MinIO put + Postgres index; rejects `public` classification on sealed path.
- Prove: unit `init_list_tree_commit_roundtrip` + smoke **PASS** (E0).

## 2026-07-15 — Standalone HELIXANVIL (substrate); HelixCode extreme unblocked

- User intent: before HelixCode extreme, create **another project** for a completely native IDE from scratch — via portfolio **new-project substrate protocol** (`~/shared/substrate/new-project-prompt.md`), not a monorepo product deep-build.
- **Correction:** agent briefly embedded `helix-anvil` as HelixForge product #22 with ropey/eframe. **Reverted** (catalog 21 products, no workspace member, tree removed).
- Correct home: **`C:\Users\divin\PROJECTS\HELIXANVIL`** (standalone substrate project).
- HelixForge active goal returns to **HELIXCODE-EXTREME**.

## 2026-07-14 — HelixCollab polish pass

- UI: markdown toolbar/preview, focus mode, nested folder tree, mention autocomplete, toasts, conflict recovery, export.
- API: folder rename, comment PATCH, mention-suggest.
- Smoke extended for polish endpoints; web typecheck green.

## 2026-07-14 — HelixCollab comments, mentions, multi-workspace folders

- Migration 0019: `collab.folders`, `comments`, `mentions`, `documents.folder_id`.
- API: folders under workspace, comments CRUD-ish, mention inbox, doc move.
- UI: workspace selector, folder tree, Comments rail with @mentions, mention chips.
- Smoke PASS: folder doc + comment mentions=2 + inbox_alice=1.

## 2026-07-14 — HelixCollab Yjs web + richer workspace UI

- Browser `HelixYjsProvider` wires Yjs to collab WS (`crdt_sync`/`crdt_update`).
- 3-pane workspace UI: filterable doc list, Y.Text editor, People/Share/History rail.
- `pnpm typecheck` + `pnpm build` green for `@helixforge/helix-collab-web`.

## 2026-07-14 — HelixCollab depth 1–4 complete

- WS auth (`token`/`dev_user`) + durable WS patch with tenant/ACL.
- Share: POST/GET `/v1/documents/{id}/share` + web Invite UI.
- Optional CRDT: yrs rooms behind `HELIX_COLLAB_CRDT=1` (`crdt_update`/`crdt_sync`).
- Console Catalog deep-link “Open UI” → collab web :3101.
- Smoke PASS with ws_auth, durable_ws_patch, share, crdt flags.

## 2026-07-14 — HelixCollab deep slice started (product 1)

- Active goal switched to `HELIXCOLLAB-DEEP`.
- Collab: revisions/restore/delete, ACL on create, domain status, graceful shutdown, web editor :3101.
- Smoke `scripts/helix_collab_smoke.ps1` PASS (create, patch, 409 conflict, revisions, presence, restore).

## 2026-07-14 — HelixCore sovereign-ready local proof

- Deep smoke PASS: healthz 8080-8085, Kratos+Hydra, vault HVA3, payments, proxy, compliance.
- Depth: NATS JetStream (`HELIX_CORE` stream), PG shared rate buckets (0018), audit HMAC + genesis env, vault reencrypt on rotate, OTEL RNG+retries, gateway retry, Helm HPA/PDB, CI docker/helm.
- Goal `HELIXCORE-FULL` marked sovereign-ready (local); Stripe/Terraform/multi-region still out of scope.
- Portfolio can proceed to product forges; HelixPulse remains last.

## 2026-07-14 — HelixPulse (product 21) cataloged as build-last cluster plane

- New product: **helix-pulse** order 21, port **8121**, tier frontier.
- Vision: sovereign multi-tenant distributed memory (modern Redis-class), not a day-one Redis clone.
- **Build last**: after HelixCore FULL + products 1–20. Cluster (P3) is explicit endgame.
- Until then: Core uses NATS + Postgres + in-process rate limit.
- Scaffold: `projects/helix-pulse/` + `helix_pulse_api`; catalog + console fallback updated.

## 2026-07-14 — Kimi recommended build order 1–11 applied

- AetherID fail-closed (`HELIX_ALLOW_DEV_HEADERS` + local only); Platform gated; Hydra basic auth; auth audit.
- Service-kit CORS allowlist; local-only memory DB/NATS; rate-limit TTL eviction.
- Isolation: audit_recent tenant-scoped; agent get_run tenant check; unknown tenant inactive.
- Audit rehash: Platform + local + `HELIX_ALLOW_AUDIT_REHASH` only.
- Vault HVA4 Argon2id+AAD; KMS fallback only with `HELIX_VAULT_KMS_FALLBACK=1` in local.
- Gateway discovery envs, proxy timeouts, X-Forwarded-*.
- Billing idempotency + HMAC webhooks + i64 cents.
- Agent tool timeouts/allowlist/cancel; all services graceful shutdown.
- Dockerfile + entrypoint bin map; Helm secrets fail if empty; migration 0017.
- Tests expanded; `docs/runbooks/windows-msvc-toolchain.md`.

## 2026-07-14 — Kimi full HelixCore review complete

- Report: `docs/reviews/HELIXCORE_FULL/KIMI_REPORT.md`
- Verdict: **NOT_COMPLETE** (would FAIL production-sovereignty gate)
- Matrix: every capability PARTIAL; 15 P0, many P1/P2 findings
- Top work: AetherID fail-closed + isolation fixes + audit rehash restriction + Helm/Dockerfile
- PowerShell exit 1 was stderr noise from kimi; report body is complete (~18KB)

## 2026-07-14 — Closed remaining Core gaps (ACL, governance, region, WS, backup)

- Migration 0016: resource_acl, retention_policies, legal_holds, purpose_bindings, regions.
- Gateway APIs for ACL/governance/regions; product helpers on `clients.acl` / `governance` / `regions`.
- Gateway WebSocket proxy `/p/{slug}/ws/**` with auth header forward.
- Continuous backup loop + MinIO versioning script.
- Smoke: ACL check true; can_delete false under hold+retention; regions=3; caps resource_acl/governance/ws_proxy/multi_region.

## 2026-07-14 — Hydra OIDC + mTLS strategy + backup/DR

- Ory Hydra v2.2 in compose profile `ory` (migrate + serve); discovery on :4444.
- Hybrid auth: Kratos session first, then Hydra token introspect for Bearer OAuth tokens.
- Auth-adapter: `/v1/oidc/status|introspect|clients`.
- ADR-0011 mesh/mTLS; Helm linkerd/istio inject annotations; local cert script.
- Backup/restore scripts + `docs/runbooks/backup-dr.md`.
- Platform fit analysis: `docs/architecture/platform-enterprise-fit.md`.

## 2026-07-14 — HelixCore enterprise baseline (no Stripe)

- Migration 0015: tenant status, service_api_keys, vault_key_meta.
- Middleware: rate limit, body limit, security headers; API key auth (`hk_live_*`).
- Tenant lifecycle platform APIs; suspended tenants fail closed.
- Compliance export + summary; vault key rotation meta ledger.
- Graceful shutdown; Helm NetworkPolicy; enterprise runbook.
- Smoke: headers, tenant suspend, API key /me, compliance, key version++, status tier=sovereign-core.
- Stripe still out of scope; Kimi still operator-gated.

## 2026-07-14 — Ory Bearer fix + payment cancel/webhook

- Hybrid auth: session token failures no longer fall back to default dev user
  (only when `X-Helix-Dev-User` also present). Live Kratos whoami decode hardened.
- Smoke: Bearer `/v1/me` returns real Kratos session UUID; `auth_backend=ory_kratos`.
- Payments: cancel API, webhook stub, NATS/audit on create/paid; provider info endpoint.
- Gateway `/v1/core/status` capabilities block (HVA3, kms, payments, otlp).
- Script: `scripts/helixcore_deep_smoke.ps1`.

## 2026-07-14 — Closed "still later": Ory, KMS, OTEL, payments

- Live Kratos: compose profile ory; cipher secret length fix; auth `/v1/ory/register|login`.
- KMS: `KeyManagement` local + HTTP; HVA3 sealed secrets; vault `/v1/kms/*` software HSM.
- OTEL: OTLP/HTTP JSON to Jaeger 4318; request-id middleware emits spans.
- Payments: migration 0014; create/confirm local_sim; activates tenant plan.
- Smoke: kratos mode, HVA3 plain, kms wrap match, payment paid→team, otlp active.
- ADR-0010. Kimi still operator-gated.

## 2026-07-14 — HelixCore residuals: HVA2 DEK + MinIO + audit tenant

- ADR-0009: per-tenant software DEK (HVA2); MinIO S3 SigV4 ObjectStore in `vault_client`.
- Vault-service `value_b64` on objects → seal HVA2 → MinIO put; get opens DEK.
- Audit `list_for_tenant` + `GET /v1/audit/tenant`; request-id middleware on all services.
- Health includes minio check; core health reports `vault_crypto=postgres-aes-gcm-tenant-dek`.
- Smoke green: MinIO uploaded + bytes roundtrip; Kimi still blocked until operator asks.

## 2026-07-14 — HelixCore deep pass (Kimi still blocked)

- Raised FULL bar past prototype: user requested deep build before Kimi.
- Auth: cookie session + `X-Helix-Dev-Scopes` / `X-Helix-Dev-Residency`; fast Kratos probe; introspect/scopes.
- Vault: AES-GCM (ADR-0008) + MinIO object refs (`0013_vault_objects_billing.sql`).
- Billing: plan catalog, durable `PgPlanStore`, invoice-style `/summary`.
- Agents: durable runs + `utc_now` / `tenant_context` tools.
- Gateway: reverse proxy (ADR-0007) + `/v1/workspaces` cross-product.
- Deep smoke green: healthz 6×200, scope 403, AES roundtrip, object ref, plans, multi-tool agent, audit verified, core health ok.
- Kimi **not** run — continue deepening until operator says review.

## 2026-07-14 — HelixCore FULL build bar met (await Kimi) [superseded — bar raised]

- Audit rehash (local Admin) repairs chain; verify returns verified true.
- Smoke: ports 8080–8085 up; catalog 20; vault postgres; agent run ok.
- Workspace tests EXIT=0. Goal status was build complete → Kimi; **later same day: deepen more first**.
- Prototype limits documented: ADR-0007 edge, ADR-0008 vault XOR, Ory optional.

## 2026-07-14 — HelixCore depth slice (vault durable + core ops APIs)

- `PgVault` + migration `0011_vault.sql`; service_kit uses Postgres vault when DB up.
- Agent runs emit audit, meter, NATS `helix.core.agent.completed`.
- Gateway `/v1/core/status`; observability Prometheus export; billing/vault tenant isolation.
- ADR-0007: direct-port product edge (proxy deferred).
- UtcTimestamp truncated to micros for audit hash/Postgres alignment.

## 2026-07-14 — Goal HELIXCORE-FULL (build first, then Kimi)

- Single active goal: **HelixCore fully built** (`docs/goals/HELIXCORE_FULL.md`).
- Order is strict: implement + self-verify → only then Kimi full review.
- Kimi script reserved for post-build: `scripts/kimi_helixcore_full_review.ps1`.

## 2026-07-14 — HelixCore deep-build program (feature 010)

- Product widen (1–20 thin APIs) complete; focus shifts to **HelixCore depth**.
- Spec: `docs/features/010-helix-core-deep/requirements.md` (not “Completed”).
- Phases A–G: AetherID → audit/billing → vault → agents → observability → gateway → infra.
- Brand names (AetherID, etc.) map to existing crates; no rename required.

## 2026-07-14 — Widen products 10–20 (thin durable parent/child)

- Migration `0010_products_10_20.sql` + generators: Studio, SynthBio, Lex, Cura, Terra, Climate, Orbit, Quantum, Vita, Grid, Nova.
- Each: parent entity list/create/get + child list/create under parent; audit/meter on create.
- Generator: `scripts/widen_products_10_20.py`. All 20 products now have durable API slices.
- Goal: finish widening before deep product work.

## 2026-07-14 — HelixNetwork durable profiles/connections + shared dev tenant

- Migration `0009_network.sql`: `network.profiles`, `network.connections`, `network.opportunities`.
- `helix_db::NetworkRepo` + API on 8109.
- Dev identity: all `x-helix-dev-user` labels share one local tenant (`helixforge-tenant:local-dev`) so multi-user features work; user_id still per label.
- Smoke: Alice2/Bob2 same tenant → connection request/accept → opportunity create + audit.

## 2026-07-14 — HelixWell durable habits/check-ins

- Migration `0008_well.sql`: `well.habits`, `well.checkins`, `well.habit_logs`.
- `helix_db::WellRepo` — habits, mood/energy check-ins (1..=10), habit logs.
- API on 8108; smoke: habit → log qty 1 → checkin mood 8/energy 7 + audit.

## 2026-07-14 — HelixCapital durable accounts/journals

- Migration `0007_capital.sql`: `capital.accounts`, `capital.journals`, `capital.journal_lines`.
- `helix_db::CapitalRepo` — chart of accounts; balanced double-entry post (debits==credits); balance_cents debit+/credit−.
- API on 8107; smoke: Cash debit 5000 / Revenue credit 5000 → balances 5000 / -5000 + audit.

## 2026-07-14 — HelixEdu durable courses/enrollments

- Migration `0006_edu.sql`: `edu.courses`, `edu.enrollments`.
- `helix_db::EduRepo` — create/list/publish courses; enroll (unique learner/course); progress 0..=100 → completed.
- API on 8106; smoke: draft → publish → enroll → progress 100 + audit.

## 2026-07-14 — HelixCommerce durable catalog/orders

- Migration `0005_commerce.sql`: `commerce.products`, `commerce.orders`, `commerce.order_items`.
- `helix_db::CommerceRepo` — product CRUD list/create/get; order create with `FOR UPDATE` inventory decrement in a transaction.
- API on 8105: `/v1/products`, `/v1/orders` + audit/meter/NATS `helix.commerce.order.created`.
- Smoke: product inventory 10 → order qty 2 → inventory 8; total_cents 3998; audit chain ok.

## 2026-07-14 — Collab multi-instance WS fan-out via NATS

- ADR-0006 follow-up: `RealtimeHub` publishes `FanoutEnvelope` to `helix.collab.ws.{doc_id}`.
- Bridge task subscribes to `helix.collab.ws.>` and applies remote messages, skipping own `origin`.
- REST create/patch/presence already call `hub.publish` so they cross instances too.

## 2026-07-14 — HelixInsights durable datasets/metrics

- Migration `0004_insights.sql`: `insights.datasets`, `insights.metrics`, `insights.metric_points`.
- `helix_db::InsightsRepo` + product API routes on port 8104 (same memory-fallback pattern as Code/Flow).
- Smoke verified against docker Postgres (55432): create/list dataset, metric, point; audit `dataset.create` / `metric.create`.
- Workspace tests pass with MSVC toolchain override (gnu host lacks gcc for ring).

## 2026-07-14 — Durable audit/meter + HelixCollab/Code/Flow depth

- Added `helix_db` (sqlx migrations, PgAuditSink, PgMetering, WorkspaceRepo, CollabRepo, CodeRepoStore, FlowRepo).
- Service kit falls back to memory when Postgres is down; health reports postgres status.
- HelixCollab: durable documents, optimistic patches, presence, WebSocket hub.
- HelixCode + HelixFlow: durable repos/workflows reusing helix_db.
- ADRs 0005 (durable audit/meter), 0006 (Collab realtime).

## 2026-07-14 — Bootstrap HelixForge monorepo

- Created Rust-first monorepo with HelixCore + 20 product scaffolds.
- Auth: Ory Kratos/Hydra + local dev identity headers.
- Audit: in-memory BLAKE3 chain (Postgres table prepared in init SQL).
- JS: Next.js console + Turborepo/pnpm workspaces.
- Infra: docker-compose, Helm, ArgoCD app, Terraform network module skeleton.

## 2026-07-15 — Founder chooses canonical HelixAnvil home and ratifies Foundation Integrity umbrella

- **HelixAnvil canonical location:** `projects/helix-anvil` inside the HelixForge monorepo.
  - The intended external root `C:\Users\divin\PROJECTS\HELIXANVIL` is abandoned.
  - The existing scaffold at `projects/helix-anvil` becomes the canonical home.
  - Sequencing remains **portfolio-last**: Anvil implementation still waits until HelixForge
    monorepo endgame (including HelixPulse) is complete, unless the founder explicitly
    changes sequencing later.
  - Agents may now plan, spec, and maintain the `projects/helix-anvil` directory; they
    must still not create, move, merge, rename, delete, or implement it without an
    activated product packet.
- **Foundation Integrity umbrella `011-foundation-integrity` ratified.**
  - Child packet `011.1` — Repository boundary, clean build, and native CI design —
    is explicitly activated for implementation.
  - Child packets `011.2` and `011.3` are approved for documentation but not yet
    activated; implementation requires separate founder activation after `011.1` closes.
- Living state documents (`PROJECT_STATE.json`, `PROGRAM_MANIFEST.json`, `NEXT_ACTION.md`,
  `AGENTS.md`, `docs/product-program/specs/90-helix-anvil.md`) are updated to reflect
  these decisions in the same scoped change.

## 2026-07-15 — Foundation Integrity child packet 011.1 closed

- `011.1` — repository boundary, clean build, formatting, and native CI design —
  is proven and closed.
- Evidence recorded in `docs/features/011.1-git-clean-build-ci/status.json`.
- Checks passed on the current Windows MSVC host:
  - `cargo build --workspace`
  - `cargo test --workspace --all-features`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - TypeScript type checks for console, HelixCode web, and HelixCollab web
  - `helm lint infra/helm/helix-core`
- Cross-platform CI now runs native Windows, macOS, and Linux runners and records
  artifact hashes.
- The root Git repository is initialized with a `.gitignore`, `CONTRIBUTING.md`,
  and branch plan; no commits were made by the agent.
- The next action is a founder decision: activate `011.2`.

## 2026-07-16 — HELIXCORE-FULL end-to-end local verification script passes

- Created `scripts/verify_helixcore_full.ps1` to orchestrate the full local proof:
  start all 6 core services, run `helixcore_deep_smoke.ps1`, export a backup,
  restore to an isolated compose project, run `cargo test -p helix_db` against
  the restored database, and clean up.
- Added `deploy/local/restore.override.yml` with `!override` port mappings so the
  isolated restore project avoids colliding with the main local stack.
- Hardened `scripts/migrate-export.ps1` and `scripts/migrate-restore.ps1`:
  fresh `mc alias` per mirror, Windows PowerShell 5-compatible hashing, base
  compose file always included, readiness checks before restore, and fail-fast
  exit-code handling.
- Evidence: `scripts/verify_helixcore_full.ps1` exits 0 on Windows MSVC host with
  19/19 `helix_db` tests passing against the restored database.
- Remaining work: CI re-proof of the installer / live-restore roundtrip.

## 2026-07-17 — HELIXCOMMERCE-FULL closed and CI-proven

- Completed the HelixCommerce second-wave depth packet:
  - Migration `0040_commerce_depth.sql` + down migration: soft-delete columns,
    partial active indexes, and `commerce.order_status_history`.
  - Extended `CommerceRepo` (`crates/helix-db/src/commerce.rs`) with
    `soft_delete_product`, `restore_product`, `update_order_status`, and
    `cancel_order` with inventory restore + status-history audit.
  - Added routes in `projects/helix-commerce/backend/src/main.rs`:
    `POST /v1/products/{id}/delete`, `POST /v1/products/{id}/restore`,
    `POST /v1/orders/{id}/cancel`, `POST /v1/orders/{id}/status`, and
    `GET /v1/domain/status` with planes.
  - In-process tests: mixed-currency rejection, checked arithmetic.
  - Ignored data-plane race test `two_buyers_cannot_oversell_last_unit` for
    Postgres CI.
  - PowerShell smoke `scripts/helix_commerce_smoke.ps1` and `commerce-smoke`
    CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29599963866` is all green, including the new
    **HelixCommerce smoke** job.
- Commit `6bb1a65` pushed to `main`.
- Next action: founder selects the next explicit named goal.

## 2026-07-18 — HELIXCAPITAL-FULL closed and CI-proven

- Completed the HelixCapital second-wave depth packet:
  - Migration `0042_capital_depth.sql` + down migration: `closed_at`/`deleted_at`
    lifecycle columns on `capital.accounts`, `voided_at`/`void_reason` on
    `capital.journals`, `is_reversal` marker on `capital.journal_lines`, partial
    active indexes, and `capital.account_balance_history` snapshot table.
  - Extended `CapitalRepo` (`crates/helix-db/src/capital.rs`) with
    `update_account`, `close_account`, `reopen_account`,
    `soft_delete_account`, `void_journal`, `get_trial_balance`, and
    `record_balance_snapshot`.
  - Added routes in `projects/helix-capital/backend/src/main.rs`:
    `PATCH /v1/accounts/{id}`, `POST /v1/accounts/{id}/close`,
    `POST /v1/accounts/{id}/reopen`, `POST /v1/accounts/{id}/delete`,
    `POST /v1/journals/{id}/void`, `GET /v1/reports/trial-balance`,
    `POST /v1/reports/balance-snapshot`, and `GET /v1/domain/status` with planes.
  - In-process tests: validation, reversal, voiding, trial balance.
  - Ignored Postgres integration test for account lifecycle + journal void + trial balance.
  - PowerShell smoke `scripts/helix_capital_smoke.ps1` and `capital-smoke`
    CI job in `.github/workflows/ci.yml`.
- Verification:
  - `cargo fmt --all -- --check` clean.
  - `cargo clippy --workspace --all-targets -- -D warnings` clean.
  - `cargo test --workspace --all-features` clean.
  - Local smoke against Postgres/NATS/MinIO passes.
  - GitHub Actions run `29621350739` is all green, including the new
    **HelixCapital smoke** job and the gitleaks security scan.
- Commit `21e6522` pushed to `main`; a follow-up commit `4434982` fixed the
  gitleaks scan by fetching full history in the security job.
- `PROJECT_STATE.json` and `NEXT_ACTION.md` updated to mark HELIXCAPITAL-FULL
  closed and clear the active goal.
- Next action: founder selects the next explicit named goal.
