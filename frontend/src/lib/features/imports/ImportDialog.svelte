<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { api, type Source, type ImportRow, type ImportOutcome } from '#lib/api.ts';
  import { money } from '#lib/money.ts';
  import ImportReviewRow from './ImportReviewRow.svelte';
  import { importSelection } from './selection.ts';

  let {
    sources,
    initialSource,
    onclose,
    onimported,
  }: {
    sources: Source[];
    initialSource: string;
    onclose: () => void;
    onimported: (result: ImportOutcome, source: string) => Promise<void>;
  } = $props();
  let dialog: HTMLDialogElement;
  let previewHeading = $state<HTMLHeadingElement>();
  let choices = $state<Source[]>([]);
  let sourceId = $state('');
  let sourceName = $state('');
  let newSource = $state(false);
  let file = $state<File | null>(null);
  let csv = $state('');
  let rows = $state<ImportRow[] | null>(null);
  let decisions = $state<Record<number, string>>({});
  let busy = $state(false);
  let error = $state('');
  let selected = $state<number[]>([]);
  const selection = $derived(importSelection(rows ?? [], selected, decisions));
  const attention = $derived(
    (rows ?? []).filter(
      (row) =>
        ['invalid', 'conflict'].includes(row.status) ||
        (row.status === 'review' && !decisions[row.record]),
    ),
  );
  const readyRows = $derived(
    (rows ?? []).filter(
      (row) => row.status === 'new' || (row.status === 'review' && !!decisions[row.record]),
    ),
  );
  const knownRows = $derived((rows ?? []).filter((row) => row.status === 'known'));
  const eligible = $derived(
    readyRows.filter((row) => row.status === 'new' || decisions[row.record] === 'keep'),
  );
  function select(record: number, checked: boolean) {
    selected = checked
      ? [...new Set([...selected, record])]
      : selected.filter((id) => id !== record);
  }
  function decide(record: number, choice: string) {
    decisions = { ...decisions, [record]: choice };
    select(record, choice === 'keep');
  }
  onMount(() => {
    choices = [...sources];
    sourceId = initialSource;
    dialog.showModal();
  });

  function fail(cause: unknown) {
    error = cause instanceof Error ? cause.message : 'Could not import this file.';
  }

  async function createSource(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    error = '';
    try {
      const result = await api<{ id: string }>('/sources', 'POST', { name: sourceName });
      choices = [
        ...choices,
        { id: result.id, name: sourceName.trim(), latest_transaction_date: null },
      ];
      sourceId = result.id;
      newSource = false;
      sourceName = '';
    } catch (cause) {
      fail(cause);
    } finally {
      busy = false;
    }
  }

  async function preview(event: SubmitEvent) {
    event.preventDefault();
    if (!file) return;
    error = '';
    busy = true;
    try {
      if (file.size > 256 * 1024) throw new Error('CSV files must be at most 256 KiB.');
      csv = new TextDecoder('utf-8', { fatal: true }).decode(await file.arrayBuffer());
      const result = await api<{ rows: ImportRow[] }>('/imports/preview', 'POST', {
        source_id: sourceId,
        csv,
      });
      rows = result.rows;
      decisions = {};
      selected = result.rows.filter((row) => row.status === 'new').map((row) => row.record);
      await tick();
      previewHeading?.focus();
    } catch (cause) {
      fail(cause);
    } finally {
      busy = false;
    }
  }

  async function commit() {
    busy = true;
    error = '';
    try {
      const result = await api<ImportOutcome>('/imports/commit', 'POST', {
        source_id: sourceId,
        csv,
        filename: file?.name,
        selected_records: selection.records,
        decisions: selection.decisions,
      });
      // Import succeeded. Refresh errors must never invite a duplicate submission.
      dialog.close();
      await onimported(result, sourceId);
    } catch (cause) {
      fail(cause);
    } finally {
      busy = false;
    }
  }
</script>

<dialog
  bind:this={dialog}
  aria-labelledby="import-title"
  class="import-dialog"
  oncancel={(event) => {
    if (busy) event.preventDefault();
  }}
  {onclose}
