# Product scope

## Purpose

Help one person understand where their money goes, find spending patterns, and
make simple plans. Building and learning are part of the value. Keep the app
pleasant and small enough to maintain alone with agent assistance.

## Implementation status

The manual-ledger foundation is implemented; full verification remains pending.
The [manual-ledger spec](specs/manual-ledger.md) defines its current contract and
acceptance examples. Other features below are agreed direction, not implemented
behavior. See the [spec index](specs/README.md) for refinement and upkeep rules.

## Confirmed needs

- Initially used by one person, with logins and user ownership built in.
- EUR only; multiple personal accounts, initially N26 and Erste Bank.
- Responsive browser access on laptop and phone. Native clients are deferred.
- Import bank CSVs and allow simple manual income and expense entry.
- Historical Excel material contains both summaries and individual manual entries.
- Flexible labels, filters, summaries, and charts that reveal spending patterns.
- Explicit budgets and a simple future balance projection.
- Local development first; Docker Compose and later deployment on a NAS.
- Breaking schema changes and development data resets are acceptable before v1.0.
  Make intentional resets visible; accidental incorrect totals are still bugs.

## Product decisions

### Screens and iteration

The visual direction is Sunday Studio's warm palette with restrained dry humor.
Screen layout, navigation, chart selection, and metric prominence remain
provisional. Discover these through working slices and actual use rather than
specifying every screen before implementation. Do not build a configurable
dashboard or widget system to accommodate this uncertainty.

Begin with transaction entry/listing and a simple period spending summary. Keep
the visual presentation separate from the calculations so we can rearrange it
without changing financial behavior. Gamification can be added or removed later;
it must not be embedded in the transaction model or required to use the app.

### Recorded figures and data freshness

Imports happen intermittently. The app knows recorded finances, not the live bank
balance. Arithmetic over recorded entries must be exact even when the underlying
history is incomplete.

- Call a balance calculated from the opening balance and recorded entries a
  **recorded balance**, not a current or live bank balance. Do not make it the
  dominant overview metric by default; prominence remains a design choice.
- Show account-specific freshness where useful: last import time and latest
  recorded transaction date are different facts and must be labelled separately.
  Neither proves that all transactions through that date have been captured.
- Show an export's covered date range only when explicitly known. Do not infer
  complete coverage from an upload timestamp or the newest transaction date.
  A separate coverage-verification workflow is not needed for the first version.
- Spending and income summaries identify their period, account/filter scope, and
  that they describe recorded transactions. A partial or unverified period must
  not be presented as a complete month's finances.
- Keep recorded balances, any explicitly bank-confirmed snapshots added later,
  and future projections distinct. Do not invent missing transactions or quietly
  roll a stale balance forward using a trend estimate.

Overview ideas such as spending by label, comparison with a previous period,
largest merchants, and budget remaining are candidates to try, not a settled
dashboard specification. Choose charts using the questions that real data raises.

### Labels and spending

Start with multiple labels per transaction, with bulk editing and filtering added
with the label slice. Do not require a category tree. A primary category or label
group is an open decision to revisit using actual reporting examples.

Transactions matching several selected labels count once in a combined result.
Reports must make their any-label/all-label semantics visible. Separate label
totals can overlap and must not be presented as mutually exclusive slices of a
pie or added together as total spending.

Transfers between owned accounts are balance movements, not income or expenses.
Treat refunds as reductions in the associated spending where a relationship is
known; exact editing and cross-period presentation will be specified in that slice.
Show unlabelled spending rather than hiding it.

### Imports

Use an account-specific import preview before committing changes. Remember format
choices so routine subsequent imports are short. Add source adapters as concrete
export samples become available; do not assume a bank's CSV headers or ID fields.

Separate known imported identities from possible matches:

- Re-importing the same committed source rows must not create new transactions.
- Prefer a bank-provided stable ID, scoped to its account, where available.
- Retain file and row identity for repeat-file recognition; preserve two legitimate
  identical rows within one export as separate purchases.
- For overlapping exports without stable IDs, dates, amounts, and descriptions
  indicate candidates, not proof. Ask for review rather than silently dropping rows.
- Let the user keep valid identical purchases, including manual entries.
- Matching a manual entry to an imported entry must preserve labels and notes.
- Preview invalid rows explicitly. A failed commit must not leave a partial batch.
- Record enough provenance to explain why an item was skipped or matched.

Historical summaries are not ordinary transactions. Keep summary coverage separate
from detailed history; do not add a monthly summary to transactions it already
includes. Decide the smallest useful migration approach after inspecting a
sanitized example. A one-off conversion is acceptable; a general spreadsheet
import engine is not required.

### Budgets

Start with monthly limits over selected labels. Count each matching transaction
once within a budget; different budgets may overlap. Show used and remaining
amounts and link to matching transactions. Rollover, envelope allocation, and
complex budgeting rules are deferred.

### Planning

Keep planned entries separate from recorded transactions. Start with dated,
one-off and recurring income/expenses plus a starting balance and date. Show the
projection horizon, the date/source of its starting balance, and its assumptions.
A forecast anchored to incomplete or old recorded data must make that limitation
visible. A planned entry that becomes an actual transaction must not be counted
twice.

Historical trends and additional variable-spending estimates come later. Do not
deduct the entire budget allowance as well as the actual spending it includes.
Projected balance and remaining discretionary budget are distinct measures.

### Personality

Use Sunday Studio's warm colors and a calm, readable interface, with occasional
dry humor. Pocket Quest's levels and cosmetic rewards remain optional ideas for
a later slice, not requirements for the finance core. Do not implement an XP
system, rewards schema, or game engine during the foundation milestone.

Reward understanding and useful organization: reviewing a month, resolving import
questions, and labelling transactions. Avoid shame, punitive streaks, or rewards
based on the user's income or spending amount. Errors and important financial
messages stay clear. A simple off switch should remove game elements.

## Delivery sequence

Each milestone gets a small, executable set of acceptance examples before work.
The following is sequencing, not a promise to implement every item immediately.

1. **Foundation and manual ledger.** Login, user isolation, accounts, dated opening
   balances, manual income/expenses, transaction list, and correct recorded balances
   and monthly totals with clear period/freshness labels. Establish tests, checks,
   hooks, Docker, and Compose.
2. **First bank import.** One real CSV format, preview, repeat-import protection,
   candidate review, and manual/import matching. Add transfer handling as required
   to prevent misreporting imported account movements.
3. **Daily usefulness.** Second bank format, labels, bulk editing, filters, refund
   handling, and spending summaries/charts with transaction drill-down.
4. **Historical data.** Inspect and migrate useful Excel detail and summaries
   without overlapping totals. Schedule earlier if it blocks real use.
5. **Budgets and planning.** Monthly label budgets, planned entries, recurrence,
   and explainable projections.
6. **Personality and iteration.** Optional light gamification and improvements
   motivated by actual usage. Refine accessibility and phone workflows throughout.

## First milestone acceptance

See the [manual-ledger specification](specs/manual-ledger.md#acceptance-evidence)
for the concrete scenarios and test locations. Complete the browser and container
runtime verification before treating the milestone as finished.

## Deferred scope

Bank API synchronization, investments, multi-currency conversion, native mobile
apps, offline synchronization, household sharing, public registration, tax
accounting, plugins, AI classification, and deployment automation.

Basic CI may be added after remote setup. Pre-v1.0 compatibility is not a project
goal; production hardening and backup/restore validation are required before
depending on the future homelab installation.
