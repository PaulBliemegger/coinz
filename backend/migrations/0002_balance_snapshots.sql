-- Preserve opening figures without reinterpreting a start-of-day value as an
-- end-of-day observation. They no longer constrain or affect the ledger.
CREATE TABLE legacy_opening_balances (
    account_id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    opening_date TEXT NOT NULL,
    opening_cents INTEGER NOT NULL,
    FOREIGN KEY (account_id, user_id) REFERENCES accounts(id, user_id)
);
INSERT INTO legacy_opening_balances
SELECT id, user_id, opening_date, opening_cents FROM accounts;
ALTER TABLE accounts DROP COLUMN opening_date;
ALTER TABLE accounts DROP COLUMN opening_cents;

CREATE TABLE balance_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    as_of_date TEXT NOT NULL,
    balance_cents INTEGER NOT NULL,
    FOREIGN KEY (account_id, user_id) REFERENCES accounts(id, user_id),
    UNIQUE (account_id, as_of_date)
);
CREATE INDEX snapshots_owner_account_date
ON balance_snapshots(user_id, account_id, as_of_date);
