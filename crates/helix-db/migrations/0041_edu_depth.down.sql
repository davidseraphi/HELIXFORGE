DROP INDEX IF EXISTS edu_progress_history_tenant_idx;
DROP INDEX IF EXISTS edu_progress_history_enrollment_idx;
DROP TABLE IF EXISTS edu.enrollment_progress_history;
DROP INDEX IF EXISTS edu_courses_active_idx;
ALTER TABLE edu.courses DROP COLUMN IF EXISTS deleted_at;
