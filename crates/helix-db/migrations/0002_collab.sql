-- HelixCollab durable documents + presence sessions
CREATE SCHEMA IF NOT EXISTS collab;

CREATE TABLE IF NOT EXISTS collab.documents (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    workspace_id UUID,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    version INT NOT NULL DEFAULT 1 CHECK (version >= 1),
    created_by UUID,
    updated_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS collab_documents_tenant_idx ON collab.documents (tenant_id);
CREATE INDEX IF NOT EXISTS collab_documents_workspace_idx ON collab.documents (workspace_id);

CREATE TABLE IF NOT EXISTS collab.document_revisions (
    id UUID PRIMARY KEY,
    document_id UUID NOT NULL REFERENCES collab.documents(id) ON DELETE CASCADE,
    version INT NOT NULL,
    content TEXT NOT NULL,
    author_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (document_id, version)
);

CREATE TABLE IF NOT EXISTS collab.presence (
    document_id UUID NOT NULL,
    user_id UUID NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    cursor_pos INT NOT NULL DEFAULT 0,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (document_id, user_id)
);
