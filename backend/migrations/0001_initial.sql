-- Clean pre-v0.1 baseline. Earlier development databases require an explicit reset.
-- Future schema changes use new migrations; startup never resets data.

CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE transaction_sources (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 80),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (id, user_id)
);

CREATE TABLE transactions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    source_id TEXT,
    date TEXT NOT NULL,
    description TEXT NOT NULL CHECK (length(description) BETWEEN 1 AND 200),
    amount_cents INTEGER NOT NULL CHECK (amount_cents != 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    origin TEXT NOT NULL DEFAULT 'unknown' CHECK (origin IN ('unknown', 'manual', 'csv')),
    revision INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (source_id, user_id) REFERENCES transaction_sources(id, user_id)
);

CREATE TABLE import_profiles (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
    layout TEXT NOT NULL,
    UNIQUE(id,user_id)
);

CREATE TABLE classifications (
 id TEXT PRIMARY KEY NOT NULL,
 user_id TEXT NOT NULL REFERENCES users(id),
 kind TEXT NOT NULL CHECK(kind IN ('category','label')),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 60),
 name_key TEXT NOT NULL,
 color TEXT NOT NULL CHECK(color IN ('sage','clay','sand','lavender','blue')),
 archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0,1)),
 UNIQUE(id,user_id,kind), UNIQUE(user_id,kind,name_key)
);

CREATE TABLE entry_categories (
 transaction_id TEXT PRIMARY KEY NOT NULL,
 user_id TEXT NOT NULL,
 category_id TEXT NOT NULL,
 kind TEXT NOT NULL DEFAULT 'category' CHECK(kind='category'),
 FOREIGN KEY(transaction_id,user_id) REFERENCES transactions(id,user_id) ON DELETE CASCADE,
 FOREIGN KEY(category_id,user_id,kind) REFERENCES classifications(id,user_id,kind)
);

CREATE TABLE entry_labels (
 transaction_id TEXT NOT NULL,
 user_id TEXT NOT NULL,
 label_id TEXT NOT NULL,
 kind TEXT NOT NULL DEFAULT 'label' CHECK(kind='label'),
 PRIMARY KEY(transaction_id,label_id),
 FOREIGN KEY(transaction_id,user_id) REFERENCES transactions(id,user_id) ON DELETE CASCADE,
 FOREIGN KEY(label_id,user_id,kind) REFERENCES classifications(id,user_id,kind)
);

CREATE TABLE classification_setup (
 user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id)
);

CREATE TABLE label_rule_state (
 user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id),
 revision INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE label_rules (
 id TEXT PRIMARY KEY NOT NULL,
 user_id TEXT NOT NULL REFERENCES users(id),
 name TEXT NOT NULL,
 operator TEXT NOT NULL CHECK(operator IN ('contains','contains_word','equals')),
 pattern TEXT NOT NULL,
 source_id TEXT,
 category_id TEXT,
 category_kind TEXT NOT NULL DEFAULT 'category' CHECK(category_kind='category'),
 enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
 revision INTEGER NOT NULL DEFAULT 1,
 pause_reason TEXT,
 UNIQUE(id,user_id),
 FOREIGN KEY(source_id,user_id) REFERENCES transaction_sources(id,user_id),
 FOREIGN KEY(category_id,user_id,category_kind) REFERENCES classifications(id,user_id,kind)
);

CREATE TABLE label_rule_labels (
 rule_id TEXT NOT NULL,
 user_id TEXT NOT NULL,
 label_id TEXT NOT NULL,
 kind TEXT NOT NULL DEFAULT 'label' CHECK(kind='label'),
 PRIMARY KEY(rule_id,label_id),
 FOREIGN KEY(rule_id,user_id) REFERENCES label_rules(id,user_id) ON DELETE CASCADE,
 FOREIGN KEY(label_id,user_id,kind) REFERENCES classifications(id,user_id,kind)
);

CREATE TABLE budget_months (
 user_id TEXT NOT NULL REFERENCES users(id),
 month TEXT NOT NULL,
 savings_cents INTEGER NOT NULL CHECK(savings_cents>=0),
 revision INTEGER NOT NULL DEFAULT 1,
 PRIMARY KEY(user_id,month)
);

CREATE TABLE category_budgets (
 user_id TEXT NOT NULL,
 month TEXT NOT NULL,
 category_id TEXT NOT NULL,
 kind TEXT NOT NULL DEFAULT 'category' CHECK(kind='category'),
 amount_cents INTEGER NOT NULL CHECK(amount_cents>=0),
 PRIMARY KEY(user_id,month,category_id),
 FOREIGN KEY(user_id,month) REFERENCES budget_months(user_id,month) ON DELETE CASCADE,
 FOREIGN KEY(category_id,user_id,kind) REFERENCES classifications(id,user_id,kind)
);

CREATE TABLE expected_income (
 id TEXT PRIMARY KEY NOT NULL,
 user_id TEXT NOT NULL,
 month TEXT NOT NULL,
 name TEXT NOT NULL,
 amount_cents INTEGER NOT NULL CHECK(amount_cents>=0),
 complete INTEGER NOT NULL CHECK(complete IN (0,1)),
 UNIQUE(id,user_id),
 FOREIGN KEY(user_id,month) REFERENCES budget_months(user_id,month) ON DELETE CASCADE
);

