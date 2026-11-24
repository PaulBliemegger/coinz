# CSV import profiles

Status: implemented in source. Browser journeys and visual review remain unverified
because the sandbox prevents local server startup; see the
[README verification status](../../README.md#verification-status).
This is the authoritative profile workflow and parsing contract. The
[CSV import spec](csv-import.md) owns shared duplicate, provenance and commit rules.

## Goal and agreed scope

Users can import different CSV layouts by creating reusable import profiles,
without needing a separately programmed adapter for each bank or exporter.
A profile maps CSV columns to transaction fields and describes how to interpret
their values. After choosing or confirming a profile, the user continues to the
existing import overview for validation, selection, and duplicate review.

The first version uses one signed amount column. Separate debit/credit columns
are deferred. File-managed YAML profiles are a desired later capability for
sharing and declarative setup without the UI; see [file-managed profiles](#deferred-file-managed-profiles).
They are not required for the first version.

## Description and saved entries

Use one **Description** field, following the saved-entry requirements in the
[manual-ledger spec](manual-ledger.md#entries-and-reporting). A bank CSV need not
have a column literally named Description: a Name, Counterparty or Reference
column can supply readable text, such as **Example Cafe**. No separate structured
counterparty or payment-reason field is required. Manual entry stays unchanged.

All three primary column mappings are required. If the mapped text cell is blank
or too long, the row needs an edited description in review before it can be added,
or it can be left for later. Never invent a **No description** placeholder.
Description edits affect new entries, while original imported text remains matching
evidence; see [description editing](#description-editing).

Source remains separate provenance identifying an export stream. Structured
counterparties, profiles without a text-column mapping, text composition and
fallback chains remain deferred.

## Import workflow

1. Start an import, select a CSV, and choose or create a source as today.
2. Choose an existing named profile or create a new one within the import flow.
3. For a new profile, show the CSV headers with sample values. Map date, amount,
   and the column to use as Description, which may be named `Counterparty`, `Name`,
   or `Reference` in the file. Optionally map a stable external transaction ID.
   Unmapped columns are ignored. All three main mappings are required to confirm
   the profile; blank cells can be resolved during entry review.
4. Show representative interpreted rows so the user can verify dates, amounts,
   income/expense signs, and text. The user explicitly confirms the profile
   layout and interpretation and saves it with a name for reuse.
5. Continue to the existing import overview. Allow inline description editing,
   including supplying blank descriptions. Mapping confirmation alone never
   imports transactions. Use the existing validation, selected-row totals,
   duplicate decisions, and final import action.

An existing profile avoids repeating column setup, but its mapping must be checked
against each new file. Missing or ambiguous mapped columns require correction
before proceeding. Data errors in individual rows remain visible in the overview;
they do not require discarding otherwise valid rows.

Allow returning from the overview to correct the mapping. Recompute the preview
and clear row selections and duplicate decisions that depended on the previous
interpretation. Cancelling writes no transactions or import evidence. Explicitly
saved profiles and separately created sources may remain, like today's source
creation; make this clear in the flow.

## Profile behavior and parsing scope

Profiles are reusable by their owner across imports and sources with the same
layout. The profile describes a format, not an account identity. Choosing a
different profile must not bypass source-scoped duplicate protection.

Save named profiles persistently for the authenticated user. Reusing a compatible
profile goes directly to entry review; checking or changing its mapping remains
available. Distinguish saving a new profile from explicitly saving changes to an
existing one, and preserve its stable identity when updating it. Profile deletion
remains deferred. The Imports tab also supports creating and editing profiles
independently of an import; see the editor below.

Implemented parsing scope:

- UTF-8 CSV with a header row, ordinary quoting, and comma or semicolon delimiters.
  At most 100 named columns; header names are at most 200 characters.
  Headerless files, preambles, and other encodings can be later extensions.
- Map columns by header rather than position so reordered columns still work.
  Do not silently choose between duplicate header names.
- Store one explicit date format: `YYYY-MM-DD`, `DD.MM.YYYY`, or
  `DD/MM/YYYY`. Do not guess the meaning of an ambiguous date per row.
- Store an explicit dot or comma decimal separator, and no grouping, dot, comma,
  or ASCII-space grouping (different from the decimal separator). Grouped whole
  numbers have 1–3 leading digits and groups of exactly three digits. No currency
  symbols, exponent notation, or plus prefix are accepted. For example, `-1.234,56` can
  mean an expense of EUR 1,234.56 under the appropriate settings. Parse exactly
  into integer cents; reject excess precision instead of rounding.
- Use the agreed signed amount column: positive income and negative expense.
  Preview the interpretation before confirmation. Zero remains invalid.
- Allow an optional external-ID mapping so supported exports retain today's
  stable-ID duplicate protection. Profiles without IDs use existing review rules.

No custom scripts, arbitrary formulas, or bank-specific code are needed for this
first mapping flow. Automatic format detection may suggest settings later, but
explicit confirmed settings define interpretation. EUR remains the only currency;
profiles do not add currency conversion or make mixed-currency exports valid.

## Profile editor in Imports

The Imports tab has a collapsible **Import profiles** section alongside history.
List owned profiles with their names, primary mappings and reading presets. **New
profile** and **Edit** open an inline form for the name, exact CSV header names
(Date, Amount, Description, optional External ID) and the existing parsing presets.
No file, sample values, source or visual picker is needed. Header spelling, spaces
and capital letters matter; the actual file compatibility check still happens at
import time. A text column named Name or Counterparty can supply Description.

Saving creates a profile or updates its stable identity with revision checking.
It creates no entries, provenance or import-history batch and never rewrites past
imports. Cancel/Escape discards the local form. Unsaved fields survive list refresh;
a stale save is rejected and asks the user to cancel, reload and reopen the latest
profile. A successful save updates the local catalog; the import picker loads saved
profiles on opening. Profile deletion and automatic profile suggestions are deferred.

## Existing guarantees and profile changes

The [CSV import spec](csv-import.md) remains authoritative for source ownership,
duplicate review, selected subsets, atomic commit, provenance, user-edit
preservation, and upload limits. Its fixed example header/parser contract remains
available for legacy API clients that omit a profile. The UI uses saved profiles
for both example and mapped CSVs.

The backend validates the mapping and reinterprets the file at commit using the
same confirmed settings as the preview. A profile change between preview and
commit must require a fresh preview rather than silently changing the result.
Changes to a saved profile affect future imports, never historical transactions
or the original evidence explaining prior imports. The same source/file/record is
known only when its original normalized date, description, cents and external ID
agree. A changed interpretation is a conflict,
left out in the UI; check the mapping or correct the file. A new file with an
existing external ID and changed original fields also conflicts. Changing profiles
never creates another export stream. ID-less rows in different files retain the
existing exact-field candidate rules; mapping cannot prove identity after arbitrary
changes to those fields. No conflicting row overwrites an entry or its evidence.

Profiles and all profile operations are scoped to the authenticated user.
Sample transaction values belong to the current preview, not to the reusable
profile definition. Keep uploads and financial contents out of logs.

## Approved interaction design

The implemented import flow uses Guided selection in a modal. A dedicated-page
workspace and automatic profile suggestions remain deferred; they are separate
from the existing file-free profile editor in Imports.

### File and profile

- Keep the selected file and source visible as context. Source and profile are
  separate choices; a layout can be reused for more than one source.
- Offer a saved-profile selector, a compact mapping summary, **Check column
  mapping**, and **Create a new profile**. Validate the saved mappings against
  the new file before enabling direct continuation to review.
- Use three clear stages: **File & profile**, **Map columns**, **Review entries**.
  Existing-profile imports need not repeat the mapping step.

### Map columns

- Show a real CSV table with headers and representative original values. Display
  empty cells explicitly and identify that the table shows a sample, not all rows.
- Above the table, show three field controls: Date, Amount, Description, each with
  its assigned CSV header or **Choose column**. Clearly mark the current field
  and progress, for example **1 of 3 mapped**.
- Prompt **Choose the Date column**. Activating a header assigns it and advances
  to Amount, then Description. For Description, explain that a name, counterparty,
  or reference column is suitable. No dedicated counterparty input is shown.
- Give Date a blue tint, Amount an ochre tint, and Description a plum tint,
  shared by their numbered field controls, header badges and column highlights.
  Keep text labels and active-field outlines so colour is never the only cue.
  Use a wider desktop mapping workspace with separated reading/preview panels;
  stack controls and panels on phones. Selecting a column configures a mapping,
  not an import-row selection.
- Keep the CSV table connected with vertical column borders. Available headings
  show **Do I describe Date/Amount/Description?** for the active field, without
  separate button boxes or pulsing. Hovering or focusing a heading highlights its
  whole column; native keyboard focus remains visible. On phones, keep these
  interactive headers visible and clickable inside the horizontally scrollable
  sample table; do not transform the sample into headerless cards.
- Allow direct remapping by activating any of the three field controls. After all
  mappings are complete, show that state rather than silently assigning another
  field when a header is clicked.
- Place date, decimal/grouping, and delimiter settings in an expandable **Reading
  the values** section. Keep sign conventions and EUR interpretation visible when
  those settings are open. Interpretation remains explicit, not guessed per row.
- Show a separate **Entry preview** with interpreted descriptions, dates, and exact
  amounts; the original CSV table stays unchanged. Surface row problems without
  requiring the user to fix every row before reaching review.
- Place the profile name and **Save profile & review** action at the bottom; use
  explicit save-changes wording for an existing profile. Required mapping or
  structural errors block continuation; individual bad/blank cells go to review.

### Review and responsive behavior

Use the existing attention-first import review and persistent bottom action. Show
blank descriptions as an actionable **Add description**, not invented saved text.
Valid rows remain selectable while other rows are left for later. The bottom
summary names additions, matches, selected income/spending, and rows left out.
Description editing follows the next section.

Changing a mapping invalidates dependent preview choices and draft descriptions.
Warn before discarding them, with an in-place choice to continue changing the
mapping or keep reviewing. Recompute the preview before final import.

Follow the existing [design philosophy](../design.md): cream/warm-brown surfaces,
terracotta primary action, serif heading, compact readable rows, and native
controls with visible keyboard focus. State labels accompany color. On phones,
the table can scroll horizontally inside its own region; mapping controls, active
prompt, errors, and primary action must reflow without page-wide overflow or tiny
text. Support header activation by keyboard and comfortable touch targets.

## Description editing

Prefill Description from the selected CSV column and allow adding or changing it
inline in import review. A user can replace **Example
Cafe** with **Lunch with Sam**, or leave the imported name as it is. Existing inline
entry editing provides the same opportunity after import. This does not require a
second user-facing text field or a general spreadsheet editor.

Most imported rows should already have useful text and need no typing. A blank
description must be supplied before adding a row; it cannot be deferred until
after an otherwise undescribed entry is saved. A nonempty description may always
be refined after import. Activate descriptions directly; show a small native
input with Save/Cancel and Escape to cancel, following the ledger's editing style.
Do not save on blur or turn the raw CSV table into a spreadsheet editor.
Finish or cancel open description editors, draft-to-match confirmations, and
mapping-discard confirmations before the final import action. Navigation controls
are disabled while an import commit is pending.

Draft descriptions apply to new additions only at final commit. Preserve the
normalized original row as matching evidence; a
user's description edit must not change file/bank identity or defeat repeat-import
detection. Matching an existing entry retains its user-edited fields; do not use
a draft description as an implicit update to a matched entry. When changing an
edited row to a match, explicitly show that the existing entry's description is
kept and disclose any draft text being discarded. The server revalidates draft
descriptions along with selection/ownership and commits all selected writes
atomically; preview edits alone do not write ledger entries.

## Deferred: file-managed profiles

Desired future capability: define profiles in version-controlled files and make
them available without creating each profile in the UI. Recommended design,
subject to implementation review:

- Read versioned YAML definitions from an explicitly configured, read-only mounted
  directory, for example `profiles/*.yaml`. Exact paths, environment variable
  names, and reload mechanics are undecided; no loader exists today.
- Let deployment tooling deliver/reconcile those files. coinz need not clone a
  repository, hold Git credentials, or run a GitOps controller. Reading mounted
  files enables configuration as code; automated delivery/reconciliation belongs
  to the deployment workflow ([OpenGitOps principles](https://opengitops.dev/)).
- Use a schema version and stable profile key, independent of the display name.
  Reapplying the same definitions is idempotent. Validate the entire update before
  activating it; invalid definitions produce actionable configuration errors and
  leave the last valid catalog intact. Reject ambiguous key collisions.
- Define file-managed versus UI-managed ownership explicitly. File-managed
  profiles are read-only in the UI; users can make an independently owned editable
  copy. A later file update must not overwrite that copy.
- Keep portable layout definitions separate from private user/source bindings.
  Resolve availability through an explicit deployment policy; do not grant access
  to another user's saved profiles, sources, or entries. Decide whether files
  publish shared templates or per-user definitions before implementation.
- Store the confirmed profile interpretation with import evidence, so updating,
  removing, or reverting a file does not reinterpret history. Removing a profile
  makes it unavailable for new imports and never deletes imported entries.
- Include only layout/settings in shareable definitions: no sample rows, financial
  values, credentials, or installation-specific source/user IDs. UI-created
  profiles should eventually export to the same portable schema.

Other declarative setup candidates are source definitions and, when implemented,
label definitions, labeling rules, and planning templates. These are ideas for
later scope, not part of this slice. Sources currently have a UI/authenticated API
but no declarative setup; user creation already has a CLI and deployment is already
file-configured. Browser appearance/selection preferences remain personal UI
settings. Transaction data belongs in private exports/backups, not configuration
Git history; see [entry export and reimport](data-portability.md).

## Acceptance examples

These scenarios define the implemented behavior. Automated evidence and remaining
verification limits are listed below; all fixtures are wholly synthetic.

| Scenario | Expected result |
| --- | --- |
| Map `Booking date`, `Value`, and `Reference` | They become date, signed amount, and readable Description in the existing overview |
| Map the CSV's `Name` or `Counterparty` column to Description | Its nonempty value is a sufficient description for income or expense; no separate reason is required |
| Manually enter `Lunch with Sam` | Save with date and amount without requiring a separate counterparty |
| The file has no column named `Description` | Another suitable text column can supply Description |
| The mapped text cell is blank | The row cannot be added until text is supplied; it can be left out for later |
| Add or change a description during review | The new entry gets that description at commit; repeat import still uses original evidence |
| Map Date, then Amount, then Description | Each header click assigns the active field and advances; all three assignments remain visible and revisitable |
| Cancel description editing or the whole import | Draft text changes do not write any ledger entry |
| Change a mapping after editing descriptions | Warn before clearing drafts/selections; rebuild preview after confirmation |
| Use a saved compatible profile | Go straight to review without repeating column setup |
| Reuse a profile on a file with reordered headers | Header mappings still resolve without manual remapping |
| A required mapped header is missing | Ask the user to correct the mapping; never substitute another column silently |
| Confirm the profile layout | Open the import overview without creating any transactions |
| Reimport into the same source with another profile | Existing duplicate/provenance checks still apply |
| Edit a saved profile after an earlier import | Earlier transactions and their original import evidence stay intact |
| Create/edit a profile in Imports without a file | Save its owned layout without writing entries, evidence or history; editing preserves the profile ID |
| Cancel the profile editor | Saved profile stays unchanged |
| Resolve a draft-to-match confirmation | Import stays disabled until the user keeps the draft or confirms discarding it; the existing entry is never overwritten |
| Another user requests a profile by ID | No access or mutation is allowed |

## Other deferred work and remaining design choices

- Separate debit/credit columns, unsigned amounts plus a direction column,
  multi-column text composition, and other transformations.
- Additional date/number conventions, encodings and delimiter detection remain deferred;
  the implemented presets do not imply arbitrary bank-export support.
- Structured counterparty metadata, profiles without any description-column
  mapping, and profile deletion remain deferred.
- Transfer/refund semantics remain a separate prerequisite for safely supporting
  exports containing those movements; mapping alone does not classify them.

## Persistence and API

The schema stores owned profile IDs, names, layouts and monotonically increasing
revisions. Names are trimmed, 1–80 characters. Updating requires the current
revision and preserves identity. No sample values belong to a profile. Each new
batch and evidence row stores an immutable JSON layout snapshot; older evidence
has a null snapshot identifying the legacy example parser. Raw uploads are not stored.

All endpoints require authentication; writes, including sample/preview requests,
require CSRF and Origin validation through the existing middleware.

| Endpoint | Contract |
| --- | --- |
| `GET /api/imports/profiles` | Owned `{ id, name, revision, layout }[]` |
| `POST /api/imports/profiles` | `{ name, layout }`, returns the created profile (201) |
| `PUT /api/imports/profiles/{id}` | `{ name, layout, revision }`, returns the updated profile |
| `POST /api/imports/sample` | `{ csv, delimiter }`, returns `{ headers, sample, total }`; first five original records |
| `POST /api/imports/preview` | Existing import request plus `profile_id` and `profile_revision`; a draft `layout` instead is allowed only for mapping preview |
| `POST /api/imports/commit` | Existing import request plus saved `profile_id`, `profile_revision`, optional `descriptions` keyed by record number |

A layout contains header names `date`, `amount`, `description`, nullable
`external_id`, and explicit `delimiter`, `date_format`, `decimal`, `grouping`.
Mapped fields use distinct headers; missing/ambiguous mapped headers reject preview
and commit. Unmapped columns remain ignored. Sample requests share CSV bounds
and structural validation with mapped parsing. The CSV endpoints have the existing
2 MiB JSON limit; profile operations retain 16 KiB.

Both preview and commit reparse the upload. At commit, profile revision checking
and duplicate revalidation run under the same SQLite write lock. Stale revisions
require another preview. Preview has separate `description_error` and structural/value `error`:
blank or overlong text can be corrected without changing original matching evidence.
A description override is trimmed and validated (1–200 characters), may name only
a selected record, and applies only to a new addition. Matches/known rows retain
existing entries. Invalid selected rows or descriptions roll back the entire commit,
including batch history and evidence.

## Verification

`backend/src/imports/parse.rs` tests reordered/missing/ambiguous headers, explicit
date presets, grouped exact cents, invalid precision/signs/overflow, and blank text.
`backend/tests/ledger.rs` adds temporary-database profile ownership/CSRF, revision,
immutable evidence, atomic description validation, original-field matching, and
cross-profile/repeated-file regressions. Existing import/history/concurrency tests
continue covering shared guarantees. `frontend/tests/import-selection.test.ts`
covers description eligibility with exact selection totals.

The desktop/phone Playwright journeys include profile creation/editing without a
file, Cancel/Escape, stable IDs/revisions, picker reuse, unchanged history, and
light/dark editor screenshots. They also include keyboard header mapping, semicolon
and explicit parsing presets, description Save/Cancel/Escape, draft-discard warning,
saved-profile reuse, repeat import after edits, and light/dark synthetic screenshots.
They are defined coverage, not completed browser assertions or visual review.
The sandbox blocks local server startup, and the previously declined escalation
has not been retried. See the README for the complete check results and limits.
