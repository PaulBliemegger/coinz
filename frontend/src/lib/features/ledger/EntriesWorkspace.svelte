<script lang="ts">
  import { SvelteURLSearchParams } from 'svelte/reactivity';
  import { onMount, tick } from 'svelte';
  import { api, ApiError, type EntriesPage, type Source, type Transaction } from '#lib/api.ts';
  import { localDate, money } from '#lib/money.ts';
  import type { ClassificationCatalog } from '#lib/features/classification/types.ts';
  import EntriesList from './EntriesList.svelte';
  import EntryClassificationFilters from './EntryClassificationFilters.svelte';
  let {
    sources,
    catalog,
    oncatalogchanged,
    always,
    refresh,
    onchanged,
    oncreaterule,
    onunauthorized,
  }: {
    onunauthorized: () => void;
    oncreaterule: (entry: Transaction) => void;
    sources: Source[];
    catalog: ClassificationCatalog;
    oncatalogchanged: () => Promise<void>;
    always: boolean;
    refresh: number;
    onchanged: (notice: string) => Promise<void>;
  } = $props();
  let period = $state('month');
  let year = $state<number | undefined>(Number(localDate().slice(0, 4)));
  let start = $state('');
  let end = $state('');
  let search = $state('');
  let appliedSearch = $state('');
  let page = $state(1);
  let categoryIds = $state<string[]>([]);
  let labelIds = $state<string[]>([]);
  let uncategorized = $state(false);
  let unlabeled = $state(false);
  let labelMatch = $state('any');
  let entryType = $state('all');
  let loadedQuery = '';
  let data = $state<EntriesPage | null>(null);
  let loading = $state(false);
  let error = $state('');
  let retry = $state(0);
  let pendingFocus: string | undefined;
  async function changed(notice: string, focusId?: string) {
    pendingFocus = focusId;
    await onchanged(notice);
  }
  onMount(() => {
    const params = new SvelteURLSearchParams(window.location.search);
    categoryIds = params.get('category_ids')?.split(',').filter(Boolean) ?? [];
    uncategorized = params.get('uncategorized') === 'true';
    const requestedType = params.get('entry_type');
    if (requestedType === 'spending' || requestedType === 'income') entryType = requestedType;
    if (params.has('all') || params.has('start') || params.has('end')) {
      start = params.get('start') ?? '';
      end = params.get('end') ?? '';
      period = 'custom';
    }
  });
  let query = $derived.by(() => {
    const now = localDate();
    const params = new SvelteURLSearchParams({ page: String(page), page_size: '25' });
    if (period === 'month') {
      const [y, m] = now.split('-').map(Number);
      const last = new Date(y, m, 0).getDate();
      params.set('start', now.slice(0, 7) + '-01');
      params.set('end', now.slice(0, 7) + `-${last}`);
    } else if (period === 'year') {
      params.set('start', `${String(year ?? '').padStart(4, '0')}-01-01`);
      params.set('end', `${String(year ?? '').padStart(4, '0')}-12-31`);
    } else {
      if (start) params.set('start', start);
      if (end) params.set('end', end);
    }
    if (appliedSearch) params.set('q', appliedSearch);
    if (categoryIds.length) params.set('category_ids', categoryIds.join(','));
    if (labelIds.length) params.set('label_ids', labelIds.join(','));
    if (uncategorized) params.set('uncategorized', 'true');
    if (unlabeled) params.set('unlabeled', 'true');
    params.set('label_match', labelMatch);
    params.set('entry_type', entryType);
    return params.toString();
  });
  $effect(() => {
    const path = query;
    void refresh;
    void retry;
    let active = true;
    let previouslyFocused: Element | null = null;
    loading = path !== loadedQuery;
    error = '';
    api<EntriesPage>(`/entries?${path}`)
      .then((result) => {
        if (active) {
          previouslyFocused = document.activeElement;
          data = result;
          loadedQuery = path;
          page = result.page;
        }
      })
      .catch((cause) => {
        if (active) {
          if (cause instanceof ApiError && cause.status === 401) onunauthorized();
          else error = cause instanceof Error ? cause.message : 'Could not load entries.';
        }
      })
      .finally(async () => {
        if (active) {
          loading = false;
          await tick();
          if (active && pendingFocus) {
            (
              document.getElementById(pendingFocus) ?? document.getElementById('ledger-heading')
            )?.focus();
            pendingFocus = undefined;
          } else if (active && previouslyFocused && !previouslyFocused.isConnected) {
            document.getElementById('ledger-heading')?.focus();
          }
        }
      });
    return () => {
      active = false;
    };
  });
