<script lang="ts">
  import { api, ApiError, type Source } from '#lib/api.ts';
  import { money } from '#lib/money.ts';
  import LabelChip from '#lib/features/classification/LabelChip.svelte';
  import type { ClassificationCatalog } from '#lib/features/classification/types.ts';
  import RuleEditor from './RuleEditor.svelte';
  import { operatorNames, type LabelRule, type RuleSnapshot, type RuleExample } from './types.ts';
  let {
    catalog,
    sources,
    onchanged,
    onbusy = () => {},
    onunauthorized = () => {},
  }: {
    catalog: ClassificationCatalog;
    sources: Source[];
    onchanged: (notice: string) => Promise<void>;
    onbusy?: (value: boolean) => void;
    onunauthorized?: () => void;
  } = $props();
  let snapshot = $state<RuleSnapshot | null>(null);
  let error = $state('');
  let loading = $state(true);
  let busy = $state(false);
  let editing = $state<LabelRule | null>(null);
  let creating = $state(false);
  let retry = $state(0);
  let deletion = $state<string | null>(null);
  let review = $state<{
    rule: LabelRule;
    data: {
      affected_entries: number;
      token: string;
      examples: { entry: RuleExample; category_id: string | null; label_ids: string[] }[];
    };
  } | null>(null);
  $effect(() => {
    void retry;
    let active = true;
    loading = true;
    error = '';
    api<RuleSnapshot>('/rules')
      .then((result) => {
        if (active) snapshot = result;
      })
      .catch((cause) => {
        if (active) {
          snapshot = null;
          if (cause instanceof ApiError && cause.status === 401) onunauthorized();
          error = cause instanceof Error ? cause.message : 'Could not load rules.';
        }
      })
      .finally(() => {
        if (active) loading = false;
      });
    return () => {
      active = false;
    };
  });
  function setBusy(value: boolean) {
    busy = value;
    onbusy(value);
  }
  async function changed(notice = 'Rule saved. Existing entries stay unchanged.') {
    review = null;
    retry++;
    await onchanged(notice);
  }
  async function toggle(rule: LabelRule) {
    setBusy(true);
    error = '';
    try {
      await api(`/rules/${rule.id}`, 'PUT', {
        name: rule.name,
        operator: rule.operator,
        pattern: rule.pattern,
        source_id: rule.source_id,
        category_id: rule.category_id,
        label_ids: rule.label_ids,
        enabled: !rule.enabled,
        revision: rule.revision,
      });
      await changed(rule.enabled ? 'Rule paused.' : 'Rule enabled.');
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not update the rule.';
    } finally {
      setBusy(false);
    }
  }
  async function remove(rule: LabelRule) {
    setBusy(true);
    error = '';
    try {
      await api(`/rules/${rule.id}`, 'DELETE', { revision: rule.revision });
      deletion = null;
      await changed('Rule deleted. Existing entries stay unchanged.');
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not delete rule.';
    } finally {
      setBusy(false);
    }
  }
  async function previewHistory(rule: LabelRule) {
    setBusy(true);
    error = '';
    try {
      review = { rule, data: await api(`/rules/${rule.id}/apply-preview`) };
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error = cause instanceof Error ? cause.message : 'Could not preview history.';
    } finally {
      setBusy(false);
    }
  }
  async function applyHistory() {
    if (!review) return;
    setBusy(true);
    error = '';
    try {
      await api(`/rules/${review.rule.id}/apply`, 'POST', { token: review.data.token });
      review = null;
      await changed('Rule applied to the reviewed existing entries.');
    } catch (cause) {
      review = null;
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      error =
        cause instanceof Error ? cause.message : 'Could not apply rule. Preview history again.';
    } finally {
      setBusy(false);
    }
  }
</script>

