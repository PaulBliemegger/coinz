<script lang="ts">
  import { api, ApiError } from '#lib/api.ts';
  let {
    onrestored,
    onunauthorized,
  }: { onrestored: () => Promise<void>; onunauthorized: () => void } = $props();
  type Summary = {
    categories: number;
    labels: number;
    sources: number;
    profiles: number;
    rules: number;
    budgets: number;
    receipt_links: number;
  };
  let file = $state<File | null>(null);
  let fileInput = $state<HTMLInputElement>();
  let snapshot = $state<unknown>(null);
  let summary = $state<Summary | null>(null);
  let busy = $state(false);
  let error = $state('');
  let notice = $state('');
  function failed(cause: unknown) {
    if (cause instanceof ApiError && cause.status === 401) onunauthorized();
    error = cause instanceof Error ? cause.message : 'Could not restore settings.';
  }
  async function preview() {
    if (!file || busy) return;
    busy = true;
    error = '';
    notice = '';
    summary = null;
    snapshot = null;
    try {
      if (file.size > 8 * 1024 * 1024)
        throw new Error('Choose a settings file no larger than 8 MiB.');
      const settings: unknown = JSON.parse(await file.text());
      const next = await api<Summary>('/settings/restore/preview', 'POST', settings);
      snapshot = settings;
      summary = next;
    } catch (cause) {
      failed(cause);
    } finally {
      busy = false;
    }
  }
  async function restore() {
    if (!summary || !snapshot || busy) return;
    busy = true;
    error = '';
    try {
      await api('/settings/restore/commit', 'POST', snapshot);
      summary = null;
      snapshot = null;
      file = null;
      // Clearing the native input lets choosing the same file trigger change again.
      if (fileInput) fileInput.value = '';
      notice = 'Settings restored. Now restore your entries through Import CSV.';
      try {
        await onrestored();
      } catch {
        error = 'Settings were restored, but the screen could not refresh. Reload the page.';
      }
    } catch (cause) {
      failed(cause);
    } finally {
      busy = false;
    }
  }
</script>

<div class="settings-portability">
  <h4>Settings copy</h4>
  <p>
    Keep categories, labels, sources, CSV profiles, rules and budgets in a versioned JSON file.
    Download both files before resetting an installation.
  </p>
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- Axum serves this download outside SvelteKit routes. -->
  <a class="button" href="/api/exports/settings.json" download>Export settings</a>
  <h4>Restore settings</h4>
  <p>
    Use an empty account. Restore settings first, then import your entries CSV. Existing setup or
    entries will block restoration.
  </p>
  <label>
    Settings JSON file<input
      type="file"
      bind:this={fileInput}
      accept=".json,application/json"
      disabled={busy}
      onchange={(event) => {
        file = event.currentTarget.files?.[0] ?? null;
        summary = null;
        snapshot = null;
        error = '';
        notice = '';
      }}
    />
  </label>
  <button class="button" disabled={!file || busy} onclick={preview}>Preview settings</button>
  {#if summary}
    <div role="region" aria-label="Settings restore preview">
      <p>
        Ready to restore {summary.categories} categories, {summary.labels} labels, {summary.sources} sources,
        {summary.profiles} profiles, {summary.rules} rules and {summary.budgets} monthly budgets.
      </p>
      <p>
        {summary.receipt_links} income receipt links will reconnect when their CSV entries are restored.
        Import history and account credentials are excluded.
      </p>
      <button class="button primary" disabled={busy} onclick={restore}>Restore settings</button>
      <button
        class="button quiet"
        disabled={busy}
        onclick={() => {
          summary = null;
          snapshot = null;
        }}
      >
        Cancel restore
      </button>
    </div>
  {/if}
  {#if busy}<p role="status">Checking settings…</p>{/if}
  {#if notice}<p class="notice" role="status">{notice}</p>{/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}
</div>

<style>
  .settings-portability {
    display: grid;
    gap: 0.75rem;
    margin-top: 1.5rem;
  }
  h4,
  p {
    margin: 0;
  }
  input {
    max-width: 100%;
    min-width: 0;
  }
  label {
    min-width: 0;
  }
</style>
