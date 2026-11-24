CREATE UNIQUE INDEX transactions_identity_owner ON transactions(id,user_id);

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
CREATE INDEX entry_labels_owner_label ON entry_labels(user_id,label_id,transaction_id);
CREATE INDEX entry_categories_owner_category ON entry_categories(user_id,category_id,transaction_id);
CREATE TABLE classification_setup (
 user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id)
);

-- Assignment changes protect imported entries even if the user later reverts them.
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
