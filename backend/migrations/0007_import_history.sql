ALTER TABLE transactions ADD COLUMN revision INTEGER NOT NULL DEFAULT 0;
CREATE TRIGGER track_transaction_edits AFTER UPDATE OF date,description,amount_cents,source_id ON transactions
WHEN OLD.date IS NOT NEW.date OR OLD.description IS NOT NEW.description
 OR OLD.amount_cents IS NOT NEW.amount_cents OR OLD.source_id IS NOT NEW.source_id
BEGIN
 UPDATE transactions SET revision=OLD.revision+1 WHERE id=NEW.id;
END;
CREATE TABLE import_batches (
 id TEXT PRIMARY KEY, user_id TEXT NOT NULL REFERENCES users(id),
 source_id TEXT NOT NULL, filename TEXT NOT NULL,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 start_date TEXT, end_date TEXT,
 added INTEGER NOT NULL DEFAULT 0, matched INTEGER NOT NULL DEFAULT 0,
 known INTEGER NOT NULL DEFAULT 0, excluded INTEGER NOT NULL DEFAULT 0,
 removed INTEGER NOT NULL DEFAULT 0,
 UNIQUE(id,user_id),
 FOREIGN KEY(source_id,user_id) REFERENCES transaction_sources(id,user_id)
);
CREATE TABLE import_batch_entries (
 batch_id TEXT NOT NULL, user_id TEXT NOT NULL,
 transaction_id TEXT REFERENCES transactions(id) ON DELETE SET NULL,
 role TEXT NOT NULL CHECK(role IN ('added','matched','known')),
 revision INTEGER NOT NULL,
 FOREIGN KEY(batch_id,user_id) REFERENCES import_batches(id,user_id)
);
CREATE INDEX batch_entries_batch ON import_batch_entries(batch_id,user_id);
CREATE INDEX batch_entries_transaction ON import_batch_entries(transaction_id,user_id);
CREATE INDEX transaction_period ON transactions(user_id,date DESC);
