# Entry export and reimport

Status: desired future capability; deferred, not implemented. This records scope
for later definition, not an existing download/restore promise. Current operational
backup instructions remain in the [README](../../README.md#later-homelab-deployment).

## Goal

Export entries as a clean, documented CSV for personal copies, spreadsheets, and
moving recorded entries into another installation. Provide a corresponding import
flow. Prefer a small native format with predictable behavior over requiring a user
to construct a bank-import profile to read coinz's own exports.

## Proposed first scope

- Export all entries owned by the authenticated user by default, across every
  month/page. A filtered export may be offered separately, with its scope and
  count stated explicitly. Never silently export only the visible page.
- Use UTF-8 CSV with a header, standard quoting, ISO calendar dates, and exact signed
  EUR decimal amounts with a dot and two fractional digits. No float conversion,
  localization-dependent parsing, or rounding.
- Preserve date, description, signed amount, optional counterparty if implemented,
  and optional source association. Preserve distinct sources even when their
  display names are identical. Entries without a source stay without a source.
- Include stable opaque entry/source references and a format/dataset identity,
  either in the CSV or an accompanying manifest; settle the exact schema before
  implementation. These references support repeat imports and source mapping;
  they never authorize access or require copying database/user IDs between installs.
- Preview and validate native imports before an atomic selected-row commit.
  Preserve legitimate identical entries. Reimporting an unchanged native export
  into the same destination must not add copies. Conflicting edits require review;
  importing a file must not silently replace or delete existing records.
- Import into the authenticated user's data only. Resolve source references within
  that user's namespace. Do not reinterpret native IDs as bank-issued IDs or create
  one artificial bank source for every restored entry.
- Make export/reimport usable without a browser eventually, through a deliberately
  scoped CLI or equivalent supported interface. Commands and authentication are
  not designed yet. Data files remain private and outside the application/config
  Git repository, following the [privacy boundary](../privacy.md).

An export is a snapshot of recorded entries, including future-dated entries that
are already in the ledger. It does not imply complete bank coverage or include
unimplemented planning assumptions.

## Entry copies and full backups

A clean entries CSV is a useful portable copy of transaction data. By itself it
does not preserve all import evidence, duplicate decisions, batch history, profile
definitions, user identities, password hashes, or sessions. Do not label it a full
application backup or promise complete duplicate/history restoration from it.
Future labels, notes, and other entry metadata need an explicit round-trip format
as those features are implemented.

Full recovery currently uses the private SQLite/data-directory procedure in the
README. A future supported backup command could create a consistent database
snapshot with SQLite's [Online Backup API or VACUUM INTO](https://www.sqlite.org/backup.html);
do not replace the existing procedure with copying a running main database file.
Backup scheduling/storage are deployment concerns. Full backup automation and a
richer portable archive are deferred, separate from the first entries CSV slice.

## Acceptance examples for future implementation

No test evidence exists yet; use synthetic fixtures only.

- Export more than one page across two years: all owned entries appear, and another
  user's entries do not. A deliberately filtered export clearly states its scope.
- Round-trip an expense of EUR 12.34 and income of EUR 2,000.00: amounts and calendar
  dates remain exact; commas, quotes, and line breaks in descriptions survive.
- Round-trip an entry whose description is just a merchant or payer name: retain
  that text exactly, without requiring a separate counterparty or payment reason.
- Export two identical purchases and restore into an empty destination: both remain.
  Restore the same export again: no new copies are added.
- Round-trip a source-less entry and entries from two equally named sources: the
  source-less entry remains so, and the two sources remain distinct.
- Restore over a locally edited entry: show a conflict rather than overwriting it.
- Invalid selected rows or foreign ownership references reject the mutation atomically.

## Decisions before implementation

Settle the native CSV schema/version, repeat-import identity, conflict choices,
cross-installation source mapping, and treatment of entry method/origin. Native
restore must not invent bank provenance or claim it restored omitted batch history.
Design the small export/reimport UI when this slice is selected; no placeholder
export control is added now.