CREATE TABLE income_receipts (
 user_id TEXT NOT NULL,
 expected_id TEXT NOT NULL,
 transaction_id TEXT NOT NULL,
 PRIMARY KEY(user_id,transaction_id),
 FOREIGN KEY(expected_id,user_id) REFERENCES expected_income(id,user_id) ON DELETE CASCADE,
 FOREIGN KEY(transaction_id,user_id) REFERENCES transactions(id,user_id) ON DELETE CASCADE
);

CREATE TABLE import_batches (
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

CREATE TABLE import_batch_entries (
 batch_id TEXT NOT NULL, user_id TEXT NOT NULL,
 transaction_id TEXT REFERENCES transactions(id) ON DELETE SET NULL,
 role TEXT NOT NULL CHECK(role IN ('added','matched','known')),
 revision INTEGER NOT NULL,
 FOREIGN KEY(batch_id,user_id) REFERENCES import_batches(id,user_id)
);

CREATE TABLE import_records (
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

-- Portable references are local identities, never ownership credentials.
CREATE TABLE portable_entry_refs (
 user_id TEXT NOT NULL,
 reference TEXT NOT NULL,
 transaction_id TEXT NOT NULL,
 PRIMARY KEY(user_id,reference),
 FOREIGN KEY(transaction_id,user_id) REFERENCES transactions(id,user_id) ON DELETE CASCADE
);

CREATE TABLE portable_source_refs (
 user_id TEXT NOT NULL,
 reference TEXT NOT NULL,
 source_id TEXT NOT NULL,
 PRIMARY KEY(user_id,reference),
 FOREIGN KEY(source_id,user_id) REFERENCES transaction_sources(id,user_id) ON DELETE CASCADE
);

-- Receipt links wait here until their entries are restored from CSV.
CREATE TABLE portable_income_receipts (
 user_id TEXT NOT NULL,
 entry_ref TEXT NOT NULL,
 expected_id TEXT NOT NULL,
 PRIMARY KEY(user_id,entry_ref),
 FOREIGN KEY(expected_id,user_id) REFERENCES expected_income(id,user_id) ON DELETE CASCADE
);

CREATE INDEX transaction_sources_owner ON transaction_sources(user_id);

CREATE INDEX transactions_owner_date ON transactions(user_id, date DESC, created_at DESC);

CREATE INDEX transactions_source ON transactions(source_id);

CREATE INDEX transaction_period ON transactions(user_id,date DESC);

CREATE UNIQUE INDEX transactions_identity_owner ON transactions(id,user_id);

CREATE INDEX entry_labels_owner_label ON entry_labels(user_id,label_id,transaction_id);

CREATE INDEX entry_categories_owner_category ON entry_categories(user_id,category_id,transaction_id);

CREATE INDEX label_rules_owner ON label_rules(user_id);

CREATE INDEX batch_entries_batch ON import_batch_entries(batch_id,user_id);

CREATE INDEX batch_entries_transaction ON import_batch_entries(transaction_id,user_id);

CREATE INDEX import_external_id ON import_records(user_id,source_id,external_id);

CREATE INDEX import_transaction ON import_records(transaction_id);

CREATE INDEX portable_entry_target ON portable_entry_refs(user_id,transaction_id);

CREATE INDEX portable_source_target ON portable_source_refs(user_id,source_id);

-- Later edits protect entries from removal with their original import batch.
CREATE TRIGGER track_transaction_edits AFTER UPDATE OF date,description,amount_cents,source_id ON transactions
WHEN OLD.date IS NOT NEW.date OR OLD.description IS NOT NEW.description
 OR OLD.amount_cents IS NOT NEW.amount_cents OR OLD.source_id IS NOT NEW.source_id
BEGIN
 UPDATE transactions SET revision=OLD.revision+1 WHERE id=NEW.id;
END;

CREATE TRIGGER category_added AFTER INSERT ON entry_categories BEGIN
 UPDATE transactions SET revision=revision+1 WHERE id=NEW.transaction_id AND user_id=NEW.user_id;
END;

CREATE TRIGGER category_changed AFTER UPDATE OF category_id ON entry_categories
WHEN OLD.category_id IS NOT NEW.category_id BEGIN
 UPDATE transactions SET revision=revision+1 WHERE id=NEW.transaction_id AND user_id=NEW.user_id;
END;

CREATE TRIGGER category_removed AFTER DELETE ON entry_categories BEGIN
 UPDATE transactions SET revision=revision+1 WHERE id=OLD.transaction_id AND user_id=OLD.user_id;
END;

CREATE TRIGGER label_added AFTER INSERT ON entry_labels BEGIN
 UPDATE transactions SET revision=revision+1 WHERE id=NEW.transaction_id AND user_id=NEW.user_id;
END;

CREATE TRIGGER label_removed AFTER DELETE ON entry_labels BEGIN
 UPDATE transactions SET revision=revision+1 WHERE id=OLD.transaction_id AND user_id=OLD.user_id;
END;

-- Deleting an entry permits a deliberate reimport.
CREATE TRIGGER forget_deleted_import_evidence BEFORE DELETE ON transactions BEGIN
 DELETE FROM import_records WHERE transaction_id=OLD.id AND user_id=OLD.user_id;
END;
