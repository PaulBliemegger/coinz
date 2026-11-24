# Dashboard, Insights and Entries

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

## Insights

`/insights` is available in main navigation. It shows the monthly recorded
spending/income/net summary and the full category spending breakdown, without
Home's six-category limit, latest entries or entry/import actions. The same month
picker, owned read snapshot and category drill-down rules apply. Home retains its
compact panel; Explore insights opens this page with Home's selected month.
Direct navigation defaults to the current calendar month. A valid `month=YYYY-MM`
URL parameter selects that month; malformed/out-of-range parameters fall back to
the current month. Loading/errors hide old figures; session expiry returns to login.
Two monthly trend charts show income versus expenses and expenses for a selected
category. Label analysis and automatic period-change comparisons remain proposed.

### Monthly trends

A shared selector chooses six (default) or twelve rolling calendar months ending
in the selected overview month. The window crosses years and includes months with
no entries as zero; at year 0001 it shortens rather than inventing earlier dates.
The category selector defaults to Uncategorized and includes owned archived and
unused categories. Selecting a category affects only the category series; the
income/expenses chart always covers all categories. Selections persist while
browsing overview months on this page.

Income/expenses use paired bars on a common scale. Category expenses use a single
bar per month. Scales are independent between the charts. Each has a named legend,
its upper scale amount, accessible entry links and an expandable exact-figures
table. The income/expenses table also shows recorded net. Narrow layouts scroll
the charts within their containers; all figures are available without hover.
All-zero series show explicit empty states and retain their exact-figures tables.

The browser's local calendar marks the current month In progress and later months
Future, in the chart and table. All recorded entries within each calendar month
are included, including future-dated entries. No extrapolation or automatic
comparison of partial and complete months is presented. Empty/past months do not
establish complete coverage. These are recorded trends, not forecasts or balances.

Income bars open that month's Income entries; expense bars open Expenses entries.
Category bars also apply that category or Uncategorized, including archives.
Both trend series are read from one owned backend snapshot. The selected-month
summary/breakdown uses its own dashboard request. Trend loading/errors hide old
charts/tables and offer Retry trends; superseded responses are ignored and session
expiry returns to login. Existing figures are never displayed under changed filters.

## Spending by category

Implemented; browser/visual verification status is recorded in the README.
Below the monthly totals, Home shows spending for that selected calendar month
as ranked horizontal bars. Each row exposes the category name, exact EUR amount,
and percentage of that month's recorded spending as text, without requiring hover.
The six largest buckets appear initially; Show all reveals the remainder. Changing
months resets the expanded view. Categories with no spending have no bar.

Spending means negative recorded entries, shown as positive magnitudes. Income
does not offset it. Uncategorized is a fallback bucket; archived categories
remain included and marked Archived. Label overlap never multiplies amounts.
All bucket amounts sum exactly to the monthly spending total from the same read
snapshot. Shares are rounded to one decimal using integer arithmetic and may sum
to slightly more or less than 100%. Bars use shares of the whole month, including
buckets hidden behind Show all. With no spending, show an explicit empty state.

Each bar opens Entries with the month's inclusive dates, that category (or
Uncategorized), and Expenses entry type selected. Selected archived categories
remain visible in the filter choices. A separate Uncategorized count/amount link
is available even when its bar is outside the first six. These are spending-entry
counts, not counts of all income and expenses. Entries' income total is zero for
this drill-down; users can change Entry type to All or Income explicitly.

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

Entry type defaults to All; Expenses selects negative entries and Income selects
positive entries. Its segmented control sits beside the compact Categories and
Labels pickers below period/search controls. It combines with period, search, categories and labels,
resets pagination on change, and scopes both the rows and all-result totals.
Initial URL parameters `category_ids`, `uncategorized`, and `entry_type` restore
the corresponding visible filters for chart drill-downs, alongside `start`/`end`.
The API value for Expenses remains `spending`. The
[classification spec](classification.md#filtering-and-totals) owns picker behavior
and active-filter chips.

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
  Optional `entry_type` accepts `all` (default), `spending`, or `income`.
- `GET /api/dashboard?month=YYYY-MM` returns `month`, `monthly`, `yearly`, twelve
  `months` with the same totals, `latest` (five), `latest_date`, and `latest_import`.
  `spending_categories` contains the selected month's nonempty buckets, descending
  by spending with case-insensitive name tie-breaking. Each has nullable `category`
  (the catalog item; null means Uncategorized), positive `spending_cents` string,
  `entry_count`, and one-decimal `share_percent` string.
- Reads are session-owned and use one SQLite snapshot per response. Financial
  aggregation remains exact checked integer arithmetic on the backend.
- `GET /api/insights/trends?month=YYYY-MM&months=6&category_id=…` returns
  `{ category, months }`. Months accepts 6 or 12; omission means 6. Omitted
  `category_id` selects Uncategorized; a supplied ID must be an owned category
  (including archived). Each chronological month has the usual totals plus
  `category_spending_cents` (a string) and `category_entry_count` for expenses only.
  No label join or financial floating-point aggregation is used.
- Legacy monthly `/transactions` and `/summary` reads remain available; the new
  screens use the explicit reporting endpoints.

Small synthetic regression fixture (`backend/tests/fixtures/import-regression.csv`):
16 records span October/November 2026. Across both months,
spending is €2,024.03; November is €1,063.79. A five-row API page must still report
16 matches and €2,024.03. November's inclusive range excludes October. Another
user's entries affect none of these figures. Invalid/reversed dates are rejected.

`backend/tests/ledger.rs` covers those cases, literal search, page separation,
dashboard year/month totals and the supported maximum year. The desktop/phone
journeys in `frontend/e2e/ledger.spec.ts` cover navigation, filters, pagination,
editing, selection reset, month-picker navigation/keyboard/boundaries and screenshots.
[Categories and labels](classification.md) defines implemented classification
filters/entry display and proposed comparison reports.

Category-report integration cases additionally cover month boundaries, exact sums,
income exclusion, two-user isolation, archived/unused categories, overlapping
labels, empty/income-only periods, drill-down totals across pages, rounded shares,
and cents above JavaScript integer precision. `frontend/tests/category-spending.test.ts`
covers drill-down dates including leap years, years 0001–0099 and 9999.
`frontend/e2e/insights.spec.ts` defines desktop/phone category expansion, archived
and Uncategorized drill-downs, visible filters, empty state, and screenshots.
It also defines Home-to-Insights month preservation, full category visibility and
navigation state. The expanded public [demo CSV](csv-import.md#acceptance-examples-and-evidence)
has independent import/rule/report/repeat/overlap acceptance coverage.
Trend integration cases cover rolling year boundaries, missing months, category
and income separation, label overlap, archives, ownership/authentication, invalid
filters, years 0001/9999, large cents and monthly overflow. Frontend trend-display
tests cover exact chart scaling, calendar markers and income drill-down boundaries.
The Insights browser journey additionally defines range/category selection,
read-error recovery, chart/table drill-down and responsive screenshots.
