<script lang="ts">
  import { tick } from 'svelte';
  import { api } from '#lib/api.ts';
  import LabelChip from './LabelChip.svelte';
  import type { ClassificationCatalog, ClassificationItem, ClassificationChange } from './types.ts';
  let {
    catalog,
    category,
    labels,
    disabled = false,
    compactUncategorized = false,
    name,
    idPrefix,
    onchange,
    oncatalogchanged,
    onbusy = () => {},
  }: {
    catalog: ClassificationCatalog;
    category: ClassificationItem | null;
    labels: ClassificationItem[];
    disabled?: boolean;
    compactUncategorized?: boolean;
    name: string;
    idPrefix: string;
    onchange: (change: ClassificationChange) => Promise<void>;
    oncatalogchanged: () => Promise<void>;
    onbusy?: (value: boolean) => void;
  } = $props();
  let mode = $state<'category' | 'quick' | 'labels' | null>(null);
  let search = $state('');
  let busy = $state(false);
  let error = $state('');
  let quickIds = $state<string[]>([]);
  let searchInput = $state<HTMLInputElement>();
  let categoryTrigger = $state<HTMLButtonElement>();
  let labelTrigger = $state<HTMLButtonElement>();
  const candidates = $derived(
    (mode === 'category' ? catalog.categories : catalog.labels)
      .filter(
        (item) =>
          !item.archived ||
          (mode === 'category' ? category?.id === item.id : labels.some((l) => l.id === item.id)),
      )
      .filter((item) => item.name.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase())),
  );
  const canCreate = $derived(
    Boolean(search.trim()) &&
      !(mode === 'category' ? catalog.categories : catalog.labels).some(
        (item) => item.name.toLocaleLowerCase() === search.trim().toLocaleLowerCase(),
      ) &&
      !(mode === 'category' && search.trim().toLocaleLowerCase() === 'uncategorized'),
  );
  function escapePicker(node: HTMLElement) {
    const handle = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && mode) {
        event.preventDefault();
        event.stopPropagation();
        if (!busy) close();
      }
    };
    node.addEventListener('keydown', handle);
    return { destroy: () => node.removeEventListener('keydown', handle) };
  }
  function close() {
    const target = mode === 'category' ? categoryTrigger : labelTrigger;
    mode = null;
    error = '';
    target?.focus();
  }
  async function open(next: 'category' | 'quick' | 'labels') {
    if (mode === next) {
      close();
      return;
    }
    mode = next;
    search = '';
    error = '';
    if (next === 'quick')
      quickIds = catalog.labels
        .filter((l) => !l.archived)
        .slice(0, 4)
        .map((l) => l.id);
    await tick();
    if (next !== 'quick') searchInput?.focus();
  }
  async function apply(change: ClassificationChange) {
    if (busy) return;
    const focused = document.activeElement;
    let saved = false;
    busy = true;
    onbusy(true);
    error = '';
    try {
      await onchange(change);
      saved = true;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Could not update classification.';
    } finally {
      busy = false;
      onbusy(false);
      await tick();
      if (saved && change.action === 'set_category') close();
      else if (focused instanceof HTMLElement && focused.isConnected) focused.focus();
      else labelTrigger?.focus();
    }
  }
  async function create(event: Event) {
    event.preventDefault();
    if (busy || !canCreate) return;
    busy = true;
    onbusy(true);
    error = '';
    let created = false;
    let assignedCategory = false;
    try {
      const kind = mode === 'category' ? 'category' : 'label';
      const result = await api<{ id: string }>('/classification', 'POST', {
        kind,
        name: search.trim(),
        color: 'sage',
      });
      created = true;
      search = '';
      await oncatalogchanged();
      await onchange(
        kind === 'category'
          ? { action: 'set_category', category_id: result.id }
          : { action: 'add_labels', label_ids: [result.id] },
      );
      assignedCategory = kind === 'category';
    } catch (cause) {
      const detail = cause instanceof Error ? cause.message : 'Please try again.';
      error = created
        ? `Created, but could not finish loading or assigning it. Refresh and select it from the list. ${detail}`
        : detail;
    } finally {
      busy = false;
      onbusy(false);
      await tick();
      if (assignedCategory) close();
      else searchInput?.focus();
    }
  }
</script>

