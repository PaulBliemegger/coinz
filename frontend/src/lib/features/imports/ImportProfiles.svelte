<script lang="ts">
  import { tick } from 'svelte';
  import { api, ApiError, type ImportProfile } from '#lib/api.ts';
  import ImportProfileForm from './ImportProfileForm.svelte';
  let { refresh, onunauthorized }: { refresh: number; onunauthorized: () => void } = $props();
  let profiles = $state<ImportProfile[]>([]);
  let editor = $state<{ profile: ImportProfile | null } | null>(null);
  let loading = $state(true);
  let error = $state('');
  let notice = $state('');
  let retry = $state(0);
  let triggerId = '';
  let catalogVersion = 0;
  $effect(() => {
    void refresh;
    void retry;
    let active = true;
    const version = ++catalogVersion;
    loading = true;
    error = '';
    api<ImportProfile[]>('/imports/profiles')
      .then((result) => {
        if (active && version === catalogVersion) profiles = result;
      })
      .catch((cause) => {
        if (
          active &&
          version === catalogVersion &&
          cause instanceof ApiError &&
          cause.status === 401
        )
          onunauthorized();
        else if (active && version === catalogVersion)
          error = cause instanceof Error ? cause.message : 'Could not load profiles.';
      })
      .finally(() => {
        if (active && version === catalogVersion) loading = false;
      });
    return () => {
      active = false;
    };
  });
  function edit(profile: ImportProfile | null) {
    triggerId = profile ? `edit-profile-${profile.id}` : 'new-import-profile';
    notice = '';
    editor = { profile };
  }
  async function close() {
    editor = null;
    await tick();
    document.getElementById(triggerId)?.focus();
  }
  async function saved(profile: ImportProfile) {
    catalogVersion++;
    loading = false;
    error = '';
    profiles = [...profiles.filter((item) => item.id !== profile.id), profile].sort((a, b) =>
      a.name.localeCompare(b.name),
    );
    notice = 'Profile saved. Earlier imports are unchanged.';
    await close();
  }
</script>

<details class="ledger profiles">
  <summary>Import profiles <span class="muted">· {profiles.length}</span></summary>
  <div class="profiles-body">
    <p class="muted">
      Reusable CSV layouts, separate from your import sources. Create or edit one without importing
      a file.
    </p>
    {#if notice}<p class="notice" role="status">{notice}</p>{/if}
    {#if editor}
      {#key editor}<ImportProfileForm
          profile={editor.profile}
          onsaved={(profile) => void saved(profile)}
          oncancel={() => void close()}
          {onunauthorized}
        />{/key}
    {:else}
      <div class="profile-actions">
        <button id="new-import-profile" type="button" class="button" onclick={() => edit(null)}>
          New profile
        </button><button
          type="button"
          class="button quiet"
          disabled={loading}
          onclick={() => retry++}
        >
          Reload profiles
        </button>
      </div>
      {#if error}<p class="error" role="alert">{error}</p>{/if}
      {#if loading}<p role="status">Loading profiles…</p>
      {:else if !error && !profiles.length}<p class="muted">
          No saved profiles yet. You can also create one by clicking CSV headers during an import.
        </p>
      {:else if !error}<ul class="profile-list">
          {#each profiles as profile (profile.id)}<li>
              <div>
                <h3>{profile.name}</h3>
                <p class="muted">
                  Date: {profile.layout.date} · Amount: {profile.layout.amount} · Description: {profile
                    .layout.description}
                </p>
                <p class="muted">
                  {profile.layout.date_format} · {profile.layout.delimiter === ','
                    ? 'Comma'
                    : 'Semicolon'} separated · EUR
                </p>
              </div>
              <button
                id={`edit-profile-${profile.id}`}
                type="button"
                class="button quiet"
                aria-label={`Edit profile ${profile.name}`}
                onclick={() => edit(profile)}
              >
                Edit
              </button>
            </li>{/each}
        </ul>{/if}
    {/if}
  </div>
</details>

<style>
  .profiles {
    padding: 0;
    margin-bottom: 1.5rem;
  }
  summary {
    min-height: 56px;
    padding: 1rem 1.25rem;
    cursor: pointer;
    font-size: 1.1rem;
    font-weight: 500;
  }
  .profiles-body {
    padding: 0 1.25rem 1.25rem;
  }
  .profile-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin: 1rem 0;
  }
  .profile-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .profile-list li {
    display: flex;
    align-items: start;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 1rem;
    padding: 1rem 0;
    border-top: 1px solid var(--line);
  }
  .profile-list li > div {
    flex: 1;
    min-width: min(100%, 16rem);
    overflow-wrap: anywhere;
  }
  h3 {
    font-size: 1rem;
    margin: 0;
    font-weight: 500;
  }
  li p {
    margin: 0.35rem 0 0;
  }
</style>
