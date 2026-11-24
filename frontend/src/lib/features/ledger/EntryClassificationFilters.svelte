<script lang="ts">
  import { tick } from 'svelte';
  import type { ClassificationCatalog } from '../classification/types.ts';
  import ClassificationFilterPicker from './ClassificationFilterPicker.svelte';
  import './entry-filters.css';

  let {
    catalog,
    entryType = $bindable(),
    categoryIds = $bindable(),
    labelIds = $bindable(),
    uncategorized = $bindable(),
    unlabeled = $bindable(),
    labelMatch = $bindable(),
    onchange,
  }: {
    catalog: ClassificationCatalog;
    entryType: string;
    categoryIds: string[];
    labelIds: string[];
    uncategorized: boolean;
    unlabeled: boolean;
    labelMatch: string;
    onchange: () => void;
  } = $props();
  const id = $props.id();
  let root: HTMLElement;
  let open = $state<'category' | 'label' | null>(null);
  let selectedCategories = $derived(
    catalog.categories.filter((item) => categoryIds.includes(item.id)),
  );
  let selectedLabels = $derived(catalog.labels.filter((item) => labelIds.includes(item.id)));
  let active = $derived(
    entryType !== 'all' || categoryIds.length || labelIds.length || uncategorized || unlabeled,
  );

  function toggleCategory(value: string | null) {
    if (value === null) uncategorized = !uncategorized;
    else
      categoryIds = categoryIds.includes(value)
        ? categoryIds.filter((id) => id !== value)
        : [...categoryIds, value];
    onchange();
  }
  function toggleLabel(value: string | null) {
    if (value === null) {
      unlabeled = !unlabeled;
      labelIds = [];
    } else {
      unlabeled = false;
      labelIds = labelIds.includes(value)
        ? labelIds.filter((id) => id !== value)
        : [...labelIds, value];
    }
    onchange();
  }
  async function focusTrigger(kind: 'category' | 'label') {
    await tick();
    root.querySelector<HTMLButtonElement>(`[data-filter-trigger="${kind}"]`)?.focus();
  }
  async function removeType() {
    entryType = 'all';
    onchange();
    await tick();
    root.querySelector<HTMLInputElement>('input[value="all"]')?.focus();
  }
  function clear() {
    categoryIds = [];
    labelIds = [];
    uncategorized = false;
    unlabeled = false;
    labelMatch = 'any';
    open = null;
    void removeType();
  }
</script>

<section class="entry-classification-filters" aria-label="Classification filters" bind:this={root}>
  <div class="entry-filter-controls">
    <fieldset class="entry-type-filter">
      <legend class="sr-only">Entry type</legend>
      {#each [['all', 'All'], ['spending', 'Expenses'], ['income', 'Income']] as [value, name] (value)}
        <label class:chosen={entryType === value}>
          <input type="radio" name={`${id}-type`} {value} bind:group={entryType} {onchange} />{name}
        </label>
      {/each}
    </fieldset>
    <ClassificationFilterPicker
      kind="category"
      items={catalog.categories}
      selectedIds={categoryIds}
      fallbackSelected={uncategorized}
      open={open === 'category'}
      onopen={() => (open = 'category')}
      onclose={() => {
        if (open === 'category') open = null;
      }}
      onselect={toggleCategory}
    />
    <ClassificationFilterPicker
      kind="label"
      items={catalog.labels}
      selectedIds={labelIds}
      fallbackSelected={unlabeled}
      open={open === 'label'}
      onopen={() => (open = 'label')}
      onclose={() => {
        if (open === 'label') open = null;
      }}
      onselect={toggleLabel}
      {labelMatch}
      onmatchchange={(value) => {
        labelMatch = value;
        onchange();
      }}
    />
  </div>
  {#if active}
    <div class="active-entry-filters" role="group" aria-label="Active filters">
      {#if entryType !== 'all'}
        <button class="filter-chip" onclick={removeType} aria-label="Remove entry type filter">
          {entryType === 'spending' ? 'Expenses' : 'Income'} <span aria-hidden="true">×</span>
        </button>
      {/if}
      {#each selectedCategories as item (item.id)}
        <button
          class="filter-chip"
          onclick={() => {
            toggleCategory(item.id);
            void focusTrigger('category');
          }}
          aria-label={`Remove category filter ${item.name}`}
        >
          <svg
            aria-hidden="true"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
          >
            <path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v10H3Z" />
          </svg>{item.name}{item.archived ? ' · Archived' : ''}<span aria-hidden="true"> × </span>
        </button>
      {/each}
      {#if uncategorized}
        <button
          class="filter-chip"
          onclick={() => {
            toggleCategory(null);
            void focusTrigger('category');
          }}
          aria-label="Remove Uncategorized filter"
        >
          Uncategorized <span aria-hidden="true">×</span>
        </button>
      {/if}
      {#each selectedLabels as item (item.id)}
        <button
          class="filter-chip label-chip"
          data-color={item.color}
          onclick={() => {
            toggleLabel(item.id);
            void focusTrigger('label');
          }}
          aria-label={`Remove label filter ${item.name}`}
        >
          {item.name}{item.archived ? ' · Archived' : ''}<span aria-hidden="true">×</span>
        </button>
      {/each}
      {#if unlabeled}
        <button
          class="filter-chip"
          onclick={() => {
            toggleLabel(null);
            void focusTrigger('label');
          }}
          aria-label="Remove Unlabeled filter"
        >
          Unlabeled <span aria-hidden="true">×</span>
        </button>
      {/if}
      {#if labelIds.length >= 2}<span class="muted filter-match-note">
          Match {labelMatch} label{labelMatch === 'all' ? 's' : ''}
        </span>{/if}
      <button class="button quiet clear-entry-filters" onclick={clear}>Clear filters</button>
    </div>
  {/if}
</section>
