<script lang="ts">
  import RuleExplanation from '#lib/features/rules/RuleExplanation.svelte';
  import ClassificationEditor from '#lib/features/classification/ClassificationEditor.svelte';
  import type {
    ClassificationCatalog,
    ClassificationChange,
  } from '#lib/features/classification/types.ts';
  import type { ImportClassificationDraft } from './classification.ts';
  import { escapeToCancel } from '../ledger/escape.ts';
  import { onDestroy, tick } from 'svelte';
  import SelectionHighlight from '#lib/components/selection/SelectionHighlight.svelte';
  import RoundSelection from '#lib/components/selection/RoundSelection.svelte';
  import { rowSelection } from '#lib/components/selection/row-selection.ts';
  import type { ImportRow } from '#lib/api.ts';
  import { money } from '#lib/money.ts';
  let {
    row,
    catalog,
    oncatalogchanged,
    classification,
    discardedClassification,
    onclassification,
    onclassificationbusy,
    selected,
    choice,
    busy,
    onselect,
    onchoice,
    draft,
    discardedDraft,
    ondescription,
    onediting,
  }: {
    row: ImportRow;
    catalog: ClassificationCatalog;
    oncatalogchanged: () => Promise<void>;
    classification?: ImportClassificationDraft;
    discardedClassification?: boolean;
    onclassification: (change: ClassificationChange) => Promise<void>;
    onclassificationbusy: (busy: boolean) => void;
    selected: boolean;
    choice: string;
    busy: boolean;
    onselect: (checked: boolean) => void;
    onchoice: (choice: string) => void;
    draft?: string;
    discardedDraft?: string;
    ondescription: (value: string) => void;
    onediting: (editing: boolean) => void;
  } = $props();
  const selectable = $derived(
    !row.description_error &&
      (row.status === 'new' || (row.status === 'review' && choice === 'keep')),
  );
  let editing = $state(false);
  let classifying = $state(false);
  const blocked = $derived(busy || classifying);
  const category = $derived(
    catalog.categories.find((c) => c.id === classification?.category_id) ?? null,
  );
  const labels = $derived(catalog.labels.filter((l) => classification?.label_ids.includes(l.id)));
  const hasClassification = $derived(
    Boolean(classification?.category_id || classification?.label_ids.length),
  );
  let text = $state('');
  let input = $state<HTMLInputElement>();
  let pendingMatch = $state('');
  async function edit() {
    pendingMatch = '';
    text = row.description;
    editing = true;
    onediting(true);
    await tick();
    input?.focus();
  }
  onDestroy(() => {
    onediting(false);
    onclassificationbusy(false);
  });
  function choose(value: string) {
    pendingMatch = '';
    onediting(false);
    if (value.startsWith('match:') && (draft !== undefined || hasClassification)) {
      pendingMatch = value;
      onediting(true);
      return;
    }
    onchoice(value);
  }
</script>

<article
  class="review-row"
  class:selected
  class:selectable={selectable && !blocked}
  use:rowSelection={{ selected, disabled: blocked || !selectable, onselect }}
  class:attention={row.status === 'invalid' ||
    row.status === 'conflict' ||
    (row.status === 'review' && !choice) ||
    !!row.description_error}