</script>

<h2 id="ledger-heading" tabindex="-1" class="sr-only">Entries workspace</h2>
<section class="workspace-filters" aria-label="Entry filters">
  <fieldset class="period-pills">
    <legend class="sr-only">Period</legend>
    {#each [['month', 'Current month'], ['year', 'Year'], ['custom', 'Custom']] as [value, label] (value)}
      <label class:chosen={period === value}>
        <input
          type="radio"
          name="period"
          {value}
          bind:group={period}
          onchange={() => (page = 1)}
        />{label}
      </label>
    {/each}
  </fieldset>
  {#if period === 'year'}<label>
      Year<input type="number" min="1" max="9999" bind:value={year} onchange={() => (page = 1)} />
    </label>{/if}
  {#if period === 'custom'}<label>
      From<input
        type="date"
        min="0001-01-01"
        max="9999-12-31"
        bind:value={start}
        onchange={() => (page = 1)}
      />
    </label><label>
      Through<input
        type="date"
        min="0001-01-01"
        max="9999-12-31"
        bind:value={end}
        onchange={() => (page = 1)}
      />
    </label>{/if}
  <form
    class="search-form"
    onsubmit={(event) => {
      event.preventDefault();
      appliedSearch = search.trim();
      page = 1;
    }}
  >
    <label>
      Search descriptions<input
        type="search"
        bind:value={search}
        maxlength="200"
        placeholder="Find that suspicious coffee…"
      />
    </label><button class="button">Search</button>
  </form>
</section>
<EntryClassificationFilters
  {catalog}
  bind:entryType
  bind:categoryIds
  bind:labelIds
  bind:uncategorized
  bind:unlabeled
  bind:labelMatch
  onchange={() => (page = 1)}
/>
<p class="muted">
  {period === 'custom' && !start && !end
    ? 'All recorded history'
    : period === 'month'
      ? 'Current calendar month'
      : period === 'year'
        ? `Calendar year ${year}`
        : `${start || 'Beginning'} – ${end || 'Latest'}`} · EUR · Totals cover every matching entry.
</p>
{#if error}<p class="error" role="alert">
    {error} <button class="button" onclick={() => retry++}>Retry</button>
  </p>
{:else if loading}<p role="status">Finding the paper trail…</p>
{:else if data}
  <div class="result-totals" aria-label="Filtered totals">
    <span>
      <small>Recorded spending</small><strong data-testid="spending">
        {money(data.totals.spending_cents)}
      </strong>
    </span><span>
      <small>Recorded income</small><strong data-testid="income">
        {money(data.totals.income_cents)}
      </strong>
    </span><span>
      <small>Income minus spending</small><strong data-testid="net">
        {money(data.totals.net_cents, true)}
      </strong>
    </span>
  </div>
  <section class="ledger" aria-label="Entries">
    <div class="section-heading">
      <h2>The entries</h2>
      <span class="muted">{data.totals.transaction_count} matching</span>
    </div>
    {#if data.entries.length}
      {#key query}<EntriesList
          entries={data.entries}
          {sources}
          {catalog}
          {oncreaterule}
          {refresh}
          {oncatalogchanged}
          {always}
          scope="this page"
          onchanged={changed}
        />{/key}
    {:else}<div class="empty-state">
        <h3>Nothing recorded for this period.</h3>
        <p>Your money is maintaining a low profile. Try a wider date range.</p>
      </div>{/if}
    <nav class="pagination" aria-label="Entries pages">
      <button class="button" disabled={page <= 1} onclick={() => page--}>Previous</button><span>
        Page {data.page} of {data.pages}
      </span><button class="button" disabled={page >= data.pages} onclick={() => page++}>
        Next
      </button>
    </nav>
  </section>
{/if}
