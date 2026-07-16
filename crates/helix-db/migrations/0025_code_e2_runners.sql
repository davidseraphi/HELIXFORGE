-- HelixCode E2: pipeline runner artifacts + richer run metadata

ALTER TABLE code.pipeline_runs
    ADD COLUMN IF NOT EXISTS workdir TEXT,
    ADD COLUMN IF NOT EXISTS artifacts JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS exit_code INT;

CREATE TABLE IF NOT EXISTS code.pipeline_artifacts (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES code.pipeline_runs(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    repo_id UUID NOT NULL,
    name TEXT NOT NULL,
    storage_key TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    byte_len BIGINT NOT NULL DEFAULT 0,
    sha256 TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS code_pipeline_artifacts_run_idx ON code.pipeline_artifacts (run_id);
CREATE INDEX IF NOT EXISTS code_pipeline_artifacts_tenant_idx ON code.pipeline_artifacts (tenant_id);
