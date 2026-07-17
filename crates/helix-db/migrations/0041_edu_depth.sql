-- HelixEdu W4 depth: course soft-delete + enrollment progress history
CREATE SCHEMA IF NOT EXISTS edu;

ALTER TABLE edu.courses
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Active course lookups for enrollment and listing
CREATE INDEX IF NOT EXISTS edu_courses_active_idx
    ON edu.courses (tenant_id, status)
    WHERE deleted_at IS NULL;

-- Durable progress audit trail for every enrollment
CREATE TABLE IF NOT EXISTS edu.enrollment_progress_history (
    id UUID PRIMARY KEY,
    enrollment_id UUID NOT NULL REFERENCES edu.enrollments(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    progress_pct INT NOT NULL CHECK (progress_pct >= 0 AND progress_pct <= 100),
    status TEXT NOT NULL,
    actor_id UUID,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS edu_progress_history_enrollment_idx
    ON edu.enrollment_progress_history (enrollment_id, recorded_at DESC);

CREATE INDEX IF NOT EXISTS edu_progress_history_tenant_idx
    ON edu.enrollment_progress_history (tenant_id);
