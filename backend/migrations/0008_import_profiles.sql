CREATE TABLE import_profiles (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
    layout TEXT NOT NULL,
    UNIQUE(id,user_id)
);
-- Immutable interpretation snapshots; NULL identifies the legacy example parser.
ALTER TABLE import_batches ADD COLUMN interpretation TEXT;
ALTER TABLE import_records ADD COLUMN interpretation TEXT;
