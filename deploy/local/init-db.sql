-- HelixForge local database bootstrap (extensions + companion DBs only).
-- Application tables are owned by sqlx migrations in crates/helix-db/migrations/.

CREATE DATABASE kratos;
CREATE DATABASE hydra;

\c helixforge

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
-- TimescaleDB is available on timescale/timescaledb-ha image
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- Optional: promote meter hypertable after sqlx migrations have created the table.
-- Application schema is applied by helix_db::connect_and_migrate on service boot.
