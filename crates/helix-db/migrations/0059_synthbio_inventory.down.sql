-- Drop the HelixSynthBio inventory layer (S2).
DROP TRIGGER IF EXISTS custody_events_immutable ON synthbio.custody_events;
DROP TABLE IF EXISTS synthbio.custody_events;
DROP TABLE IF EXISTS synthbio.samples;
