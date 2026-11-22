# Import history and removal

Status: implemented for commits made after migration 0007. Browser verification
status is recorded in the README. [CSV import](csv-import.md) owns parsing,
matching, and duplicate semantics.

## History

`/imports` lists import commits newest first, 20 per page, with source, display
filename, import time, valid file date span, added/matched/known counts, and the
number removed through batch removal. The date span describes valid file rows,
including excluded rows; it does not promise continuous coverage. Raw uploads
and client file paths are not stored. Filename is an optional basename, trimmed
and limited to 160 characters; API clients may omit it (display: CSV import).

Each successful commit creates a history record atomically with its changes,
including known-only or empty-selection API commits. Preview/cancellation/failed
commits create none. The UI still disables submission when nothing is selected.
Old import evidence remains valid but is not retroactively invented into batches.
A subset import and a later import of the remainder are separate batches.

## Review and removal

Review opens batch details in place. Only transactions **created** by that batch
can be removed. Manual/existing entries merely matched by the batch are preserved.

Each entry has at most one **creating import**: the batch that first inserted it.
Only that batch's `added` association owns the entry for batch removal. Later
`known`/`matched` associations are provenance and duplicate evidence, not additional
owners. They never transfer ownership or block the creator from removing an entry.
Manual entries matched by imports remain manual and cannot be batch-removed.

Live additions are grouped as unchanged/removable or edited/protected, regardless
of later overlaps. The review reports how many selected entries also appear in
other imports, but does not create a separate protected-overlap group. Removing a
creator removes its entries from the recorded ledger, including their later links.
Removing a later batch only removes its own additions, leaving earlier entries alone.
Already-deleted additions remain historical records. Reimport creates new entries
owned by the new importing batch; deleted originals are not reattached or revived.

The removal section starts open with up to five compact, read-only entry previews
(description, current date, exact amount). Show all expands the list in place.
The edited group starts collapsed, with its count/protection status visible.
It exposes the same preview fields and individual inclusion controls.
Already-deleted entries are a note, not another list. Matched-existing counts are
explained separately and do not inflate protected-addition counts.

Edited entries are protected by default. Each edited entry has its own
unchecked inclusion checkbox; there is no all-edits override. Selecting an edited
entry adds it to the removal preview and updates counts immediately. Its checkbox
remains in the edited group so the choice can be reversed. Editable inclusion
rows reuse the round selector, whole-row highlight, row-background toggling,
keyboard control, and reduced-motion feedback from Entries; read-only previews
have no selection control. Show all/fewer controls
are independent for each group. Reloading clears edited selections.
Changes to date, description, amount, or source increment a
persistent revision, including edits subsequently reverted. Saving identical
values does not increment it. Label editing must participate when implemented.

Remove N entries reveals adjacent Confirm removal and Cancel controls; no centered
modal or automatic focus on the destructive action. The count names the actual
scope, across pages/months. Confirmation is reset when inclusion changes. This
is permanent deletion, not a rollback of matches, edits, sources, or other batches.

The backend rechecks membership, revisions, overlaps, and ownership under the
SQLite write lock. A stale review token rejects removal with an actionable reload
message, rather than silently changing the deletion set. All chosen deletions and
history counts commit atomically. Deletion clears duplicate evidence, allowing
reimport. History remains; already-deleted counts include ordinary entry deletion
and batch removal, whereas `removed` counts only this batch-removal operation.

## API and acceptance evidence

- `GET /api/imports/history?page=1` returns `{ batches, page, pages }`.
- `GET /api/imports/history/{id}` returns `{ batch, removable, edited, overlapping,
  already_deleted, token, removable_entries, edited_entries }`.
  The preview arrays contain current owned `id`, `date`, `description`, and exact
  `amount_cents` strings plus an `overlapping` boolean, newest first. Deleted
  entries have no preview. The top-level overlapping count spans live additions;
  the UI discloses overlaps within the actual current removal selection.
- `POST /api/imports/history/{id}/remove` accepts `{ token, selected_edited_ids: [] }`
  and returns `{ removed }`. It requires session ownership, CSRF, and normal Origin
  validation. IDs must be distinct current edited additions from this
  batch; missing, foreign, unchanged, and other-batch IDs reject the entire
  operation before deletion. The list defaults empty and is limited to 1,000 IDs;
  this route allows a 64 KiB request to accommodate the import's maximum size.
  The token is an opaque review fingerprint, not authorization.

Synthetic example: a batch adds four entries and matches an existing manual entry.
Then one addition is edited, one is referenced by another batch, and one is deleted.
Default removal deletes both unchanged additions, including the overlapping one.
Explicit inclusion can later delete the edited addition; the manual entry remains.
Changing entries after review rejects the old token. Removed entries can be
previewed/imported again; retained entries remain known. Other users cannot list,
review, or remove this batch.

`backend/tests/ledger.rs` covers these persistence/ownership/CSRF/stale-review cases.
Desktop/phone journeys cover a multi-month 32-entry import, cross-page removal,
protected edits, explicit inclusion, subset reimport and repeated preview.

Individual-selection regression: with three edited additions and one unchanged
addition, choosing two edited IDs removes those two plus the unchanged addition.
The third edited entry stays. Invalid/duplicate IDs delete nothing. Preview fields
reflect current edits. Overlapped edited entries still require explicit inclusion.
Dense-overlap coverage proves later batches own only their new additions, creator
removal is not blocked, unrelated entries survive, and removed rows can be reimported.
