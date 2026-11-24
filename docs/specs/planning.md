# Monthly budgets

Status: implemented. Browser/visual verification status is recorded in the README.

## Plan and recorded figures

`/budgets` provides independent calendar-month plans. A month contains a savings
reservation, optional targets for owned categories, and named expected-income
items. No plan is required to record or import entries. A missing category target
means Not set; an explicit zero means a zero spending target. There is no rollover,
automatic allocation, recurring commitment engine or account balance tracking.

The visible Recorded through date defaults to the browser's local calendar date.
Budget actuals include only owned entries within the selected month on/before that
date. Future-dated entries do not count early. This differs intentionally from
Insights, which shows all records dated within its selected periods.

The backend calculates exact integer cents:

- From recorded income = recorded income − all recorded expenses − reserved savings.
- Including pending income = recorded income + pending expected income − all recorded
  expenses − reserved savings.
- Unallocated in plan = named planned income − category targets − savings reservation.
- Category remaining = its target − its recorded expenses; Not set has no remaining
  target. Negative amounts mean the limit is exceeded, not that an entry is rejected.

Unbudgeted and Uncategorized expenses still reduce both overall spending-room
figures and are shown separately. Income is never silently netted against a
category's expenses. Labels do not multiply actuals. Savings is a planned
reservation, not a recorded expense or proof of money saved. Transfers between
one's own accounts must be excluded from ordinary income/expenses; dedicated
transfer/refund handling remains outside this slice. These monthly calculations
are not bank balances or assurances of available cash.

## Expected income and matching

Each item has a name, planned amount, optional recorded-income links, and a
Nothing more expected flag. Linked entries must be owned positive income from the
same calendar month. Each entry can fulfill only one expected item across plans.
Received totals include eligible links through Recorded through. Pending equals
max(planned − received, 0), or zero when Nothing more expected is checked. This
supports partial payments, larger receipts, cancellation and final lower payments.
All recorded income is counted once in overall figures whether matched or not;
matching only reduces the still-expected amount. It creates no transactions.

Receipt ownership/date/sign are checked again on save. Later ledger edits can
make a link ineligible; the report flags it, excludes it from received income,
and lets the user remove unavailable links. A moved receipt must be unlinked from
its former plan before matching in a different month. Deleting an entry removes
its link, without deleting the plan; pending reopens unless explicitly completed.
An explicit completion flag continues to mean nothing further is expected.
Category expense drill-down links use the same recorded-through cutoff; months
wholly beyond that cutoff have no recorded-expense link.

## Editing, copying and category lifecycle

The default view is Saved plan vs actual, with planned, actual and remaining
amounts per category. Edit monthly plan reveals the editor.
Changes stay in a local draft until Save budget. Financial cards explicitly show
the saved plan while the draft is dirty. Month/date changes and copying ask before
discarding a dirty draft. Loading/error states hide old results; expiry returns to
login. A successful save is reported separately from a failed subsequent reload.
Concurrent saves use an owned month revision checked under the write lock; stale
writers must reload. Validation failures roll back all changes. No entry amounts,
classifications or import provenance are changed by budgeting. Adding, changing
or removing receipt links advances the linked entries' edit revisions for import
removal protection; unchanged links and other plan edits do not.

Copy plan copies amounts, names and category targets into the destination draft.
It clears expected-item identities, receipt links and completion flags, skips
archived categories with an explicit notice, and requires Save to persist. Saving
one month never modifies another. Archived targets remain editable in their
existing months but cannot be added to a new month until restored. Physical
category deletion previews the affected monthly target count, removes those
specific targets and advances affected plan revisions; archival preserves them.
Deletion review fingerprints include target amounts and plan revisions.

The year overview shows category expenses from January through the selected
month, cut off at Recorded through. Budget comparison includes only months where
that category has a target; it exposes the count and actual expenses for those
same months. Unset months are not treated as zero budgets. Insights provides
explicitly selected-month averages for irregular-cost context.

## API and evidence

- `GET /api/budgets/{YYYY-MM}?as_of=YYYY-MM-DD` returns a saved plan or an empty
  revision-0 draft, category targets/actuals/year comparison, expected-income
  items and eligible receipt choices, and backend-calculated spending-room sums.
- `PUT /api/budgets/{YYYY-MM}` accepts `revision`, `savings` (EUR input string),
  `categories: [{category_id, amount}]`, and `income: [{id, name, amount, complete,
  transaction_ids}]`. New expected items use null/omitted IDs. Existing IDs must
  belong to this user and month. Amounts are exact nonnegative EUR with at most
  two decimals; reads return integer-cent strings. At most 500 category targets,
  100 income items and 500 links per item are accepted.
- Derived differences use exact i128 cents strings; persisted amounts and each
  independently summed amount use checked i64 cents. Ownership and category kind
  also have composite foreign keys. Cookie writes require CSRF/Origin validation.

Integration cases in `backend/tests/ledger.rs` cover unset/zero, savings, partial
and completed receipts, unbudgeted spending, cutoff dates, year comparisons,
foreign targets/receipts, duplicate matching, rollback, stale saves and deletion
reviews, archive behavior and edited receipt lifecycle. The desktop/phone journey
in `frontend/e2e/budgets.spec.ts` defines editing, matching, savings, copy/reset and
screenshots. The README records which checks could actually run.

Settings restoration can temporarily leave receipt links waiting for their CSV
entries. The [portability contract](data-portability.md#settings-json) defines the
warning, edit guard and reconnection behavior for that state.

## Deferred

Automatic rollover, envelope funding, annual saving-up targets, label budgets,
recurring expense commitments, scenario forecasts and balance calculators remain
future work. Budgets use manually entered assumptions; historical averages do not
automatically become targets.
