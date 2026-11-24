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
