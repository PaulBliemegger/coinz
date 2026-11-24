<script lang="ts">
  import { SvelteURLSearchParams } from 'svelte/reactivity';
  import { api, ApiError } from '#lib/api.ts';
  import { localDate, monthLabel } from '#lib/money.ts';
  import ExpenseDonut from './ExpenseDonut.svelte';
  import type { ExpenseSplit } from './split-types.ts';
  let {
    month,
    refresh,
    colorKeys,
    onunauthorized,
  }: { month: string; refresh: number; colorKeys: string[]; onunauthorized: () => void } = $props();
  let selected = $state<string[] | null>(null);
  let data = $state<ExpenseSplit | null>(null);
  let loading = $state(true);
  let error = $state('');
  let retry = $state(0);
  $effect(() => {
    const params = new SvelteURLSearchParams({ month, as_of: localDate() });
    if (selected !== null) params.set('average_months', selected.join(','));
    void refresh;
    void retry;
    let active = true;
    loading = true;
    error = '';
    api<ExpenseSplit>(`/insights/split?${params}`)
      .then((value) => {
        if (active) data = value;
      })
      .catch((cause) => {
        if (active) {
          if (cause instanceof ApiError && cause.status === 401) onunauthorized();
          else error = cause instanceof Error ? cause.message : 'Could not load expense splits.';
        }
      })
      .finally(() => {
        if (active) loading = false;
      });
    return () => {
      active = false;
    };
  });
  function toggle(value: string) {
    const current = selected ?? data?.selected_months ?? [];
    selected = current.includes(value)
      ? current.filter((m) => m !== value)
      : [...current, value].sort();
  }
</script>

<section aria-label="Monthly expense splits">
  {#if error}<p role="alert" class="error">
      {error}<button class="button" onclick={() => retry++}>Retry expense splits</button>
    </p>
  {:else if loading}<p role="status">Gathering expense splits…</p>
  {:else if data}
    <div class="expense-splits">
      <ExpenseDonut
        {colorKeys}
        title={monthLabel(month)}
        slices={data.current}
        total={data.current_total_cents}
        months={[month]}
      />
      <ExpenseDonut
        {colorKeys}
        title="Average month"
        slices={data.average}
        total={data.average_total_cents}
        months={data.selected_months}
      />
    </div>
    <p class="muted">
      Average of {data.selected_months.length} selected completed months: {data.selected_months.join(
        ', ',
      ) || 'none'}. Initially includes completed months with recorded entries; this does not prove
      complete history. Deselect partial history. Amounts are rounded to cents.
    </p>
  {/if}
  {#if data}
    <details>
      <summary>Months included in the average</summary>
      <div class="average-months">
        {#each data.months as option (option.month)}<label class="setup-choice">
            <input
              type="checkbox"
              disabled={!option.eligible || loading}
              checked={(selected ?? data.selected_months).includes(option.month)}
              onchange={() => toggle(option.month)}
            />{monthLabel(option.month)} · {option.entry_count} entries{!option.eligible
              ? ' · Not completed'
              : ''}
          </label>{/each}
      </div>
    </details>
  {/if}
</section>

<style>
  section {
    margin: 1.5rem 0;
  }
  .expense-splits {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr));
    gap: 1.5rem;
  }
  .average-months {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, 220px), 1fr));
    gap: 0.25rem 1rem;
    margin: 0.75rem 0;
  }
</style>
