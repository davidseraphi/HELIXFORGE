-- HelixCollab deeper: e2ee flag, archive/pin, anchored resolvable comments, activity.

ALTER TABLE collab.documents
    ADD COLUMN IF NOT EXISTS encrypted BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS pinned BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX IF NOT EXISTS collab_documents_active_idx
    ON collab.documents (tenant_id, workspace_id, updated_at DESC)
    WHERE archived_at IS NULL;

ALTER TABLE collab.comments
    ADD COLUMN IF NOT EXISTS anchor_start INT,
    ADD COLUMN IF NOT EXISTS anchor_end INT,
    ADD COLUMN IF NOT EXISTS anchor_quote TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS resolved_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS resolved_by UUID;

CREATE INDEX IF NOT EXISTS collab_comments_unresolved_idx
    ON collab.comments (document_id, created_at)
    WHERE deleted_at IS NULL AND resolved_at IS NULL;

CREATE TABLE IF NOT EXISTS collab.activity (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES collab.documents(id) ON DELETE CASCADE,
    actor_id UUID,
    actor_label TEXT NOT NULL DEFAULT '',
    action TEXT NOT NULL,
    detail JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS collab_activity_doc_idx
    ON collab.activity (document_id, created_at DESC);
