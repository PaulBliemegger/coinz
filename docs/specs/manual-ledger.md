# Manual ledger

Status: implemented; current verification limits are in the
[README](../../README.md#verification-status).
Manual tracking is independent of accounts and balances. Sources are optional metadata.
Purpose: record personal EUR finances accurately while making incomplete history
visible. [Product scope](../product.md) owns the general recorded-figure semantics.

## Identity

Users are created locally with `create-user`; there is no public signup. Usernames
are normalized to lowercase ASCII letters/numbers plus dot, underscore, or hyphen,
1–80 characters. Passwords contain 12–128 characters and are Argon2id-hashed.

Sessions are stored in SQLite and use an opaque HttpOnly, SameSite=Strict cookie,
with seven-day inactivity expiry. Login rotates the session ID and CSRF token;
logout invalidates the session. Unauthorized feature reads return the interface to
login, including after logout in another tab; logging in refreshes the CSRF token. Failed login uses a generic credential error.
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
the UI calls the source No source. Entry method is shown separately. The default manual form uses no source. Source metadata may be attached, replaced, or cleared
while editing. All provided IDs must belong to the authenticated user. Source/entry writes reject
unknown fields, including the retired `account_id`, rather than silently discarding
provenance from an outdated client.

Euro inputs are strings such as `12`, `12.3`, `12.34`, or `12,34`, with no grouping,
exponent, or currency symbol. Ignore surrounding whitespace; reject precision
beyond two decimal places instead of rounding. Transaction inputs are strictly
positive, with `income` or `expense` determining the stored sign. Stored amounts
are i64 cents. Return cents as decimal strings and display them exactly with BigInt.
Derived period totals outside the supported i64 range produce a validation error.

## Categories and labels

The [classification spec](classification.md) owns the single category bucket,
optional labels, management, setup, manual form, bulk assignment and filters.
Classification is independent of sources and income/expense. The optional source
controls in the manual form are secondary and collapsed initially.

## Entries and reporting

An entry has an optional owned source, strict calendar date (`YYYY-MM-DD`, years 0001–9999),
trimmed description (1–200 characters), and nonzero signed amount. Creation, editing
(including changing or clearing source), and deletion update the ledger atomically. No date
constraint depends on source creation. Future-dated entries are currently ordinary
recorded transactions in their respective month; planned entries are deferred.

Identical legitimate purchases are allowed. The UI prevents repeated submission
while saving; content-based matching/deduplication is a separate import feature.

The [Dashboard and Entries spec](reporting.md) owns screen navigation, period
filters, pagination, and reported totals. Income sums positive amounts, spending
is the positive magnitude of expenses, and net is income minus spending.
No balance observation affects these totals. The legacy `/transactions` and
`/summary` API reads retain monthly/source filtering, defaulting to the server's
UTC month when omitted. New screens send explicit date/month bounds.

Dedicated transfer/refund types remain deferred. Ordinary income/expense entries
keep their entered meaning; the app does not infer a transfer or linked refund.

## Direct editing, selection, and provenance

Compact rows keep description and date/source metadata close together, without
extra disclosure panels. On fine pointers field controls have compact heights;
coarse pointers retain 44px touch targets. Descriptions, amounts (with income/expense), dates, and sources are activated directly
in the list. There is no Edit button or global edit mode. The inline form has
Save/Cancel and Escape; Enter submits native single-input forms. Blur does not save. After saving, focus returns to the edited field once fresh
results render, or to the workspace heading if the entry left the filtered results.
Field-specific PATCH requests avoid overwriting unrelated stale fields. Concurrent
changes to the same field use last-write-wins; optimistic conflict detection is deferred.

Entry selection uses a round native checkbox in the left gutter. Selected entries
highlight the whole row and show a minus inside the round control, indicating
that it can be deselected. Clicking the row background or noninteractive metadata
toggles selection; description/date/source/amount buttons and any inline-form
interaction retain their existing editing behavior. Selecting text does not select
a row. Disabled/saving rows ignore background selection. Native checkbox focus
and Space provide the keyboard equivalent; row containers do not add tab stops.
Selection does not reset an unsaved inline draft. Entries use separated row
surfaces without horizontal separators. The dashboard's unshaded latest-entry
preview retains its horizontal lines.

The background alone flips in over 280 ms on selection and fades out over 160 ms
on deselection. Text, controls, row height, and spacing remain stationary. Quick
changes cancel old animations from their current visual state. Initial rendering
does not animate. Reduced-motion preference disables effects immediately,
including when enabled mid-animation. Select-all and Clear selection use the same
row feedback. The selection scope and deletion semantics below are unchanged.

On fine pointers row selection controls reveal on hover or keyboard focus. Reserve
their space to avoid shifting content. Once any row is selected, show all controls.
Touch devices keep controls visible. The separate Settings sidebar contains Always show selection
controls, defaults off and stores only a boolean in localStorage; storage failure
keeps the default/session view usable. It does not enable editing or contain finance
or authentication data.

Selection is scoped to displayed entries and cleared when filters change or the list
reloads. Select up to 100 at a time; the select-all label states first 100 when needed.
The selection line places count, clear selection, and an accessible bin icon
on the right of the Select displayed entries control, without a separate card.
Controls wrap on narrow screens.
The bin reveals adjacent Delete N entries and Cancel, with period and reimport
consequences. The bin does not move into the confirmation position. Escape or a
second bin click cancels; selection/filter changes reset confirmation. No centered
modal and no automatic focus on destructive confirmation. Confirmation buttons form a tight group immediately to the left of the bin,
without a separate card or full-width explanation. The whole action group may
wrap on phones; do not separate the confirmation from the bin.

Bulk deletion validates all IDs and ownership under one write lock and deletes
atomically. A missing/foreign ID rejects the entire set; reject empty, duplicate,
or more-than-100 ID lists. Delete associated import evidence so these entries
can be imported again. No undo is promised. CSRF applies to bulk deletion and
inline writes.

`origin` is `manual`, `csv`, or `unknown`, independent of the editable source.
New manual/API entries have manual origin; new import additions have csv origin.
Matching adds evidence without changing origin. Unknown origin remains supported
for portable entries; source metadata never determines origin.
List responses include `csv_linked`; non-CSV-origin entries with evidence say CSV
matched. The entry list no longer renders an import-details foldout. The owned provenance
API remains available (up to 20 rows) for future interfaces and regression tests,
showing original source, record number, date, description, and amount. Filenames
are stored only for new [import-history batches](import-history.md), never inferred
for older evidence.

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
| `PATCH /transactions/{id}` | Change `field` (`description`, `date`, `source`, `amount`) using `value`; amount also requires `kind` |
| `POST /transactions/bulk-delete` | Atomically delete 1–100 distinct owned `ids` |
| `GET /transactions/{id}/provenance` | Up to 20 owned original CSV evidence rows |
| `DELETE /transactions/{id}` | Delete an owned entry |
| `GET /summary?month=…&source_id=…` | Income, spending, net, count, month |

Success is 200 for reads/login, 201 for source/entry creation, and 204 for
entry updates, logout, and deletions (including bulk deletion). Domain errors use
`{"error":"message"}`: 401 authentication, 403 CSRF/origin, 404 missing/inaccessible
records, and 422 validation. Origin errors name the configured app address;
token errors ask for a refresh. Framework JSON/body-size rejections can have another
shape, which the UI handles generically.

## Schema baseline

`0001_initial.sql` creates the current ledger directly, with nullable transaction
sources and no account, opening-balance or snapshot tables. The pre-v0.1 migration
history has been replaced; databases from that history require an explicit reset.
The [README reset procedure](../../README.md#development-schema-baseline-and-reset)
owns the commands. Startup must reject incompatible histories rather than reset
or rewrite existing data. All agent verification uses temporary synthetic databases.

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
- Field-specific updates preserve the other entry fields and origin; CSV matches
  retain the existing entry and expose its original import evidence.
- A bulk deletion containing a missing or foreign ID rolls back the entire set.
  Successful deletion releases linked imports for reimport.

`backend/tests/ledger.rs` covers these API and persistence cases along with session
rotation, persistence, logout, and CSRF/origin. The expiry policy uses the session
library and has not been tested with a simulated clock. `frontend/tests/money.test.ts`
covers exact display/edit values; `frontend/tests/appearance-period.test.ts` covers
appearance preference validation and bounded month navigation.
`frontend/tests/selection-highlight.test.ts` covers flip/fade separation, interrupted
motion, stale completion callbacks, reduced motion, and action cleanup. The desktop/phone
journeys in `frontend/e2e/ledger.spec.ts` cover inline editing, selection preferences,
deletion confirmation, round row selection, edit/selection independence, keyboard
selection, reduced motion, appearance, dashboard navigation, and period filters
alongside manual entry.
These browser journeys define coverage, not a claim that their assertions or
screenshots have been verified; see the README verification status.

## Appearance and user menu

Clicking the username opens a native disclosure containing Appearance, Settings,
and Log out. Escape and outside pointer interaction close it. Settings still opens
the separate sidebar for selection visibility; no export placeholder is added.

Appearance offers System (default), Light, and Dark. A validated localStorage
preference (`coinz.appearance`) contains only that enum; missing/invalid/unavailable
storage means System. Apply the saved choice before initial paint. CSS `color-scheme`
and `light-dark()` tokens follow OS changes immediately in System mode, including
native inputs and dialogs. Overrides remain independent of OS changes and persist
across reloads when storage is available. No financial or authentication data is
stored with the preference. Backend figures and APIs are unaffected.
