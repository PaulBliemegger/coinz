-- Existing account identities become lightweight import/transaction sources.
-- Snapshot and legacy-opening tables remain stored but have no active API.
ALTER TABLE accounts RENAME TO transaction_sources;
DROP INDEX accounts_owner;
CREATE INDEX transaction_sources_owner ON transaction_sources(user_id);

CREATE TABLE transactions_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id),
    source_id TEXT,
    date TEXT NOT NULL,
    description TEXT NOT NULL CHECK (length(description) BETWEEN 1 AND 200),
    amount_cents INTEGER NOT NULL CHECK (amount_cents != 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (source_id, user_id) REFERENCES transaction_sources(id, user_id)
);
INSERT INTO transactions_new (id, user_id, source_id, date, description, amount_cents, created_at)
SELECT id, user_id, account_id, date, description, amount_cents, created_at FROM transactions;
DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;
CREATE INDEX transactions_owner_date ON transactions(user_id, date DESC, created_at DESC);
CREATE INDEX transactions_source ON transactions(source_id);
