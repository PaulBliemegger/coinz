<script lang="ts">
  import { escapeToCancel } from './escape.ts';
  import SelectionHighlight from '#lib/components/selection/SelectionHighlight.svelte';
  import RoundSelection from '#lib/components/selection/RoundSelection.svelte';
  import { rowSelection } from '#lib/components/selection/row-selection.ts';
  import { tick } from 'svelte';
  import { api, type Transaction, type Source } from '#lib/api.ts';
  import { money, amountInput, dateLabel } from '#lib/money.ts';
  let {
    entry,
    sources,
    selected,
    disabled,
    onselect,
    onchanged,
  }: {
    entry: Transaction;
    sources: Source[];
    selected: boolean;
    disabled: boolean;
    onselect: (checked: boolean) => void;
    onchanged: (notice: string) => Promise<void>;
  } = $props();
  type Field = 'description' | 'amount' | 'date' | 'source';
  let field = $state<Field | null>(null);
  let value = $state('');
  let kind = $state('expense');
  let busy = $state(false);
  let error = $state('');
  let input = $state<HTMLInputElement | HTMLSelectElement>();
  let trigger: HTMLButtonElement | null = null;
  const method = $derived(
    entry.origin === 'csv' ? 'CSV' : entry.origin === 'manual' ? 'Manual' : 'Origin unknown',
  );

  async function start(next: Field, event: MouseEvent) {
    trigger = event.currentTarget as HTMLButtonElement;
    field = next;
    error = '';
    kind = BigInt(entry.amount_cents) > 0n ? 'income' : 'expense';
    value =
      next === 'description'
        ? entry.description
        : next === 'amount'
          ? amountInput(entry.amount_cents)
          : next === 'date'
            ? entry.date
            : (entry.source_id ?? '');
    await tick();
    input?.focus();
  }
  function cancel() {
    field = null;
    error = '';
    trigger?.focus();
  }
  async function save(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    error = '';
    const savedField = field;
    try {
      await api(`/transactions/${entry.id}`, 'PATCH', {
        field,
        value: field === 'source' ? value || null : value,
        ...(field === 'amount' ? { kind } : {}),
      });
      field = null;
      await onchanged('Transaction updated.');
      await tick();
      (
        document.getElementById(`entry-${entry.id}-${savedField}`) ??
        document.getElementById('ledger-heading')
      )?.focus();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Could not update this entry.';
    } finally {
      busy = false;
    }
  }
</script>

<li
  class="entry-row"
  class:selected
  class:selection-disabled={disabled || busy}
  use:rowSelection={{ selected, disabled: disabled || busy, onselect }}
>
  <SelectionHighlight {selected} />
  <RoundSelection
    label={`Select ${entry.description}`}
    {selected}
    disabled={disabled || busy}
    {onselect}
  />
  <div class="entry-content">
    <button
      type="button"
      class="field description"
      disabled={busy || disabled}
      id={`entry-${entry.id}-description`}
      aria-label={`Change description: ${entry.description}`}
      onclick={(event) => start('description', event)}
    >
      {entry.description}
    </button>
    <div class="metadata">
      <button
        type="button"
        class="field"
        disabled={busy || disabled}
        id={`entry-${entry.id}-date`}
        aria-label={`Change date: ${entry.description}`}
        onclick={(event) => start('date', event)}
      >
        {dateLabel(entry.date)}
      </button><span>·</span><button
        type="button"
        class="field"
        disabled={busy || disabled}
        id={`entry-${entry.id}-source`}
        aria-label={`Change source: ${entry.description}`}
        onclick={(event) => start('source', event)}
      >
        {entry.source_name ?? 'No source'}
      </button><span>
        · {method}{entry.origin !== 'csv' && entry.csv_linked ? ' · CSV matched' : ''}
      </span>
    </div>
  </div>
  <button
    type="button"
    class="field amount"
    class:income={BigInt(entry.amount_cents) > 0n}
    disabled={busy || disabled}
    id={`entry-${entry.id}-amount`}
    aria-label={`Change amount: ${entry.description}`}
    onclick={(event) => start('amount', event)}
  >
    {money(entry.amount_cents, true)}
  </button>
  {#if field}
    <form
      class="inline-form"
      onsubmit={save}
      use:escapeToCancel={() => {
        if (!busy) cancel();
      }}
    >
      {#if field === 'source'}<label>
          Source<select bind:this={input} bind:value disabled={busy}>
            <option value="">No source</option>{#each sources as source (source.id)}<option
                value={source.id}
              >
                {source.name}
              </option>{/each}
          </select>
        </label>
      {:else}<label>
          {field === 'description' ? 'Description' : field === 'amount' ? 'Amount' : 'Date'}<input
            bind:this={input}
            bind:value
            type={field === 'date' ? 'date' : 'text'}
            inputmode={field === 'amount' ? 'decimal' : undefined}
            maxlength={field === 'description' ? 200 : 30}
            required
            disabled={busy}
          />
        </label>{/if}
      {#if field === 'amount'}<label>
          Kind<select bind:value={kind} disabled={busy}>
            <option value="expense">Expense</option><option value="income">Income</option>
          </select>
        </label>{/if}
      <div class="inline-actions">
        <button class="button primary" disabled={busy}>Save</button><button
          type="button"
          class="button"
          disabled={busy}
          onclick={cancel}
        >
          Cancel
        </button>
      </div>
      {#if error}<p class="error" role="alert">{error}</p>{/if}
    </form>
  {/if}
</li>

<style>
  .entry-row {
    display: grid;
    position: relative;
    isolation: isolate;
    grid-template-columns: 44px minmax(0, 1fr) auto;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem 0.5rem 0.25rem;
    border-radius: 0.65rem;
    cursor: pointer;
    background: var(--paper);
  }
  .entry-row.selection-disabled {
    cursor: default;
  }
  .entry-row > :global(:not(.selection-highlight)) {
    position: relative;
    z-index: 1;
  }
  .field {
    font: inherit;
    color: inherit;
    border: 0;
    border-radius: 0.3rem;
    padding: 0;
    background: transparent;
    text-align: left;
    min-height: 24px;
    overflow-wrap: anywhere;
  }
  .field:hover:not(:disabled),
  .field:focus-visible {
    background: var(--soft);
    text-decoration: underline dotted;
    text-underline-offset: 0.3rem;
  }
  .description {
    font-weight: 500;
  }
  .metadata {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem;
    color: var(--muted);
    font-size: 0.875rem;
  }
  .amount {
    white-space: normal;
    overflow-wrap: anywhere;
    font-variant-numeric: tabular-nums;
    align-self: start;
  }
  .income {
    color: var(--status-ink);
  }
  .inline-form {
    grid-column: 1/-1;
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
    align-items: end;
  }
  .inline-form label {
    flex: 1;
    min-width: min(160px, 100%);
  }
  .inline-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .inline-form .error {
    width: 100%;
  }
  @media (hover: hover) and (pointer: fine) {
    :global(.entries-list:not(.show-selection))
      .entry-row:not(:hover):not(:focus-within)
      :global(.selection-control) {
      opacity: 0;
    }
  }
  @media (pointer: coarse) {
    .field {
      min-height: 44px;
    }
  }
  @media (max-width: 420px) {
    .entry-row {
      grid-template-columns: 44px minmax(0, 1fr);
    }
    .amount {
      grid-column: 2;
      grid-row: 2;
    }
  }
</style>
