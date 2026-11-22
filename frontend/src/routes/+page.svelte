<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { resolve } from '$app/paths';
  import {
    api,
    ApiError,
    setCsrf,
    ledgerQuery,
    type Source,
    type Session,
    type Summary,
    type Transaction,
    type User,
  } from '#lib/api.ts';
  import { localDate, money, monthLabel } from '#lib/money.ts';
  import ImportDialog from '#lib/features/imports/ImportDialog.svelte';
  import EntriesList from '#lib/features/ledger/EntriesList.svelte';
  import {
    parseAppearance,
    setAppearance,
    appearanceKey,
    type Appearance,
  } from '#lib/appearance.ts';
  import { escapeToCancel } from '#lib/features/ledger/escape.ts';
  import { adjacentMonth } from '#lib/features/ledger/period.ts';
  import './app.css';

  let user = $state<User | null>(null);
  let ready = $state(false);
  let busy = $state(false);
  let loading = $state(false);
  let error = $state('');
  let formError = $state('');
  let notice = $state('');
  let sources = $state<Source[]>([]);
  let transactions = $state<Transaction[]>([]);
  let summary = $state<Summary | null>(null);
  let month = $state(localDate().slice(0, 7));
  let sourceFilter = $state('');
  let sourceDialog: HTMLDialogElement;
  let transactionDialog: HTMLDialogElement;
  let entrySource = $state('');
  let entryDate = $state(localDate());
  let entryKind = $state('expense');
  let entryAmount = $state('');
  let entryDescription = $state('');
  let loadVersion = 0;
  let importOpen = $state(false);
  let alwaysSelection = $state(false);
  let settingsDialog: HTMLDialogElement;
  let userMenu = $state<HTMLDetailsElement>();
  let appearance = $state<Appearance>('system');
  function chooseAppearance(value: string) {
    appearance = parseAppearance(value);
    setAppearance(appearance);
  }
  function moveMonth(direction: -1 | 1) {
    const next = adjacentMonth(month, direction);
    if (next) {
      month = next;
      changeFilter();
    }
  }
  function saveSelectionPreference(value: boolean) {
    alwaysSelection = value;
    try {
      localStorage.setItem('coinz.always-show-selection', String(value));
    } catch {
      /* Keep the preference usable without storage. */
    }
  }

  const message = (value: unknown) =>
    value instanceof Error ? value.message : 'Something went wrong. Please try again.';

  async function bootstrap() {
    error = '';
    ready = false;
    try {
      const session = await api<Session>('/session');
      setCsrf(session.csrf_token);
      user = session.user;
      if (user) await loadData();
    } catch (cause) {
      error = message(cause);
    } finally {
      ready = true;
    }
  }
  onMount(() => {
    try {
      alwaysSelection = localStorage.getItem('coinz.always-show-selection') === 'true';
    } catch {
      /* Default visibility applies. */
    }
    try {
      appearance = parseAppearance(localStorage.getItem(appearanceKey));
    } catch {
      /* Follow the system. */
    }
    document.documentElement.dataset.appearance = appearance;
    void bootstrap();
  });

  async function loadData() {
    const version = ++loadVersion;
    loading = true;
    error = '';
    const query = ledgerQuery(month, sourceFilter);
    try {
      const [nextSources, nextTransactions, nextSummary] = await Promise.all([
        api<Source[]>('/sources'),
        api<Transaction[]>(`/transactions?${query}`),
        api<Summary>(`/summary?${query}`),
      ]);
      if (version !== loadVersion) return;
      sources = nextSources;
      transactions = nextTransactions;
      summary = nextSummary;
    } catch (cause) {
      if (version !== loadVersion) return;
      if (cause instanceof ApiError && cause.status === 401) {
        user = null;
        sources = [];
        transactions = [];
        summary = null;
      }
      error = message(cause);
    } finally {
      if (version === loadVersion) loading = false;
    }
  }

  async function login(event: SubmitEvent) {
    event.preventDefault();
    const form = new FormData(event.currentTarget as HTMLFormElement);
    busy = true;
    error = '';
    try {
      // Refresh anonymous state as well, so expired or logged-out tabs can log in again.
      const anonymous = await api<Session>('/session');
      setCsrf(anonymous.csrf_token);
      const session = await api<Session>('/login', 'POST', {
        username: form.get('username'),
        password: form.get('password'),
      });
      setCsrf(session.csrf_token);
      user = session.user;
      sourceFilter = '';
      await loadData();
    } catch (cause) {
      error = message(cause);
    } finally {
      busy = false;
    }
  }

  async function logout() {
    busy = true;
    try {
      await api('/logout', 'POST');
      ++loadVersion;
      user = null;
      sources = [];
      transactions = [];
      summary = null;
      notice = '';
      await bootstrap();
    } catch (cause) {
      error = message(cause);
    } finally {
      busy = false;
    }
  }

  function openSource() {
    formError = '';
    sourceDialog.showModal();
  }

  async function saveSource(event: SubmitEvent) {
    event.preventDefault();
    const form = event.currentTarget as HTMLFormElement;
    const values = new FormData(form);
    busy = true;
    formError = '';
    try {
      const created = await api<{ id: string }>('/sources', 'POST', Object.fromEntries(values));
      entrySource = created.id;
      sourceDialog.close();
      form.reset();
      notice = 'Source added.';
      await loadData();
    } catch (cause) {
      formError = message(cause);
    } finally {
      busy = false;
    }
  }

  async function openEntry() {
    entrySource = sourceFilter;
    entryDate = localDate();
    entryKind = 'expense';
    entryAmount = '';
    entryDescription = '';
    formError = '';
    await tick();
    transactionDialog.showModal();
  }

  async function saveEntry(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    formError = '';
    try {
      await api('/transactions', 'POST', {
        source_id: entrySource || null,
        date: entryDate,
        kind: entryKind,
        amount: entryAmount,
        description: entryDescription,
      });
      transactionDialog.close();
      notice = 'Transaction recorded.';
      const entryMonth = entryDate.slice(0, 7);
      if (entryMonth !== month) {
        month = entryMonth;
        notice += ` Showing ${monthLabel(month)}.`;
      }
      if (sourceFilter && sourceFilter !== entrySource) sourceFilter = entrySource;
      await loadData();
    } catch (cause) {
      formError = message(cause);
    } finally {
      busy = false;
    }
  }

  function changeFilter() {
    notice = '';
    void loadData();
  }
