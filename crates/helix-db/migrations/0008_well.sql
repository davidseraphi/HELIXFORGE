-- HelixWell durable habits + wellness check-ins
CREATE SCHEMA IF NOT EXISTS well;

CREATE TABLE IF NOT EXISTS well.habits (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    cadence TEXT NOT NULL DEFAULT 'daily',
    target_per_period INT NOT NULL DEFAULT 1 CHECK (target_per_period >= 1),
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS well_habits_tenant_idx ON well.habits (tenant_id);
CREATE INDEX IF NOT EXISTS well_habits_owner_idx ON well.habits (tenant_id, owner_id);

CREATE TABLE IF NOT EXISTS well.checkins (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    mood INT NOT NULL CHECK (mood >= 1 AND mood <= 10),
    energy INT NOT NULL CHECK (energy >= 1 AND energy <= 10),
    notes TEXT NOT NULL DEFAULT '',
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS well_checkins_tenant_idx ON well.checkins (tenant_id);
CREATE INDEX IF NOT EXISTS well_checkins_user_idx ON well.checkins (tenant_id, user_id, recorded_at DESC);

CREATE TABLE IF NOT EXISTS well.habit_logs (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    habit_id UUID NOT NULL REFERENCES well.habits(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    quantity INT NOT NULL DEFAULT 1 CHECK (quantity >= 1),
    notes TEXT NOT NULL DEFAULT '',
    logged_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS well_habit_logs_habit_idx ON well.habit_logs (habit_id, logged_at DESC);
CREATE INDEX IF NOT EXISTS well_habit_logs_tenant_idx ON well.habit_logs (tenant_id);
