-- Drop the HelixSynthBio claims layer (S4).
DROP TRIGGER IF EXISTS evidence_links_immutable ON synthbio.evidence_links;
DROP TRIGGER IF EXISTS notes_immutable ON synthbio.notes;
DROP TABLE IF EXISTS synthbio.notes;
DROP TABLE IF EXISTS synthbio.evidence_links;
DROP TABLE IF EXISTS synthbio.claims;
