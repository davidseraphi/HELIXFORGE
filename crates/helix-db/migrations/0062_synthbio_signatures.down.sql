-- Drop the HelixSynthBio e-signatures layer (S5).
DROP TRIGGER IF EXISTS signatures_immutable ON synthbio.signatures;
DROP TABLE IF EXISTS synthbio.signatures;
ALTER TABLE synthbio.risk_cases DROP COLUMN IF EXISTS locked_at;
