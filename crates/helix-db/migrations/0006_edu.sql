-- HelixEdu durable courses + enrollments (reuse helix_core tenancy)
CREATE SCHEMA IF NOT EXISTS edu;

CREATE TABLE IF NOT EXISTS edu.courses (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    level TEXT NOT NULL DEFAULT 'beginner',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, slug)
);

CREATE INDEX IF NOT EXISTS edu_courses_tenant_idx ON edu.courses (tenant_id);
CREATE INDEX IF NOT EXISTS edu_courses_status_idx ON edu.courses (tenant_id, status);

CREATE TABLE IF NOT EXISTS edu.enrollments (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    course_id UUID NOT NULL REFERENCES edu.courses(id) ON DELETE CASCADE,
    learner_id UUID NOT NULL,
    learner_label TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'active',
    progress_pct INT NOT NULL DEFAULT 0 CHECK (progress_pct >= 0 AND progress_pct <= 100),
    enrolled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    UNIQUE (tenant_id, course_id, learner_id)
);

CREATE INDEX IF NOT EXISTS edu_enrollments_tenant_idx ON edu.enrollments (tenant_id);
CREATE INDEX IF NOT EXISTS edu_enrollments_course_idx ON edu.enrollments (course_id);
CREATE INDEX IF NOT EXISTS edu_enrollments_learner_idx ON edu.enrollments (tenant_id, learner_id);
