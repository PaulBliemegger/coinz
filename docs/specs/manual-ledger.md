# Manual ledger

Status: implemented; browser and container runtime verification pending.
Purpose: record personal EUR finances accurately while making incomplete history
visible. [Product scope](../product.md) owns the general recorded-figure semantics.

## Identity

Users are created locally with `create-user`; there is no public signup. Usernames
are normalized to lowercase ASCII letters/numbers plus dot, underscore, or hyphen,
1–80 characters. Passwords contain 12–128 characters and are Argon2id-hashed.

Sessions are stored in SQLite and use an opaque HttpOnly, SameSite=Strict cookie,
with seven-day inactivity expiry. Login rotates the session ID and CSRF token;
logout invalidates the session. Failed login uses a generic credential error.
HTTPS deployments enable the Secure cookie flag.

Private reads and writes require a valid user. Account or transaction IDs owned
by someone else produce the same not-found result as nonexistent IDs. List and
summary responses never include another user's data. Cookie-authenticated writes,
including login/logout, require the CSRF token obtained from the session endpoint.
An Origin header, if supplied, must match the configured browser origin.

## Accounts and money

An account has a name (1–80 trimmed characters), opening calendar date, and signed
opening balance. It belongs to its creator and uses EUR. The opening balance is
at the start of that date, before recorded transactions for that date. It affects
the recorded balance, never income/spending. Account editing/deletion is deferred.

Inputs are plain euro strings, e.g. `12`, `12.3`, `12.34`, or `12,34`, with no
thousands separator, exponent, or currency symbol. Whitespace around an amount is
ignored. Precision beyond two decimal places is rejected, never rounded. Negative
opening balances are allowed. Income/expense amounts are strictly positive inputs;
the selected kind determines their sign in the ledger.

SQLite stores signed 64-bit integer cents. Requests that exceed the supported
range or cause an account balance to overflow are rejected atomically. Responses
encode cents as decimal strings so browser display preserves precision.

## Entries and dates

A transaction has an owned account, calendar date, description (1–200 trimmed
characters), and nonzero signed cents. Dates use strict `YYYY-MM-DD`, years
0001–9999, and must be on/after the selected account's opening date.

Entry creation, editing (including changing the account), and deletion update
balances and period totals. Dependent balance checks and the write commit together.
Invalid requests leave the ledger unchanged. Identical legitimate purchases are
allowed; no content-based automatic deduplication exists in this first slice.
The UI disables repeated submissions while saving. Import matching/review is a
separate planned feature, not a guarantee made by this manual-entry API.

Future-dated manual entries are currently accepted and included in the all-dates
recorded account balance and their selected month's totals. They are ordinary
recorded entries, not planning objects. Dedicated planned entries come later.

## Views

- Filter by explicit `YYYY-MM` and optionally one owned account. If the API caller
  omits month, it defaults to the server's current UTC month. The UI always sends
  its selected month, initially the browser's local month.
- List transactions newest calendar date first, then creation time and ID for a
  stable tie-break. The initial list is not paginated.
- Income sums positive amounts; spending is the positive magnitude of expenses;
  net is income minus spending. Summaries include the transaction count and month.
- Account balances include all recorded dates, independently of the period filter.
  Show each account's latest entry date (or opening date if empty); this does not
  establish complete coverage. No import time exists until imports are implemented.
- Transfers and linked refunds are not modeled yet. Do not record both sides of
  an owned-account transfer as ordinary income/expense and expect correct spending
  reports. Refund-specific treatment will be specified with that feature.

## API boundary

All routes below are under `/api`. JSON uses snake_case. Input euro fields are
strings; output fields ending `_cents` are integer strings.

| Method and route | Purpose |
| --- | --- |
| `GET /health` | Public process health response |
| `GET /session` | User or null, plus CSRF token |
| `POST /login` | Username/password; returns user and rotated CSRF token |
| `POST /logout` | Invalidate session |
| `GET /accounts` | Owned accounts and all-dates recorded balances |
| `POST /accounts` | `name`, `opening_date`, `opening_balance` |
| `GET /transactions?month=…&account_id=…` | Filtered owned transactions |
| `POST /transactions` | `account_id`, `date`, `description`, `amount`, `kind` |
| `PUT /transactions/{id}` | Replace editable fields using the same request shape |
| `DELETE /transactions/{id}` | Delete an owned transaction |
| `GET /summary?month=…&account_id=…` | Income, spending, net, count, month |

`kind` is `income` or `expense`. Success statuses are 200 for reads/login, 201 for
creation, and 204 for update/logout/delete. Application errors use
`{"error":"message"}`: 401 for authentication, 403 for CSRF/origin, 404 for missing
or inaccessible records, and 422 for domain validation. Framework rejection of
malformed JSON or oversized bodies may use its own response. The frontend handles
that with a generic error. API responses normally disable caching.

## Acceptance evidence

| Example | Expected behavior | Test home |
| --- | --- | --- |
| Opening €1,000; income €2,000; expense €12.34 | Balance €2,987.66; spending €12.34; opening excluded from income | `backend/tests/ledger.rs` |
| Change expense to €13.34, then delete it | Balances and summaries reflect each operation | `backend/tests/ledger.rs` |
| 30 November versus 1 December | Only the November entry affects November totals | `backend/tests/ledger.rs` |
| User B supplies User A's account/entry IDs | Empty owned lists or not found; no mutation/leak | `backend/tests/ledger.rs` |
| Invalid precision/date, pre-opening entry, or overflow | Rejection with no partial mutation | `backend/tests/ledger.rs` |
| Concurrent entries near maximum balance | At most the valid write commits | `backend/tests/ledger.rs` |
| Session rotation, restart, logout, CSRF/origin | Identity persists or is rejected as appropriate | `backend/tests/ledger.rs` |
| Very large cents and negative sub-euro values | Browser formatting/editing preserves cents | `frontend/tests/money.test.ts` |
| Login → account → entries → edit → filter → logout | Complete desktop and phone journey | `frontend/e2e/ledger.spec.ts` |

Unit/integration checks have passed during implementation; the browser suite is
present but has not completed because its local server could not start in the
restricted sandbox. Expiry is configured via the session library; the current
integration tests cover rotation/persistence/logout, not simulated clock expiry.
See the README for the current verification limits and runnable commands.
