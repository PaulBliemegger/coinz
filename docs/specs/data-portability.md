# Entry export and reimport

Status: CSV export and restoration implemented in source. Settings JSON remains
proposed. See the [README](../../README.md#verification-status) for current checks.

## Workflow and scope

**User menu → Settings → Your data → Export all entries** downloads
`coinz-entries.csv` containing every
entry owned by the signed-in user, across all dates, filters and pages, including
future-dated entries. It is a snapshot of recorded entries, not proof of complete
bank history. Empty ledgers produce the header only.

Use **Import CSV**, choose the downloaded file, then **Review entries**. The
complete coinz header is recognized automatically; no bank profile, mapping or
single import source is needed. Review shows each row's saved category, labels
and optional source. Select additions and resolve duplicate candidates using the
existing import decisions. Confirmation is atomic and creates a new import batch.
Cancelling preview writes nothing. Sources and classification names are created
only for committed additions, inside the same transaction.

Exports/restores support up to 50,000 entries and 16 MiB of UTF-8 CSV. Export fails
explicitly above either limit and never returns a partial snapshot. Ordinary
bank CSV limits remain 1,000 rows/256 KiB. The restore endpoints allow a larger
JSON envelope for escaping; parsing still enforces the CSV byte/row bounds.

## CSV contract

UTF-8, comma separated, standard CSV quoting, with this exact header order:

```csv
date,amount,description,category,labels,source,entry_ref,source_ref,origin
```

- `date`: ISO calendar date. `amount`: signed EUR decimal with a dot and two
  fractional digits on export. Import uses exact integer cents and rejects zero,
  grouping, excess precision and overflow. No float conversion or rounding.
- `description`: the saved text, including quoted commas, quotes and line breaks.
- `category`: saved name, or empty for Uncategorized. `labels`: a JSON array of
  saved label names inside the CSV field, preserving separators and punctuation.
  Duplicate label names are invalid. Labels are sorted for deterministic output.
- `source`: saved name, or empty. `source_ref`: UUID reference, or empty with the
  source. Distinct references preserve distinct sources even with identical names.
  A source-less entry remains source-less. Existing source-reference/name
  conflicts reject the commit; names alone never merge sources.
- `entry_ref`: UUID reference preserving each entry's identity, including valid
  identical purchases. `origin`: `manual`, `csv` or `unknown`, preserved on restore.

References resolve only within the authenticated user's namespace. They do not
authorize access or copy internal user IDs. A fresh installation generates its
own entry/source IDs and remembers portable references. Subsequent exports keep
those references. Native references are never treated as bank-issued IDs.

Missing category/label names are created as active items using the default sage
colour. Existing names resolve case-insensitively within the same kind and keep
their current spelling, colour and archived state. Saved assignments to archived
items can be restored without unarchiving those items. CSV restores assignments;
unused vocabulary and its colour/archive settings require the later settings
export. Import rules do not reinterpret the saved classification. Edit restored
entries normally afterward; the native review does not offer classification or
description overrides. Matching an existing entry retains all its current fields.

There is no CSV format version or manifest initially. Explicit conversion scripts
are acceptable for a future breaking change.

## Duplicates and conflicts

The shared import preview, selection, match/keep choices and serialized commit
remain in use. Portable references add native identity ahead of ordinary exact
candidate matching:

- Restoring an unchanged snapshot again adds no copies, including after the file
  is reordered or exported from the restored installation.
- Two identical purchases with different references survive an empty-install
  restore. Repeated references within one file conflict rather than being silently
  discarded.
- A known reference whose current fields/classification/source differ conflicts;
  restoration never overwrites or deletes local edits. Leave it out, reconcile
  the file deliberately, or use a fresh destination.
- An unknown reference with an exact existing candidate requires ordinary review.
  Keep it as a separate purchase or match an owned candidate. Matching attaches
  the reference without changing the existing entry's fields.
- Invalid selected rows, unavailable matches, ownership violations and failures
  roll back entries, new source/classification names and import evidence together.
  Unselected rows retain the existing import semantics.

`GET /api/exports/entries.csv` requires a session and returns a no-store CSV
attachment. `POST /api/imports/restore/preview` and `/commit` use the existing import
request/response types and require session authentication and CSRF protection.
The parser also recognizes this format through the ordinary preview/commit routes
when it fits their smaller request envelope. No bank profile or single source may
be supplied for a coinz snapshot. Preview creates no persistent data.

## Entry copies, setup and full recovery

CSV does not preserve original bank import evidence, duplicate decisions, batch
history, profiles, rules, budgets, unused classifications, user accounts, password
hashes or sessions. It is a portable entry copy, not a full application backup.
Later overlapping bank files may require review again, especially for edited
bank descriptions. Credentials must be created independently on a fresh install.

Versioned settings JSON export/import is the next proposed slice: preserve
categories/labels, sources, CSV profiles, labeling rules and saved budgets,
including their relationships. Restore settings before entries. CLI portability
and full database backup tooling remain deferred. The existing private SQLite
recovery procedure is in the [README](../../README.md#later-homelab-deployment).

After a verified round trip, deliberate development schema resets are acceptable:
export, explicitly create an empty database, restore settings/entries, then check
counts and totals. Startup must never silently reset an incompatible database.
All private exports and runtime data stay outside the repository and agent context;
use wholly synthetic fixtures for development, following [privacy](../privacy.md).

## Acceptance evidence

Synthetic tests in `backend/tests/ledger.rs` cover a 1,005-entry round trip into
an independent temporary installation, two years, exact large money values,
quoted/multiline descriptions and label punctuation, identical purchases,
source-less entries, equally named distinct sources, origin/classification
assignments, repeat and overlapping restores, local-edit conflicts, authentication,
CSRF, two-user isolation, explicit matching and atomic failure. Parser tests in
`backend/src/imports/portable.rs` validate money, names, references and JSON labels.
`frontend/e2e/portability.spec.ts` defines desktop/phone download, preview, restore
and repeat-preview journeys. The README distinguishes executed checks from pending
browser verification.
