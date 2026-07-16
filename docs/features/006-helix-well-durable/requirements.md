# 006 — HelixWell durable habits & check-ins

### ADDED Requirements

#### Requirement
WHEN Postgres is available HelixWell SHALL persist habits via
`helix_db::WellRepo` after migration `0008_well.sql`.

##### Scenario
- GIVEN docker Postgres is healthy
- WHEN `POST /v1/habits` creates a daily habit
- THEN the row is active and audit logs `habit.create`

#### Requirement
WHEN a check-in is recorded mood and energy SHALL be constrained to 1..=10.

##### Scenario
- GIVEN an authenticated learner
- WHEN `POST /v1/checkins` with mood 8 and energy 7
- THEN the check-in is stored and listed by GET
