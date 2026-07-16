-- HelixCollab depth: folders, comments, mentions.

ALTER TABLE collab.documents
    ADD COLUMN IF NOT EXISTS folder_id UUID;

CREATE TABLE IF NOT EXISTS collab.folders (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    parent_id UUID REFERENCES collab.folders(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS collab_folders_workspace_idx
    ON collab.folders (tenant_id, workspace_id, parent_id);

CREATE TABLE IF NOT EXISTS collab.comments (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES collab.documents(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES collab.comments(id) ON DELETE CASCADE,
    author_id UUID NOT NULL,
    author_label TEXT NOT NULL DEFAULT '',
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS collab_comments_doc_idx
    ON collab.comments (document_id, created_at)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS collab.mentions (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    document_id UUID NOT NULL REFERENCES collab.documents(id) ON DELETE CASCADE,
    comment_id UUID NOT NULL REFERENCES collab.comments(id) ON DELETE CASCADE,
    mentioned_user_id UUID,
    mentioned_label TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS collab_mentions_user_idx
    ON collab.mentions (tenant_id, mentioned_user_id, created_at DESC);

CREATE INDEX IF NOT EXISTS collab_mentions_doc_idx
    ON collab.mentions (document_id, created_at DESC);

CREATE INDEX IF NOT EXISTS collab_documents_folder_idx
    ON collab.documents (tenant_id, folder_id);
