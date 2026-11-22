# Example CSV import

Status: implemented; current verification limits are in the
[README](../../README.md#verification-status). The checked-in examples are wholly synthetic,
not N26 or Erste format samples. This extends the [manual ledger](manual-ledger.md)
with a small working importer, not a configurable import framework.

## First workflow

1. Open Import CSV from Home, Entries, or Imports and choose or create a source.
2. Select a CSV in the example format below.
3. Preview dated income/expenses, validation errors, and duplicate candidates.
4. Select additions and explicitly match or keep any duplicates you want to include.
   Leave other rows for later, then import the selected subset.
5. Commit atomically and return to the transactions. Cancelling changes no entries.

Require a source for imports: stable IDs describe one export stream, not a global
bank identity. Manual entries still need no source. The selected source and every
matched transaction must belong to the authenticated user. Preview and commit
require authentication and CSRF protection; the server owns parsing, decisions,
and totals. Never trust submitted ownership IDs or preview totals.

## Example format

UTF-8 comma-separated CSV with a header and ordinary CSV quoting:

```csv
date,description,amount,external_id
2026-11-07,Example coffee,-3.50,nov-004
```

- `date`: a valid `YYYY-MM-DD` calendar date.
- `description`: nonempty text within the manual-entry limit.
- `amount`: signed EUR decimal, dot separator, at most two fractional digits;
  negative means expense and positive means income. Reject zero, grouping,
  currency symbols, excess precision, and values outside supported integer cents.
  Parse exactly, without floating point or rounding.
- `external_id`: optional value; the header is present but a cell may be empty.
  IDs are scoped to the authenticated user and selected source.

Support quoted commas and CRLF/LF records through an established CSV parser.
Malformed structure or invalid required headers blocks preview; invalid data rows
show their record number in details and an actionable error. Invalid or unresolved
selected rows block commit; unselected rows can be left for later. Limits are 256 KiB of UTF-8 CSV and 1,000 data records,
shown in the upload flow. Import JSON requests are capped at 2 MiB to allow
escaping; other API routes retain their 16 KiB limit. Headers must appear in the
listed order. Dates, descriptions, amounts, and IDs are trimmed; external IDs
are limited to 200 characters. Record numbers begin at 2 after the header and
count CSV records, including quoted multiline fields.

The examples intentionally contain ordinary income and expenses only. Transfers
and refunds require explicit behavior before a bank adapter is released; do not
infer them from descriptions or claim arbitrary bank exports are supported.
Labels are added to imported entries through a subsequent organization slice,
not embedded into this initial CSV contract.

## Duplicate review and provenance

- Reimporting the same file into the same source adds no entries. Preserve a file
  fingerprint and record provenance, scoped to the user/source. Different sources
  are separate streams; identical files there must not silently match each other.
- An already imported external ID with unchanged normalized bank fields is known.
  A repeated ID with changed fields is a conflict requiring review, not an update
  to a user's entry. Duplicate external IDs within one file also need resolution.
  The UI leaves conflicts out; correct the file to include them. The legacy explicit
  API `skip` action remains available and records permanent file/row exclusion.
- Identical date/description/amount with different stable IDs can be legitimate
  purchases. Preserve both. Do not collapse the two coffees in the demo. A row with
  a stable ID can still suggest an exact manual or ID-less match, provided the
  candidate has no accepted nonempty bank ID for the selected source.
- Without a stable ID, use file/record identity only for an exact repeat. Exact
  normalized date/description/amount matches against existing entries (including
  original import evidence) are review candidates,
  not proof of duplication. Let the user keep a separate purchase or explicitly
  match an existing entry; unresolved selected candidates block commit. Offer up to 20
  candidate transactions in the same source or without a source, plus entries
  whose provenance already belongs to the selected source. Identical ID-less
  rows within one file still require keep/match review or exclusion for later.
  If identical valid rows mix blank and stable IDs, the blank-ID rows require
  review regardless of their position in the file; rows with distinct stable IDs
  remain eligible as separate purchases. Match choices refer to existing stored
  entries; rows in the current preview cannot be matched directly to each other.
  Keep a separate purchase or leave it for later; after importing one entry, a
  later preview can offer it as a match. Fuzzy matching is deferred.
- Matching associates import provenance with the existing transaction and retains
  its user-edited fields. Never overwrite future labels/notes. Deduplication uses
  original import evidence, not a user's subsequently edited description/amount.
  Matching retains the entry's current source too. Deleting an imported entry
  removes its associated evidence. Deleted entries become eligible for reimport,
  including the same file and bank IDs. Migration 0006 clears legacy accepted
  deletion markers as well. Live entries still retain duplicate protection;
  identical ID-less purchases still require review when ambiguous.
- Persist enough normalized original row data, file identity, and record identity
  to explain a match. Do not log uploads or transaction contents. Avoid persisting
  the uploaded file itself when row provenance suffices.
- Revalidate ownership and duplicates when committing; repeat submissions or a
  concurrent import must not create another copy. Resolve all additions and
  provenance associations in one database transaction, or write none.

## HTTP contract

`POST /api/imports/preview` and `POST /api/imports/commit` accept JSON with
`source_id`, `csv` (the decoded file text), optional `filename` (display basename),
optional `decisions` (default `[]`),
and optional `selected_records` (CSV record numbers). The UI always sends explicit
selection; an omitted selection retains legacy whole-file import behavior.
Neither endpoint stores the uploaded file itself. Commit stores a batch with
counts, display filename and entry associations; see [import history](import-history.md). Preview writes no entries/provenance;
creating a source is a separate explicit operation and survives cancellation.

Preview returns `{ rows }`, each with `record`, `date`, `description`,
`amount_cents` (exact cents string, empty if invalid), `external_id`, `error`,
`status` (`new`, `known`, `review`, `conflict`, `invalid`), and `candidates`.
Candidates expose the owned transaction's ID, date, description, and cents string.

Commit reparses the CSV and rechecks duplicates under the SQLite write lock.
Each decision uses its CSV record number and an action:

```json
{"record": 2, "action": "match", "transaction_id": "owned-candidate-id"}
```

The other actions are `keep` and the legacy `skip`, without a transaction ID.
For explicit selection, only selected rows may carry decisions. Duplicate or unknown
selection/decision numbers are rejected. Selected new rows are added; selected review
rows need an explicit keep/match decision; known rows add nothing. Match IDs must
still be offered candidates. Invalid selected rows fail the entire mutation.

Unselected rows add no entry or skip evidence and remain available in a later
import. Known rows are counted across the file and may retain additional provenance.
An empty selection is a valid no-op for new rows. If selection is omitted, existing
whole-file validation and automatic inclusion of new rows apply.

Return `{ added, matched, skipped, known, excluded, income_cents, spending_cents,
latest_month }`. `excluded` counts non-known rows left out by explicit selection;
selected-addition totals are calculated by the server with checked integer cents.
The month is the latest newly added entry's month or null. The UI refreshes the
current view without changing its period or source scope. Refresh failure
after success must not suggest resubmitting the mutation.

## Review interface

Show attention-needed entries first, ready entries below, and known rows in a
collapsed excluded group, independent of CSV ordering. New valid rows start
selected; review/conflict/invalid rows do not. Select-all affects only eligible
new/explicitly-kept rows. Matches are disclosed separately from additions.

Selectable rows use the same round checkbox, whole-row highlight, and flip-in /
fade-out feedback as Entries. Clicking a row's noninteractive content toggles
inclusion. Native checkbox/Space remains available; duplicate decisions and
Import details disclosures never toggle selection. Busy, known, invalid,
conflicting, or unresolved rows cannot be selected through row clicks. These
visual controls do not change duplicate decisions or import eligibility.

The dialog body scrolls while the bottom selection totals, counts, and import
action remain visible. Show how many rows will be left for later. Keep descriptions
and amounts prominent, dates secondary, and record/bank IDs in expandable details.
No selected additions/matches means the import button is disabled; an all-known
file can simply be closed. Choosing another file discards local selection.

## Acceptance examples and evidence

Fixtures: [demo.csv](../../examples/imports/demo.csv) and
[demo-overlap.csv](../../examples/imports/demo-overlap.csv).

| Scenario | Expected result |
| --- | --- |
| First demo import into a fresh source | 16 entries, with both coffees on each date retained |
| October 2026 totals | Income €2,400.00; spending €960.24; net €1,439.76 |
| November 2026 totals | Income €2,400.00; spending €1,063.79; net €1,336.21 |
| Exact repeat into the same source | No additions |
| Overlap file after demo | Three known rows and one new €16 cinema expense |
| November after overlap | Spending €1,079.79; net €1,320.21 |
| Same bank ID with changed amount | Conflict; no silent overwrite |
| Blank ID and a similar manual entry | Review; explicit match or keep separate |
| Identical rows with blank and stable IDs, in either order | Blank-ID row needs review; distinct stable-ID rows remain eligible; unresolved selected rows reject the entire commit |
| Invalid date/precision selected | Reject all selected writes; unselected invalid rows may remain for later |
| Malformed CSV | Actionable error; no commit |
| Import just one of two valid rows | Other row remains available on repeat; no skip provenance |
| Cancel, cross-user source/match, or failed commit | No entries or provenance written |

Parser regression coverage is in `backend/src/imports/parse.rs`; temporary
SQLite/API scenarios are in `backend/tests/ledger.rs`, including edits/deletion,
manual matching, conflicts, mixed blank/stable IDs in either order, selected-row
exclusions, atomic rollback, CSRF, two-user isolation, request limits, and concurrent
submissions. `frontend/tests/import-selection.test.ts` covers exact selected totals,
explicit match decisions, and leaving rows for later without permanent skips.
`frontend/e2e/ledger.spec.ts` defines desktop/phone preview, selected-subset,
repeat-preview and reimport journeys with batch removal, plus synthetic screenshots.
The README records whether those browser assertions/screenshots have been
verified. No private exports or running private app are needed.

## Deferred

Reusable column mapping and parsing settings are proposed in
[CSV import profiles](import-profiles.md); they are not implemented here.
Bank-specific adapters, Excel, transfer/refund treatment, and label/report UI
remain separate work. YAML profile sharing is deferred in the profile proposal.
