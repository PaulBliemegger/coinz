# Synthetic import workflow

All entries and merchant-style descriptions here are invented. Use these files
only in a synthetic development runtime. `demo.csv` has 106 entries across
September–November 2026; `demo-overlap.csv` repeats three November entries and
adds a single €16 cinema expense. Exact expected monthly totals live in the
[import spec](../../docs/specs/csv-import.md#acceptance-examples-and-evidence).

Use a fresh source for this version of the demo. Its `demo-v2-` IDs differ from
the old 16-entry example, so importing it alongside the old file does not replace
or deduplicate the earlier dataset. No existing user data is reset automatically.

## Try the full flow

1. In Organize, choose useful default categories and labels, or create your own.
   Create a `Salzburg weekend` label if you want to explore that occasion later.
2. In Organize → Rules, create any of these optional rules. Use **description
   contains** with the literal text below. Leave the source restriction unset.

   | Description contains | Category | Labels |
   | --- | --- | --- |
   | `Morgenrot coffee` | Eating out | Discretionary |
   | `Stadtmarkt Lebensmittel` | Groceries | Essential |
   | `Rent and service charges` | Housing | Essential |
   | `subscription` | Leisure | Subscription |
   | `Salzburg` | Vacation | Salzburg weekend |

3. Import `demo.csv`. Create a source and profile, mapping Date → `date`,
   Amount → `amount`, Description → `description`, and optional transaction ID →
   `external_id`. Default comma/ISO-date/decimal-point settings fit this file.
4. Review suggested classifications. Some entries deliberately remain
   Uncategorized so that per-row and bulk labeling can be exercised. Change an
   assignment manually to check that your choice survives rule suggestions.
5. Import, then open Insights and select September, October or November 2026.
   Drill into a category and check the selected dates and Expenses filter.
   September includes a small Salzburg trip. November includes freelance income.
6. In Entries, try the Subscription or Salzburg weekend label filter, matching
   any/all labels, and removing or clearing filters. Search for `Flat white`:
   each month has two legitimate identical purchases on the 7th, with distinct IDs.
7. Reimport the unchanged file into the same source: all 106 entries are known.
   Import `demo-overlap.csv`: three are known and only the new cinema entry is added.
8. Edit a description or label, then review import-history removal to check the
   edited-entry protection. Preview a rule against history before applying it.

The CSV has no category/label columns; those are assigned through rules or import
review. It contains ordinary income and expenses, not transfers, linked refunds,
opening balances or forecasts. Dates are fixed, including later-in-month dates;
recorded figures are not proof of complete coverage.