>
  <SelectionHighlight {selected} />
  <div class="row-main">
    <RoundSelection
      label={`Include ${row.description || 'invalid row'}`}
      {selected}
      disabled={blocked || !selectable}
      {onselect}
    />

    <div>
      {#if editing}
        <form
          class="description-form"
          onsubmit={(event) => {
            event.preventDefault();
            if (!text.trim()) {
              input?.setCustomValidity('Enter a description before saving.');
              input?.reportValidity();
              return;
            }
            editing = false;
            onediting(false);
            ondescription(text);
          }}
          use:escapeToCancel={() => {
            editing = false;
            onediting(false);
            void tick().then(() =>
              document.getElementById(`import-description-${row.record}`)?.focus(),
            );
          }}
        >
          <label>
            Description<input
              bind:this={input}
              bind:value={text}
              oninput={() => input?.setCustomValidity('')}
              required
              maxlength="200"
              disabled={blocked}
            />
          </label>
          <div>
            <button class="button" disabled={blocked}>Save description</button><button
              type="button"
              class="button quiet"
              onclick={() => {
                editing = false;
                onediting(false);
                void tick().then(() =>
                  document.getElementById(`import-description-${row.record}`)?.focus(),
                );
              }}
            >
              Cancel
            </button>
          </div>
        </form>
      {:else if !row.portable && row.status !== 'known' && !choice.startsWith('match:')}
        <h4>
          <button
            id={`import-description-${row.record}`}
            type="button"
            class="description-button"
            disabled={blocked}
            onclick={edit}
          >
            {row.description || 'Add description'}
          </button>
        </h4>
      {:else}<h4>{row.description || 'Original description empty'}</h4>{/if}
      <p class="muted">{row.date}</p>
      {#if row.status === 'known'}<span class="badge">Already imported</span>
      {:else if row.status === 'invalid'}<span class="badge warning">Couldn’t read this entry</span>
      {:else if row.status === 'conflict'}<span class="badge warning">
          Conflicting import evidence
        </span>
      {:else if row.status === 'review'}<span class="badge" class:warning={!choice}>
          {choice.startsWith('match:')
            ? 'Matched to existing entry'
            : choice === 'keep'
              ? 'Separate purchase'
              : 'Possible duplicate'}
        </span>{/if}
    </div>
    <span class="amount">{row.amount_cents ? money(row.amount_cents, true) : 'Not validated'}</span>
  </div>
  {#if row.portable}
    <p class="muted">
      {row.portable.category || 'Uncategorized'} · Labels: {row.portable.labels.join(', ') ||
        'None'} · Source: {row.portable.source || 'No source'}
    </p>
  {:else if selectable}
    <div class="draft-classification">
      <ClassificationEditor
        {catalog}
        {category}
        {labels}
        name={row.description || 'this imported entry'}
        idPrefix={`import-${row.record}`}
        disabled={busy || editing}
        onchange={onclassification}
        {oncatalogchanged}
        onbusy={(value) => {
          classifying = value;
          onclassificationbusy(value);
        }}
      />
    </div>
  {/if}
  {#if selectable && row.rule_suggestions?.matched_rules.length}
    <div class="draft-classification">
      <RuleExplanation
        plan={row.rule_suggestions}
        {catalog}
        confirmed={classification?.category_confirmed ?? false}
      /><button
        type="button"
        class="button quiet"
        disabled={blocked}
        onclick={async () => {
          if (!classification?.category_confirmed)
            await onclassification({ action: 'set_category', category_id: null });
          if (!classification?.labels_confirmed)
            await onclassification({
              action: 'remove_labels',
              label_ids: row.rule_suggestions?.label_ids ?? [],
            });
        }}
      >
        Remove suggestions
      </button>
    </div>
  {/if}
  {#if row.description_error}<p class="row-error">
      {row.description_error} Add or edit the description before selecting this entry.
    </p>{/if}
  {#if choice.startsWith('match:')}<p class="muted">
      The existing entry’s description, category and labels are kept.{discardedDraft
        ? ` Draft “${discardedDraft}” was discarded.`
        : ''}{discardedClassification ? ' Draft category/labels were discarded.' : ''}
    </p>{/if}
  {#if pendingMatch}<div role="alert">
      <p>
        The existing entry’s description, category and labels will be kept. Discard your draft
        changes{draft !== undefined ? ` (“${draft}”)` : ''}?
      </p>
      <button
        type="button"
        class="button"
        disabled={blocked}
        onclick={() => {
          onediting(false);
          onchoice(pendingMatch);
          pendingMatch = '';
        }}
      >
        Discard draft & match
      </button><button
        type="button"
        class="button quiet"
        disabled={blocked}
        onclick={() => {
          pendingMatch = '';
          onediting(false);
        }}
      >
        Keep draft
      </button>
    </div>{/if}
  {#if row.error}<p class="row-error">{row.error} Left out for now.</p>{/if}
  {#if row.status === 'conflict'}<p class="row-error">
      This record conflicts with original import evidence or repeats a bank ID. Check the mapping or
      correct the CSV.
    </p>{/if}
  {#if row.status === 'review'}
    <label class="decision">
      Duplicate decision for {row.description}<select
        value={choice}
        disabled={blocked || editing}
        onchange={(event) => {
          const control = event.currentTarget;
          choose(control.value);
          if (pendingMatch) control.value = choice;
        }}
      >
        <option value="">Leave for later</option><option value="keep">
          Keep as a separate purchase
        </option>
        {#each row.candidates as candidate (candidate.id)}<option value={`match:${candidate.id}`}>
            Match: {candidate.date} · {candidate.description} · {money(candidate.amount_cents)}
          </option>{/each}
      </select>
    </label>
  {/if}
  <details>
    <summary>Import details</summary>
    <p class="muted">
      CSV record {row.record}{row.external_id ? ` · Bank ID ${row.external_id}` : ' · No bank ID'}
    </p>
  </details>
</article>

<style>
  .draft-classification {
    margin-left: 3.5rem;
    min-width: 0;
  }
  .review-row {
    position: relative;
    isolation: isolate;
    padding: 0.75rem;
    border-radius: 0.7rem;
    margin-bottom: 0.5rem;
    overflow-wrap: anywhere;
  }
  .review-row.selectable {
    cursor: pointer;
  }
  .review-row > :global(:not(.selection-highlight)) {
    position: relative;
    z-index: 1;
  }
  .attention {
    background: var(--review-fill);
    padding: 1rem;
    border-radius: 0.7rem;
    margin-bottom: 0.75rem;
    border: 0;
  }
  .row-main {
    display: grid;
    grid-template-columns: 44px minmax(0, 1fr) auto;
    gap: 0.75rem;
    align-items: start;
  }
  .description-button {
    background: transparent;
    color: inherit;
    border: 0;
    padding: 0.15rem 0;
    text-align: left;
    font: inherit;
    cursor: pointer;
    min-height: 32px;
    overflow-wrap: anywhere;
  }
  .description-button:hover {
    text-decoration: underline;
  }
  .description-form {
    display: grid;
    gap: 0.5rem;
  }
  .description-form div {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  @media (pointer: coarse) {
    .description-button {
      min-height: 44px;
    }
  }
  h4 {
    margin: 0;
    font-size: 1rem;
    font-weight: 500;
  }
  .amount {
    font-variant-numeric: tabular-nums;
    overflow-wrap: anywhere;
    max-width: 100%;
  }
  .badge {
    display: inline-block;
    background: var(--status-fill);
    color: var(--status-ink);
    font-size: 0.875rem;
    padding: 0.15rem 0.5rem;
    border-radius: 1rem;
    margin-top: 0.35rem;
  }
  .warning {
    background: var(--review-fill);
    color: var(--review-ink);
  }
  .decision,
  .row-error,
  details {
    margin-top: 0.75rem;
  }
  details {
    font-size: 0.875rem;
    color: var(--muted);
  }
  summary {
    min-height: 36px;
  }
  .row-error {
    font-size: 0.875rem;
    color: var(--review-ink);
  }
  @media (max-width: 420px) {
    .amount {
      grid-column: 2;
    }
  }
</style>