<section aria-label="Labeling rules" class="rules-manager">
  <div class="section-heading">
    <h2>Rules</h2>
    <button
      class="button primary"
      disabled={busy}
      onclick={() => {
        creating = true;
        editing = null;
        review = null;
      }}
    >
      + New rule
    </button>
  </div>
  <p class="muted">
    Local description rules suggest categories and labels. Your choices take precedence.
  </p>
  {#if error}<p class="error" role="alert">
      {error} <button class="button quiet" onclick={() => retry++}>Reload rules</button>
    </p>{/if}
  {#if creating || editing}{#key editing?.id ?? 'new'}<RuleEditor
        {catalog}
        {sources}
        initial={editing}
        onbusy={setBusy}
        {onunauthorized}
        onsaved={() => changed()}
        onclose={() => {
          creating = false;
          editing = null;
        }}
      />{/key}{/if}
  {#if loading}<p role="status">Loading rules…</p>{:else if snapshot}
    {#if !snapshot.rules.length}<p class="muted">
        No rules yet. Start with a description you recognize.
      </p>{/if}
    <ul class="rule-list">
      {#each snapshot.rules as rule (rule.id)}<li>
          <div>
            <strong>{rule.name}</strong>
            <p class="muted">
              Description {operatorNames[rule.operator]} “{rule.pattern}”{rule.source_id
                ? ` · ${sources.find((s) => s.id === rule.source_id)?.name ?? 'Source restricted'}`
                : ''}
            </p>
            <div class="classification-line">
              {#if rule.category_id}<span>
                  {catalog.categories.find((c) => c.id === rule.category_id)?.name ??
                    'Category unavailable'}
                </span>{/if}{#each catalog.labels.filter( (l) => rule.label_ids.includes(l.id) ) as item (item.id)}<LabelChip
                  {item}
                />{/each}
            </div>
            {#if rule.pause_reason}<p class="muted">Paused: {rule.pause_reason}</p>{/if}
          </div>
          <div class="classification-actions">
            <label class="setup-choice">
              <input
                type="checkbox"
                checked={rule.enabled}
                disabled={busy}
                onchange={() => toggle(rule)}
              />{rule.enabled ? 'Enabled' : 'Paused'}<span class="sr-only"> {rule.name}</span>
            </label><button
              class="button quiet"
              disabled={busy}
              onclick={() => {
                editing = rule;
                creating = false;
                review = null;
              }}
            >
              Edit<span class="sr-only"> {rule.name}</span>
            </button><button
              class="button quiet"
              disabled={busy || !rule.enabled}
              onclick={() => previewHistory(rule)}
            >
              Preview history<span class="sr-only"> {rule.name}</span>
            </button><button
              class="button quiet"
              disabled={busy}
              onclick={() => (deletion = deletion === rule.id ? null : rule.id)}
            >
              Delete<span class="sr-only"> {rule.name}</span>
            </button>
          </div>
          {#if deletion === rule.id}<div class="rule-confirmation">
              <p>Delete “{rule.name}”? Existing entry classifications will stay unchanged.</p>
              <button class="button destructive" disabled={busy} onclick={() => remove(rule)}>
                Delete rule
              </button><button class="button" disabled={busy} onclick={() => (deletion = null)}>
                Cancel
              </button>
            </div>{/if}
        </li>{/each}
    </ul>
  {/if}
  {#if review}<section class="rule-editor" aria-label="Apply rule to history">
      <h3>{review.rule.name} · existing entries</h3>
      <p>
        {review.data.affected_entries} entries would change across all recorded history. Only Uncategorized
        entries receive a category; existing categories and labels are preserved. Missing labels are added.
      </p>
      {#each review.data.examples as change (change.entry.id)}<div class="rule-example">
          <div>
            <span>{change.entry.description} · {change.entry.date}</span>
            <p class="muted">
              {change.category_id
                ? `Category → ${catalog.categories.find((c) => c.id === change.category_id)?.name ?? 'Unavailable'}`
                : 'Keep existing category'}{change.label_ids.length
                ? ` · Add labels: ${change.label_ids.map((id) => catalog.labels.find((l) => l.id === id)?.name ?? 'Unavailable').join(', ')}`
                : ''}
            </p>
          </div>
          <span>
            {money(change.entry.amount_cents, true)}
          </span>
        </div>{/each}
      <p class="muted">
        Showing up to 10 entries. The full affected set is rechecked before applying.
      </p>
      <div class="classification-actions">
        <button
          class="button primary"
          disabled={busy || !review.data.affected_entries}
          onclick={applyHistory}
        >
          Apply to {review.data.affected_entries} existing entries
        </button><button class="button" disabled={busy} onclick={() => (review = null)}>
          Cancel
        </button>
      </div>
    </section>{/if}
</section>
