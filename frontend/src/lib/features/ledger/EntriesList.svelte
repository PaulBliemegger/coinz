<script lang="ts">
  import { escapeToCancel } from './escape.ts';
  import { api, type Transaction, type Source } from '#lib/api.ts';
  import type {
    ClassificationCatalog,
    ClassificationChange,
  } from '#lib/features/classification/types.ts';
  import EntryRow from './EntryRow.svelte';
  let {
    entries,
    sources,
    catalog,
    refresh,
    oncatalogchanged,
    scope,
    always,
    onchanged,
    oncreaterule,
  }: {
    entries: Transaction[];
    oncreaterule: (entry: Transaction) => void;
    sources: Source[];
    catalog: ClassificationCatalog;
    refresh: number;
    oncatalogchanged: () => Promise<void>;
    scope: string;
    always: boolean;
    onchanged: (notice: string, focusId?: string) => Promise<void>;
  } = $props();
  let selected = $state<string[]>([]);
  let confirm = $state(false);
  let busy = $state(false);
  let error = $state('');
  let bin = $state<HTMLButtonElement>();
  let bulkAction = $state('');
  let bulkCategory = $state('');
  let bulkLabels = $state<string[]>([]);
  $effect(() => {
    void refresh;
    selected = [];
    confirm = false;
    bulkAction = '';
  });
  async function classify(event: SubmitEvent) {
    event.preventDefault();
    if (!selected.length || busy) return;
    const change: ClassificationChange =
      bulkAction === 'set_category'
        ? { action: 'set_category', category_id: bulkCategory || null }
        : {
            action: bulkAction === 'add_labels' ? 'add_labels' : 'remove_labels',
            label_ids: bulkLabels,
          };
    busy = true;
    error = '';
    try {
      await api('/transactions/classify', 'POST', { ids: selected, ...change });
      bulkAction = '';
      selected = [];
      await onchanged('Selected entries updated.');
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Could not update selected entries.';
    } finally {
      busy = false;
    }
  }
  function select(id: string, checked: boolean) {
    confirm = false;
    bulkAction = '';
    error = '';
    selected = checked
      ? [...new Set([...selected, id])].slice(0, 100)
      : selected.filter((item) => item !== id);
  }
  function cancel() {
    confirm = false;
    bin?.focus();
  }
  async function remove() {
    if (!confirm || !selected.length || busy) return;
    busy = true;
    error = '';
    try {
      await api('/transactions/bulk-delete', 'POST', { ids: selected });
      const count = selected.length;
      selected = [];
      confirm = false;
      await onchanged(`${count} ${count === 1 ? 'entry' : 'entries'} deleted.`);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Could not delete the selection.';
      confirm = false;
    } finally {
      busy = false;
    }
  }
</script>

