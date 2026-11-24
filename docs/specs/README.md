# Specifications

These documents define observable behavior and examples, not every function or
screen. Read [product scope](../product.md) for goals and sequencing, and
[architecture](../architecture.md) for implementation boundaries.

| Spec | State | Scope |
| --- | --- | --- |
| [Manual ledger](manual-ledger.md) | Implemented | Identity, optional sources, entries, period totals, preserved legacy data |
| [CSV import and duplicate review](csv-import.md) | Implemented | Example format, shared duplicate review, atomic commit |
| [CSV import profiles](import-profiles.md) | Implemented; browser verification blocked | Reusable profiles and independent editor, guided CSV table mapping, editable import review |
| [Dashboard, Insights and Entries](reporting.md) | Implemented; browser verification blocked | Home, monthly Insights report, annual charts and monthly/average expense splits, drill-downs, filters, pagination, all-result totals |
| [Categories and labels](classification.md) | Core and import review implemented; browser verification blocked | Single category, optional labels, setup, management, bulk assignment, filters, import drafts and counted deletion; proposed insights |
| [Local labeling rules](labeling-rules.md) | Implemented; browser verification blocked | Deterministic rules, suggestions, conflicts, historical previews and target lifecycle |
| [Import history](import-history.md) | Implemented | Batch history and protected cross-period removal |
| [Entry export and reimport](data-portability.md) | CSV implemented; browser verification pending | Per-user CSV snapshots and native restore through import review; settings JSON proposed |
| [Monthly budgets](planning.md) | Implemented; browser verification blocked | Category targets, savings reservations, named expected income, receipt matching, copy and year context |

Implemented means the behavior exists in source; it does not imply completed
browser/visual verification. The [README](../../README.md#verification-status)
records the current shared verification status. Proposed/deferred specs describe
future behavior, not capabilities of the running app.

Richer insights remain proposed in the
[categories and labels spec](classification.md). Account balances, reconciliation, and historical balance reconstruction
are outside the core. Snapshot APIs/UI have been retired; legacy data remains stored.

## Keeping specs useful

- One authoritative home per rule. Link instead of copying it across documents.
- Each feature spec states its status, observable behavior, important boundaries,
  acceptance examples, test locations, and unresolved questions.
- Update the spec and relevant tests in the same change as behavior. Review old
  assertions and remove superseded decisions rather than appending corrections.
- UI layout and copy can evolve without rewriting financial behavior. Record a
  screen decision only when it changes a meaningful user interaction or promise.
- Split a document when independently changing features make it hard to navigate.
  Move content to named feature specs, preserving links; do not copy it.
- Add an architecture decision record only for a consequential tradeoff whose
  rationale would otherwise be lost. Do not create one for every library or edit.
- Keep source/test details in code and setup commands in the README/Makefile.
  Specs describe what should happen and why it matters.
