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
CREATE INDEX label_rules_owner ON label_rules(user_id);
CREATE TABLE label_rule_labels (
 rule_id TEXT NOT NULL,
 user_id TEXT NOT NULL,
 label_id TEXT NOT NULL,
 kind TEXT NOT NULL DEFAULT 'label' CHECK(kind='label'),
 PRIMARY KEY(rule_id,label_id),
 FOREIGN KEY(rule_id,user_id) REFERENCES label_rules(id,user_id) ON DELETE CASCADE,
 FOREIGN KEY(label_id,user_id,kind) REFERENCES classifications(id,user_id,kind)
);
