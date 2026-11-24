<script lang="ts">
  import { tick } from 'svelte';
  import { api, ApiError, type Source } from '#lib/api.ts';
  import RulesManager from '#lib/features/rules/RulesManager.svelte';
  import LabelChip from './LabelChip.svelte';
  import {
    labelColors,
    suggestedCategories,
    suggestedLabels,
    type ClassificationCatalog,
    type ClassificationItem,
    type ClassificationKind,
  } from './types.ts';
  let {
    catalog,
    sources,
    onchanged,
    onunauthorized = () => {},
  }: {
    catalog: ClassificationCatalog;
    sources: Source[];
    onchanged: (notice: string) => Promise<void>;
    onunauthorized?: () => void;
  } = $props();
  let kind = $state<ClassificationKind>('category');
  let showDefaults = $state(false);
  let showRules = $state(false);
  let deletion = $state<{
    item: ClassificationItem;
    entry_count: number;
    rule_count: number;
    token: string;
  } | null>(null);
  let includeArchived = $state(false);
  let nameInput = $state<HTMLInputElement>();
  let name = $state('');
  let color = $state('sage');
  let editId = $state<string | null>(null);
  let busy = $state(false);
  let error = $state('');
  let categoryDefaults = $state<string[]>([...suggestedCategories]);
  let labelDefaults = $state<string[]>([]);
  const items = $derived(
    (kind === 'category' ? catalog.categories : catalog.labels).filter(
      (i) => includeArchived || !i.archived,
    ),
  );
  function reset() {
    deletion = null;
    editId = null;
    name = '';
    color = 'sage';
    error = '';
  }
  async function edit(item: ClassificationItem) {
    editId = item.id;
    name = item.name;
    color = item.color;
    error = '';
    await tick();
    nameInput?.focus();
  }
  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    busy = true;
    error = '';
    try {
      if (editId) {
        const item = (kind === 'category' ? catalog.categories : catalog.labels).find(
          (i) => i.id === editId,
        );
        if (!item) return;
        await api(`/classification/${editId}`, 'PUT', { name, color, archived: item.archived });
      } else await api('/classification', 'POST', { kind, name, color });
      reset();
      await onchanged('Saved. Existing entries keep their assignments.');
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not save this item.';
    } finally {
      busy = false;
    }
  }
  async function archive(item: ClassificationItem) {
    busy = true;
    error = '';
    try {
      await api(`/classification/${item.id}`, 'PUT', {
        ...item,
        id: undefined,
        archived: !item.archived,
      });
      reset();
      await onchanged(
        item.archived ? 'Restored.' : 'Archived. Existing entries keep their assignments.',
      );
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not update this item.';
    } finally {
      busy = false;
    }
  }
  async function previewDeletion(item: ClassificationItem) {
    busy = true;
    error = '';
    deletion = null;
    try {
      deletion = {
        item,
        ...(await api<{ entry_count: number; rule_count: number; token: string }>(
          `/classification/${item.id}/deletion-preview`,
        )),
      };
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not review deletion.';
    } finally {
      busy = false;
    }
  }
  async function deleteItem() {
    if (!deletion) return;
    busy = true;
    error = '';
    try {
      await api(`/classification/${deletion.item.id}`, 'DELETE', { token: deletion.token });
      reset();
      await onchanged('Deleted. Entry amounts and other classifications are preserved.');
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error =
        cause instanceof Error
          ? cause.message
          : 'Could not delete this item. Review deletion again.';
      deletion = null;
    } finally {
      busy = false;
    }
  }
  async function setup(skip: boolean) {
    busy = true;
    error = '';
    try {
      await api('/classification/setup', 'POST', {
        categories: skip ? [] : categoryDefaults,
        labels: skip ? [] : labelDefaults,
      });
      showDefaults = false;
      await onchanged(
        skip ? 'Start with your own categories and labels.' : 'Selected defaults added.',
      );
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not save setup.';
    } finally {
      busy = false;
    }
  }
</script>

