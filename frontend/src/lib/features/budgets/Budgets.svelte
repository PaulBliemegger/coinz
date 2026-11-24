<script lang="ts">
  import { api, ApiError } from '#lib/api.ts';
  import { money, amountInput, localDate, monthLabel } from '#lib/money.ts';
  import { resolve } from '$app/paths';
  import MonthPicker from '../dashboard/MonthPicker.svelte';
  import { budgetEntriesPath } from './entry-link.ts';
  import type { BudgetPlan, IncomeDraft } from './types.ts';

  let { onunauthorized }: { onunauthorized: () => void } = $props();
  let month = $state(localDate().slice(0, 7));
  let asOf = $state(localDate());
  let plan = $state<BudgetPlan | null>(null);
  let savings = $state('0.00');
  let categories = $state<{ id: string; amount: string }[]>([]);
  let income = $state<IncomeDraft[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let dirty = $state(false);
  let error = $state('');
  let saveError = $state('');
  let notice = $state('');
  let retry = $state(0);
  let copyMonth = $state('');
  let copyNotice = $state('');
  let editing = $state(false);
  function message(cause: unknown) {
    if (cause instanceof ApiError && cause.status === 401) onunauthorized();
    return cause instanceof Error ? cause.message : 'Could not update the budget.';
  }
  function draft(next: BudgetPlan, copying = false) {
    savings = amountInput(next.savings_cents);
    categories = (
      copying ? next.categories.filter((c) => !c.category.archived) : next.categories
    ).map((c) => ({
      id: c.category.id,
      amount: c.budget_cents === null ? '' : amountInput(c.budget_cents),
    }));
    income = next.income.map((i) => ({
      key: crypto.randomUUID(),
      id: copying ? null : i.id,
      name: i.name,
      amount: amountInput(i.planned_cents),
      complete: copying ? false : i.complete,
      transaction_ids: copying ? [] : [...i.transaction_ids],
    }));
    dirty = copying;
  }
  function canLeave() {
    return !dirty || window.confirm('Discard the unsaved budget changes?');
  }
  $effect(() => {
    const selected = month;
    const through = asOf;
    void retry;
    let active = true;
    loading = true;
    error = '';
    saveError = '';
    api<BudgetPlan>(`/budgets/${selected}?as_of=${through}`)
      .then((next) => {
        if (active) {
          plan = next;
          draft(next);
        }
      })
      .catch((cause) => {
        if (active) error = message(cause);
      })
      .finally(() => {
        if (active) loading = false;
      });
    return () => {
      active = false;
    };
  });
  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (!plan || saving) return;
    saving = true;
    saveError = '';
    notice = '';
    try {
      await api(`/budgets/${month}`, 'PUT', {
        revision: plan.revision,
        savings: savings || '0',
        categories: categories
          .filter((c) => c.amount.trim() !== '')
          .map((c) => ({ category_id: c.id, amount: c.amount })),
        income: income.map(({ id, name, amount, complete, transaction_ids }) => ({
          id,
          name,
          amount,
          complete,
          transaction_ids,
        })),
      });
      dirty = false;
      notice = 'Budget saved.';
      copyNotice = '';
      retry++;
    } catch (cause) {
      saveError = message(cause);
    } finally {
      saving = false;
    }
  }
  async function copy() {
    if (!copyMonth || copyMonth === month || !canLeave()) return;
    saving = true;
    saveError = '';
    try {
      const source = await api<BudgetPlan>(`/budgets/${copyMonth}?as_of=${asOf}`);
      if (!source.revision) throw new Error('There is no saved plan in that month.');
      draft(source, true);
      copyNotice = `Copied ${monthLabel(copyMonth)} into this draft. Receipts and completion flags were cleared; archived categories were skipped. Save to apply.`;
    } catch (cause) {
      saveError = message(cause);
    } finally {
      saving = false;
    }
  }
  function receiptUsed(id: string, key: string) {
    return income.some((i) => i.key !== key && i.transaction_ids.includes(id));
  }
</script>