<div class="classification-editor" data-entry-controls use:escapePicker>
  <div class="classification-line">
    <button
      type="button"
      class="category-trigger"
      class:empty-category={compactUncategorized && !category}
      title={compactUncategorized && !category ? 'Uncategorized · choose a category' : undefined}
      bind:this={categoryTrigger}
      id={`${idPrefix}-category`}
      aria-label={`Change category for ${name}: ${category?.name ?? 'Uncategorized'}`}
      aria-expanded={mode === 'category'}
      aria-controls={`${idPrefix}-picker`}
      disabled={busy || disabled}
      onclick={() => open('category')}
    >
      {#if compactUncategorized && !category}<span aria-hidden="true">—</span>
      {:else}
        <svg
          viewBox="0 0 24 24"
          width="16"
          height="16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          aria-hidden="true"
        >
          <path d="M3 7V5h6l2 2h10v13H3Z" />
        </svg>
        <span>{category?.name ?? 'Uncategorized'}</span><span
          class="category-chevron"
          aria-hidden="true"
        >
          ⌄
        </span>
      {/if}
    </button>
    <div class="entry-labels">
      {#each labels as item (item.id)}<LabelChip
          {item}
          disabled={busy || disabled}
          onremove={() => apply({ action: 'remove_labels', label_ids: [item.id] })}
        />{/each}
      <button
        type="button"
        class="label-plus"
        bind:this={labelTrigger}
        id={`${idPrefix}-labels`}
        aria-label={`Add labels to ${name}`}
        aria-expanded={mode === 'quick' || mode === 'labels'}
        aria-controls={`${idPrefix}-picker`}
        disabled={busy || disabled}
        onclick={() => open('quick')}
      >
        +
      </button>
    </div>
  </div>
  {#if mode}
    <section
      class="classification-picker"
      id={`${idPrefix}-picker`}
      aria-label={mode === 'category' ? 'Choose a category' : 'Choose labels'}
    >
      <div class="picker-heading">
        <span>
          {mode === 'category'
            ? 'Choose one'
            : mode === 'quick'
              ? 'Recently used'
              : 'Choose labels'}
        </span><button type="button" class="button quiet" disabled={busy} onclick={close}>
          Done
        </button>
      </div>
      {#if mode === 'quick'}
        <div class="quick-labels">
          {#each quickIds
            .map((id) => catalog.labels.find((l) => l.id === id))
            .filter((l): l is ClassificationItem => Boolean(l && !l.archived)) as item (item.id)}
            <button
              type="button"
              class="quick-label"
              data-color={item.color}
              disabled={busy || disabled}
              aria-pressed={labels.some((l) => l.id === item.id)}
              onclick={() =>
                apply({
                  action: labels.some((l) => l.id === item.id) ? 'remove_labels' : 'add_labels',
                  label_ids: [item.id],
                })}
            >
              {labels.some((l) => l.id === item.id) ? '✓' : '+'}
              {item.name}
            </button>
          {/each}
        </div>
        <button type="button" class="button quiet" disabled={busy} onclick={() => open('labels')}>
          All labels…
        </button>
      {:else}
        <div>
          <label class="sr-only" for={`${idPrefix}-search`}>
            Find or create {mode === 'category' ? 'a category' : 'a label'}
          </label>
          <input
            id={`${idPrefix}-search`}
            bind:this={searchInput}
            bind:value={search}
            onkeydown={(event) => {
              if (event.key === 'Enter') {
                event.preventDefault();
                if (canCreate) void create(event);
              }
            }}
            maxlength="60"
            disabled={busy}
            placeholder={mode === 'category' ? 'Find a category…' : 'Find a label…'}
          />
          <div class="classification-choices">
            {#if mode === 'category' && 'uncategorized'.includes(search.trim().toLocaleLowerCase())}
              <button
                type="button"
                class="classification-choice"
                disabled={busy}
                aria-pressed={!category}
                onclick={() => apply({ action: 'set_category', category_id: null })}
              >
                Uncategorized <span aria-hidden="true">{!category ? '✓' : ''}</span>
              </button>
            {/if}
            {#each candidates as item (item.id)}
              <button
                type="button"
                class="classification-choice"
                disabled={busy}
                aria-pressed={mode === 'category'
                  ? category?.id === item.id
                  : labels.some((l) => l.id === item.id)}
                onclick={() =>
                  apply(
                    mode === 'category'
                      ? { action: 'set_category', category_id: item.id }
                      : {
                          action: labels.some((l) => l.id === item.id)
                            ? 'remove_labels'
                            : 'add_labels',
                          label_ids: [item.id],
                        },
                  )}
              >
                <span>{item.name}{item.archived ? ' · Archived' : ''}</span><span
                  aria-hidden="true"
                >
                  {(
                    mode === 'category'
                      ? category?.id === item.id
                      : labels.some((l) => l.id === item.id)
                  )
                    ? '✓'
                    : ''}
                </span>
              </button>
            {/each}
          </div>
          {#if canCreate}<button type="button" class="button" disabled={busy} onclick={create}>
              Create “{search.trim()}”
            </button>{/if}
        </div>
      {/if}
    </section>
  {/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}
</div>
