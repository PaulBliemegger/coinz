-- Older entries may have been imported or manually matched; do not invent origin.
ALTER TABLE transactions ADD COLUMN origin TEXT NOT NULL DEFAULT 'unknown'
    CHECK (origin IN ('unknown', 'manual', 'csv'));
