-- Drop the HelixCode allocation counters (numbers fall back to MAX+1
-- reads; the counters are derived state).
DROP TABLE IF EXISTS code.number_counters;
