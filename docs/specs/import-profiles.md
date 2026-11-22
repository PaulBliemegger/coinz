# CSV import profiles

Status: proposed implementation; behavior and **Guided selection** design approved,
not implemented. This is the authoritative feature definition for the next
implementation chat. Parsing details marked as recommendations remain bounded
implementation choices. The [example CSV importer](csv-import.md) describes current
behavior; approving this spec does not change the running application.

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

## Shared transaction fields

Agreed rule: every saved entry must have readable descriptive text, whether it
was entered manually or imported. A bank CSV need not have a dedicated description
column: a counterparty/account-holder name can supply that text.

Use one primary **Description** field. Defer a separate structured counterparty
field until a concrete need justifies it. The normal manual form keeps its
existing small field set:

| Field | Required on a saved entry | Meaning |
| --- | --- | --- |
| Date | Yes | Calendar date of the transaction |
| Amount | Yes | Nonzero EUR amount; positive is income, negative is expense |
| Description | Yes | Readable text identifying the entry: a name, reference, or the user's own wording |

For example, a user can enter **Lunch with Sam** manually, or an import profile
can map the bank's `Counterparty` column to Description and produce **Example Cafe**.
Both are valid descriptions. The field does not require prose or a payment reason.
If the bank supplies a useful `Reference` instead, that column can be chosen.

This distinguishes the input layout from the saved-entry requirement. A dedicated
CSV description column is optional; readable text on the final entry is required.
If the mapped cell is blank, the row needs a description before it can be added,
or it can be left out for later. Do not satisfy the rule with an automatically
invented **No description** value. The approved first flow maps all three fields;
allowing a profile with no text-column mapping is a later extension, if needed.

Manual entry stays short and keeps its current amount plus income/expense control;
it need not expose a signed-number input or ask whether a name belongs in a
different field. The current manual form and fixed example importer already
require descriptions; no runtime behavior changes here. Description limits and
exact money/date rules remain in the [manual-ledger spec](manual-ledger.md).

Counterparty names are suitable input for Description, without introducing
another entry field. Source remains separate provenance identifying an account's
export stream. Structured counterparties, column combination, and fallback chains
are deferred; they are not requirements for this profile slice.

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

## Profile behavior and recommended parsing scope

Profiles are reusable by their owner across imports and sources with the same
layout. The profile describes a format, not an account identity. Choosing a
different profile must not bypass source-scoped duplicate protection.

Save named profiles persistently for the authenticated user. Reusing a compatible
profile goes directly to entry review; checking or changing its mapping remains
available. Distinguish saving a new profile from explicitly saving changes to an
existing one, and preserve its stable identity when updating it. Profile deletion
and a separate management screen are deferred unless needed for the core flow.

Recommended first scope, to confirm before implementation:

- UTF-8 CSV with a header row, ordinary quoting, and comma or semicolon delimiters.
  Headerless files, preambles, and other encodings can be later extensions.
- Map columns by header rather than position so reordered columns still work.
  Do not silently choose between duplicate header names.
- Store an explicit date format, for example `YYYY-MM-DD`, `DD.MM.YYYY`, or
  `DD/MM/YYYY`. Do not guess the meaning of an ambiguous date per row.
- Store explicit decimal and grouping conventions. For example, `-1.234,56` can
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

## Existing guarantees and profile changes

The [CSV import spec](csv-import.md) remains authoritative for source ownership,
duplicate review, selected subsets, atomic commit, provenance, user-edit
preservation, and upload limits. Its fixed example header/parser contract is
current behavior, not a restriction on this proposed mapping feature.

The backend validates the mapping and reinterprets the file at commit using the
same confirmed settings as the preview. A profile change between preview and
commit must require a fresh preview rather than silently changing the result.
Changes to a saved profile affect future imports, never historical transactions
or the original evidence explaining prior imports. Define how conflicting
reinterpretations of previously imported rows are reviewed before implementation;
do not silently overwrite edits or treat a profile change as a new export stream.

Profiles and all profile operations are scoped to the authenticated user.
Sample transaction values belong to the current preview, not to the reusable
profile definition. Keep uploads and financial contents out of logs.

## Approved interaction design

**Guided selection** is the approved design. The alternative **Header menus** is
not selected for implementation. Prototypes are synthetic chat artifacts outside
the repository; the description below is self-contained so another chat does not
depend on a machine-specific artifact path or earlier conversation history.

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
- Give mapped headers text badges, with a quiet highlight through the selected
  column. Selecting a column configures a mapping, not an import-row selection.
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

The profile picker shows layout separately from source. Saved mappings are checked
against the chosen file before review. Future file-managed profiles show a
read-only explanation and **Make an editable copy**; ordinary imports can reuse
them without opening an editor. This design state is deferred, not a
requirement to implement file loading in the first slice.

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

## Acceptance examples for future implementation

These are proposed acceptance criteria, not completed test evidence. Use wholly
synthetic CSVs and temporary databases.

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
| Another user requests a profile by ID | No access or mutation is allowed |

## Other deferred work and remaining design choices

- YAML upload/download and sharing use the future file schema above. The schema
  itself and file lifecycle remain undecided.
- Separate debit/credit columns, unsigned amounts plus a direction column,
  multi-column text composition, and other transformations.
- Confirm the recommended delimiter, date, number, and encoding scope using
  synthetic examples before implementation; do not claim every CSV is supported.
- Structured counterparty metadata, profiles without any description-column
  mapping, profile deletion, and a separate management screen are deferred.
- Choose the concrete database/API shape and immutable import-interpretation
  mechanism during implementation; YAML schema remains deferred.
- Transfer/refund semantics remain a separate prerequisite for safely supporting
  exports containing those movements; mapping alone does not classify them.

## Verification and handoff

No production implementation or acceptance evidence exists yet. Prototype logic
checks are design aids, not proof of browser rendering or accessibility.
Implementation needs parser tests, temporary-database ownership and duplicate
regressions, browser journeys, and synthetic desktop/phone/light/dark visual review.
Follow the [feature workflow](../feature-workflow.md) for checks and review.

Temporary continuation context, prototype-check details, and a starter prompt are
in the [implementation handoff](../handoffs/import-profiles.md). Remove that handoff
after implementation and retain this spec as the authoritative behavior.
