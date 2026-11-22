<script lang="ts">
  import { api, type ImportHistoryPage, type ImportReview } from '#lib/api.ts';
  import { dateLabel } from '#lib/money.ts';
  import ImportRemovalEntries from './ImportRemovalEntries.svelte';
  import { escapeToCancel } from '#lib/features/ledger/escape.ts';
  let {
    refresh,
    onchanged,
    onimport,
  }: { refresh: number; onchanged: (notice: string) => Promise<void>; onimport: () => void } =
    $props();
  let history = $state<ImportHistoryPage | null>(null);
  let review = $state<ImportReview | null>(null);
  let page = $state(1);
  let selected = $state('');
  let selectedEdited = $state<string[]>([]);
  let confirm = $state(false);
  let loading = $state(false);
  let busy = $state(false);
  let error = $state('');
  let retry = $state(0);
  let removalEntries = $derived(
    [
      ...(review?.removable_entries ?? []),
      ...(review?.edited_entries ?? []).filter((entry) => selectedEdited.includes(entry.id)),
    ].sort((a, b) => b.date.localeCompare(a.date) || b.id.localeCompare(a.id)),
  );
  let eligible = $derived(removalEntries.length);
  let overlappingRemovals = $derived(removalEntries.filter((entry) => entry.overlapping).length);
  function chooseEdited(id: string, checked: boolean) {
    if (busy) return;
    selectedEdited = checked
      ? [...new Set([...selectedEdited, id])]
      : selectedEdited.filter((item) => item !== id);
    confirm = false;
  }
  $effect(() => {
    const id = selected;
    const requestedPage = page;
    void refresh;
    void retry;
    let active = true;
    loading = true;
    error = '';
    confirm = false;
    selectedEdited = [];
    if (id) review = null;
    const request = id
      ? api<ImportReview>(`/imports/history/${id}`)
      : api<ImportHistoryPage>(`/imports/history?page=${requestedPage}`);
    request
      .then((result) => {
        if (active) {
          if ('batch' in result) review = result;
          else {
            history = result;
            page = result.page;
          }
        }
      })
      .catch((cause) => {
        if (active)
          error = cause instanceof Error ? cause.message : 'Could not load import history.';
      })
      .finally(() => {
        if (active) loading = false;
      });
    return () => {
      active = false;
    };
  });
  async function remove() {
    if (!review || !confirm || busy) return;
    busy = true;
    error = '';
    try {
      const result = await api<{ removed: number }>(
        `/imports/history/${review.batch.id}/remove`,
        'POST',
        { token: review.token, selected_edited_ids: selectedEdited },
      );
      confirm = false;
      await onchanged(`${result.removed} imported entries removed. Protected entries were kept.`);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : 'Could not remove these entries.';
      confirm = false;
    } finally {
      busy = false;
    }
  }
</script>

<div class="section-heading">
  <p class="muted">The paperwork has paperwork.</p>
  <button class="button" disabled={busy} onclick={onimport}>Import CSV</button>