<div class="entries-list" class:show-selection={always || selected.length > 0}>
  <div class="selection-line">
    <label class="select-all">
      <input
        type="checkbox"
        aria-label="Select displayed entries"
        disabled={busy}
        checked={entries.length > 0 &&
          entries.slice(0, 100).every((entry) => selected.includes(entry.id))}
        indeterminate={selected.length > 0 &&
          !entries.slice(0, 100).every((entry) => selected.includes(entry.id))}
        onchange={(event) => {
          confirm = false;
          error = '';
          selected = event.currentTarget.checked
            ? entries.slice(0, 100).map((entry) => entry.id)
            : [];
        }}
      />
      {entries.length > 100 ? `Select first 100 for ${scope}` : 'Select displayed entries'}
    </label>
    {#if selected.length}
      <section
        class="bulk"
        aria-label="Selected entries"
        use:escapeToCancel={() => {
          if (!busy) cancel();
        }}
      >
        <div class="bulk-heading">
          <span role="status">{selected.length} selected</span><button
            type="button"
            class="button quiet"
            disabled={busy}
            onclick={() => {
              selected = [];
              confirm = false;
              error = '';
            }}
          >
            Clear selection
          </button>
        </div>
        <div class="classification-actions">
          {#each [['set_category', 'Set category'], ['add_labels', 'Add labels'], ['remove_labels', 'Remove labels']] as [action, label] (action)}
            <button
              type="button"
              class="button quiet"
              disabled={busy}
              onclick={() => {
                bulkAction = bulkAction === action ? '' : action;
                bulkLabels = [];
                confirm = false;
              }}
            >
              {label}
            </button>
          {/each}
        </div>
        <div class="delete-actions">
          <button
            bind:this={bin}
            type="button"
            class="button bin"
            aria-label="Delete selected entries"
            aria-expanded={confirm}
            aria-controls="delete-confirmation"
            disabled={busy}
            onclick={() => {
              confirm = !confirm;
              error = '';
            }}
          >
            <svg
              viewBox="0 0 24 24"
              width="20"
              height="20"
              fill="none"
              stroke="currentColor"
              stroke-width="1.7"
              aria-hidden="true"
            >
              <path d="M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7" />
            </svg>
          </button>
          <div id="delete-confirmation" hidden={!confirm} class="confirmation">
            <button type="button" class="button destructive" disabled={busy} onclick={remove}>
              {busy
                ? 'Deleting…'
                : `Delete ${selected.length} ${selected.length === 1 ? 'entry' : 'entries'}`}
            </button><button type="button" class="button" disabled={busy} onclick={cancel}>
              Cancel
            </button>
            <span class="sr-only">
              {scope} · Deleted entries can be imported again.
            </span>
          </div>
        </div>
        {#if error}<p class="error" role="alert">{error}</p>{/if}
      </section>
    {/if}
  </div>
  {#if bulkAction && selected.length}
    <form class="classification-picker" onsubmit={classify}>
      {#if bulkAction === 'set_category'}<label>
          Category for {selected.length} selected entries<select
            bind:value={bulkCategory}
            disabled={busy}
          >
            <option value="">
              Uncategorized
            </option>{#each catalog.categories.filter((c) => !c.archived) as item (item.id)}<option
                value={item.id}
              >
                {item.name}
              </option>{/each}
          </select>
        </label>
      {:else}<fieldset>
          <legend>
            {bulkAction === 'add_labels' ? 'Add' : 'Remove'} labels on {selected.length} selected entries
          </legend>{#each catalog.labels.filter((l) => bulkAction === 'remove_labels' || !l.archived) as item (item.id)}<label
              class="setup-choice"
            >
              <input
                type="checkbox"
                value={item.id}
                bind:group={bulkLabels}
                disabled={busy}
              />{item.name}{item.archived ? ' · Archived' : ''}
            </label>{/each}
        </fieldset>{/if}
      <div class="classification-actions">
        <button
          class="button primary"
          disabled={busy || (bulkAction !== 'set_category' && !bulkLabels.length)}
        >
          Apply to {selected.length} entries
        </button><button
          type="button"
          class="button"
          disabled={busy}
          onclick={() => (bulkAction = '')}
        >
          Cancel
        </button>
      </div>
    </form>
  {/if}
  <ul aria-label="Recorded entries">
    {#each entries as entry (entry.id)}<EntryRow
        {entry}
        {sources}
        {catalog}
        {oncreaterule}
        {oncatalogchanged}
        selected={selected.includes(entry.id)}
        disabled={busy || (selected.length >= 100 && !selected.includes(entry.id))}
        onselect={(checked) => select(entry.id, checked)}
        {onchanged}
      />{/each}
  </ul>
</div>

<style>
  .selection-line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .bulk-heading span {
    font-size: 0.875rem;
  }
  label {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-height: 44px;
    flex-wrap: wrap;
  }
  input[type='checkbox'] {
    width: 19px;
    min-height: 19px;
    accent-color: var(--accent);
  }
  ul {
    display: grid;
    gap: 0.35rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .bulk {
    max-width: 100%;
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .bulk-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0;
  }
  .delete-actions {
    display: flex;
    flex-direction: row-reverse;
    align-items: center;
    gap: 0.4rem;
  }
  .bin {
    width: 44px;
    padding: 0.6rem;
  }
  .confirmation {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    justify-content: flex-end;
  }
  .confirmation[hidden] {
    display: none;
  }
  @media (max-width: 620px) {
    .confirmation {
      max-width: calc(100vw - 110px);
    }
    .delete-actions {
      align-items: flex-start;
    }
    .bulk {
      flex-basis: 100%;
      justify-content: flex-end;
    }
  }
</style>