<section aria-label="Categories and labels">
  {#if !catalog.setup_complete || showDefaults}
    <section class="classification-setup" aria-label="Choose your starting setup">
      <h2>A little structure, your way.</h2>
      <p>
        Choose the defaults you want. You can rename or archive them later. Existing entries stay
        Uncategorized.
      </p>
      <p class="muted">
        Housing covers your usual home; Accommodation covers hotels and temporary stays. Vacation
        can group an entire trip if you prefer that broader view.
      </p>
      <div class="setup-columns">
        <fieldset>
          <legend>Categories · choose any</legend>{#each suggestedCategories as item (item)}<label
              class="setup-choice"
            >
              <input
                type="checkbox"
                value={item}
                bind:group={categoryDefaults}
                disabled={busy}
              />{item}
            </label>{/each}
        </fieldset>
        <fieldset>
          <legend>Optional labels</legend>{#each suggestedLabels as item (item)}<label
              class="setup-choice"
            >
              <input
                type="checkbox"
                value={item}
                bind:group={labelDefaults}
                disabled={busy}
              />{item}
            </label>{/each}
        </fieldset>
      </div>
      <div class="classification-actions">
        <button class="button primary" disabled={busy} onclick={() => setup(false)}>
          Use selected defaults
        </button><button class="button" disabled={busy} onclick={() => setup(true)}>
          Start empty
        </button>
      </div>
    </section>
  {/if}
  <div class="classification-actions" aria-label="Manage classification">
    {#if catalog.setup_complete}<button
        class="button quiet"
        disabled={busy}
        onclick={() => (showDefaults = !showDefaults)}
      >
        {showDefaults ? 'Close defaults' : 'Choose defaults'}
      </button>{/if}
    <button
      class="button"
      aria-pressed={!showRules && kind === 'category'}
      disabled={busy}
      onclick={() => {
        showRules = false;
        kind = 'category';
        reset();
      }}
    >
      Categories
    </button>
    <button
      class="button"
      aria-pressed={!showRules && kind === 'label'}
      disabled={busy}
      onclick={() => {
        showRules = false;
        kind = 'label';
        reset();
      }}
    >
      Labels
    </button>
    <button
      class="button"
      aria-pressed={showRules}
      disabled={busy}
      onclick={() => {
        showRules = true;
        reset();
      }}
    >
      Rules
    </button>
  </div>
  {#if showRules}<RulesManager
      {catalog}
      {sources}
      {onchanged}
      {onunauthorized}
      onbusy={(value) => (busy = value)}
    />{:else}
    <p class="muted">
      {kind === 'category'
        ? 'One category per entry. Uncategorized keeps unfinished entries in your totals.'
        : 'Optional groupings for occasions, subscriptions, and anything else you want to explore.'}
    </p>
    <form class="classification-manage-form" onsubmit={save}>
      <label>
        {editId ? 'Rename' : 'New'}
        {kind}<input
          bind:this={nameInput}
          bind:value={name}
          maxlength="60"
          required
          disabled={busy}
        />
      </label>
      {#if kind === 'label'}<label>
          Colour<select bind:value={color} disabled={busy}>
            {#each labelColors as c (c)}<option value={c}>{c}</option>{/each}
          </select>
        </label>{/if}
      <button class="button primary" disabled={busy}>
        {editId ? 'Save changes' : `Create ${kind}`}
      </button>
      {#if editId}<button type="button" class="button" disabled={busy} onclick={reset}>
          Cancel
        </button>{/if}
    </form>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <label class="setup-choice">
      <input type="checkbox" bind:checked={includeArchived} />Include archived
    </label>
    <ul class="classification-library">
      {#if kind === 'category'}<li>
          <span>Uncategorized <span class="muted">· Built-in fallback</span></span>
        </li>{/if}
      {#each items as item (item.id)}
        <li>
          <div>
            {#if kind === 'label'}<LabelChip {item} />{:else}<span>
                {item.name}
              </span>{/if}{#if item.archived}<span class="muted"> · Archived</span>{/if}
          </div>
          <div class="classification-actions">
            <button class="button quiet" disabled={busy} onclick={() => edit(item)}>
              Edit <span class="sr-only">{item.name}</span>
            </button><button class="button quiet" disabled={busy} onclick={() => archive(item)}>
              {item.archived ? 'Restore' : 'Archive'}<span class="sr-only"> {item.name}</span>
            </button><button
              class="button quiet"
              disabled={busy}
              onclick={() => previewDeletion(item)}
            >
              Delete<span class="sr-only"> {item.name}</span>
            </button>
          </div>
          {#if deletion?.item.id === item.id}<div
              class="classification-delete-preview"
              role="alert"
            >
              <p>
                Delete “{item.name}”? This affects {deletion.entry_count} entries. {kind ===
                'category'
                  ? 'These entries will become Uncategorized.'
                  : 'This label will be removed from those entries.'} Amounts and other labels stay unchanged.
                {deletion.rule_count} affected rules will be paused and their deleted target removed.
                This cannot be undone.
              </p>
              <div class="classification-actions">
                <button class="button destructive" disabled={busy} onclick={deleteItem}>
                  Delete {kind}
                </button><button class="button" disabled={busy} onclick={() => (deletion = null)}>
                  Cancel
                </button>
              </div>
            </div>{/if}
        </li>
      {/each}
    </ul>
    {#if !items.length}<p class="muted">
        No {kind === 'category' ? 'custom categories' : 'labels'} yet. Create one above.
      </p>{/if}
  {/if}
</section>