</div>
{#if selected}<button
    class="button quiet"
    disabled={busy}
    onclick={() => {
      selected = '';
      review = null;
    }}
  >
    ← All imports
  </button>{/if}
{#if error}<p class="error" role="alert">
    {error} <button class="button" disabled={busy} onclick={() => retry++}>Reload</button>
  </p>{/if}
{#if loading}<p role="status">Retrieving the receipts for the receipts…</p>
{:else if selected && review}
  <section class="ledger import-detail" aria-label="Review import removal" aria-busy={busy}>
    <p class="eyebrow">IMPORT HISTORY / REVIEW</p>
    <h2>A closer look before it goes.</h2>
    <p class="import-filename">{review.batch.filename}</p>
    <p class="muted import-meta">
      {review.batch.source_name} · Imported {new Date(review.batch.created_at).toLocaleString()}
    </p>
    <p class="muted import-meta">
      {review.batch.start_date ? dateLabel(review.batch.start_date) : 'No valid dates'} – {review
        .batch.end_date
        ? dateLabel(review.batch.end_date)
        : '—'} · {review.batch.added} entries added
    </p>
    <section class="removal-group" aria-labelledby="removal-title">
      <div class="removal-heading">
        <h3 id="removal-title">Will be removed <span class="muted">· {eligible}</span></h3>
      </div>
      <div class="removal-body">
        {#if eligible}<ImportRemovalEntries
            entries={removalEntries}
            label="Entries included in removal"
          />
        {:else}<p class="muted group-note">No entries currently selected for removal.</p>{/if}
      </div>
    </section>
    {#if review.edited}
      <details class="removal-group">
        <summary>
          <span>Edited entries · {review.edited}</span><span class="protection-tag">
            {review.edited - selectedEdited.length} protected
          </span><span class="disclosure-arrow" aria-hidden="true">›</span>
        </summary>
        <div class="removal-body">
          <p class="muted group-note">
            You changed these after importing. Choose individually to include them.
          </p>
          <ImportRemovalEntries
            entries={review.edited_entries}
            label="Edited entries"
            selected={selectedEdited}
            disabled={busy}
            onselect={chooseEdited}
          />
        </div>
      </details>
    {/if}
    <div class="removal-notes">
      {#if overlappingRemovals}<p class="muted">
          {overlappingRemovals} selected entries also appear in other imports. They will be deleted from
          your records.
        </p>{/if}
      {#if review.batch.known}<p class="muted">
          Already-recorded entries belong to their original import or were entered manually. This
          import cannot remove them.
        </p>{/if}
      {#if review.batch.matched}<p class="muted">
          {review.batch.matched} existing entries were matched. They will stay.
        </p>{/if}
      {#if review.already_deleted}<p class="muted">
          {review.already_deleted} entries have already been deleted.
        </p>{/if}
    </div>
    <div
      class="removal-footer"
      use:escapeToCancel={() => {
        if (!busy) confirm = false;
      }}
    >
      <p class="muted" aria-live="polite">
        {eligible} to remove · {review.edited - selectedEdited.length} edited entries protected
      </p>
      <div class="removal-actions">
        <button
          class="button"
          disabled={busy || eligible === 0}
          aria-expanded={confirm}
          aria-controls="import-removal-confirm"
          onclick={() => (confirm = !confirm)}
        >
          Remove {eligible} entries
        </button>
        <div id="import-removal-confirm" class="removal-actions" hidden={!confirm}>
          <button class="button destructive" disabled={busy} onclick={remove}>
            {busy ? 'Removing…' : 'Confirm removal'}
          </button><button class="button" disabled={busy} onclick={() => (confirm = false)}>
            Cancel
          </button>
        </div>
      </div>
    </div>
    {#if confirm}<p class="muted removal-warning">
        Permanent removal from your records and reports. You can import these entries again later.
      </p>{/if}
  </section>
{:else if !selected && history}
  {#if !history.batches.length}<div class="empty-state">
      <h2>No imports recorded yet.</h2>
      <p>New imports will appear here. Older imports have no batch history.</p>
    </div>
  {:else}<ul class="batch-list">
      {#each history.batches as batch (batch.id)}<li class="ledger">
          <div>
            <h2>{batch.filename}</h2>
            <p class="muted">{batch.source_name} · {new Date(batch.created_at).toLocaleString()}</p>
            <p>
              {batch.start_date ? dateLabel(batch.start_date) : 'No valid dates'} – {batch.end_date
                ? dateLabel(batch.end_date)
                : '—'}
            </p>
            <p>
              {batch.added} added · {batch.matched} matched · {batch.known} already known{batch.removed
                ? ` · ${batch.removed} removed`
                : ''}
            </p>
          </div>
          <button
            class="button"
            aria-label={`Review ${batch.filename}`}
            onclick={() => (selected = batch.id)}
          >
            Review
          </button>
        </li>{/each}
    </ul>{/if}
  <nav class="pagination" aria-label="Import history pages">
    <button class="button" disabled={page <= 1} onclick={() => page--}>Previous</button><span>
      Page {history.page} of {history.pages}
    </span><button class="button" disabled={page >= history.pages} onclick={() => page++}>
      Next
    </button>
  </nav>
{/if}

<style>
  .import-detail > h2 {
    margin: 0.6rem 0 1rem;
    font-size: 1.8rem;
  }
  .import-detail .import-filename {
    margin: 0;
    font-weight: 500;
    overflow-wrap: anywhere;
  }
  .import-detail .import-meta {
    margin: 0.25rem 0;
  }
  .removal-group {
    margin-top: 1rem;
    border: 1px solid var(--line);
    border-radius: 0.75rem;
    overflow: hidden;
  }
  .removal-heading,
  summary {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.5rem;
    padding: 1rem;
  }
  .removal-heading {
    background: var(--soft);
  }
  .removal-heading h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 500;
  }
  summary {
    min-height: 60px;
    cursor: pointer;
    list-style: revert;
  }
  .removal-group[open] summary {
    border-bottom: 1px solid var(--line);
  }
  .disclosure-arrow {
    font-size: 1.2rem;
    color: var(--muted);
  }
  details[open] .disclosure-arrow {
    transform: rotate(90deg);
  }
  .removal-body {
    padding: 0 1rem;
  }
  .protection-tag {
    background: var(--status-fill);
    color: var(--status-ink);
    padding: 0.2rem 0.6rem;
    border-radius: 999px;
    font-size: 0.875rem;
  }
  .import-detail .group-note {
    margin: 0;
    padding: 0.75rem 0;
  }
  .removal-notes {
    padding: 1rem 0;
  }
  .import-detail .removal-notes p {
    margin: 0.35rem 0;
  }
  .removal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.75rem;
    border-top: 1px solid var(--line);
    padding-top: 1rem;
  }
  .removal-footer p {
    margin: 0;
  }
  .removal-actions {
    align-items: center;
  }
  .removal-actions[hidden] {
    display: none;
  }
  .import-detail .removal-warning {
    margin: 0.75rem 0 0;
  }
</style>
