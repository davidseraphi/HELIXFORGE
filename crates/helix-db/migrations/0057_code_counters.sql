-- HelixCode atomic allocation counters (issues / PRs / agent event seqs).
-- One row per allocation scope, incremented under a row lock in a single
-- INSERT ... ON CONFLICT DO UPDATE ... RETURNING statement — allocation
-- is fully serialized with no MAX+1 read window (including zero rows).
CREATE TABLE IF NOT EXISTS code.number_counters (
    tenant_id UUID NOT NULL,
    scope_kind TEXT NOT NULL,
    scope_id UUID NOT NULL,
    next_value BIGINT NOT NULL,
    PRIMARY KEY (tenant_id, scope_kind, scope_id)
);

-- Backfill from live data so existing repos never re-allocate an in-use
-- number.
INSERT INTO code.number_counters (tenant_id, scope_kind, scope_id, next_value)
SELECT tenant_id, 'issue', repo_id, MAX(number)::bigint + 1
FROM code.issues
GROUP BY tenant_id, repo_id
ON CONFLICT (tenant_id, scope_kind, scope_id) DO NOTHING;

INSERT INTO code.number_counters (tenant_id, scope_kind, scope_id, next_value)
SELECT tenant_id, 'pr', repo_id, MAX(number)::bigint + 1
FROM code.pull_requests
GROUP BY tenant_id, repo_id
ON CONFLICT (tenant_id, scope_kind, scope_id) DO NOTHING;

INSERT INTO code.number_counters (tenant_id, scope_kind, scope_id, next_value)
SELECT tenant_id, 'agent_event', job_id, MAX(seq)::bigint + 1
FROM code.agent_job_events
GROUP BY tenant_id, job_id
ON CONFLICT (tenant_id, scope_kind, scope_id) DO NOTHING;
