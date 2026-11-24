# Code cleanup plan

Status: proposed, 2026-11-25. This is a temporary implementation plan following the
repository review, not a feature roadmap. No code cleanup has started. It does
not block the first Pi deployment; keep that trial and cleanup as separate slices.
Remove this plan when its useful work is complete or superseded. Durable decisions
belong in [architecture](architecture.md), and behavior belongs in the
[feature specs](specs/README.md).

## Goal and boundaries

Make existing behavior easier to find, change and test. The current feature
folders, single Rust crate and single-container runtime remain appropriate.
Large files are candidates because of mixed responsibilities, not because they
exceed an arbitrary line limit.

Work on one slice at a time, with one reviewable commit per slice when authorized.
Preserve observable behavior, API contracts, schema, ownership, exact money/date
rules, import evidence and duplicate decisions. Use synthetic fixtures only under
[the privacy rules](privacy.md). New abstractions, dependencies, router rewrites,
a design-system overhaul and database changes are outside this plan.

## Suggested sequence

### 1. Remove obsolete styles

Start in [app.css](../frontend/src/routes/app.css). The review found responsive
rules for `.accounts-panel`, `.account-row` and `.account-balance` with no matching
usage elsewhere in the current frontend. Recheck usage, including dynamic classes,
before removing each rule.

Keep shared theme tokens, typography and reusable controls global. Move feature
styles locally only when changing that feature; do not relocate the entire
stylesheet at once.

Acceptance: retired rules are gone, live selectors remain, and synthetic desktop
and phone views retain their layout in light/dark mode. Run frontend formatting,
lint, type/build checks and review affected screenshots. Run the full suite before
handing off the implementation slice.

### 2. Organize backend integration tests by feature

[backend/tests/ledger.rs](../backend/tests/ledger.rs) is approximately 4,800 lines
and covers authentication, entries, reports, imports, classification, rules,
budgets and portability. Separate those cases into feature modules while retaining
a small shared temporary-database/API harness.

Prefer retaining one integration-test entrypoint if that preserves the existing
execution model with less change. Keep cross-feature regression cases intact;
place them with the feature whose behavior they primarily protect. Do not create
a fixture framework or rewrite assertions during the move.

Acceptance: all existing cases remain discoverable, meaningful assertions and
Alice/Bob isolation checks survive, and temporary databases remain independent.
Run Rust formatting, Clippy and the full Rust suite; compare discovered tests
before/after. Update documentation references to moved coverage, then run the full
project checks. Add tests only for an actual coverage gap or bug found during work.

### 3. Extract the manual entry flow from the application shell

[App.svelte](../frontend/src/lib/App.svelte) currently combines session/navigation,
manual entry, source creation, rule suggestions and settings. Start with the manual
entry dialog and its local draft/validation state as one extraction. Reassess the
remaining shell afterward; source creation or settings can be separate later slices
if they still have independently understandable responsibilities.

Keep session ownership, shared source/catalog refresh and route coordination in
the shell. Use explicit component inputs and callbacks. Preserve confirmation of
successful saves separately from any subsequent refresh error; avoid introducing
global stores or a new routing architecture.

Acceptance: manual income/expense creation, optional source selection, category/
label choices, rule suggestions and manual overrides behave as before. Cancel,
Escape, focus handling, save failures, expired sessions and post-save refresh
behavior remain correct. Run focused frontend/browser checks and desktop/phone
visual review, then the full suite before completing the slice.

### 4. Separate import setup from review coordination

[ImportDialog.svelte](../frontend/src/lib/features/imports/ImportDialog.svelte) is
approximately 960 lines and coordinates file/profile setup, optional sources,
review drafts, rule suggestions, duplicate choices and commit. Extract the file/
profile setup first, retaining clear ownership of the upload and review state.
Keep the existing selection/classification helpers rather than replacing them.

Treat bank imports and native coinz restoration as distinct supported paths within
the existing workflow. Preserve generation/revision guards, selected rows, draft
edits, explicit match/keep decisions, known-row handling and atomic commit behavior.
Do not change the API or add a generic workflow/state-machine framework as part of
this extraction.

Acceptance: guided profile mapping, description/classification edits, cancellation,
source-free imports, duplicate review, native CSV restoration and repeated imports
all retain their existing results. Same-file reselection and stale-response guards
remain covered. Run focused import/portability journeys in both layouts, inspect
affected screenshots, then run the full suite.

## Optional follow-ups

These are separate decisions after the slices above or Pi feedback, not work to
bundle into those commits:

- **Ignore-file housekeeping:** replace unrelated generated framework templates
  and duplicates in [.gitignore](../.gitignore) with a small set matching the repo.
  Preserve protection for databases/sidecars, private imports, `.env` files,
  generated builds, browser evidence and editor files. Check representative fake
  paths with `git check-ignore` and confirm synthetic fixtures remain trackable;
  never scan private data to validate ignore patterns.
- **Browser test organization and parallelism:** split the roughly 900-line
  [ledger journey file](../frontend/e2e/ledger.spec.ts) by behavior when useful.
  File splitting alone does not isolate data. Before enabling two workers for the
  full suite, provide independent accounts/fixtures for concurrent tests and remove
  cleanup that can delete another test's entries. Prove the full suite passes with
  two workers and that repeated isolated runs leave no fixture collisions. Until
  then retain the current one-worker default.

## Completing each slice

Use [development checks](development.md#tests-and-checks) and the scoped agent
instructions. Capture the current passing baseline before moving code; distinguish
behavior failures from an unavailable browser environment. Keep required CI checks
blocking and use owner-run local/CI browser results when the agent cannot execute
them. Report that distinction rather than claiming an unexecuted suite passed.

Review the diff for accidental behavior changes and update only affected durable
docs. Stop after a useful boundary; do not add line-count targets, duplicate tests
or new tooling merely to make the repository look more uniform.
