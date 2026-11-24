<script lang="ts">
  import { tick } from 'svelte';
  import type { ClassificationItem, ClassificationKind } from '../classification/types.ts';
  import { filterChoices } from './filter-choices.ts';

  let {
    kind,
    items,
    selectedIds,
    fallbackSelected,
    open,
    onopen,
    onclose,
    onselect,
    labelMatch = 'any',
    onmatchchange,
  }: {
    kind: ClassificationKind;
    items: ClassificationItem[];
    selectedIds: string[];
    fallbackSelected: boolean;
    open: boolean;
    onopen: () => void;
    onclose: () => void;
    onselect: (id: string | null) => void;
    labelMatch?: string;
    onmatchchange?: (value: string) => void;
  } = $props();
  const id = $props.id();
  let root: HTMLDivElement;
  let trigger: HTMLButtonElement;
  let searchInput: HTMLInputElement | undefined = $state();
  let search = $state('');
  let includeArchived = $state(false);
  let title = $derived(kind === 'category' ? 'Categories' : 'Labels');
  let fallback = $derived(kind === 'category' ? 'Uncategorized' : 'Unlabeled');
  let count = $derived(selectedIds.length + Number(fallbackSelected));
  let choices = $derived(filterChoices(items, selectedIds, search, includeArchived));

  function close(restoreFocus = false) {
    onclose();
    if (restoreFocus) trigger.focus();
  }
  async function toggle() {
    if (open) return close();
    search = '';
    onopen();
    await tick();
    if (open) searchInput?.focus();
  }
  async function select(value: string | null, input: HTMLInputElement) {
    onselect(value);
    await tick();
    // Keep keyboard focus when selected-first ordering moves a checkbox, or
    // when deselecting an archived choice hides it from the available list.
    if (open) (input.isConnected ? input : searchInput)?.focus();
  }
</script>

<svelte:window
  onpointerdown={(event) => {
    if (open && event.target instanceof Node && !root.contains(event.target)) close();
  }}
  onkeydown={(event) => {
    if (open && event.key === 'Escape') {
      event.preventDefault();
      close(true);
    }
  }}
/>

<div
  class="filter-picker"
  bind:this={root}
  onfocusout={(event) => {
    if (open && event.relatedTarget instanceof Node && !root.contains(event.relatedTarget)) close();
  }}
>
  <button
    class="button filter-trigger"
    data-filter-trigger={kind}
    bind:this={trigger}
    aria-expanded={open}
    aria-controls={`${id}-options`}
    onclick={toggle}
  >
    {title}{#if count}<span class="filter-count">{count}</span>{/if}
    <span aria-hidden="true">⌄</span>
  </button>
  {#if open}
    <section
      class="filter-popover"
      id={`${id}-options`}
      aria-label={`Filter ${title.toLowerCase()}`}
    >
      <div class="filter-picker-heading">
        <strong>{title}</strong>
        <button
          class="button quiet"
          aria-label={`Close ${title.toLowerCase()} filter`}
          onclick={() => close(true)}
        >
          Done
        </button>
      </div>
      <label>
        <span class="sr-only">Search {title.toLowerCase()}</span>
        <input
          type="search"
          bind:this={searchInput}
          bind:value={search}
          placeholder={`Search ${title.toLowerCase()}…`}
        />
      </label>
      {#if kind === 'label' && selectedIds.length >= 2}
        <fieldset class="filter-match">
          <legend>Match labels</legend>
          {#each [['any', 'Any'], ['all', 'All']] as [value, name] (value)}
            <label>
              <input
                type="radio"
                name={`${id}-match`}
                {value}
                checked={labelMatch === value}
                onchange={() => onmatchchange?.(value)}
              />{name}
            </label>
          {/each}
        </fieldset>
      {/if}
      <label class="setup-choice filter-fallback">
        <input
          type="checkbox"
          checked={fallbackSelected}
          onchange={(event) => select(null, event.currentTarget)}
        />{fallback}
      </label>
      <div class="filter-choice-list" role="group" aria-label={`${title} choices`}>
        {#each choices as item (item.id)}
          <label class="setup-choice filter-choice">
            <input
              type="checkbox"
              checked={selectedIds.includes(item.id)}
              onchange={(event) => select(item.id, event.currentTarget)}
            />
            <span
              class:label-chip={kind === 'label'}
              data-color={kind === 'label' ? item.color : undefined}
            >
              {item.name}
            </span>
            {#if item.archived}<span class="muted"> · Archived</span>{/if}
          </label>
        {:else}
          <p class="muted filter-no-results">
            {search.trim() ? 'No matches.' : `No ${title.toLowerCase()} available.`}
          </p>
        {/each}
      </div>
      <label class="setup-choice filter-archived">
        <input type="checkbox" bind:checked={includeArchived} />Include archived
      </label>
    </section>
  {/if}
</div>
