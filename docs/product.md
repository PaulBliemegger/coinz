# Product scope

## Purpose

coinz helps one person understand spending and make plans. Its core questions are
“Where does my money go?”, “Which patterns can I change?”, and “What would this
choice cost over the coming months?” Bank apps remain the place to check actual
account balances. Building and learning are also part of the project's value.

## Current implementation

Local user creation, login/logout, isolated private data, manual income/expenses,
editing/deleting entries, monthly summaries, month/source filters, exact money
handling, and example-format CSV preview/import with duplicate review. A source
is optional provenance metadata, not a bank ledger. A new user can record an entry
immediately without setup. The warm Sunday Studio palette
and restrained dry humor remain the visual direction.

The [design philosophy](design.md) guides the implemented visual hierarchy,
selected-row import review, direct editing, and contextual bulk actions. Detailed
behavior lives in the feature specs.

The account/snapshot interface and API are retired. Existing account identities
become transaction sources; old opening amounts and snapshots are preserved as
inactive legacy records. See the [manual-ledger spec](specs/manual-ledger.md).
Planning, bank-specific import adapters, labels, and richer insights are not yet
implemented. The [example importer](specs/csv-import.md) provides a synthetic
workflow for testing the next ideas.

## Core experience

- **Transactions:** record or import income and expenses, label them, and filter
  by period and source. Routine manual entry should stay short.
- **Insights:** explore period income/spending/net, merchants, labels, recurring
  costs, and comparisons. Charts link to their underlying transactions.
- **Planning:** monthly expected income/spending, budget targets, recurring
  commitments, and one-off choices. See the [planning direction](specs/planning.md).

These are product responsibilities, not a requirement to add empty navigation
pages. Evolve screens through working slices and actual use. No dashboard-builder
framework, native client, or game engine is needed.

## Sources, not balance-bearing accounts

Preserve where imported entries came from: for example, a distinct N26 or Erste
export stream. Sources help with filtering, duplicate matching, and eventual
transfer recognition. Two accounts at the same bank can have distinct sources.
Manual entries can omit a source; a source can be attached or removed later.

Do not require opening balances, complete historical account ledgers, snapshots,
or reconciliation to track spending. Neither creating a source nor recording an
entry should prompt for any of those. A source must not be inferred from the
merchant or spending label.

## Recorded figures

All sums describe recorded entries for an explicit period and filter scope, not
complete bank history. Date of last entry and date of last import are separate
facts; neither proves complete coverage. Net means recorded income minus spending,
not bank balance, net worth, or an amount guaranteed available to spend.

A balance-based What-if calculator is optional later work. It could take an
explicit available amount/date plus future assumptions; it is not a prerequisite
for planning. Historical reconstructed balances and ongoing reconciliation are
out of the core scope. Historical monthly income/spending/net need no balance anchor.

## Labels and insights

Start with multiple labels per transaction, not a mandatory category tree. Combined
any/all-label filters count a transaction once. Separate label totals may overlap;
do not add them as a spending total or render them as exclusive pie slices.
Choose additional views based on useful questions raised by actual use.

Transfers between the owner's accounts should ultimately be excluded from income
and spending, while linked refunds reduce the relevant spending. Dedicated types
and matching rules for these are deferred; do not silently reinterpret today's
ordinary entries. Define these before importing data that would misstate reports.

## Imports and historical data

The current importer supports the [example CSV format](specs/csv-import.md).
N26 and Erste are the first planned EUR bank adapters; bank-specific formats and
saved mappings remain deferred. Preview and validate rows before an atomic commit.
Implement adapters from wholly synthetic examples matching the format, never by
reading private bank exports.

Prefer stable bank IDs scoped to the source. Track file/row provenance for repeat
files. Identical purchases can be legitimate: ambiguous overlaps and manual/import
matches need review rather than silent deletion. Matching preserves user labels
and notes. Changing a source does not prove two purchases are the same.

The old Excel material includes both summaries and individual entries. Summary
coverage must remain separate so it cannot be added to detailed entries it already
includes. A one-off migration is acceptable; no general spreadsheet engine is needed.

## Delivery sequence

1. The spending-first manual workflow is implemented; browser/visual verification
   remains pending.
2. The [example CSV importer](specs/csv-import.md) is implemented with selected-row
   preview and duplicate review; browser/visual verification remains pending.
3. Add labels, bulk labeling, useful insight reports, and N26/Erste adapters.
   Define transfer/refund treatment before supporting real bank formats.
4. Migrate useful historical Excel material without overlapping totals, when needed.
5. Implement monthly budgets/planning, commitments, and one-off scenarios. Consider
   a balance-based calculator only when it answers a concrete decision.
6. Iterate using feedback. Optional humor/rewards must not become finance-core data
   dependencies; reward reviewing/organizing, never income/spending amounts.

## Boundaries and maintenance

Initially one person, EUR, laptop/phone browser access, Docker Compose, later NAS.
Logins and user ownership remain required. Bank API sync, multi-currency, investments,
net worth, native apps, offline sync, household sharing, public signup, and deployment
automation are deferred. Pre-v1.0 compatibility is not required, but data must never
be reset implicitly. Keep private runtime data inaccessible to the development
agent; [privacy requirements](privacy.md) take precedence over convenient debugging.

[Feature specs](specs/README.md) own detailed behavior and acceptance evidence;
[architecture](architecture.md) owns technical boundaries. Update them together
with changes, and split independent features rather than accumulating history here.
