# Categories and labels

Status: core and import-review classification implemented. Browser/visual
verification limits are recorded in the [README](../../README.md#verification-status).
Local [labeling rules](labeling-rules.md) are implemented; category/label
comparison charts remain proposed.

## Meaning and boundaries

Each entry belongs to one category bucket and zero or more labels. A missing
category is the built-in **Uncategorized** bucket; users may record/import first
and organize later. Categories answer “What was this money for?”; labels express
occasions, payment patterns, or other overlapping connections. Categories and
labels are flat and user-owned. Sources remain optional, secondary provenance;
classification never requires a source or determines one.

Income/expense is independent of classification. A category does not need income
and spending to balance within it. Classification never infers transfers or
refunds, divides an amount, creates a commitment, or changes import identity.
Existing entries and imports without classification drafts start Uncategorized
with no labels.

## Core management and setup

Home/Entries/Imports offer a nonblocking setup invitation until the user chooses
defaults or skips. **Organize** provides the same setup plus category and label
management. Suggested defaults are individually selectable. Start empty/Skip
records completion without inserting defaults; custom creation works at any time.
Setup can add missing names to an existing catalog without duplicating items or
reclassifying history. Choose defaults remains available after skipping/completing setup; existing
names are preserved. Customization also uses the ordinary management controls.

Categories and labels have trimmed names of 1–60 characters. Names are unique per
user within each kind using Unicode lowercase normalization (no Unicode canonical
equivalence or locale-specific case folding is promised). Archived names remain
reserved. Uncategorized is a reserved category name and cannot be edited/archived.
Renaming preserves identity and all assignments. Labels have one of five muted
colours: sage, clay, sand, lavender, blue; colour never carries meaning alone.

Archive hides an item from new-assignment pickers and the default management list.
Historical entries keep it, and filters can include it through Include archived.
Restore makes it assignable again. Existing archived assignments may be retained
or removed, but cannot be added to other entries without restoration. Merge, hierarchy, expiry, and automatic archiving remain deferred.

## Defaults and physical deletion

Suggested categories include Housing for usual home costs, Accommodation for
hotels/temporary lodging, and Vacation as an optional whole-trip bucket. These
are selectable alternatives: users may categorize a trip broadly as Vacation or
keep detailed categories and use a trip label across them. Adding Vacation to
defaults does not silently add it to existing catalogs; Choose defaults or ordinary
creation is available in Organize.

Delete is available for custom categories and labels, including archived items.
First load a server preview with the exact number of affected entries and dependent
rules. Adjacent confirmation names the consequences: category deletion moves its
entries to Uncategorized; label deletion removes only that label. Amounts, dates,
entry identity, provenance, and unrelated classifications remain. No financial entry
is deleted. This action cannot be undone; archive remains the reversible option.
Uncategorized cannot be deleted.

Affected rules are paused and the deleted action references are detached, as
specified in [labeling rules](labeling-rules.md#target-lifecycle). A fingerprint
covers the item metadata, affected entry IDs/revisions, and dependent rule
IDs/revisions. Delete rechecks this under one write lock. Stale confirmation
requires a fresh review, even when the affected count happens to be unchanged.
Assignment deletion triggers the usual entry revisions/import-removal protection.
Names become reusable after deletion, but recreating a name does not restore its
former assignments or rule actions.

## Entry interaction

The manual form starts with Uncategorized and no labels. Sources live in an
optional disclosure. The category is a quiet folder/name/chevron button, without
a visible “Category” caption. Selecting a category opens searchable choices
directly under the entry, replaces the current category, and closes the picker.
Wide Entries rows place category/label controls to the right of the description
and before the amount. Narrow layouts stack them below the metadata. Open pickers
span the row beneath the entry rather than squeezing into its right-hand column.
In Entries, an unassigned category is shown as a compact “—” button, with the
full Uncategorized name retained in its accessible label, tooltip, picker and
filters. Manual-entry and import pickers retain the explanatory name. Selecting
Uncategorized clears the assignment. Creation is available in search.

Optional labels appear as muted chips, without a “Labels” caption. The plus opens
up to four recently assigned active labels, with alphabetical fallback for unused
labels. Quick choices remain stable while the picker is open. All labels expands
search and creation in the same inline area. On hover/focus, a chip reveals a minus
at its right edge in a small contrasting circle, overlapping the end of the text
without reserving extra width. The whole editable chip is the remove button,
including on touch, where it has a larger hit target. Add/remove requests save immediately
after server confirmation. Failed requests retain assignments and show an error.
Done/Escape closes the picker without undoing saved edits. Creating vocabulary
items is immediate even if the surrounding new-entry draft is later cancelled.

Native buttons, labelled inputs, visible focus, and touch targets support keyboard
and phone use. Descriptions and amounts remain primary. Home's latest entries
show category and labels read-only.

Bulk Set category, Add labels, and Remove labels apply to selected displayed
entries only (1–100 distinct IDs). Actions name their scope, require an explicit
Apply, and preserve unrelated assignments. Category/label and entry ownership is
validated under one write lock; any invalid/foreign ID rolls back the operation.

Assignment changes increment the entry revision for import-removal protection,
even when later reverted. Idempotent additions/category saves and removing an
absent label do not increment it. Renaming/archiving vocabulary does not edit all
associated entries. Matched/known reimports preserve assignments and provenance.

## Import-review classification

Ready new/explicitly-kept rows expose the same category button, label chips, quick
choices, and search/creation picker. Classification edits remain local drafts;
ledger assignments are saved only on final import. New vocabulary items are
explicit creations and survive cancellation, as in the manual form.

Contextual Set category/Add labels/Remove labels actions apply to currently
selected additions only, preserving unrelated drafts. They never classify matches
or known rows. Deselected rows retain local drafts for later selection, but their
drafts are omitted from commit. Changing file or mapping warns that classifications,
descriptions, selections, and decisions will be discarded. Cancel writes no entries,
assignments, provenance, or import batch.

Choosing Match keeps the existing entry's description, category, and labels. If a
row has description or classification drafts, require an explicit discard-or-keep
choice before switching to Match. Explain what was discarded. Pending creation
requests and unresolved draft/match confirmations block final import.

The backend validates record numbers, selection, category/label kind, ownership,
archival state, and label uniqueness under the import write lock. Any invalid
addition draft rolls back the entire batch. Classification applies only to newly
created entries, using the same assignment operations as manual entry. Capture the
batch's initial entry revision after those assignments: initial classification
is part of the import, not a later protected edit. Subsequent classification edits
are protected normally. Known rows ignore retry drafts, preserving existing edits;
explicit classification drafts on matches are rejected. Original CSV evidence and
duplicate identity remain independent of these drafts and of parsing profiles.

## Filtering and totals

Entries combines category selection (any selected category and optionally
Uncategorized), any/all selected label matching, Unlabeled, existing dates, and
description search. Unlabeled and selected labels are mutually exclusive.
Filters reset pagination/selection. Archived items remain available explicitly.
Categories and Labels use compact counted buttons opening searchable multi-select
panels, with a bounded scrollable list and selected choices first. Search only
narrows vocabulary choices; it does not change entry-description search or remove
selected filters. Uncategorized/Unlabeled remain separate quick choices. Each
picker has its own Include archived option; selected archived choices stay visible
even when it is off. Only one picker is open at a time. Opening focuses search;
Done/Escape closes and restores trigger focus, while outside clicks or tabbing out
dismiss without taking focus back. Choices apply immediately and remain open for
multiple selections. On phones panels fit within the viewport width.

Any/All label matching appears in the Labels picker when two or more labels are
selected. The active match mode also remains visible beside the selected chips.
Below the controls, removable chips expose entry type, category and label filters
even with the pickers closed. Clear filters resets those choices and label match
mode, retaining the current dates and applied description search. It does not
change vocabulary or entry assignments. Include archived only changes available
choices; it is not an entry filter. The [reporting spec](reporting.md#entries)
owns period, description search and the adjacent All/Expenses/Income control.

Reloads clear selection; inline picker state survives successful same-query reads
when the entry still matches. Read failures hide old results and offer Retry.

Totals cover all matching entries, not the displayed page. SQL EXISTS predicates
keep every entry counted once when labels overlap. Category buckets, including
Uncategorized, partition spending for the same scope. Income is reported
separately; it is not silently subtracted from category spending. Separate label
totals would overlap and must not be summed or shown as exclusive pie slices.

## API

All routes require session ownership; writes also require CSRF/Origin checks.

- `GET /api/classification`: `{ categories, labels, setup_complete }`; items have
  `id`, `name`, `color`, `archived`. Includes archived items. Categories sort by
  normalized name; labels prioritize most recent surviving assignments.
- `POST /api/classification`: `{ kind: "category" | "label", name, color }`;
  returns 201 with `id`.
- `PUT /api/classification/{id}`: `{ name, color, archived }`; returns 204.
- `GET /api/classification/{id}/deletion-preview`: `{ entry_count, rule_count, token }`.
  `DELETE /api/classification/{id}` requires `{ token }`, returning 204 after a
  fresh owned review. Foreign/missing items return 404; stale review returns 422.
- `POST /api/classification/setup`: `{ categories: string[], labels: string[] }`,
  up to 30 each; adds missing names and completes setup atomically. Empty arrays
  skip. Safe to repeat.
- `POST /api/transactions` additionally accepts nullable `category_id` and
  optional `label_ids` (up to 100 distinct IDs). Entry and assignments commit
  atomically. Existing PUT/PATCH entry operations preserve classification; use
  the dedicated action for assignment changes.
- `POST /api/transactions/classify`: `ids` plus an `action`: `set_category` with
  nullable `category_id`, or `add_labels`/`remove_labels` with nonempty `label_ids`.
  Returns 204. A single entry uses the same action as bulk changes.
- `GET /api/entries` accepts comma-separated `category_ids`, `label_ids`,
  `uncategorized=true`, `unlabeled=true`, `label_match=any|all`. IDs must be distinct
  owned UUIDs (up to 100 per list). Foreign/missing IDs return 404. Filter totals
  share the entry-page read snapshot.
- Entry reads (`entries`, dashboard `latest`, legacy `transactions`) include
  nullable `category` and `labels: []` with the same item shape.

- `POST /api/imports/commit` additionally accepts optional `classifications`, an
  object keyed by CSV record number, with `{ category_id: string | null,
  label_ids: string[] }` per selected addition. Missing category/labels mean
  Uncategorized/no labels. At most 100 distinct labels per row. Unknown or
  unselected records reject the batch. Owned drafts on known records are ignored
  for idempotent retries; match/non-addition drafts are rejected. Archived items
  cannot be newly assigned. Preview does not persist drafts or assignments.
  [Rules](labeling-rules.md) defines suggestion seeding, explicit manual precedence,
  category conflict confirmation, and ruleset-revision checks.

## Acceptance evidence

Synthetic dinner €40 → Eating out + Italy 2027 + Holiday; hotel €120 →
Accommodation + Italy 2027; shop €25 → Uncategorized. Any Italy/Holiday filter
counts €160 once, including with a one-row page; all counts only dinner (€40).
Including both named categories and Uncategorized counts €185. Reassigning dinner
moves €40 between categories without changing spending. Archiving/renaming Italy
preserves historic matches; new assignment is rejected until restored.

`backend/tests/ledger.rs` covers these cases, cross-user operations/filter IDs,
atomic rollback, CSRF, reserved/duplicate names, colour validation, setup
idempotency, import revision protection, repeated imports, and legacy reads.
`frontend/e2e/classification.spec.ts` defines desktop/phone setup, manual creation,
inline pickers, quick labels, creation/removal, filters, archives and screenshots.
`frontend/tests/import-classification.test.ts` covers additive/removal draft edits
and addition-only commit payloads. Import-classification cases in
`backend/tests/ledger.rs` cover atomic rollback (including after earlier rows were
inserted), wrong-kind/foreign/archived items, selected subsets, baseline revisions,
matching and retry preservation. `frontend/e2e/ledger.spec.ts` defines bulk/per-row
review editing, cancellation, draft-to-match discard, commit, and screenshots;
classification journeys also check desktop/phone row placement.
See README for whether those journeys actually ran; source coverage is not visual QA.
`frontend/e2e/entry-filters.spec.ts` defines desktop/phone compact-picker search,
combined type/category/label matching, selected archived retention, Unlabeled,
active-chip removal, Escape/outside dismissal, clear-without-losing-period/search,
and screenshots. `frontend/tests/filter-choices.test.ts` covers vocabulary search,
selected-first order, and selected archived retention.

## Follow-ups

The implemented [local labeling rules](labeling-rules.md) own suggestion behavior,
management, manual-entry drafts, imports, history application, and target lifecycle.

Home's [category spending breakdown](reporting.md#spending-by-category) is implemented.
Category income breakdowns, period comparisons, and label analysis remain
proposed. All figures should link to their underlying entries. Split allocations
are separate later work and must sum exactly to the entry amount.
