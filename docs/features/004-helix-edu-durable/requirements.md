# 004 — HelixEdu durable courses & enrollments

### ADDED Requirements

#### Requirement
WHEN Postgres is available HelixEdu SHALL persist courses via
`helix_db::EduRepo` after migration `0006_edu.sql`.

##### Scenario
- GIVEN docker Postgres is healthy
- WHEN `POST /v1/courses` creates a course
- THEN status is `draft` and audit logs `course.create`

#### Requirement
WHEN a course is published and a learner enrolls the system SHALL store a unique
enrollment per (tenant, course, learner) and track progress_pct 0..=100.

##### Scenario
- GIVEN a published course
- WHEN `POST /v1/enrollments` succeeds and progress is set to 100
- THEN enrollment status is `completed` with completed_at set
