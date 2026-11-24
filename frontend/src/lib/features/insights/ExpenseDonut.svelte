<script lang="ts">
  import { resolve } from '$app/paths';
  import { money } from '#lib/money.ts';
  import { SvelteURLSearchParams } from 'svelte/reactivity';
  import type { ExpenseSlice } from './split-types.ts';
  import { monthEntriesPath } from '../dashboard/category-spending.ts';
  let {
    title,
    slices,
    total,
    months,
    colorKeys,
  }: {
    title: string;
    slices: ExpenseSlice[];
    total: string;
    months: string[];
    colorKeys: string[];
  } = $props();
  const id = $props.id();
  function color(key: string) {
    const index = Math.max(0, colorKeys.indexOf(key));
    return `hsl(${(index * 137.5 + 25) % 360} 30% 48%)`;
  }
  let segments = $derived.by(() => {
    let offset = 0;
    return slices.map((slice) => {
      const percent =
        BigInt(total) === 0n
          ? 0
          : Number((BigInt(slice.amount_cents) * 1000000n) / BigInt(total)) / 10000;
      const segment = {
        ...slice,
        percent,
        offset,
        color: color(slice.category?.id ?? 'uncategorized'),
      };
      offset += percent;
      return segment;
    });
  });
  function path(categoryId: string | null): `/entries?${string}` {
    const ordered = [...months].sort();
    const first = ordered[0];
    const last = ordered.at(-1);
    if (!first || !last) return '/entries?entry_type=spending';
    const params = new SvelteURLSearchParams(
      monthEntriesPath(first, 'spending', categoryId).split('?')[1],
    );
    params.set(
      'end',
      new SvelteURLSearchParams(monthEntriesPath(last, 'spending').split('?')[1]).get('end')!,
    );
    params.set('months', ordered.join(','));
    return `/entries?${params}`;
  }
</script>

<section aria-labelledby={`${id}-heading`}>
  <h3 id={`${id}-heading`}>{title}</h3>
  {#if BigInt(total) > 0n}
    <div class="expense-donut">
      <svg viewBox="0 0 42 42" aria-label={`${title} expense distribution`}>
        <title>{title} expense split</title>
        {#each segments as slice (slice.category?.id ?? 'uncategorized')}
          <a
            href={resolve(path(slice.category?.id ?? null))}
            aria-label={`${slice.category?.name ?? 'Uncategorized'}: ${money(slice.amount_cents)}. View entries.`}
          >
            <circle
              cx="21"
              cy="21"
              r="15.915494309"
              fill="none"
              stroke={slice.color}
              stroke-width="8"
              stroke-dasharray={`${slice.percent} ${100 - slice.percent}`}
              stroke-dashoffset={-slice.offset}
              transform="rotate(-90 21 21)"
            />
          </a>
        {/each}
      </svg>
      <strong class="donut-total">{money(total)}<small>expenses</small></strong>
    </div>
    <ul class="donut-legend">
      {#each segments as slice (slice.category?.id ?? 'uncategorized')}<li>
          <a href={resolve(path(slice.category?.id ?? null))}>
            <span class="swatch" style:background={slice.color} aria-hidden="true"></span><span>
              {slice.category?.name ?? 'Uncategorized'}{slice.category?.archived
                ? ' · Archived'
                : ''}
            </span><strong>{money(slice.amount_cents)}</strong><span class="muted">
              {slice.percent.toFixed(1)}%
            </span>
          </a>
        </li>{/each}
    </ul>
  {:else}<p class="muted">
      {months.length
        ? 'No expenses in the selected months.'
        : 'Select completed months for an average.'}
    </p>{/if}
</section>

<style>
  section {
    min-width: 0;
  }
  .expense-donut {
    position: relative;
    width: 240px;
    max-width: 100%;
    margin: 1rem auto;
  }
  svg {
    display: block;
    width: 100%;
  }
  svg a:focus-visible circle {
    stroke-width: 10;
  }
  .donut-total {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    pointer-events: none;
    padding: 0 2.8rem;
    text-align: center;
    overflow-wrap: anywhere;
    font-size: 1.25rem;
    font-weight: 500;
  }
  .donut-total small {
    font-size: 0.875rem;
    color: var(--muted);
    font-weight: 400;
  }
  .donut-legend {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .donut-legend a {
    display: grid;
    grid-template-columns: 0.75rem minmax(0, 1fr) auto auto;
    gap: 0.5rem;
    align-items: center;
    min-height: 44px;
    color: var(--ink);
    text-decoration: none;
    font-size: 0.875rem;
  }
  .donut-legend a:hover {
    background: var(--soft);
  }
  .donut-legend strong {
    font-weight: 500;
  }
  .donut-legend a > span:nth-child(2) {
    overflow-wrap: anywhere;
  }
  .swatch {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 50%;
  }
</style>
