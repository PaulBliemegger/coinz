<script lang="ts">
  import { onMount } from 'svelte';
  import { api, ApiError, type DashboardData } from '#lib/api.ts';
  import { localDate } from '#lib/money.ts';
  import MonthPicker from '../dashboard/MonthPicker.svelte';
  import MonthlySummary from '../dashboard/MonthlySummary.svelte';
  import SpendingByCategory from '../dashboard/SpendingByCategory.svelte';
  import InsightTrends from './InsightTrends.svelte';
  import type { ClassificationCatalog } from '../classification/types.ts';

  let {
    refresh,
    catalog,
    onunauthorized,
  }: { refresh: number; catalog: ClassificationCatalog; onunauthorized: () => void } = $props();
  let month = $state(localDate().slice(0, 7));
  let data = $state<DashboardData | null>(null);
  let loading = $state(false);
  let error = $state('');
  let retry = $state(0);
  onMount(() => {
    const requested = new URLSearchParams(window.location.search).get('month');
    if (requested && /^(?!0000)\d{4}-(0[1-9]|1[0-2])$/.test(requested)) month = requested;
  });
  $effect(() => {
    const chosen = month;
    void refresh;
    void retry;
    let active = true;
    loading = true;
    error = '';
    api<DashboardData>(`/dashboard?month=${encodeURIComponent(chosen)}`)
      .then((result) => {
        if (active) data = result;
      })
      .catch((cause) => {
        if (!active) return;
        if (cause instanceof ApiError && cause.status === 401) onunauthorized();
        else error = cause instanceof Error ? cause.message : 'Could not load insights.';
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
{#if error}
  <p class="error" role="alert">
    {error} <button class="button" onclick={() => retry++}>Retry</button>
  </p>
{:else if loading}
  <p role="status">Gathering the details…</p>
{:else if data}
  <MonthlySummary month={data.month} totals={data.monthly} />
{/if}
<InsightTrends {month} {refresh} {catalog} {onunauthorized} />
{#if data && !loading && !error}
  <SpendingByCategory month={data.month} items={data.spending_categories} compact={false} />
  <p class="data-note">
    Recorded entries for the selected calendar month, including future-dated records. Missing
    entries can change the picture. Category amounts add up to recorded spending; percentage shares
    are rounded. Income is shown separately.
  </p>
{/if}
