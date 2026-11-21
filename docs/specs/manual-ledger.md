# Manual ledger

Status: implemented; browser verification remains pending.
Manual tracking is independent of accounts and balances. Sources are optional metadata.
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

Private reads and writes require a valid user. Source or transaction IDs owned
by someone else produce the same not-found result as nonexistent IDs. List and
summary responses never include another user's data. Cookie-authenticated writes,
including login/logout, require the CSRF token obtained from the session endpoint.
An Origin header, if supplied, must match the configured browser origin.

## Optional sources and exact amounts

A source has an owned ID, a trimmed name of 1–80 characters, and a latest entry date
when known. It identifies provenance such as N26, Erste, cash, or a distinct export
stream. Creating a source requires no balance. Source names are not unique and are
not identity keys for deduplication. Source editing/deletion is not implemented.

A transaction may omit `source_id` or explicitly set it to null. Both mean no source;
the UI calls this Manual. The default manual form uses no source, except when opened
from an active source filter. Source metadata may be attached, replaced, or cleared
while editing. All provided IDs must belong to the authenticated user. Source/entry writes reject
unknown fields, including the retired `account_id`, rather than silently discarding
provenance from an outdated client.

Euro inputs are strings such as `12`, `12.3`, `12.34`, or `12,34`, with no grouping,
exponent, or currency symbol. Ignore surrounding whitespace; reject precision
beyond two decimal places instead of rounding. Transaction inputs are strictly
positive, with `income` or `expense` determining the stored sign. Stored amounts
are i64 cents. Return cents as decimal strings and display them exactly with BigInt.
Derived period totals outside the supported i64 range produce a validation error.

## Entries and reporting

An entry has an optional owned source, strict calendar date (`YYYY-MM-DD`, years 0001–9999),
trimmed description (1–200 characters), and nonzero signed amount. Creation, editing
(including changing or clearing source), and deletion update the ledger atomically. No date
constraint depends on source creation. Future-dated entries are currently ordinary
recorded transactions in their respective month; planned entries are deferred.

Identical legitimate purchases are allowed. The UI prevents repeated submission
while saving; content-based matching/deduplication is a separate import feature.

The main view filters by `YYYY-MM` and optionally one owned source. The API defaults
an omitted month to the server's UTC month; the UI explicitly sends its selected
month, initially the browser's local month. List newest dates first, then creation
time and ID. The initial list is not paginated. Income sums positive amounts,
spending is the positive magnitude of expenses, and net is income minus spending.
No balance observation affects these totals. See the product's recorded-data semantics:
a selected period does not imply complete bank history.

Dedicated transfer/refund types remain deferred. Ordinary income/expense entries
keep their entered meaning; the app does not infer a transfer or linked refund.

## API

All routes use `/api` and snake_case JSON. Input euro amounts are strings; output
`*_cents` fields are decimal integer strings. CSRF-protected writes use the token
from `/session`.

| Method and route | Purpose |
| --- | --- |
| `GET /health` | Public process health |
| `GET /session` | User or null, plus CSRF token |
| `POST /login` | Username/password; returns user and rotated CSRF token |
| `POST /logout` | Invalidate session |
| `GET /sources` | Owned source names/IDs and latest entry date; no balance |
| `POST /sources` | Create provenance metadata using `name` |
| `GET /transactions?month=…&source_id=…` | Filtered owned entries |
| `POST /transactions` | `source_id`, `date`, `description`, `amount`, `kind` |
| `PUT /transactions/{id}` | Replace editable fields with the same request shape |
| `DELETE /transactions/{id}` | Delete an owned entry |
| `GET /summary?month=…&source_id=…` | Income, spending, net, count, month |

Success is 200 for reads/login, 201 for source/entry creation, and 204 for
update/logout/delete. Domain errors use `{"error":"message"}`: 401 authentication,
403 CSRF/origin, 404 missing/inaccessible records, and 422 validation. Framework
JSON/body-size rejections can have another shape, which the UI handles generically.

## Preserved legacy data

Migration `0003_optional_transaction_sources.sql` renames accounts to transaction
sources and preserves IDs, owners, names, entry IDs, amounts, dates, descriptions,
and creation times. It makes the source reference nullable. Prior migrations remain
unchanged. Opening figures and snapshots remain stored as retired records, with no
active API/UI or effect on reports. They are not repurposed into planning inputs.
No personal database is inspected/migrated during development; migration tests
use fresh synthetic databases. Deployment applies the migration on startup.

## Acceptance evidence

- Income €2,000 and expense €12.34 in November yield November spending €12.34
  and net €1,987.66, without any opening figure. Older history is accepted.
- Changing/deleting the expense updates totals; December entries stay out of
  November. Source responses contain no inferred balance/opening fields.
- Another user cannot access/mutate entries or sources through guessed IDs.
- Invalid precision/date/sign requests leave records unchanged.
- Manual entries appear in all-source totals despite having no source. Selecting
  a source includes only its entries; attaching/clearing provenance changes only
  that filter's membership, not the all-source totals.
- Former Accounts/snapshot routes return 404; no balance CRUD is exposed.

`backend/tests/ledger.rs` covers these API and persistence cases along with session
rotation, persistence, logout, and CSRF/origin. The expiry policy uses the session
library and has not been tested with a simulated clock. `frontend/tests/money.test.ts`
covers exact display/edit values; `frontend/e2e/ledger.spec.ts` defines the desktop
and phone journeys. Browser verification remains limited by sandbox permissions;
the owner previously confirmed login and manual entry locally before this refinement.
