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
  import { amountInput, dateLabel, localDate, money, monthLabel } from '#lib/money.ts';
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
  let editing = $state<Transaction | null>(null);
  let deleting = $state(false);
  let entrySource = $state('');
  let entryDate = $state(localDate());
  let entryKind = $state('expense');
  let entryAmount = $state('');
  let entryDescription = $state('');
  let loadVersion = 0;

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

  async function openEntry(transaction: Transaction | null = null) {
    editing = transaction;
    entrySource = transaction ? (transaction.source_id ?? '') : sourceFilter;
    entryDate = transaction?.date ?? localDate();
    entryKind = transaction && BigInt(transaction.amount_cents) > 0n ? 'income' : 'expense';
    entryAmount = transaction ? amountInput(transaction.amount_cents) : '';
    entryDescription = transaction?.description ?? '';
    formError = '';
    deleting = false;
    await tick();
    transactionDialog.showModal();
  }

  async function saveEntry(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    formError = '';
    try {
      await api(
        editing ? `/transactions/${editing.id}` : '/transactions',
        editing ? 'PUT' : 'POST',
        {
          source_id: entrySource || null,
          date: entryDate,
          kind: entryKind,
          amount: entryAmount,
          description: entryDescription,
        },
      );
      transactionDialog.close();
      notice = editing ? 'Transaction updated.' : 'Transaction recorded.';
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

  async function deleteEntry() {
    if (!editing) return;
    busy = true;
    formError = '';
    try {
      await api(`/transactions/${editing.id}`, 'DELETE');
      transactionDialog.close();
      notice = 'Transaction deleted.';
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
      <div class="user-menu">
        <span>{user.username}</span><button class="button quiet" onclick={logout} disabled={busy}>
          Log out
        </button>
      </div>
    </header>
    <main class="notebook" aria-busy={loading}>
      <div class="page-heading">
        <div>
          <p class="eyebrow">YOUR MONEY NOTEBOOK</p>
          <h1>Where did it go?</h1>
          <p class="muted">A closer look at the entries you've recorded.</p>
        </div>
        <button class="button primary" onclick={() => openEntry()} disabled={busy}>
          + Add transaction
        </button>
      </div>
      {#if error}<div class="error banner" role="alert">
          {error}<button class="button quiet" onclick={loadData} disabled={loading}>Retry</button>
        </div>{/if}
      {#if notice}<p class="notice" role="status">{notice}</p>{/if}
      <section class="period-controls" aria-label="Transaction filters">
        <label>
          Month<input
            aria-label="Month"
            type="month"
            min="0001-01"
            max="9999-12"
            bind:value={month}
            onchange={changeFilter}
            required
          />
        </label>
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
            <h2 id="ledger-heading">The entries</h2>
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
            <table>
              <caption class="sr-only">
                Recorded transactions for {monthLabel(month)}{sourceFilter
                  ? ' in the selected source'
                  : ' across all sources'}
              </caption><thead>
                <tr>
                  <th>Date</th><th>Description</th><th>Source</th><th class="number">
                    Amount
                  </th><th><span class="sr-only">Actions</span></th>
                </tr>
              </thead><tbody>
                {#each transactions as transaction (transaction.id)}<tr>
                    <td class="entry-date">{dateLabel(transaction.date)}</td><td
                      class="entry-description"
                    >
                      {transaction.description}
                    </td><td class="entry-source">{transaction.source_name ?? 'Manual'}</td><td
                      class:income={BigInt(transaction.amount_cents) > 0n}
                      class="number entry-amount"
                    >
                      {money(transaction.amount_cents, true)}
                    </td><td class="entry-action">
                      <button
                        class="button icon-button"
                        aria-label={`Edit ${transaction.description}`}
                        onclick={() => openEntry(transaction)}
                      >
                        Edit
                      </button>
                    </td>
                  </tr>{/each}
              </tbody>
            </table>
          {/if}
        </section>
      </div>
      <footer>coinz · Small steps. Fewer financial mysteries.</footer>
    </main>
  </div>
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
        <h2 id="transaction-title">{editing ? 'Edit transaction' : 'Add a transaction'}</h2>
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
    <button class="button primary full" disabled={busy || deleting}>
      {busy ? 'Saving…' : editing ? 'Save changes' : 'Record transaction'}
    </button>
    {#if editing}<div class="delete-area">
        {#if deleting}<p>Delete this transaction? Its amount will be removed from your records.</p>
          <div class="delete-actions">
            <button type="button" class="button" onclick={() => (deleting = false)} disabled={busy}>
              Keep it
            </button><button
              type="button"
              class="button destructive"
              onclick={deleteEntry}
              disabled={busy}
            >
              Delete transaction
            </button>
          </div>{:else}<button
            type="button"
            class="button quiet danger-text"
            onclick={() => (deleting = true)}
            disabled={busy}
          >
            Delete transaction
          </button>{/if}
      </div>{/if}
  </form>
</dialog>
