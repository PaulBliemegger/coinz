<script lang="ts">
  import { resolve } from '$app/paths';
  import type { TrendMonth } from '#lib/api.ts';
  import { localDate, money, monthLabel } from '#lib/money.ts';
  import { monthEntriesPath } from '../dashboard/category-spending.ts';
  import { barPercent, monthStatus } from './trend-display.ts';

  let {
    months,
    mode,
    categoryId = null,
    categoryName = 'Uncategorized',
  }: {
    months: TrendMonth[];
    mode: 'income_expenses' | 'category';
    categoryId?: string | null;
    categoryName?: string;
  } = $props();
  const id = $props.id();
  const today = localDate();
  type Series = {
    key: 'income_cents' | 'spending_cents' | 'category_spending_cents';
    name: string;
    type: 'income' | 'spending';
  };
  let series: Series[] = $derived(
    mode === 'income_expenses'
      ? [
          { key: 'income_cents', name: 'Income', type: 'income' },
          { key: 'spending_cents', name: 'Expenses', type: 'spending' },
        ]
      : [{ key: 'category_spending_cents', name: 'Expenses', type: 'spending' }],
  );
  let title = $derived(
    mode === 'income_expenses' ? 'Income and expenses over time' : 'Category spending over time',
  );
  let maximum = $derived(
    months.reduce(
      (highest, point) =>
        series.reduce(
          (max, s) => (BigInt(point[s.key]) > max ? BigInt(point[s.key]) : max),
          highest,
        ),
      0n,
    ),
  );
  function path(month: string, type: 'income' | 'spending') {
    return monthEntriesPath(month, type, mode === 'category' ? categoryId : undefined);
  }
</script>

<section class="trend-chart" aria-labelledby={`${id}-title`}>
  <div class="section-heading">
    <div>
      <h2 id={`${id}-title`}>{title}</h2>
      {#if mode === 'category'}<p class="muted">{categoryName}</p>{/if}
    </div>
  </div>
  <div class="trend-legend">
    {#each series as s (s.key)}<span>
        <span class="legend-swatch" class:income={s.type === 'income'} aria-hidden="true"
        ></span>{s.name}
      </span>{/each}
    <span class="muted">Scale: €0–{money(maximum.toString())}</span>
  </div>
  {#if maximum > 0n}
    <div class="trend-scroll" role="region" aria-label={`${title} chart; scroll for more months`}>
      <div class="trend-columns" style:--month-count={months.length}>
        {#each months as point (point.month)}
          <div class="trend-month">
            <div class="trend-bars">
              {#each series as s (s.key)}
                <a
                  class="trend-bar-target"
                  href={resolve(path(point.month, s.type))}
                  aria-label={`${s.name}, ${mode === 'category' ? categoryName + ', ' : ''}${monthLabel(point.month)}: ${money(point[s.key])}${monthStatus(point.month, today) ? ', ' + monthStatus(point.month, today) : ''}. View entries.`}
                  title={`${s.name}: ${money(point[s.key])}`}
                >
                  <span
                    class="trend-bar"
                    class:income={s.type === 'income'}
                    class:nonzero={BigInt(point[s.key]) > 0n}
                    style:height={`${barPercent(point[s.key], maximum)}%`}
                    aria-hidden="true"
                  ></span>
                </a>
              {/each}
            </div>
            <span class="trend-month-name">{monthLabel(point.month)}</span>
            <span class="trend-month-status">{monthStatus(point.month, today)}</span>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <p class="muted trend-empty">
      {mode === 'category'
        ? 'No recorded expenses for this category in this range.'
        : 'No recorded income or expenses in this range.'}
    </p>
  {/if}
  <details class="chart-table">
    <summary>Exact figures: {mode === 'category' ? categoryName : 'income and expenses'}</summary>
    <div class="trend-table-scroll">
      <table>
        <caption class="sr-only">{title}{mode === 'category' ? ': ' + categoryName : ''}</caption>
        <thead>
          <tr>
            <th scope="col">Month</th>{#each series as s (s.key)}<th scope="col">
                {s.name}
              </th>{/each}{#if mode === 'income_expenses'}<th scope="col">
                Income minus expenses
              </th>{/if}
          </tr>
        </thead>
        <tbody>
          {#each months as point (point.month)}<tr>
              <th scope="row">
                {monthLabel(point.month)}{#if monthStatus(point.month, today)}<span class="muted">
                    · {monthStatus(point.month, today)}
                  </span>{/if}
              </th>
              {#each series as s (s.key)}<td>
                  <a href={resolve(path(point.month, s.type))}>{money(point[s.key])}</a>
                </td>{/each}
              {#if mode === 'income_expenses'}<td>{money(point.net_cents, true)}</td>{/if}
            </tr>{/each}
        </tbody>
      </table>
    </div>
  </details>
</section>

<style>
  .trend-chart {
    margin: 1.75rem 0;
    min-width: 0;
  }
  .trend-legend,
  .trend-legend > span {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.4rem 0.9rem;
    font-size: 0.875rem;
  }
  .legend-swatch {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 0.15rem;
    background: var(--accent);
  }
  .legend-swatch.income {
    background: var(--status-ink);
  }
  .trend-scroll {
    overflow-x: auto;
    padding: 0.5rem 0.2rem;
    margin-top: 1rem;
  }
  .trend-columns {
    display: grid;
    grid-template-columns: repeat(var(--month-count), minmax(0, 1fr));
    gap: 0.4rem;
    min-width: calc(var(--month-count) * 6rem);
  }
  .trend-month {
    text-align: center;
    min-width: 0;
  }
  .trend-bars {
    display: flex;
    height: 190px;
    justify-content: center;
    border-bottom: 1px solid var(--line);
    gap: 3px;
  }
  .trend-bar-target {
    display: flex;
    align-items: end;
    flex: 1;
    min-width: 44px;
    max-width: 48px;
    height: 100%;
    border-radius: 0.2rem;
  }
  .trend-bar-target:hover {
    background: var(--soft);
  }
  .trend-bar {
    display: block;
    width: 100%;
    background: var(--accent);
    border-radius: 0.2rem 0.2rem 0 0;
  }
  .trend-bar.nonzero {
    min-height: 2px;
  }
  .trend-bar.income {
    background: var(--status-ink);
  }
  .trend-month-name {
    display: block;
    font-size: 0.875rem;
    margin-top: 0.5rem;
  }
  .trend-month-status {
    display: block;
    color: var(--muted);
    min-height: 1.5rem;
    font-size: 0.875rem;
  }
  .trend-empty {
    padding: 1.25rem 0;
  }
  .trend-table-scroll {
    overflow-x: auto;
  }
  table {
    display: table;
    min-width: 28rem;
  }
  thead {
    display: table-header-group;
    position: static;
    width: auto;
    height: auto;
    clip: auto;
    overflow: visible;
  }
  tbody {
    display: table-row-group;
    width: auto;
  }
  tbody tr {
    display: table-row;
    padding: 0;
  }
  th,
  td {
    padding: 0.6rem 0.75rem;
  }
  th {
    text-align: left;
    font-weight: 500;
  }
  td {
    font-variant-numeric: tabular-nums;
  }
</style>
