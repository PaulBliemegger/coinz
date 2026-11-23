<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { api, ApiError, type ImportLayout, type ImportProfile } from '#lib/api.ts';
  import { escapeToCancel } from '#lib/features/ledger/escape.ts';
  let {
    profile,
    onsaved,
    oncancel,
    onunauthorized,
  }: {
    profile: ImportProfile | null;
    onsaved: (profile: ImportProfile) => void;
    oncancel: () => void;
    onunauthorized: () => void;
  } = $props();
  const original = untrack(() => profile);
  let name = $state(original?.name ?? '');
  let layout = $state<ImportLayout>(
    original
      ? { ...original.layout }
      : {
          date: '',
          amount: '',
          description: '',
          external_id: null,
          delimiter: ',',
          date_format: 'YYYY-MM-DD',
          decimal: '.',
          grouping: '',
        },
  );
  let externalId = $state(original?.layout.external_id ?? '');
  let busy = $state(false);
  let error = $state('');
  let nameInput: HTMLInputElement;
  onMount(() => nameInput.focus());
  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    busy = true;
    error = '';
    try {
      const result = await api<ImportProfile>(
        original ? `/imports/profiles/${original.id}` : '/imports/profiles',
        original ? 'PUT' : 'POST',
        {
          name,
          layout: { ...layout, external_id: externalId.trim() ? externalId : null },
          ...(original ? { revision: original.revision } : {}),
        },
      );
      onsaved(result);
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) onunauthorized();
      else error = cause instanceof Error ? cause.message : 'Could not save profile.';
    } finally {
      busy = false;
    }
  }
</script>

<form
  class="profile-form"
  aria-labelledby="profile-form-title"
  aria-busy={busy}
  onsubmit={save}
  use:escapeToCancel={() => {
    if (!busy) oncancel();
  }}
>
  <h3 id="profile-form-title">{original ? 'Edit import profile' : 'New import profile'}</h3>
  <p class="muted">
    Enter column names exactly as they appear in the CSV header, including spaces and capital
    letters. No file is needed here; the layout is checked when you import.
  </p>
  <label>
    Profile name<input
      bind:this={nameInput}
      bind:value={name}
      required
      maxlength="80"
      disabled={busy}
    />
  </label>
  <fieldset disabled={busy}>
    <legend>CSV column names</legend>
    <div class="profile-fields">
      <label>Date column<input bind:value={layout.date} required maxlength="200" /></label>
      <label>Amount column<input bind:value={layout.amount} required maxlength="200" /></label>
      <label>
        Description column<input bind:value={layout.description} required maxlength="200" /><span
          class="muted"
        >
          A name, counterparty or reference column is suitable.
        </span>
      </label>
      <label>External ID column (optional)<input bind:value={externalId} maxlength="200" /></label>
    </div>
  </fieldset>
  <fieldset disabled={busy}>
    <legend>Reading the values</legend>
    <p class="muted">
      EUR only, with one signed amount column: positive is income, negative is spending. No rounding
      or format guessing.
    </p>
    <div class="profile-fields">
      <label>
        Delimiter<select bind:value={layout.delimiter}>
          <option value=",">Comma</option><option value=";">Semicolon</option>
        </select>
      </label>
      <label>
        Date format<select bind:value={layout.date_format}>
          <option>YYYY-MM-DD</option><option>DD.MM.YYYY</option><option>DD/MM/YYYY</option>
        </select>
      </label>
      <label>
        Decimal separator<select bind:value={layout.decimal}>
          <option value=".">Dot</option><option value=",">Comma</option>
        </select>
      </label>
      <label>
        Grouping separator<select bind:value={layout.grouping}>
          <option value="">None</option><option value=".">Dot</option><option value=",">
            Comma
          </option><option value=" ">Space</option>
        </select>
      </label>
    </div>
  </fieldset>
  <p class="muted">
    Changes apply to future imports only. Saving adds no entries and never changes earlier imports
    or their original evidence.
  </p>
  {#if error}<p class="error" role="alert">{error}</p>
    <p class="muted">
      If this profile changed elsewhere, cancel, reload profiles and reopen it to use the latest
      saved version.
    </p>{/if}
  <div class="profile-actions">
    <button type="button" class="button quiet" disabled={busy} onclick={oncancel}>
      Cancel
    </button><button class="button primary" disabled={busy}>
      {busy ? 'Saving…' : original ? 'Save profile changes' : 'Create profile'}
    </button>
  </div>
</form>

<style>
  .profile-form {
    display: grid;
    gap: 1.25rem;
    padding: 1.5rem 0;
    border-top: 1px solid var(--line);
  }
  h3,
  p {
    margin: 0;
  }
  fieldset {
    min-width: 0;
    border: 1px solid var(--line);
    border-radius: 0.75rem;
    padding: 1rem;
  }
  legend {
    font-weight: 500;
    padding: 0 0.4rem;
  }
  fieldset p {
    margin-bottom: 1rem;
  }
  .profile-fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
    align-items: start;
  }
  label > span {
    font-size: 0.875rem;
    font-weight: 400;
  }
  .profile-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    flex-wrap: wrap;
  }
  @media (max-width: 620px) {
    .profile-fields {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