<fieldset class="budget-period" disabled={saving}>
  <MonthPicker
    value={month}
    onchange={(value) => {
      if (canLeave()) {
        month = value;
        notice = '';
        copyNotice = '';
      }
    }}
  />
  <label>
    Recorded through<input
      type="date"
      min="0001-01-01"
      max="9999-12-31"
      value={asOf}
      onchange={(event) => {
        if (event.currentTarget.value && canLeave()) asOf = event.currentTarget.value;
        else event.currentTarget.value = asOf;
      }}
    />
  </label>
</fieldset>
{#if notice}<p role="status" class="notice">{notice}</p>{/if}
{#if error}<p class="error" role="alert">
    {error} <button class="button" onclick={() => retry++}>Reload budget</button>
  </p>
{:else if loading}<p role="status">Gathering this month's plan…</p>
{:else if plan}
  <p class="muted">
    Monthly spending room from recorded entries through {asOf}, after reserving savings. These
    figures are not a bank balance. Account-to-account transfers must be kept out of income and
    expenses.
  </p>
  <div class="budget-cards">
    <div><span>From recorded income</span><strong>{money(plan.from_recorded_cents)}</strong></div>
    <div>
      <span>Including pending income</span><strong>{money(plan.including_pending_cents)}</strong>
    </div>
    <div>
      <span>Unallocated in saved plan</span><strong>
        {plan.income.length ? money(plan.unallocated_cents) : 'Expected income not set'}
      </strong>
    </div>
  </div>
  <p class="muted">
    Recorded income {money(plan.recorded_income_cents)} · Expenses {money(plan.spending_cents)} · Pending
    income {money(plan.pending_income_cents)} · Reserved savings {money(plan.savings_cents)}
  </p>
  <section aria-labelledby="budget-comparison-heading">
    <div class="section-heading">
      <h2 id="budget-comparison-heading">Saved plan vs actual</h2>
      <button class="button" onclick={() => (editing = !editing)}>
        {editing ? 'Close editor' : 'Edit monthly plan'}
      </button>
    </div>
    {#if dirty && !editing}<p class="notice" role="status">
        The editor has unsaved changes. This overview still shows the saved plan.
      </p>{/if}
    <p class="muted">
      Saved targets, recorded expenses through {asOf}, and remaining category budgets. Unset
      categories have no spending limit.
    </p>
    <div class="plan-comparison-header" aria-hidden="true">
      <span>Category</span><span>Planned</span><span>Actual expenses</span><span>Remaining</span>
    </div>
    {#each plan.categories.filter((row) => row.budget_cents !== null || row.spending_cents !== '0') as row (row.category.id)}
      <div class="plan-comparison-row">
        <strong>{row.category.name}{row.category.archived ? ' · Archived' : ''}</strong><span>
          <small>Planned</small>{row.budget_cents === null ? 'Not set' : money(row.budget_cents)}
        </span><span><small>Actual expenses</small>{money(row.spending_cents)}</span><span>
          <small>Remaining</small>{row.remaining_cents === null
            ? 'Not set'
            : money(row.remaining_cents)}
        </span>
      </div>
    {:else}<p>
        No category targets or category expenses yet. Edit the monthly plan to set your first
        budget.
      </p>{/each}
    <p class="muted">
      Expenses without a category target, including Uncategorized: {money(
        plan.unbudgeted_spending_cents,
      )}.
    </p>
  </section>
  <div hidden={!editing}>
    <form onsubmit={save}>
      <fieldset disabled={saving} class="budget-form">
        <div class="budget-toolbar">
          <label>
            Copy from month<input type="month" min="0001-01" max="9999-12" bind:value={copyMonth} />
          </label>
          <button
            type="button"
            class="button"
            disabled={!copyMonth || copyMonth === month}
            onclick={copy}
          >
            Copy plan
          </button>
          <button class="button primary" disabled={!dirty}>Save budget</button>
        </div>
        {#if dirty}<p role="status" class="notice">
            Unsaved draft. Figures reflect the saved plan; save to update them.
          </p>{/if}
        {#if copyNotice}<p role="status">{copyNotice}</p>{/if}
        {#if saveError}<p class="error" role="alert">{saveError}</p>
          <button
            type="button"
            class="button"
            onclick={() => {
              if (canLeave()) retry++;
            }}
          >
            Reload saved plan
          </button>{/if}
        <label class="savings-input">
          Reserved for savings (€)<input
            inputmode="decimal"
            bind:value={savings}
            oninput={() => (dirty = true)}
          />
        </label>
        <section aria-labelledby="budget-income-heading">
          <div class="section-heading">
            <h2 id="budget-income-heading">Expected income</h2>
            <button
              type="button"
              class="button"
              disabled={income.length >= 100}
              onclick={() => {
                income = [
                  ...income,
                  {
                    key: crypto.randomUUID(),
                    id: null,
                    name: '',
                    amount: '',
                    complete: false,
                    transaction_ids: [],
                  },
                ];
                dirty = true;
              }}
            >
              + Expected income
            </button>
          </div>
          {#each income as item (item.key)}
            {@const saved = plan.income.find((i) => i.id === item.id)}
            <div class="expected-income-item">
              <div class="budget-toolbar">
                <label>
                  Income name<input
                    required
                    maxlength="100"
                    bind:value={item.name}
                    oninput={() => (dirty = true)}
                  />
                </label>
                <label>
                  Expected amount (€)<input
                    required
                    inputmode="decimal"
                    bind:value={item.amount}
                    oninput={() => (dirty = true)}
                  />
                </label>
                <button
                  type="button"
                  class="button quiet"
                  aria-label={`Remove expected income ${item.name || 'new item'}`}
                  onclick={() => {
                    income = income.filter((i) => i.key !== item.key);
                    dirty = true;
                  }}
                >
                  Remove
                </button>
              </div>
              <p class="muted">
                Recorded {money(saved?.received_cents ?? '0')} · Still expected {money(
                  saved?.pending_cents ?? '0',
                )} (saved plan)
              </p>
              <label class="setup-choice">
                <input
                  type="checkbox"
                  bind:checked={item.complete}
                  onchange={() => (dirty = true)}
                />Nothing more expected for this item
              </label>
              <details>
                <summary>Match recorded income ({item.transaction_ids.length} linked)</summary>
                <div class="budget-receipts">
                  {#each plan.receipts as receipt (receipt.id)}<label class="setup-choice">
                      <input
                        type="checkbox"
                        value={receipt.id}
                        bind:group={item.transaction_ids}
                        disabled={receiptUsed(receipt.id, item.key)}
                        onchange={() => (dirty = true)}
                      />{receipt.date} · {receipt.description} · {money(receipt.amount_cents)}
                    </label>{:else}<p class="muted">
                      No recorded income in this month through the selected date.
                    </p>{/each}
                </div>
                {#if item.transaction_ids.some((id) => !plan?.receipts.some((r) => r.id === id))}<p
                    class="error"
                  >
                    Some links are no longer available as income in this month through this date.
                  </p>
                  <button
                    type="button"
                    class="button"
                    onclick={() => {
                      item.transaction_ids = item.transaction_ids.filter((id) =>
                        plan?.receipts.some((r) => r.id === id),
                      );
                      dirty = true;
                    }}
                  >
                    Remove unavailable links
                  </button>{/if}
              </details>
            </div>
          {:else}<p class="muted">
              Add salary, freelance work or other income you expect. Recorded income always counts
              once, even when it has no expected item.
            </p>{/each}
        </section>
        <section aria-labelledby="budget-categories-heading">
          <h2 id="budget-categories-heading">Category budgets</h2>
          <p class="muted">
            Blank means Not set; €0 is an explicit zero budget. Unbudgeted expenses still reduce
            monthly spending room: {money(plan.unbudgeted_spending_cents)} recorded.
          </p>
          <div class="budget-category-list">
            {#each plan.categories.filter((c) => !c.category.archived || c.budget_cents !== null || c.spending_cents !== '0') as row (row.category.id)}
              {@const entry = categories.find((c) => c.id === row.category.id)}
              {#if entry}<div class="budget-category-row">
                  <strong>{row.category.name}{row.category.archived ? ' · Archived' : ''}</strong>
                  <label>
                    Budget (€)<input
                      aria-label={`${row.category.name} budget (€)`}
                      inputmode="decimal"
                      placeholder="Not set"
                      bind:value={entry.amount}
                      oninput={() => (dirty = true)}
                    />
                  </label>
                  <span>
                    <small>Recorded expenses</small>
                    {#if budgetEntriesPath(month, row.category.id, asOf)}<a
                        href={resolve(budgetEntriesPath(month, row.category.id, asOf)!)}
                      >
                        {money(row.spending_cents)}
                      </a>{:else}{money(row.spending_cents)}{/if}
                  </span>
                  <span>
                    <small>Remaining budget</small>{row.remaining_cents === null
                      ? 'Not set'
                      : money(row.remaining_cents)}
                  </span>
                </div>{/if}
            {/each}
          </div>
        </section>
        <button class="button primary" disabled={!dirty}>Save budget</button>
      </fieldset>
    </form>
  </div>
  <details class="budget-year">
    <summary>{month.slice(0, 4)} overview through {monthLabel(month)}</summary>
    <p class="muted">
      Expenses include all recorded months from January through this month, through {asOf}. Budget
      comparison uses only months where that category had a target; Not set is excluded.
    </p>
    {#each plan.categories.filter((c) => c.year_spending_cents !== '0' || c.budgeted_months) as row (row.category.id)}<div
        class="budget-year-row"
      >
        <strong>{row.category.name}</strong><span>
          Year expenses: {money(row.year_spending_cents)}
        </span><span>
          {row.budgeted_months} budgeted months: {money(row.year_budget_cents)} planned / {money(
            row.year_budgeted_spending_cents,
          )} recorded
        </span>
      </div>{/each}
  </details>
{/if}

<style>
  .budget-period,
  .budget-form {
    border: 0;
    padding: 0;
    margin: 0;
    min-width: 0;
  }
  .plan-comparison-header,
  .plan-comparison-row {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--line);
  }
  .plan-comparison-header {
    color: var(--muted);
    font-size: 0.875rem;
  }
  .plan-comparison-row strong {
    overflow-wrap: anywhere;
  }
  .plan-comparison-row small {
    display: none;
    font-size: 0.875rem;
    color: var(--muted);
  }
  @media (max-width: 620px) {
    .plan-comparison-header {
      display: none;
    }
    .plan-comparison-row {
      grid-template-columns: 1fr 1fr;
    }
    .plan-comparison-row small {
      display: block;
    }
  }
  .budget-period > label,
  .savings-input {
    max-width: 20rem;
    margin: 1rem 0;
  }
  .budget-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 230px), 1fr));
    gap: 1rem;
    margin: 1.5rem 0;
  }
  .budget-cards > div {
    padding: 1rem;
    background: var(--soft);
    border-radius: 0.65rem;
  }
  .budget-cards span,
  .budget-cards strong {
    display: block;
  }
  .budget-cards strong {
    font-size: 1.7rem;
    font-weight: 500;
    overflow-wrap: anywhere;
  }
  .budget-toolbar {
    display: flex;
    align-items: end;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin: 1rem 0;
  }
  .budget-toolbar label {
    flex: 1 1 180px;
    min-width: 0;
    max-width: 25rem;
  }
  section {
    margin: 2rem 0;
  }
  .expected-income-item {
    border: 1px solid var(--line);
    border-radius: 0.6rem;
    padding: 1rem;
    margin: 0.75rem 0;
  }
  .budget-receipts {
    max-height: 16rem;
    overflow-y: auto;
  }
  .budget-category-row {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(7rem, 1fr) 1fr 1fr;
    align-items: center;
    gap: 1rem;
    padding: 0.85rem 0;
    border-bottom: 1px solid var(--line);
  }
  .budget-category-row small {
    display: block;
    font-size: 0.875rem;
    color: var(--muted);
  }
  .budget-category-row strong {
    overflow-wrap: anywhere;
  }
  .budget-year {
    margin-top: 2rem;
  }
  .budget-year-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    padding: 0.7rem 0;
    border-bottom: 1px solid var(--line);
  }
  @media (max-width: 620px) {
    .budget-category-row {
      grid-template-columns: 1fr 1fr;
      gap: 0.75rem;
    }
  }
</style>
