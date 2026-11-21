# Planning direction

Status: agreed product direction, not implemented. Refine acceptance examples and
UI immediately before its implementation slice. No planning API/schema exists yet.

## Useful questions

- What monthly surplus/shortfall follows from my expected income and spending?
- What changes if I reduce a spending area or add a recurring commitment?
- What does a one-off purchase or holiday do to my expected savings over six months?

Planning does not require maintaining account balances. Keep actual recorded entries
separate from assumptions, planned commitments, and scenario results.

## First useful slice

Choose a monthly horizon, expected income, and expected spending. Support recurring
commitments and dated one-off expenses/income. Label budgets can provide targets;
selected historical periods may provide editable estimates once labels/data exist.
Make every assumption visible and editable. Do not silently turn a partial month
into a normal spending baseline.

Show expected income, spending, net change per month, and cumulative net change
from the scenario's beginning. Call the last measure projected savings/change,
not total money available or a bank balance. Targets and historical estimates are
alternative inputs where they cover the same spending, not additive expenses.

Avoid counting rent once in historical spending and again as a planned commitment.
A forecast allocation should make clear which expected spending a commitment
replaces, or keep a fully manual baseline until that behavior is defined. Planned
items that become actual transactions likewise need explicit matching semantics.

## Optional What-if balance calculator

Only add this when a concrete use case justifies it. The user supplies an available
amount and as-of date; the scenario adds expected subsequent movements. Keep the
scope/date/assumptions visible. Persisting bank accounts, snapshots, or reconciled
balances is not required. A positive projection is an estimate, not assurance that
money is available in a particular bank account at a particular time.

## Outside the core

Reconstructing historical balances from an incomplete transaction ledger, maintaining
net worth, and account reconciliation. Historical insights use period income,
spending, and net change directly. No game mechanics are required for planning.
