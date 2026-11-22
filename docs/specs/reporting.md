# Dashboard and Entries workspace

Status: implemented. Automated evidence is described below; browser verification
status is recorded in the README.

## Home dashboard

`/` opens a dashboard after login. A month picker defaults to the browser's current
calendar month. It shows recorded monthly spending, income, and net, plus spending
for that calendar year and a twelve-month spending chart. Chart links open the
corresponding inclusive date range in Entries. An expandable table exposes exact
monthly figures without depending on hover or chart height.

The month control is centered above the summary. Previous/next arrows move one
month, including across years. Clicking the month opens a compact month-only
picker with year arrows, a directly editable year, and twelve month buttons.
Browsing years does not change the report until a month is chosen. This month
returns to the browser's current calendar month. Supported years are 1–9999;
navigation stops at the boundaries. Invalid/empty years disable month selection.
Choosing a month, Close, or Escape closes the picker and returns focus to its
trigger. Clicking or tabbing outside dismisses it without moving focus back.
Opening focuses the selected month; arrow keys move between month buttons
(up/down by one three-column row), and Home/End reach January/December.
Native controls supply tab/activation behavior; no day-calendar dependency is needed.

Figures use ordinary recorded income/expenses, including future-dated records.
They are not account balances, coverage guarantees, budgets, or forecasts.
The latest entry date and latest recorded import time are separate facts. There
is no inferred last-import date for legacy imports lacking batch history.

Import CSV and Add transaction sit directly above Latest entries. That preview
shows the five newest entries across all dates, read-only; a View all link below
opens all history in Entries. Manual additions/imports refresh the current view
without switching its selected month or silently restricting it to a source.

## Entries

`/entries` is the separate workspace. Rounded radio pills select Current month,
Year, or Custom. Current month uses the browser's calendar month; Year defaults
to its current year and accepts years 0001–9999. Custom offers optional From and
Through dates, both inclusive; leaving both blank means all recorded history.
The default navigation entry is Current month. Dashboard View all uses Custom
with blank dates. No Source filter is displayed.

Submit Search to apply a case-insensitive description substring (up to 200
characters). Search is literal: `%` and `_` do not act as wildcard operators.
SQLite's built-in lowercasing handles ASCII case; full Unicode case folding is
not promised. Results sort by date, creation time, then ID, all descending.

Pages contain 25 entries. Totals/count describe **all matching entries**, not
just the current page. Changing filters/search resets pagination and selection;
page changes and successful reloads clear selection. Bulk selection/deletion
applies only to displayed entries. Direct field editing and the adjacent bin
confirmation follow the [manual ledger](manual-ledger.md).

Loading/error states hide old financial results so stale numbers cannot appear
under newly selected filters. Superseded responses are ignored. Empty filters
show a useful empty state. Page requests beyond the result set clamp to its last
page after deletions. No all-results bulk deletion is offered: use
[import history](import-history.md) for removing an import across pages/months.

## API and acceptance evidence

- `GET /api/entries?start=YYYY-MM-DD&end=YYYY-MM-DD&q=…&page=1&page_size=25`
  returns `{ entries, totals, page, page_size, pages }`. Dates/search are optional;
  omitted dates mean all history. Page sizes are 1–100. `totals` contains cents
  strings `income_cents`, `spending_cents`, `net_cents`, and `transaction_count`.
- `GET /api/dashboard?month=YYYY-MM` returns `month`, `monthly`, `yearly`, twelve
  `months` with the same totals, `latest` (five), `latest_date`, and `latest_import`.
- Reads are session-owned and use one SQLite snapshot per response. Financial
  aggregation remains exact checked integer arithmetic on the backend.
- Legacy monthly `/transactions` and `/summary` reads remain available; the new
  screens use the explicit reporting endpoints.

Synthetic demo: 16 records span October/November 2026. Across both months,
spending is €2,024.03; November is €1,063.79. A five-row API page must still report
16 matches and €2,024.03. November's inclusive range excludes October. Another
user's entries affect none of these figures. Invalid/reversed dates are rejected.

`backend/tests/ledger.rs` covers those cases, literal search, page separation,
dashboard year/month totals and the supported maximum year. The desktop/phone
journeys in `frontend/e2e/ledger.spec.ts` cover navigation, filters, pagination,
editing, selection reset, month-picker navigation/keyboard/boundaries and screenshots.
Labels and label reports remain deferred.
