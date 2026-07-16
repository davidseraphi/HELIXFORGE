-- HelixCode E4: agent sandbox mesh metadata on jobs

ALTER TABLE code.agent_jobs
    ADD COLUMN IF NOT EXISTS workdir TEXT,
    ADD COLUMN IF NOT EXISTS commit_sha TEXT,
    ADD COLUMN IF NOT EXISTS log_text TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS files_changed JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS agent_run_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS mesh_steps JSONB NOT NULL DEFAULT '[]'::jsonb;

CREATE INDEX IF NOT EXISTS code_agent_jobs_tenant_status_idx
    ON code.agent_jobs (tenant_id, status);
