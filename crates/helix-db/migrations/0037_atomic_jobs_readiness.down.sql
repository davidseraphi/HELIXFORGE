-- Rollback for Foundation Integrity 011.3.

DROP TABLE IF EXISTS helix_core.jobs;
DROP TABLE IF EXISTS helix_core.outbox;
