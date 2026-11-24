-- Portable references are identities within an owner's namespace, never ownership IDs.
CREATE TABLE portable_entry_refs (
 user_id TEXT NOT NULL,
 reference TEXT NOT NULL,
 transaction_id TEXT NOT NULL,
 PRIMARY KEY(user_id,reference),
 FOREIGN KEY(transaction_id,user_id) REFERENCES transactions(id,user_id) ON DELETE CASCADE
);
CREATE INDEX portable_entry_target ON portable_entry_refs(user_id,transaction_id);
CREATE TABLE portable_source_refs (
 user_id TEXT NOT NULL,
 reference TEXT NOT NULL,
 source_id TEXT NOT NULL,
 PRIMARY KEY(user_id,reference),
 FOREIGN KEY(source_id,user_id) REFERENCES transaction_sources(id,user_id) ON DELETE CASCADE
);
CREATE INDEX portable_source_target ON portable_source_refs(user_id,source_id);
