-- Hold receipt identities until their entries are restored from CSV.
-- Saving/deleting the expected item deliberately replaces these pending links.
CREATE TABLE portable_income_receipts (
 user_id TEXT NOT NULL,
 entry_ref TEXT NOT NULL,
 expected_id TEXT NOT NULL,
 PRIMARY KEY(user_id,entry_ref),
 FOREIGN KEY(expected_id,user_id) REFERENCES expected_income(id,user_id) ON DELETE CASCADE
);
