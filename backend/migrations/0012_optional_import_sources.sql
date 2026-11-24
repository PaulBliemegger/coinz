-- Preserve history and evidence while allowing imports without a named source.
CREATE TABLE import_batch_entries_saved AS SELECT * FROM import_batch_entries;
DROP TABLE import_batch_entries;
CREATE TABLE import_batches_new (
 id TEXT PRIMARY KEY, user_id TEXT NOT NULL REFERENCES users(id),
 source_id TEXT, filename TEXT NOT NULL,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 start_date TEXT, end_date TEXT,
 added INTEGER NOT NULL DEFAULT 0, matched INTEGER NOT NULL DEFAULT 0,
 known INTEGER NOT NULL DEFAULT 0, excluded INTEGER NOT NULL DEFAULT 0,
 removed INTEGER NOT NULL DEFAULT 0,
 interpretation TEXT,
 UNIQUE(id,user_id),
 FOREIGN KEY(source_id,user_id) REFERENCES transaction_sources(id,user_id)
);
INSERT INTO import_batches_new SELECT id,user_id,source_id,filename,created_at,start_date,end_date,added,matched,known,excluded,removed,interpretation FROM import_batches;
DROP TABLE import_batches;
ALTER TABLE import_batches_new RENAME TO import_batches;
CREATE TABLE import_batch_entries (
 batch_id TEXT NOT NULL, user_id TEXT NOT NULL,
 transaction_id TEXT REFERENCES transactions(id) ON DELETE SET NULL,
 role TEXT NOT NULL CHECK(role IN ('added','matched','known')),
 revision INTEGER NOT NULL,
 FOREIGN KEY(batch_id,user_id) REFERENCES import_batches(id,user_id)
);
INSERT INTO import_batch_entries SELECT * FROM import_batch_entries_saved;
DROP TABLE import_batch_entries_saved;
CREATE INDEX batch_entries_batch ON import_batch_entries(batch_id,user_id);
CREATE INDEX batch_entries_transaction ON import_batch_entries(transaction_id,user_id);

CREATE TABLE import_records_new (
 user_id TEXT NOT NULL REFERENCES users(id),
 source_id TEXT,
 scope_id TEXT NOT NULL CHECK(scope_id=COALESCE(source_id,'')),
 file_hash TEXT NOT NULL, record_number INTEGER NOT NULL, external_id TEXT NOT NULL,
 date TEXT NOT NULL, description TEXT NOT NULL, amount_cents INTEGER NOT NULL,
 transaction_id TEXT REFERENCES transactions(id) ON DELETE SET NULL,
 disposition TEXT NOT NULL CHECK(disposition IN ('accepted','skipped')),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 interpretation TEXT,
 PRIMARY KEY(user_id,scope_id,file_hash,record_number),
 FOREIGN KEY(source_id,user_id) REFERENCES transaction_sources(id,user_id)
);
INSERT INTO import_records_new SELECT user_id,source_id,COALESCE(source_id,''),file_hash,record_number,external_id,date,description,amount_cents,transaction_id,disposition,created_at,interpretation FROM import_records;
DROP TRIGGER forget_deleted_import_evidence;
DROP TABLE import_records;
ALTER TABLE import_records_new RENAME TO import_records;
CREATE INDEX import_external_id ON import_records(user_id,source_id,external_id);
CREATE INDEX import_transaction ON import_records(transaction_id);
-- Deleting an entry must still allow deliberate reimport.
CREATE TRIGGER forget_deleted_import_evidence BEFORE DELETE ON transactions BEGIN
 DELETE FROM import_records WHERE transaction_id=OLD.id AND user_id=OLD.user_id;
END;
