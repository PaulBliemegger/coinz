<script lang="ts">
  import { SvelteURLSearchParams } from 'svelte/reactivity';
  import { api, ApiError, type InsightTrends } from '#lib/api.ts';
  import type { ClassificationCatalog } from '../classification/types.ts';
  import TrendChart from './TrendChart.svelte';
  let {
    month,
    refresh,
    catalog,
    onunauthorized,
  }: {
    month: string;
    refresh: number;
    catalog: ClassificationCatalog;
    onunauthorized: () => void;
  } = $props();
  let range = $state('6');
  let categoryId = $state('');
  let data = $state<InsightTrends | null>(null);
  let loading = $state(true);
  let error = $state('');
  let retry = $state(0);
  $effect(() => {
    if (categoryId && !catalog.categories.some((c) => c.id === categoryId)) categoryId = '';
  });
  $effect(() => {
    const params = new SvelteURLSearchParams({ month, months: range });
    if (categoryId) params.set('category_id', categoryId);
    void refresh;
    void retry;
    let active = true;
    loading = true;
    error = '';
    api<InsightTrends>(`/insights/trends?${params}`)
      .then((value) => {
        if (active) data = value;
      })
      .catch((cause) => {
        if (!active) return;
        if (cause instanceof ApiError && cause.status === 401) onunauthorized();
        else error = cause instanceof Error ? cause.message : 'Could not load trends.';
      })
      .finally(() => {
        if (active) loading = false;
      });
    return () => {
      active = false;
    };
  });
</script>

<section class="insight-trends" aria-label="Monthly trends">
  <div class="trend-controls">
    <label>
      Trend range<select bind:value={range}>
        <option value="6">6 months</option><option value="12">12 months</option>
      </select>
    </label>
    <label>
      Trend category<select bind:value={categoryId}>
        <option value="">Uncategorized</option>
        {#each catalog.categories as category (category.id)}<option value={category.id}>
            {category.name}{category.archived ? ' · Archived' : ''}
          </option>{/each}
      </select>
    </label>
  </div>
  <p class="muted trend-scope">
    Rolling months ending in the selected month. In-progress and future months include all recorded
    entries, including future-dated records. Empty months may mean missing history; these are not
    forecasts.
  </p>
  {#if error}<p class="error" role="alert">
      {error} <button class="button" onclick={() => retry++}>Retry trends</button>
    </p>
  {:else if loading}<p role="status">Gathering monthly trends…</p>
  {:else if data}
    <TrendChart months={data.months} mode="income_expenses" />
    <TrendChart
      months={data.months}
      mode="category"
      categoryId={data.category?.id ?? null}
      categoryName={data.category?.name ?? 'Uncategorized'}
    />
  {/if}
</section>

<style>
  .insight-trends {
    margin-top: 1.5rem;
    min-width: 0;
  }
  .trend-controls {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    gap: 0.75rem;
  }
  .trend-controls label {
    flex: 0 1 18rem;
    min-width: 0;
    max-width: 100%;
  }
  .trend-scope {
    margin-top: 0.75rem;
    font-size: 0.875rem;
  }
</style>