>
  <div class="dialog-heading">
    <div>
      <p class="eyebrow">A LITTLE LESS TYPING</p>
      <h2 id="import-title">Import CSV</h2>
    </div>
    <button
      type="button"
      class="button quiet"
      disabled={busy}
      onclick={() => dialog.close()}
      aria-label="Close import"
    >
      ✕
    </button>
  </div>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if !rows}
    <p>Start with the example format. Your bank's CSV will need its own adapter.</p>
    <p class="muted">
      Headers: <code>date,description,amount,external_id</code>. Dates: YYYY-MM-DD; EUR amounts:
      -12.34 for spending, 12.34 for income. External IDs may be blank.
    </p>
    {#if newSource}
      <form onsubmit={createSource}>
        <label>
          New source name<input bind:value={sourceName} required maxlength="80" disabled={busy} />
        </label>
        <div class="import-actions">
          <button class="button" disabled={busy}>Create import source</button><button
            type="button"
            class="button quiet"
            disabled={busy}
            onclick={() => (newSource = false)}
          >
            Cancel source
          </button>
        </div>
      </form>
    {:else}
      <form onsubmit={preview}>
        <label>
          Import source<select bind:value={sourceId} required disabled={busy}>
            <option value="">Choose a source</option>{#each choices as source (source.id)}<option
                value={source.id}
              >
                {source.name}
              </option>{/each}
          </select>
        </label>
        <button
          type="button"
          class="button quiet"
          disabled={busy}
          onclick={() => (newSource = true)}
        >
          + New source
        </button>
        <label>
          CSV file<input
            type="file"
            accept=".csv,text/csv"
            required
            disabled={busy}
            onchange={(event) => {
              file = event.currentTarget.files?.[0] ?? null;
              error = '';
            }}
          />
        </label>
        <p class="muted">
          Up to 256 KiB and 1,000 records. Preview first; nothing is added until you confirm.
          Sources hold no bank balance.
        </p>
        <button class="button primary" disabled={busy || !sourceId || !file}>
          {busy ? 'Reading…' : 'Preview CSV'}
        </button>
      </form>
    {/if}
  {:else}
    <div class="review-scroll">
      <h3 bind:this={previewHeading} tabindex="-1">Review {rows.length} entries</h3>
      <p class="muted">{file?.name} · {choices.find((source) => source.id === sourceId)?.name}</p>
      {#if attention.length}<h3 class="group-heading">Needs attention · {attention.length}</h3>
        {#each attention as row (row.record)}<ImportReviewRow
            {row}
            selected={false}
            choice={decisions[row.record] ?? ''}
            {busy}
            onselect={(checked) => select(row.record, checked)}
            onchoice={(choice) => decide(row.record, choice)}
          />{/each}
      {/if}
      <div class="group-heading select-heading">
        <h3>Ready to add</h3>
        <label>
          <input
            type="checkbox"
            disabled={busy || !eligible.length}
            checked={eligible.length > 0 && eligible.every((row) => selected.includes(row.record))}
            indeterminate={eligible.some((row) => selected.includes(row.record)) &&
              !eligible.every((row) => selected.includes(row.record))}
            onchange={(event) => {
              selected = event.currentTarget.checked ? eligible.map((row) => row.record) : [];
            }}
          /> Select new entries
        </label>
      </div>
      {#each readyRows as row (row.record)}<ImportReviewRow
          {row}
          selected={selected.includes(row.record)}
          choice={decisions[row.record] ?? ''}
          {busy}
          onselect={(checked) => select(row.record, checked)}
          onchoice={(choice) => decide(row.record, choice)}
        />{/each}
      {#if !readyRows.length}<p class="muted">
          No new entries are ready. You can resolve a duplicate above or choose another file.
        </p>{/if}
      {#if knownRows.length}<details class="known">
          <summary>
            {knownRows.length} already imported · excluded
          </summary>{#each knownRows as row (row.record)}<ImportReviewRow
              {row}
              selected={false}
              choice=""
              {busy}
              onselect={() => {}}
              onchoice={() => {}}
            />{/each}
        </details>{/if}
    </div>
    <div class="import-bottom">
      <div class="selected-totals">
        <div>
          <span class="muted">Selected income</span><strong>{money(selection.income)}</strong>
        </div>
        <div>
          <span class="muted">Selected spending</span><strong>{money(selection.spending)}</strong>
        </div>
      </div>
      <p class="muted" role="status">
        {selection.additions.length} selected · {selection.matches.length} matches · {selection.excluded}
        left for later · {knownRows.length} already imported
      </p>
      <div class="import-actions">
        <button
          type="button"
          class="button"
          disabled={busy}
          onclick={() => {
            rows = null;
            error = '';
          }}
        >
          Choose file again
        </button><button
          type="button"
          class="button primary"
          disabled={busy || !selection.records.length}
          onclick={commit}
        >
          {busy
            ? 'Importing…'
            : `Import ${selection.additions.length} entries${selection.matches.length ? ` + ${selection.matches.length} matches` : ''}`}
        </button>
      </div>
    </div>
  {/if}
</dialog>

<style>
  .import-dialog {
    width: min(780px, calc(100% - 2rem));
  }
  .import-dialog[open] {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow: hidden;
  }
  .import-dialog > :global(form) {
    overflow-y: auto;
  }
  .review-scroll {
    overflow-y: auto;
    min-height: 0;
    padding-right: 0.3rem;
  }
  .import-bottom {
    flex-shrink: 0;
    border-top: 1px solid var(--line);
    padding-top: 1rem;
    display: grid;
    gap: 0.75rem;
  }
  .selected-totals {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
  }
  .selected-totals strong {
    display: block;
    font-size: 1.3rem;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }
  .group-heading {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
  }
  .select-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  .select-heading label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 44px;
  }
  input[type='checkbox'] {
    width: 19px;
    min-height: 19px;
    accent-color: var(--accent);
  }
  .known {
    margin: 1rem 0;
  }
  summary {
    min-height: 44px;
  }
  code {
    overflow-wrap: anywhere;
  }
  .import-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  @media (max-width: 620px) {
    .import-actions > :global(button) {
      flex: 1;
    }
  }
</style>
