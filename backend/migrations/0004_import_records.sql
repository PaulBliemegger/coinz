-- Keep original bank fields independent of subsequent transaction edits/deletion.
CREATE TABLE import_records (
    user_id TEXT NOT NULL REFERENCES users(id),
    source_id TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    record_number INTEGER NOT NULL,
    external_id TEXT NOT NULL,
    date TEXT NOT NULL,
    description TEXT NOT NULL,
    amount_cents INTEGER NOT NULL,
    transaction_id TEXT REFERENCES transactions(id) ON DELETE SET NULL,
    disposition TEXT NOT NULL CHECK (disposition IN ('accepted', 'skipped')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (user_id, source_id, file_hash, record_number),
    FOREIGN KEY (source_id, user_id) REFERENCES transaction_sources(id, user_id)
);
CREATE INDEX import_external_id ON import_records(user_id, source_id, external_id);
CREATE INDEX import_transaction ON import_records(transaction_id);
