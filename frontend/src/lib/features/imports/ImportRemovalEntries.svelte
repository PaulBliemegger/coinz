<script lang="ts">
  import SelectionHighlight from '#lib/components/selection/SelectionHighlight.svelte';
  import RoundSelection from '#lib/components/selection/RoundSelection.svelte';
  import { rowSelection } from '#lib/components/selection/row-selection.ts';
  import type { ImportEntryPreview } from '#lib/api.ts';
  import { dateLabel, money } from '#lib/money.ts';
  let {
    entries,
    label,
    selected = [],
    disabled = false,
    onselect,
  }: {
    entries: ImportEntryPreview[];
    label: string;
    selected?: string[];
    disabled?: boolean;
    onselect?: (id: string, checked: boolean) => void;
  } = $props();
  let expanded = $state(false);
  let visible = $derived(expanded ? entries : entries.slice(0, 5));
</script>

<ul class="removal-entries" aria-label={label}>
  {#each visible as entry (entry.id)}
    <li
      class:selectable={!!onselect}
      class:selected={selected.includes(entry.id)}
      use:rowSelection={{
        selected: selected.includes(entry.id),
        disabled: disabled || !onselect,
        onselect: (checked) => onselect?.(entry.id, checked),
      }}
    >
      {#if onselect}
        <SelectionHighlight selected={selected.includes(entry.id)} />
        <RoundSelection
          label={`Include ${entry.description} in removal`}
          selected={selected.includes(entry.id)}
          {disabled}
          onselect={(checked) => onselect?.(entry.id, checked)}
        />
      {/if}
      <div>
        <strong>{entry.description}</strong><span class="entry-date">
          {dateLabel(entry.date)}{selected.includes(entry.id) ? ' · Included in removal' : ''}
        </span>
      </div>
      <span class="entry-amount" class:income={BigInt(entry.amount_cents) > 0n}>
        {money(entry.amount_cents, true)}
      </span>
    </li>
  {/each}
</ul>
{#if entries.length > 5}
  <button type="button" class="button quiet show-more" onclick={() => (expanded = !expanded)}>
    {expanded ? 'Show fewer entries' : `Show all ${entries.length} entries`}
  </button>
{/if}

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 1rem;
    align-items: center;
    padding: 0.65rem 0;
    border-bottom: 1px solid var(--line);
  }
  li:last-child {
    border-bottom: 0;
  }
  li.selectable {
    position: relative;
    isolation: isolate;
    grid-template-columns: 44px minmax(0, 1fr) auto;
    padding: 0.65rem 0.5rem;
    border: 0;
    border-radius: 0.65rem;
    cursor: pointer;
    margin: 0.35rem 0;
  }
  li.selectable > :global(:not(.selection-highlight)) {
    position: relative;
    z-index: 1;
  }
  strong {
    display: block;
    font-weight: 500;
    overflow-wrap: anywhere;
  }
  .entry-date {
    display: block;
    color: var(--muted);
    font-size: 0.875rem;
  }
  .entry-amount {
    font-variant-numeric: tabular-nums;
    text-align: right;
    overflow-wrap: anywhere;
  }
  .show-more {
    margin: 0.25rem 0;
  }
  @media (max-width: 420px) {
    li {
      gap: 0.5rem;
    }
    li.selectable .entry-amount {
      grid-column: 2;
      text-align: left;
    }
  }
</style>
