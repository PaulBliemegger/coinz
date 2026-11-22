<script lang="ts">
  import { resolve } from '$app/paths';
  import { api, ApiError, type DashboardData } from '#lib/api.ts';
  import { localDate, money, monthLabel, dateLabel } from '#lib/money.ts';
  import MonthPicker from './MonthPicker.svelte';
  let {
    refresh,
    onadd,
    onimport,
    onunauthorized,
  }: { onunauthorized: () => void; refresh: number; onadd: () => void; onimport: () => void } =
    $props();
  let month = $state(localDate().slice(0, 7));
  let data = $state<DashboardData | null>(null);
  let loading = $state(false);
  let error = $state('');
  let retry = $state(0);
  let highest = $derived(
    data?.months.reduce(
      (max, m) => (BigInt(m.spending_cents) > max ? BigInt(m.spending_cents) : max),
      0n,
    ) ?? 0n,
  );
  function height(cents: string) {
    return highest === 0n ? 0 : Number((BigInt(cents) * 100n) / highest);
  }
  $effect(() => {
    const chosen = month;
    void refresh;
    void retry;
    let active = true;
    loading = true;
    error = '';
    api<DashboardData>(`/dashboard?month=${encodeURIComponent(chosen)}`)
      .then((value) => {
        if (active) data = value;
      })
      .catch((cause) => {
        if (active && cause instanceof ApiError && cause.status === 401) onunauthorized();
        else if (active)
          error = cause instanceof Error ? cause.message : 'Could not load the dashboard.';
      })
      .finally(() => {
        if (active) loading = false;
      });
    return () => {
      active = false;
    };
  });
</script>

<MonthPicker value={month} onchange={(value) => (month = value)} />
{#if error}<p class="error" role="alert">
    {error} <button class="button" onclick={() => retry++}>Retry</button>
  </p>
{:else if loading}<p role="status">Gathering the usual suspects…</p>
{:else if data}
  <section class="month-summary" aria-label="Monthly summary">
    <div class="spending-total">
      <p class="eyebrow">RECORDED SPENDING</p>
      <p class="hero-amount" data-testid="spending">{money(data.monthly.spending_cents)}</p>
      <p class="muted">{monthLabel(data.month)} · {data.monthly.transaction_count} entries</p>
    </div>
    <div class="secondary-total">
      <p>Recorded income</p>
      <strong data-testid="income">{money(data.monthly.income_cents)}</strong><span class="muted">
        Same month. Different direction.
      </span>
    </div>
    <div class="secondary-total">
      <p>Income minus spending</p>
      <strong data-testid="net">{money(data.monthly.net_cents, true)}</strong><span class="muted">
        Recorded net, not your bank balance
      </span>
    </div>
  </section>
  <section class="ledger annual" aria-labelledby="annual-heading">
    <div class="section-heading">
      <div>
        <p class="eyebrow">THE LONGER STORY</p>
        <h2 id="annual-heading">{data.month.slice(0, 4)} spending</h2>
      </div>
      <strong>{money(data.yearly.spending_cents)}</strong>
    </div>
    <div class="spending-chart" aria-label="Recorded spending by month">
      {#each data.months as item (item.month)}<a
          class="chart-month"
          href={resolve(
            `/entries?start=${item.month}-01&end=${item.month}-${new Date(Number(item.month.slice(0, 4)), Number(item.month.slice(5)), 0).getDate()}`,
          )}
          aria-label={`${monthLabel(item.month)}: ${money(item.spending_cents)}. View entries.`}
          title={`${monthLabel(item.month)}: ${money(item.spending_cents)}`}
        >
          <span class="bar-track">
            <span class="bar" style:height={`${height(item.spending_cents)}%`}></span>
          </span><span>{item.month.slice(5)}</span>
        </a>{/each}
    </div>
    <details class="chart-table">
      <summary>Monthly figures</summary>
      <table>
        <thead><tr><th>Month</th><th>Spending</th></tr></thead><tbody>
          {#each data.months as item (item.month)}<tr>
              <td>{monthLabel(item.month)}</td><td>{money(item.spending_cents)}</td>
            </tr>{/each}
        </tbody>
      </table>
    </details>
  </section>
  <p class="data-note">
    These figures reflect your records, not a live bank connection. Missing entries can change the
    picture.<br />Latest entry: {data.latest_date ? dateLabel(data.latest_date) : 'none yet'} · Latest
    import: {data.latest_import ? dateLabel(data.latest_import.slice(0, 10)) : 'none recorded'}
  </p>
  <div class="dashboard-actions">
    <button class="button" onclick={onimport}>Import CSV</button><button
      class="button primary"
      onclick={onadd}
    >
      + Add transaction
    </button>
  </div>
  <section class="ledger" aria-labelledby="latest-heading">
    <div class="section-heading">
      <h2 id="latest-heading">Latest entries</h2>
      <span class="muted">Across all dates</span>
    </div>
    {#if !data.latest.length}<div class="empty-state">
        <h3>A fresh page.</h3>
        <p>Add an entry or import a CSV. The plot starts there.</p>
      </div>
    {:else}<ul class="latest-entries">
        {#each data.latest as entry (entry.id)}<li>
            <div>
              <strong>{entry.description}</strong><span class="muted">{dateLabel(entry.date)}</span>
            </div>
            <strong class:income={BigInt(entry.amount_cents) > 0n}>
              {money(entry.amount_cents, true)}
            </strong>
          </li>{/each}
      </ul>{/if}
    <a class="view-all" href={resolve('/entries?all=1')}>
      View all <span aria-hidden="true">→</span>
    </a>
  </section>
{/if}
