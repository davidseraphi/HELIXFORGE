-- Drop the HelixSynthBio registry layer.
DROP TRIGGER IF EXISTS design_versions_immutable ON synthbio.design_versions;
DROP TRIGGER IF EXISTS lineage_events_immutable ON synthbio.lineage_events;
DROP FUNCTION IF EXISTS synthbio.immutable_record();
DROP TABLE IF EXISTS synthbio.lineage_edges;
DROP TABLE IF EXISTS synthbio.lineage_events;
DROP TABLE IF EXISTS synthbio.risk_cases;
DROP TABLE IF EXISTS synthbio.design_versions;
DROP TABLE IF EXISTS synthbio.registry_designs;
DROP TABLE IF EXISTS synthbio.accession_counters;