</script>

<svelte:window
  onpointerdown={(event) => {
    if (userMenu?.open && event.target instanceof Node && !userMenu.contains(event.target))
      userMenu.open = false;
  }}
/>

<svelte:head>
  <title>coinz · A little clarity</title>
  <meta
    name="description"
    content="Your personal notebook for income, spending, and a little clarity."
  />
</svelte:head>

{#if !ready}
  <main class="loading-page">
    <span class="wordmark">coinz<span>.</span></span>
    <p>Counting our coinz…</p>
  </main>
{:else if !user}
  <main class="login-page">
    <section class="welcome" aria-labelledby="welcome-heading">
      <span class="wordmark">coinz<span>.</span></span>
      <p class="eyebrow">A PERSONAL FINANCE NOTEBOOK</p>
      <h1 id="welcome-heading">A little clarity.<br />A little less guessing.</h1>
      <p>Make sense of what comes in, what goes out, and where it all wanders off to.</p>
      <span class="welcome-note">Your numbers. At your pace.</span>
    </section>
    <section class="login-panel" aria-labelledby="login-heading">
      <p class="eyebrow">WELCOME BACK</p>
      <h2 id="login-heading">Settle in.</h2>
      <p class="muted">Your notebook is right where you left it.</p>
      <form onsubmit={login}>
        <label>
          Username<input name="username" autocomplete="username" required maxlength="80" />
        </label>
        <label>
          Password<input
            name="password"
            type="password"
            autocomplete="current-password"
            required
            maxlength="128"
          />
        </label>
        {#if error}<p class="error" role="alert">{error}</p>{/if}
        <button class="button primary full" disabled={busy}>
          {busy ? 'Signing in…' : 'Log in'}
        </button>
      </form>
      <p class="setup-note">
        First visit? Create your local user with the command in the project README.
      </p>
    </section>
  </main>
{:else}
  <div class="app-shell">
    <header class="topbar">
      <a href={resolve('/')} class="wordmark" aria-label="coinz home">coinz<span>.</span></a>
      <span class="tagline">A little clarity for your everyday.</span>
      <details
        class="user-menu"
        bind:this={userMenu}
        use:escapeToCancel={() => {
          if (userMenu) {
            userMenu.open = false;
            userMenu.querySelector('summary')?.focus();
          }
        }}
      >
        <summary>{user.username} <span aria-hidden="true">▾</span></summary>
        <div class="user-dropdown">
          <label>
            Appearance<select
              value={appearance}
              onchange={(event) => chooseAppearance(event.currentTarget.value)}
            >
              <option value="system">System</option><option value="light">Light</option><option
                value="dark"
              >
                Dark
              </option>
            </select>
          </label>
          <button
            class="button quiet"
            onclick={() => {
              if (userMenu) userMenu.open = false;
              settingsDialog.showModal();
            }}
          >
            Settings
          </button>
          <button class="button quiet" onclick={logout} disabled={busy}>Log out</button>
        </div>
      </details>
    </header>
    <main class="notebook" aria-busy={loading}>
      <div class="page-heading">
        <div>
          <p class="eyebrow">YOUR MONEY NOTEBOOK</p>
          <h1>Where did it go?</h1>
          <p class="muted">A closer look at the entries you've recorded.</p>
        </div>
        <div class="delete-actions">
          <button class="button" onclick={() => (importOpen = true)} disabled={busy}>
            Import CSV
          </button>
          <button class="button primary" onclick={() => openEntry()} disabled={busy}>
            + Add transaction
          </button>
        </div>
      </div>
      {#if error}<div class="error banner" role="alert">
          {error}<button class="button quiet" onclick={loadData} disabled={loading}>Retry</button>
        </div>{/if}
      {#if notice}<p class="notice" role="status">{notice}</p>{/if}
      <section class="period-controls" aria-label="Transaction filters">
        <div class="month-picker">
          <button
            type="button"
            class="button"
            aria-label="Previous month"
            disabled={!adjacentMonth(month, -1)}
            onclick={() => moveMonth(-1)}
          >
            ‹
          </button>
          <label>
            Showing entries for<input
              aria-label="Month"
              type="month"
              min="0001-01"
              max="9999-12"
              bind:value={month}
              onchange={changeFilter}
              required
            />
          </label>
          <button
            type="button"
            class="button"
            aria-label="Next month"
            disabled={!adjacentMonth(month, 1)}
            onclick={() => moveMonth(1)}
          >
            ›
          </button>
        </div>
        <label>
          Source<select bind:value={sourceFilter} onchange={changeFilter}>
            <option value="">All sources</option>{#each sources as source (source.id)}<option
                value={source.id}
              >
                {source.name}
              </option>{/each}
          </select>
        </label>
        <span class="scope-note">{loading ? 'Updating…' : 'Recorded transactions · EUR'}</span>
      </section>
      {#if summary && !loading && !error}
        <section class="month-summary" aria-label="Monthly summary">
          <div class="spending-total">
            <p class="eyebrow">RECORDED SPENDING</p>
            <p class="hero-amount" data-testid="spending">{money(summary.spending_cents)}</p>
            <p class="muted">
              {monthLabel(summary.month)} · {summary.transaction_count}
              {summary.transaction_count === 1 ? 'entry' : 'entries'}
            </p>
          </div>
          <div class="secondary-total">
            <p>Recorded income</p>
            <strong data-testid="income">{money(summary.income_cents)}</strong><span class="muted">
              Same period and filters
            </span>
          </div>
          <div class="secondary-total">
            <p>Income minus spending</p>
            <strong data-testid="net">{money(summary.net_cents, true)}</strong><span class="muted">
              From recorded entries
            </span>
          </div>
        </section>
      {/if}
      <p class="data-note">
        These figures reflect your records, not a live bank connection. Missing entries can change
        the picture.
      </p>
      <div class="ledger-workspace">
        <section class="ledger" aria-labelledby="ledger-heading">
          <div class="section-heading">
            <h2 id="ledger-heading" tabindex="-1">The entries</h2>
            <span class="muted">{month ? monthLabel(month) : 'Choose a month'}</span>
          </div>
          {#if loading}<div class="empty-state"><p>Updating your notebook…</p></div>
          {:else if error}<div class="empty-state">
              <p>The entries couldn't be loaded.</p>
              <p class="muted">Use Retry above to get the latest recorded figures.</p>
            </div>
          {:else if transactions.length === 0}<div class="empty-state">
              <h3>Nothing recorded for this period.</h3>
              <p>Your money is maintaining a low profile.</p>
              <button class="button" onclick={() => openEntry()}>Add a transaction</button>
            </div>
          {:else}
            {#key `${user.id}:${month}:${sourceFilter}`}
              <EntriesList
                always={alwaysSelection}
                entries={transactions}
                {sources}
                {month}
                onchanged={async (message) => {
                  notice = message;
                  await loadData();
                }}
              />
            {/key}
          {/if}
        </section>
      </div>
      <footer>coinz · Small steps. Fewer financial mysteries.</footer>
    </main>
  </div>
{/if}

{#if user && importOpen}
  <ImportDialog
    {sources}
    initialSource={sourceFilter}
    onclose={() => {
      importOpen = false;
      void loadData();
    }}
    onimported={async (result, source) => {
      notice = `Import complete: ${result.added} added, ${result.matched} matched, ${result.known} already processed, ${result.excluded} left for later.`;
      sourceFilter = source;
      if (result.latest_month) month = result.latest_month;
      await loadData();
    }}
  />
{/if}

<dialog
  bind:this={sourceDialog}
  oncancel={(event) => {
    if (busy) event.preventDefault();
  }}
  aria-labelledby="source-title"
>
  <form onsubmit={saveSource}>
    <div class="dialog-heading">
      <div>
        <p class="eyebrow">WHERE IT CAME FROM</p>
        <h2 id="source-title">Add a source</h2>
      </div>
      <button
        type="button"
        class="button quiet"
        onclick={() => sourceDialog.close()}
        disabled={busy}
        aria-label="Close add source"
      >
        Close
      </button>
    </div>
    <label>Source name<input name="name" placeholder="e.g. N26" required maxlength="80" /></label>
    {#if formError}<p class="error" role="alert">{formError}</p>{/if}
    <button class="button primary full" disabled={busy}>
      {busy ? 'Saving…' : 'Create source'}
    </button>
  </form>
</dialog>

<dialog
  bind:this={transactionDialog}
  oncancel={(event) => {
    if (busy) event.preventDefault();
  }}
  aria-labelledby="transaction-title"
>
  <form onsubmit={saveEntry}>
    <div class="dialog-heading">
      <div>
        <p class="eyebrow">IN THE NOTEBOOK</p>
        <h2 id="transaction-title">Add a transaction</h2>
      </div>
      <button
        type="button"
        class="button quiet"
        onclick={() => transactionDialog.close()}
        disabled={busy}
        aria-label="Close transaction"
      >
        Close
      </button>
    </div>
    <div class="kind-selector" aria-label="Transaction type">
      <label class:chosen={entryKind === 'expense'}>
        <input type="radio" name="kind" value="expense" bind:group={entryKind} />Expense
      </label><label class:chosen={entryKind === 'income'}>
        <input type="radio" name="kind" value="income" bind:group={entryKind} />Income
      </label>
    </div>
    <label>
      Amount (€)<input bind:value={entryAmount} inputmode="decimal" placeholder="0.00" required />
    </label>
    <label>
      Description<input
        bind:value={entryDescription}
        placeholder="What was it for?"
        required
        maxlength="200"
      />
    </label>
    <div class="form-row">
      <label>
        Date<input type="date" bind:value={entryDate} min="0001-01-01" max="9999-12-31" required />
      </label><label>
        Source (optional)<select bind:value={entrySource}>
          <option value="">Manual · no source</option>
          {#each sources as source (source.id)}<option value={source.id}>
              {source.name}
            </option>{/each}
        </select>
      </label>
    </div>
    <div class="source-help">
      <p class="form-help">
        A source records where an entry came from, such as N26 or Erste. You can leave it blank.
      </p>
      <button type="button" class="button quiet" onclick={openSource} disabled={busy}>
        + Add source
      </button>
    </div>
    {#if formError}<p class="error" role="alert">{formError}</p>{/if}
    <button class="button primary full" disabled={busy}>
      {busy ? 'Saving…' : 'Record transaction'}
    </button>
  </form>
</dialog>

<dialog bind:this={settingsDialog} class="settings-panel" aria-labelledby="settings-title">
  <div class="dialog-heading">
    <h2 id="settings-title">Settings</h2>
    <button
      type="button"
      class="button quiet"
      onclick={() => settingsDialog.close()}
      aria-label="Close settings"
    >
      Close
    </button>
  </div>
  <h3>Appearance</h3>
  <label class="setting-toggle">
    <input
      type="checkbox"
      checked={alwaysSelection}
      onchange={(event) => saveSelectionPreference(event.currentTarget.checked)}
    />Always show selection controls
  </label>
  <p class="muted">Otherwise controls appear on hover or focus. Touch screens always show them.</p>
</dialog>

<style>
  .user-menu {
    position: relative;
    display: block;
    margin-left: auto;
  }
  .user-menu summary {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 44px;
    cursor: pointer;
    overflow-wrap: anywhere;
  }
  .user-dropdown {
    position: absolute;
    right: 0;
    top: 100%;
    z-index: 10;
    width: min(260px, calc(100vw - 2.5rem));
    padding: 1rem;
    background: var(--paper);
    border: 1px solid var(--line);
    border-radius: 0.75rem;
    box-shadow: 0 12px 35px #0002;
    display: grid;
    gap: 0.5rem;
  }

  .settings-panel {
    margin: 0 0 0 auto;
    height: 100svh;
    max-height: 100svh;
    width: min(420px, 100%);
    border-radius: 0;
  }
  .settings-panel h3 {
    margin: 2rem 0 1rem;
  }
  .setting-toggle {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  .setting-toggle input {
    width: 19px;
    min-height: 19px;
    accent-color: var(--accent);
  }
</style>
