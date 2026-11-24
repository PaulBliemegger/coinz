<script lang="ts">
  import { onMount } from 'svelte';
  import { api, ApiError, type Source } from '#lib/api.ts';
  import { money } from '#lib/money.ts';
  import type { ClassificationCatalog } from '#lib/features/classification/types.ts';
  import {
    operatorNames,
    type LabelRule,
    type RuleDefinition,
    type RuleExample,
    type RuleOperator,
    type RuleSeed,
  } from './types.ts';
  let {
    catalog,
    sources,
    initial = null,
    seed = null,
    onsaved,
    onclose,
    onbusy = () => {},
    onunauthorized = () => {},
  }: {
    catalog: ClassificationCatalog;
    sources: Source[];
    initial?: LabelRule | null;
    seed?: RuleSeed | null;
    onsaved: () => Promise<void>;
    onclose: () => void;
    onbusy?: (busy: boolean) => void;
    onunauthorized?: () => void;
  } = $props();
  let name = $state('');
  let pattern = $state('');
  let operator = $state<RuleOperator>('contains');
  let categoryId = $state('');
  let labelIds = $state<string[]>([]);
  let sourceId = $state('');
  let enabled = $state(true);
  let busy = $state(false);
  let error = $state('');
  let preview = $state<{ match_count: number; examples: RuleExample[] } | null>(null);
  const definition: RuleDefinition = $derived({
    name,
    operator,
    pattern,
    category_id: categoryId || null,
    label_ids: labelIds,
    source_id: sourceId || null,
    enabled,
    revision: initial?.revision,
  });
  const previewKey = $derived(JSON.stringify(definition));
  let shownKey = $state('');
  onMount(() => {
    name = initial?.name ?? seed?.description.slice(0, 60) ?? '';
    pattern = initial?.pattern ?? seed?.description ?? '';
    operator = initial?.operator ?? (seed ? 'equals' : 'contains');
    categoryId = initial?.category_id ?? seed?.category_id ?? '';
    labelIds = [...(initial?.label_ids ?? seed?.label_ids ?? [])];
    sourceId = initial?.source_id ?? '';
    enabled = initial?.enabled ?? true;
  });
  async function showPreview() {
    busy = true;
    onbusy(true);
    error = '';
    const key = previewKey;
    preview = null;
    try {
      preview = await api('/rules/preview', 'POST', definition);
      shownKey = key;
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not preview this rule.';
    } finally {
      busy = false;
      onbusy(false);
    }
  }
  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    busy = true;
    onbusy(true);
    error = '';
    try {
      await api(initial ? `/rules/${initial.id}` : '/rules', initial ? 'PUT' : 'POST', definition);
      onclose();
      await onsaved();
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not save this rule.';
    } finally {
      busy = false;
      onbusy(false);
    }
  }
</script>

<section class="rule-editor" aria-label={initial ? 'Edit rule' : 'New rule'}>
  <h2>{initial ? 'Edit rule' : 'New rule'}</h2>
  {#if initial?.pause_reason}<p class="rule-conflict">{initial.pause_reason}</p>{/if}
  <form onsubmit={save}>
    <label>Rule name<input bind:value={name} required maxlength="60" disabled={busy} /></label>
    <fieldset>
      <legend>When</legend>
      <div class="rule-condition">
        <span>Description</span><label>
          <span class="sr-only">Match condition</span><select bind:value={operator} disabled={busy}>
            {#each Object.entries(operatorNames) as [value, label] (value)}<option {value}>
                {label}
              </option>{/each}
          </select>
        </label><label>
          <span class="sr-only">Description text to match</span><input
            bind:value={pattern}
            required
            maxlength="200"
            disabled={busy}
          />
        </label>
      </div>
    </fieldset>
    <fieldset>
      <legend>Then</legend><label>
        Set category<select bind:value={categoryId} disabled={busy}>
          <option value="">
            Leave category unchanged
          </option>{#each catalog.categories.filter((c) => !c.archived || c.id === categoryId) as item (item.id)}<option
              value={item.id}
            >
              {item.name}{item.archived ? ' · Archived' : ''}
            </option>{/each}
        </select>
      </label>
      <fieldset>
        <legend>Add labels</legend>
        <div class="rule-label-options">
          {#each catalog.labels.filter((l) => !l.archived || labelIds.includes(l.id)) as item (item.id)}<label
              class="setup-choice"
            >
              <input
                type="checkbox"
                value={item.id}
                bind:group={labelIds}
                disabled={busy}
              />{item.name}{item.archived ? ' · Archived' : ''}
            </label>{/each}
        </div>
      </fieldset>
    </fieldset>
    <details>
      <summary>Advanced · optional source restriction</summary><label>
        Applies to<select bind:value={sourceId} disabled={busy}>
          <option value="">
            All sources and manual entries
          </option>{#each sources as source (source.id)}<option value={source.id}>
              {source.name}
            </option>{/each}
        </select>
      </label>
    </details>
    <label class="setup-choice">
      <input type="checkbox" bind:checked={enabled} disabled={busy} />Enabled
    </label>
    <div class="classification-actions">
      <button
        type="button"
        class="button"
        disabled={busy || !pattern.trim() || !name.trim()}
        onclick={showPreview}
      >
        Preview matching entries
      </button><span class="muted">
        Saving affects future suggestions. Existing entries stay unchanged.
      </span>
    </div>
    {#if preview && shownKey === previewKey}<section
        class="rule-preview"
        aria-label="Matching entries"
      >
        <p role="status">{preview.match_count} matching entries · showing up to 10</p>
        {#each preview.examples as entry (entry.id)}<div class="rule-example">
            <span>{entry.description}<small class="muted"> · {entry.date}</small></span><span>
              {money(entry.amount_cents, true)}
            </span>
          </div>{/each}
      </section>{/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <div class="classification-actions">
      <button class="button primary" disabled={busy || (!categoryId && !labelIds.length)}>
        Save rule
      </button><button type="button" class="button" disabled={busy} onclick={onclose}>
        Cancel
      </button>
    </div>
  </form>
</section>
