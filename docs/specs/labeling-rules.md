# Local labeling rules

Status: implemented. Browser/visual verification limits are recorded in the
[README](../../README.md#verification-status). This feature uses local,
deterministic description matching; no AI, external service, merchant database,
or silent learning is involved. [Classification](classification.md) owns category
and label meaning, assignment, archival, and deletion.

## Rule definition and management

Organize → Rules lists owned rules with conditions, category/label actions,
enabled/paused state, and editing controls. New/Edit opens an inline form. A rule
has a name (1–60 trimmed characters), one description condition, optional source
restriction, and at least one action. Source restriction is under Advanced and
defaults to all sources and source-free manual entries. It is independent of CSV
parsing profiles.

Conditions: equals, contains, or contains word, with 1–200 trimmed characters.
Matching uses Unicode lowercase conversion. Contains is a literal substring, not
regex/SQL wildcard syntax. Equals matches the whole description. Contains word
requires one alphanumeric word; punctuation and non-alphanumeric characters mark
word boundaries. It matches “COFFEE-shop” but not “coffeeshop”. Unicode canonical
equivalence and locale-specific case folding are not promised.

Actions set a category (or leave it unchanged) and/or add distinct labels (up to
100). Enabled rules must target active owned items. Paused rules may retain
archived targets for editing. Choosing no actions is invalid. Edits require the
current rule revision, preventing stale saves. Rules can be deleted with explicit
confirmation; existing entry classifications are preserved.

Preview matching entries evaluates the draft against owned recorded descriptions
and optional source scope. It reports the exact match count and up to ten examples,
with dates and exact amounts. Changing the draft hides stale preview results.
Saving a rule changes future suggestions only; it does not rewrite history.

## Suggestion semantics

Evaluate all enabled matching rules. Distinct suggested labels combine; identical
category proposals agree. Different nonempty category proposals produce a conflict,
with candidate IDs and matching rule names rather than an arbitrary winner. Rules
that only add labels do not introduce category conflicts. Empty category actions
mean “leave unchanged”, never “clear”. A source-restricted rule applies only when
the entry has that exact owned source.

Manual entry requests suggestions as description/source changes and ignores
superseded responses. A disclosure explains matching rules. Use suggestions for
untouched fields explicitly accepts proposals; manually chosen category (including
Uncategorized) and label sets remain unchanged. Conflicting categories are not
automatically selected. Users can still record an entry manually or without labels
if suggestion reads fail. General entry-creation API calls do not run rules silently.

After successful manual entry, a contextual Create rule action opens a prefilled
rule draft. Entries offers the same action after manual classification edits. The
draft uses the exact description with Equals and the chosen category/labels.
The user can broaden it, preview, save, or cancel. Entry-origin drafts use a dialog;
Organize uses the inline editor. Neither path silently learns or saves a rule.

## Import review and consistency

Preview evaluates rules for eligible new/possible-duplicate rows and returns a
ruleset revision. Suggested classifications seed local drafts for new rows and
explicitly kept purchases. Matching-rule disclosures explain the result. Known or
matched entries preserve their stored classification. Existing draft-to-match
confirmation applies to suggestion drafts too.

Description edits refresh suggestions independently of original import evidence.
Manually edited category and label sets retain precedence when suggestions refresh.
Unspecified dimensions follow the new proposals. A failed refresh blocks importing
the affected selected row until refreshed/reviewed. Conflicting selected additions
require an explicit category choice; explicitly choosing Uncategorized is valid.
Remove suggestions clears only untouched suggested dimensions, preserving manual
choices.

Commit verifies the preview ruleset revision under the same SQLite write lock as
duplicate validation. Rule creation, edits, enable/pause, deletion, and affected
target archive/delete invalidate stale previews. The user must review again;
there are no silent new rule applications. Supplied classification drafts are
authoritative and revalidated. For an addition without a draft, rules can supply
classification only when a rules revision was provided. Conflicts still block.
Legacy API imports omitting the revision retain their explicit/manual behavior.
Known-row retry drafts never overwrite later entry edits.

Initial rule assignments are part of the creating import's revision baseline;
subsequent manual or explicit historical-rule changes are protected edits.

## Applying to existing history

Preview history is a separate explicit action for an enabled rule. It covers all
owned recorded history in that rule's source scope. Only Uncategorized entries
receive a category; existing categories remain. Missing labels are added without
removing existing labels. This explicit action may re-add a previously removed
label, which is why it requires a reviewed confirmation.

The preview reports the affected-entry count and up to ten examples with proposed
category/label additions. Confirmation names the full affected count, not merely
the example count. A fingerprint covers the ruleset revision, rule revision, and
matching changes/entry revisions. Apply re-evaluates under a write lock; any stale
affected set rejects the operation. All changes commit atomically. This action
never creates/deletes financial entries or changes amounts, dates, or provenance.

## Target lifecycle

Archiving a targeted category or label pauses dependent rules with an explanation.
Restoring a target does not resume them. Deleting a target pauses dependent rules
and removes the deleted action reference; other actions remain for review. A rule
left without actions must be repaired or deleted. The classification deletion
warning includes both affected entry and dependent rule counts. Rule deletion
does not reverse past applications.

## API

All routes are session-owned. All POST/PUT/DELETE requests require CSRF/Origin
protection, including read-only POST previews/suggestions.

- `GET /api/rules`: `{ revision, rules }`, including paused rules. A rule includes
  `id`, `name`, `operator`, `pattern`, nullable `source_id`/`category_id`, `label_ids`,
  `enabled`, `revision`, and nullable `pause_reason`.
- `POST /api/rules`: definition fields above except ID/pause reason; returns 201
  with ID. `PUT /api/rules/{id}` requires the current `revision`; returns 204.
- `DELETE /api/rules/{id}` requires `{ revision }`; returns 204.
- `POST /api/rules/preview`: draft definition; returns `match_count` and `examples`.
- `POST /api/rules/suggest`: `{ description, source_id? }`; returns `revision`,
  `matched_rules` with conditions/actions, `category_id`, `category_candidates`,
  `label_ids`, and `conflict`.
- `GET /api/rules/{id}/apply-preview`: `affected_entries`, `examples`, and `token`.
  `POST /api/rules/{id}/apply` requires `{ token }`; returns 204.
- Import preview additionally returns `rules_revision` and per-row
  `rule_suggestions`. Commit accepts optional `rules_revision`. Classification
  drafts include optional `category_confirmed`/`labels_confirmed` markers; only
  explicit category confirmation resolves a conflicting proposal.

## Acceptance and verification

- Two matching rules proposing Eating out with overlapping labels produce one
  category and a deduplicated label union. A third rule proposing Shopping produces
  an unresolved category conflict. A source-restricted rule does not match manual
  entries or another source, and another user's rules never contribute.
- A preview followed by a rule edit rejects import confirmation without writing
  entries or a batch. Choosing Uncategorized explicitly resolves a conflict;
  clearing suggested labels is respected at commit.
- Applying a historical Coffee rule fills an Uncategorized entry but preserves a
  manually assigned Shopping category. Existing labels stay. Entry/rule changes
  after preview reject stale application, and applying an already-satisfied rule
  affects zero entries.
- Archiving/restoring a target leaves its rule paused; physical target deletion
  preserves financial entries and unrelated classifications and detaches the rule
  action. Counts and revisions are rechecked before deletion.

`backend/src/rules/matching.rs` tests literal/case/word matching. Synthetic API and
temporary-SQLite cases in `backend/tests/ledger.rs` cover ownership, suggestions,
conflicts, source scope, stale revisions, import baselines, historical previews,
deletion counts/atomicity, and target lifecycle. Frontend import tests cover manual
precedence across suggestion refresh. Browser journeys define responsive rule
editing, suggestions, conflicts, and deletion confirmations; the README records
whether browser assertions and screenshots actually ran.
