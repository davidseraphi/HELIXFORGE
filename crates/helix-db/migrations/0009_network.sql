-- HelixNetwork durable profiles, connections, opportunities
CREATE SCHEMA IF NOT EXISTS network;

CREATE TABLE IF NOT EXISTS network.profiles (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    display_name TEXT NOT NULL,
    headline TEXT NOT NULL DEFAULT '',
    bio TEXT NOT NULL DEFAULT '',
    skills JSONB NOT NULL DEFAULT '[]'::jsonb,
    location TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, user_id)
);

CREATE INDEX IF NOT EXISTS network_profiles_tenant_idx ON network.profiles (tenant_id);

CREATE TABLE IF NOT EXISTS network.connections (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    from_profile_id UUID NOT NULL REFERENCES network.profiles(id) ON DELETE CASCADE,
    to_profile_id UUID NOT NULL REFERENCES network.profiles(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    message TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (from_profile_id <> to_profile_id),
    UNIQUE (tenant_id, from_profile_id, to_profile_id)
);

CREATE INDEX IF NOT EXISTS network_connections_tenant_idx ON network.connections (tenant_id);
CREATE INDEX IF NOT EXISTS network_connections_to_idx ON network.connections (to_profile_id, status);

CREATE TABLE IF NOT EXISTS network.opportunities (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    owner_profile_id UUID NOT NULL REFERENCES network.profiles(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    kind TEXT NOT NULL DEFAULT 'role',
    status TEXT NOT NULL DEFAULT 'open',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS network_opportunities_tenant_idx ON network.opportunities (tenant_id);
CREATE INDEX IF NOT EXISTS network_opportunities_status_idx ON network.opportunities (tenant_id, status);
