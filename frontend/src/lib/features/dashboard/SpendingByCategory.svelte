<script lang="ts">
  import { resolve } from '$app/paths';
  import type { CategorySpending } from '#lib/api.ts';
  import { money, monthLabel } from '#lib/money.ts';
  import { spendingEntriesPath } from './category-spending.ts';

  let {
    month,
    items,
    compact = true,
  }: { month: string; items: CategorySpending[]; compact?: boolean } = $props();
  let expanded = $state(false);
  let visible = $derived(!compact || expanded ? items : items.slice(0, 6));
  let uncategorized = $derived(items.find((item) => item.category === null));
</script>

<section class="ledger category-spending" aria-labelledby="category-spending-heading">
  <div class="section-heading">
    <div>
      <p class="eyebrow">WHERE IT WENT</p>
      <h2 id="category-spending-heading">Spending by category</h2>
    </div>
    <span class="muted">{monthLabel(month)}</span>
  </div>
  {#if items.length}
    <ol class="category-bars">
      {#each visible as item (item.category?.id ?? 'uncategorized')}
        <li>
          <a href={resolve(spendingEntriesPath(month, item.category?.id ?? null))}>
            <span class="category-bar-heading">
              <span class="category-bar-name">
                {item.category?.name ?? 'Uncategorized'}
                {#if item.category?.archived}<small class="muted">Archived</small>{/if}
              </span>
              <span class="category-bar-figures">
                <strong>{money(item.spending_cents)}</strong>
                <span class="muted">{item.share_percent}%</span>
              </span>
            </span>
            <span class="category-bar-track" aria-hidden="true">
              <span
                class="category-bar-fill"
                class:uncategorized={!item.category}
                style:width={`${item.share_percent}%`}
              ></span>
            </span>
            <span class="sr-only">
              View {item.entry_count} spending {item.entry_count === 1 ? 'entry' : 'entries'}.
            </span>
          </a>
        </li>
      {/each}
    </ol>
    <div class="category-spending-footer">
      {#if compact && items.length > 6}
        <button
          class="button quiet"
          aria-expanded={expanded}
          onclick={() => (expanded = !expanded)}
        >
          {expanded ? 'Show fewer categories' : `Show all ${items.length} categories`}
        </button>
      {/if}
      {#if uncategorized}
        <a class="uncategorized-review" href={resolve(spendingEntriesPath(month, null))}>
          {uncategorized.entry_count} uncategorized spending {uncategorized.entry_count === 1
            ? 'entry'
            : 'entries'} · {money(uncategorized.spending_cents)}
          <span aria-hidden="true"> →</span>
        </a>
      {/if}
    </div>
  {:else}
    <p class="category-spending-empty muted">No spending recorded for {monthLabel(month)} yet.</p>
  {/if}
</section>

<style>
  .category-spending {
    margin-top: 1.5rem;
  }
  .category-bars {
    list-style: none;
    padding: 0;
    margin: 1rem 0 0;
  }
  .category-bars a {
    display: block;
    padding: 0.75rem 0.25rem;
    border-radius: 0.35rem;
    color: var(--ink);
    text-decoration: none;
  }
  .category-bars a:hover {
    background: var(--soft);
  }
  .category-bar-heading {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.4rem;
  }
  .category-bar-name {
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .category-bar-name small {
    display: block;
    font-size: 0.875rem;
  }
  .category-bar-figures {
    display: flex;
    flex-wrap: wrap;
    justify-content: end;
    align-items: baseline;
    gap: 0.3rem 1rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .category-bar-figures strong {
    white-space: nowrap;
    font-weight: 500;
  }
  .category-bar-figures > span {
    font-size: 0.875rem;
    min-width: 4ch;
  }
  .category-bar-track {
    display: block;
    height: 0.4rem;
    border-radius: 1rem;
    background: var(--soft);
    overflow: hidden;
  }
  .category-bar-fill {
    display: block;
    height: 100%;
    min-width: 2px;
    border-radius: inherit;
    background: var(--accent);
  }
  .category-bar-fill.uncategorized {
    background: var(--muted);
  }
  .category-spending-footer {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem 1rem;
    align-items: center;
    margin-top: 0.5rem;
  }
  .uncategorized-review {
    display: inline-flex;
    align-items: center;
    min-height: 44px;
    font-size: 0.875rem;
  }
  .category-spending-empty {
    padding-top: 1rem;
  }
  @media (max-width: 620px) {
    .category-spending .section-heading {
      align-items: start;
    }
    .category-bar-heading {
      gap: 0.5rem;
    }
  }
</style>
