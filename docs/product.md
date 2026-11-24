# Product scope

## Purpose

coinz helps one person understand spending and make plans. Its core questions are
“Where does my money go?”, “Which patterns can I change?”, and “What would this
choice cost over the coming months?” Bank apps remain the place to check actual
account balances. Building and learning are also part of the project's value.

## Current implementation

Local user creation, login/logout, isolated private data, manual income/expenses,
editing/deleting entries, a home dashboard with monthly/yearly spending and a
monthly category spending breakdown linking to Entries, an Insights page with
annual income/expense and category graphs, monthly/average expense splits, monthly
budgets with savings reservations and expected-income matching, a
paginated Entries workspace with period/search/category/label filters, categories
and coloured labels with setup, management, counted deletion and import-review
drafts, local labeling rules with previewed history application, exact money handling,
and reusable CSV import profiles with guided column mapping, editable review,
duplicate protection and batch history. Profiles can also be created/edited
directly in Imports without a file. A source
is optional provenance metadata (including for imports), not a bank ledger. A new user can record an entry
immediately without setup. The warm Sunday Studio palette
and restrained dry humor remain the visual direction.

The [design philosophy](design.md) guides the implemented visual hierarchy,
selected-row import review, direct editing, and contextual bulk actions. Detailed
behavior lives in the feature specs.

The account/snapshot interface and API are retired. Existing account identities
become transaction sources; old opening amounts and snapshots are preserved as
inactive legacy records. See the [manual-ledger spec](specs/manual-ledger.md).
Bank-specific import adapters,
and richer insights are not yet implemented. The
[example importer](specs/csv-import.md) provides a synthetic workflow for testing
the next ideas.

## Core experience

- **Transactions:** record or import income and expenses, label them, and filter
  by period and search (source filtering remains available in the legacy API). Routine manual entry should stay short.
- **Insights:** explore period income/spending/net, merchants, labels, recurring
  costs, and comparisons. Charts link to their underlying transactions.
- **Planning:** implemented monthly category budgets, savings reservations and
  named expected income with receipt matching. Recurring commitments and scenarios
  remain deferred. See [monthly budgets](specs/planning.md).

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

## Categories, labels, and insights

Each entry has one category bucket (Uncategorized allowed) plus optional labels.
Categories partition spending; labels express overlapping connections such as an
occasion or subscription. No mandatory category hierarchy. Sources stay optional
and secondary. Setup is skippable; recording never depends on organization.
[Categories and labels](specs/classification.md) owns management, assignment,
filtering, import-review drafts and overlap semantics.
[Local labeling rules](specs/labeling-rules.md) owns deterministic suggestions,
conflicts and explicit history application. Home's first category insight and
the Insights page are defined in [reporting](specs/reporting.md); automatic period comparisons,
label analysis and richer reports remain proposed.

Transfers between the owner's accounts should ultimately be excluded from income
and spending, while linked refunds reduce the relevant spending. Dedicated types
and matching rules for these are deferred; do not silently reinterpret today's
ordinary entries. Define these before importing data that would misstate reports.

## Imports and historical data

The current importer supports the [example CSV format](specs/csv-import.md) and
explicit layouts through reusable profiles.
[Reusable import profiles](specs/import-profiles.md) have an approved Guided
selection design for supporting different CSV layouts without a programmed
adapter for each one. The flow is implemented; browser verification remains blocked.
They map columns and parsing settings before the existing import overview; YAML
sharing and file-managed declarative setup are desired later work. N26 and Erste
remain initial target export streams,
with dedicated adapters only if configurable mappings cannot cover a demonstrated
need. Bank-specific adapters are not implemented. Preview and validate
rows before an atomic commit. Develop format support from wholly synthetic
examples matching the format, never by reading private bank exports.

Prefer stable bank IDs scoped to the source. Track file/row provenance for repeat
files. Identical purchases can be legitimate: ambiguous overlaps and manual/import
matches need review rather than silent deletion. Matching preserves user labels
and notes. Batch-removal ownership and edited-entry protection are defined in
[import history](specs/import-history.md). Changing a source does not prove two
purchases are the same.

The old Excel material includes both summaries and individual entries. Summary
coverage must remain separate so it cannot be added to detailed entries it already
includes. A one-off migration is acceptable; no general spreadsheet engine is needed.

Per-user entry CSV export and restoration through import review are implemented.
Versioned settings JSON is the next portability slice; full database recovery
remains a separate operational backup concern. See [entry export and reimport](specs/data-portability.md).

## Next slices

Current implemented behavior is summarized above and detailed in the
[specification index](specs/README.md). Work on one coherent slice at a time:

1. Extend [categories and labels](specs/classification.md) with useful insight
   reports in separate slices; local rules are implemented.
   [CSV import profiles](specs/import-profiles.md) now provide reusable layouts.
   N26/Erste remain target export streams; define transfer/refund treatment before
   supporting real bank formats.
2. Migrate useful historical Excel material without overlapping totals, when needed.
3. Implement monthly budgets/planning, commitments, and one-off scenarios. Consider
   a balance-based calculator only when it answers a concrete decision.
4. Iterate using feedback. Optional humor/rewards must not become finance-core data
   dependencies; reward reviewing/organizing, never income/spending amounts.

## Boundaries and maintenance

Initially one person, EUR, laptop/phone browser access, Docker Compose, later NAS.
Logins and user ownership remain required. Bank API sync, multi-currency, investments,
net worth, native apps, offline sync, household sharing, and public signup are
deferred. GitHub checks and multi-platform dev image delivery are defined; private
deployment remains an explicit owner action. See the README for workflow setup
and verification status. Pre-v1.0 compatibility is not required, but data must never
be reset implicitly. Keep private runtime data inaccessible to the development
agent; [privacy requirements](privacy.md) take precedence over convenient debugging.

[Feature specs](specs/README.md) own detailed behavior and acceptance evidence;
[architecture](architecture.md) owns technical boundaries. Update them together
with changes, and split independent features rather than accumulating history here.
